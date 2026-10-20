use super::*;
use std::{cell::RefCell, collections::BTreeMap};

#[derive(Clone)]
struct Config {
    ids: Vec<&'static str>,
    label: Option<String>,
    aria: Option<String>,
    show_all: Option<bool>,
    collapsed: Option<bool>,
    limit: usize,
    suppress: bool,
    hide_action: bool,
    count: Option<usize>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            ids: vec!["one", "two", "three", "four"],
            label: Some("Assets".into()),
            aria: None,
            show_all: None,
            collapsed: None,
            limit: 2,
            suppress: false,
            hide_action: false,
            count: None,
        }
    }
}
#[derive(Default)]
struct Fixture {
    config: RefCell<Config>,
    veto: Cell<bool>,
    requests: RefCell<Vec<(bool, CardListRevealSource)>>,
    accepted: RefCell<Vec<bool>>,
    mounts: Rc<Cell<usize>>,
}
fn fixture(data: Rc<Fixture>) -> Element {
    let config = data.config.borrow().clone();
    let requests = data.clone();
    let items = config
        .ids
        .iter()
        .map(|id| {
            CardListItem::new(
                *id,
                *id,
                rsx! { StatefulCard { id: *id, mounts: data.mounts.clone() } },
            )
        })
        .collect();
    rsx! { CardList {
        items, label: config.label, aria_label_expand_button: config.aria,
        show_all: config.show_all, collapsed: config.collapsed,
        max_visible_cards: config.limit, suppress_overflow_handling: config.suppress,
        hide_show_all: config.hide_action, show_all_count: config.count,
        show_all_label: "Alle anzeigen", show_less_label: "Weniger anzeigen",
        more_cards_label: "Weitere Karten verfügbar",
        onrevealrequest: move |request: CardListRevealEvent| {
            requests.requests.borrow_mut().push((request.show_all, request.source));
            if requests.veto.get() { request.prevent_default(); }
        },
        onshowall: move |show| data.accepted.borrow_mut().push(show),
    } }
}
#[component]
fn StatefulCard(id: &'static str, mounts: Rc<Cell<usize>>) -> Element {
    // Its initialization count distinguishes retained keyed Dioxus ownership
    // from re-created hidden children when the parent reorders the entries.
    let identity = use_hook(|| {
        mounts.set(mounts.get() + 1);
        format!("{id}-{}", mounts.get())
    });
    rsx! { span { "data-own-state": identity, "{id}" } }
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
    fn modifiers(&self) -> dioxus::html::Modifiers {
        dioxus::html::Modifiers::empty()
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
struct Focus;
impl dioxus::html::HasFocusData for Focus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
struct Mount(Rc<Cell<usize>>);
impl dioxus::html::RenderedElementBacking for Mount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        let calls = self.0.clone();
        Box::pin(async move {
            calls.set(calls.get() + 1);
            Ok(())
        })
    }
}
struct Converter;
macro_rules! converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for Converter {
                $(fn $convert(&self, event: &dioxus::html::PlatformEventData) -> dioxus::html::$data {
                    let _ = event; converter!(@data $group, event)
                })*
            }
        };
        (@data Mouse,$event:ident) => { dioxus::html::MouseData::new(Mouse) };
        (@data Focus,$event:ident) => { dioxus::html::FocusData::new(Focus) };
        (@data Mounted,$event:ident) => { MountedData::new($event.downcast::<Mount>().unwrap().clone()) };
        (@data $other:ident,$event:ident) => { panic!("Unexpected card collection event") };
    }
dioxus::html::with_html_event_groups!(converter);
#[derive(Default)]
struct Targets {
    listeners: BTreeMap<String, Vec<dioxus::core::ElementId>>,
    entries: BTreeMap<String, dioxus::core::ElementId>,
}
impl Targets {
    fn collect(&mut self, changes: dioxus::core::Mutations) {
        let mut current = dioxus::core::ElementId::ROOT;
        for edit in changes.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = id
                }
                dioxus::core::Mutation::NewEventListener { name } => {
                    self.listeners.entry(name).or_default().push(current)
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(id),
                    ..
                } if name == "data-card-list-entry" => {
                    self.entries.insert(id, current);
                }
                _ => {}
            }
        }
    }
    fn listener(&self, name: &str, index: usize) -> dioxus::core::ElementId {
        self.listeners[name][index]
    }
}
fn dispatch(dom: &VirtualDom, name: &str, target: dioxus::core::ElementId) {
    dispatch_data(dom, name, target, Box::new(()));
}
fn dispatch_data(
    dom: &VirtualDom,
    name: &str,
    target: dioxus::core::ElementId,
    data: Box<dyn std::any::Any>,
) {
    dom.runtime().handle_event(
        name,
        Event::new(Rc::new(dioxus::html::PlatformEventData::new(data)), true),
        target,
    );
}
fn settle(dom: &mut VirtualDom) {
    for _ in 0..4 {
        dom.render_immediate_to_vec();
        dom.process_events();
    }
}
fn html(dom: &VirtualDom) -> String {
    dioxus_ssr::render(dom)
}
fn attr(html: &str, name: &str, value: usize) -> bool {
    html.contains(&format!("{name}={value}")) || html.contains(&format!("{name}=\"{value}\""))
}

#[test]
fn card_list_real_controls_limit_reveal_veto_and_controlled_authority() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let mut targets = Targets::default();
    targets.collect(dom.rebuild_to_vec());
    settle(&mut dom);
    let initial = html(&dom);
    assert!(
        attr(&initial, "data-count", 4) && attr(&initial, "data-visible-count", 2),
        "{initial}"
    );
    assert_eq!(initial.matches("class=\"ix-card-list-item\"").count(), 4);
    assert_eq!(
        initial
            .split('<')
            .filter(|tag| {
                let opening = tag.split('>').next().unwrap();
                opening.contains("class=\"ix-card-list-item\"") && opening.contains(" hidden")
            })
            .count(),
        2,
        "{initial}"
    );
    assert!(
        initial.contains("Alle anzeigen (4)") && initial.contains("Weitere Karten verfügbar (2)")
    );
    let header = targets.listener("click", 1);
    let more = targets.listener("click", 2);
    dispatch(&dom, "click", targets.listener("click", 0));
    settle(&mut dom);
    assert!(attr(&html(&dom), "data-visible-count", 0));
    dispatch(&dom, "click", targets.listener("click", 0));
    settle(&mut dom);
    assert!(attr(&html(&dom), "data-visible-count", 2));
    data.veto.set(true);
    dispatch(&dom, "click", more);
    settle(&mut dom);
    dispatch(&dom, "click", header);
    settle(&mut dom);
    assert!(data.accepted.borrow().is_empty());
    assert!(attr(&html(&dom), "data-visible-count", 2));
    data.veto.set(false);
    data.config.borrow_mut().show_all = Some(false);
    dom.mark_dirty(ScopeId::APP);
    settle(&mut dom);
    dispatch(&dom, "click", header);
    settle(&mut dom);
    assert_eq!(*data.accepted.borrow(), [true]);
    assert!(
        attr(&html(&dom), "data-visible-count", 2),
        "controlled held reveal stays limited"
    );
    data.config.borrow_mut().show_all = Some(true);
    dom.mark_dirty(ScopeId::APP);
    settle(&mut dom);
    assert!(attr(&html(&dom), "data-visible-count", 4));
    dispatch(&dom, "click", header);
    settle(&mut dom);
    assert_eq!(*data.accepted.borrow(), [true, false]);
    assert!(
        attr(&html(&dom), "data-visible-count", 4),
        "controlled show less stays held"
    );
    data.config.borrow_mut().show_all = None;
    dom.mark_dirty(ScopeId::APP);
    settle(&mut dom);
    assert!(attr(&html(&dom), "data-visible-count", 2));
    dispatch(&dom, "click", header);
    settle(&mut dom);
    assert!(attr(&html(&dom), "data-visible-count", 4));
    dispatch(&dom, "click", header);
    settle(&mut dom);
    assert!(attr(&html(&dom), "data-visible-count", 2));
    assert_eq!(
        *data.requests.borrow(),
        [
            (true, CardListRevealSource::MoreCard),
            (true, CardListRevealSource::Header),
            (true, CardListRevealSource::Header),
            (false, CardListRevealSource::Header),
            (true, CardListRevealSource::Header),
            (false, CardListRevealSource::Header),
        ]
    );
}

#[test]
fn card_list_labels_dynamic_items_suppression_and_legacy_children() {
    let data = Rc::new(Fixture::default());
    data.config.borrow_mut().label = None;
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    dom.rebuild_in_place();
    assert!(!html(&dom).contains("ix-card-list-header"));
    data.config.borrow_mut().aria = Some("Toggle assets".into());
    data.config.borrow_mut().hide_action = true;
    dom.mark_dirty(ScopeId::APP);
    settle(&mut dom);
    assert!(html(&dom).contains("aria-label=\"Toggle assets\""));
    assert!(!html(&dom).contains("ix-card-list-show-all"));
    assert!(
        html(&dom).contains("ix-card-list-more"),
        "More survives hidden header action"
    );
    data.config.borrow_mut().ids = vec!["four", "one", "three"];
    data.config.borrow_mut().limit = 1;
    data.config.borrow_mut().suppress = true;
    dom.mark_dirty(ScopeId::APP);
    settle(&mut dom);
    let changed = html(&dom);
    assert!(attr(&changed, "data-count", 3) && attr(&changed, "data-visible-count", 3));
    assert!(!changed.contains("ix-card-list-more"));
    assert_eq!(
        data.mounts.get(),
        4,
        "hidden/reordered children retain local ownership"
    );
    assert!(changed.find("data-own-state=\"four-4\"") < changed.find("data-own-state=\"one-1\""));
    assert!(!changed.contains("data-own-state=\"two-2\""));
    data.config.borrow_mut().ids.push("five");
    dom.mark_dirty(ScopeId::APP);
    settle(&mut dom);
    assert_eq!(data.mounts.get(), 5);
    assert!(attr(&html(&dom), "data-count", 4));
    fn legacy() -> Element {
        rsx! { CardList { label: "Legacy", show_all_count: 19,
            span { "Unbounded child" } span { "Second child" }
        } }
    }
    let mut old = VirtualDom::new(legacy);
    old.rebuild_in_place();
    let old = html(&old);
    assert!(old.contains("Show all (19)") && old.contains("Unbounded child"));
    assert!(!old.contains("data-card-list-entry") && !old.contains("ix-card-list-more"));
}

#[test]
fn card_list_mounted_reveal_and_collapse_follow_actual_visibility_and_focus() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    data.config.borrow_mut().label = None;
    data.config.borrow_mut().aria = Some("Toggle assets".into());
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let mut targets = Targets::default();
    targets.collect(dom.rebuild_to_vec());
    let collapse_calls = Rc::new(Cell::new(0));
    let reveal_calls = Rc::new(Cell::new(0));
    dispatch_data(
        &dom,
        "mounted",
        targets.listener("mounted", 0),
        Box::new(Mount(collapse_calls.clone())),
    );
    dispatch_data(
        &dom,
        "mounted",
        targets.entries["three"],
        Box::new(Mount(reveal_calls.clone())),
    );
    settle(&mut dom);
    dispatch(&dom, "click", targets.listener("click", 2));
    settle(&mut dom);
    assert_eq!(
        reveal_calls.get(),
        1,
        "More focuses first actually revealed wrapper"
    );
    dispatch(&dom, "focusin", targets.listener("focusin", 0));
    data.config.borrow_mut().collapsed = Some(true);
    dom.mark_dirty(ScopeId::APP);
    settle(&mut dom);
    assert_eq!(collapse_calls.get(), 1);
    assert!(attr(&html(&dom), "data-visible-count", 0));
    data.config.borrow_mut().collapsed = Some(false);
    dom.mark_dirty(ScopeId::APP);
    settle(&mut dom);
    dispatch(&dom, "focusout", targets.listener("focusout", 0));
    data.config.borrow_mut().collapsed = Some(true);
    dom.mark_dirty(ScopeId::APP);
    settle(&mut dom);
    assert_eq!(collapse_calls.get(), 1, "outside focus is not stolen");
    assert_eq!(
        reveal_calls.get(),
        1,
        "expanding the collection does not repeat reveal focus"
    );
}
