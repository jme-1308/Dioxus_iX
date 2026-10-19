//! Stable Dioxus Header presentation. Rust owns slot presence and mounted control
//! groups; shared canonical focus observations supply native eligibility facts.
use crate::header_control_scope::{HeaderControlGroup, HeaderControls};
use crate::overlay_registry::{CloseBehavior, OverlayKind};
use crate::overlay_runtime::{
    self, Callbacks, Change, FocusObservation, KeyDecision, KeyFacts, NodeHandle, Reason,
    SurfaceObservation,
};
use dioxus::prelude::*;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[path = "header_layout_dom.rs"]
mod native;
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

#[derive(Deserialize)]
struct Report {
    owner: String,
    generation: u64,
    sequence: u64,
    kind: String,
    #[serde(default)]
    viewport_width: f64,
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
    sequence: u64,
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
            sequence: 0,
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
        self.sequence = 0;
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
    fn accepts(&self, report: &Report) -> bool {
        self.alive
            && self.mounted
            && report.owner == self.owner
            && report.generation == self.generation
            && report.sequence > self.sequence
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
    fn observation(&self, event: FormEvent) {
        event.stop_propagation();
        let Ok(report) = serde_json::from_str::<Report>(&event.value()) else {
            return;
        };
        if report.kind != "layout" {
            return;
        }
        {
            let mut model = self.model.borrow_mut();
            if !model.accepts(&report) {
                return;
            }
            model.sequence = report.sequence;
            model.layout_epoch = model.layout_epoch.wrapping_add(1);
            if report.viewport_width.is_finite() && report.viewport_width > 0.0 {
                model.viewport_width = report.viewport_width;
            }
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

    let bridge = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
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
        }));
    let focus_controller = controller.clone();
    let focus_controls = controls.clone();
    use_effect(move || {
        let _ = revision();
        focus_controller.project_controls(&focus_controls.groups());
        focus_controller.dispatch_focus();
        focus_controller.measure_layout();
    });
    let mount_model = model.clone();
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
    let transport_id = format!("{owner}-transport");
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
    let small_popover = crate::native_popover::use_native_popover(
        small_id.clone(),
        small && open && enable_top_layer,
    );
    let large_popover = crate::native_popover::use_native_popover(
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
                let owner = serde_json::to_string(&model.owner).expect("Header owner serialization");
                drop(model);
                *bridge.borrow_mut() = Some(dioxus::document::eval(&native::BRIDGE.replace("__OWNER__",&owner)));
            },
            overlay_runtime::OverlayTransport {runtime}
            input {id:transport_id,r#type:"hidden","data-header-layout-observer":owner.clone(),
                oninput:move |event| controller.observation(event)
            }
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

    #[derive(Clone, Default)]
    struct FocusFixture(
        Rc<RefCell<Option<Controller>>>,
        Rc<std::cell::Cell<bool>>,
        Rc<RefCell<Vec<bool>>>,
    );
    impl PartialEq for FocusFixture {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.0, &other.0)
        }
    }
    fn registered_header(fixture: FocusFixture) -> Element {
        let runtime = overlay_runtime::use_overlay_runtime();
        let model = use_hook(|| {
            let mut model = Model::new(true, false);
            model.mounted = true;
            model.presence = [true; 4];
            Rc::new(RefCell::new(model))
        });
        let read = model.clone();
        let apply = model.clone();
        let veto = fixture.1.clone();
        let requests = fixture.2.clone();
        let node = runtime.register(
            model.borrow().owner.clone(),
            1,
            OverlayKind::Header,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(move || {
                    let model = read.borrow();
                    model.alive.then_some(model.open)
                }),
                before: Rc::new(move |change| {
                    requests.borrow_mut().push(change.open);
                    !veto.get()
                }),
                apply: Rc::new(move |change| {
                    let mut model = apply.borrow_mut();
                    model.open = change.open;
                    model.apply_focus(change.restore_focus);
                }),
                key: None,
            },
        );
        node.set_ownership(Vec::new(), true);
        node.set_mounted();
        let revision = use_signal(|| 0);
        *fixture.0.borrow_mut() = Some(Controller {
            model,
            revision,
            node,
        });
        rsx! { div {} }
    }
    #[derive(Clone, Default)]
    struct OwnershipFixture(Rc<OwnershipState>);
    #[derive(Default)]
    struct OwnershipState {
        small: std::cell::Cell<Option<Signal<bool>>>,
        groups: RefCell<std::collections::BTreeMap<String, Vec<NodeHandle>>>,
    }
    impl PartialEq for OwnershipFixture {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.0, &other.0)
        }
    }
    #[component]
    fn OwnershipProbe(group: String, fixture: OwnershipFixture) -> Element {
        let ancestors = overlay_runtime::use_overlay_ancestors();
        fixture.0.groups.borrow_mut().insert(group, ancestors);
        VNode::empty()
    }
    fn ownership_fixture(fixture: OwnershipFixture) -> Element {
        let small = use_signal(|| false);
        fixture.0.small.set(Some(small));
        rsx! {
            HeaderLayout {
                small: small(), discovery_ready: true, more_label: "More",
                secondary: rsx! { OwnershipProbe {group:"secondary",fixture:fixture.clone()} },
                default_content: rsx! { OwnershipProbe {group:"default",fixture:fixture.clone()} },
                overflow: rsx! { OwnershipProbe {group:"overflow",fixture:fixture.clone()} },
                avatar: rsx! { OwnershipProbe {group:"avatar",fixture} }
            }
        }
    }
    #[test]
    fn responsive_header_ownership_changes_without_relocating_slot_components() {
        let fixture = OwnershipFixture::default();
        let mut dom = VirtualDom::new_with_props(ownership_fixture, fixture.clone());
        dom.rebuild_in_place();
        let header = fixture.0.groups.borrow()["overflow"][0].clone();
        let metadata = header.surface_metadata().unwrap();
        assert_eq!(metadata.trigger_id, format!("{}-more", header.id()));
        assert_eq!(metadata.items_host, format!("{}-large", header.id()));
        assert!(metadata.popover_host.is_empty());
        for group in ["secondary", "default", "avatar"] {
            assert!(fixture.0.groups.borrow()[group].is_empty());
        }
        dom.in_scope(ScopeId::ROOT, || fixture.0.small.get().unwrap().set(true));
        settle_focus(&mut dom);
        for group in ["secondary", "default", "overflow"] {
            assert!(fixture.0.groups.borrow()[group] == [header.clone()]);
        }
        assert!(fixture.0.groups.borrow()["avatar"].is_empty());
        assert_eq!(
            header.surface_metadata().unwrap().items_host,
            format!("{}-small", header.id())
        );
        dom.in_scope(ScopeId::ROOT, || fixture.0.small.get().unwrap().set(false));
        settle_focus(&mut dom);
        for group in ["secondary", "default", "avatar"] {
            assert!(fixture.0.groups.borrow()[group].is_empty());
        }
        assert_eq!(
            header.surface_metadata().unwrap().items_host,
            format!("{}-large", header.id())
        );
        assert!(fixture.0.groups.borrow()["overflow"] == [header]);
    }

    #[derive(Clone)]
    struct FocusMount {
        id: String,
        calls: Rc<RefCell<Vec<String>>>,
    }
    impl dioxus::html::RenderedElementBacking for FocusMount {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn set_focus(
            &self,
            focus: bool,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>>
        {
            assert!(focus);
            self.calls.borrow_mut().push(self.id.clone());
            Box::pin(async { Ok(()) })
        }
    }
    fn install_focus(
        dom: &VirtualDom,
        controller: &Controller,
        id: &str,
        calls: &Rc<RefCell<Vec<String>>>,
    ) {
        let mount = Rc::new(MountedData::new(FocusMount {
            id: id.into(),
            calls: calls.clone(),
        }));
        dom.in_runtime(|| {
            controller
                .node
                .runtime()
                .unwrap()
                .register_item_mount(id, id, mount)
        });
    }
    fn settle_focus(dom: &mut VirtualDom) {
        for _ in 0..4 {
            dom.render_immediate_to_vec();
        }
    }

    #[test]
    fn shared_pointer_cancels_pending_header_focus_without_repeated_renders() {
        let fixture = FocusFixture::default();
        let mut dom = VirtualDom::new_with_props(registered_header, fixture.clone());
        dom.rebuild_in_place();
        let controller = fixture.0.borrow().clone().unwrap();
        let calls = Rc::new(RefCell::new(Vec::new()));
        install_focus(&dom, &controller, "registered-field", &calls);
        {
            let mut model = controller.model.borrow_mut();
            model.open = true;
            model.controls = controls(&["registered-field"], None);
            model.focus("first".into());
        }
        dom.in_runtime(|| controller.dispatch_focus());
        let runtime = controller.node.runtime().unwrap();
        assert_eq!(runtime.pending_item_focus_count(), 1);
        {
            let target = controller
                .node
                .capture_registered_focus("registered-field")
                .unwrap();
            let mut model = controller.model.borrow_mut();
            model.preserve_focus(FocusedControl {
                id: "registered-field".into(),
                group: 0,
            });
            model.preserve_target = Some(target);
        }
        let revision = *controller.revision.peek();
        dom.in_runtime(|| controller.pointer_input());
        assert_eq!(runtime.pending_item_focus_count(), 0);
        {
            let model = controller.model.borrow();
            assert_eq!(model.focus, "none");
            assert!(model.preserve.is_none() && model.preserve_target.is_none());
            assert!(model.open, "pointer input alone never closes the surface");
        }
        assert_eq!(*controller.revision.peek(), revision + 1);
        dom.in_runtime(|| controller.pointer_input());
        assert_eq!(
            *controller.revision.peek(),
            revision + 1,
            "ordinary pointer input does not request a render without pending Header intent"
        );
        settle_focus(&mut dom);
        assert!(
            calls.borrow().is_empty(),
            "the canceled job cannot steal native pointer focus"
        );
        assert!(
            fixture.2.borrow().is_empty(),
            "no synthetic close request or veto bypass"
        );
    }

    #[test]
    fn registered_header_opening_edges_and_escape_return_use_current_mounted_controls() {
        let fixture = FocusFixture::default();
        let mut dom = VirtualDom::new_with_props(registered_header, fixture.clone());
        dom.rebuild_in_place();
        let controller = fixture.0.borrow().clone().unwrap();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let more = format!("{}-more", controller.model.borrow().owner);
        for id in ["registered-field", "registered-action", more.as_str()] {
            install_focus(&dom, &controller, id, &calls);
        }
        {
            let mut model = controller.model.borrow_mut();
            model.open = true;
            model.focus("first".into());
        }
        dom.in_runtime(|| controller.dispatch_focus());
        settle_focus(&mut dom);
        assert!(
            calls.borrow().is_empty(),
            "opening waits for visible candidate facts"
        );
        controller.model.borrow_mut().controls = controls(
            &["", "unregistered", "registered-field", "registered-action"],
            None,
        );
        dom.in_runtime(|| controller.dispatch_focus());
        settle_focus(&mut dom);
        controller.model.borrow_mut().focus("last".into());
        dom.in_runtime(|| controller.dispatch_focus());
        settle_focus(&mut dom);
        {
            let mut model = controller.model.borrow_mut();
            model.open = false;
            model.apply_focus(true);
        }
        dom.in_runtime(|| controller.dispatch_focus());
        settle_focus(&mut dom);
        assert_eq!(
            calls.borrow().as_slice(),
            ["registered-field", "registered-action", more.as_str()]
        );
    }

    #[test]
    fn header_tab_wrap_requires_a_registered_target_and_keeps_interior_tab_native() {
        let fixture = FocusFixture::default();
        let mut dom = VirtualDom::new_with_props(registered_header, fixture.clone());
        dom.rebuild_in_place();
        let controller = fixture.0.borrow().clone().unwrap();
        let calls = Rc::new(RefCell::new(Vec::new()));
        install_focus(&dom, &controller, "registered-action", &calls);
        {
            let mut model = controller.model.borrow_mut();
            model.open = true;
            model.controls = controls(&["unregistered", "registered-action"], None);
        }
        let mut fact = key_input(&controller.model.borrow());
        fact.key = "Tab".into();
        fact.focus_count = 2;
        fact.focused_index = Some(1);
        let mut decision = decide(&mut controller.model.borrow_mut(), &fact, false);
        dom.in_runtime(|| controller.filter_tab_focus(&fact, &mut decision));
        assert!(!decision.prevent && !decision.stop && decision.focus.is_none());
        fact.shift = true;
        fact.focused_index = Some(0);
        let mut decision = decide(&mut controller.model.borrow_mut(), &fact, false);
        dom.in_runtime(|| controller.filter_tab_focus(&fact, &mut decision));
        assert!(decision.prevent && decision.stop);
        controller.model.borrow_mut().focus(decision.focus.unwrap());
        dom.in_runtime(|| controller.dispatch_focus());
        settle_focus(&mut dom);
        assert_eq!(calls.borrow().as_slice(), ["registered-action"]);
        fact.shift = false;
        fact.focused_index = Some(0);
        assert!(!decide(&mut controller.model.borrow_mut(), &fact, false).prevent);
    }

    #[test]
    fn responsive_focus_keeps_registered_visible_control_and_rejects_replacement_or_outside_focus()
    {
        let fixture = FocusFixture::default();
        let mut dom = VirtualDom::new_with_props(registered_header, fixture.clone());
        dom.rebuild_in_place();
        let controller = fixture.0.borrow().clone().unwrap();
        let calls = Rc::new(RefCell::new(Vec::new()));
        install_focus(&dom, &controller, "registered-field", &calls);
        let captured = controller
            .node
            .capture_registered_focus("registered-field")
            .unwrap();
        {
            let mut model = controller.model.borrow_mut();
            model.open = true;
            model.small = false;
            model.controls = vec![control("registered-field", 1, 0, true, false)];
            assert!(!model.sync_mode(true, false));
            model.preserve_target = Some(captured.clone());
            model.focus_generation = model.generation;
            model.controls = vec![control("registered-field", 1, 0, true, true)];
        }
        dom.in_runtime(|| controller.dispatch_focus());
        settle_focus(&mut dom);
        assert_eq!(calls.borrow().as_slice(), ["registered-field"]);
        controller.model.borrow_mut().preserve_focus(capture(1));
        controller.model.borrow_mut().preserve_target = Some(captured.clone());
        install_focus(&dom, &controller, "registered-field", &calls);
        dom.in_runtime(|| controller.dispatch_focus());
        settle_focus(&mut dom);
        assert_eq!(
            calls.borrow().len(),
            1,
            "replacement cannot inherit an old snapshot"
        );
        let current = controller
            .node
            .capture_registered_focus("registered-field")
            .unwrap();
        {
            let mut model = controller.model.borrow_mut();
            model.preserve_focus(capture(1));
            model.preserve_target = Some(current);
            for control in &mut model.controls {
                control.active = false;
            }
            model.active_default = false;
        }
        dom.in_runtime(|| controller.dispatch_focus());
        settle_focus(&mut dom);
        assert_eq!(
            calls.borrow().len(),
            1,
            "outside focus is not stolen by mode preservation"
        );
        let current = controller
            .node
            .capture_registered_focus("registered-field")
            .unwrap();
        {
            let mut model = controller.model.borrow_mut();
            model.preserve_focus(capture(1));
            model.preserve_target = Some(current);
            model.active_default = true;
            model.controls.clear();
        }
        dom.in_runtime(|| controller.dispatch_focus());
        settle_focus(&mut dom);
        assert_eq!(
            calls.borrow().len(),
            1,
            "a hidden registered control is not revived"
        );
        controller.model.borrow_mut().invalidate();
        dom.in_runtime(|| controller.dispatch_focus());
        settle_focus(&mut dom);
        assert_eq!(calls.borrow().len(), 1);
    }

    fn control(
        id: &str,
        group: u8,
        tab_index: i32,
        active: bool,
        in_popup: bool,
    ) -> PresentedControl {
        PresentedControl {
            id: id.into(),
            group: Some(group),
            tab_index,
            active,
            in_popup,
        }
    }
    fn controls(ids: &[&str], active: Option<usize>) -> Vec<PresentedControl> {
        ids.iter()
            .enumerate()
            .map(|(index, id)| control(id, 2, 0, active == Some(index), true))
            .collect()
    }
    #[test]
    fn canonical_controls_derive_dom_order_boundaries_and_current_focus() {
        let mut model = Model::new(true, false);
        model.controls = vec![
            control("more", 4, 0, false, false),
            control("", 2, 0, false, true),
            control("field", 2, 0, false, true),
            control("positive", 2, 3, true, true),
            control("negative", 1, -1, false, true),
        ];
        assert_eq!(
            model.popup_ids(),
            ["", "field", "positive"],
            "Header keeps its existing DOM-order policy"
        );
        assert_eq!(
            model.popup_controls().count(),
            3,
            "unidentified controls retain their boundary position"
        );
        assert_eq!(
            model.popup_controls().position(|control| control.active),
            Some(2)
        );
        assert_eq!(
            model.focused_control(),
            Some(FocusedControl {
                id: "positive".into(),
                group: 2
            })
        );
        assert!(
            model.presented("negative"),
            "negative tabindex can still preserve a current mounted control"
        );
        assert!(
            !model.presented(""),
            "an unidentified control is never a registered destination"
        );
        for (index, control) in model.controls.iter_mut().enumerate() {
            control.active = index == 4;
        }
        assert_eq!(
            model.popup_controls().position(|control| control.active),
            None
        );
        assert_eq!(
            model.focused_control(),
            Some(FocusedControl {
                id: "negative".into(),
                group: 1
            })
        );
        for (index, control) in model.controls.iter_mut().enumerate() {
            control.active = index == 1;
        }
        assert!(model.focused_control().is_none());
    }
    #[test]
    fn canonical_focus_projection_requires_current_group_registration_and_exact_focus() {
        let candidate =
            |id: &str, hosts: &[&str], active: bool| crate::popover_focus::NativeFocusCandidate {
                target: id.into(),
                tab_index: 0,
                hosts: hosts.iter().map(|host| (*host).into()).collect(),
                nearest_host: String::new(),
                popover: String::new(),
                exact_active: active,
                contains_active: !active,
            };
        let observation = FocusObservation {
            candidates: vec![
                candidate("outside", &["header"], false),
                candidate("field", &["header", "nested"], true),
                candidate("inline", &[], false),
                candidate("header-more", &[], false),
            ],
            active_id: "field".into(),
            active_default: false,
        };
        let groups = BTreeMap::from([("inline".into(), 1), ("field".into(), 2)]);
        let projected = presented_controls(&observation, &groups, "header");
        assert_eq!(
            projected
                .iter()
                .map(|control| control.id.as_str())
                .collect::<Vec<_>>(),
            ["field", "inline", "header-more"]
        );
        assert!(projected[0].active && projected[0].in_popup);
        assert_eq!(projected[0].group, Some(2));
        assert!(!projected[1].active && !projected[1].in_popup);
        assert_eq!(projected[2].group, Some(4));
        let mut retired = groups;
        retired.remove("field");
        assert!(
            !presented_controls(&observation, &retired, "header")
                .iter()
                .any(|control| control.id == "field")
        );
    }

    fn report(model: &Model, kind: &str) -> Report {
        Report {
            owner: model.owner.clone(),
            generation: model.generation,
            sequence: model.sequence + 1,
            kind: kind.into(),
            viewport_width: 1000.0,
        }
    }
    fn key_input(_: &Model) -> KeyInput {
        KeyInput {
            focus_count: 6,
            focused_index: Some(0),
            inside: true,
            ..KeyInput::default()
        }
    }
    #[test]
    fn mounted_popup_placement_clamps_and_rejects_obsolete_layout_work() {
        assert_eq!(
            popup_position(1000.0, 64.0, 48.0, 240.0),
            Some(PopupPosition {
                left: 64.0,
                top: 48.0
            })
        );
        assert_eq!(
            popup_position(360.0, 330.0, 48.0, 200.0),
            Some(PopupPosition {
                left: 152.0,
                top: 48.0
            })
        );
        assert_eq!(
            popup_position(360.0, -20.0, 48.0, 344.0),
            Some(PopupPosition {
                left: 8.0,
                top: 48.0
            })
        );
        assert!(
            popup_position(360.0, 16.0, 48.0, 0.0).is_none(),
            "hidden native surface waits for its visible resize"
        );
        assert!(popup_position(f64::NAN, 16.0, 48.0, 200.0).is_none());
        let mut model = Model::new(true, true);
        model.mounted = true;
        model.open = true;
        model.viewport_width = 360.0;
        let epoch = model.layout_epoch;
        let generation = model.generation;
        assert!(model.accepts_layout(epoch, generation, true));
        model.position = popup_position(360.0, 330.0, 48.0, 200.0);
        assert!(
            model
                .popup_style(true)
                .unwrap()
                .contains("max-width:344px;left:152px;top:48px;")
        );
        assert!(
            model.popup_style(false).is_none(),
            "inactive host retains no geometry writes"
        );
        model.invalidate_layout();
        assert!(!model.accepts_layout(epoch, generation, true));
        let epoch = model.layout_epoch;
        model.sync_mode(false, true);
        assert!(
            !model.accepts_layout(epoch, generation, true),
            "responsive replacement invalidates pending measurement"
        );
        let epoch = model.layout_epoch;
        let generation = model.generation;
        model.open = false;
        assert!(!model.accepts_layout(epoch, generation, false));
        assert!(model.popup_style(false).is_none());
        model.open = true;
        model.invalidate();
        assert!(
            !model.accepts_layout(epoch, generation, false),
            "unmounted owner cannot publish geometry"
        );
    }

    #[test]
    fn decision_preserves_opening_defaults_veto_intent_and_boundary_keys() {
        let mut model = Model::new(true, false);
        model.mounted = true;
        model.presence = [true; 4];
        for key in ["Home", "End", "ArrowUp", "ArrowDown", "Enter", " "] {
            let mut fact = key_input(&model);
            fact.more = true;
            fact.key = key.into();
            let decision = decide(&mut model, &fact, false);
            assert!(decision.prevent);
            assert_eq!(decision.request, Some(true));
            assert!(!decision.stop);
        }
        model.open = true;
        let mut trigger = key_input(&model);
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
        let mut fact = key_input(&model);
        fact.key = "Escape".into();
        let decision = decide(&mut model, &fact, false);
        assert!(decision.prevent && decision.stop);
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
                    let mut fact = key_input(&model);
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
        let mut ordinary = key_input(&model);
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
        model.focus("last".into());
        let previous = report(&model, "layout");
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
        assert_eq!(model.focus, "none");
        assert!(!model.sync_mode(true, true), "same props are inert");
        assert!(model.open);
        assert!(
            model.sync_mode(true, false),
            "top-layer configuration remains separate"
        );
        assert!(!model.open);
    }

    fn capture(group: u8) -> FocusedControl {
        FocusedControl {
            id: "registered-field".into(),
            group,
        }
    }

    fn pending_capture(group: u8) -> Model {
        let mut model = Model::new(false, true);
        model.mounted = true;
        model.open = true;
        model.presence = [true; 4];
        model.controls = vec![control("registered-field", group, 0, true, false)];
        assert!(!model.sync_mode(true, true));
        model
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
        closed.controls = vec![control("registered-field", 1, 0, true, false)];
        assert!(!closed.sync_mode(true, true));
        assert_eq!(closed.focus, "none", "hidden Small content is not revived");
        assert!(closed.preserve.is_none());
    }

    #[test]
    fn accepted_close_retains_only_visible_pending_mode_capture() {
        let mut large_default = pending_capture(1);
        large_default.small = false;
        large_default.open = false;
        large_default.apply_focus(false);
        assert_eq!(large_default.preserve, Some(capture(1)));

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

        let mut hidden_more = pending_capture(1);
        hidden_more.small = false;
        hidden_more.presence[2] = false;
        hidden_more.open = false;
        assert!(!hidden_more.more_visible());
        hidden_more.apply_focus(true);
        assert_eq!(hidden_more.preserve, Some(capture(1)));
    }

    #[test]
    fn rust_presence_and_owner_reconciliation_reject_stale_native_work() {
        let mut model = Model::new(false, false);
        model.mounted = true;
        model.presence = [true, true, false, true];
        assert!(!model.more_visible());
        model.small = true;
        assert!(model.more_visible());
        let mut fact = report(&model, "layout");
        assert!(model.accepts(&fact));
        fact.owner.push('x');
        assert!(!model.accepts(&fact));
        fact = report(&model, "layout");
        model.sequence = fact.sequence;
        assert!(!model.accepts(&fact));
        fact = report(&model, "layout");
        model.open = true;
        assert!(model.sync_mode(false, true));
        assert!(!model.open);
        assert!(!model.accepts(&fact));
        let fact = report(&model, "layout");
        model.invalidate();
        assert!(!model.accepts(&fact));
    }

    struct TypedKey(Key, Modifiers);
    impl dioxus::html::ModifiersInteraction for TypedKey {
        fn modifiers(&self) -> Modifiers {
            self.1
        }
    }
    impl dioxus::html::HasKeyboardData for TypedKey {
        fn key(&self) -> Key {
            self.0.clone()
        }
        fn code(&self) -> dioxus::html::Code {
            dioxus::html::Code::Unidentified
        }
        fn location(&self) -> dioxus::html::Location {
            dioxus::html::Location::Standard
        }
        fn is_auto_repeating(&self) -> bool {
            false
        }
        fn is_composing(&self) -> bool {
            false
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    fn key_event(key: Key, shift: bool) -> KeyboardEvent {
        Event::new(
            Rc::new(dioxus::html::KeyboardData::new(TypedKey(
                key,
                if shift {
                    Modifiers::SHIFT
                } else {
                    Modifiers::empty()
                },
            ))),
            true,
        )
    }
    #[test]
    fn direct_more_keys_and_native_field_boundaries_use_registered_focus() {
        let fixture = FocusFixture::default();
        let mut dom = VirtualDom::new_with_props(registered_header, fixture.clone());
        dom.rebuild_in_place();
        let controller = fixture.0.borrow().clone().unwrap();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let more = format!("{}-more", controller.model.borrow().owner);
        for id in ["first-field", "last-action", more.as_str()] {
            install_focus(&dom, &controller, id, &calls);
        }
        {
            let mut model = controller.model.borrow_mut();
            model.controls = controls(&["first-field", "last-action"], Some(0));
        }
        let down = key_event(Key::ArrowDown, false);
        dom.in_runtime(|| controller.key(down.clone(), true, None));
        settle_focus(&mut dom);
        assert!(
            controller.model.borrow().open && !down.default_action_enabled() && !down.propagates()
        );
        assert_eq!(calls.borrow().as_slice(), ["first-field"]);
        let home = key_event(Key::Home, false);
        dom.in_runtime(|| controller.key(home.clone(), false, Some(true)));
        assert!(
            home.default_action_enabled() && home.propagates(),
            "native editing remains native"
        );
        let interior = key_event(Key::Tab, false);
        dom.in_runtime(|| controller.key(interior.clone(), false, Some(true)));
        assert!(interior.default_action_enabled() && interior.propagates());
        for (index, control) in controller
            .model
            .borrow_mut()
            .controls
            .iter_mut()
            .enumerate()
        {
            control.active = index == 1;
        }
        let boundary = key_event(Key::Tab, false);
        dom.in_runtime(|| controller.key(boundary.clone(), false, Some(true)));
        settle_focus(&mut dom);
        assert!(!boundary.default_action_enabled() && !boundary.propagates());
        assert_eq!(calls.borrow().as_slice(), ["first-field", "first-field"]);
        let escape = key_event(Key::Escape, false);
        dom.in_runtime(|| controller.key(escape.clone(), false, Some(true)));
        settle_focus(&mut dom);
        assert!(
            !controller.model.borrow().open
                && !escape.default_action_enabled()
                && !escape.propagates()
        );
        assert_eq!(
            calls.borrow().as_slice(),
            ["first-field", "first-field", more.as_str()]
        );
        fixture.1.set(true);
        let end = key_event(Key::End, false);
        dom.in_runtime(|| controller.key(end.clone(), true, None));
        settle_focus(&mut dom);
        assert!(!controller.model.borrow().open && !end.default_action_enabled());
        assert_eq!(calls.borrow().len(), 3, "veto never moves focus");
        assert_eq!(*fixture.2.borrow(), [true, false, true]);
    }

    struct Click;
    impl dioxus::html::InteractionLocation for Click {
        fn client_coordinates(&self) -> dioxus::html::geometry::ClientPoint {
            Default::default()
        }
        fn page_coordinates(&self) -> dioxus::html::geometry::PagePoint {
            Default::default()
        }
        fn screen_coordinates(&self) -> dioxus::html::geometry::ScreenPoint {
            Default::default()
        }
    }
    impl dioxus::html::InteractionElementOffset for Click {
        fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
            Default::default()
        }
    }
    impl dioxus::html::ModifiersInteraction for Click {
        fn modifiers(&self) -> Modifiers {
            Modifiers::empty()
        }
    }
    impl dioxus::html::PointerInteraction for Click {
        fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
            None
        }
        fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
            Default::default()
        }
    }
    impl dioxus::html::HasMouseData for Click {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    fn click() -> MouseEvent {
        Event::new(Rc::new(dioxus::html::MouseData::new(Click)), true)
    }

    #[test]
    fn more_click_toggles_current_owner_and_dismisses_peers_independently_of_veto() {
        let fixture = FocusFixture::default();
        let mut dom = VirtualDom::new_with_props(registered_header, fixture.clone());
        dom.rebuild_in_place();
        let controller = fixture.0.borrow().clone().unwrap();
        let peer = Rc::new(std::cell::Cell::new(true));
        dom.in_scope(ScopeId::ROOT, || {
            let read = peer.clone();
            let apply = peer.clone();
            let peer_node = controller.node.runtime().unwrap().register(
                "click-peer".into(),
                1,
                OverlayKind::Dropdown,
                CloseBehavior::Both,
                Callbacks {
                    read: Rc::new(move || Some(read.get())),
                    before: Rc::new(|_| true),
                    apply: Rc::new(move |change| apply.set(change.open)),
                    key: None,
                },
            );
            peer_node.set_ownership(Vec::new(), true);
            peer_node.set_mounted();
        });
        let event = click();
        dom.in_runtime(|| controller.more_click(event.clone()));
        assert!(controller.model.borrow().open);
        assert!(!peer.get());
        assert!(
            event.default_action_enabled() && !event.propagates(),
            "only virtual ancestors are stopped"
        );
        assert_eq!(*fixture.2.borrow(), [true]);
        let event = click();
        dom.in_runtime(|| controller.more_click(event));
        assert!(!controller.model.borrow().open);
        fixture.1.set(true);
        peer.set(true);
        let vetoed = click();
        dom.in_runtime(|| controller.more_click(vetoed.clone()));
        assert!(!controller.model.borrow().open);
        assert!(
            !peer.get(),
            "peer dismissal remains independent of the local veto"
        );
        assert!(vetoed.default_action_enabled());
        assert_eq!(*fixture.2.borrow(), [true, false, true]);
        fixture.1.set(false);
        let prevented = click();
        prevented.prevent_default();
        dom.in_runtime(|| controller.more_click(prevented));
        controller.model.borrow_mut().presence = [false; 4];
        dom.in_runtime(|| controller.more_click(click()));
        assert_eq!(
            fixture.2.borrow().len(),
            3,
            "prevented and invisible controls cannot toggle"
        );
        controller.model.borrow_mut().presence = [true; 4];
        dom.in_runtime(|| controller.node.unregister());
        dom.in_runtime(|| controller.more_click(click()));
        assert_eq!(fixture.2.borrow().len(), 3, "a stale owner cannot act");
    }

    #[test]
    fn active_group_clicks_dismiss_without_canceling_form_defaults_and_inline_groups_stay_idle() {
        let fixture = FocusFixture::default();
        let mut dom = VirtualDom::new_with_props(registered_header, fixture.clone());
        dom.rebuild_in_place();
        let controller = fixture.0.borrow().clone().unwrap();
        {
            let mut model = controller.model.borrow_mut();
            model.open = true;
            model.small = false;
        }
        let inline = click();
        dom.in_runtime(|| controller.group_click(1, inline.clone()));
        assert!(
            controller.model.borrow().open
                && inline.propagates()
                && inline.default_action_enabled()
        );
        let inactive = click();
        dom.in_runtime(|| controller.background_click(true, inactive.clone()));
        assert!(inactive.propagates());
        let background = click();
        dom.in_runtime(|| controller.background_click(false, background.clone()));
        assert!(
            controller.model.borrow().open
                && background.default_action_enabled()
                && !background.propagates()
        );
        let content = click();
        dom.in_runtime(|| controller.group_click(2, content.clone()));
        assert!(!controller.model.borrow().open);
        assert!(
            content.default_action_enabled() && !content.propagates(),
            "native input/form defaults remain enabled"
        );
        {
            let mut model = controller.model.borrow_mut();
            model.open = true;
            model.small = true;
        }
        let prevented = click();
        prevented.prevent_default();
        dom.in_runtime(|| controller.group_click(0, prevented.clone()));
        assert!(controller.model.borrow().open && !prevented.propagates());
        fixture.1.set(true);
        let vetoed = click();
        dom.in_runtime(|| controller.group_click(1, vetoed.clone()));
        assert!(controller.model.borrow().open && vetoed.default_action_enabled());
    }
}
