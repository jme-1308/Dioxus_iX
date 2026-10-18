//! Stable arbitrary-content Header presentation. Native code reports DOM facts;
//! Rust owns presence, open state, cancellation, keyboard choices and focus intent.
use crate::overlay_registry::{CloseBehavior, OverlayKind};
use crate::overlay_runtime::{self, Callbacks, Change, KeyDecision, KeyFacts, NodeHandle, Reason};
use dioxus::prelude::*;
use serde::Deserialize;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[path = "header_layout_dom.rs"]
mod native;
static NEXT_OWNER: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct CapturedFocus {
    token: String,
    group: u8,
    epoch: u64,
}

#[derive(Deserialize)]
struct FocusCheck {
    version: u64,
    token: String,
    target_connected: bool,
    target_owned: bool,
    target_visible: bool,
    target_group: Option<u8>,
}

#[derive(Deserialize)]
struct Report {
    owner: String,
    generation: u64,
    sequence: u64,
    connected: bool,
    kind: String,
    #[serde(default)]
    key: String,
    #[serde(default)]
    shift: bool,
    #[serde(default)]
    alt: bool,
    #[serde(default)]
    default_prevented: bool,
    presence: [bool; 4],
    #[serde(default)]
    focus_count: usize,
    #[serde(default)]
    focused_index: Option<usize>,
    #[serde(default)]
    more: bool,
    #[serde(default)]
    inside: bool,
    #[serde(default)]
    background: bool,
    #[serde(default)]
    hosts: Vec<String>,
    #[serde(default)]
    triggers: Vec<String>,
    #[serde(default)]
    focused_control: Option<CapturedFocus>,
    #[serde(default)]
    focus_epoch: u64,
    #[serde(default)]
    active_default: bool,
    #[serde(default)]
    focus_check: Option<FocusCheck>,
}

struct Model {
    owner: String,
    alive: bool,
    mounted: bool,
    generation: u64,
    sequence: u64,
    small: bool,
    top_layer: bool,
    open: bool,
    presence: [bool; 4],
    keyboard: bool,
    focus_version: u64,
    focus: String,
    focused_control: Option<CapturedFocus>,
    preserve: Option<CapturedFocus>,
    stop: bool,
}
impl Model {
    fn new(small: bool, top_layer: bool) -> Self {
        Self {
            owner: format!(
                "ix-header-layout-{}",
                NEXT_OWNER.fetch_add(1, Ordering::Relaxed)
            ),
            alive: true,
            mounted: false,
            generation: 1,
            sequence: 0,
            small,
            top_layer,
            open: false,
            presence: [false; 4],
            keyboard: false,
            focus_version: 0,
            focus: "none".into(),
            focused_control: None,
            preserve: None,
            stop: false,
        }
    }
    fn more_visible(&self) -> bool {
        self.presence[2] || (self.small && (self.presence[0] || self.presence[1]))
    }
    fn focus(&mut self, target: String) {
        self.focus = target;
        self.focus_version = self.focus_version.wrapping_add(1);
        self.preserve = None;
    }
    fn group_presented(&self, group: u8) -> bool {
        match group {
            0 | 1 => !self.small || self.open,
            2 => self.open,
            3 => true,
            4 => self.more_visible(),
            _ => false,
        }
    }
    fn preserve_focus(&mut self, capture: CapturedFocus) {
        self.focus("preserve".into());
        self.preserve = Some(capture);
    }
    fn apply_focus(&mut self, restore: bool) {
        let retained = (!restore)
            .then(|| self.preserve.clone())
            .flatten()
            .filter(|capture| self.group_presented(capture.group));
        if let Some(capture) = retained {
            self.preserve_focus(capture);
        } else {
            self.focus(if restore { "more" } else { "none" }.into());
        }
    }
    fn authorize_preserve(&mut self, report: &Report) -> bool {
        if !self.accepts(report) {
            return false;
        }
        self.sequence = report.sequence;
        let Some(check) = &report.focus_check else {
            return false;
        };
        let Some(capture) = &self.preserve else {
            return false;
        };
        let same_active = report
            .focused_control
            .as_ref()
            .is_some_and(|active| active.token == capture.token && active.epoch == capture.epoch);
        if self.focus != "preserve"
            || check.version != self.focus_version
            || check.token != capture.token
            || report.focus_epoch != capture.epoch
            || !check.target_connected
            || !check.target_owned
            || !check.target_visible
            || check.target_group != Some(capture.group)
            || !self.group_presented(capture.group)
            || !(same_active || report.active_default)
        {
            return false;
        }
        self.preserve = None;
        true
    }
    fn sync_mode(&mut self, small: bool, top_layer: bool) -> bool {
        if self.small == small && self.top_layer == top_layer {
            return false;
        }
        // Source retains the same overflow Dropdown and its opening-time
        // child cache when slots move across a breakpoint. Only a top-layer
        // configuration change keeps this adapter's prior forced-close policy.
        let layer_changed = self.top_layer != top_layer;
        let closed = self.open && layer_changed;
        self.small = small;
        self.top_layer = top_layer;
        if layer_changed {
            self.open = false;
        }
        self.generation = self.generation.wrapping_add(1);
        self.sequence = 0;
        self.stop = false;
        let capture = (!layer_changed && self.alive && self.mounted)
            .then(|| self.focused_control.take())
            .flatten()
            .filter(|capture| self.group_presented(capture.group));
        if let Some(capture) = capture {
            self.preserve_focus(capture);
        } else {
            self.focused_control = None;
            self.focus("none".into());
        }
        closed
    }
    fn accepts(&self, report: &Report) -> bool {
        self.alive
            && self.mounted
            && report.connected
            && report.owner == self.owner
            && report.generation == self.generation
            && report.sequence > self.sequence
    }
    fn invalidate(&mut self) {
        self.alive = false;
        self.mounted = false;
        self.open = false;
        self.generation = self.generation.wrapping_add(1);
        self.focused_control = None;
        self.preserve = None;
    }
}

#[derive(Default)]
struct Decision {
    prevent: bool,
    stop: bool,
    request: Option<bool>,
    focus: Option<String>,
}
fn decide(model: &mut Model, report: &Report, popover_ancestor: bool) -> Decision {
    let mut decision = Decision::default();
    match report.kind.as_str() {
        "pointer" => model.keyboard = false,
        "key" if !report.default_prevented => {
            model.keyboard = true;
            // Inside a Popover, the source Header Dropdown's local query has
            // no eligible controls. More Tab instead belongs to the shared
            // trigger callback below, including its no-target/default case.
            if report.key == "Tab" && popover_ancestor {
                return decision;
            }
            if report.more
                && matches!(
                    report.key.as_str(),
                    "Home" | "End" | "ArrowDown" | "ArrowUp" | "Enter" | " "
                )
            {
                if !model.open && model.more_visible() {
                    decision.prevent = !report.alt;
                    decision.request = Some(true);
                    if !report.alt {
                        decision.focus = Some(
                            if matches!(report.key.as_str(), "End" | "ArrowUp") {
                                "last"
                            } else {
                                "first"
                            }
                            .into(),
                        );
                    }
                }
            } else if model.open && (report.inside || report.more) && report.key == "Escape" {
                decision.stop = true;
                decision.request = Some(false);
                decision.focus = Some("more".into());
            } else if model.open && report.inside && report.key == "Tab" && report.focus_count > 0 {
                let boundary = report.focused_index.is_none()
                    || if report.shift {
                        report.focused_index == Some(0)
                    } else {
                        report.focused_index == Some(report.focus_count - 1)
                    };
                if boundary {
                    decision.prevent = true;
                    decision.stop = true;
                    decision.focus = Some(format!(
                        "index:{}",
                        if report.shift {
                            report.focus_count - 1
                        } else {
                            0
                        }
                    ));
                }
            }
        }
        "click" if !report.default_prevented => {
            if report.more && model.more_visible() {
                decision.request = Some(!model.open);
            } else if model.open && report.inside {
                if report.background {
                    decision.prevent = true;
                } else {
                    decision.request = Some(false);
                    if model.keyboard {
                        decision.focus = Some("more".into());
                    }
                }
            }
        }
        _ => {}
    }
    decision
}

fn releases_more_tab(model: &Model, facts: &KeyFacts) -> bool {
    model.alive
        && model.mounted
        && model.open
        && !facts.default_prevented
        && facts.key == "Tab"
        && facts.on_trigger
        && facts.popover_ancestor
}

fn header_tab_key(
    model: &Rc<RefCell<Model>>,
    slot: &Rc<RefCell<Option<NodeHandle>>>,
    facts: &KeyFacts,
) -> KeyDecision {
    if !releases_more_tab(&model.borrow(), facts) {
        return KeyDecision::default();
    }
    let Some(node) = slot.borrow().clone().filter(NodeHandle::is_registered) else {
        return KeyDecision::default();
    };
    let Some(runtime) = node.runtime() else {
        return KeyDecision::default();
    };
    // A NodeHandle holds only a weak runtime reference. Resolve it after the
    // callback borrow, and keep release intent even if the close is vetoed.
    runtime.dismiss_root(node.id(), Reason::Keyboard, false);
    KeyDecision {
        release_focus: true,
        ..KeyDecision::default()
    }
}

#[derive(Clone)]
struct Controller {
    model: Rc<RefCell<Model>>,
    revision: Signal<u64>,
    node: NodeHandle,
}
impl Controller {
    fn notify(&self) {
        let mut revision = self.revision;
        let next = revision.peek().wrapping_add(1);
        revision.set(next);
    }
    fn observation(&self, event: FormEvent) {
        event.stop_propagation();
        let Ok(report) = serde_json::from_str::<Report>(&event.value()) else {
            return;
        };
        if report.kind == "preserve-focus" {
            if self.model.borrow_mut().authorize_preserve(&report) {
                event.prevent_default();
            }
            return;
        }
        // Domain presence reads can enter this owner; do not hold Model's
        // mutable borrow while computing source-aware ancestry.
        let popover_ancestor =
            report.kind == "key" && report.key == "Tab" && self.node.has_popover_ancestor();
        let (decision, generation, previous) = {
            let mut model = self.model.borrow_mut();
            if !model.accepts(&report) {
                return;
            }
            model.sequence = report.sequence;
            model.presence = report.presence;
            model.focused_control = report.focused_control.clone();
            let child_owned = self.node.runtime().is_some_and(|runtime| {
                if report.kind == "key" {
                    runtime.key_child_path(
                        &model.owner,
                        &report.hosts,
                        &report.triggers,
                        &report.key,
                    )
                } else {
                    runtime.child_path(&model.owner, &report.hosts, &report.triggers)
                }
            });
            let decision = if child_owned && matches!(report.kind.as_str(), "key" | "click") {
                Decision {
                    prevent: report.kind == "click"
                        && report.triggers.iter().any(|id| id != &model.owner),
                    ..Decision::default()
                }
            } else {
                decide(&mut model, &report, popover_ancestor)
            };
            model.stop = decision.stop;
            (decision, model.generation, model.open)
        };
        if decision.prevent {
            event.prevent_default();
        }
        let accepted = decision
            .request
            .filter(|next| *next != previous)
            .is_none_or(|next| {
                self.node.request(
                    next,
                    if report.kind == "key" {
                        Reason::Keyboard
                    } else {
                        Reason::Trigger
                    },
                    false,
                )
            });
        {
            let mut model = self.model.borrow_mut();
            if !model.alive || model.generation != generation {
                return;
            }
            if accepted {
                if let Some(focus) = decision.focus {
                    model.focus(focus);
                } else if decision.request.is_some() {
                    model.focus("none".into());
                }
            }
        }
        self.notify();
    }
    fn stop_query(&self, event: FormEvent) {
        event.stop_propagation();
        let Ok(report) = serde_json::from_str::<Report>(&event.value()) else {
            return;
        };
        let model = self.model.borrow();
        if model.alive
            && model.mounted
            && report.connected
            && report.owner == model.owner
            && report.generation == model.generation
            && report.sequence == model.sequence
            && model.stop
        {
            event.prevent_default();
        }
    }
}

#[component]
pub(crate) fn HeaderLayout(
    #[props(default)] default_content: Option<Element>,
    #[props(default)] secondary: Option<Element>,
    #[props(default)] overflow: Option<Element>,
    #[props(default)] avatar: Option<Element>,
    #[props(default)] small: bool,
    #[props(default)] enable_top_layer: bool,
    #[props(default)] discovery_ready: bool,
    more_label: String,
    #[props(default)] onrequest: EventHandler<crate::HeaderOverflowEvent>,
    #[props(default)] onchange: EventHandler<bool>,
) -> Element {
    let model = use_hook(|| Rc::new(RefCell::new(Model::new(small, enable_top_layer))));
    let revision = use_signal(|| 0_u64);
    let runtime = overlay_runtime::use_overlay_runtime();
    let node_slot = use_hook(|| Rc::new(RefCell::new(None::<NodeHandle>)));
    let node = {
        let read = model.clone();
        let before = model.clone();
        let apply = model.clone();
        let key_model = model.clone();
        let key_slot = node_slot.clone();
        let owner = model.borrow().owner.clone();
        runtime.register(
            owner,
            1,
            OverlayKind::Header,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(move || {
                    let model = read.borrow();
                    model.alive.then_some(model.open)
                }),
                before: Rc::new(move |change| {
                    let (previous, generation, visible) = {
                        let model = before.borrow();
                        (
                            model.open,
                            model.generation,
                            model.alive && (!change.open || model.more_visible()),
                        )
                    };
                    if !visible {
                        return false;
                    }
                    let request = crate::HeaderOverflowEvent::new(change.open, previous);
                    onrequest.call(request.clone());
                    let model = before.borrow();
                    !request.default_prevented() && model.alive && model.generation == generation
                }),
                apply: Rc::new(move |change: Change| {
                    {
                        let mut model = apply.borrow_mut();
                        if !model.alive {
                            return;
                        }
                        model.open = change.open;
                        model.apply_focus(change.restore_focus);
                    }
                    let mut revision = revision;
                    if let Ok(mut value) = revision.try_write() {
                        *value = value.wrapping_add(1);
                    }
                    onchange.call(change.open);
                }),
                key: Some(Rc::new(move |facts| {
                    header_tab_key(&key_model, &key_slot, facts)
                })),
            },
        )
    };
    *node_slot.borrow_mut() = Some(node.clone());

    let bridge = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    // A slot projection preserves presence and the admitted child cache. A
    // top-layer change still closes through bounded configuration reconciliation,
    // rather than a vetoable user request. Both invalidate private focus tasks.
    if model.borrow_mut().sync_mode(small, enable_top_layer) {
        node.sync_open(false);
        let changed_node = node.clone();
        let current = model.clone();
        let generation = current.borrow().generation;
        dioxus::core::queue_effect(move || {
            let valid = {
                let model = current.borrow();
                model.alive && model.generation == generation
            };
            if valid {
                changed_node.close_children(Reason::Configuration);
                onchange.call(false);
            }
        });
    }
    let cleanup_model = model.clone();
    let cleanup_bridge = bridge.clone();
    let cleanup_node = node.clone();
    let cleanup_slot = node_slot.clone();
    use_drop(move || {
        cleanup_model.borrow_mut().invalidate();
        cleanup_node.close_children(Reason::Drop);
        cleanup_node.unregister();
        cleanup_slot.borrow_mut().take();
        if let Some(bridge) = cleanup_bridge.borrow_mut().take() {
            let _ = bridge.send("finish");
        }
    });
    let controller = Controller {
        model: model.clone(),
        revision,
        node,
    };
    let stop_controller = controller.clone();
    let mount_model = model.clone();
    let _ = revision();
    let state = model.borrow();
    let owner = state.owner.clone();
    let generation = state.generation.to_string();
    let open = state.open;
    let visible = state.more_visible();
    let presence = state.presence;
    let focus_version = state.focus_version.to_string();
    let focus = state.focus.clone();
    let capture = state.preserve.as_ref().map(|capture| capture.token.clone());
    let active = if small { "small" } else { "large" };
    let more_id = format!("{owner}-more");
    let small_id = format!("{owner}-small");
    let large_id = format!("{owner}-large");
    drop(state);
    let runtime_owner = runtime.owner_id();
    rsx! {
        div { id:owner.clone(), class:"ix-header-layout",
            "data-ix-overlay-owner":runtime_owner,
            "data-ix-overlay-root":owner.clone(),"data-ix-overlay-generation":"1",
            "data-ix-overlay-open":open.to_string(),"data-ix-overlay-ready":discovery_ready.to_string(),
            "data-ix-overlay-mode":"active-descendant",
            "data-small":small.to_string(),"data-open":open.to_string(),"data-top-layer":enable_top_layer.to_string(),
            "data-generation":generation,"data-focus-version":focus_version,"data-focus-target":focus,"data-focus-capture":capture,
            onmounted:move |_| {
                let mut model = mount_model.borrow_mut();
                if !model.alive || model.mounted { return; }
                model.mounted = true;
                let owner = serde_json::to_string(&model.owner).expect("Header owner serialization");
                drop(model);
                *bridge.borrow_mut() = Some(dioxus::document::eval(&native::BRIDGE.replace("__OWNER__",&owner)));
            },
            overlay_runtime::OverlayTransport {runtime}
            input {r#type:"hidden","data-header-layout-observer":owner.clone(),
                oninput:move |event| controller.observation(event),
                onchange:move |event| stop_controller.stop_query(event)
            }
            dialog {id:small_id.clone(),class:"ix-header-popup-host ix-header-popup-small",
                "data-ix-overlay-host":small.then_some(owner.clone()),
                "data-header-popup-host":"small","data-active":small.to_string(),"data-open":(small&&open).to_string(),
                role:if small {Some("presentation")} else {None},
                popover:if small&&enable_top_layer {Some("manual")} else {None},
                div {class:"ix-header-slot-group ix-header-slot-secondary","data-header-group":"secondary","data-populated":presence[0].to_string(),{secondary}}
                div {class:"ix-header-slot-group ix-header-slot-default","data-header-group":"default","data-populated":presence[1].to_string(),"data-separator":(small&&presence[0]&&presence[1]).to_string(),{default_content}}
                dialog {id:large_id.clone(),class:"ix-header-popup-host ix-header-popup-large",
                    "data-ix-overlay-host":(!small).then_some(owner.clone()),
                    "data-header-popup-host":"large","data-active":(!small).to_string(),"data-open":(!small&&open).to_string(),
                    role:if !small {Some("presentation")} else {None},
                    popover:if !small&&enable_top_layer {Some("manual")} else {None},
                    div {class:"ix-header-slot-group ix-header-slot-overflow","data-header-group":"overflow","data-populated":presence[2].to_string(),"data-separator":(small&&presence[2]&&(presence[0]||presence[1])).to_string(),{overflow}}
                }
            }
            button {id:more_id,r#type:"button",class:"ix-app-icon-button ix-header-more-button",
                "data-ix-overlay-trigger":owner.clone(),
                "data-header-more-button":owner.clone(),"data-testid":"show-more",hidden:!visible,
                "aria-label":more_label,"aria-expanded":open.to_string(),"aria-controls":if active=="small" {small_id} else {large_id},
                crate::Icon {name:"more-menu".to_string(),size:24}
            }
            div {class:"ix-header-avatar-slot","data-header-group":"avatar",{avatar}}
        }
    }
}

pub(crate) const STYLES: &str = r"
.ix-header-layout{display:grid;grid-template-columns:minmax(0,1fr) max-content max-content max-content;align-items:center;flex:1 1 auto;min-width:min-content;height:100%;gap:.25rem}
.ix-header-popup-host{color:inherit;font:inherit}
.ix-header-popup-small{display:contents}
.ix-header-slot-group,.ix-header-avatar-slot{display:flex;align-items:center;gap:.25rem;min-width:0}
.ix-header-slot-secondary{grid-column:1;overflow:hidden}
.ix-header-slot-default{grid-column:2;margin-inline-start:auto}
.ix-header-avatar-slot{grid-column:4;flex:0 0 auto}
.ix-header-more-button{grid-column:3;flex:0 0 auto;margin-inline-start:auto}
.ix-header-more-button[hidden]{display:none}
.ix-header-popup-host[data-active=true]{display:none;position:fixed;inset:auto;margin:0;box-sizing:border-box;min-width:12rem;max-width:calc(100vw - 1rem);max-height:calc(100dvh - 4rem);overflow:auto;padding:var(--ix-dropdown--padding,.25rem 0);border:0;border-radius:var(--ix-dropdown--border-radius,.25rem);background:var(--ix-dropdown--background,var(--si-sys-color-background-3,var(--ix-color-surface-1)));color:var(--ix-color-text);box-shadow:var(--ix-dropdown--box-shadow,var(--si-sys-color-effects-shadow-4,0 4px 16px #0004));z-index:1060;height:auto;flex:none}
.ix-header-popup-host[data-active=true][data-open=true]{display:flex;flex-direction:column;align-items:stretch;gap:0}
.ix-header-layout[data-small=true]{display:flex;justify-content:flex-end}
.ix-header-layout[data-small=true] .ix-header-popup-large{display:contents}
.ix-header-popup-host[data-active=true] .ix-header-slot-group{flex:none;overflow:visible;flex-direction:column;align-items:stretch;margin:0}
.ix-header-popup-host[data-active=true] .ix-header-slot-group[data-populated=true]{padding:var(--ix-application-header-slot-content-active--padding,.5rem 1.5rem)}
.ix-header-layout[data-small=true] [data-separator=true]{border-top:var(--ix-application-header-dropdown-item--border-width,1px) solid var(--ix-color-border)}
";

#[cfg(test)]
mod tests {
    use super::*;
    fn report(model: &Model, kind: &str) -> Report {
        Report {
            owner: model.owner.clone(),
            generation: model.generation,
            sequence: model.sequence + 1,
            connected: true,
            kind: kind.into(),
            key: String::new(),
            shift: false,
            alt: false,
            default_prevented: false,
            presence: [true; 4],
            focus_count: 6,
            focused_index: Some(0),
            more: false,
            inside: true,
            background: false,
            hosts: vec![],
            triggers: vec![],
            focused_control: None,
            focus_epoch: 0,
            active_default: false,
            focus_check: None,
        }
    }
    #[test]
    fn decision_preserves_opening_defaults_veto_intent_and_boundary_keys() {
        let mut model = Model::new(true, false);
        model.mounted = true;
        model.presence = [true; 4];
        for key in ["Home", "End", "ArrowUp", "ArrowDown", "Enter", " "] {
            let mut fact = report(&model, "key");
            fact.more = true;
            fact.key = key.into();
            let decision = decide(&mut model, &fact, false);
            assert!(decision.prevent);
            assert_eq!(decision.request, Some(true));
            assert!(!decision.stop);
        }
        model.open = true;
        let mut trigger = report(&model, "key");
        trigger.more = true;
        trigger.key = "Enter".into();
        let decision = decide(&mut model, &trigger, false);
        assert!(!decision.prevent);
        assert_eq!(
            decision.request, None,
            "open trigger retains native click activation"
        );
        model.open = false;
        trigger.alt = true;
        let decision = decide(&mut model, &trigger, false);
        assert!(!decision.prevent);
        assert_eq!(decision.request, Some(true));
        assert_eq!(decision.focus, None);
        model.open = true;
        let mut fact = report(&model, "key");
        fact.key = "Escape".into();
        let decision = decide(&mut model, &fact, false);
        assert!(!decision.prevent && decision.stop);
        assert_eq!(decision.request, Some(false));
        assert_eq!(decision.focus.as_deref(), Some("more"));
        fact.key = "Tab".into();
        fact.focused_index = Some(1);
        assert!(!decide(&mut model, &fact, false).prevent);
        fact.focused_index = Some(5);
        assert_eq!(
            decide(&mut model, &fact, false).focus.as_deref(),
            Some("index:0"),
        );
        fact.shift = true;
        fact.focused_index = Some(0);
        assert_eq!(
            decide(&mut model, &fact, false).focus.as_deref(),
            Some("index:5"),
        );
    }
    #[test]
    fn popover_content_tab_does_not_use_standalone_header_wrap_or_more_focus() {
        let mut model = Model::new(true, true);
        model.open = true;
        model.presence = [true; 4];
        for more in [false, true] {
            for shift in [false, true] {
                for index in [None, Some(0), Some(5)] {
                    let mut fact = report(&model, "key");
                    fact.key = "Tab".into();
                    fact.more = more;
                    fact.shift = shift;
                    fact.focused_index = index;
                    let result = decide(&mut model, &fact, true);
                    assert!(!result.prevent && !result.stop);
                    assert_eq!(result.request, None);
                    assert_eq!(result.focus, None);
                    assert!(model.open, "capture cannot close or redirect More Tab");
                }
            }
        }
        let mut ordinary = report(&model, "key");
        ordinary.key = "Tab".into();
        ordinary.focused_index = Some(5);
        assert_eq!(
            decide(&mut model, &ordinary, false).focus.as_deref(),
            Some("index:0"),
            "standalone Header boundary behavior is retained"
        );
    }

    #[test]
    fn only_live_open_more_tab_in_popover_enters_shared_release_policy() {
        let mut model = Model::new(true, true);
        model.open = true;
        model.mounted = true;
        let mut facts = KeyFacts {
            key: "Tab".into(),
            on_trigger: true,
            popover_ancestor: true,
            ..KeyFacts::default()
        };
        assert!(releases_more_tab(&model, &facts));
        facts.on_trigger = false;
        facts.inside = true;
        assert!(
            !releases_more_tab(&model, &facts),
            "ordinary content is not More"
        );
        facts.on_trigger = true;
        facts.popover_ancestor = false;
        assert!(!releases_more_tab(&model, &facts));
        facts.popover_ancestor = true;
        facts.default_prevented = true;
        assert!(!releases_more_tab(&model, &facts));
        facts.default_prevented = false;
        facts.key = "Escape".into();
        assert!(!releases_more_tab(&model, &facts));
        facts.key = "Tab".into();
        model.open = false;
        assert!(!releases_more_tab(&model, &facts));
        model.open = true;
        model.invalidate();
        assert!(!releases_more_tab(&model, &facts));
    }

    #[test]
    fn small_projection_preserves_opening_without_reusing_private_focus_work() {
        let mut model = Model::new(false, true);
        model.mounted = true;
        model.open = true;
        model.presence = [true; 4];
        model.sequence = 42;
        model.stop = true;
        model.focus("last".into());
        let previous = report(&model, "presence");
        let version = model.focus_version;
        assert!(
            !model.sync_mode(true, true),
            "slot movement must not request close"
        );
        assert!(model.open && model.small && model.more_visible());
        assert!(
            !model.accepts(&previous),
            "old physical host generation is stale"
        );
        assert_eq!(model.sequence, 0);
        assert!(!model.stop);
        assert_eq!(model.focus, "none");
        assert_eq!(model.focus_version, version.wrapping_add(1));
        assert!(!model.sync_mode(true, true), "same props are inert");
        assert!(model.open);
        assert!(
            model.sync_mode(true, false),
            "top-layer configuration remains separate"
        );
        assert!(!model.open);
    }

    fn capture(group: u8) -> CapturedFocus {
        CapturedFocus {
            token: "actual-captured-object".into(),
            group,
            epoch: 17,
        }
    }

    fn pending_capture(group: u8) -> Model {
        let mut model = Model::new(false, true);
        model.mounted = true;
        model.open = true;
        model.presence = [true; 4];
        model.focused_control = Some(capture(group));
        assert!(!model.sync_mode(true, true));
        model
    }

    fn preserve_report(model: &Model) -> Report {
        let capture = model.preserve.as_ref().unwrap();
        let mut report = report(model, "preserve-focus");
        report.focus_epoch = capture.epoch;
        report.active_default = true;
        report.focus_check = Some(FocusCheck {
            version: model.focus_version,
            token: capture.token.clone(),
            target_connected: true,
            target_owned: true,
            target_visible: true,
            target_group: Some(capture.group),
        });
        report
    }

    #[test]
    fn mode_focus_preservation_uses_current_group_presentation() {
        for (small, open, expected) in [
            (false, false, [true, true, false, true, true]),
            (false, true, [true, true, true, true, true]),
            (true, false, [false, false, false, true, true]),
            (true, true, [true, true, true, true, true]),
        ] {
            let mut model = Model::new(small, true);
            model.open = open;
            model.presence = [true; 4];
            for (group, expected) in expected.into_iter().enumerate() {
                assert_eq!(model.group_presented(group as u8), expected);
            }
            assert!(!model.group_presented(5));
        }
        let mut no_overflow = Model::new(false, true);
        no_overflow.presence = [false, true, false, false];
        assert!(!no_overflow.group_presented(4));
        no_overflow.small = true;
        assert!(no_overflow.group_presented(4));

        let mut closed = Model::new(false, true);
        closed.mounted = true;
        closed.focused_control = Some(capture(1));
        assert!(!closed.sync_mode(true, true));
        assert_eq!(closed.focus, "none", "hidden Small content is not revived");
        assert!(closed.preserve.is_none());
    }

    #[test]
    fn mode_capture_is_generation_scoped_and_authorized_only_once() {
        let mut model = pending_capture(1);
        assert_eq!(model.focus, "preserve");
        assert_eq!(model.preserve, Some(capture(1)));
        assert!(model.focused_control.is_none());
        let mut stale = preserve_report(&model);
        stale.generation -= 1;
        assert!(!model.authorize_preserve(&stale));
        let accepted = preserve_report(&model);
        assert!(model.authorize_preserve(&accepted));
        assert!(model.preserve.is_none());
        let mut repeated = accepted;
        repeated.sequence += 1;
        assert!(!model.authorize_preserve(&repeated));
    }

    #[test]
    fn mode_capture_rejects_outside_focus_replacement_and_stale_execution() {
        for condition in 0..11 {
            let mut model = pending_capture(1);
            let mut report = preserve_report(&model);
            match condition {
                0 => report.owner.push_str("-other-owner"),
                1 => report.connected = false,
                2 => report.focus_epoch += 1,
                3 => report.active_default = false,
                4 => report.focus_check.as_mut().unwrap().version += 1,
                5 => report
                    .focus_check
                    .as_mut()
                    .unwrap()
                    .token
                    .push_str("-replacement"),
                6 => report.focus_check.as_mut().unwrap().target_connected = false,
                7 => report.focus_check.as_mut().unwrap().target_owned = false,
                8 => report.focus_check.as_mut().unwrap().target_visible = false,
                9 => report.focus_check.as_mut().unwrap().target_group = Some(2),
                10 => report.sequence = model.sequence,
                _ => unreachable!(),
            }
            assert!(!model.authorize_preserve(&report), "condition {condition}");
        }
        let mut model = pending_capture(1);
        let mut active = preserve_report(&model);
        active.active_default = false;
        active.focused_control = Some(capture(1));
        assert!(
            model.authorize_preserve(&active),
            "already focused is admitted without native refocus"
        );

        let mut newer = pending_capture(1);
        let old = preserve_report(&newer);
        newer.focus("more".into());
        assert!(!newer.authorize_preserve(&old));
        let mut dropped = pending_capture(1);
        let old = preserve_report(&dropped);
        dropped.invalidate();
        assert!(!dropped.authorize_preserve(&old));
        let mut reprojected = pending_capture(1);
        let old = preserve_report(&reprojected);
        assert!(!reprojected.sync_mode(false, true));
        assert!(!reprojected.authorize_preserve(&old));
    }

    #[test]
    fn accepted_close_retains_only_visible_pending_mode_capture() {
        let mut large_default = pending_capture(1);
        large_default.small = false;
        large_default.open = false;
        large_default.apply_focus(false);
        assert_eq!(large_default.preserve, Some(capture(1)));
        let admitted = preserve_report(&large_default);
        assert!(large_default.authorize_preserve(&admitted));

        for group in [1, 2] {
            let mut hidden = pending_capture(group);
            hidden.open = false;
            hidden.apply_focus(false);
            assert!(hidden.preserve.is_none(), "closed Small group {group}");
            assert_eq!(hidden.focus, "none");
        }
        let mut restoring = pending_capture(1);
        restoring.open = false;
        restoring.apply_focus(true);
        assert_eq!(restoring.focus, "more");
        assert!(restoring.preserve.is_none());
    }

    #[test]
    fn presence_and_owner_reconciliation_reject_stale_native_work() {
        let mut model = Model::new(false, false);
        model.mounted = true;
        model.presence = [true, true, false, true];
        assert!(!model.more_visible());
        model.small = true;
        assert!(model.more_visible());
        let mut fact = report(&model, "presence");
        assert!(model.accepts(&fact));
        fact.owner.push('x');
        assert!(!model.accepts(&fact));
        fact = report(&model, "presence");
        model.sequence = fact.sequence;
        assert!(!model.accepts(&fact));
        fact = report(&model, "presence");
        model.open = true;
        assert!(model.sync_mode(false, true));
        assert!(!model.open);
        assert!(!model.accepts(&fact));
        let fact = report(&model, "presence");
        model.invalidate();
        assert!(!model.accepts(&fact));
    }
    #[test]
    fn click_choices_distinguish_background_defaults_and_input_modality() {
        let mut model = Model::new(true, false);
        model.open = true;
        model.presence = [true; 4];
        let mut fact = report(&model, "click");
        fact.background = true;
        assert!(decide(&mut model, &fact, false).prevent);
        assert_eq!(decide(&mut model, &fact, false).request, None);
        fact.background = false;
        assert_eq!(decide(&mut model, &fact, false).request, Some(false));
        assert_eq!(decide(&mut model, &fact, false).focus, None);
        model.keyboard = true;
        assert_eq!(
            decide(&mut model, &fact, false).focus.as_deref(),
            Some("more")
        );
        fact.default_prevented = true;
        assert_eq!(decide(&mut model, &fact, false).request, None);
    }
}
