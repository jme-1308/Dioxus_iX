//! Native field constraints and scoped methods, separate from source fixtures.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn NativeFieldReview() -> Element {
    let input = use_input();
    let textarea = use_textarea();
    let mut email = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut suppressed = use_signal(String::new);
    let mut events = use_signal(|| 0usize);
    let mut blurs = use_signal(|| 0usize);
    let mut veto = use_signal(|| false);
    let mut mounted = use_signal(|| true);
    let mut inspection = use_signal(|| "Not inspected".to_string());
    let mut custom = use_signal(|| FieldState::Default);
    let inspect = input.clone();
    let focus = input.clone();
    let focus_notes = textarea.clone();
    rsx! { section { class:"demo-section", id:"native-field-review",
        h2 { "Native field validation and methods" }
        p { "Leave a required field empty, then correct it. Native validity changes are counted once per valid/invalid transition. The second form disables automatic feedback." }
        div { class:"demo-row",
            Button { onclick:move |_|{let controller=focus.clone();spawn(async move {let _=controller.focus_input().await;});}, "Focus email" }
            Button { variant:ButtonVariant::Secondary, onclick:move |_|{
                let controller=inspect.clone();spawn(async move {
                    let touched=controller.is_touched();let value=controller.has_valid_value();
                    let validity=controller.get_validity_state().await;let form=controller.get_associated_form_element().await;
                    inspection.set(format!("Touched: {touched:?}; nonempty: {value:?}; validity: {validity:?}; form: {form:?}"));
                });
            }, "Inspect email" }
            Button { variant:ButtonVariant::Secondary, onclick:move |_|{let controller=focus_notes.clone();spawn(async move {let _=controller.focus_input().await;});}, "Focus notes" }
            Button { variant:ButtonVariant::Secondary, onclick:move |_|mounted.toggle(), if mounted() {"Unmount email"} else {"Mount email"} }
        }
        Checkbox { label:"Veto automatic validity feedback", checked:veto(), oncheckedchange:move |value|veto.set(value) }
        form { id:"native-field-form", onsubmit:move|event|event.prevent_default(),
            div { class:"form-demo-grid",
                if mounted() { Input { id:"field-review-email", controller:input, label:"Required email", name:"email", input_type:"email", required:true,
                    value:email(), onvaluechange:move|value|email.set(value), helper_text:"Use an address such as operator@example.com.",
                    invalid_text:"Enter a complete email address.", onixblur:move|_|blurs+=1,
                    onvaliditystatechange:move|event:ValidityStateChangeEvent|{events+=1;if veto(){event.prevent_default();}}
                } }
                Textarea { id:"field-review-notes", controller:textarea, label:"Required service notes", name:"notes", required:true,
                    value:notes(), onvaluechange:move|value|notes.set(value), invalid_text:"Enter the service notes.", max_length:80
                }
            }
            Button { variant:ButtonVariant::Secondary, onclick:move |_|{}, "Leave native fields" }
        }
        form { id:"native-field-unvalidated", novalidate:true,
            Input { id:"field-review-suppressed", label:"Required field without automatic validation", required:true,
                value:suppressed(), onvaluechange:move|value|suppressed.set(value), helper_text:"This form has noValidate enabled.", invalid_text:"This feedback is suppressed by the form." }
        }
        p { id:"field-review-events", role:"status", "Validity transitions: {events}. Email blurs: {blurs}." }
        p { id:"field-review-inspection", role:"status", style:"overflow-wrap:anywhere", "{inspection}" }
        h3 { "Custom control feedback" }
        p { "The wrapper follows validation classes on nested controls, including changes after mounting." }
        div { class:"demo-row",
            Button { variant:ButtonVariant::Secondary, onclick:move |_|custom.set(FieldState::Invalid), "Set nested error" }
            Button { variant:ButtonVariant::Secondary, onclick:move |_|custom.set(FieldState::Valid), "Set nested valid" }
            Button { variant:ButtonVariant::Secondary, onclick:move |_|custom.set(FieldState::Default), "Clear nested feedback" }
        }
        CustomField { label:"Custom service control", helper_text:"No nested validation state.", invalid_text:"The nested control needs attention.", valid_text:"The nested control is valid.",
            div { class:match custom(){FieldState::Invalid=>"ix-invalid",FieldState::Valid=>"ix-valid",_=>""},
                button { r#type:"button", class:"ix-button ix-button-secondary", "Custom action" }
            }
        }
    } }
}
