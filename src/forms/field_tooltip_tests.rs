//! Presentation and child-state regressions; mounted focus/pointer behavior is
//! verified by the paired web/native workflow rather than inferred from SSR.
use super::*;
use std::cell::Cell;

fn render(component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    settle(&mut dom);
    dioxus_ssr::render(&dom)
}
fn opening<'a>(html: &'a str, tag: &str, marker: &str) -> &'a str {
    html.split('<')
        .filter(|part| part.starts_with(&format!("{tag} ")))
        .map(|part| part.split('>').next().unwrap())
        .find(|part| part.contains(marker))
        .unwrap_or_else(|| panic!("missing {tag} {marker}: {html}"))
}
fn attribute<'a>(opening: &'a str, name: &str) -> &'a str {
    opening
        .split_once(&format!(" {name}=\""))
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap()
}
fn settle(dom: &mut VirtualDom) {
    for _ in 0..6 {
        dom.render_immediate_to_vec();
        dom.process_events();
    }
}
#[test]
fn tooltip_mode_moves_one_helper_target_and_keeps_default_inline_footer_and_native_association() {
    let html = render(|| {
        rsx! {
            FieldWrapper {label:"Default",html_for:"default-control",helper_id:"default-help",helper_text:"Default inline guidance",input {id:"default-control","aria-describedby":"default-help"}}
            FieldWrapper {label:"Explicit inline",html_for:"inline-control",helper_id:"inline-help",helper_text:"Explicit inline guidance",show_text_as_tooltip:false,input {id:"inline-control","aria-describedby":"inline-help"}}
            CustomField {label:"Tooltip",html_for:"tooltip-control",helper_id:"tooltip-help",helper_text:"Contextual guidance",show_text_as_tooltip:true,
                bottom_right:rsx!{button {id:"tooltip-clear",r#type:"button","Clear"}},
                input {id:"tooltip-control","aria-describedby":"tooltip-help"}
            }
        }
    });
    assert_eq!(html.matches("role=\"tooltip\"").count(), 1);
    for (id, text) in [
        ("default-help", "Default inline guidance"),
        ("inline-help", "Explicit inline guidance"),
        ("tooltip-help", "Contextual guidance"),
    ] {
        assert_eq!(html.matches(&format!("id=\"{id}\"")).count(), 1);
        assert!(html.contains(text));
    }
    let panel = opening(&html, "div", "role=\"tooltip\"");
    assert!(panel.contains("ix-field-feedback-tooltip"));
    assert!(panel.contains("data-placement=\"bottom\""));
    assert_ne!(attribute(panel, "id"), "tooltip-help");
    assert!(html.find("id=\"tooltip-help\"").unwrap() > html.find(panel).unwrap());
    assert!(html.find("id=\"tooltip-clear\"").unwrap() < html.find(panel).unwrap());
    assert!(opening(&html, "button", "id=\"tooltip-clear\"").contains("type=\"button\""));
    assert!(
        opening(&html, "input", "id=\"tooltip-control\"")
            .contains("aria-describedby=\"tooltip-help\"")
    );
    assert_eq!(html.matches("for=\"tooltip-control\"").count(), 1);
    assert_eq!(html.matches("ix-field-composed-footer").count(), 1);
}
#[test]
fn blank_feedback_omits_the_panel_while_error_and_helper_fallback_use_one_chosen_message() {
    let html = render(|| {
        rsx! {
            FieldWrapper {helper_id:"empty-help",helper_text:"  ",invalid_text:"\t",state:FieldState::Invalid,show_text_as_tooltip:true,input {id:"empty-control"}}
            CustomField {helper_id:"error-help",helper_text:"Hidden normal help",invalid_text:"Choose an equipment identifier",state:FieldState::Invalid,show_text_as_tooltip:true,input {id:"error-control"}}
            FieldWrapper {helper_id:"fallback-help",helper_text:"Fallback guidance",warning_text:"  ",state:FieldState::Warning,show_text_as_tooltip:true,input {id:"fallback-control"}}
        }
    });
    assert_eq!(html.matches("role=\"tooltip\"").count(), 2);
    assert!(!html.contains("id=\"empty-help\""));
    assert!(!html.contains("Hidden normal help"));
    assert!(html.contains("Choose an equipment identifier"));
    assert!(html.contains("Fallback guidance"));
    assert!(opening(&html, "div", "id=\"error-help\"").contains("role=\"alert\""));
    assert!(opening(&html, "div", "id=\"error-help\"").contains("ix-field-invalid"));
    assert!(opening(&html, "div", "id=\"fallback-help\"").contains("role=\"status\""));
    assert!(!opening(&html, "div", "id=\"fallback-help\"").contains("ix-field-warning"));
}

#[derive(Default)]
struct Capture {
    mode: Cell<Option<Signal<u8>>>,
    mounts: Cell<usize>,
}
impl PartialEq for Capture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn PrivateTooltipDraft(capture: Rc<Capture>) -> Element {
    use_hook(|| capture.mounts.set(capture.mounts.get() + 1));
    let mut draft = use_signal(|| String::from("Initial draft"));
    let id = String::from("tooltip-private-draft");
    rsx! {input {id,value:draft(),oninput:move|event|draft.set(event.value())}}
}
fn retained(capture: Rc<Capture>) -> Element {
    let mode = use_signal(|| 0u8);
    capture.mode.set(Some(mode));
    rsx! {CustomField {label:"Identifier",html_for:"tooltip-private-draft",helper_id:format!("reactive-help-{}",mode()),helper_text:"Draft guidance",invalid_text:"Check identifier",
        show_text_as_tooltip:mode()!=0,state:if mode()==1{FieldState::Invalid}else{FieldState::Default},
        PrivateTooltipDraft{capture:capture.clone()}
    }}
}
struct DraftForm(String);
impl dioxus::html::HasFileData for DraftForm {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for DraftForm {
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
struct DraftConverter;
macro_rules! draft_converter{
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* })=>{
        impl dioxus::html::HtmlEventConverter for DraftConverter{
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data{let _=event;draft_converter!(@data $group,event)})*
        }
    };
    (@data Form,$event:ident)=>{dioxus::html::FormData::new(DraftForm($event.downcast::<String>().unwrap().clone()))};
    (@data $other:ident,$event:ident)=>{panic!("unexpected field tooltip test event")};
}
dioxus::html::with_html_event_groups!(draft_converter);
fn input_id(changes: dioxus::core::Mutations) -> dioxus::core::ElementId {
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
            } if name == "id" && value == "tooltip-private-draft" => return current,
            _ => {}
        }
    }
    panic!("dynamic native draft ID must be mounted")
}
#[test]
fn private_draft_root_and_panel_identity_survive_mode_state_and_helper_id_changes() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(DraftConverter));
    let capture = Rc::new(Capture::default());
    let mut dom = VirtualDom::new_with_props(retained, capture.clone());
    let input = input_id(dom.rebuild_to_vec());
    let initial = dioxus_ssr::render(&dom);
    let root_id = attribute(opening(&initial, "div", "class=\"ix-field "), "id").to_string();
    let event = Rc::new(dioxus::html::PlatformEventData::new(Box::new(
        String::from("Retained actual draft"),
    ))) as Rc<dyn std::any::Any>;
    dom.runtime()
        .handle_event("input", Event::new(event, true), input);
    settle(&mut dom);
    let edited = dioxus_ssr::render(&dom);
    assert!(
        opening(&edited, "input", "id=\"tooltip-private-draft\"")
            .contains("value=\"Retained actual draft\""),
        "event must edit the native draft before modes change: {edited}"
    );
    let mut panel_id = None;
    for mode in [1, 2, 0, 3, 1] {
        dom.in_scope(ScopeId::APP, || capture.mode.get().unwrap().set(mode));
        settle(&mut dom);
        let html = dioxus_ssr::render(&dom);
        assert!(
            opening(&html, "input", "id=\"tooltip-private-draft\"")
                .contains("value=\"Retained actual draft\""),
            "mode{mode}: {html}"
        );
        assert_eq!(
            attribute(opening(&html, "div", "class=\"ix-field "), "id"),
            root_id
        );
        assert_eq!(
            html.matches(&format!("id=\"reactive-help-{mode}\""))
                .count(),
            1
        );
        assert_eq!(
            html.matches("role=\"tooltip\"").count(),
            usize::from(mode != 0)
        );
        assert!(html.contains(if mode == 1 {
            "Check identifier"
        } else {
            "Draft guidance"
        }));
        if mode != 0 {
            let current = attribute(opening(&html, "div", "role=\"tooltip\""), "id");
            assert_ne!(current, format!("reactive-help-{mode}"));
            if let Some(previous) = panel_id.as_ref() {
                assert_eq!(current, previous);
            } else {
                panel_id = Some(current.to_string());
            }
        }
        assert_eq!(
            capture.mounts.get(),
            1,
            "mode{mode}: child scope must remain mounted"
        );
    }
}
