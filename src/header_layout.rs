//! Stable arbitrary-content Header presentation. Native code reports DOM facts;
//! Rust owns presence, open state, cancellation, keyboard choices and focus intent.
use crate::overlay_registry::{CloseBehavior, OverlayKind};
use crate::overlay_runtime::{self, Callbacks, Change, NodeHandle, Reason};
use dioxus::prelude::*;
use serde::Deserialize;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[path = "header_layout_dom.rs"]
mod native;
static NEXT_OWNER: AtomicUsize = AtomicUsize::new(1);

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
            stop: false,
        }
    }
    fn more_visible(&self) -> bool {
        self.presence[2] || (self.small && (self.presence[0] || self.presence[1]))
    }
    fn focus(&mut self, target: String) {
        self.focus = target;
        self.focus_version = self.focus_version.wrapping_add(1);
    }
    fn sync_mode(&mut self, small: bool, top_layer: bool) -> bool {
        if self.small == small && self.top_layer == top_layer {
            return false;
        }
        let closed = self.open;
        self.small = small;
        self.top_layer = top_layer;
        self.open = false;
        self.generation = self.generation.wrapping_add(1);
        self.sequence = 0;
        self.stop = false;
        self.focus("none".into());
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
    }
}

#[derive(Default)]
struct Decision {
    prevent: bool,
    stop: bool,
    request: Option<bool>,
    focus: Option<String>,
}
fn decide(model: &mut Model, report: &Report) -> Decision {
    let mut decision = Decision::default();
    match report.kind.as_str() {
        "pointer" => model.keyboard = false,
        "key" if !report.default_prevented => {
            model.keyboard = true;
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

#[derive(Clone)]
struct Controller {
    model: Rc<RefCell<Model>>,
    revision: Signal<u64>,
    onchange: EventHandler<bool>,
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
        let (decision, generation, previous, forced_close) = {
            let mut model = self.model.borrow_mut();
            if !model.accepts(&report) {
                return;
            }
            model.sequence = report.sequence;
            model.presence = report.presence;
            let forced_close = model.open && !model.more_visible();
            if forced_close {
                model.open = false;
                model.focus("none".into());
            }
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
                decide(&mut model, &report)
            };
            model.stop = decision.stop;
            (decision, model.generation, model.open, forced_close)
        };
        if decision.prevent {
            event.prevent_default();
        }
        if forced_close {
            self.onchange.call(false);
        }
        if forced_close {
            self.node.sync_open(false);
            self.node.close_children(Reason::Configuration);
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
    let node = {
        let read = model.clone();
        let before = model.clone();
        let apply = model.clone();
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
                        model.focus(if change.restore_focus { "more" } else { "none" }.into());
                    }
                    let mut revision = revision;
                    if let Ok(mut value) = revision.try_write() {
                        *value = value.wrapping_add(1);
                    }
                    onchange.call(change.open);
                }),
                key: None,
            },
        )
    };

    let bridge = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    // Configuration changes close the old presentation without replacing children.
    // This is configuration reconciliation, not a user request that can be vetoed.
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
    use_drop(move || {
        cleanup_model.borrow_mut().invalidate();
        cleanup_node.close_children(Reason::Drop);
        cleanup_node.unregister();
        if let Some(bridge) = cleanup_bridge.borrow_mut().take() {
            let _ = bridge.send("finish");
        }
    });
    let controller = Controller {
        model: model.clone(),
        revision,
        onchange,
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
            "data-small":small.to_string(),"data-open":open.to_string(),"data-top-layer":enable_top_layer.to_string(),
            "data-generation":generation,"data-focus-version":focus_version,"data-focus-target":focus,
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
            let decision = decide(&mut model, &fact);
            assert!(decision.prevent);
            assert_eq!(decision.request, Some(true));
            assert!(!decision.stop);
        }
        model.open = true;
        let mut trigger = report(&model, "key");
        trigger.more = true;
        trigger.key = "Enter".into();
        let decision = decide(&mut model, &trigger);
        assert!(!decision.prevent);
        assert_eq!(
            decision.request, None,
            "open trigger retains native click activation"
        );
        model.open = false;
        trigger.alt = true;
        let decision = decide(&mut model, &trigger);
        assert!(!decision.prevent);
        assert_eq!(decision.request, Some(true));
        assert_eq!(decision.focus, None);
        model.open = true;
        let mut fact = report(&model, "key");
        fact.key = "Escape".into();
        let decision = decide(&mut model, &fact);
        assert!(!decision.prevent && decision.stop);
        assert_eq!(decision.request, Some(false));
        assert_eq!(decision.focus.as_deref(), Some("more"));
        fact.key = "Tab".into();
        fact.focused_index = Some(1);
        assert!(!decide(&mut model, &fact).prevent);
        fact.focused_index = Some(5);
        assert_eq!(decide(&mut model, &fact).focus.as_deref(), Some("index:0"));
        fact.shift = true;
        fact.focused_index = Some(0);
        assert_eq!(decide(&mut model, &fact).focus.as_deref(), Some("index:5"));
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
        assert!(decide(&mut model, &fact).prevent);
        assert_eq!(decide(&mut model, &fact).request, None);
        fact.background = false;
        assert_eq!(decide(&mut model, &fact).request, Some(false));
        assert_eq!(decide(&mut model, &fact).focus, None);
        model.keyboard = true;
        assert_eq!(decide(&mut model, &fact).focus.as_deref(), Some("more"));
        fact.default_prevented = true;
        assert_eq!(decide(&mut model, &fact).request, None);
    }
}
