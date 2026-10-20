use super::*;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
#[derive(Clone)]
struct Config {
    value: Option<String>,
    position: KeyValueLabelPosition,
    horizontal: bool,
    striped: bool,
    reverse: bool,
    named: Option<String>,
    link: bool,
    label_prefix: String,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            value: None,
            position: KeyValueLabelPosition::Top,
            horizontal: false,
            striped: false,
            reverse: false,
            named: Some("Asset properties".into()),
            link: true,
            label_prefix: "Property".into(),
        }
    }
}
#[derive(Default)]
struct Fixture {
    config: RefCell<Config>,
    mounts: RefCell<Vec<String>>,
    actions: RefCell<Vec<String>>,
}
impl PartialEq for Fixture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn CustomValue(key_name: String, data: Rc<Fixture>) -> Element {
    let initial = format!("Draft {key_name}");
    let mut note = use_signal(|| initial);
    let mut changes = use_signal(|| 0usize);
    let mounted_key = key_name.clone();
    let action_key = key_name.clone();
    let action = data.clone();
    rsx! {div{
        input{id:format!("field-{key_name}"),value:note(),oninput:move|event|note.set(event.value()),onmounted:move|_|data.mounts.borrow_mut().push(mounted_key.clone())}
        button{id:format!("action-{key_name}"),r#type:"button",onclick:move|_|{changes+=1;action.actions.borrow_mut().push(action_key.clone());},"Apply {changes}"}
    }}
}
#[component]
fn Property(
    key_name: String,
    data: Rc<Fixture>,
    value: Option<String>,
    position: KeyValueLabelPosition,
    link: bool,
    label_prefix: String,
) -> Element {
    rsx! {KeyValue{id:format!("property-{key_name}"),class:"custom-property",label:format!("{label_prefix} {key_name}"),value,icon:"check",aria_label_icon:"Property icon",label_position:position,label_id:format!("label-{key_name}"),label_for:link.then(||format!("field-{key_name}")),
        CustomValue{key_name,data}
    }}
}
fn fixture(data: Rc<Fixture>) -> Element {
    let config = data.config.borrow().clone();
    let keys = if config.reverse {
        ["b", "a"]
    } else {
        ["a", "b"]
    };
    rsx! {KeyValueList{id:"properties",class:"custom-properties",aria_label:config.named,horizontal:config.horizontal,striped:config.striped,
        for key in keys{Property{key:"{key}",key_name:key.to_string(),data:data.clone(),value:config.value.clone(),position:config.position,link:config.link,label_prefix:config.label_prefix.clone()}}
    }}
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
        (@data $other:ident,$event:ident) => { panic!("Unexpected key-value test event") };
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
fn true_attribute(tag: &str, name: &str) -> bool {
    tag.trim_end_matches('/')
        .split_ascii_whitespace()
        .any(|attribute| {
            attribute == name
                || attribute == format!("{name}=true")
                || attribute == format!("{name}=\"true\"")
        })
}
fn custom_tag(html: &str) -> &str {
    tag(html, "class", "ix-key-value-custom")
}
#[test]
fn key_value_text_and_empty_text_take_precedence_without_destroying_custom_drafts() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let (mut dom, mut targets) = setup(data.clone());
    assert!(!custom_tag(&dioxus_ssr::render(&dom)).contains("inert"));
    dispatch(
        &dom,
        "input",
        targets.attr("id", "field-a"),
        Box::new(String::from("Staged tools")),
    );
    dispatch(&dom, "click", targets.attr("id", "action-a"), Box::new(()));
    settle(&mut dom, &mut targets);
    assert_eq!(data.actions.borrow().as_slice(), ["a"]);
    data.config.borrow_mut().value = Some("Stored value".into());
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("Stored value"));
    assert!(true_attribute(custom_tag(&html), "hidden"));
    assert!(custom_tag(&html).contains("inert"));
    assert!(!html.contains("for=\"field-a\""));
    assert!(tag(&html, "id", "field-a").contains("value=\"Staged tools\""));
    assert!(html.contains("Apply 1"));
    data.config.borrow_mut().value = Some(String::new());
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains("Stored value"));
    assert!(html.contains("class=\"ix-key-value-text\"></span>"));
    assert!(true_attribute(custom_tag(&html), "hidden"));
    assert!(custom_tag(&html).contains("inert"));
    assert!(!html.contains("for=\"field-a\""));
    data.config.borrow_mut().value = None;
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(!true_attribute(custom_tag(&html), "hidden"));
    assert!(!custom_tag(&html).contains("inert"));
    assert!(html.contains("for=\"field-a\""));
    assert!(tag(&html, "id", "field-a").contains("value=\"Staged tools\""));
    dispatch(&dom, "click", targets.attr("id", "action-a"), Box::new(()));
    settle(&mut dom, &mut targets);
    assert_eq!(data.actions.borrow().as_slice(), ["a", "a"]);
    assert_eq!(data.mounts.borrow().as_slice(), ["a", "b"]);
    assert!(dioxus_ssr::render(&dom).contains("Apply 2"));
}
#[test]
fn key_value_explicit_field_labels_icon_names_and_named_list_semantics_are_native() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let (mut dom, mut targets) = setup(data.clone());
    let html = dioxus_ssr::render(&dom);
    let list = tag(&html, "id", "properties");
    assert!(list.contains("role=\"group\""));
    assert!(list.contains("aria-label=\"Asset properties\""));
    assert!(list.contains("custom-properties"));
    assert!(tag(&html, "id", "label-a").starts_with("dt "));
    assert!(html.contains("<label for=\"field-a\">Property a</label>"));
    assert!(tag(&html, "aria-label", "Property icon").contains("role=\"img\""));
    assert!(tag(&html, "id", "property-a").contains("custom-property"));
    data.config.borrow_mut().named = None;
    data.config.borrow_mut().link = false;
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(!tag(&html, "id", "properties").contains("role="));
    assert!(!html.contains("for=\"field-a\""));
    assert!(html.contains("Property a"));
    let mut legacy = VirtualDom::new(
        || rsx! {crate::layout::KeyValueList{crate::layout::KeyValue{label:"Status",value:"Operational",label_position:crate::KeyValueLabelPosition::Left}}},
    );
    legacy.rebuild_in_place();
    let html = dioxus_ssr::render(&legacy);
    assert!(html.contains("Status"));
    assert!(html.contains("Operational"));
    assert!(html.contains("<dl>"));
    assert!(html.contains("<dd>"));
    assert!(!html.contains("role=\"group\""));
}
#[test]
fn key_value_layout_and_keyed_order_updates_preserve_fields_and_callbacks() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let (mut dom, mut targets) = setup(data.clone());
    dispatch(
        &dom,
        "input",
        targets.attr("id", "field-a"),
        Box::new(String::from("Asset A draft")),
    );
    dispatch(
        &dom,
        "input",
        targets.attr("id", "field-b"),
        Box::new(String::from("Asset B draft")),
    );
    settle(&mut dom, &mut targets);
    {
        let mut config = data.config.borrow_mut();
        config.position = KeyValueLabelPosition::Left;
        config.horizontal = true;
        config.striped = true;
        config.reverse = true;
        config.label_prefix = "Updated".into();
    }
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(true_attribute(
        tag(&html, "id", "properties"),
        "data-horizontal"
    ));
    assert!(true_attribute(
        tag(&html, "id", "properties"),
        "data-striped"
    ));
    assert!(true_attribute(
        tag(&html, "id", "property-a"),
        "data-horizontal"
    ));
    assert!(html.find("id=\"property-b\"").unwrap() < html.find("id=\"property-a\"").unwrap());
    assert!(html.contains("Updated a"));
    assert!(tag(&html, "id", "field-a").contains("value=\"Asset A draft\""));
    assert!(tag(&html, "id", "field-b").contains("value=\"Asset B draft\""));
    dispatch(&dom, "click", targets.attr("id", "action-b"), Box::new(()));
    dispatch(&dom, "click", targets.attr("id", "action-a"), Box::new(()));
    settle(&mut dom, &mut targets);
    assert_eq!(data.actions.borrow().as_slice(), ["b", "a"]);
    assert_eq!(data.mounts.borrow().as_slice(), ["a", "b"]);
    {
        let mut config = data.config.borrow_mut();
        config.position = KeyValueLabelPosition::Top;
        config.horizontal = false;
        config.striped = false;
    }
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(!true_attribute(
        tag(&html, "id", "properties"),
        "data-striped"
    ));
    assert!(tag(&html, "id", "field-a").contains("value=\"Asset A draft\""));
    assert_eq!(data.mounts.borrow().as_slice(), ["a", "b"]);
}
