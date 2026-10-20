// Reuses the existing forms converter; native tooltip timing/positioning is
// covered by shared Tooltip tests and the representative browser workflow.
struct GroupFeedbackMouse;
impl dioxus::html::InteractionLocation for GroupFeedbackMouse {
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
impl dioxus::html::InteractionElementOffset for GroupFeedbackMouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for GroupFeedbackMouse {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for GroupFeedbackMouse {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for GroupFeedbackMouse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct GroupFeedbackKey(Key);
impl dioxus::html::ModifiersInteraction for GroupFeedbackKey {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::HasKeyboardData for GroupFeedbackKey {
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
struct GroupFeedbackForm(String);
impl dioxus::html::HasFileData for GroupFeedbackForm {
    fn files(&self) -> Vec<FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for GroupFeedbackForm {
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
fn feedback_attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let value = tag.split_once(&format!(" {name}="))?.1;
    if let Some(value) = value.strip_prefix('"') {
        Some(value.split('"').next().unwrap())
    } else {
        Some(
            value
                .split(|ch: char| ch.is_whitespace() || ch == '>')
                .next()
                .unwrap(),
        )
    }
}
fn feedback_group<'a>(html: &'a str, class: &str) -> &'a str {
    html.split('<')
        .find(|part| {
            part.starts_with("fieldset ")
                && feedback_attribute(part, "class")
                    .is_some_and(|classes| classes.split_whitespace().any(|value| value == class))
        })
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
fn feedback_description<'a>(html: &'a str, class: &str) -> Option<&'a str> {
    feedback_attribute(feedback_group(html, class), "aria-describedby")
}
fn feedback_tag<'a>(html: &'a str, id: &str) -> &'a str {
    html.split('<')
        .find(|part| feedback_attribute(part, "id") == Some(id))
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
#[derive(Default)]
struct FeedbackFixture {
    tooltip: std::cell::Cell<bool>,
    message: std::cell::RefCell<Option<String>>,
    visible: std::cell::Cell<bool>,
    checked: std::cell::Cell<bool>,
    value: std::cell::RefCell<String>,
    mounts: std::cell::Cell<usize>,
    calls: std::cell::RefCell<Vec<String>>,
}
impl PartialEq for FeedbackFixture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn FeedbackPreparation(data: Rc<FeedbackFixture>) -> Element {
    use_hook(|| data.mounts.set(data.mounts.get() + 1));
    let mut note = use_signal(|| String::from("Prepare tools"));
    rsx! {input{id:String::from("feedback-preparation"),value:note(),oninput:move|event|note.set(event.value())}}
}
fn group_feedback_fixture(data: Rc<FeedbackFixture>) -> Element {
    let checks = data.clone();
    let radios = data.clone();
    let message = data.message.borrow().clone();
    rsx! {
        if data.visible.get(){
            CheckboxGroup {label:"Protections",required:true,show_text_as_tooltip:data.tooltip.get(),state:FieldState::Invalid,invalid_text:message.clone(),
                Checkbox{id:"feedback-check",label:"Isolate",checked:data.checked.get(),required:true,oncheckedchange:move|checked|{checks.checked.set(checked);checks.calls.borrow_mut().push(format!("check:{checked}"));}}
                FeedbackPreparation{data:data.clone()}
            }
            RadioGroup {label:"Priority",name:"feedback-priority",required:true,show_text_as_tooltip:data.tooltip.get(),state:FieldState::Info,
                info_text:message,value:data.value.borrow().clone(),options:vec![SelectOption::new("normal","Normal"),SelectOption::new("urgent","Urgent")],
                onvaluechange:move|value:String|{*radios.value.borrow_mut()=value.clone();radios.calls.borrow_mut().push(value);}
            }
        }
    }
}
fn feedback_dom() -> (
    VirtualDom,
    Rc<FeedbackFixture>,
    std::collections::BTreeMap<String, dioxus::core::ElementId>,
) {
    let data = Rc::new(FeedbackFixture::default());
    data.visible.set(true);
    *data.message.borrow_mut() = Some("Review guidance".into());
    *data.value.borrow_mut() = "normal".into();
    let mut dom = VirtualDom::new_with_props(group_feedback_fixture, data.clone());
    let changes = dom.rebuild_to_vec();
    let mut ids = std::collections::BTreeMap::new();
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
            } if name == "id" => {
                ids.insert(value, current);
            }
            _ => {}
        }
    }
    (dom, data, ids)
}
fn feedback_refresh(dom: &mut VirtualDom) {
    dom.mark_dirty(ScopeId::APP);
    for _ in 0..6 {
        dom.render_immediate_to_vec();
        dom.process_events();
    }
}
fn feedback_event(
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
fn group_feedback_inline_and_tooltip_describe_exact_active_messages_without_style_changes() {
    fn app() -> Element {
        rsx! {
            CheckboxGroup{label:"Checks",helper_text:"Select protection",invalid_text:"Inactive error",show_text_as_tooltip:true,Checkbox{label:"Isolate"}}
            RadioGroup{label:"Priority",state:FieldState::Warning,warning_text:"Plan supervision",options:vec![SelectOption::new("normal","Normal")]}
            CheckboxGroup{class:"blank-feedback",show_text_as_tooltip:true,helper_text:" ",span{"No feedback"}}
        }
    }
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    let checks = feedback_description(&html, "ix-checkbox-group").unwrap();
    let radios = feedback_description(&html, "ix-radio-group").unwrap();
    assert_ne!(checks, radios);
    assert_eq!(
        feedback_attribute(feedback_tag(&html, checks), "role"),
        Some("tooltip")
    );
    assert!(
        feedback_attribute(feedback_tag(&html, radios), "class")
            .unwrap()
            .contains("ix-group-bottom-text")
    );
    let tooltip_text = html
        .split('<')
        .find(|tag| {
            feedback_attribute(tag, "class").is_some_and(|classes| {
                classes
                    .split_whitespace()
                    .any(|class| class == "ix-group-bottom-text")
            }) && feedback_attribute(tag, "id").is_none()
        })
        .unwrap();
    assert!(
        !feedback_attribute(tooltip_text, "class")
            .unwrap()
            .split_whitespace()
            .any(|class| class.starts_with("ix-text-")),
        "tooltip feedback inherits the panel foreground"
    );
    assert!(
        feedback_attribute(feedback_tag(&html, radios), "class")
            .unwrap()
            .split_whitespace()
            .any(|class| class == "ix-text-std"),
        "inline feedback keeps its existing semantic color"
    );
    assert_eq!(html.matches("Select protection").count(), 1);
    assert_eq!(html.matches("Plan supervision").count(), 1);
    assert!(!html.contains("Inactive error"));
    assert_eq!(html.matches("role=\"tooltip\"").count(), 1);
    assert_eq!(html.matches("ix-group-bottom-right").count(), 3);
    assert!(
        feedback_attribute(feedback_group(&html, "blank-feedback"), "aria-describedby").is_none()
    );
}
#[test]
fn group_feedback_live_modes_messages_and_remount_clean_descriptions_preserving_preparation() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(PaneFormsConverter));
    let (mut dom, data, ids) = feedback_dom();
    let original = dioxus_ssr::render(&dom);
    let id = feedback_description(&original, "ix-checkbox-group")
        .unwrap()
        .to_string();
    feedback_event(
        &dom,
        "input",
        ids["feedback-preparation"],
        String::from("Tools ready"),
    );
    feedback_refresh(&mut dom);
    data.tooltip.set(true);
    feedback_refresh(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert_eq!(
        feedback_description(&html, "ix-checkbox-group"),
        Some(id.as_str())
    );
    assert_eq!(html.matches("role=\"tooltip\"").count(), 2);
    for tag in html.split('<').filter(|tag| {
        feedback_attribute(tag, "class").is_some_and(|classes| {
            classes
                .split_whitespace()
                .any(|class| class == "ix-group-bottom-text")
        })
    }) {
        assert!(
            !feedback_attribute(tag, "class")
                .unwrap()
                .split_whitespace()
                .any(|class| class.starts_with("ix-text-")),
            "validation feedback also inherits the tooltip foreground"
        );
    }
    *data.message.borrow_mut() = Some("Updated guidance".into());
    feedback_refresh(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains("Review guidance"));
    assert!(html.contains("Updated guidance"));
    data.tooltip.set(false);
    feedback_refresh(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert_eq!(
        feedback_description(&html, "ix-checkbox-group"),
        Some(id.as_str())
    );
    assert!(!html.contains("role=\"tooltip\""));
    *data.message.borrow_mut() = Some(" ".into());
    feedback_refresh(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(feedback_description(&html, "ix-checkbox-group").is_none());
    *data.message.borrow_mut() = None;
    feedback_refresh(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(feedback_description(&html, "ix-radio-group").is_none());
    assert!(!html.contains(&format!("id=\"{id}\"")));
    assert!(feedback_tag(&html, "feedback-preparation").contains("value=\"Tools ready\""));
    assert_eq!(data.mounts.get(), 1);
    data.visible.set(false);
    feedback_refresh(&mut dom);
    assert!(!dioxus_ssr::render(&dom).contains(&id));
    data.visible.set(true);
    *data.message.borrow_mut() = Some("Restored guidance".into());
    feedback_refresh(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert_ne!(
        feedback_description(&html, "ix-checkbox-group"),
        Some(id.as_str())
    );
    assert_eq!(data.mounts.get(), 2);
    assert!(data.calls.borrow().is_empty());
}
#[test]
fn group_feedback_does_not_intercept_native_choice_keys_or_controlled_requests() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(PaneFormsConverter));
    let (mut dom, data, ids) = feedback_dom();
    let check = ids["feedback-check"];
    assert!(feedback_event(
        &dom,
        "keydown",
        check,
        Key::Character(" ".into())
    ));
    assert!(data.calls.borrow().is_empty());
    feedback_event(&dom, "click", check, ());
    feedback_refresh(&mut dom);
    assert!(data.checked.get());
    let html = dioxus_ssr::render(&dom);
    let radio = html
        .split('<')
        .find(|tag| tag.starts_with("input ") && feedback_attribute(tag, "value") == Some("urgent"))
        .unwrap();
    let radio_id = feedback_attribute(radio, "id").unwrap().to_string();
    assert!(feedback_event(
        &dom,
        "keydown",
        ids[&radio_id],
        Key::ArrowRight
    ));
    assert!(feedback_event(
        &dom,
        "change",
        ids[&radio_id],
        String::from("urgent")
    ));
    feedback_refresh(&mut dom);
    assert_eq!(&*data.value.borrow(), "urgent");
    assert_eq!(&*data.calls.borrow(), &["check:true", "urgent"]);
    data.tooltip.set(true);
    feedback_refresh(&mut dom);
    assert!(feedback_event(&dom, "keydown", check, Key::Escape));
    let html = dioxus_ssr::render(&dom);
    assert!(feedback_tag(&html, "feedback-check").contains("aria-checked=\"true\""));
    assert_eq!(html.matches("role=\"tooltip\"").count(), 2);
    let radio = feedback_tag(&html, &radio_id);
    assert!(radio.contains("required"));
    assert!(radio.contains("checked"));
    assert_eq!(&*data.calls.borrow(), &["check:true", "urgent"]);
}
