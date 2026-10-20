use super::*;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};
#[derive(Clone)]
struct Config {
    disabled: bool,
    selected: bool,
    filled: bool,
    color: Option<String>,
    height: EventItemHeight,
    reverse: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            disabled: false,
            selected: true,
            filled: false,
            color: None,
            height: EventItemHeight::Small,
            reverse: false,
        }
    }
}
#[derive(Default)]
struct Fixture {
    config: RefCell<Config>,
    selected: RefCell<Vec<String>>,
    actions: Cell<usize>,
    mounts: RefCell<Vec<String>>,
    chevron: RefCell<Option<Signal<Option<bool>>>>,
    animated: RefCell<Option<Signal<bool>>>,
}
impl PartialEq for Fixture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn WrappedEvent(
    key_name: String,
    data: Rc<Fixture>,
    disabled: bool,
    selected: bool,
    filled: bool,
    color: Option<String>,
) -> Element {
    let mut note = use_signal(|| String::from("Check bearing"));
    let mut acknowledged = use_signal(|| false);
    let request = data.clone();
    let action = data.clone();
    let mounted = data.clone();
    let value = key_name.clone();
    let mounted_key = key_name.clone();
    rsx! { div { class:"test-event-wrapper",
        EventListItem { id:format!("row-{key_name}"), class:"custom-event-row", status:CardVariant::Warning,
            chevron:key_name=="a", disabled,selected,filled,item_color:color,
            aria_label:(key_name=="a").then(||String::from("Select A")),
            onselect:move |_|request.selected.borrow_mut().push(value.clone()),
            span { "{key_name} equipment event" }
            label { r#for:format!("note-{key_name}"), "Note" }
            input { id:format!("note-{key_name}"),value:note(),
                onmounted:move |_|mounted.mounts.borrow_mut().push(mounted_key.clone()),
                oninput:move |event|note.set(event.value()) }
            button { id:format!("ack-{key_name}"),r#type:"button",
                onclick:move |_|{acknowledged.toggle();action.actions.set(action.actions.get()+1);},
                if acknowledged() {"Reopen"} else {"Acknowledge"}
            }
        }
    } }
}
fn fixture(data: Rc<Fixture>) -> Element {
    let config = data.config.borrow().clone();
    let chevron = use_signal(|| None::<bool>);
    let animated = use_signal(|| false);
    *data.chevron.borrow_mut() = Some(chevron);
    *data.animated.borrow_mut() = Some(animated);
    let keys = if config.reverse {
        ["b", "a"]
    } else {
        ["a", "b"]
    };
    rsx! { EventList {id:"events",class:"events-extra",aria_label:"Equipment queue",item_height:config.height,chevron,animated,
        for key in keys { WrappedEvent {key:"{key}",key_name:key.to_string(),data:data.clone(),disabled:config.disabled,selected:config.selected,filled:config.filled,color:config.color.clone()} }
        div { EventList {id:"inner-events",EventListItem {id:"inner-row",chevron:true,"Nested event"}} }
    } }
}
struct RawForm(String);
impl dioxus::html::HasFileData for RawForm {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for RawForm {
    fn value(&self) -> String {
        self.0.clone()
    }
    fn valid(&self) -> bool {
        true
    }
    fn values(&self) -> Vec<(String, dioxus::html::FormValue)> {
        Vec::new()
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
        (@data Mounted,$event:ident) => { MountedData::new(()) };
        (@data Form,$event:ident) => { dioxus::html::FormData::new(RawForm($event.downcast::<String>().unwrap().clone())) };
        (@data $other:ident,$event:ident) => { panic!("Unexpected event-list test event") };
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

fn setup(data: Rc<Fixture>) -> (VirtualDom, Targets) {
    let mut dom = VirtualDom::new_with_props(fixture, data);
    let mut targets = Targets::default();
    targets.collect(dom.rebuild_to_vec());
    for id in targets.mounted.clone() {
        dispatch(&dom, "mounted", id, Box::new(()));
    }
    settle(&mut dom, &mut targets);
    (dom, targets)
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
fn bool_attr(tag: &str, name: &str, value: bool) -> bool {
    tag.split('>')
        .next()
        .unwrap()
        .split_ascii_whitespace()
        .any(|attribute| {
            attribute == format!("{name}={value}") || attribute == format!("{name}=\"{value}\"")
        })
}
fn row<'a>(html: &'a str, id: &str) -> &'a str {
    let start = html.find(&format!("id=\"{id}\"")).unwrap();
    let rest = &html[start..];
    let end = rest.find("</button>").unwrap();
    &rest[..end + 9]
}
fn row_chevron(html: &str, id: &str) -> bool {
    let start = html.find(&format!("id=\"{id}\"")).unwrap();
    let rest = &html[start..];
    let end = rest.find("</div></div></div>").unwrap_or(rest.len());
    rest[..end].contains("ix-event-chevron")
}
#[test]
fn event_list_native_selection_and_composed_controls_are_independent_and_retained() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let (mut dom, mut targets) = setup(data.clone());
    let initial = dioxus_ssr::render(&dom);
    let primary = tag(&initial, "aria-label", "Select A");
    assert!(primary.starts_with("button "));
    assert!(!primary.contains("aria-labelledby"));
    let body = initial.split("id=\"row-b\"").nth(1).unwrap();
    let primary = body.split("</button>").next().unwrap();
    let content = attribute(primary, "aria-labelledby").to_string();
    assert!(tag(&initial, "id", &content).contains("ix-event-content"));
    let mut depth = 0;
    for tag in initial.split('<').skip(1) {
        if tag.starts_with("button ") {
            depth += 1;
            assert_eq!(depth, 1, "composed buttons must be siblings of selection");
        } else if tag.starts_with("/button>") {
            depth -= 1;
        }
    }
    assert_eq!(depth, 0);
    assert!(dispatch(
        &dom,
        "click",
        targets.attr("aria-label", "Select A"),
        Box::new(())
    ));
    assert_eq!(*data.selected.borrow(), ["a"]);
    dispatch(
        &dom,
        "input",
        targets.attr("id", "note-a"),
        Box::new(String::from("Inspect seal")),
    );
    dispatch(&dom, "click", targets.attr("id", "ack-a"), Box::new(()));
    settle(&mut dom, &mut targets);
    assert_eq!(data.actions.get(), 1);
    assert_eq!(*data.selected.borrow(), ["a"]);
    let html = dioxus_ssr::render(&dom);
    assert!(tag(&html, "id", "note-a").contains("value=\"Inspect seal\""));
    assert!(html.contains("Reopen"));
    data.config.borrow_mut().disabled = true;
    refresh(&mut dom, &mut targets);
    dispatch(
        &dom,
        "click",
        targets.attr("aria-label", "Select A"),
        Box::new(()),
    );
    assert_eq!(*data.selected.borrow(), ["a"]);
    dispatch(&dom, "click", targets.attr("id", "ack-a"), Box::new(()));
    settle(&mut dom, &mut targets);
    assert_eq!(
        data.actions.get(),
        2,
        "custom controls own their disabled state"
    );
    assert_eq!(data.mounts.borrow().as_slice(), ["a", "b"]);
}
#[test]
fn event_list_shared_reactive_options_cross_wrappers_without_remounting_or_overriding_nested_lists()
{
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let (mut dom, mut targets) = setup(data.clone());
    let html = dioxus_ssr::render(&dom);
    assert!(row_chevron(&html, "row-a"));
    assert!(!row_chevron(&html, "row-b"));
    assert!(row_chevron(&html, "inner-row"));
    dispatch(
        &dom,
        "input",
        targets.attr("id", "note-a"),
        Box::new(String::from("Retained note")),
    );
    settle(&mut dom, &mut targets);
    dom.in_runtime(|| {
        data.chevron.borrow().unwrap().set(Some(false));
        data.animated.borrow().unwrap().set(true);
    });
    settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(!row_chevron(&html, "row-a"));
    assert!(!row_chevron(&html, "row-b"));
    assert!(row_chevron(&html, "inner-row"));
    assert!(bool_attr(tag(&html, "id", "row-a"), "data-animated", true));
    assert!(bool_attr(
        tag(&html, "id", "inner-row"),
        "data-animated",
        false
    ));
    dom.in_runtime(|| data.chevron.borrow().unwrap().set(Some(true)));
    settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(row_chevron(&html, "row-a"));
    assert!(row_chevron(&html, "row-b"));
    data.config.borrow_mut().reverse = true;
    refresh(&mut dom, &mut targets);
    dom.in_runtime(|| data.chevron.borrow().unwrap().set(None));
    settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(html.find("id=\"row-b\"").unwrap() < html.find("id=\"row-a\"").unwrap());
    assert!(row_chevron(&html, "row-a"));
    assert!(!row_chevron(&html, "row-b"));
    assert!(tag(&html, "id", "note-a").contains("value=\"Retained note\""));
    assert_eq!(data.mounts.borrow().as_slice(), ["a", "b"]);
}
#[test]
fn event_list_custom_indicator_tokens_and_existing_states_labels_and_heights_are_preserved() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    data.config.borrow_mut().color = Some(String::from("--ix-color-primary"));
    data.config.borrow_mut().filled = true;
    let (mut dom, mut targets) = setup(data.clone());
    let html = dioxus_ssr::render(&dom);
    assert!(tag(&html, "id", "events").contains("aria-label=\"Equipment queue\""));
    assert!(tag(&html, "id", "events").contains("--ix-event-height:40px"));
    assert!(tag(&html, "id", "row-a").contains("--ix-event-status:var(--ix-color-primary)"));
    let primary = row(&html, "row-a");
    assert!(bool_attr(
        primary
            .split('<')
            .find(|tag| tag.starts_with("button "))
            .unwrap(),
        "data-selected",
        true
    ));
    assert!(bool_attr(
        primary
            .split('<')
            .find(|tag| tag.starts_with("button "))
            .unwrap(),
        "data-filled",
        true
    ));
    assert!(primary.contains("ix-event-status-warning"));
    data.config.borrow_mut().color = Some(String::from("--ix-color-primary);background:red"));
    data.config.borrow_mut().height = EventItemHeight::Large;
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(!tag(&html, "id", "row-a").contains("style="));
    assert!(tag(&html, "id", "row-a").contains("ix-event-status-warning"));
    assert!(tag(&html, "id", "events").contains("--ix-event-height:104px"));
    assert!(!html.contains("background:red"));
    data.config.borrow_mut().height = EventItemHeight::Pixels(60);
    data.config.borrow_mut().disabled = true;
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(tag(&html, "id", "events").contains("--ix-event-height:60px"));
    assert!(tag(&html, "aria-label", "Select A").contains("disabled"));
    assert_eq!(data.mounts.borrow().len(), 2);
}
