//! Stable Dioxus Header presentation. Rust owns slot presence and mounted control
//! groups; shared canonical focus observations supply native eligibility facts.
use crate::header_control_scope::{HeaderControlGroup, HeaderControls};
use crate::overlay_registry::{CloseBehavior, OverlayKind};
use crate::overlay_runtime::{
    self, Callbacks, Change, FocusObservation, KeyDecision, KeyFacts, NodeHandle, Reason,
    SurfaceObservation,
};
use dioxus::prelude::*;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_OWNER: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Debug, PartialEq, Eq)]
struct FocusedControl {
    id: String,
    group: u8,
}

// Rust projection of explicitly registered shared focus facts.
#[derive(Clone, Debug, PartialEq, Eq)]
struct PresentedControl {
    id: String,
    tab_index: i32,
    active: bool,
    group: Option<u8>,
    in_popup: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PopupPosition {
    left: f64,
    top: f64,
}
fn popup_position(
    viewport: f64,
    trigger_left: f64,
    trigger_bottom: f64,
    width: f64,
) -> Option<PopupPosition> {
    if ![viewport, trigger_left, trigger_bottom, width]
        .into_iter()
        .all(f64::is_finite)
        || viewport <= 0.0
        || width <= 0.0
    {
        return None;
    }
    Some(PopupPosition {
        left: 8.0_f64.max(trigger_left.min(viewport - width - 8.0)),
        top: trigger_bottom,
    })
}

struct Model {
    owner: String,
    alive: bool,
    mounted: bool,
    generation: u64,
    small: bool,
    top_layer: bool,
    open: bool,
    presence: [bool; 4],
    focus: String,
    controls: Vec<PresentedControl>,
    preserve: Option<FocusedControl>,
    preserve_target: Option<overlay_runtime::MountedFocusTarget>,
    active_default: bool,
    focus_observation: Option<FocusObservation>,
    focus_generation: u64,
    changed: Option<EventHandler<bool>>,
    viewport_width: f64,
    more_mount: Option<Rc<MountedData>>,
    popup_mounts: [Option<Rc<MountedData>>; 2],
    position: Option<PopupPosition>,
    layout_epoch: u64,
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
            small,
            top_layer,
            open: false,
            presence: [false; 4],
            focus: "none".into(),
            controls: Vec::new(),
            preserve: None,
            preserve_target: None,
            active_default: false,
            focus_observation: None,
            focus_generation: 0,
            changed: None,
            viewport_width: 0.0,
            more_mount: None,
            popup_mounts: [None, None],
            position: None,
            layout_epoch: 0,
        }
    }
    fn popup_controls(&self) -> impl Iterator<Item = &PresentedControl> {
        self.controls
            .iter()
            .filter(|control| control.in_popup && control.tab_index >= 0)
    }
    fn popup_ids(&self) -> Vec<String> {
        self.popup_controls()
            .map(|control| control.id.clone())
            .collect()
    }
    fn focused_control(&self) -> Option<FocusedControl> {
        let control = self.controls.iter().find(|control| control.active)?;
        if control.id.is_empty() {
            return None;
        }
        Some(FocusedControl {
            id: control.id.clone(),
            group: control.group?,
        })
    }
    fn presented(&self, id: &str) -> bool {
        !id.is_empty() && self.controls.iter().any(|control| control.id == id)
    }
    fn invalidate_layout(&mut self) {
        self.layout_epoch = self.layout_epoch.wrapping_add(1);
        self.position = None;
    }
    fn accepts_layout(&self, epoch: u64, generation: u64, small: bool) -> bool {
        self.alive
            && self.mounted
            && self.open
            && self.small == small
            && self.generation == generation
            && self.layout_epoch == epoch
    }
    fn popup_style(&self, small: bool) -> Option<String> {
        if !self.open || self.small != small {
            return None;
        }
        let mut style = String::from("position:fixed;margin:0;");
        if self.viewport_width > 0.0 {
            style.push_str(&format!(
                "max-width:{}px;",
                (self.viewport_width - 16.0).max(0.0)
            ));
        }
        if let Some(position) = self.position {
            style.push_str(&format!("left:{}px;top:{}px;", position.left, position.top));
        }
        Some(style)
    }
    fn more_visible(&self) -> bool {
        self.presence[2] || (self.small && (self.presence[0] || self.presence[1]))
    }
    fn focus(&mut self, target: String) {
        self.focus = target;
        self.preserve = None;
        self.preserve_target = None;
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
    fn preserve_focus(&mut self, capture: FocusedControl) {
        self.focus = "preserve".into();
        self.preserve = Some(capture);
    }
    fn apply_focus(&mut self, restore: bool) {
        // The source's attempt to focus hidden More has no native effect. Keep
        // an eligible visible mode-handoff capture in that case.
        let restore = restore && self.more_visible();
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
    fn sync_mode(&mut self, small: bool, top_layer: bool) -> bool {
        if self.small == small && self.top_layer == top_layer {
            return false;
        }
        let focused = self.focused_control();
        // Responsive slot movement retains the same overflow Dropdown and
        // current children. A top-layer configuration change still closes it.
        let layer_changed = self.top_layer != top_layer;
        let closed = self.open && layer_changed;
        self.small = small;
        self.top_layer = top_layer;
        if layer_changed {
            self.open = false;
        }
        self.generation = self.generation.wrapping_add(1);
        self.invalidate_layout();
        self.controls.clear();
        self.focus_generation = 0;
        let capture = (!layer_changed && self.alive && self.mounted)
            .then_some(focused)
            .flatten()
            .filter(|capture| self.group_presented(capture.group));
        if let Some(capture) = capture {
            self.preserve_focus(capture);
        } else {
            self.focus("none".into());
        }
        closed
    }
    fn invalidate(&mut self) {
        self.alive = false;
        self.mounted = false;
        self.open = false;
        self.generation = self.generation.wrapping_add(1);
        self.invalidate_layout();
        self.more_mount = None;
        self.popup_mounts = [None, None];
        self.controls.clear();
        self.focus_observation = None;
        self.focus_generation = 0;
        self.focus("none".into());
    }
}

fn queue_changed(model: &Rc<RefCell<Model>>, node: &NodeHandle, open: bool) {
    let current = model.clone();
    // Breakpoint/focus generations do not replace the source Dropdown. Its
    // admitted payload survives those updates; the registered node owns actual
    // teardown/replacement cancellation, and delivery reads the current handler.
    node.queue_completion(Rc::new(move || {
        let handler = {
            let model = current.borrow();
            if !model.alive {
                return;
            }
            model.changed
        };
        if let Some(handler) = handler {
            handler.call(open);
        }
    }));
}

// Event fields come directly from Dioxus; only focus order/index are cached
// read-only browser facts, never an event replay or native policy transport.
#[derive(Default)]
struct KeyInput {
    key: String,
    shift: bool,
    alt: bool,
    default_prevented: bool,
    focus_count: usize,
    focused_index: Option<usize>,
    more: bool,
    inside: bool,
}

#[derive(Default)]
struct Decision {
    prevent: bool,
    stop: bool,
    request: Option<bool>,
    focus: Option<String>,
}
fn decide(model: &mut Model, report: &KeyInput, popover_ancestor: bool) -> Decision {
    let mut decision = Decision::default();
    if !report.default_prevented {
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
            decision.prevent = true;
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
    runtime.close_and_release_focus(node.id());
    KeyDecision {
        release_focus: true,
        ..KeyDecision::default()
    }
}

fn focus_ids<'a>(choice: &str, ids: &'a [String]) -> Vec<&'a str> {
    match choice {
        "first" => ids.iter().map(String::as_str).collect(),
        "last" => ids.iter().rev().map(String::as_str).collect(),
        _ => choice
            .strip_prefix("index:")
            .and_then(|index| index.parse::<usize>().ok())
            .and_then(|index| ids.get(index))
            .map(|id| vec![id.as_str()])
            .unwrap_or_default(),
    }
}

// Header membership is explicit Dioxus registration. Native candidates retain
// their canonical DOM order and eligibility; there is no Header-local scanner.
fn presented_controls(
    observation: &FocusObservation,
    groups: &BTreeMap<String, u8>,
    owner: &str,
) -> Vec<PresentedControl> {
    let more = format!("{owner}-more");
    observation
        .candidates
        .iter()
        .filter_map(|candidate| {
            let group = if candidate.target == more {
                4
            } else {
                *groups.get(&candidate.target)?
            };
            Some(PresentedControl {
                id: candidate.target.clone(),
                tab_index: candidate.tab_index,
                active: candidate.exact_active,
                group: Some(group),
                in_popup: candidate.hosts.iter().any(|host| host == owner),
            })
        })
        .collect()
}

#[derive(Clone)]
struct Controller {
    model: Rc<RefCell<Model>>,
    revision: Signal<u64>,
    node: NodeHandle,
}
impl Controller {
    fn project_controls(&self, groups: &BTreeMap<String, u8>) -> bool {
        let mut model = self.model.borrow_mut();
        if !model.alive || !model.mounted || model.focus_generation != model.generation {
            return false;
        }
        let Some(observation) = model.focus_observation.as_ref() else {
            return false;
        };
        let controls = presented_controls(observation, groups, self.node.id());
        let active_default = observation.active_default;
        let changed = model.controls != controls || model.active_default != active_default;
        model.controls = controls;
        model.active_default = active_default;
        changed
    }
    fn observe_focus(&self, observation: FocusObservation, groups: &BTreeMap<String, u8>) {
        {
            let mut model = self.model.borrow_mut();
            if !model.alive || !model.mounted {
                return;
            }
            model.focus_observation = Some(observation);
            model.focus_generation = model.generation;
        }
        if self.project_controls(groups) {
            self.notify();
        }
    }
    fn pointer_input(&self) {
        let changed = {
            let mut model = self.model.borrow_mut();
            if !model.alive || !model.mounted {
                return;
            }
            let changed = model.focus != "none"
                || model.preserve.is_some()
                || model.preserve_target.is_some();
            model.focus("none".into());
            changed
        };
        // The shared primary input adapter reports mousedown/touchstart. Exact
        // pointerdown chronology is not needed to cancel an obsolete focus job.
        self.node.cancel_registered_focus();
        if changed {
            self.notify();
        }
    }
    fn measure_layout(&self) {
        let Some((more, popup, viewport, epoch, generation, small)) = ({
            let mut model = self.model.borrow_mut();
            model.layout_epoch = model.layout_epoch.wrapping_add(1);
            if !model.alive || !model.mounted || !model.open || model.viewport_width <= 0.0 {
                None
            } else {
                model
                    .more_mount
                    .clone()
                    .zip(model.popup_mounts[usize::from(!model.small)].clone())
                    .map(|(more, popup)| {
                        (
                            more,
                            popup,
                            model.viewport_width,
                            model.layout_epoch,
                            model.generation,
                            model.small,
                        )
                    })
            }
        }) else {
            return;
        };
        let controller = self.clone();
        spawn(async move {
            let Ok(trigger) = more.get_client_rect().await else {
                return;
            };
            let Ok(bounds) = popup.get_client_rect().await else {
                return;
            };
            let Some(position) =
                popup_position(viewport, trigger.min_x(), trigger.max_y(), bounds.width())
            else {
                return;
            };
            let changed = {
                let mut model = controller.model.borrow_mut();
                if !model.accepts_layout(epoch, generation, small) {
                    return;
                }
                if model.position == Some(position) {
                    false
                } else {
                    model.position = Some(position);
                    true
                }
            };
            if changed {
                controller.notify();
            }
        });
    }
    fn mount_layout(&self, index: Option<usize>, node: Rc<MountedData>) {
        {
            let mut model = self.model.borrow_mut();
            model.invalidate_layout();
            match index {
                Some(index) => model.popup_mounts[index] = Some(node),
                None => model.more_mount = Some(node),
            }
        }
        self.measure_layout();
    }
    fn more_click(&self, event: MouseEvent) {
        if !event.default_action_enabled() {
            return;
        }
        let handled = self
            .node
            .with_owner(|| {
                let next = {
                    let model = self.model.borrow();
                    if !model.alive || !model.mounted || !model.more_visible() {
                        return false;
                    }
                    !model.open
                };
                self.node.request(next, Reason::Trigger, false);
                if let Some(runtime) = self.node.runtime() {
                    runtime.dismiss_dropdown_others(self.node.id(), self.node.generation());
                }
                true
            })
            .unwrap_or(false);
        if handled {
            event.stop_propagation();
        }
    }
    fn group_click(&self, group: u8, event: MouseEvent) {
        let active = {
            let model = self.model.borrow();
            model.alive
                && model.mounted
                && model.open
                && match group {
                    0 | 1 => model.small,
                    2 => true,
                    _ => false,
                }
        };
        if active {
            self.node.dioxus_inside_click(&event);
        }
    }
    fn background_click(&self, small: bool, event: MouseEvent) {
        let model = self.model.borrow();
        if model.alive
            && model.mounted
            && model.open
            && model.small == small
            && self.node.is_registered()
        {
            event.stop_propagation();
        }
    }
    fn notify(&self) {
        let mut revision = self.revision;
        let next = revision.peek().wrapping_add(1);
        revision.set(next);
    }
    fn registered_target(&self, choice: &str) -> Option<overlay_runtime::MountedFocusTarget> {
        let (owner, ids, preserve, preserve_target) = {
            let model = self.model.borrow();
            (
                model.owner.clone(),
                model.popup_ids(),
                model.preserve.clone(),
                model.preserve_target.clone(),
            )
        };
        match choice {
            "more" => self.node.capture_registered_focus(&format!("{owner}-more")),
            "preserve" => {
                let model = self.model.borrow();
                let valid = preserve.is_some_and(|capture| {
                    model.focus_generation == model.generation
                        && model.group_presented(capture.group)
                        && model.presented(&capture.id)
                        && (model.active_default
                            || model.focused_control().as_ref() == Some(&capture))
                });
                valid
                    .then_some(preserve_target)
                    .flatten()
                    .filter(|target| target.live_node().is_some())
            }
            _ => focus_ids(choice, &ids)
                .into_iter()
                .filter(|id| !id.is_empty())
                .find_map(|id| self.node.capture_registered_focus(id)),
        }
    }
    fn filter_tab_focus(&self, report: &KeyInput, decision: &mut Decision) {
        if report.key == "Tab"
            && decision
                .focus
                .as_deref()
                .is_some_and(|focus| self.registered_target(focus).is_none())
        {
            decision.prevent = false;
            decision.stop = false;
            decision.focus = None;
        }
    }
    fn dispatch_focus(&self) {
        let (choice, generation, open, more_visible) = {
            let model = self.model.borrow();
            if !model.alive || !model.mounted {
                return;
            }
            (
                model.focus.clone(),
                model.generation,
                model.open,
                model.more_visible(),
            )
        };
        if choice == "none" {
            return;
        }
        if choice == "more" && !more_visible
            || !open && !matches!(choice.as_str(), "more" | "preserve")
        {
            self.model.borrow_mut().focus("none".into());
            return;
        }
        let Some(target) = self.registered_target(&choice) else {
            // Opening candidates arrive with the next shared focus observation.
            // Unsupported controls never become a native focus fallback.
            let awaiting_presentation = {
                let model = self.model.borrow();
                choice == "preserve" && model.focus_generation != model.generation
                    || matches!(choice.as_str(), "first" | "last")
                        && model.popup_controls().next().is_none()
            };
            if !awaiting_presentation {
                self.model.borrow_mut().focus("none".into());
            }
            return;
        };
        let expected_open = (choice != "preserve").then_some(open);
        if self.node.queue_registered_focus(target, expected_open) {
            let mut model = self.model.borrow_mut();
            if model.generation == generation && model.focus == choice {
                model.focus("none".into());
            }
        }
    }
    fn key(&self, event: KeyboardEvent, more: bool, host_small: Option<bool>) {
        let (input, generation, previous) = {
            let model = self.model.borrow();
            if !model.alive
                || !model.mounted
                || !self.node.is_registered()
                || host_small.is_some_and(|small| small != model.small)
                || !event.default_action_enabled()
            {
                return;
            }
            (
                KeyInput {
                    key: event.key().to_string(),
                    shift: event.modifiers().contains(Modifiers::SHIFT),
                    alt: event.modifiers().contains(Modifiers::ALT),
                    default_prevented: !event.default_action_enabled(),
                    focus_count: model.popup_controls().count(),
                    focused_index: model.popup_controls().position(|control| control.active),
                    more,
                    inside: host_small.is_some(),
                },
                model.generation,
                model.open,
            )
        };
        let Some(facts) = self.node.dioxus_key_facts(&event, more, None, input.inside) else {
            return;
        };
        // More Tab releases through a containing Popover using shared Rust
        // focus policy. Other keys remain local Dioxus events.
        if more && input.key == "Tab" && facts.popover_ancestor {
            let decision = self.node.dioxus_key(facts);
            if decision.prevent {
                event.prevent_default();
            }
            if decision.stop || decision.release_focus {
                event.stop_propagation();
            }
            self.dispatch_focus();
            self.notify();
            return;
        }
        // A newer nested/peer overlay owns Escape until it closes. The local
        // Header policy retains its registered More restoration after acceptance.
        if input.key == "Escape" && !self.node.owns_keyboard_topmost() {
            return;
        }
        let mut decision = decide(&mut self.model.borrow_mut(), &input, facts.popover_ancestor);
        self.filter_tab_focus(&input, &mut decision);
        if decision.prevent {
            event.prevent_default();
        }
        if decision.stop || decision.request.is_some() {
            event.stop_propagation();
        }
        let trigger_others = more && !previous && decision.request == Some(true);
        let accepted = decision
            .request
            .filter(|next| *next != previous)
            .is_none_or(|next| self.node.request(next, Reason::Keyboard, false));
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
        if trigger_others && let Some(runtime) = self.node.runtime() {
            runtime.dismiss_dropdown_others(self.node.id(), self.node.generation());
        }
        self.dispatch_focus();
        self.notify();
    }
    fn viewport_changed(&self, width: f64) {
        {
            let mut model = self.model.borrow_mut();
            if !model.alive || !model.mounted || !width.is_finite() || width <= 0.0 {
                return;
            }
            model.viewport_width = width;
            model.layout_epoch = model.layout_epoch.wrapping_add(1);
        }
        self.dispatch_focus();
        self.notify();
    }
}

#[component]
pub(crate) fn HeaderLayout(
    #[props(default)] default_content: Option<Element>,
    #[props(default)] secondary: Option<Element>,
    #[props(default)] overflow: Option<Element>,
    #[props(default)] avatar: Option<Element>,
    #[props(default)] actions_present: Option<bool>,
    #[props(default)] secondary_present: Option<bool>,
    #[props(default)] overflow_present: Option<bool>,
    #[props(default)] avatar_present: Option<bool>,
    #[props(default)] small: bool,
    #[props(default)] enable_top_layer: bool,
    #[props(default)] discovery_ready: bool,
    more_label: String,
    #[props(default)] onrequest: EventHandler<crate::HeaderOverflowEvent>,
    #[props(default)] onchange: EventHandler<bool>,
) -> Element {
    let model = use_hook(|| Rc::new(RefCell::new(Model::new(small, enable_top_layer))));
    {
        let mut model = model.borrow_mut();
        model.changed = Some(onchange);
        model.presence = [
            secondary_present.unwrap_or(secondary.is_some()),
            actions_present.unwrap_or(default_content.is_some()),
            overflow_present.unwrap_or(overflow.is_some()),
            avatar_present.unwrap_or(avatar.is_some()),
        ];
    }
    let revision = use_signal(|| 0_u64);
    let runtime = overlay_runtime::use_overlay_runtime();
    let node_slot = use_hook(|| Rc::new(RefCell::new(None::<NodeHandle>)));
    let node = {
        let read = model.clone();
        let before = model.clone();
        let apply = model.clone();
        let key_model = model.clone();
        let key_slot = node_slot.clone();
        let completion_slot = node_slot.clone();
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
                    let changed = {
                        let mut model = apply.borrow_mut();
                        if !model.alive {
                            return;
                        }
                        let changed = model.open != change.open;
                        model.open = change.open;
                        if changed {
                            // Invalidate only traversal candidates. Retain the
                            // last observed active/group facts for a responsive
                            // handoff before the next presentation observation.
                            for control in &mut model.controls {
                                control.in_popup = false;
                            }
                            model.invalidate_layout();
                        }
                        model.apply_focus(change.restore_focus);
                        changed
                    };
                    let mut revision = revision;
                    if let Ok(mut value) = revision.try_write() {
                        *value = value.wrapping_add(1);
                    }
                    if let Some(node) = completion_slot.borrow().clone() {
                        node.cancel_registered_focus();
                        if changed {
                            queue_changed(&apply, &node, change.open);
                        }
                    }
                }),
                key: Some(Rc::new(move |facts| {
                    header_tab_key(&key_model, &key_slot, facts)
                })),
            },
        )
    };
    *node_slot.borrow_mut() = Some(node.clone());
    node.set_ownership(overlay_runtime::use_overlay_ancestors(), discovery_ready);
    let mounted_node = node.clone();
    let secondary_scope = small.then(|| node.clone());
    let default_scope = small.then(|| node.clone());
    let overflow_scope = Some(node.clone());

    let viewport = crate::document_viewport::use_document_viewport(true);
    // Slot projection preserves presence and current logical relationships. A
    // top-layer change still closes through bounded configuration reconciliation,
    // rather than a vetoable user request. Both invalidate private focus tasks.
    let mode_changed = {
        let model = model.borrow();
        model.small != small || model.top_layer != enable_top_layer
    };
    let preserve_target = mode_changed
        .then(|| {
            let id = model.borrow().focused_control().map(|control| control.id);
            id.and_then(|id| node.capture_registered_focus(&id))
        })
        .flatten();
    if mode_changed {
        node.cancel_registered_focus();
    }
    let mode_closed = model.borrow_mut().sync_mode(small, enable_top_layer);
    if mode_changed {
        let mut model = model.borrow_mut();
        model.preserve_target = model.preserve.as_ref().and(preserve_target);
    }
    if mode_closed {
        node.sync_open(false);
        queue_changed(&model, &node, false);
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
            }
        });
    }
    let cleanup_model = model.clone();
    let cleanup_node = node.clone();
    let cleanup_slot = node_slot.clone();
    use_drop(move || {
        cleanup_model.borrow_mut().invalidate();
        cleanup_node.close_children(Reason::Drop);
        cleanup_node.unregister();
        cleanup_slot.borrow_mut().take();
    });
    let controller = Controller {
        model: model.clone(),
        revision,
        node,
    };
    let controls = use_hook({
        let current = controller.clone();
        move || {
            HeaderControls::new(Rc::new(move || {
                if current.model.borrow().alive {
                    current.notify();
                }
            }))
        }
    });
    let observer_controller = controller.clone();
    let observer_controls = controls.clone();
    controller
        .node
        .set_surface_observer(Rc::new(move |observation| match observation {
            SurfaceObservation::Focus(focus) => {
                observer_controller.observe_focus(focus, &observer_controls.groups());
            }
            SurfaceObservation::Pointer => observer_controller.pointer_input(),
            SurfaceObservation::RequestedFocus { .. } => {}
        }));
    let viewport_controller = controller.clone();
    let viewport_facts = viewport.clone();
    use_effect(move || {
        let _ = viewport_facts.layout_revision();
        if let Some(facts) = viewport_facts.facts() {
            viewport_controller.viewport_changed(facts.width);
        }
    });
    let focus_controller = controller.clone();
    let focus_controls = controls.clone();
    use_effect(move || {
        let _ = revision();
        focus_controller.project_controls(&focus_controls.groups());
        focus_controller.dispatch_focus();
        focus_controller.measure_layout();
    });
    let mount_model = model.clone();
    let mount_viewport = controller.clone();
    let _ = revision();
    let state = model.borrow();
    let owner = state.owner.clone();
    let generation = state.generation.to_string();
    let open = state.open;
    let visible = state.more_visible();
    let presence = state.presence;
    let small_style = state.popup_style(true);
    let large_style = state.popup_style(false);
    let active = if small { "small" } else { "large" };
    let more_id = format!("{owner}-more");
    let small_id = format!("{owner}-small");
    let large_id = format!("{owner}-large");
    controller
        .node
        .set_surface_metadata(overlay_runtime::SurfaceMetadata {
            trigger_id: more_id.clone(),
            items_host: if small {
                small_id.clone()
            } else {
                large_id.clone()
            },
            popover_host: String::new(),
        });
    drop(state);
    let more_mount = crate::dropdown::use_focus_mount(more_id.clone());
    let small_popover = crate::native_surface::use_native_popover(
        small_id.clone(),
        small && open && enable_top_layer,
    );
    let large_popover = crate::native_surface::use_native_popover(
        large_id.clone(),
        !small && open && enable_top_layer,
    );
    let small_layout = controller.clone();
    let large_layout = controller.clone();
    let more_layout = controller.clone();
    let small_resize = controller.clone();
    let large_resize = controller.clone();
    let more_resize = controller.clone();
    let visible_layout = controller.clone();
    let layout_controller = controller.clone();
    let layout_resize = use_callback(move |_: ResizeEvent| layout_controller.measure_layout());
    let visible_model = model.clone();
    let visible_node = controller.node.clone();
    let runtime_owner = runtime.owner_id();
    let more_key = controller.clone();
    let small_key = controller.clone();
    let large_key = controller.clone();
    let more_click = controller.clone();
    let secondary_click = controller.clone();
    let default_click = controller.clone();
    let overflow_click = controller.clone();
    let small_background = controller.clone();
    let large_background = controller.clone();
    rsx! {
        div { id:owner.clone(), class:"ix-header-layout",
            onresize:layout_resize,
            "data-ix-overlay-owner":runtime_owner,
            "data-ix-overlay-root":owner.clone(),"data-ix-overlay-generation":"1",
            "data-ix-overlay-open":open.to_string(),"data-ix-overlay-ready":discovery_ready.to_string(),
            "data-ix-overlay-mode":"active-descendant",
            "data-small":small.to_string(),"data-open":open.to_string(),"data-top-layer":enable_top_layer.to_string(),
            "data-generation":generation,
            onmounted:move |_| {
                mounted_node.set_mounted();
                let mut model = mount_model.borrow_mut();
                if !model.alive || model.mounted { return; }
                model.mounted = true;
                drop(model);
                if let Some(facts) = viewport.facts() {
                    mount_viewport.viewport_changed(facts.width);
                }
            },
            overlay_runtime::OverlayTransport {runtime}
            dialog {id:small_id.clone(),class:"ix-header-popup-host ix-header-popup-small",
                "data-ix-overlay-host":small.then_some(owner.clone()),
                "data-header-popup-host":"small","data-active":small.to_string(),"data-open":(small&&open).to_string(),
                role:if small {Some("presentation")} else {None},
                popover:if small&&enable_top_layer {Some("manual")} else {None},
                "data-ix-native-popover":small_popover.marker,
                style:small_style,
                onmounted:move |event:MountedEvent| {
                    small_popover.onmounted.call(event.clone());
                    small_layout.mount_layout(Some(0),event.data());
                },
                onresize:move |_|small_resize.measure_layout(),
                onclick:move |event|small_background.background_click(true,event),
                onkeydown:move |event|small_key.key(event,false,Some(true)),
                div {class:"ix-header-slot-group ix-header-slot-secondary","data-header-group":"secondary","data-populated":presence[0].to_string(),onresize:layout_resize,onclick:move |event|secondary_click.group_click(0,event),HeaderOwnershipScope {node:secondary_scope,controls:controls.clone(),group:0,{secondary}}}
                div {class:"ix-header-slot-group ix-header-slot-default","data-header-group":"default","data-populated":presence[1].to_string(),"data-separator":(small&&presence[0]&&presence[1]).to_string(),onresize:layout_resize,onclick:move |event|default_click.group_click(1,event),HeaderOwnershipScope {node:default_scope,controls:controls.clone(),group:1,{default_content}}}
                dialog {id:large_id.clone(),class:"ix-header-popup-host ix-header-popup-large",
                    "data-ix-overlay-host":(!small).then_some(owner.clone()),
                    "data-header-popup-host":"large","data-active":(!small).to_string(),"data-open":(!small&&open).to_string(),
                    role:if !small {Some("presentation")} else {None},
                    popover:if !small&&enable_top_layer {Some("manual")} else {None},
                    "data-ix-native-popover":large_popover.marker,
                    style:large_style,
                    onmounted:move |event:MountedEvent| {
                        large_popover.onmounted.call(event.clone());
                        large_layout.mount_layout(Some(1),event.data());
                    },
                    onresize:move |_|large_resize.measure_layout(),
                    onclick:move |event|large_background.background_click(false,event),
                    onkeydown:move |event|large_key.key(event,false,Some(false)),
                    div {class:"ix-header-slot-group ix-header-slot-overflow","data-header-group":"overflow","data-populated":presence[2].to_string(),"data-separator":(small&&presence[2]&&(presence[0]||presence[1])).to_string(),onresize:layout_resize,onclick:move |event|overflow_click.group_click(2,event),HeaderOwnershipScope {node:overflow_scope,controls:controls.clone(),group:2,{overflow}}}
                }
            }
            button {id:more_id,r#type:"button",class:"ix-app-icon-button ix-header-more-button",
                onmounted:move |event:MountedEvent| {
                    more_mount.call(event.clone());
                    more_layout.mount_layout(None,event.data());
                },
                onresize:move |_|more_resize.measure_layout(),
                onclick:move |event|more_click.more_click(event),
                onkeydown:move |event|more_key.key(event,true,None),
                onvisible:move |event:VisibleEvent| {
                    let alive = visible_model.borrow().alive;
                    if alive
                        && let Ok(intersecting) = event.is_intersecting()
                    {
                        visible_node.observe_anchor_visibility(intersecting, None);
                        visible_layout.measure_layout();
                    }
                },
                "data-ix-overlay-trigger":owner.clone(),
                "data-header-more-button":owner.clone(),"data-testid":"show-more",hidden:!visible,
                "aria-label":more_label,"aria-expanded":open.to_string(),"aria-controls":if active=="small" {small_id} else {large_id},
                crate::Icon {name:"more-menu".to_string(),size:24}
            }
            div {class:"ix-header-avatar-slot","data-header-group":"avatar",onresize:layout_resize,HeaderOwnershipScope {node:None,controls:controls.clone(),group:3,{avatar}}}
        }
    }
}

// Scope-only component: responsive ownership changes without new DOM wrappers
// or relocating consumer content and its mounted element handles.
#[component]
fn HeaderOwnershipScope(
    node: Option<NodeHandle>,
    controls: HeaderControls,
    group: u8,
    children: Element,
) -> Element {
    overlay_runtime::use_overlay_scope(node);
    use_context_provider(|| HeaderControlGroup { controls, group });
    rsx! {{children}}
}

pub(crate) const STYLES: &str = include_str!("styles/header_layout.css");

#[cfg(test)]
mod tests;
