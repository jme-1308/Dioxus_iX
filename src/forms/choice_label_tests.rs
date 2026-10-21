//! Actual choice rendering, handlers and Dioxus-owned composed label lifetimes.
use super::*;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let value = tag.split_once(&format!(" {name}="))?.1;
    if let Some(value) = value.strip_prefix('"') {
        value.split_once('"').map(|(value, _)| value)
    } else {
        value.split(|c: char| c.is_whitespace() || c == '>').next()
    }
}
fn control<'a>(html: &'a str, id: &str) -> &'a str {
    html.split('<')
        .filter(|tag| tag.starts_with("button ") || tag.starts_with("input "))
        .find(|tag| attr(tag, "id") == Some(id))
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
fn case<'a>(html: &'a str, name: &str) -> &'a str {
    html.split_once(&format!("data-choice-case=\"{name}\""))
        .unwrap()
        .1
        .split("</section>")
        .next()
        .unwrap()
}
#[test]
fn composed_choice_labels_keep_plain_defaults_and_explicit_semantic_names() {
    let mut dom = VirtualDom::new(|| {
        let renderer = use_callback(
            |option: SelectOption| rsx! {strong {"{option.label}"} small {" ({option.value})"}},
        );
        rsx! {
            section {"data-choice-case":"plain-check",Checkbox {id:"plain-check",label:"Plain checkbox"}}
            section {"data-choice-case":"composed-check",Checkbox {id:"composed-check",label:"Semantic checkbox",strong {"Visible check"}}}
            section {"data-choice-case":"override-check",Checkbox {id:"override-check",label:"Fallback checkbox",aria_label:"Explicit checkbox",span {"Visible override"}}}
            section {"data-choice-case":"child-check",Checkbox {id:"child-check",span {"Child checkbox text"}}}
            section {"data-choice-case":"plain-radio",Radio {id:"plain-radio",value:"plain",label:"Plain radio"}}
            section {"data-choice-case":"composed-radio",Radio {id:"composed-radio",value:"composed",label:"Semantic radio",em {"Visible radio"}}}
            section {"data-choice-case":"child-radio",Radio {id:"child-radio",value:"child",aria_label:"Explicit child radio",span {"Child radio text"}}}
            section {"data-choice-case":"plain-group",RadioGroup {name:"plain-group",options:vec![SelectOption::new("a","Plain group option")],value:"a"}}
            section {"data-choice-case":"composed-group",RadioGroup {name:"composed-group",options:vec![SelectOption::new("a","Semantic group option")],value:"a",render_label:Some(renderer)}}
        }
    });
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert_eq!(attr(control(&html, "plain-check"), "aria-label"), None);
    assert!(case(&html, "plain-check").contains(">Plain checkbox</div>"));
    assert_eq!(
        attr(control(&html, "composed-check"), "aria-label"),
        Some("Semantic checkbox")
    );
    assert!(case(&html, "composed-check").contains("<strong>Visible check</strong>"));
    assert!(!case(&html, "composed-check").contains(">Semantic checkbox<"));
    assert_eq!(
        attr(control(&html, "override-check"), "aria-label"),
        Some("Explicit checkbox")
    );
    assert_eq!(attr(control(&html, "child-check"), "aria-label"), None);
    assert!(case(&html, "child-check").contains("data-label-less=\"false\""));
    assert_eq!(
        attr(control(&html, "plain-radio"), "aria-label"),
        Some("Plain radio")
    );
    assert!(case(&html, "plain-radio").contains(">Plain radio</div>"));
    assert_eq!(
        attr(control(&html, "composed-radio"), "aria-label"),
        Some("Semantic radio")
    );
    assert!(case(&html, "composed-radio").contains("<em>Visible radio</em>"));
    assert_eq!(
        attr(control(&html, "child-radio"), "aria-label"),
        Some("Explicit child radio")
    );
    assert!(
        case(&html, "child-radio").contains("ix-radio-label")
            && case(&html, "child-radio").contains("data-label-less=\"false\"")
    );
    assert!(case(&html, "plain-group").contains("aria-label=\"Plain group option\""));
    let group = case(&html, "composed-group");
    assert!(group.contains("aria-label=\"Semantic group option\""));
    assert!(group.contains("<strong>Semantic group option</strong><small> (a)</small>"));
}

#[derive(Clone)]
struct Settings {
    checked: bool,
    mixed: bool,
    disabled: bool,
    name: String,
    value: String,
    label: String,
    hide_content: bool,
    options: Vec<SelectOption>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            checked: false,
            mixed: true,
            disabled: false,
            name: "choice".into(),
            value: "day".into(),
            label: "Current choice".into(),
            hide_content: false,
            options: vec![
                SelectOption::new("a", "Option A"),
                SelectOption::new("b", "Option B"),
            ],
        }
    }
}
#[derive(Default)]
struct Observation {
    settings: Cell<Option<Signal<Settings>>>,
    checks: RefCell<Vec<bool>>,
    radio_checks: RefCell<Vec<bool>>,
    radio_values: RefCell<Vec<String>>,
    group_values: RefCell<Vec<String>>,
    private: RefCell<BTreeMap<String, Signal<String>>>,
    mounts: RefCell<BTreeMap<String, usize>>,
    drops: RefCell<BTreeMap<String, usize>>,
}
impl PartialEq for Observation {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn LocalChoiceLabel(data: Rc<Observation>, identity: String, text: String) -> Element {
    use_hook({
        let data = data.clone();
        let identity = identity.clone();
        move || *data.mounts.borrow_mut().entry(identity).or_default() += 1
    });
    use_drop({
        let data = data.clone();
        let identity = identity.clone();
        move || *data.drops.borrow_mut().entry(identity).or_default() += 1
    });
    let local = use_signal(|| String::from("Initial private text"));
    data.private.borrow_mut().insert(identity.clone(), local);
    let hidden = (data.settings.get().unwrap())().hide_content;
    rsx! {span {"data-choice-label":identity,hidden,strong {"{text}"} small {"{local}"}}}
}
fn fixture(data: Rc<Observation>) -> Element {
    let settings = use_signal(Settings::default);
    data.settings.set(Some(settings));
    let current = settings();
    let check = data.clone();
    let radio = data.clone();
    let radio_value = data.clone();
    let group = data.clone();
    let renderer = use_callback({
        let data = data.clone();
        move |option: SelectOption| rsx! {LocalChoiceLabel {data:data.clone(),identity:option.value,text:option.label}}
    });
    rsx! {
        Checkbox {id:"live-check",label:current.label.clone(),name:current.name.clone(),value:current.value.clone(),form:"work-form",checked:current.checked,indeterminate:current.mixed,disabled:current.disabled,
            oncheckedchange:move|next|check.checks.borrow_mut().push(next),
            LocalChoiceLabel {data:data.clone(),identity:"checkbox",text:current.label.clone()}
        }
        Radio {id:"live-radio",label:current.label.clone(),name:current.name.clone(),value:current.value.clone(),form:"work-form",checked:current.checked,disabled:current.disabled,
            oncheckedchange:move|next|radio.radio_checks.borrow_mut().push(next),onvaluechange:move|next|radio_value.radio_values.borrow_mut().push(next),
            LocalChoiceLabel {data:data.clone(),identity:"radio",text:current.label}
        }
        RadioGroup {name:"group-choice",options:current.options,value:"a",render_label:Some(renderer),
            onvaluechange:move|next|group.group_values.borrow_mut().push(next)}
    }
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
struct Form;
impl dioxus::html::HasFileData for Form {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for Form {
    fn value(&self) -> String {
        String::new()
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
macro_rules! choice_label_converter {
    (enum Event {$(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)*})=>{
        impl dioxus::html::HtmlEventConverter for Converter {$(fn $convert(&self,_:&dioxus::html::PlatformEventData)->dioxus::html::$data {choice_label_converter!(@data $group)})*}
    };
    (@data Mouse)=>{MouseData::new(Mouse)};
    (@data Form)=>{FormData::new(Form)};
    (@data $other:ident)=>{panic!("Unexpected composed choice test event")};
}
dioxus::html::with_html_event_groups!(choice_label_converter);
fn setup() -> (
    VirtualDom,
    Rc<Observation>,
    BTreeMap<String, dioxus::core::ElementId>,
) {
    let data = Rc::new(Observation::default());
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let mut current = dioxus::core::ElementId::ROOT;
    let mut ids = BTreeMap::new();
    for edit in dom.rebuild_to_vec().edits {
        match edit {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                current = id
            }
            dioxus::core::Mutation::SetAttribute {
                name,
                value: dioxus::core::AttributeValue::Text(value),
                ..
            } if name == "id" => {
                ids.insert(value, current);
            }
            _ => {}
        }
    }
    (dom, data, ids)
}
fn settle(dom: &mut VirtualDom) {
    for _ in 0..6 {
        dom.render_immediate_to_vec();
        dom.process_events();
    }
}
fn dispatch(dom: &VirtualDom, event: &str, id: dioxus::core::ElementId) {
    let payload =
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(()))) as Rc<dyn std::any::Any>;
    dom.runtime()
        .handle_event(event, Event::new(payload, true), id);
}
#[test]
fn composed_choices_keep_controlled_mixed_guard_callbacks_and_native_form_fields() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let (mut dom, data, ids) = setup();
    let mut settings = data.settings.get().unwrap();
    let html = dioxus_ssr::render(&dom);
    assert_eq!(
        attr(control(&html, "live-check"), "aria-checked"),
        Some("mixed")
    );
    dispatch(&dom, "click", ids["live-check"]);
    dispatch(&dom, "change", ids["live-radio"]);
    assert_eq!(&*data.checks.borrow(), &[true]);
    assert_eq!(&*data.radio_checks.borrow(), &[true]);
    assert_eq!(&*data.radio_values.borrow(), &[String::from("day")]);
    assert!(
        !settings.peek().checked,
        "callbacks do not replace controlled props"
    );
    dom.in_runtime(|| {
        settings.with_mut(|current| {
            current.checked = true;
            current.mixed = false;
            current.name = "accepted-choice".into();
            current.value = "night".into();
        })
    });
    settle(&mut dom);
    let html = dioxus_ssr::render(&dom);
    let mirror = html
        .split('<')
        .find(|tag| tag.starts_with("input ") && attr(tag, "type") == Some("checkbox"))
        .unwrap();
    for (name, value) in [
        ("name", "accepted-choice"),
        ("value", "night"),
        ("form", "work-form"),
        ("aria-hidden", "true"),
    ] {
        assert_eq!(attr(mirror, name), Some(value));
    }
    assert!(mirror.split('>').next().unwrap().contains(" checked"));
    assert_eq!(attr(control(&html, "live-radio"), "value"), Some("night"));
    dispatch(&dom, "click", ids["live-check"]);
    dispatch(&dom, "change", ids["live-radio"]);
    assert_eq!(&*data.checks.borrow(), &[true, false]);
    assert_eq!(
        data.radio_values.borrow().last().map(String::as_str),
        Some("night")
    );
    dom.in_runtime(|| settings.with_mut(|current| current.disabled = true));
    settle(&mut dom);
    dispatch(&dom, "click", ids["live-check"]);
    assert_eq!(&*data.checks.borrow(), &[true, false]);
    let html = dioxus_ssr::render(&dom);
    assert!(control(&html, "live-radio").contains(" disabled"));
    assert_eq!(
        attr(control(&html, "live-check"), "aria-disabled"),
        Some("true")
    );
}
#[test]
fn composed_choice_and_group_children_retain_private_scopes_through_live_props_and_reorder() {
    let (mut dom, data, _) = setup();
    let mut settings = data.settings.get().unwrap();
    for key in ["checkbox", "radio", "a", "b"] {
        let mut private = data.private.borrow()[key];
        dom.in_runtime(|| private.set(format!("Retained {key} draft")));
    }
    for hidden in [true, false] {
        dom.in_runtime(|| {
            settings.with_mut(|current| {
                current.checked = !current.checked;
                current.mixed = !current.mixed;
                current.name = "new-name".into();
                current.value = "new-value".into();
                current.label = if hidden {
                    String::new()
                } else {
                    "Updated semantic choice".into()
                };
                current.hide_content = hidden;
                current.options.reverse();
                for option in &mut current.options {
                    option.label = format!("Updated {}", option.value);
                }
            })
        });
        settle(&mut dom);
        let html = dioxus_ssr::render(&dom);
        for key in ["checkbox", "radio", "a", "b"] {
            assert!(html.contains(&format!("Retained {key} draft")));
            assert_eq!(data.mounts.borrow().get(key), Some(&1));
            assert_eq!(data.drops.borrow().get(key), None);
        }
        assert!(!html.contains("data-label-less=\"true\""));
        assert!(
            html.contains("<strong>Updated a</strong>")
                && html.contains("<strong>Updated b</strong>")
        );
        for option in ["a", "b"] {
            assert!(html.contains(&format!("aria-label=\"Updated {option}\"")));
        }
    }
    drop(dom);
    for key in ["checkbox", "radio", "a", "b"] {
        assert_eq!(data.drops.borrow().get(key), Some(&1));
    }
}
