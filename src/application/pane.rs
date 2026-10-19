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
    use_context_provider(|| PaneLayoutContext { close_requested });
    rsx! {
        div { class: "ix-pane-layout", "data-layout": if layout == PaneLayoutOrder::FullVertical { "full-vertical" } else { "full-horizontal" },
            if let Some(left) = left { PaneSlot { composition: PaneComposition::Left, variant, borderless, {left} } }
            if let Some(right) = right { PaneSlot { composition: PaneComposition::Right, variant, borderless, {right} } }
            if let Some(top) = top { PaneSlot { composition: PaneComposition::Top, variant, borderless, {top} } }
            if let Some(bottom) = bottom { PaneSlot { composition: PaneComposition::Bottom, variant, borderless, {bottom} } }
            div { class: "ix-pane-layout-content", onclick: move |_| { let next = close_requested().wrapping_add(1); close_requested.set(next); }, {children} }
        }
    }
}

#[derive(Default, PartialEq, Eq, serde::Deserialize)]
struct PaneFocusFacts {
    #[serde(default)]
    ids: Vec<String>,
    #[serde(default)]
    autofocus: Vec<String>,
    #[serde(default)]
    active: Option<String>,
    #[serde(default)]
    inside: bool,
    #[serde(default)]
    active_default: bool,
    #[serde(default)]
    nested: bool,
    #[serde(default)]
    return_eligible: bool,
}

#[derive(serde::Deserialize)]
struct PaneObservation {
    kind: String,
    #[serde(default)]
    epoch: u64,
    #[serde(default)]
    width: f64,
    #[serde(default)]
    height: f64,
    #[serde(default)]
    mobile: bool,
    #[serde(flatten)]
    focus: PaneFocusFacts,
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
struct PaneFocusController {
    runtime: crate::overlay_runtime::RuntimeHandle,
    model: Rc<RefCell<PaneFocusModel>>,
    revision: Signal<u64>,
}
#[cfg(test)]
#[derive(Clone, Default)]
struct PaneFocusCapture(Rc<RefCell<Option<PaneFocusController>>>);
impl PaneFocusController {
    fn new(runtime: crate::overlay_runtime::RuntimeHandle) -> Self {
        Self {
            runtime,
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
        }
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
    fn refresh(&self, bridge: &Rc<RefCell<Option<dioxus::document::Eval>>>) {
        let model = self.model.borrow();
        if model.alive
            && let Some(bridge) = bridge.borrow().as_ref()
        {
            let _ = bridge.send(serde_json::json!({"epoch":model.epoch,
                "invoker":model.invoker.as_ref().map(|(id,_)|id)}));
        }
    }
    fn current(&self) -> Option<String> {
        let _ = (self.revision)();
        self.model.borrow().facts.active.clone()
    }
    fn observe(&self, epoch: u64, facts: PaneFocusFacts) {
        let (intent, changed) = {
            let mut model = self.model.borrow_mut();
            if !model.alive || model.epoch != epoch {
                return;
            }
            let changed = model.facts != facts;
            model.facts = facts;
            (model.intent.take(), changed)
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
                let invoker = active.and_then(|id| {
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
/// default styling. Desktop size is capped at half of the parent dimension;
/// the upstream small breakpoint (48em) uses full-height mobile overlays.
#[component]
pub fn Pane(
    children: Element,
    heading: String,
    #[props(default)] expanded: bool,
    #[props(default)] composition: PaneComposition,
    #[props(default)] variant: PaneVariant,
    #[props(default = "240px".to_string())] size: String,
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
    let mut dimensions = use_signal(|| (0.0, 0.0, false));
    let (width, height, mobile) = dimensions();
    let vertical = matches!(composition, PaneComposition::Left | PaneComposition::Right);
    let size = pane_expanded_size(&size, if vertical { width } else { height });
    let expanded = state();
    let id = use_hook(|| format!("ix-pane-{}", NAV_ID.fetch_add(1, Ordering::Relaxed)));
    let root_id = format!("{id}-root");
    let toggle_id = format!("{id}-toggle");
    let toggle_mount = crate::dropdown::use_focus_mount(toggle_id.clone());
    let runtime = crate::overlay_runtime::use_overlay_runtime();
    let focus = use_hook(|| PaneFocusController::new(runtime));
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
    let sync_bridge = bridge.clone();
    use_effect(use_reactive((&variant,), move |(variant,)| {
        let _ = dimensions();
        sync_focus.transition(state(), variant == PaneVariant::Floating);
        sync_focus.refresh(&sync_bridge);
    }));
    let mount_focus = focus.clone();
    let key_focus = focus.clone();
    let focus_bridge = bridge.clone();
    let focused = focus.clone();
    let blur_bridge = bridge.clone();
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
            aside { id: root_id.clone(), class: "ix-pane ix-pane-{composition.class()}", "aria-label": heading.clone(), hidden: !expanded && hide_on_collapse,
                "data-expanded": expanded.to_string(), "data-floating": (variant == PaneVariant::Floating).to_string(),
                "data-mobile": mobile.to_string(), "data-hide-on-collapse": hide_on_collapse.to_string(),
                "data-close-on-outside": close_on_click_outside.to_string(),"data-pane-focus-current":current_focus,
                "data-borderless": borderless.to_string(), style: "--ix-pane-size:{size}px",
                onmounted: move |_| {
                    let mut task_bridge = dioxus::document::eval(PANE_DOM_BRIDGE);
                    if task_bridge.send(root_id.clone()).is_err() { return; }
                    *bridge.borrow_mut() = Some(task_bridge);
                    mount_focus.transition(*state.peek(),variant == PaneVariant::Floating);
                    mount_focus.refresh(&bridge);
                    let focus = mount_focus.clone();
                    spawn(async move {
                        while let Ok(report) = task_bridge.recv::<PaneObservation>().await {
                            match report.kind.as_str() {
                                "measure" if report.width.is_finite() && report.height.is_finite() => dimensions.set((report.width.max(0.0), report.height.max(0.0), report.mobile)),
                                "outside" => request.call(false),
                                "focus" => focus.observe(report.epoch,report.focus),
                                _ => {}
                            }
                        }
                    });
                },
                onfocusin: move |_| focused.refresh(&focus_bridge),
                onfocusout: move |_| blurred.refresh(&blur_bridge),
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

// Pinned MountedData can focus and measure a registered element, but cannot
// enumerate descendant controls, report the active control or test CSS/inert
// visibility (dioxus-html alpha.2 events/mounted.rs and events/focus.rs). These
// bounded facts contain existing Rust IDs only; every focus uses a live mounted
// snapshot in Rust. Resize/outside delivery remains legacy native plumbing.
// Dioxus lifecycle/focus events refresh the cache; interior Tab stays native.
const PANE_DOM_BRIDGE: &str = r#"
const pane=document.getElementById(await dioxus.recv());
if(!pane)return;
const parent=pane.closest('.ix-pane-layout')??pane.parentElement;
let disposed=false;
const presented=node=>node instanceof HTMLElement&&!node.matches(':disabled,[disabled],[hidden],[aria-disabled="true"]')&&!node.closest('[aria-hidden="true"],[hidden],[inert]')&&node.getClientRects().length>0&&getComputedStyle(node).visibility!=='hidden'&&getComputedStyle(node).visibility!=='collapse';
const measure=()=>{if(disposed)return;const rect=parent?.getBoundingClientRect();dioxus.send({kind:'measure',width:rect?.width??0,height:rect?.height??0,mobile:matchMedia('(max-width:48em)').matches});};
const facts=command=>{
    const active=document.activeElement;
    const items=[...pane.querySelectorAll('button,a[href],input,select,textarea,[tabindex],[contenteditable]')].filter(node=>node.tabIndex>=0&&presented(node));
    dioxus.send({kind:'focus',epoch:command.epoch,ids:items.map(node=>node.id),autofocus:items.filter(node=>node.hasAttribute('autofocus')).map(node=>node.id),
        active:active?.id||null,inside:pane.contains(active),active_default:active===document.body||active===document.documentElement,
        nested:!!active?.closest?.('dialog[open],[data-ix-overlay-open="true"]')||(()=>{try{return !!active?.closest?.('[popover]:popover-open');}catch{return false;}})(),
        return_eligible:!!command.invoker&&presented(document.getElementById(command.invoker))});
};
const outsideEvents=new WeakSet();
const captureOutside=event=>{if(pane.dataset.expanded==='true'&&pane.dataset.closeOnOutside==='true'&&!event.composedPath().includes(pane))outsideEvents.add(event);};
const outside=event=>{if(!outsideEvents.has(event))return;outsideEvents.delete(event);if(!disposed&&pane.dataset.expanded==='true')dioxus.send({kind:'outside'});};
const resize=new ResizeObserver(measure);if(parent)resize.observe(parent);
window.addEventListener('resize',measure);
window.addEventListener('click',captureOutside,true);window.addEventListener('click',outside);
try{
    measure();
    for(;;){const command=await dioxus.recv();if(command==='finish')break;facts(command);}
}finally{
    disposed=true;resize.disconnect();window.removeEventListener('resize',measure);
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
    use super::*;
    use dioxus::core::{ElementId, Mutation};

    #[derive(Clone)]
    struct Mount {
        id: String,
        log: Rc<RefCell<Vec<String>>>,
    }
    impl dioxus::html::RenderedElementBacking for Mount {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn set_focus(
            &self,
            focus: bool,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>>
        {
            let this = self.clone();
            Box::pin(async move {
                assert!(focus);
                this.log.borrow_mut().push(this.id);
                Ok(())
            })
        }
    }
    struct Converter;
    macro_rules! converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for Converter {
                $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;converter!(@data $group,event)})*
            }
        };
        (@data Mounted,$event:ident)=>{MountedData::new($event.downcast::<Mount>().unwrap().clone())};
        (@data $other:ident,$event:ident)=>{panic!("unexpected Pane test event")};
    }
    dioxus::html::with_html_event_groups!(converter);
    #[derive(Default)]
    struct Config {
        capture: PaneFocusCapture,
        expanded: Option<Signal<bool>>,
        floating: Option<Signal<bool>>,
        visible: Option<Signal<bool>>,
    }
    #[component]
    fn Field() -> Element {
        let id = "pane-field".to_string();
        let mounted = crate::dropdown::use_focus_mount(id.clone());
        rsx! { input {id,onmounted:mounted,autofocus:true} }
    }
    fn fixture(config: Rc<RefCell<Config>>) -> Element {
        let expanded = use_signal(|| false);
        let floating = use_signal(|| true);
        let visible = use_signal(|| true);
        let capture = config.borrow().capture.clone();
        use_context_provider(|| capture);
        {
            let mut config = config.borrow_mut();
            config.expanded = Some(expanded);
            config.floating = Some(floating);
            config.visible = Some(visible);
        }
        rsx! {
            crate::Button {id:"pane-opener", "Open"}
            if visible() { Pane {heading:"Test Pane",expanded:expanded(),variant:if floating(){PaneVariant::Floating}else{PaneVariant::Inline},hide_on_collapse:true,
                Field {}
                crate::Button {id:"pane-action","Action"}
            }}
        }
    }
    struct Harness {
        dom: VirtualDom,
        config: Rc<RefCell<Config>>,
        focus: PaneFocusController,
        ids: BTreeMap<String, ElementId>,
        toggle: String,
        log: Rc<RefCell<Vec<String>>>,
    }
    impl Harness {
        fn new() -> Self {
            dioxus::html::set_event_converter(Box::new(Converter));
            let config = Rc::new(RefCell::new(Config::default()));
            let mut dom = VirtualDom::new_with_props(fixture, config.clone());
            let mut ids = BTreeMap::new();
            let mut current = ElementId::ROOT;
            for edit in dom.rebuild_to_vec().edits {
                match edit {
                    Mutation::SetId { id } | Mutation::PushId { id } => current = id,
                    Mutation::SetAttribute {
                        name,
                        value: dioxus::core::AttributeValue::Text(value),
                        ..
                    } if name == "id" => {
                        ids.insert(value, current);
                    }
                    _ => {}
                }
            }
            let focus = config.borrow().capture.0.borrow().clone().unwrap();
            let toggle = ids
                .keys()
                .find(|id| id.ends_with("-toggle"))
                .unwrap()
                .clone();
            let result = Self {
                dom,
                config,
                focus,
                ids,
                toggle,
                log: Rc::new(RefCell::new(Vec::new())),
            };
            for id in [
                "pane-opener",
                "pane-field",
                "pane-action",
                result.toggle.as_str(),
            ] {
                result.mount(id, id);
            }
            result
        }
        fn mount(&self, id: &str, record: &str) {
            let data: Rc<dyn std::any::Any> =
                Rc::new(dioxus::html::PlatformEventData::new(Box::new(Mount {
                    id: record.into(),
                    log: self.log.clone(),
                })));
            self.dom
                .runtime()
                .handle_event("mounted", Event::new(data, false), self.ids[id]);
        }
        fn settle(&mut self) {
            for _ in 0..6 {
                self.dom.render_immediate_to_vec();
            }
        }
        fn expanded(&mut self, value: bool) {
            self.dom.in_scope(ScopeId::ROOT, || {
                self.config.borrow().expanded.unwrap().set(value)
            });
            self.settle();
        }
        fn facts(&self, active: Option<&str>, inside: bool) -> PaneFocusFacts {
            PaneFocusFacts {
                ids: vec![
                    self.toggle.clone(),
                    "pane-field".into(),
                    "pane-action".into(),
                ],
                autofocus: vec!["pane-field".into()],
                active: active.map(str::to_string),
                inside,
                active_default: active.is_none(),
                return_eligible: true,
                ..Default::default()
            }
        }
        fn report_pending(&self, facts: PaneFocusFacts) {
            let epoch = self.focus.model.borrow().epoch;
            self.dom
                .in_scope(ScopeId::ROOT, || self.focus.observe(epoch, facts));
        }
        fn report(&mut self, facts: PaneFocusFacts) {
            self.report_pending(facts);
            self.settle();
        }
        fn tab(&mut self, backwards: bool) -> bool {
            let prevented = self
                .dom
                .in_scope(ScopeId::ROOT, || self.focus.tab(backwards));
            self.settle();
            prevented
        }
    }
    #[test]
    fn floating_pane_opens_registered_autofocus_and_wraps_only_native_boundaries() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        h.expanded(true);
        let mut facts = h.facts(Some("pane-opener"), false);
        facts.autofocus.insert(0, "unregistered-autofocus".into());
        h.report(facts);
        assert_eq!(*h.log.borrow(), ["pane-field"]);
        h.report(h.facts(Some("pane-field"), true));
        assert!(!h.tab(false), "interior Tab keeps its native default");
        assert_eq!(h.log.borrow().len(), 1);
        h.report(h.facts(Some("pane-action"), true));
        assert!(h.tab(false));
        assert_eq!(h.log.borrow().last(), Some(&h.toggle));
        h.report(h.facts(Some(&h.toggle), true));
        assert!(h.tab(true));
        assert_eq!(
            h.log.borrow().last().map(String::as_str),
            Some("pane-action")
        );
        let mut unsupported = h.facts(Some("pane-action"), true);
        unsupported.ids.insert(0, "unregistered-boundary".into());
        h.report(unsupported);
        assert!(
            !h.tab(false),
            "an unsupported destination cannot cancel native Tab"
        );
        let mut nested = h.facts(Some("pane-action"), true);
        nested.nested = true;
        h.report(nested);
        assert!(
            !h.tab(false),
            "nested overlays retain their own keyboard handling"
        );
        h.dom.in_scope(ScopeId::ROOT, || {
            h.config.borrow().floating.unwrap().set(false)
        });
        h.settle();
        h.report(h.facts(Some("pane-action"), true));
        assert!(!h.tab(false), "inline panes never trap Tab");
    }
    #[test]
    fn pane_close_returns_live_owned_opener_and_preserves_independent_outside_focus() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        h.expanded(true);
        h.report(h.facts(Some("pane-opener"), false));
        h.expanded(false);
        h.report(h.facts(None, false));
        assert_eq!(
            h.log.borrow().last().map(String::as_str),
            Some("pane-opener")
        );
        h.expanded(true);
        h.report(h.facts(Some("pane-opener"), false));
        h.log.borrow_mut().clear();
        h.expanded(false);
        h.report(h.facts(Some("outside-destination"), false));
        assert!(
            h.log.borrow().is_empty(),
            "outside close must preserve an independent focus destination"
        );
        h.expanded(true);
        h.report(h.facts(Some("pane-opener"), false));
        h.log.borrow_mut().clear();
        h.expanded(false);
        let mut facts = h.facts(None, false);
        facts.return_eligible = false;
        h.report(facts);
        assert!(
            h.log.borrow().is_empty(),
            "a hidden or disabled opener is not a return destination"
        );
    }
    #[test]
    fn pane_mounted_focus_rejects_replacement_close_and_scope_teardown() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        h.expanded(true);
        h.report_pending(h.facts(Some("pane-opener"), false));
        h.mount("pane-field", "replacement-field");
        h.settle();
        assert!(
            h.log.borrow().is_empty(),
            "a snapshot cannot focus a replacement control"
        );
        h.expanded(false);
        h.report(h.facts(Some("outside"), false));
        h.expanded(true);
        h.report(h.facts(Some("pane-opener"), false));
        assert_eq!(
            h.log.borrow().last().map(String::as_str),
            Some("replacement-field")
        );
        h.log.borrow_mut().clear();
        h.report_pending(h.facts(Some("pane-action"), true));
        h.dom.in_scope(ScopeId::ROOT, || {
            assert!(h.focus.tab(false));
            h.config.borrow().expanded.unwrap().set(false);
        });
        h.settle();
        assert!(
            h.log.borrow().is_empty(),
            "closing invalidates a deferred wrap"
        );
        h.expanded(true);
        h.report_pending(h.facts(Some("pane-opener"), false));
        let retained = h.focus.clone();
        let log = h.log.clone();
        drop(h);
        assert!(!retained.model.borrow().alive);
        assert!(log.borrow().is_empty());
    }
}
