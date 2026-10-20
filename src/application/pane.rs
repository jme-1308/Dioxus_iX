/// Side pane presentation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaneVariant {
    #[default]
    Inline,
    Floating,
}

/// Side of the content occupied by a pane.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaneComposition {
    Left,
    Right,
    #[default]
    Top,
    Bottom,
}

impl PaneComposition {
    fn class(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }
}

/// Determines which pair of panes spans the full available cross axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaneLayoutOrder {
    #[default]
    FullVertical,
    FullHorizontal,
}

#[derive(Clone, Copy, PartialEq)]
struct PaneSettings {
    composition: PaneComposition,
    variant: PaneVariant,
    borderless: bool,
}

#[derive(Clone, Copy)]
struct PaneContext {
    settings: Memo<PaneSettings>,
}

#[derive(Clone, Copy)]
struct PaneLayoutContext {
    close_requested: Signal<u64>,
    dimensions: Signal<Option<(f64, f64)>>,
}

/// Cancelable expansion request. Vetoing it preserves both the current pane
/// state and the post-change notification. `slot` is empty outside a layout.
#[derive(Clone)]
pub struct PaneExpandedChangeEvent {
    pub slot: String,
    pub expanded: bool,
    prevented: Rc<Cell<bool>>,
}

impl PaneExpandedChangeEvent {
    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }
    pub fn default_prevented(&self) -> bool {
        self.prevented.get()
    }
}

fn request_pane_expansion(
    current: bool,
    next: bool,
    slot: String,
    before: EventHandler<PaneExpandedChangeEvent>,
) -> bool {
    if current == next {
        return false;
    }
    let event = PaneExpandedChangeEvent {
        slot,
        expanded: next,
        prevented: Rc::new(Cell::new(false)),
    };
    before.call(event.clone());
    !event.default_prevented()
}

#[component]
fn PaneSlot(
    children: Element,
    composition: PaneComposition,
    variant: PaneVariant,
    borderless: bool,
) -> Element {
    let settings = use_memo(use_reactive(
        (&composition, &variant, &borderless),
        |(composition, variant, borderless)| PaneSettings {
            composition,
            variant,
            borderless,
        },
    ));
    use_context_provider(|| PaneContext { settings });
    rsx! { div { class: "ix-pane-slot ix-pane-slot-{composition.class()}", {children} } }
}

struct PaneLayoutMeasureModel {
    alive: bool,
    epoch: u64,
    root: Option<Rc<MountedData>>,
}
#[derive(Clone)]
struct PaneLayoutMeasure {
    model: Rc<RefCell<PaneLayoutMeasureModel>>,
    dimensions: Signal<Option<(f64, f64)>>,
}
impl PaneLayoutMeasure {
    fn mount(&self, root: Rc<MountedData>) {
        {
            let mut model = self.model.borrow_mut();
            if !model.alive {
                return;
            }
            model.root = Some(root);
        }
        self.measure();
    }
    fn measure(&self) {
        let snapshot = {
            let mut model = self.model.borrow_mut();
            if !model.alive {
                return;
            }
            model.epoch = model.epoch.wrapping_add(1);
            model.root.clone().map(|root| (model.epoch, root))
        };
        let Some((epoch, root)) = snapshot else {
            return;
        };
        let controller = self.clone();
        spawn(async move {
            let Ok(rect) = root.get_client_rect().await else {
                return;
            };
            let dimensions = (rect.width(), rect.height());
            let model = controller.model.borrow();
            if !model.alive
                || model.epoch != epoch
                || !model
                    .root
                    .as_ref()
                    .is_some_and(|current| Rc::ptr_eq(current, &root))
                || !valid_pane_dimensions(dimensions)
            {
                return;
            }
            let mut value = controller.dimensions;
            if *value.peek() != Some(dimensions) {
                value.set(Some(dimensions));
            }
        });
    }
    fn dispose(&self) {
        let mut model = self.model.borrow_mut();
        model.alive = false;
        model.epoch = model.epoch.wrapping_add(1);
        model.root = None;
    }
}
fn valid_pane_dimensions((width, height): (f64, f64)) -> bool {
    width.is_finite() && height.is_finite() && width >= 0.0 && height >= 0.0
}
fn pane_available_dimensions(
    explicit: Option<(f64, f64)>,
    layout: Option<(f64, f64)>,
    viewport: Option<(f64, f64)>,
) -> (f64, f64) {
    explicit
        .filter(|size| valid_pane_dimensions(*size))
        .or(layout)
        .or(viewport)
        .unwrap_or((0.0, 0.0))
}
#[cfg(test)]
#[derive(Clone, Default)]
struct PaneLayoutMeasureCapture(Rc<RefCell<Option<PaneLayoutMeasure>>>);

/// Layout with independently collapsible left, right, top and bottom pane slots.
#[component]
pub fn PaneLayout(
    children: Element,
    #[props(default)] left: Option<Element>,
    #[props(default)] right: Option<Element>,
    #[props(default)] top: Option<Element>,
    #[props(default)] bottom: Option<Element>,
    #[props(default)] layout: PaneLayoutOrder,
    #[props(default)] variant: PaneVariant,
    #[props(default)] borderless: bool,
) -> Element {
    let mut close_requested = use_signal(|| 0_u64);
    let dimensions = use_signal(|| None::<(f64, f64)>);
    let measurement = use_hook(|| PaneLayoutMeasure {
        model: Rc::new(RefCell::new(PaneLayoutMeasureModel {
            alive: true,
            epoch: 0,
            root: None,
        })),
        dimensions,
    });
    #[cfg(test)]
    if let Some(capture) = try_consume_context::<PaneLayoutMeasureCapture>() {
        *capture.0.borrow_mut() = Some(measurement.clone());
    }
    let cleanup = measurement.clone();
    use_drop(move || cleanup.dispose());
    use_context_provider(|| PaneLayoutContext {
        close_requested,
        dimensions,
    });
    let resized = measurement.clone();
    rsx! {
        div { class: "ix-pane-layout", "data-layout": if layout == PaneLayoutOrder::FullVertical { "full-vertical" } else { "full-horizontal" },
            onmounted:move |event:MountedEvent|measurement.mount(event.data()),
            onresize:move |_:ResizeEvent|resized.measure(),
            if let Some(left) = left { PaneSlot { composition: PaneComposition::Left, variant, borderless, {left} } }
            if let Some(right) = right { PaneSlot { composition: PaneComposition::Right, variant, borderless, {right} } }
            if let Some(top) = top { PaneSlot { composition: PaneComposition::Top, variant, borderless, {top} } }
            if let Some(bottom) = bottom { PaneSlot { composition: PaneComposition::Bottom, variant, borderless, {bottom} } }
            div { class: "ix-pane-layout-content", onclick: move |_| { let next = close_requested().wrapping_add(1); close_requested.set(next); }, {children} }
        }
    }
}

/// Standard controls register their actual Dioxus mounted focus destination.
/// This composition marker carries no native registry or overlay hierarchy.
#[derive(Clone, Copy)]
pub(crate) struct PaneFocusScope;

#[derive(Default, PartialEq, Eq)]
struct PaneFocusFacts {
    ids: Vec<String>,
    autofocus: Vec<String>,
    active: Option<String>,
    inside: bool,
    active_default: bool,
    nested: bool,
    return_eligible: bool,
}

#[derive(Clone, Copy)]
enum PaneFocusIntent {
    Open,
    Return,
}
struct PaneFocusModel {
    alive: bool,
    expanded: bool,
    floating: bool,
    epoch: u64,
    facts: PaneFocusFacts,
    intent: Option<PaneFocusIntent>,
    invoker: Option<(String, crate::overlay_runtime::MountedFocusTarget)>,
    task: Option<dioxus::core::Task>,
}
#[derive(Clone)]
struct PaneFocusController(Rc<PaneFocusOwner>);
struct PaneFocusOwner {
    runtime: crate::overlay_runtime::RuntimeHandle,
    root_id: String,
    registration: Cell<u64>,
    model: Rc<RefCell<PaneFocusModel>>,
    revision: Signal<u64>,
}
impl std::ops::Deref for PaneFocusController {
    type Target = PaneFocusOwner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[cfg(test)]
#[derive(Clone, Default)]
struct PaneFocusCapture(Rc<RefCell<Option<PaneFocusController>>>);
impl PaneFocusController {
    fn new(runtime: crate::overlay_runtime::RuntimeHandle, root_id: String) -> Self {
        let controller = Self(Rc::new(PaneFocusOwner {
            runtime,
            root_id,
            registration: Cell::new(0),
            revision: Signal::new(0),
            model: Rc::new(RefCell::new(PaneFocusModel {
                alive: true,
                expanded: false,
                floating: false,
                epoch: 0,
                facts: PaneFocusFacts::default(),
                intent: None,
                invoker: None,
                task: None,
            })),
        }));
        let weak = Rc::downgrade(&controller.0);
        let generation = controller.runtime.register_focus_observer(
            controller.root_id.clone(),
            Rc::new(move |event| {
                if let Some(owner) = weak.upgrade() {
                    PaneFocusController(owner).observe(event);
                }
            }),
        );
        controller.registration.set(generation);
        controller
    }
    fn transition(&self, expanded: bool, floating: bool) {
        let pending = {
            let mut model = self.model.borrow_mut();
            if !model.alive || (model.expanded == expanded && model.floating == floating) {
                return;
            }
            let opening = expanded && floating && !(model.expanded && model.floating);
            let closing = !expanded && model.expanded && model.floating;
            model.expanded = expanded;
            model.floating = floating;
            model.epoch = model.epoch.wrapping_add(1);
            model.intent = if opening {
                Some(PaneFocusIntent::Open)
            } else if closing {
                Some(PaneFocusIntent::Return)
            } else {
                None
            };
            if opening {
                model.invoker = None;
            }
            model.task.take()
        };
        if let Some(task) = pending {
            task.cancel();
        }
    }
    fn refresh(&self) {
        let model = self.model.borrow();
        if model.alive {
            let _ = self.runtime.request_scoped_focus(
                &self.root_id,
                self.registration.get(),
                model.epoch,
            );
        }
    }
    fn current(&self) -> Option<String> {
        let _ = (self.revision)();
        self.model.borrow().facts.active.clone()
    }
    fn observe(&self, event: crate::overlay_runtime::ScopedFocusEvent) {
        use crate::overlay_runtime::ScopedFocusEvent;
        let (request, shared) = match event {
            ScopedFocusEvent::Observe(facts) => (None, facts),
            ScopedFocusEvent::Requested { request, facts } => (Some(request), facts),
        };
        let (intent, changed) = {
            let mut model = self.model.borrow_mut();
            if !model.alive || request.is_some_and(|epoch| model.epoch != epoch) {
                return;
            }
            let candidates = &shared.observation.candidates;
            let ids = crate::popover_focus::tab_order_targets(
                &crate::popover_focus::project_tab_candidates(candidates, |candidate| {
                    candidate.hosts.contains(&self.root_id)
                }),
            );
            let facts = PaneFocusFacts {
                autofocus: ids.iter().filter(|id| candidates.iter().any(|candidate| {
                    candidate.target == **id && candidate.autofocus
                })).cloned().collect(),
                ids,
                active: (!shared.observation.active_id.is_empty())
                    .then_some(shared.observation.active_id),
                inside: shared.active_hosts.contains(&self.root_id),
                active_default: shared.observation.active_default,
                nested: shared.nested,
                // The shared scan also reports visible/enabled programmatic
                // destinations, preserving registered negative-tabindex openers.
                return_eligible: model.invoker.as_ref().is_some_and(|(id,_)| {
                    shared.eligible_focus_ids.contains(id)
                }),
            };
            let changed = model.facts != facts;
            model.facts = facts;
            (request.and_then(|_| model.intent.take()), changed)
        };
        if changed {
            let mut revision = self.revision;
            let next = revision.peek().wrapping_add(1);
            revision.set(next);
        }
        let target = match intent {
            Some(PaneFocusIntent::Open) => {
                let (active, ids) = {
                    let model = self.model.borrow();
                    let mut ids = model.facts.autofocus.clone();
                    ids.extend(model.facts.ids.iter().cloned());
                    (model.facts.active.clone(), ids)
                };
                let invoker = active.filter(|id| shared.eligible_focus_ids.contains(id)).and_then(|id| {
                    self.runtime
                        .capture_registered_focus(&id)
                        .map(|target| (id, target))
                });
                self.model.borrow_mut().invoker = invoker;
                ids.iter()
                    .find_map(|id| self.runtime.capture_registered_focus(id))
            }
            Some(PaneFocusIntent::Return) => {
                let mut model = self.model.borrow_mut();
                let owned = model.facts.inside || model.facts.active_default;
                let eligible = model.facts.return_eligible;
                model
                    .invoker
                    .take()
                    .and_then(|(_, target)| (owned && eligible).then_some(target))
            }
            None => None,
        };
        if let Some(target) = target {
            self.queue(target);
        }
    }
    fn tab(&self, backwards: bool) -> bool {
        let target = {
            let model = self.model.borrow();
            if !model.alive || !model.expanded || !model.floating || model.facts.nested {
                return false;
            }
            let ids = &model.facts.ids;
            let current = model
                .facts
                .active
                .as_ref()
                .and_then(|active| ids.iter().position(|id| id == active));
            if !model.facts.inside || current.is_none() || ids.is_empty() {
                return false;
            }
            let boundary = if backwards {
                current == Some(0)
            } else {
                current == Some(ids.len() - 1)
            };
            if !boundary {
                return false;
            }
            let id = if backwards { ids.last() } else { ids.first() };
            id.and_then(|id| self.runtime.capture_registered_focus(id))
        };
        target.is_some_and(|target| self.queue(target))
    }
    fn queue(&self, target: crate::overlay_runtime::MountedFocusTarget) -> bool {
        if target.live_node().is_none() {
            return false;
        }
        let (epoch, pending) = {
            let mut model = self.model.borrow_mut();
            if !model.alive {
                return false;
            }
            model.epoch = model.epoch.wrapping_add(1);
            (model.epoch, model.task.take())
        };
        if let Some(task) = pending {
            task.cancel();
        }
        let weak = Rc::downgrade(&self.model);
        let owner = Rc::downgrade(&dioxus::core::Runtime::current());
        let scope = dioxus::core::current_scope_id();
        dioxus::core::queue_effect(move || {
            let Some(model) = weak.upgrade() else {
                return;
            };
            if !model.borrow().alive || model.borrow().epoch != epoch {
                return;
            }
            let Some(node) = target.live_node() else {
                return;
            };
            if let Some(owner) = owner.upgrade() {
                let task = owner.in_scope(scope, || {
                    spawn(async move {
                        if target.live_node().is_some() {
                            let _ = node.set_focus(true).await;
                        }
                    })
                });
                model.borrow_mut().task = Some(task);
            }
        });
        dioxus::core::needs_update();
        true
    }
    fn dispose(&self) {
        self.runtime.unregister_focus_observer(&self.root_id, self.registration.get());
        let task = {
            let mut model = self.model.borrow_mut();
            model.alive = false;
            model.epoch = model.epoch.wrapping_add(1);
            model.invoker = None;
            model.task.take()
        };
        if let Some(task) = task {
            task.cancel();
        }
    }
}

/// Collapsible pane with mutable expansion and optional external state updates.
/// Changing `expanded` synchronizes the local state; user requests can be vetoed
/// through `onbeforeexpandedchange`. The owning layout supplies composition and
/// default styling. Desktop size is capped at half the owning PaneLayout's
/// measured dimension. Standalone panes use the shared viewport dimensions;
/// constrained standalone containers supply `available_size` explicitly.
/// The upstream small breakpoint (48em) uses full-height mobile overlays.
#[component]
pub fn Pane(
    children: Element,
    heading: String,
    #[props(default)] expanded: bool,
    #[props(default)] composition: PaneComposition,
    #[props(default)] variant: PaneVariant,
    #[props(default = "240px".to_string())] size: String,
    /// Available container width/height, overriding PaneLayout or viewport size.
    /// Supply finite nonnegative dimensions for constrained standalone panes.
    #[props(default)]
    available_size: Option<(f64, f64)>,
    #[props(default)] icon: Option<String>,
    #[props(default)] header: Option<Element>,
    #[props(default)] hide_on_collapse: bool,
    #[props(default)] borderless: bool,
    #[props(default)] no_padding: bool,
    #[props(default)] close_on_click_outside: bool,
    #[props(default)] ignore_layout_settings: bool,
    #[props(default)] aria_label_icon: Option<String>,
    #[props(default)] aria_label_collapse_close_button: Option<String>,
    #[props(default)] onbeforeexpandedchange: EventHandler<PaneExpandedChangeEvent>,
    #[props(default)] onexpandedchange: EventHandler<bool>,
) -> Element {
    let context = try_consume_context::<PaneContext>().map(|c| (c.settings)());
    let composition = context.map(|c| c.composition).unwrap_or(composition);
    let styling = context.filter(|_| !ignore_layout_settings);
    let variant = styling.map(|c| c.variant).unwrap_or(variant);
    let borderless = styling.map(|c| c.borderless).unwrap_or(borderless);
    let layout = try_consume_context::<PaneLayoutContext>();
    let mut state = use_signal(|| expanded);
    let previous_expanded = use_hook(|| Rc::new(Cell::new(expanded)));
    let incoming_expansion = (previous_expanded.replace(expanded) != expanded).then_some(expanded);
    use_effect(use_reactive((&expanded,), move |(expanded,)| {
        state.set(expanded)
    }));
    let slot = context
        .map(|_| composition.class().to_string())
        .unwrap_or_default();
    let request = use_callback(move |next: bool| {
        if request_pane_expansion(*state.peek(), next, slot.clone(), onbeforeexpandedchange) {
            state.set(next);
            onexpandedchange.call(next);
        }
    });
    let mut observed_close = use_signal(|| {
        layout
            .map(|layout| *layout.close_requested.peek())
            .unwrap_or(0)
    });
    use_effect(use_reactive((&variant,), move |(variant,)| {
        if let Some(layout) = layout {
            let tick = (layout.close_requested)();
            if tick != *observed_close.peek() {
                observed_close.set(tick);
                if variant == PaneVariant::Floating {
                    request.call(false);
                }
            }
        }
    }));
    let viewport = crate::application_viewport::use_application_viewport(None, None);
    let layout_dimensions = layout.and_then(|layout| (layout.dimensions)());
    let (width, height) =
        pane_available_dimensions(available_size, layout_dimensions, viewport.viewport_size());
    let mobile = viewport.breakpoint() == ApplicationBreakpoint::Small;
    let vertical = matches!(composition, PaneComposition::Left | PaneComposition::Right);
    let size = pane_expanded_size(&size, if vertical { width } else { height });
    let expanded = state();
    let id = use_hook(|| format!("ix-pane-{}", NAV_ID.fetch_add(1, Ordering::Relaxed)));
    let root_id = format!("{id}-root");
    let toggle_id = format!("{id}-toggle");
    let toggle_mount = crate::dropdown::use_focus_mount(toggle_id.clone());
    let runtime = crate::overlay_runtime::use_overlay_runtime();
    let focus = use_hook(|| PaneFocusController::new(runtime.clone(), root_id.clone()));
    use_context_provider(|| PaneFocusScope);
    #[cfg(test)]
    if let Some(capture) = try_consume_context::<PaneFocusCapture>() {
        *capture.0.borrow_mut() = Some(focus.clone());
    }
    // A controlled prop can change before its state-copy effect runs. Reject
    // focus queued for the old presentation during this render, rather than
    // allowing that focus effect to race the prop synchronization effect.
    focus.transition(
        incoming_expansion.unwrap_or(expanded),
        variant == PaneVariant::Floating,
    );
    let current_focus = focus.current();
    let bridge = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    let cleanup_focus = focus.clone();
    let cleanup_bridge = bridge.clone();
    use_drop(move || {
        cleanup_focus.dispose();
        if let Some(bridge) = cleanup_bridge.borrow_mut().take() {
            let _ = bridge.send("finish");
        }
    });
    let sync_focus = focus.clone();
    use_effect(use_reactive((&variant,), move |(variant,)| {
        sync_focus.transition(state(), variant == PaneVariant::Floating);
        sync_focus.refresh();
    }));
    let mount_focus = focus.clone();
    let key_focus = focus.clone();
    let focused = focus.clone();
    let blurred = focus.clone();
    let label = if expanded {
        if hide_on_collapse || mobile {
            "Close pane"
        } else {
            "Collapse pane"
        }
    } else {
        "Expand pane"
    };
    let label = aria_label_collapse_close_button.unwrap_or_else(|| label.to_string());
    let toggle_icon = if (hide_on_collapse || mobile) && expanded {
        "close"
    } else {
        let position = if mobile {
            match composition {
                PaneComposition::Left => PaneComposition::Top,
                PaneComposition::Right => PaneComposition::Bottom,
                other => other,
            }
        } else {
            composition
        };
        match (position, expanded) {
            (PaneComposition::Left, true) | (PaneComposition::Right, false) => {
                "double-chevron-left"
            }
            (PaneComposition::Right, true) | (PaneComposition::Left, false) => {
                "double-chevron-right"
            }
            (PaneComposition::Top, true) | (PaneComposition::Bottom, false) => "double-chevron-up",
            _ => "double-chevron-down",
        }
    };
    rsx! {
            crate::overlay_runtime::OverlayTransport {runtime}
            aside { id: root_id.clone(), class: "ix-pane ix-pane-{composition.class()}", "aria-label": heading.clone(), hidden: !expanded && hide_on_collapse,
                "data-ix-overlay-host":root_id.clone(),
                "data-expanded": expanded.to_string(), "data-floating": (variant == PaneVariant::Floating).to_string(),
                "data-mobile": mobile.to_string(), "data-hide-on-collapse": hide_on_collapse.to_string(),
                "data-close-on-outside": close_on_click_outside.to_string(),"data-pane-focus-current":current_focus,
                "data-borderless": borderless.to_string(), style: "--ix-pane-size:{size}px",
                onmounted: move |_| {
                    let mut task_bridge = dioxus::document::eval(PANE_OUTSIDE_EVENTS);
                    if task_bridge.send(root_id.clone()).is_err() { return; }
                    *bridge.borrow_mut() = Some(task_bridge);
                    mount_focus.transition(*state.peek(),variant == PaneVariant::Floating);
                    mount_focus.refresh();
                    spawn(async move {
                        while task_bridge.recv::<()>().await.is_ok() {
                            request.call(false);
                        }
                    });
                },
                onfocusin: move |_| focused.refresh(),
                onfocusout: move |_| blurred.refresh(),
                onkeydown: move |event: KeyboardEvent| {
                    if !event.default_action_enabled() || key_focus.model.borrow().facts.nested { return; }
                    match event.key() {
                        Key::Escape if expanded && variant == PaneVariant::Floating => { event.prevent_default(); event.stop_propagation(); request.call(false); }
                        Key::Tab if key_focus.tab(event.modifiers().shift()) => { event.prevent_default(); event.stop_propagation(); }
                        _ => {}
                    }
                },
                div { class: "ix-pane-header",
                    button { id:toggle_id,onmounted:toggle_mount,class: "ix-app-icon-button", r#type: "button", "aria-label": label, "aria-expanded": expanded.to_string(), "aria-controls": id.clone(),
                        onclick: move |_| request.call(!expanded), crate::Icon { name: toggle_icon.to_string(), size: 24 }
                    }
                    div { class: "ix-pane-heading", if let Some(icon) = icon { span { role: if aria_label_icon.is_some() { Some("img") } else { None }, "aria-label": aria_label_icon, crate::Icon { name: icon, size: 24 } } } span { "{heading}" } }
                    if expanded { {header} }
                }
                div { id, class: "ix-pane-content", hidden: !expanded, "data-no-padding": no_padding.to_string(), {children} }
            }
    }
}

fn pane_expanded_size(size: &str, parent: f64) -> f64 {
    let size = pane_size(size);
    if size.ends_with('%') {
        return if size == "50%" {
            (parent / 2.0).round()
        } else {
            (parent / 3.0).round()
        };
    }
    let pixels = size.trim_end_matches("px").parse::<f64>().unwrap_or(320.0);
    let half_parent = (parent / 2.0).round();
    if half_parent > 0.0 {
        pixels.min(half_parent)
    } else {
        pixels
    }
}

// Pinned Dioxus has no global click subscription with capture/bubble phases.
// Capture the expanded/close-on-outside state before target handlers can change
// it; Rust owns dismissal. Canonical shared observations supply all focus facts.
const PANE_OUTSIDE_EVENTS: &str = r#"
const pane=document.getElementById(await dioxus.recv());
if(!pane)return;
let disposed=false;
const outsideEvents=new WeakSet();
const captureOutside=event=>{if(pane.dataset.expanded==='true'&&pane.dataset.closeOnOutside==='true'&&!event.composedPath().includes(pane))outsideEvents.add(event);};
const outside=event=>{if(!outsideEvents.has(event))return;outsideEvents.delete(event);if(!disposed&&pane.dataset.expanded==='true')dioxus.send(null);};
window.addEventListener('click',captureOutside,true);window.addEventListener('click',outside);
try{
    await dioxus.recv();
}finally{
    disposed=true;
    window.removeEventListener('click',captureOutside,true);window.removeEventListener('click',outside);
}
"#;

fn pane_size(size: &str) -> &str {
    match size {
        "240px" | "320px" | "360px" | "480px" | "600px" | "33%" | "50%" => size,
        _ => "320px",
    }
}

#[cfg(test)]
mod pane_focus_tests {
    include!("pane_focus_tests.rs");
}

#[cfg(test)]
mod pane_layout_measure_tests {
    use super::*;

    #[derive(Clone)]
    struct LayoutMount(Rc<Cell<(f64, f64)>>);
    impl dioxus::html::RenderedElementBacking for LayoutMount {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn get_client_rect(
            &self,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = dioxus::html::MountedResult<dioxus::html::geometry::PixelsRect>,
                    >,
            >,
        > {
            let (width, height) = self.0.get();
            Box::pin(async move {
                Ok(dioxus::html::geometry::PixelsRect::new(
                    dioxus::html::geometry::euclid::Point2D::new(0.0, 0.0),
                    dioxus::html::geometry::euclid::Size2D::new(width, height),
                ))
            })
        }
    }
    fn mounted(size: Rc<Cell<(f64, f64)>>) -> Rc<MountedData> {
        Rc::new(MountedData::new(LayoutMount(size)))
    }
    fn settle(dom: &mut VirtualDom) {
        for _ in 0..5 {
            dom.render_immediate_to_vec();
            dom.process_events();
        }
    }
    fn layout_fixture() -> Element {
        rsx! {PaneLayout {
            left:rsx!{Pane {heading:"Left",size:"33%",expanded:true,"Left content"}},
            right:rsx!{Pane {heading:"Right",size:"480px",expanded:true,"Right content"}},
            top:rsx!{Pane {heading:"Top",size:"50%",expanded:true,"Top content"}},
            bottom:rsx!{Pane {heading:"Override",size:"600px",available_size:Some((200.0,150.0)),expanded:true,"Override content"}},
            "Main content"
        }}
    }
    fn assert_size(html: &str, heading: &str, size: f64) {
        let label = format!("aria-label=\"{heading}\"");
        let pane = html
            .split("<aside")
            .skip(1)
            .find(|pane| pane.split('>').next().unwrap().contains(&label))
            .unwrap();
        assert!(
            pane.split('>')
                .next()
                .unwrap()
                .contains(&format!("--ix-pane-size:{size}px")),
            "{heading}: {html}"
        );
    }
    #[test]
    fn mounted_layout_resizes_percent_pixel_axes_and_explicit_override() {
        let capture = PaneLayoutMeasureCapture::default();
        let mut dom = VirtualDom::new(layout_fixture).with_root_context(capture.clone());
        dom.rebuild_in_place();
        settle(&mut dom);
        let measurement = capture.0.borrow().clone().unwrap();
        let size = Rc::new(Cell::new((900.0, 600.0)));
        dom.in_scope(ScopeId::ROOT, || measurement.mount(mounted(size.clone())));
        settle(&mut dom);
        let html = dioxus_ssr::render(&dom);
        assert_size(&html, "Left", 300.0);
        assert_size(&html, "Right", 450.0);
        assert_size(&html, "Top", 300.0);
        assert_size(&html, "Override", 75.0);
        size.set((600.0, 360.0));
        dom.in_scope(ScopeId::ROOT, || measurement.measure());
        settle(&mut dom);
        let html = dioxus_ssr::render(&dom);
        assert_size(&html, "Left", 200.0);
        assert_size(&html, "Right", 300.0);
        assert_size(&html, "Top", 180.0);
        assert_size(&html, "Override", 75.0);
    }
    #[test]
    fn layout_measurement_rejects_older_mount_reads_and_disposed_owners() {
        let capture = PaneLayoutMeasureCapture::default();
        let mut dom = VirtualDom::new(layout_fixture).with_root_context(capture.clone());
        dom.rebuild_in_place();
        settle(&mut dom);
        let measurement = capture.0.borrow().clone().unwrap();
        let old = mounted(Rc::new(Cell::new((900.0, 600.0))));
        let current = mounted(Rc::new(Cell::new((420.0, 240.0))));
        dom.in_scope(ScopeId::ROOT, || {
            measurement.mount(old);
            measurement.mount(current.clone());
        });
        settle(&mut dom);
        assert_eq!(*measurement.dimensions.peek(), Some((420.0, 240.0)));
        assert!(Rc::ptr_eq(
            measurement.model.borrow().root.as_ref().unwrap(),
            &current
        ));
        let html = dioxus_ssr::render(&dom);
        assert_size(&html, "Left", 140.0);
        assert_size(&html, "Top", 120.0);
        dom.in_scope(ScopeId::ROOT, || {
            measurement.mount(mounted(Rc::new(Cell::new((800.0, 700.0)))))
        });
        drop(dom);
        assert!(!measurement.model.borrow().alive);
        assert!(measurement.model.borrow().root.is_none());
        measurement.measure();
        measurement.mount(mounted(Rc::new(Cell::new((99.0, 99.0)))));
        assert!(measurement.model.borrow().root.is_none());
    }
    #[test]
    fn pane_dimensions_prefer_explicit_layout_then_viewport_without_dom_discovery() {
        assert_eq!(
            pane_available_dimensions(
                Some((200.0, 90.0)),
                Some((600.0, 400.0)),
                Some((1200.0, 800.0))
            ),
            (200.0, 90.0)
        );
        assert_eq!(
            pane_available_dimensions(None, Some((600.0, 400.0)), Some((1200.0, 800.0))),
            (600.0, 400.0)
        );
        assert_eq!(
            pane_available_dimensions(None, None, Some((1200.0, 800.0))),
            (1200.0, 800.0)
        );
        assert_eq!(pane_available_dimensions(None, None, None), (0.0, 0.0));
        assert_eq!(
            pane_available_dimensions(Some((f64::NAN, 50.0)), Some((600.0, 400.0)), None),
            (600.0, 400.0)
        );
        assert_eq!(pane_expanded_size("33%", 800.0), 267.0);
        assert_eq!(pane_expanded_size("50%", 501.0), 251.0);
        assert_eq!(pane_expanded_size("600px", 501.0), 251.0);
        assert_eq!(pane_expanded_size("not-allowed", 1000.0), 320.0);
    }
}
