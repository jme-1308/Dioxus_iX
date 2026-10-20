//! Native form properties belong to the textarea. Actual FormData, validation,
//! focus and controlled reset are verified in the gallery's browser/native probe.
use super::*;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
struct FormConfig {
    form: Option<String>,
    required: bool,
    disabled: bool,
    readonly: bool,
    state: ChatInputState,
    submit_label: String,
    processing_label: String,
    value: String,
}
impl Default for FormConfig {
    fn default() -> Self {
        Self {
            form: Some("thread-form".into()),
            required: true,
            disabled: false,
            readonly: false,
            state: ChatInputState::Input,
            submit_label: "Envoyer le message".into(),
            processing_label: "Arrêter le traitement".into(),
            value: "Inspect Motor 001".into(),
        }
    }
}
fn form_fixture(config: Rc<RefCell<FormConfig>>) -> Element {
    let config = config.borrow().clone();
    rsx! {
        form {id:"thread-form"}
        form {id:"alternate-form"}
        ChatInput {
            id:"thread-composer",name:"prompt",form:config.form,required:config.required,
            disabled:config.disabled,readonly:config.readonly,state:config.state,value:config.value,
            submit_label:config.submit_label,processing_label:config.processing_label,max_length:120,
        }
    }
}
fn setup(config: Rc<RefCell<FormConfig>>) -> VirtualDom {
    let mut dom = VirtualDom::new_with_props(form_fixture, config);
    dom.rebuild_in_place();
    settle(&mut dom);
    dom
}
fn settle(dom: &mut VirtualDom) {
    for _ in 0..3 {
        dom.render_immediate_to_vec();
    }
}
fn refresh(dom: &mut VirtualDom) {
    dom.mark_dirty(ScopeId::APP);
    settle(dom);
}
fn textarea(html: &str) -> &str {
    html.split('<')
        .find(|tag| tag.starts_with("textarea "))
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
fn action<'a>(html: &'a str, label: &str) -> &'a str {
    html.split('<')
        .find(|tag| {
            tag.starts_with("button ")
                && tag
                    .split('>')
                    .next()
                    .unwrap()
                    .contains(&format!("aria-label=\"{label}\""))
        })
        .unwrap_or_else(|| panic!("Missing native action {label}"))
        .split('>')
        .next()
        .unwrap()
}
fn has_attribute(tag: &str, name: &str) -> bool {
    tag.trim_end_matches('/')
        .split_ascii_whitespace()
        .any(|attribute| attribute == name || attribute.starts_with(&format!("{name}=")))
}

#[test]
fn chat_form_native_textarea_association_and_required_follow_owner_updates() {
    let config = Rc::new(RefCell::new(FormConfig::default()));
    let mut dom = setup(config.clone());
    let html = dioxus_ssr::render(&dom);
    let field = textarea(&html);
    for attribute in [
        "id=\"thread-composer\"",
        "name=\"prompt\"",
        "form=\"thread-form\"",
    ] {
        assert!(field.contains(attribute), "missing {attribute}");
    }
    assert!(has_attribute(field, "required"));
    assert!(!has_attribute(field, "disabled"));
    assert!(!has_attribute(field, "readonly"));
    assert!(field.contains("maxlength=120") || field.contains("maxlength=\"120\""));
    {
        let mut config = config.borrow_mut();
        config.form = Some("alternate-form".into());
        config.required = false;
        config.value = "Review Pump 002".into();
    }
    refresh(&mut dom);
    let html = dioxus_ssr::render(&dom);
    let field = textarea(&html);
    assert!(field.contains("form=\"alternate-form\""));
    assert!(!has_attribute(field, "required"));
    assert!(html.contains("Review Pump 002"));
    config.borrow_mut().form = None;
    refresh(&mut dom);
    assert!(!has_attribute(textarea(&dioxus_ssr::render(&dom)), "form"));
    assert_eq!(
        dioxus_ssr::render(&dom).matches("<textarea").count(),
        1,
        "owner association stays on one composed native field"
    );
}

#[test]
fn chat_form_localized_actions_preserve_disabled_readonly_and_processing_policy() {
    let config = Rc::new(RefCell::new(FormConfig::default()));
    let mut dom = setup(config.clone());
    let html = dioxus_ssr::render(&dom);
    assert!(!has_attribute(
        action(&html, "Envoyer le message"),
        "disabled"
    ));
    config.borrow_mut().disabled = true;
    refresh(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(has_attribute(textarea(&html), "disabled"));
    assert!(has_attribute(
        action(&html, "Envoyer le message"),
        "disabled"
    ));
    {
        let mut config = config.borrow_mut();
        config.disabled = false;
        config.readonly = true;
    }
    refresh(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(has_attribute(textarea(&html), "readonly"));
    assert!(!has_attribute(textarea(&html), "disabled"));
    assert!(has_attribute(
        action(&html, "Envoyer le message"),
        "disabled"
    ));
    {
        let mut config = config.borrow_mut();
        config.readonly = false;
        config.state = ChatInputState::Processing;
        config.processing_label = "Stoppa behandlingen".into();
    }
    refresh(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(has_attribute(
        action(&html, "Stoppa behandlingen"),
        "disabled"
    ));
    assert!(
        !has_attribute(textarea(&html), "disabled"),
        "processing keeps native editing policy unchanged"
    );
    assert!(!has_attribute(textarea(&html), "readonly"));
    {
        let mut config = config.borrow_mut();
        config.state = ChatInputState::Input;
        config.submit_label = "Send til inspeksjon".into();
    }
    refresh(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(!has_attribute(
        action(&html, "Send til inspeksjon"),
        "disabled"
    ));
    assert!(!html.contains("Stoppa behandlingen"));
}

#[test]
fn chat_form_new_options_preserve_existing_defaults_and_native_markup() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            ChatInput {value:"Inspect alarms"}
            ChatInput {value:"Inspect alarms",state:ChatInputState::Processing}
        }
    });
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(!has_attribute(textarea(&html), "form"));
    assert!(!has_attribute(textarea(&html), "required"));
    assert!(!has_attribute(action(&html, "Submit prompt"), "disabled"));
    assert!(has_attribute(action(&html, "Stop processing"), "disabled"));
    assert_eq!(html.matches("<textarea").count(), 2);
    assert!(
        !html.contains("<input"),
        "native textarea remains the successful form control without hidden-field duplication"
    );
}
