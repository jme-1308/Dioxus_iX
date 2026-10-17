//! Individually selectable counterparts of the upstream HTML form examples.
//! Each branch preserves that example's labels, initial values and control states.
use dioxus::prelude::*;
use dioxus_ix::*;

pub const SCENARIOS: &[&str] = &[
    "checkbox",
    "checkbox-indeterminate",
    "custom-field",
    "custom-field-validation",
    "expanding-search",
    "form-checkbox",
    "form-checkbox-disabled",
    "form-checkbox-group",
    "form-checkbox-group-indeterminate",
    "form-checkbox-validation",
    "input",
    "input-disabled",
    "input-label",
    "input-pattern",
    "input-readonly",
    "input-types",
    "input-validation",
    "input-with-slots",
    "number-input",
    "number-input-disabled",
    "number-input-label",
    "number-input-readonly",
    "number-input-stepper-button",
    "number-input-validation",
    "number-input-with-slots",
    "radio",
    "radio-button",
    "radio-disabled",
    "radio-group",
    "radio-validation",
    "select",
    "select-editable",
    "select-multiple",
    "select-validation",
    "slider",
    "slider-error",
    "slider-marker",
    "slider-trace",
    "slider-validation",
    "textarea",
    "textarea-disabled",
    "textarea-readonly",
    "textarea-rows-cols",
    "textarea-validation",
    "toggle",
    "toggle-checked",
    "toggle-custom-label",
    "toggle-disabled",
    "toggle-indeterminate",
    "upload",
    "validation-select",
];

/// Render exactly one source scenario. Keyed selection resets its initial fixture state.
#[component]
pub fn FormScenario(name: String) -> Element {
    rsx! {
        style { {STYLES} }
        for selected in [name] {
            FormScenarioBody { key: "{selected}", name: selected }
        }
    }
}

#[component]
fn FormScenarioBody(name: String) -> Element {
    let body = match name.as_str() {
        "input" | "input-disabled" | "input-label" | "input-pattern" | "input-readonly"
        | "input-with-slots" => rsx! { InputScenario { name: name.clone() } },
        "input-types" => rsx! { InputTypesScenario {} },
        "input-validation" | "number-input-validation" | "textarea-validation" => {
            rsx! { TextValidationScenario { name: name.clone() } }
        }
        "number-input"
        | "number-input-disabled"
        | "number-input-label"
        | "number-input-readonly"
        | "number-input-stepper-button"
        | "number-input-with-slots" => rsx! { NumberScenario { name: name.clone() } },
        "textarea" | "textarea-disabled" | "textarea-readonly" | "textarea-rows-cols" => {
            rsx! { TextareaScenario { name: name.clone() } }
        }
        "checkbox"
        | "checkbox-indeterminate"
        | "form-checkbox"
        | "form-checkbox-disabled"
        | "form-checkbox-group"
        | "form-checkbox-group-indeterminate" => rsx! { CheckboxScenario { name: name.clone() } },
        "form-checkbox-validation" => {
            rsx! { div { class: "form-source-stack", for state in [FieldState::Info, FieldState::Warning, FieldState::Invalid, FieldState::Valid] { CheckboxValidationScenario { state } } } }
        }
        "radio" | "radio-disabled" | "radio-button" | "radio-group" | "radio-validation" => {
            rsx! { RadioScenario { name: name.clone() } }
        }
        "toggle"
        | "toggle-checked"
        | "toggle-custom-label"
        | "toggle-disabled"
        | "toggle-indeterminate" => rsx! { ToggleScenario { name: name.clone() } },
        "select" | "select-editable" | "select-multiple" | "select-validation"
        | "validation-select" => rsx! { SelectScenario { name: name.clone() } },
        "slider" | "slider-marker" | "slider-trace" | "slider-error" | "slider-validation" => {
            rsx! { SliderScenario { name: name.clone() } }
        }
        "custom-field" => rsx! { FileFieldScenario {} },
        "custom-field-validation" => rsx! { CustomValidationScenario {} },
        "expanding-search" => rsx! { SearchScenario {} },
        "upload" => rsx! { UploadScenario {} },
        "range-field" => rsx! { RangeScenario {} },
        _ => rsx! { p { role: "alert", "Unknown form scenario: {name}" } },
    };
    rsx! { div { class: "form-source-case", "data-form-scenario": name, {body} } }
}

fn validation_states() -> [FieldState; 4] {
    [
        FieldState::Info,
        FieldState::Warning,
        FieldState::Valid,
        FieldState::Invalid,
    ]
}

#[component]
fn InputScenario(name: String) -> Element {
    let initial = if name == "input-readonly" {
        "Some text"
    } else if name == "input-label" || name == "input-disabled" {
        "Some example text"
    } else {
        ""
    };
    let mut value = use_signal(|| initial.to_string());
    let label = match name.as_str() {
        "input-label" => Some("Some label".to_string()),
        "input-readonly" => Some("Readonly".to_string()),
        _ => None,
    };
    let placeholder = match name.as_str() {
        "input" => Some("Enter your Username".to_string()),
        "input-label" | "input-disabled" => Some("Some placeholder".to_string()),
        "input-readonly" => Some("Placeholder text...".to_string()),
        _ => None,
    };
    rsx! {
        Input { label, placeholder, value: value(), disabled: name == "input-disabled", readonly: name == "input-readonly",
            helper_text: if name == "input-label" { Some("Some helper text".to_string()) } else if name == "input-pattern" { Some("Only characters a-d are allowed".to_string()) } else { None },
            max_length: if name == "input-pattern" { Some(4) } else { None },
            allowed_characters_pattern: if name == "input-pattern" { Some("[a-d]".to_string()) } else { None },
            aria_label: if name == "input-with-slots" { Some("Input with unit".to_string()) } else { None },
            start: if name == "input-with-slots" { Some(rsx! { Icon { name: "bulb", size: 16 } }) } else { None },
            end: if name == "input-with-slots" { Some(rsx! { Typography { "unit" } }) } else { None },
            onvaluechange: move |next| value.set(next),
        }
    }
}

#[component]
fn InputTypesScenario() -> Element {
    let mut values = use_signal(|| {
        vec![
            "MaxMuster1".to_string(),
            "example@example.com".to_string(),
            "1234".to_string(),
            "+49 123 456789".to_string(),
            "https://ix.siemens.io".to_string(),
        ]
    });
    let fields = [
        ("text", "Username", None),
        ("email", "Email", None),
        ("password", "Password", Some("Enter your password")),
        ("tel", "Telephone", Some("Enter your phone number")),
        ("url", "URL", Some("Enter your URL")),
    ];
    rsx! { div { class: "form-source-types", for (index, (kind, label, placeholder)) in fields.into_iter().enumerate() {
        Input { input_type: kind, label, placeholder: placeholder.map(str::to_string), value: values()[index].clone(), onvaluechange: move |next| values.write()[index] = next }
    } } }
}

#[component]
fn NumberScenario(name: String) -> Element {
    let mut value = use_signal(|| {
        if name == "number-input-stepper-button" {
            "0".to_string()
        } else {
            String::new()
        }
    });
    rsx! {
        NumberInput { value: value(), disabled: name == "number-input-disabled", readonly: name == "number-input-readonly",
            show_stepper_buttons: name == "number-input-stepper-button",
            label: if name == "number-input-label" { Some("Some label".to_string()) } else { None },
            helper_text: if name == "number-input-label" { Some("Some helper text".to_string()) } else { None },
            start: if name == "number-input-with-slots" { Some(rsx! { Icon { name: "bulb", size: 16 } }) } else { None },
            end: if name == "number-input-with-slots" { Some(rsx! { Typography { "unit" } }) } else { None },
            onvaluechange: move |next| value.set(next),
        }
    }
}

#[component]
fn TextareaScenario(name: String) -> Element {
    let mut value = use_signal(|| {
        if name == "textarea-disabled" || name == "textarea-readonly" {
            "Some text".to_string()
        } else {
            String::new()
        }
    });
    let rows_cols = name == "textarea-rows-cols";
    rsx! {
        Textarea { value: value(), disabled: name == "textarea-disabled", readonly: name == "textarea-readonly",
            label: rows_cols.then(|| "Comment".to_string()), name: rows_cols.then(|| "comment".to_string()),
            helper_text: rows_cols.then(|| "Let us know if you have any special requests or comments. We will do our best to accommodate you.".to_string()),
            max_length: rows_cols.then_some(100), rows: if rows_cols { 3 } else { 4 }, cols: rows_cols.then_some(50),
            height: (!rows_cols).then(|| "100px".to_string()), width: (!rows_cols).then(|| "300px".to_string()), resize: "both",
            onvaluechange: move |next| value.set(next),
        }
    }
}

#[component]
fn TextValidationScenario(name: String) -> Element {
    let number = name == "number-input-validation";
    let textarea = name == "textarea-validation";
    let mut values = use_signal(|| {
        vec![
            if number {
                "5".to_string()
            } else {
                String::new()
            };
            4
        ]
    });
    rsx! { div { class: "form-source-stack", for (index, state) in validation_states().into_iter().enumerate() {
        if number {
            NumberInput { value: values()[index].clone(), state, info_text: "Info text", warning_text: "Warning text", valid_text: "Valid text", invalid_text: "Invalid text", onvaluechange: move |next| values.write()[index] = next }
        } else if textarea {
            Textarea { value: values()[index].clone(), label: "Comment", name: "comment", max_length: 100, height: "50px", width: "200px", resize: "both", state,
                info_text: "Info text", warning_text: "Warning text", valid_text: "Valid text", invalid_text: "Invalid text", onvaluechange: move |next| values.write()[index] = next }
        } else {
            Input { value: values()[index].clone(), label: "Username", placeholder: "Enter your Username", state, info_text: "Info text", warning_text: "Warning text", valid_text: "Valid text", invalid_text: "Invalid text", onvaluechange: move |next| values.write()[index] = next }
        }
    } } }
}

#[component]
fn CheckboxScenario(name: String) -> Element {
    let mut checked = use_signal(|| false);
    let mut mixed = use_signal(|| name == "checkbox-indeterminate");
    let mut children = use_signal(|| [true, true, false]);
    let mut most = use_signal(|| false);
    match name.as_str() {
        "checkbox" => {
            rsx! { div { class: "form-source-stack", Checkbox { label: "Simple checkbox", checked: checked(), oncheckedchange: move |next| checked.set(next) } Checkbox { label: "Disabled checkbox", disabled: true } } }
        }
        "checkbox-indeterminate" => {
            rsx! { Checkbox { label: "Simple checkbox", checked: checked(), indeterminate: mixed(), oncheckedchange: move |next| { checked.set(next); mixed.set(false); } } }
        }
        "form-checkbox" | "form-checkbox-disabled" => {
            rsx! { Checkbox { label: "I agree everything", name: "agreed", checked: checked(), disabled: name == "form-checkbox-disabled", oncheckedchange: move |next| checked.set(next) } }
        }
        "form-checkbox-group" => rsx! { CheckboxGroup { label: "Terms of something",
            Checkbox { label: "I agree everything", name: "agreed", checked: checked(), oncheckedchange: move |next| checked.set(next) }
            Checkbox { label: "I agree with most of it", name: "most", checked: most(), oncheckedchange: move |next| most.set(next) }
        } },
        _ => {
            let all = children.read().iter().all(|value| *value);
            let any = children.read().iter().any(|value| *value);
            rsx! { CheckboxGroup {
                Checkbox { label: "Option group", checked: all, indeterminate: any && !all, oncheckedchange: move |next| children.set([next; 3]) }
                div { class: "form-source-child-checkboxes", for (index, label) in ["Sub option one", "Another sub option", "Another sub option"].into_iter().enumerate() {
                    Checkbox { label, checked: children()[index], oncheckedchange: move |next| children.write()[index] = next }
                } }
            } }
        }
    }
}

#[component]
fn CheckboxValidationScenario(state: FieldState) -> Element {
    let mut first = use_signal(|| false);
    let mut second = use_signal(|| false);
    rsx! { CheckboxGroup { label: "Terms of something", state,
        info_text: "Info text", warning_text: "Warning text", invalid_text: "Invalid text", valid_text: "Valid text",
        Checkbox { label: "I agree everything", name: "agreed", checked: first(), state: if state == FieldState::Info { state } else { FieldState::Default }, oncheckedchange: move |next| first.set(next) }
        if state != FieldState::Info { Checkbox { label: "I disagree everything", name: "agreed", checked: second(), state, oncheckedchange: move |next| second.set(next) } }
    } }
}

fn storage_options() -> Vec<SelectOption> {
    vec![
        SelectOption::new("256", "256GB SSD storage"),
        SelectOption::new("512", "512GB SSD storage"),
        SelectOption::new("1024", "1TB SSD storage"),
        SelectOption::new("2048", "2TB SSD storage").disabled(),
    ]
}

#[component]
fn RadioScenario(name: String) -> Element {
    let mut value = use_signal(|| {
        if name == "radio-button" {
            "checked".to_string()
        } else {
            String::new()
        }
    });
    let mut validation = use_signal(|| ["256".to_string(), "256".to_string()]);
    match name.as_str() {
        "radio" => {
            rsx! { Radio { label: "256GB SSD storage", value: "1", name: "storage-256gb", checked: value() == "1", onvaluechange: move |next| value.set(next) } }
        }
        "radio-disabled" => {
            rsx! { Radio { label: "2TB SSD storage", value: "2048", name: "storage-2tb", disabled: true } }
        }
        "radio-button" => rsx! { div { class: "form-source-radio-button",
            Radio { label: "Checked", name: "group_1", value: "checked", checked: value() == "checked", onvaluechange: move |next| value.set(next) }
            Radio { label: "Normal", name: "group_1", value: "normal", checked: value() == "normal", onvaluechange: move |next| value.set(next) }
            Radio { label: "Disabled", name: "group_1", value: "disabled", disabled: true }
        } },
        "radio-group" => {
            rsx! { RadioGroup { label: "Storage options", value: value(), options: storage_options(), onvaluechange: move |next| value.set(next) } }
        }
        _ => {
            rsx! { div { class: "form-source-stack", for (index, states) in [[FieldState::Info, FieldState::Warning], [FieldState::Valid, FieldState::Invalid]].into_iter().enumerate() {
                fieldset { class: "ix-field ix-check-group", legend { class: "ix-field-label", "Storage options" }
                    div { class: "ix-choices",
                        Radio { label: "256GB SSD storage", value: "256", name: format!("storage-{index}"), checked: validation()[index] == "256", state: states[0], onvaluechange: move |next| validation.write()[index] = next }
                        Radio { label: "512GB SSD storage", value: "512", name: format!("storage-{index}"), checked: validation()[index] == "512", state: states[1], onvaluechange: move |next| validation.write()[index] = next }
                    }
                }
            } } }
        }
    }
}

#[component]
fn ToggleScenario(name: String) -> Element {
    let mut checked = use_signal(|| name == "toggle-checked");
    let mut mixed = use_signal(|| name == "toggle-indeterminate");
    let custom = name == "toggle-custom-label";
    rsx! { Toggle { checked: checked(), indeterminate: mixed(), disabled: name == "toggle-disabled",
        label: if custom { "Connection status" } else { "Toggle" }, text_off: if custom { "Offline" } else { "Off" }, text_on: if custom { "Online" } else { "On" },
        oncheckedchange: move |next| { checked.set(next); mixed.set(false); }
    } }
}

fn numbered_options() -> Vec<SelectOption> {
    (1..=4)
        .map(|number| SelectOption::new(number.to_string(), format!("Item {number}")))
        .collect()
}

#[component]
fn SelectScenario(name: String) -> Element {
    let mut value = use_signal(|| {
        if name == "validation-select" {
            String::new()
        } else {
            "1".to_string()
        }
    });
    let mut multiple = use_signal(|| vec!["1".to_string(), "3".to_string()]);
    let mut validation = use_signal(|| {
        [
            String::new(),
            "blazor".to_string(),
            "angular".to_string(),
            "svelte".to_string(),
        ]
    });
    let mut payload = use_signal(String::new);
    if name == "validation-select" {
        return rsx! { form { class: "form-source-stack", onsubmit: move |event| {
            event.prevent_default(); payload.set(event.values().into_iter().map(|(key, value)| match value { dioxus::html::FormValue::Text(value) => format!("{key}: {value}"), _ => key }).collect::<Vec<_>>().join("; "));
        },
            Select { label: "Select Car", name: "car", required: true, allow_clear: true, helper_text: "Select your car brand", invalid_text: "Value is required", value: value(),
                options: vec![SelectOption::new("audi", "Audi"), SelectOption::new("vw", "VW")], onvaluechange: move |next| value.set(next) }
            div { Button { button_type: ButtonType::Submit, "Submit" } }
            if !payload().is_empty() { output { role: "status", "{payload}" } }
        } };
    }
    if name == "select-validation" {
        return rsx! { div { class: "form-source-stack", for (index, state) in validation_states().into_iter().enumerate() {
            Select { label: "Framework options", name: "framework-option", state, value: validation()[index].clone(),
                options: match index { 1 => vec![SelectOption::new("blazor", "Blazor"), SelectOption::new("nextjs", "NextJS")], 3 => vec![SelectOption::new("svelte", "Svelte"), SelectOption::new("solid", "Solid")], _ => vec![SelectOption::new("angular", "Angular"), SelectOption::new("react", "React"), SelectOption::new("vue", "Vue")] },
                info_text: "We will show examples based on your selection.", warning_text: "Selected Framework is partly supported.", valid_text: "Selected Framework is fully supported.", invalid_text: "Selected Framework is not supported.",
                onvaluechange: move |next| validation.write()[index] = next,
            }
        } } };
    }
    rsx! { Select { options: numbered_options(), value: value(), values: multiple(), multiple: name == "select-multiple", editable: name == "select-editable",
        onvaluechange: move |next| value.set(next), onvalueschange: move |next| multiple.set(next)
    } }
}

#[component]
fn SliderScenario(name: String) -> Element {
    let mut values = use_signal(|| {
        if name == "slider-error" {
            [0.0, 20.0, 0.0, 0.0]
        } else if name == "slider-trace" {
            [25.0, 20.0, 0.0, 0.0]
        } else if name == "slider-validation" {
            [10.0, 90.0, 10.0, 100.0]
        } else {
            [25.0, 10.0, 0.0, 0.0]
        }
    });
    if name == "slider-validation" {
        return rsx! { div { class: "form-source-stack", for (index, state) in validation_states().into_iter().enumerate() {
            Slider { label: ["Info", "Warning", "Valid", "Invalid"][index], value: values()[index], trace: true, show_value: false, label_start: "0", label_end: "100", state,
                info_text: "Info text", warning_text: "Warning text", valid_text: "Valid text", invalid_text: "Invalid text", onvaluechange: move |next| values.write()[index] = next }
        } } };
    }
    let has_markers = name != "slider";
    let trace = name == "slider-trace" || name == "slider-error";
    let error = name == "slider-error";
    rsx! { div { class: if name == "slider" { "form-source-sliders form-source-slider-padding" } else { "form-source-sliders" },
        Slider { value: values()[0], show_value: false, label_start: "0", label_end: "100", trace, trace_reference: 50.0,
            state: if error { FieldState::Invalid } else { FieldState::Default },
            markers: if has_markers { [0.0,25.0,50.0,75.0,100.0].into_iter().map(|value|SliderMarker{value,label:String::new()}).collect() } else { vec![] },
            onvaluechange: move |next| values.write()[0] = next }
        Slider { value: values()[1], min: 0.0, max: 50.0, step: 10.0, show_value: false, label_start: "0", label_end: "50", trace,
            trace_reference: if error { 50.0 } else { 10.0 }, state: if error { FieldState::Invalid } else { FieldState::Default },
            invalid_text: error.then(|| "Error message".to_string()),
            markers: if has_markers { [0.0,10.0,20.0,30.0,40.0,50.0].into_iter().map(|value|SliderMarker{value,label:String::new()}).collect() } else { vec![] },
            onvaluechange: move |next| values.write()[1] = next }
    } }
}

#[component]
fn FileFieldScenario() -> Element {
    let mut filename = use_signal(|| "No file chosen".to_string());
    rsx! { CustomField { helper_text: "Choose file",
        div { class: "form-source-file-line",
            Input { value: filename(), readonly: true, aria_label: "Chosen file" }
            label { class: "form-source-file-picker",
                input { r#type: "file", "aria-label": "Choose file", onchange: move |event| filename.set(event.files().first().map(|file|file.name()).unwrap_or_else(||"No file chosen".into())) }
                Icon { name: "open-file", size: 24 }
            }
        }
    } }
}

#[component]
fn CustomValidationScenario() -> Element {
    let mut values = use_signal(|| {
        [
            "Jane".to_string(),
            "Doe".to_string(),
            "info@".to_string(),
            String::new(),
        ]
    });
    let fields = [
        ("first-name", "First name", "text", FieldState::Valid),
        ("last-name", "Last name", "text", FieldState::Info),
        ("email", "Email", "email", FieldState::Warning),
        ("address", "Address", "text", FieldState::Invalid),
    ];
    rsx! { div { class: "form-source-stack", for (index, (id,label,kind,state)) in fields.into_iter().enumerate() {
        CustomField { label, html_for: id, state,
            helper_text: (index == 0).then(|| "Add your first name here".to_string()), valid_text: "Valid first name", info_text: "Family name", warning_text: "Not a valid email address", invalid_text: "Required",
            input { class: "ix-input", id, r#type: kind, value: values()[index].clone(), oninput: move |event| values.write()[index] = event.value() }
        }
    } } }
}

#[component]
fn SearchScenario() -> Element {
    let mut value = use_signal(String::new);
    rsx! { ExpandingSearch { placeholder: "Search...", value: value(), onvaluechange: move |next| value.set(next) } }
}

#[component]
fn UploadScenario() -> Element {
    let mut names = use_signal(Vec::<String>::new);
    rsx! {
        Upload { onfileschanged: move |files: Vec<dioxus::html::FileData>| names.set(files.into_iter().map(|file|file.name()).collect()) }
        if !names().is_empty() { output { class: "form-source-upload-result", role: "status", "{names.read().join(\", \")}" } }
    }
}

/// The source body is retained for review while automatic first-to-second picker
/// handoff is pending; this case is intentionally absent from SCENARIOS.
#[component]
fn RangeScenario() -> Element {
    let today = IxDate::new(2026, 10, 16).unwrap();
    let mut values = use_signal(|| vec![String::new(); 6]);
    rsx! { div { class: "form-source-ranges",
        div { role: "group", "aria-label": "Date range", RangeField {
            DateInput { today, label: "Start date", value: values()[0].clone(), onvaluechange: move |next| values.write()[0] = next }
            DateInput { today, label: "End date", value: values()[1].clone(), onvaluechange: move |next| values.write()[1] = next }
        } }
        div { role: "group", "aria-label": "Time range", RangeField {
            TimeInput { label: "Start time", value: values()[2].clone(), onvaluechange: move |next| values.write()[2] = next }
            TimeInput { label: "End time", value: values()[3].clone(), onvaluechange: move |next| values.write()[3] = next }
        } }
        div { role: "group", "aria-label": "Datetime range", RangeField {
            DateTimeInput { today, label: "Start date and time", value: values()[4].clone(), onvaluechange: move |next| values.write()[4] = next }
            DateTimeInput { today, label: "End date and time", value: values()[5].clone(), onvaluechange: move |next| values.write()[5] = next }
        } }
    } }
}

pub const STYLES: &str = r#"
.form-source-case{max-width:100%;min-width:0}.form-source-stack{display:flex;flex-direction:column;gap:16px}.form-source-types{display:flex;flex-direction:column;gap:16px;width:11rem;max-width:100%}.form-source-child-checkboxes{display:flex;flex-direction:column;gap:12px;padding-left:1rem}.form-source-radio-button{display:flex;flex-direction:column;gap:.5rem}.form-source-sliders{display:flex;flex-direction:column;gap:16px}.form-source-slider-padding{padding:.5rem}.form-source-file-line{display:flex;align-items:center;gap:4px}.form-source-file-line>.ix-field{flex:1;min-width:0}.form-source-file-picker{display:flex;align-items:center;justify-content:center;position:relative;width:32px;height:32px;color:var(--ix-color-primary);flex:none;cursor:pointer;border-radius:var(--ix-radius)}.form-source-file-picker:hover{background:var(--ix-color-surface-2)}.form-source-file-picker input{position:absolute;inset:0;width:100%;height:100%;opacity:0;cursor:pointer}.form-source-file-picker:has(:focus-visible){outline:2px solid var(--ix-color-focus);outline-offset:2px}.form-source-upload-result{display:block;font-size:.875rem;margin-top:16px;overflow-wrap:anywhere}.form-source-ranges{display:flex;flex-direction:column;gap:16px;width:32rem;max-width:100%}
"#;
