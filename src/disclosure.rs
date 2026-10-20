//! Retained disclosure composition with native Dioxus controls and mounted reveal.
use crate::{CardVariant, Icon};
use dioxus::prelude::*;
use std::{
    cell::Cell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_DISCLOSURE: AtomicUsize = AtomicUsize::new(1);

fn use_disclosure_ids(kind: &str) -> (String, String) {
    let prefix = use_hook(|| {
        format!(
            "ix-{kind}-{}",
            NEXT_DISCLOSURE.fetch_add(1, Ordering::Relaxed)
        )
    });
    (format!("{prefix}-toggle"), format!("{prefix}-panel"))
}

// The panel remains mounted while hidden. A real opening can wait for that
// exact Dioxus mount without inspecting DOM or introducing an animation timer.
fn use_opening_reveal(open: bool, enabled: bool) -> EventHandler<MountedEvent> {
    let mut panel = use_signal(|| None::<Rc<MountedData>>);
    let previous = use_hook(|| Rc::new(Cell::new(open)));
    let pending = use_hook(|| Rc::new(Cell::new(false)));
    let current = use_hook(|| Rc::new(Cell::new((open, enabled))));
    current.set((open, enabled));
    use_effect(use_reactive((&open, &enabled), move |(open, enabled)| {
        let was_open = previous.replace(open);
        if !open || !enabled {
            pending.set(false);
        } else if !was_open {
            pending.set(true);
        }
        let mounted = panel();
        if pending.get()
            && let Some(node) = mounted
        {
            pending.set(false);
            let current = current.clone();
            // Dioxus cancels this task when its owning scope is unmounted.
            spawn(async move {
                if current.get() == (true, true) {
                    let _ = node
                        .scroll_to_with_options(dioxus::html::ScrollToOptions {
                            behavior: dioxus::html::ScrollBehavior::Smooth,
                            vertical: dioxus::html::ScrollLogicalPosition::Nearest,
                            horizontal: dioxus::html::ScrollLogicalPosition::Nearest,
                        })
                        .await;
                }
            });
        }
    }));
    use_callback(move |event: MountedEvent| panel.set(Some(event.data())))
}

/// Accessible card disclosure with retained children. `expanded` is controlled;
/// otherwise `default_expanded` initializes local state (closed by default).
/// `onexpand` reports one request per native activation. A held controlled
/// request does not reveal content. `reveal_content` scrolls an actual opening
/// into view through Dioxus, using the nearest edge on both axes.
#[component]
pub fn CardAccordion(
    children: Element,
    #[props(default = "Details".to_string())] label: String,
    #[props(default)] expanded: Option<bool>,
    #[props(default)] default_expanded: bool,
    #[props(default)] onexpand: EventHandler<bool>,
    #[props(default)] variant: CardVariant,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] aria_label: Option<String>,
    #[props(default = true)] reveal_content: bool,
) -> Element {
    let mut local = use_signal(|| default_expanded);
    let open = expanded.unwrap_or(local());
    let (toggle_id, panel_id) = use_disclosure_ids("card-accordion");
    let onmounted = use_opening_reveal(open, reveal_content);
    rsx! { div { id, class: "ix-card-accordion ix-card-{variant.class()} {class}",
        button { id: toggle_id.clone(), class: "ix-disclosure-button", r#type: "button",
            "aria-expanded": open, "aria-controls": panel_id.clone(), "aria-label": aria_label,
            onclick: move |event| {
                event.stop_propagation();
                if expanded.is_none() { local.set(!open); }
                onexpand.call(!open);
            },
            span { class: "ix-chevron", "data-open": open, Icon { name: "chevron-down" } } "{label}"
        }
        div { id: panel_id, class: "ix-card-accordion-content", role: "region",
            "aria-labelledby": toggle_id, hidden: !open, onmounted, {children} }
    } }
}

/// Retained information section with independent header actions. Custom header
/// content replaces the default label/icon presentation inside the native
/// toggle; keep it noninteractive and supply visible text or `aria_label`.
/// `header_actions` remain separate controls. `expanded` is controlled when set;
/// otherwise local state starts at `default_expanded` (closed by default).
#[component]
pub fn Blind(
    #[props(default)] label: String,
    children: Element,
    #[props(default)] sublabel: Option<String>,
    #[props(default)] icon: Option<String>,
    #[props(default = CardVariant::Filled)] variant: CardVariant,
    #[props(default)] header_actions: Option<Element>,
    #[props(default)] expanded: Option<bool>,
    #[props(default)] default_expanded: bool,
    #[props(default)] onexpand: EventHandler<bool>,
    #[props(default)] custom_header: Option<Element>,
    #[props(default)] aria_label: Option<String>,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
) -> Element {
    let mut local = use_signal(|| default_expanded);
    let open = expanded.unwrap_or(local());
    let (toggle_id, panel_id) = use_disclosure_ids("blind");
    rsx! { section { id, class: "ix-blind ix-card-{variant.class()} {class}",
        div { class: "ix-blind-heading",
            button { id: toggle_id.clone(), class: "ix-blind-header", r#type: "button",
                "aria-expanded": open, "aria-controls": panel_id.clone(), "aria-label": aria_label,
                onclick: move |_| {
                    if expanded.is_none() { local.set(!open); }
                    onexpand.call(!open);
                },
                span { class: "ix-chevron", "data-open": open, Icon { name: "chevron-down" } }
                if let Some(custom) = custom_header { span { class: "ix-blind-custom-header", {custom} } }
                else {
                    if let Some(icon) = icon { Icon { name: icon } }
                    span { class: "ix-blind-label", strong { "{label}" } if let Some(text) = sublabel { span { "{text}" } } }
                }
            }
            if let Some(actions) = header_actions { div { class: "ix-blind-actions", {actions} } }
        }
        div { id: panel_id, class: "ix-blind-content", role: "region", "aria-labelledby": toggle_id,
            hidden: !open, {children} }
    } }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Card, CardTitle};
    use std::{cell::RefCell, collections::BTreeMap};

    #[derive(Clone)]
    struct Config {
        controlled: Option<bool>,
        initial: bool,
        reveal: bool,
        custom: bool,
        visible: bool,
        variant: CardVariant,
    }
    impl Default for Config {
        fn default() -> Self {
            Self {
                controlled: None,
                initial: false,
                reveal: true,
                custom: true,
                visible: true,
                variant: CardVariant::Outline,
            }
        }
    }
    #[derive(Default)]
    struct Fixture {
        config: RefCell<Config>,
        requests: RefCell<Vec<bool>>,
        blind_requests: RefCell<Vec<bool>>,
        inspections: Cell<usize>,
        selections: Cell<usize>,
        mounts: RefCell<Vec<String>>,
    }
    impl PartialEq for Fixture {
        fn eq(&self, other: &Self) -> bool {
            std::ptr::eq(self, other)
        }
    }
    #[component]
    fn RetainedChild(kind: String, data: Rc<Fixture>) -> Element {
        let mut count = use_signal(|| 0usize);
        let mounted_kind = kind.clone();
        rsx! { div {
            "data-child-state": kind.clone(), "data-count": count().to_string(),
            onmounted: move |_| data.mounts.borrow_mut().push(mounted_kind.clone()),
            button { "data-child-action": kind.clone(), r#type:"button", onclick: move |_| count+=1, "Check preparation" }
        } }
    }
    fn fixture(data: Rc<Fixture>) -> Element {
        let config = data.config.borrow().clone();
        let selected = data.clone();
        let request = data.clone();
        let blind = data.clone();
        let inspect = data.clone();
        rsx! { if config.visible {
            Card { aria_label:"Select package", onselect:move |_| selected.selections.set(selected.selections.get()+1),
                CardTitle { "Motor 001" }
                CardAccordion { id:"parts", class:"parts-extra", label:"Parts", aria_label:"Show parts",
                    expanded:config.controlled, default_expanded:config.initial, reveal_content:config.reveal, variant:config.variant,
                    onexpand:move |value| request.requests.borrow_mut().push(value),
                    RetainedChild { kind:"parts", data:data.clone() }
                }
            }
            Blind { id:"readiness", class:"readiness-extra", label:"Fallback readiness", sublabel:"Fallback checklist", icon:"bulb", aria_label:"Show readiness", variant:config.variant,
                custom_header:config.custom.then(|| rsx! { span { "Custom readiness" } }),
                header_actions:rsx! { button { id:String::from("inspect"), r#type:"button", onclick:move |_| inspect.inspections.set(inspect.inspections.get()+1), "Inspect" } },
                onexpand:move |value| blind.blind_requests.borrow_mut().push(value),
                RetainedChild { kind:"readiness", data:data.clone() }
            }
        } }
    }
    type ScrollLog = Rc<RefCell<Vec<String>>>;
    #[derive(Clone)]
    struct ScrollMount {
        label: String,
        calls: ScrollLog,
    }
    impl dioxus::html::RenderedElementBacking for ScrollMount {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn scroll_to(
            &self,
            options: dioxus::html::ScrollToOptions,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>>
        {
            assert!(matches!(
                options.vertical,
                dioxus::html::ScrollLogicalPosition::Nearest
            ));
            assert!(matches!(
                options.horizontal,
                dioxus::html::ScrollLogicalPosition::Nearest
            ));
            assert_eq!(options.behavior, dioxus::html::ScrollBehavior::Smooth);
            self.calls.borrow_mut().push(self.label.clone());
            Box::pin(async { Ok(()) })
        }
    }
    struct Mouse;
    impl dioxus::html::InteractionLocation for Mouse {
        fn client_coordinates(&self) -> dioxus::html::geometry::ClientPoint {
            Default::default()
        }
        fn screen_coordinates(&self) -> dioxus::html::geometry::ScreenPoint {
            Default::default()
        }
        fn page_coordinates(&self) -> dioxus::html::geometry::PagePoint {
            Default::default()
        }
    }
    impl dioxus::html::InteractionElementOffset for Mouse {
        fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
            Default::default()
        }
    }
    impl dioxus::html::ModifiersInteraction for Mouse {
        fn modifiers(&self) -> Modifiers {
            Modifiers::empty()
        }
    }
    impl dioxus::html::PointerInteraction for Mouse {
        fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
            None
        }
        fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
            Default::default()
        }
    }
    impl dioxus::html::HasMouseData for Mouse {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    struct Converter;
    macro_rules! converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for Converter {
                $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data { let _=event;converter!(@data $group,event) })*
            }
        };
        (@data Mouse,$event:ident) => { dioxus::html::MouseData::new(Mouse) };
        (@data Mounted,$event:ident) => { MountedData::new($event.downcast::<ScrollMount>().unwrap().clone()) };
        (@data $other:ident,$event:ident) => { panic!("Unexpected disclosure test event") };
    }
    dioxus::html::with_html_event_groups!(converter);
    #[derive(Default)]
    struct Targets {
        attrs: BTreeMap<usize, BTreeMap<String, String>>,
        mounted: Vec<dioxus::core::ElementId>,
    }
    impl Targets {
        fn collect(&mut self, changes: dioxus::core::Mutations) {
            let mut current = dioxus::core::ElementId::ROOT;
            for edit in changes.edits {
                match edit {
                    dioxus::core::Mutation::SetId { id }
                    | dioxus::core::Mutation::PushId { id } => current = id,
                    dioxus::core::Mutation::SetAttribute {
                        name,
                        value: dioxus::core::AttributeValue::Text(value),
                        ..
                    } => {
                        self.attrs
                            .entry(current.raw())
                            .or_default()
                            .insert(name, value);
                    }
                    dioxus::core::Mutation::NewEventListener { name } if name == "mounted" => {
                        self.mounted.push(current)
                    }
                    _ => {}
                }
            }
        }
        fn attr(&self, name: &str, value: &str) -> dioxus::core::ElementId {
            dioxus::core::ElementId::from_raw(
                self.attrs
                    .iter()
                    .find_map(|(id, attrs)| {
                        (attrs.get(name).map(String::as_str) == Some(value)).then_some(*id)
                    })
                    .unwrap_or_else(|| panic!("Missing {name}={value}")),
            )
        }
    }
    fn dispatch(
        dom: &VirtualDom,
        name: &str,
        id: dioxus::core::ElementId,
        data: Box<dyn std::any::Any>,
    ) -> bool {
        let event = Event::new(
            Rc::new(dioxus::html::PlatformEventData::new(data)) as Rc<dyn std::any::Any>,
            name != "mounted",
        );
        dom.runtime().handle_event(name, event.clone(), id);
        event.default_action_enabled()
    }
    fn settle(dom: &mut VirtualDom, targets: &mut Targets) {
        for _ in 0..4 {
            targets.collect(dom.render_immediate_to_vec());
            dom.process_events();
        }
    }
    fn refresh(dom: &mut VirtualDom, targets: &mut Targets) {
        dom.mark_dirty(ScopeId::APP);
        settle(dom, targets);
    }

    fn mount(dom: &mut VirtualDom, targets: &mut Targets, calls: ScrollLog) {
        for id in targets.mounted.clone() {
            let label = targets
                .attrs
                .get(&id.raw())
                .and_then(|attrs| attrs.get("id"))
                .cloned()
                .unwrap_or_default();
            dispatch(
                dom,
                "mounted",
                id,
                Box::new(ScrollMount {
                    label,
                    calls: calls.clone(),
                }),
            );
        }
        settle(dom, targets);
    }
    fn setup(data: Rc<Fixture>) -> (VirtualDom, Targets, ScrollLog) {
        let mut dom = VirtualDom::new_with_props(fixture, data);
        let mut targets = Targets::default();
        targets.collect(dom.rebuild_to_vec());
        settle(&mut dom, &mut targets);
        let calls = Rc::new(RefCell::new(Vec::new()));
        mount(&mut dom, &mut targets, calls.clone());
        (dom, targets, calls)
    }
    fn tag<'a>(html: &'a str, name: &str, value: &str) -> &'a str {
        html.split('<')
            .map(|tag| tag.split('>').next().unwrap())
            .find(|tag| tag.contains(&format!("{name}=\"{value}\"")))
            .unwrap_or_else(|| panic!("Missing {name}={value}: {html}"))
    }
    fn attribute<'a>(tag: &'a str, name: &str) -> &'a str {
        tag.split(&format!("{name}=\""))
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
    }
    fn panel_open(dom: &VirtualDom, label: &str) -> bool {
        let html = dioxus_ssr::render(dom);
        let button = tag(&html, "aria-label", label);
        button.contains("aria-expanded=true") || button.contains("aria-expanded=\"true\"")
    }
    #[test]
    fn disclosure_retains_child_state_stable_relationships_and_independent_header_actions() {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(Converter));
        let data = Rc::new(Fixture::default());
        let (mut dom, mut targets, calls) = setup(data.clone());
        let initial = dioxus_ssr::render(&dom);
        let toggle = tag(&initial, "aria-label", "Show readiness");
        let toggle_id = attribute(toggle, "id").to_string();
        let panel_id = attribute(toggle, "aria-controls").to_string();
        let panel = tag(&initial, "id", &panel_id);
        assert!(panel.contains("role=\"region\""));
        assert_eq!(attribute(panel, "aria-labelledby"), toggle_id);
        assert!(panel.contains("hidden"));
        assert!(initial.contains("Custom readiness"));
        assert!(!initial.contains("Fallback readiness"));
        dispatch(&dom, "click", targets.attr("id", "inspect"), Box::new(()));
        settle(&mut dom, &mut targets);
        assert_eq!(data.inspections.get(), 1);
        assert!(data.blind_requests.borrow().is_empty());
        assert!(!panel_open(&dom, "Show readiness"));
        let blind = targets.attr("aria-label", "Show readiness");
        dispatch(&dom, "click", blind, Box::new(()));
        settle(&mut dom, &mut targets);
        assert!(panel_open(&dom, "Show readiness"));
        assert_eq!(*data.blind_requests.borrow(), [true]);
        dispatch(
            &dom,
            "click",
            targets.attr("data-child-action", "readiness"),
            Box::new(()),
        );
        settle(&mut dom, &mut targets);
        dispatch(&dom, "click", blind, Box::new(()));
        settle(&mut dom, &mut targets);
        data.config.borrow_mut().custom = false;
        refresh(&mut dom, &mut targets);
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("Fallback readiness"));
        assert!(!html.contains("Custom readiness"));
        assert_eq!(
            attribute(tag(&html, "aria-label", "Show readiness"), "id"),
            toggle_id
        );
        assert_eq!(
            attribute(tag(&html, "aria-label", "Show readiness"), "aria-controls"),
            panel_id
        );
        assert_eq!(
            attribute(tag(&html, "data-child-state", "readiness"), "data-count"),
            "1"
        );
        assert_eq!(data.mounts.borrow().as_slice(), ["parts", "readiness"]);
        assert!(dispatch(
            &dom,
            "click",
            targets.attr("aria-label", "Show parts"),
            Box::new(())
        ));
        settle(&mut dom, &mut targets);
        assert!(panel_open(&dom, "Show parts"));
        assert_eq!(data.selections.get(), 0);
        assert_eq!(*data.requests.borrow(), [true]);
        assert_eq!(calls.borrow().len(), 1);
        dispatch(
            &dom,
            "click",
            targets.attr("aria-label", "Select package"),
            Box::new(()),
        );
        assert_eq!(data.selections.get(), 1);
    }
    #[test]
    fn disclosure_reveals_actual_openings_but_not_held_initial_or_option_only_changes() {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(Converter));
        let data = Rc::new(Fixture::default());
        data.config.borrow_mut().controlled = Some(false);
        let (mut dom, mut targets, calls) = setup(data.clone());
        dispatch(
            &dom,
            "click",
            targets.attr("aria-label", "Show parts"),
            Box::new(()),
        );
        settle(&mut dom, &mut targets);
        assert_eq!(*data.requests.borrow(), [true]);
        assert!(!panel_open(&dom, "Show parts"));
        assert!(calls.borrow().is_empty());
        data.config.borrow_mut().controlled = Some(true);
        refresh(&mut dom, &mut targets);
        assert_eq!(calls.borrow().len(), 1);
        refresh(&mut dom, &mut targets);
        assert_eq!(calls.borrow().len(), 1);
        data.config.borrow_mut().controlled = Some(false);
        refresh(&mut dom, &mut targets);
        data.config.borrow_mut().reveal = false;
        data.config.borrow_mut().controlled = Some(true);
        refresh(&mut dom, &mut targets);
        assert_eq!(calls.borrow().len(), 1);
        data.config.borrow_mut().reveal = true;
        refresh(&mut dom, &mut targets);
        assert_eq!(calls.borrow().len(), 1);
        data.config.borrow_mut().controlled = Some(false);
        refresh(&mut dom, &mut targets);
        data.config.borrow_mut().controlled = Some(true);
        refresh(&mut dom, &mut targets);
        assert_eq!(calls.borrow().len(), 2);
        let initially_open = Rc::new(Fixture::default());
        initially_open.config.borrow_mut().initial = true;
        let (mut initial_dom, mut initial_targets, initial_calls) = setup(initially_open);
        assert!(panel_open(&initial_dom, "Show parts"));
        assert!(initial_calls.borrow().is_empty());
        let toggle = initial_targets.attr("aria-label", "Show parts");
        dispatch(&initial_dom, "click", toggle, Box::new(()));
        settle(&mut initial_dom, &mut initial_targets);
        dispatch(&initial_dom, "click", toggle, Box::new(()));
        settle(&mut initial_dom, &mut initial_targets);
        assert_eq!(initial_calls.borrow().len(), 1);
    }
    #[test]
    fn disclosure_pending_reveal_uses_its_mount_and_all_variants_preserve_composition() {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(Converter));
        let data = Rc::new(Fixture::default());
        data.config.borrow_mut().controlled = Some(false);
        let mut dom = VirtualDom::new_with_props(fixture, data.clone());
        let mut targets = Targets::default();
        targets.collect(dom.rebuild_to_vec());
        settle(&mut dom, &mut targets);
        data.config.borrow_mut().controlled = Some(true);
        refresh(&mut dom, &mut targets);
        let calls = Rc::new(RefCell::new(Vec::new()));
        assert!(calls.borrow().is_empty());
        mount(&mut dom, &mut targets, calls.clone());
        assert_eq!(calls.borrow().len(), 1);
        assert!(calls.borrow()[0].starts_with("ix-card-accordion-"));
        assert!(calls.borrow()[0].ends_with("-panel"));
        for variant in [
            CardVariant::Outline,
            CardVariant::Filled,
            CardVariant::Alarm,
            CardVariant::Critical,
            CardVariant::Warning,
            CardVariant::Success,
            CardVariant::Info,
            CardVariant::Neutral,
            CardVariant::Primary,
        ] {
            data.config.borrow_mut().variant = variant;
            refresh(&mut dom, &mut targets);
            let html = dioxus_ssr::render(&dom);
            assert!(tag(&html, "id", "parts").contains(&format!("ix-card-{}", variant.class())));
            assert!(
                tag(&html, "id", "readiness").contains(&format!("ix-card-{}", variant.class()))
            );
        }
        assert_eq!(data.mounts.borrow().len(), 2);
        assert_eq!(calls.borrow().len(), 1);
        data.config.borrow_mut().controlled = Some(false);
        refresh(&mut dom, &mut targets);
        // Remove the actual scope in the same external update as an opening;
        // there is no live disclosure or registered panel to reveal afterward.
        data.config.borrow_mut().controlled = Some(true);
        data.config.borrow_mut().visible = false;
        refresh(&mut dom, &mut targets);
        assert!(!dioxus_ssr::render(&dom).contains("ix-card-accordion"));
        assert_eq!(calls.borrow().len(), 1);
    }
}
