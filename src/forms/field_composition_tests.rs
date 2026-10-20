//! Native association/feedback compatibility and retained composed child state.
use super::*;
use std::cell::Cell;

fn render(component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}
fn element<'a>(html: &'a str, tag: &str, id: &str) -> &'a str {
    html.split('<')
        .find(|part| {
            part.starts_with(&format!("{tag} "))
                && part
                    .split('>')
                    .next()
                    .unwrap()
                    .contains(&format!("id=\"{id}\""))
        })
        .unwrap_or_else(|| panic!("missing {tag}#{id}: {html}"))
        .split('>')
        .next()
        .unwrap()
}
#[test]
fn plain_fields_keep_native_labels_feedback_fallback_and_default_layout() {
    let html = render(|| {
        rsx! {
            FieldWrapper {label:"Equipment",html_for:"plain-equipment",required:true,helper_text:"Use the identifier",invalid_text:"Identifier required",state:FieldState::Invalid,
                input {id:"plain-equipment",name:"equipment",required:true}
            }
            FieldWrapper {label:"Note",html_for:"plain-note",helper_text:"Optional note",invalid_text:"  ",state:FieldState::Invalid,
                textarea {id:"plain-note",name:"note"}
            }
        }
    });
    assert!(html.contains("class=\"ix-field ix-field-invalid\""));
    assert!(html.contains("<label class=\"ix-field-label ix-field-invalid\" for=\"plain-equipment\">Equipment<span class=\"ix-required\" aria-hidden=\"true\"> *</span></label>"));
    assert!(html.contains("role=\"alert\""));
    assert!(html.contains("Identifier required"));
    assert!(!html.contains("Use the identifier"));
    assert!(html.contains("Optional note"));
    assert!(html.contains("role=\"status\""));
    assert!(!html.contains("ix-field-composed-footer"));
    assert!(!html.contains("ix-field-bottom-right"));
    assert!(element(&html, "input", "plain-equipment").contains("name=\"equipment\""));
}
#[test]
fn composed_labels_override_text_and_keep_a_single_native_association() {
    let html = render(|| {
        rsx! {
            FieldLabel {text:"Discarded plain text",html_for:"standalone",required:true,
                strong {"Standalone"} span {" identifier"}
            }
            input {id:"standalone"}
            CustomField {label:"Discarded wrapper text",html_for:"composed",helper_id:"composed-help",helper_text:"Choose an identifier",required:true,
                label_content:rsx!{strong {"Equipment"} span {" identifier"}},
                input {id:"composed",name:"equipment","aria-describedby":"composed-help"}
            }
        }
    });
    assert!(!html.contains("Discarded"));
    assert_eq!(html.matches("<label ").count(), 2);
    assert_eq!(html.matches("for=\"standalone\"").count(), 1);
    assert_eq!(html.matches("for=\"composed\"").count(), 1);
    assert!(html.contains("<strong>Standalone</strong><span> identifier</span>"));
    assert!(html.contains("<strong>Equipment</strong><span> identifier</span>"));
    assert!(element(&html, "input", "composed").contains("aria-describedby=\"composed-help\""));
    assert!(element(&html, "div", "composed-help").contains("class=\"ix-helper"));
    assert_eq!(html.matches("class=\"ix-required\"").count(), 2);
}
#[test]
fn footer_controls_are_outside_labels_and_do_not_require_helper_text() {
    let html = render(|| {
        rsx! {
            form {id:"footer-form",
                FieldWrapper {label:"Equipment",html_for:"footer-input",helper_id:"footer-help",helper_text:"Help",bottom_right:rsx!{
                    button {id:"clear-draft",r#type:"button","Clear"}
                    span {id:"character-count","0/20"}
                },input {id:"footer-input",name:"equipment"}}
                CustomField {bottom_right:rsx!{button {id:"secondary-clear",r#type:"button","Clear optional note"}},
                    textarea {id:"footer-note",name:"note"}
                }
                button {id:"submit-record",r#type:"submit","Save"}
            }
        }
    });
    let label_end = html.find("</label>").unwrap();
    let footer_start = html.find("class=\"ix-field-composed-footer\"").unwrap();
    assert!(footer_start > label_end);
    assert!(html.find("id=\"footer-help\"").unwrap() > footer_start);
    assert!(html.find("id=\"clear-draft\"").unwrap() > footer_start);
    assert_eq!(
        html.matches("class=\"ix-field-composed-footer\"").count(),
        2
    );
    assert_eq!(html.matches("class=\"ix-helper").count(), 1);
    assert!(element(&html, "button", "clear-draft").contains("type=\"button\""));
    assert!(element(&html, "button", "secondary-clear").contains("type=\"button\""));
    assert!(element(&html, "button", "submit-record").contains("type=\"submit\""));
    assert_eq!(html.matches("<label ").count(), 1);
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
fn PrivateDraft(capture: Rc<Capture>) -> Element {
    use_hook(|| capture.mounts.set(capture.mounts.get() + 1));
    let mut draft = use_signal(|| String::from("Initial note"));
    // A dynamic ID is emitted after the runtime element ID is assigned. A static
    // template attribute precedes SetId and cannot identify this event target.
    let id = String::from("private-field-draft");
    rsx! {input {id,name:"note",value:draft(),"aria-describedby":"retained-help",oninput:move|event|draft.set(event.value())}}
}
fn retained_field(capture: Rc<Capture>) -> Element {
    let mode = use_signal(|| 0u8);
    capture.mode.set(Some(mode));
    rsx! {CustomField {label:"Plain label",html_for:"private-field-draft",helper_id:"retained-help",helper_text:"Draft guidance",invalid_text:"Check identifier",state:if mode()==1{FieldState::Invalid}else{FieldState::Default},
        label_content:(mode()!=0).then(||rsx!{strong {"Composed label"}}),
        bottom_right:(mode()!=0).then(||rsx!{button {r#type:"button","Clear"}}),
        PrivateDraft {capture:capture.clone()}
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
macro_rules! draft_converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* })=>{
        impl dioxus::html::HtmlEventConverter for DraftConverter {
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;draft_converter!(@data $group,event)})*
        }
    };
    (@data Form,$event:ident)=>{dioxus::html::FormData::new(DraftForm($event.downcast::<String>().unwrap().clone()))};
    (@data $other:ident,$event:ident)=>{panic!("unexpected field composition test event")};
}
dioxus::html::with_html_event_groups!(draft_converter);
fn draft_id(changes: dioxus::core::Mutations) -> dioxus::core::ElementId {
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
            } if name == "id" && value == "private-field-draft" => return current,
            _ => {}
        }
    }
    panic!("actual private native control must be mounted")
}
fn settle(dom: &mut VirtualDom) {
    for _ in 0..6 {
        dom.render_immediate_to_vec();
        dom.process_events();
    }
}
#[test]
fn private_draft_survives_label_footer_and_validation_changes_with_stable_helper_id() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(DraftConverter));
    let capture = Rc::new(Capture::default());
    let mut dom = VirtualDom::new_with_props(retained_field, capture.clone());
    let input = draft_id(dom.rebuild_to_vec());
    let event = Rc::new(dioxus::html::PlatformEventData::new(Box::new(
        String::from("Retained maintenance draft"),
    ))) as Rc<dyn std::any::Any>;
    dom.runtime()
        .handle_event("input", Event::new(event, true), input);
    settle(&mut dom);
    let edited = dioxus_ssr::render(&dom);
    assert!(
        element(&edited, "input", "private-field-draft")
            .contains("value=\"Retained maintenance draft\""),
        "input event must reach the actual draft before composition changes: {edited}"
    );
    for mode in [1, 2, 0, 1] {
        dom.in_scope(ScopeId::APP, || capture.mode.get().unwrap().set(mode));
        settle(&mut dom);
        let html = dioxus_ssr::render(&dom);
        assert!(
            element(&html, "input", "private-field-draft")
                .contains("value=\"Retained maintenance draft\""),
            "mode {mode}, mounts {}: {html}",
            capture.mounts.get()
        );
        assert!(
            element(&html, "input", "private-field-draft")
                .contains("aria-describedby=\"retained-help\"")
        );
        assert_eq!(html.matches("id=\"retained-help\"").count(), 1);
        let helper = element(&html, "div", "retained-help");
        assert!(helper.contains(if mode == 1 {
            "role=\"alert\""
        } else {
            "role=\"status\""
        }));
        assert!(html.contains(if mode == 1 {
            "Check identifier"
        } else {
            "Draft guidance"
        }));
        assert_eq!(html.contains("Composed label"), mode != 0);
        assert_eq!(html.contains("ix-field-composed-footer"), mode != 0);
        assert_eq!(
            capture.mounts.get(),
            1,
            "composition must not remount a private control"
        );
    }
}
