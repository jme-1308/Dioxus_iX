//! Interactive counterparts of the upstream form preview examples.
use dioxus::prelude::*;
use dioxus_ix::*;

fn items() -> Vec<SelectOption> {
    (1..=4)
        .map(|index| SelectOption::new(index.to_string(), format!("Item {index}")))
        .collect()
}

#[component]
pub fn FormExamples() -> Element {
    let mut text = use_signal(|| "Some example text".to_string());
    let mut username = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut quantity = use_signal(|| "10".to_string());
    let mut agreed = use_signal(|| false);
    let mut children = use_signal(|| [true, true, false]);
    let all_checked = children.read().iter().all(|value| *value);
    let some_checked = children.read().iter().any(|value| *value);
    let mut storage = use_signal(|| "256".to_string());
    let mut toggle = use_signal(|| false);
    let mut single = use_signal(|| "1".to_string());
    let mut multiple = use_signal(|| vec!["1".to_string(), "3".to_string()]);
    let mut editable = use_signal(|| "1".to_string());
    let mut select_query = use_signal(String::new);
    let mut select_added = use_signal(String::new);
    let mut top_layer_selection = use_signal(String::new);
    let mut reject_new_options = use_signal(|| false);
    let mut top_layer_feedback = use_signal(String::new);
    let mut rtl_selection = use_signal(String::new);
    let mut geometry_query = use_signal(String::new);
    let mut select_blur_count = use_signal(|| 0_usize);
    let geometry_filter = geometry_query.read().to_ascii_lowercase();
    let geometry_width = if geometry_filter.contains("wide") {
        "480px"
    } else {
        "320px"
    };
    let geometry_max = if geometry_filter.contains("limit") {
        "360px"
    } else {
        "640px"
    };
    let mut slider = use_signal(|| 25.0);
    let mut stepped = use_signal(|| 10.0);
    let mut traced = use_signal(|| 50.0);
    let mut start_date = use_signal(|| "2026-10-01".to_string());
    let mut end_date = use_signal(|| "2026-10-31".to_string());
    let mut search = use_signal(String::new);
    let mut last_search = use_signal(String::new);
    let mut files = use_signal(Vec::<String>::new);
    let mut upload_state = use_signal(|| UploadState::SelectFile);
    let mut upload_folder = use_signal(|| false);
    let mut upload_count = use_signal(|| 0_usize);
    rsx! {
        section { class: "demo-section", id: "inputs",
            h2 { "Input" }
            p { "Label, placeholder, helper text, native types and disabled/read-only examples." }
            div { class: "form-demo-grid",
                Input { label: "Some label", placeholder: "Some placeholder", value: text(),
                    helper_text: "Some helper text", onvaluechange: move |value| text.set(value) }
                Input { label: "Disabled", value: "Some example text", disabled: true }
                Input { label: "Read-only", value: "Some example text", readonly: true }
                Input { label: "Password", input_type: "password", placeholder: "Enter a password", autocomplete: "new-password" }
                NumberInput { label: "Number", value: quantity(), min: 0.0, max: 100.0, step: 5.0,
                    helper_text: "Values from 0 to 100 in steps of 5", onvaluechange: move |value| quantity.set(value) }
                Input { label: "Input with slots", value: text(), onvaluechange: move |value| text.set(value),
                    start: rsx! { Icon { name: "bulb".to_string(), size: 16 } },
                    end: rsx! { span { "unit" } } }
            }
            p { role: "status", "Input value: {text}. Number value: {quantity}." }
        }
        section { class: "demo-section", id: "validation",
            h2 { "Input validation" }
            p { "The four upstream feedback states. Username validation updates as you type." }
            div { class: "form-demo-grid",
                Input { label: "Username", placeholder: "Enter your Username", state: FieldState::Info, info_text: "Info text" }
                Input { label: "Username", placeholder: "Enter your Username", state: FieldState::Warning, warning_text: "Warning text" }
                Input { label: "Username", placeholder: "Enter your Username", state: FieldState::Valid, valid_text: "Valid text" }
                Input { label: "Username", placeholder: "Enter your Username", state: FieldState::Invalid, invalid_text: "Invalid text" }
                Input { label: "Username (3 characters minimum)", value: username(), min_length: 3, required: true,
                    state: if username().is_empty() { FieldState::Default } else if username().chars().count() < 3 { FieldState::Invalid } else { FieldState::Valid },
                    helper_text: "Enter at least 3 characters", invalid_text: "Username needs at least 3 characters", valid_text: "Username meets the length requirement",
                    onvaluechange: move |value| username.set(value) }
            }
        }
        section { class: "demo-section", id: "textarea",
            h2 { "Textarea" }
            div { class: "form-demo-grid",
                Textarea { label: "Some label", placeholder: "Some placeholder", value: notes(), max_length: 200,
                    helper_text: format!("{} / 200 characters", notes().chars().count()), onvaluechange: move |value| notes.set(value) }
                Textarea { label: "Read-only", value: "Some example text", readonly: true }
            }
        }
        section { class: "demo-section", id: "checkboxes",
            h2 { "Checkbox" }
            div { class: "demo-row",
                Checkbox { label: "I agree everything", name: "agreed", checked: agreed(), oncheckedchange: move |value| agreed.set(value) }
                Checkbox { label: "Disabled", disabled: true }
                Checkbox { label: "Disabled checked", disabled: true, checked: true }
            }
            h3 { "Checkbox group / indeterminate" }
            CheckboxGroup { label: "Options", helper_text: "The parent follows the state of its three sub options.",
                Checkbox { label: "Option group", checked: all_checked, indeterminate: some_checked && !all_checked,
                    oncheckedchange: move |value| children.set([value; 3]) }
                div { class: "form-demo-indent",
                    for index in 0..3 {
                        Checkbox { label: if index == 0 { "Sub option one" } else { "Another sub option" }, checked: children()[index],
                            oncheckedchange: move |value| children.write()[index] = value }
                    }
                }
            }
            p { role: "status", "Agreed: {agreed}. Selected sub options: {children.read().iter().filter(|value| **value).count()}." }
        }
        section { class: "demo-section", id: "radios-toggles",
            h2 { "Radio and toggle" }
            div { class: "form-demo-grid",
                RadioGroup { label: "Storage", name: "storage", value: storage(),
                    options: vec![SelectOption::new("256", "256GB SSD storage"), SelectOption::new("512", "512GB SSD storage"), SelectOption::new("1024", "1TB SSD storage").disabled()],
                    onvaluechange: move |value| storage.set(value) }
                div { class: "form-demo-stack",
                    FieldLabel { text: "Toggle" }
                    Toggle { checked: toggle(), oncheckedchange: move |value| toggle.set(value), label: "Enable notifications" }
                    Toggle { checked: true, disabled: true, label: "Disabled toggle" }
                    Toggle { indeterminate: true, label: "Mixed toggle", disabled: true }
                }
            }
            p { role: "status", "Storage: {storage}GB. Toggle: {toggle}." }
        }
        section { class: "demo-section", id: "selects",
            h2 { "Select" }
            div { class: "form-demo-grid",
                Select { label: "Select", options: items(), value: single(), allow_clear: true,
                    onvaluechange: move |value| single.set(value) }
                Select { label: "Multiple", options: items(), values: multiple(), multiple: true, allow_clear: true,
                    onvalueschange: move |values| multiple.set(values) }
                Select { label: "Editable", options: items(), value: editable(), editable: true,
                    helper_text: "Type in this field to filter or add an item", onvaluechange: move |value| editable.set(value),
                    oninputchange: move |text| select_query.set(text), onadd: move |text| select_added.set(text) }
                Select { label: "Disabled", options: items(), value: "1", disabled: true }
                Select { label: "Read-only", options: items(), value: "2", readonly: true }
                Select { label: "Validation", options: items(), state: FieldState::Invalid, invalid_text: "Please select an option" }
            }
            p { role: "status", "Selection: {single}. Multiple: {multiple.read().join(\", \")}. Editable: {editable}." }
            p { role: "status", "Input: {select_query}. Added: {select_added}." }
            h3 { "Top layer Select" }
            p { "The option list escapes this clipped container. Reject additions to keep the typed text without creating or selecting an item." }
            Checkbox { label: "Reject new options", checked: reject_new_options(), oncheckedchange: move |value| reject_new_options.set(value) }
            div { class: "form-select-top-layer-clip",
                Select { id: "top-layer-select", label: "Top layer choice", options: items(), value: top_layer_selection(),
                    enable_top_layer: true, editable: true, allow_clear: true,
                    onvaluechange: move |value| top_layer_selection.set(value),
                    onbeforeadd: move |event: SelectAddEvent| if reject_new_options() { event.prevent_default(); top_layer_feedback.set(format!("Addition rejected: {}", event.value)); },
                    onadd: move |value| top_layer_feedback.set(format!("Added: {value}")),
                    onnativeerror: move |error| top_layer_feedback.set(format!("Using inline options: {error}")) }
            }
            p { role: "status", "Top layer selection: {top_layer_selection}. {top_layer_feedback}" }
            h3 { "Right-to-left Select and live sizing" }
            p { "Open the list, then type wide to expand it to 480px. Type wide limit to cap it at 360px. The list stays aligned with the control’s right edge, within the viewport. Leaving the empty required field shows validation feedback." }
            div { class: "form-select-rtl", dir: "rtl",
                div { class: "form-select-rtl-control",
                    Select { id: "rtl-layer-select", label: "Right-to-left choice", required: true,
                        options: vec![SelectOption::new("wide", "Wide service area"), SelectOption::new("limit", "Wide limit")],
                        value: rtl_selection(), enable_top_layer: true, allow_clear: true,
                        dropdown_width: Some(geometry_width.to_string()), dropdown_max_width: Some(geometry_max.to_string()),
                        onvaluechange: move |value| rtl_selection.set(value),
                        oninputchange: move |value| geometry_query.set(value),
                        onixblur: move |_| select_blur_count += 1 }
                }
            }
            p { id: "select-live-geometry-status", role: "status", "Requested width: {geometry_width}. Maximum: {geometry_max}. Input: {geometry_query}. Input blur events: {select_blur_count}. Selection: {rtl_selection}." }
            SelectMethodsExample {}
        }
        section { class: "demo-section", id: "sliders",
            h2 { "Slider" }
            div { class: "form-demo-grid",
                Slider { label: "0 to 100", value: slider(), onvaluechange: move |value| slider.set(value),
                    markers: vec![SliderMarker { value: 0.0, label: "0".into() }, SliderMarker { value: 100.0, label: "100".into() }] }
                Slider { label: "Steps of 10", min: 0.0, max: 50.0, step: 10.0, value: stepped(), onvaluechange: move |value| stepped.set(value),
                    markers: vec![SliderMarker { value: 0.0, label: "0".into() }, SliderMarker { value: 50.0, label: "50".into() }] }
                Slider { label: "Trace reference", min: -100.0, max: 100.0, value: traced(), trace: true, trace_reference: 0.0,
                    onvaluechange: move |value| traced.set(value), helper_text: "The trace starts at zero" }
                Slider { label: "Disabled", value: 25.0, disabled: true }
            }
        }
        section { class: "demo-section", id: "ranges",
            h2 { "Range field" }
            div { class: "form-demo-range",
                RangeField {
                    Input { input_type: "date", label: "Start date", value: start_date(), max: end_date(), onvaluechange: move |value| start_date.set(value) }
                    Input { input_type: "date", label: "End date", value: end_date(), min: start_date(), onvaluechange: move |value| end_date.set(value) }
                }
            }
        }
        section { class: "demo-section", id: "search-upload",
            h2 { "Expanding search" }
            ExpandingSearch { value: search(), onvaluechange: move |value| search.set(value), onsearch: move |value| last_search.set(value) }
            p { role: "status", "Search text: {search}. Submitted search: {last_search}." }
            h3 { "Upload" }
            p { "Choose files or a folder to inspect names and relative paths. The application controls the upload state." }
            Checkbox { label: "Choose folders", checked: upload_folder(), oncheckedchange: move |value| { upload_folder.set(value); files.set(vec![]); } }
            Upload { key: "upload-{upload_folder}", multiple: true, directory_upload: upload_folder(), multiline: true, state: upload_state(), onentrieschanged: move |selected: Vec<UploadEntry>| {
                upload_count += 1;
                files.set(selected.iter().map(|entry| format!("{} ({} bytes)", entry.relative_path.display(), entry.file.size())).collect());
                upload_state.set(UploadState::SelectFile);
            } }
            ul { for file in files() { li { "{file}" } } }
            p { role: "status", "Selections: {upload_count}" }
            div { class: "demo-row",
                Button { variant: ButtonVariant::Secondary, onclick: move |_| upload_state.set(UploadState::Loading), "Loading state" }
                Button { variant: ButtonVariant::Secondary, onclick: move |_| upload_state.set(UploadState::Success), "Success state" }
                Button { variant: ButtonVariant::Secondary, onclick: move |_| upload_state.set(UploadState::Failed), "Failed state" }
                Button { variant: ButtonVariant::Tertiary, onclick: move |_| { files.set(vec![]); upload_state.set(UploadState::SelectFile); }, "Reset" }
            }
        }
        NativeFormExample {}
    }
}

#[component]
fn SelectMethodsExample() -> Element {
    let controller = use_select();
    let focus = controller.clone();
    let inspect = controller.clone();
    let mut mounted = use_signal(|| true);
    let mut value = use_signal(String::new);
    let mut changes = use_signal(|| 0usize);
    let mut feedback = use_signal(|| "Select methods are ready.".to_string());
    rsx! {
        h3 { "Select input methods and feedback tooltip" }
        p { "Focus the choice from the button or with Tab to read its help. Escape dismisses the tooltip. Inspect reads the current controlled value, touch state, native input and associated form. Unmount the choice to check the controller’s lifetime errors." }
        Checkbox { label: "Mount method choice", checked: mounted(), oncheckedchange: move |next| mounted.set(next) }
        form { id: "select-methods-form", name: "service-area-review", class: "form-demo-stack", onsubmit: move |event| event.prevent_default(),
            if mounted() {
                Select { id: "method-select-choice", controller: controller.clone(), label: "Method choice", name: "service-area",
                    options: items(), value: value(), required: true, allow_clear: true, show_text_as_tooltip: true,
                    helper_text: "Choose a service area. This help also appears on keyboard focus.",
                    invalid_text: "Choose a service area before continuing.",
                    onvaluechange: move |next| { value.set(next); changes += 1; } }
            }
            div { class: "demo-row",
                Button { variant: ButtonVariant::Secondary, onclick: move |_| {
                    let focus = focus.clone();
                    spawn(async move {
                        feedback.set(match focus.focus_input().await {
                            Ok(()) => "Focused the native choice input.".into(),
                            Err(error) => format!("Focus: {error}"),
                        });
                    });
                }, "Focus method choice" }
                Button { variant: ButtonVariant::Secondary, onclick: move |_| {
                    let inspect = inspect.clone();
                    spawn(async move {
                        let result = async {
                            let valid = inspect.has_valid_value()?;
                            let touched = inspect.is_touched()?;
                            let input = inspect.get_native_input_element()?;
                            let form = inspect.get_associated_form_element().await?;
                            Ok::<_, SelectMethodError>(format!("Required value: {valid}. Touched: {touched}. Native input: {}. Form: {}.", input.id(), form.map(|form| form.id).unwrap_or_else(|| "none".into())))
                        }.await;
                        feedback.set(result.unwrap_or_else(|error| format!("Inspect: {error}")));
                    });
                }, "Inspect method choice" }
            }
        }
        p { id: "select-methods-status", role: "status", "{feedback} Selection: {value}. Value events: {changes}." }
    }
}

#[component]
fn NativeFormExample() -> Element {
    let mut username = use_signal(String::new);
    let mut agreed = use_signal(|| false);
    let mut submitted = use_signal(String::new);
    let mut selection = use_signal(|| "1".to_string());
    rsx! {
        section { class: "demo-section", id: "native-form",
            h2 { "Form submission" }
            p { "Required fields use native browser validation. Submission below lists successful named controls." }
            form { class: "form-demo-stack form-demo-narrow", onsubmit: move |event| {
                event.prevent_default();
                submitted.set(event.values().iter().map(|(name,value)| format!("{name}: {value:?}")).collect::<Vec<_>>().join("; "));
            },
                Input { label: "Username", name: "username", required: true, min_length: 3, value: username(),
                    onvaluechange: move |value| username.set(value) }
                Select { label: "Item", name: "item", required: true, options: items(), value: selection(), allow_clear: true,
                    onvaluechange: move |value| selection.set(value) }
                Checkbox { label: "I agree everything", name: "agreed", required: true, checked: agreed(), oncheckedchange: move |value| agreed.set(value) }
                Button { button_type: ButtonType::Submit, "Submit" }
            }
            p { role: "status", class: "form-demo-result", "{submitted}" }
        }
    }
}

pub(crate) const STYLES: &str = r#"
.form-select-top-layer-clip{height:92px;overflow:hidden;max-width:400px;margin-block:16px}
.form-select-rtl{max-width:600px;margin-block:16px}.form-select-rtl-control{width:240px;max-width:100%}
.form-demo-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:24px 32px;max-width:900px;margin-block:24px}.form-demo-stack{display:flex;flex-direction:column;gap:16px}.form-demo-narrow{max-width:400px}.form-demo-indent{display:flex;flex-direction:column;gap:12px;padding-inline-start:24px}.form-demo-range{max-width:680px;margin-block:24px}.form-demo-result{overflow-wrap:anywhere;max-width:75ch}@media(max-width:650px){.form-demo-grid{grid-template-columns:1fr}.form-demo-range .ix-range-field{gap:24px}}
"#;
