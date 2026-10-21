//! Value-derived display labels through actual Select rendering and normal handlers.
use super::*;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

fn options() -> Vec<SelectOption> {
    vec![
        SelectOption::new("CODE", ""),
        SelectOption::new("friendly", "Friendly name"),
        SelectOption::new("spaces", "  "),
        SelectOption::new("LOCKED", "").disabled(),
    ]
}
fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let value = tag.split_once(&format!(" {name}="))?.1;
    if let Some(value) = value.strip_prefix('"') {
        value.split_once('"').map(|(value, _)| value)
    } else {
        value.split(|c: char| c.is_whitespace() || c == '>').next()
    }
}
fn input<'a>(html: &'a str, id: &str) -> &'a str {
    html.split('<')
        .find(|tag| tag.starts_with("input ") && attr(tag, "id") == Some(id))
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
fn option<'a>(html: &'a str, value: &str) -> &'a str {
    html.split("<option ")
        .skip(1)
        .find(|tag| attr(&format!(" {tag}"), "value") == Some(value))
        .unwrap()
        .split("</option>")
        .next()
        .unwrap()
}
fn presentation(options: Vec<SelectOption>) -> Element {
    rsx! {
        section {"data-select-label-case":"single",Select {id:"selected-code",options:options.clone(),value:"CODE",name:"single-code",form:"work-form"}}
        section {"data-select-label-case":"multiple",Select {id:"selected-codes",options:options.clone(),multiple:true,values:vec!["CODE".into(),"friendly".into()],name:"multiple-codes",form:"work-form"}}
        section {"data-select-label-case":"nonempty",Select {id:"selected-friendly",options:options.clone(),value:"friendly"}}
        section {"data-select-label-case":"spaces",Select {id:"selected-spaces",options,value:"spaces"}}
    }
}
#[test]
fn empty_option_labels_use_values_in_selected_input_native_options_chips_and_removal_names() {
    let supplied = options();
    let mut dom = VirtualDom::new_with_props(presentation, supplied.clone());
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert_eq!(attr(input(&html, "selected-code"), "value"), Some("CODE"));
    let native = option(&html, "CODE");
    assert!(native.split('>').next().unwrap().contains(" selected"));
    assert_eq!(native.split_once('>').unwrap().1, "CODE");
    assert!(html.contains("class=\"ix-select-chip-label\" title=\"CODE\">CODE</span>"));
    assert!(html.contains("aria-label=\"Remove CODE\""));
    assert!(
        html.contains(
            "class=\"ix-select-chip-label\" title=\"Friendly name\">Friendly name</span>"
        )
    );
    assert!(html.contains("aria-label=\"Remove Friendly name\""));
    assert_eq!(
        attr(input(&html, "selected-friendly"), "value"),
        Some("Friendly name")
    );
    assert_eq!(attr(input(&html, "selected-spaces"), "value"), Some("  "));
    assert_eq!(option(&html, "spaces").split_once('>').unwrap().1, "  ");
    assert!(html.contains("name=\"single-code\"") && html.contains("name=\"multiple-codes\""));
    assert!(html.contains("form=\"work-form\""));
    assert_eq!(supplied[0].label, "");
    assert_eq!(supplied[2].label, "  ");
}

#[derive(Clone)]
struct Settings {
    options: Vec<SelectOption>,
    value: String,
    disabled: bool,
    readonly: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            options: options(),
            value: String::new(),
            disabled: false,
            readonly: false,
        }
    }
}
#[derive(Default)]
struct Observation {
    settings: Cell<Option<Signal<Settings>>>,
    requests: RefCell<Vec<String>>,
    list_requests: RefCell<Vec<Vec<String>>>,
    additions: RefCell<Vec<String>>,
}
fn fixture(data: Rc<Observation>) -> Element {
    let settings = use_signal(Settings::default);
    data.settings.set(Some(settings));
    let current = settings();
    let value = data.clone();
    let values = data.clone();
    let additions = data.clone();
    rsx! {Select {id:"label-select",name:"asset-code",form:"work-form",options:current.options,value:current.value,disabled:current.disabled,readonly:current.readonly,editable:true,allow_clear:true,
    onvaluechange:move|next|value.requests.borrow_mut().push(next),onvalueschange:move|next|values.list_requests.borrow_mut().push(next),onadd:move|next|additions.additions.borrow_mut().push(next)}}
}
struct Mouse;
impl dioxus::html::InteractionLocation for Mouse {
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
struct Keyboard(Key);
impl dioxus::html::ModifiersInteraction for Keyboard {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::HasKeyboardData for Keyboard {
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
struct Form(String);
impl dioxus::html::HasFileData for Form {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for Form {
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
struct Converter;
macro_rules! select_label_converter {
    (enum Event {$(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)*})=>{
        impl dioxus::html::HtmlEventConverter for Converter {$(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;select_label_converter!(@data $group,event)})*}
    };
    (@data Mouse,$event:ident)=>{MouseData::new(Mouse)};
    (@data Keyboard,$event:ident)=>{KeyboardData::new(Keyboard($event.downcast::<Key>().unwrap().clone()))};
    (@data Form,$event:ident)=>{FormData::new(Form($event.downcast::<String>().unwrap().clone()))};
    (@data $other:ident,$event:ident)=>{panic!("Unexpected Select label test event")};
}
dioxus::html::with_html_event_groups!(select_label_converter);
fn collect(
    edits: dioxus::core::Mutations,
    targets: &mut BTreeMap<String, dioxus::core::ElementId>,
) {
    let mut current = dioxus::core::ElementId::ROOT;
    for edit in edits.edits {
        match edit {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                current = id
            }
            dioxus::core::Mutation::SetAttribute {
                name,
                value: dioxus::core::AttributeValue::Text(value),
                ..
            } if name == "id" || name == "aria-label" => {
                targets.insert(value, current);
            }
            _ => {}
        }
    }
}
fn settle(dom: &mut VirtualDom, targets: &mut BTreeMap<String, dioxus::core::ElementId>) {
    for _ in 0..6 {
        collect(dom.render_immediate_to_vec(), targets);
        dom.process_events();
    }
}
fn event(
    dom: &VirtualDom,
    name: &str,
    id: dioxus::core::ElementId,
    payload: impl std::any::Any,
) -> bool {
    let data =
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload))) as Rc<dyn std::any::Any>;
    let event = Event::new(data, true);
    dom.runtime().handle_event(name, event.clone(), id);
    event.default_action_enabled()
}
#[test]
fn value_derived_labels_filter_select_clear_and_follow_controlled_updates_without_changing_values()
{
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Observation::default());
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let mut targets = BTreeMap::new();
    collect(dom.rebuild_to_vec(), &mut targets);
    let id = targets["label-select"];
    let mut settings = data.settings.get().unwrap();
    assert!(!event(&dom, "keydown", id, Key::ArrowDown));
    settle(&mut dom, &mut targets);
    assert_eq!(
        attr(
            input(&dioxus_ssr::render(&dom), "label-select"),
            "aria-expanded"
        ),
        Some("true")
    );
    event(&dom, "input", id, String::from("CODE"));
    settle(&mut dom, &mut targets);
    let filtered = dioxus_ssr::render(&dom);
    assert!(filtered.contains("class=\"ix-select-option-label\">CODE</span>"));
    assert!(
        !filtered.contains("aria-label=\"Add item: CODE\""),
        "the existing value-derived label is not an editable duplicate"
    );
    assert!(!filtered.contains("class=\"ix-select-option-label\">Friendly name</span>"));
    event(&dom, "keydown", id, Key::ArrowDown);
    settle(&mut dom, &mut targets);
    event(&dom, "keydown", id, Key::Enter);
    settle(&mut dom, &mut targets);
    assert_eq!(&*data.requests.borrow(), &[String::from("CODE")]);
    assert_eq!(&*data.list_requests.borrow(), &[vec![String::from("CODE")]]);
    assert!(data.additions.borrow().is_empty());
    assert!(
        settings.peek().value.is_empty(),
        "the consumer still owns selected values"
    );
    dom.in_runtime(|| settings.with_mut(|current| current.value = "CODE".into()));
    settle(&mut dom, &mut targets);
    let selected = dioxus_ssr::render(&dom);
    assert_eq!(
        attr(input(&selected, "label-select"), "value"),
        Some("CODE")
    );
    assert!(
        option(&selected, "CODE")
            .split('>')
            .next()
            .unwrap()
            .contains(" selected")
    );
    event(&dom, "click", targets["Clear selection"], ());
    settle(&mut dom, &mut targets);
    assert_eq!(data.requests.borrow().last().map(String::as_str), Some(""));
    dom.in_runtime(|| settings.with_mut(|current| current.value.clear()));
    settle(&mut dom, &mut targets);
    event(&dom, "input", id, String::from("LOCKED"));
    settle(&mut dom, &mut targets);
    let locked = dioxus_ssr::render(&dom);
    assert!(locked.contains("class=\"ix-select-option-label\">LOCKED</span>"));
    assert!(!locked.contains("aria-label=\"Add item: LOCKED\""));
    event(&dom, "keydown", id, Key::ArrowDown);
    event(&dom, "keydown", id, Key::Enter);
    settle(&mut dom, &mut targets);
    assert_eq!(
        data.requests.borrow().len(),
        2,
        "disabled option is skipped by normal keyboard selection"
    );
    event(&dom, "keydown", id, Key::Escape);
    settle(&mut dom, &mut targets);
    dom.in_runtime(|| {
        settings.with_mut(|current| {
            current.value = "CODE".into();
            current.options[0].label = "Renamed code".into();
        })
    });
    settle(&mut dom, &mut targets);
    assert_eq!(
        attr(input(&dioxus_ssr::render(&dom), "label-select"), "value"),
        Some("Renamed code")
    );
    for (disabled, readonly) in [(false, true), (true, false)] {
        dom.in_runtime(|| {
            settings.with_mut(|current| {
                current.disabled = disabled;
                current.readonly = readonly;
            })
        });
        settle(&mut dom, &mut targets);
        event(&dom, "input", id, String::from("Friendly"));
        assert!(event(&dom, "keydown", id, Key::ArrowDown));
        settle(&mut dom, &mut targets);
        let html = dioxus_ssr::render(&dom);
        assert_eq!(
            attr(input(&html, "label-select"), "value"),
            Some("Renamed code")
        );
        assert_eq!(
            attr(input(&html, "label-select"), "aria-expanded"),
            Some("false")
        );
        assert_eq!(data.requests.borrow().len(), 2);
    }
    assert_eq!(settings.peek().value, "CODE");
    assert_eq!(settings.peek().options[3].label, "");
}
