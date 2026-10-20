use super::*;
use std::{cell::RefCell, collections::BTreeMap};

#[test]
fn breadcrumb_truncation_keeps_the_current_location_visible() {
    assert_eq!(hidden_breadcrumb_count(5, 3), 2);
    assert_eq!(hidden_breadcrumb_count(3, 9), 0);
    assert_eq!(hidden_breadcrumb_count(3, 0), 2);
    assert_eq!(hidden_breadcrumb_count(0, 3), 0);
}
#[derive(Clone)]
struct Config {
    items: Vec<BreadcrumbEntry>,
    next: Vec<BreadcrumbEntry>,
    visible: usize,
    subtle: bool,
    top: bool,
    labels: BreadcrumbLabels,
}
impl Default for Config {
    fn default() -> Self {
        let mut items: Vec<_> = [
            ("root", "Facilities"),
            ("north", "North plant"),
            ("assembly", "Assembly"),
            ("line", "Line 01"),
            ("cell", "Cell A"),
        ]
        .into_iter()
        .map(|(key, label)| BreadcrumbEntry::new(key, label))
        .collect();
        items[2].href = Some("#assembly".into());
        items[2].target = Some("_blank".into());
        items[2].rel = Some("noopener".into());
        Self {
            items,
            next: vec![
                BreadcrumbEntry::new("motor-1", "Motor 001"),
                BreadcrumbEntry::new("motor-2", "Motor 002"),
            ],
            visible: 3,
            subtle: false,
            top: true,
            labels: BreadcrumbLabels::default(),
        }
    }
}
#[derive(Default)]
struct Fixture {
    config: RefCell<Config>,
    veto: Cell<bool>,
    before: RefCell<Vec<(String, BreadcrumbNavigationSource)>>,
    items: RefCell<Vec<BreadcrumbEntry>>,
    next: RefCell<Vec<BreadcrumbEntry>>,
}
fn fixture(data: Rc<Fixture>) -> Element {
    let config = data.config.borrow().clone();
    let before = data.clone();
    let item = data.clone();
    rsx! { Breadcrumb {
        items: config.items, next_items: config.next, visible_item_count: config.visible,
        labels: config.labels, subtle: config.subtle, enable_top_layer: config.top,
        onbeforenavigate: move |event: BreadcrumbNavigateEvent| {
            before.before.borrow_mut().push((event.item.key.clone(),event.source));
            if before.veto.get() { event.clone().prevent_default(); assert!(event.default_prevented()); }
        },
        onitem: move |entry| item.items.borrow_mut().push(entry),
        onnext: move |entry| data.next.borrow_mut().push(entry),
    } }
}
#[derive(Clone)]
struct RawKey(Key);
impl dioxus::html::ModifiersInteraction for RawKey {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::HasKeyboardData for RawKey {
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
        (@data Keyboard,$event:ident) => { dioxus::html::KeyboardData::new($event.downcast::<RawKey>().unwrap().clone()) };
        (@data Mounted,$event:ident) => { MountedData::new(()) };
        (@data $other:ident,$event:ident) => { panic!("Unexpected breadcrumb test event") };
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
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = id
                }
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
fn mount(dom: &mut VirtualDom, targets: &mut Targets) {
    for id in targets.mounted.clone() {
        dispatch(dom, "mounted", id, Box::new(()));
    }
    settle(dom, targets);
}
fn opening<'a>(html: &'a str, name: &str, value: &str) -> &'a str {
    html.split('<')
        .map(|tag| tag.split('>').next().unwrap())
        .find(|tag| tag.contains(&format!("{name}=\"{value}\"")))
        .unwrap_or_else(|| panic!("Missing {name}={value}: {html}"))
}
fn expanded(dom: &VirtualDom, kind: &str) -> bool {
    let html = dioxus_ssr::render(dom);
    let trigger = opening(&html, "data-breadcrumb-menu", kind);
    trigger.contains("aria-expanded=\"true\"")
        || trigger
            .split_ascii_whitespace()
            .any(|attribute| attribute == "aria-expanded=true")
}

#[test]
fn breadcrumb_visible_native_links_preserve_metadata_and_veto_only_the_rejected_request() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let mut targets = Targets::default();
    targets.collect(dom.rebuild_to_vec());
    settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    let anchor = opening(&html, "href", "#assembly");
    assert!(
        anchor.starts_with("a ")
            && anchor.contains("target=\"_blank\"")
            && anchor.contains("rel=\"noopener\"")
    );
    assert_eq!(html.matches("aria-current=\"page\"").count(), 1);
    assert!(opening(&html, "data-breadcrumb-menu", "next").contains("aria-current=\"page\""));
    data.veto.set(true);
    assert!(!dispatch(
        &dom,
        "click",
        targets.attr("href", "#assembly"),
        Box::new(())
    ));
    assert!(data.items.borrow().is_empty());
    data.veto.set(false);
    assert!(
        dispatch(
            &dom,
            "click",
            targets.attr("href", "#assembly"),
            Box::new(())
        ),
        "accepted link retains native default"
    );
    let entries = data.items.borrow();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].target.as_deref(), Some("_blank"));
    assert_eq!(
        *data.before.borrow(),
        [
            ("assembly".into(), BreadcrumbNavigationSource::Item),
            ("assembly".into(), BreadcrumbNavigationSource::Item)
        ]
    );
}

#[test]
fn breadcrumb_retained_menus_use_distinct_requests_and_actual_open_state() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let mut targets = Targets::default();
    targets.collect(dom.rebuild_to_vec());
    mount(&mut dom, &mut targets);
    let previous = targets.attr("data-breadcrumb-menu", "previous");
    assert!(
        targets.mounted.contains(&previous),
        "actual previous trigger owns a mounted listener"
    );
    assert!(!expanded(&dom, "previous"));
    dispatch(&dom, "click", previous, Box::new(()));
    settle(&mut dom, &mut targets);
    assert!(
        expanded(&dom, "previous"),
        "mounted native trigger opens its actual menu"
    );
    data.veto.set(true);
    assert!(!dispatch(
        &dom,
        "click",
        targets.attr("data-breadcrumb-entry", "root"),
        Box::new(())
    ));
    settle(&mut dom, &mut targets);
    assert!(expanded(&dom, "previous"));
    assert!(data.items.borrow().is_empty());
    data.veto.set(false);
    dispatch(
        &dom,
        "click",
        targets.attr("data-breadcrumb-entry", "root"),
        Box::new(()),
    );
    settle(&mut dom, &mut targets);
    assert!(!expanded(&dom, "previous"));
    assert_eq!(data.items.borrow()[0].key, "root");
    let next = targets.attr("data-breadcrumb-menu", "next");
    dispatch(&dom, "click", next, Box::new(()));
    settle(&mut dom, &mut targets);
    assert!(expanded(&dom, "next"));
    dispatch(
        &dom,
        "keydown",
        targets.attr("data-breadcrumb-entry", "motor-1"),
        Box::new(RawKey(Key::Enter)),
    );
    settle(&mut dom, &mut targets);
    assert_eq!(data.next.borrow().len(), 1);
    assert_eq!(data.next.borrow()[0].key, "motor-1");
    assert_eq!(
        *data.before.borrow(),
        [
            ("root".into(), BreadcrumbNavigationSource::Previous),
            ("root".into(), BreadcrumbNavigationSource::Previous),
            ("motor-1".into(), BreadcrumbNavigationSource::Next)
        ]
    );
    assert_eq!(
        dioxus_ssr::render(&dom)
            .matches("aria-current=\"page\"")
            .count(),
        1
    );
}

#[test]
fn breadcrumb_labels_layering_order_visibility_and_legacy_children_remain_explicit() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let mut targets = Targets::default();
    targets.collect(dom.rebuild_to_vec());
    mount(&mut dom, &mut targets);
    let trigger = targets.attr("data-breadcrumb-menu", "previous");
    let first = dioxus_ssr::render(&dom);
    assert_eq!(first.matches("popover=\"manual\"").count(), 2);
    assert!(
        opening(&first, "data-breadcrumb-menu", "next").contains("aria-controls=\"ix-dropdown-")
    );
    {
        let mut config = data.config.borrow_mut();
        config.items.swap(0, 1);
        config.visible = 0;
        config.subtle = true;
        config.top = false;
        config.labels = BreadcrumbLabels {
            previous_button: "Vorherige Ebenen".into(),
            next_button: "Untere Ebenen".into(),
        };
    }
    refresh(&mut dom, &mut targets);
    let changed = dioxus_ssr::render(&dom);
    assert_eq!(
        targets.attr("data-breadcrumb-menu", "previous"),
        trigger,
        "options/reorder retain native trigger ownership"
    );
    assert!(
        changed.contains("Vorherige Ebenen")
            && changed.contains("Untere Ebenen")
            && changed.contains("ix-button-subtle-primary")
    );
    assert!(!changed.contains("popover=\"manual\""));
    assert!(
        changed.find("data-breadcrumb-entry=\"north\"")
            < changed.find("data-breadcrumb-entry=\"root\"")
    );
    assert_eq!(changed.matches("aria-current=\"page\"").count(), 1);
    fn legacy() -> Element {
        rsx! { Breadcrumb {
            BreadcrumbItem{label:"Old parent",href:"#legacy",target:"_blank",rel:"noreferrer"}
            BreadcrumbItem{label:"Current",current:true}
        } }
    }
    let mut old = VirtualDom::new(legacy);
    old.rebuild_in_place();
    let old = dioxus_ssr::render(&old);
    assert!(old.contains("Old parent") && old.contains("Current"));
    assert_eq!(old.matches("aria-current=\"page\"").count(), 1);
    assert!(opening(&old, "href", "#legacy").contains("rel=\"noreferrer\""));
    assert!(opening(&old, "aria-current", "page").starts_with("span "));
}
