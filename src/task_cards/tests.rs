use super::*;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};
#[derive(Clone)]
struct Config {
    action_passive: bool,
    action_label: Option<String>,
    heading: String,
    icon_label: Option<String>,
    passive: bool,
    selected: bool,
    expanded: Option<bool>,
    variant: CardVariant,
    notification: Option<String>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            action_passive: false,
            action_label: None,
            heading: "Start inspection".into(),
            icon_label: None,
            passive: false,
            selected: false,
            expanded: None,
            variant: CardVariant::Outline,
            notification: Some("3".into()),
        }
    }
}
#[derive(Default)]
struct Fixture {
    config: RefCell<Config>,
    actions: Cell<usize>,
    primary_clicks: Cell<usize>,
    selections: Cell<usize>,
    reviews: Cell<usize>,
    requests: RefCell<Vec<bool>>,
    mounts: Cell<usize>,
}
impl PartialEq for Fixture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn Preparation(data: Rc<Fixture>) -> Element {
    let mut note = use_signal(|| String::from("Prepare tools"));
    let mut checked = use_signal(|| false);
    rsx! { div {
        label {r#for:"task-note","Preparation note"}
        input {id:String::from("task-note"),value:note(),oninput:move|event|note.set(event.value()),onmounted:move |_|data.mounts.set(data.mounts.get()+1)}
        label {input {id:String::from("task-kit"),r#type:"checkbox",checked:checked(),onchange:move|event|checked.set(event.checked())} "Toolkit ready"}
    } }
}
fn fixture(data: Rc<Fixture>) -> Element {
    let config = data.config.borrow().clone();
    let action = data.clone();
    let primary = data.clone();
    let select = data.clone();
    let review = data.clone();
    let request = data.clone();
    rsx! {
        crate::layout::ActionCard {id:"task-action",class:"custom-action",heading:config.heading,icon:"check",subheading:"Motor 001",variant:config.variant,selected:config.selected,passive:config.action_passive,aria_label:config.action_label,aria_label_icon:config.icon_label,
            onclick:move|event:MouseEvent|{action.actions.set(action.actions.get()+1);event.prevent_default();},
            span {"Local checklist"}
        }
        PushCard {id:"task-push",class:"custom-push",heading:"Pending maintenance",icon:"bulb",notification:config.notification,subheading:"Motor 001 queue",variant:config.variant,passive:config.passive,selected:config.selected,expanded:config.expanded,reveal_content:false,
            aria_label_icon:"Pending tasks",details_label:"Task preparation",aria_label_expand_button:"Show preparation",
            onclick:move|_|primary.primary_clicks.set(primary.primary_clicks.get()+1),onselect:move|_|select.selections.set(select.selections.get()+1),
            onexpand:move|value|request.requests.borrow_mut().push(value),
            title_actions:rsx!{button{id:String::from("task-review"),r#type:"button",onclick:move|_|review.reviews.set(review.reviews.get()+1),"Review all"}},
            Preparation{data}
        }
    }
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
        (@data $other:ident,$event:ident) => { panic!("Unexpected task-card test event") };
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
fn open(dom: &VirtualDom) -> bool {
    let html = dioxus_ssr::render(dom);
    let toggle = tag(&html, "aria-label", "Show preparation");
    toggle.contains("aria-expanded=true") || toggle.contains("aria-expanded=\"true\"")
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
fn assert_preparation(dom: &VirtualDom, stage: &str) {
    let html = dioxus_ssr::render(dom);
    let note = tag(&html, "id", "task-note");
    let kit = tag(&html, "id", "task-kit");
    assert!(
        note.contains("value=\"Tools staged\""),
        "{stage}: note input {note}"
    );
    assert!(
        true_attribute(kit, "checked"),
        "{stage}: checkbox input {kit}"
    );
}
#[test]
fn task_cards_action_native_event_passive_guard_and_accessible_presentations() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let (mut dom, mut targets) = setup(data.clone());
    let html = dioxus_ssr::render(&dom);
    let button = tag(&html, "id", "task-action");
    let heading = attribute(button, "aria-labelledby").to_string();
    assert!(tag(&html, "id", &heading).starts_with("strong "));
    assert!(html.contains("Local checklist"));
    assert!(button.contains("custom-action"));
    assert!(!dispatch(
        &dom,
        "click",
        targets.attr("id", "task-action"),
        Box::new(())
    ));
    assert_eq!(data.actions.get(), 1);
    data.config.borrow_mut().action_passive = true;
    refresh(&mut dom, &mut targets);
    dispatch(
        &dom,
        "click",
        targets.attr("id", "task-action"),
        Box::new(()),
    );
    assert_eq!(data.actions.get(), 1);
    assert!(tag(&dioxus_ssr::render(&dom), "id", "task-action").contains("disabled"));
    data.config.borrow_mut().action_passive = false;
    data.config.borrow_mut().action_label = Some("Inspect Motor 001".into());
    data.config.borrow_mut().icon_label = Some("Inspection checklist".into());
    data.config.borrow_mut().selected = true;
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    let button = tag(&html, "id", "task-action");
    assert!(button.contains("aria-label=\"Inspect Motor 001\""));
    assert!(!button.contains("aria-labelledby"));
    assert!(true_attribute(button, "aria-pressed"));
    assert!(html.contains("aria-label=\"Inspection checklist\""));
    data.config.borrow_mut().heading.clear();
    data.config.borrow_mut().action_label = None;
    data.config.borrow_mut().icon_label = None;
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("aria-label=\"check\""));
    assert!(!tag(&html, "id", "task-action").contains("aria-labelledby"));
}
#[test]
fn task_cards_push_actions_disclosure_and_retained_preparation_are_independent() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let (mut dom, mut targets) = setup(data.clone());
    dispatch(
        &dom,
        "click",
        targets.attr("id", "task-review"),
        Box::new(()),
    );
    assert_eq!(data.reviews.get(), 1);
    assert_eq!(data.selections.get(), 0);
    assert!(!open(&dom));
    dispatch(
        &dom,
        "click",
        targets.attr("aria-label", "Select Pending maintenance"),
        Box::new(()),
    );
    assert_eq!(data.selections.get(), 1);
    assert_eq!(data.primary_clicks.get(), 1);
    assert_eq!(data.reviews.get(), 1);
    dispatch(
        &dom,
        "click",
        targets.attr("aria-label", "Show preparation"),
        Box::new(()),
    );
    settle(&mut dom, &mut targets);
    assert!(open(&dom));
    assert_eq!(data.selections.get(), 1);
    dispatch(
        &dom,
        "input",
        targets.attr("id", "task-note"),
        Box::new(String::from("Tools staged")),
    );
    dispatch(
        &dom,
        "change",
        targets.attr("id", "task-kit"),
        Box::new(String::from("true")),
    );
    settle(&mut dom, &mut targets);
    assert_preparation(&dom, "after editing");
    dispatch(
        &dom,
        "click",
        targets.attr("aria-label", "Show preparation"),
        Box::new(()),
    );
    settle(&mut dom, &mut targets);
    dispatch(
        &dom,
        "click",
        targets.attr("aria-label", "Show preparation"),
        Box::new(()),
    );
    settle(&mut dom, &mut targets);
    assert_preparation(&dom, "after close/reopen");
    assert_eq!(data.mounts.get(), 1);
    data.config.borrow_mut().expanded = Some(false);
    refresh(&mut dom, &mut targets);
    dispatch(
        &dom,
        "click",
        targets.attr("aria-label", "Show preparation"),
        Box::new(()),
    );
    settle(&mut dom, &mut targets);
    assert!(!open(&dom));
    assert_eq!(data.requests.borrow().last(), Some(&true));
    data.config.borrow_mut().expanded = Some(true);
    data.config.borrow_mut().variant = CardVariant::Warning;
    data.config.borrow_mut().notification = Some("0".into());
    refresh(&mut dom, &mut targets);
    assert!(open(&dom));
    assert_preparation(&dom, "after external opening and variant/count change");
    data.config.borrow_mut().passive = true;
    refresh(&mut dom, &mut targets);
    assert_preparation(&dom, "after passive true");
    assert_eq!(data.mounts.get(), 1);
    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains("aria-label=\"Select Pending maintenance\""));
    dispatch(
        &dom,
        "click",
        targets.attr("id", "task-review"),
        Box::new(()),
    );
    assert_eq!(data.reviews.get(), 2);
    assert_eq!(data.selections.get(), 1);
    data.config.borrow_mut().passive = false;
    refresh(&mut dom, &mut targets);
    assert_preparation(&dom, "after passive false");
    assert_eq!(data.mounts.get(), 1);
}
#[test]
fn task_cards_keep_legacy_defaults_and_live_root_identity_variant_and_labels() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let (mut dom, mut targets) = setup(data.clone());
    data.config.borrow_mut().variant = CardVariant::Primary;
    data.config.borrow_mut().selected = true;
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    let push = tag(&html, "id", "task-push");
    assert!(push.starts_with("article "));
    assert!(push.contains("ix-push-card custom-push"));
    assert!(push.contains("ix-card-primary"));
    assert!(tag(&html, "id", "task-action").contains("ix-card-primary"));
    assert!(html.contains("Task preparation"));
    assert!(html.contains("aria-label=\"Pending tasks\""));
    let mut legacy = VirtualDom::new(
        || rsx! {crate::layout::ActionCard{heading:"Scan",icon:"refresh"}crate::layout::PushCard{heading:"Legacy",icon:"bulb","Retained details"}},
    );
    legacy.rebuild_in_place();
    let html = dioxus_ssr::render(&legacy);
    assert!(html.contains("width=\"24\"") || html.contains("width=24"));
    assert!(!html.contains("ix-push-notification"));
    assert!(!html.contains("ix-card-select"));
    assert!(html.contains("Retained details"));
    assert!(html.contains("Details"));
    let mut unnamed = VirtualDom::new(|| rsx! {PushCard{passive:false,"Content"}});
    unnamed.rebuild_in_place();
    assert!(dioxus_ssr::render(&unnamed).contains("aria-label=\"Select notification card\""));
}
