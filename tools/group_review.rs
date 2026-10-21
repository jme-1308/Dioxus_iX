//! Shared compact group-layout review. All consumer state and prop updates live in Rust.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};

#[derive(Clone, PartialEq)]
struct Case {
    key: String,
    kind: &'static str,
    direction: &'static str,
    state: FieldState,
    label: Option<String>,
    helper: Option<String>,
    valid: Option<String>,
    info: Option<String>,
    warning: Option<String>,
    invalid: Option<String>,
    children: [String; 3],
    scope: usize,
    extra: &'static str,
    required: bool,
    width: Option<usize>,
}
fn cases() -> Vec<Case> {
    let mut result = Vec::new();
    for kind in ["checkbox", "radio"] {
        for direction in ["column", "row"] {
            for (state_name, state) in [
                ("default", FieldState::Default),
                ("valid", FieldState::Valid),
                ("info", FieldState::Info),
                ("warning", FieldState::Warning),
                ("invalid", FieldState::Invalid),
            ] {
                let mut case = base_case(kind, result.len() % 4);
                case.key = format!("{kind}-{direction}-{state_name}");
                case.direction = direction;
                case.state = state;
                result.push(case);
            }
        }
        for extra in [
            "absent-feedback",
            "blank-active",
            "inactive-helper",
            "no-label",
            "required",
            "narrow",
            "override",
            "inactive-blank",
            "whitespace-active",
            "live-props",
        ] {
            let mut case = base_case(kind, result.len() % 4);
            case.key = format!("{kind}-{extra}");
            case.extra = extra;
            match extra {
                "absent-feedback" => {
                    case.helper = None;
                    case.valid = None;
                    case.info = None;
                    case.warning = None;
                    case.invalid = None;
                }
                "blank-active" => {
                    case.state = FieldState::Invalid;
                    case.invalid = Some(" \t ".into());
                    case.helper = Some("Fallback helper".into());
                }
                "inactive-helper" => case.helper = Some("Fallback helper".into()),
                "no-label" => case.label = None,
                "required" => case.required = true,
                "narrow" => {
                    case.direction = "row";
                    case.state = FieldState::Invalid;
                    case.label = Some("Choose a long group label that wraps".into());
                    case.invalid = Some(
                        "A longer invalid selection message that wraps across several lines".into(),
                    );
                    case.children = [
                        "A longer first choice".into(),
                        "A longer second choice".into(),
                        "A longer third choice".into(),
                    ];
                    case.width = Some(200);
                }
                "override" => {
                    case.direction = "row";
                    case.state = FieldState::Warning;
                }
                "inactive-blank" => case.helper = Some(" \t ".into()),
                "whitespace-active" => {
                    case.state = FieldState::Info;
                    case.info = Some("  Informative text  ".into());
                }
                "live-props" => {
                    case.label = Some("Before update".into());
                    case.helper = Some("Before helper".into());
                }
                _ => {}
            }
            result.push(case);
        }
    }
    result
}
fn base_case(kind: &'static str, scope: usize) -> Case {
    Case {
        key: String::new(),
        kind,
        direction: "column",
        state: FieldState::Default,
        label: Some("Choose options".into()),
        helper: Some("Helpful guidance".into()),
        valid: Some("Selection accepted".into()),
        info: Some("Selection information".into()),
        warning: Some("Selection warning".into()),
        invalid: Some("Selection invalid".into()),
        children: ["Alpha".into(), "Beta".into(), "Gamma".into()],
        scope,
        extra: "",
        required: false,
        width: None,
    }
}
fn current_case(mut case: Case, revision: usize) -> Case {
    if case.extra == "live-props" && revision == 1 {
        case.label = Some("After update".into());
        case.direction = "row";
        case.state = FieldState::Invalid;
        case.invalid = Some("After invalid".into());
    }
    case
}
fn state_name(state: FieldState) -> &'static str {
    match state {
        FieldState::Default => "default",
        FieldState::Valid => "valid",
        FieldState::Info => "info",
        FieldState::Warning => "warning",
        FieldState::Invalid => "invalid",
    }
}
const COLORS: [&str; 32] = [
    "text-information",
    "text-danger",
    "text-success",
    "text-warning",
    "text-on-accent",
    "effects-focus",
    "background-accent",
    "background-accent-active",
    "text-disabled",
    "background-accent-hover",
    "background-information",
    "background-information-active",
    "background-information-hover",
    "background-danger",
    "background-danger-active",
    "background-danger-hover",
    "background-warning",
    "background-warning-active",
    "background-warning-hover",
    "border-warning",
    "text-inverse",
    "text-on-information",
    "text-on-danger",
    "text-on-warning",
    "text-primary",
    "background-1",
    "background-selected",
    "background-hover",
    "border-2",
    "border-information",
    "border-danger",
    "text-secondary",
];
fn sentinel_style(scope: usize, revision: usize) -> String {
    let mut style = String::new();
    for (index, token) in COLORS.iter().enumerate() {
        style.push_str(&format!(
            "--si-sys-color-{token}:rgb({},{},{});",
            24 + (13 * index + 7 * scope + 3 * revision) % 160,
            32 + (11 * index + 5 * scope + 7 * revision) % 150,
            40 + (7 * index + 3 * scope + 11 * revision) % 140
        ));
    }
    let size = 20 - 4 * (scope % 2) + scope / 2 + 2 * revision;
    for (token, value) in [
        ("spacing-y-10", 1 + scope + revision),
        ("spacing-y-20", 3 + scope + revision),
        ("spacing-y-30", 5 + scope + revision),
        ("spacing-y-40", 8 + scope + 2 * revision),
        ("spacing-x-20", 4 + scope + revision),
        ("spacing-x-30", 6 + scope + revision),
        ("spacing-x-40", 10 + scope + revision),
        ("spacing-x-60", 14 + scope + 2 * revision),
        ("size-60", size),
        ("size-70", size + 4),
        ("size-30", 8 + scope + revision),
        ("border-width-default", 1 + revision),
        ("border-radius-full", 99 + scope + revision),
    ] {
        style.push_str(&format!("--si-sys-sizing-{token}:{value}px;"));
    }
    style.push_str(&format!("font:400 16px/24px system-ui,sans-serif;color:var(--si-sys-color-text-primary);--si-sys-typography-body:400 {}px/{}px system-ui,sans-serif;--si-sys-typography-body-paragraph:400 {}px/{}px system-ui,sans-serif;--si-ref-typography-letter-spacing-normal:{}px;",14+scope+revision,20+scope+revision,15+scope+revision,23+scope+revision,0.25+0.25*revision as f64));
    style
}
#[component]
pub fn GroupReview() -> Element {
    let mut revision = use_signal(|| 0_usize);
    let mut phase = use_signal(|| "baseline".to_string());
    let mut supplied = use_signal(|| true);
    rsx! {
     style {{STYLES}}
     IxTheme {scheme:ColorScheme::Light,class:"group-review-shell",
      section {id:"group-review","data-phase":phase(),"data-revision":revision().to_string(),"data-supplied":supplied().to_string(),
       header {h2 {"Group layout and inline feedback"} p {"Compare checkbox and radio groups in both directions, with optional labels and inline feedback. Test tokens are authored integration inputs; Brand measurements and broader group behavior remain provisional."}}
       fieldset {class:"group-review-controls",legend {"Token and prop updates"}
        for next in["baseline","revised"] {button {id:format!("group-{next}"),r#type:"button",onclick:move|_|{revision.set(usize::from(next=="revised"));phase.set(next.into());supplied.set(true);},if next=="baseline"{"Baseline"}else{"Revise tokens and live props"}}}
        label {input {id:"group-supplied",r#type:"checkbox",checked:supplied(),onchange:move|event|supplied.set(event.checked())} "Supply test tokens"}
        output {id:"group-status",role:"status","{phase()} — 40 compact cases"}
       }
       div {class:"group-review-matrix",for case in cases(){GroupCase {key:"{case.key}",case:case.clone(),revision:revision(),supplied:supplied()}}}
       section {class:"group-review-fallback",h3 {"Existing Brand fallback"} p {"These two examples omit test tokens and remain separate from the measured cases."}
        IxTheme {scheme:ColorScheme::Light,
         CheckboxGroup {label:"Choose options",helper_text:"Choose any options",horizontal:true,Checkbox {label:"Alpha"} Checkbox {label:"Beta",checked:true}}
         RadioGroup {label:"Choose one",options:vec![SelectOption::new("alpha","Alpha"),SelectOption::new("beta","Beta")],value:"beta",helper_text:"Choose one option"}
        }
       }
       pre {id:"group-result","aria-hidden":"true"}
      }
     }
    }
}
#[component]
fn GroupCase(case: Case, revision: usize, supplied: bool) -> Element {
    let case = current_case(case, revision);
    let mut checked = use_signal(|| [false, true, false]);
    let mut selected = use_signal(|| "beta".to_string());
    let mut callbacks = use_signal(Vec::<Value>::new);
    let scheme = if case.scope >= 2 {
        ColorScheme::Dark
    } else {
        ColorScheme::Light
    };
    let density = if case.scope % 2 == 1 {
        Density::Compact
    } else {
        Density::Normal
    };
    let props = json!({"label":case.label,"direction":case.direction,"state":state_name(case.state),"helperText":case.helper,"validText":case.valid,"infoText":case.info,"warningText":case.warning,"invalidText":case.invalid,"required":case.required,"width":case.width,"extra":case.extra,"children":case.children});
    rsx! {
     article {id:format!("group-{}",case.key),class:"group-review-case","data-group-row":"true","data-key":case.key.clone(),"data-kind":case.kind,"data-scope-index":case.scope.to_string(),"data-revision":revision.to_string(),"data-props":props.to_string(),
      h3 {class:"group-review-case-title","{case.key}"}
      IxTheme {scheme,density,style:if supplied{sentinel_style(case.scope,revision)}else{String::new()},
       div {class:"group-review-sample",
        if case.kind=="checkbox" {
         CheckboxGroup {label:case.label.clone().unwrap_or_default(),horizontal:case.direction=="row",required:case.required,state:case.state,class:if case.extra=="override"{"group-component-override"}else if case.extra=="narrow"{"group-narrow-control"}else{""},helper_text:case.helper.clone(),valid_text:case.valid.clone(),info_text:case.info.clone(),warning_text:case.warning.clone(),invalid_text:case.invalid.clone(),
          for(index,label)in case.children.iter().enumerate(){Checkbox {label:label.clone(),name:format!("{}-{}",case.key,index),value:["alpha","beta","gamma"][index],checked:checked()[index],oncheckedchange:move|value|{checked.write()[index]=value;callbacks.write().push(json!({"kind":"checked","index":index,"value":value}));}}}
         }
        } else {
         RadioGroup {label:case.label.clone().unwrap_or_default(),horizontal:case.direction=="row",required:case.required,state:case.state,class:if case.extra=="override"{"group-component-override"}else if case.extra=="narrow"{"group-narrow-control"}else{""},helper_text:case.helper.clone(),valid_text:case.valid.clone(),info_text:case.info.clone(),warning_text:case.warning.clone(),invalid_text:case.invalid.clone(),name:format!("{}-choice",case.key),options:case.children.iter().enumerate().map(|(index,label)|SelectOption::new(["alpha","beta","gamma"][index],label.clone())).collect::<Vec<_>>(),value:selected(),onvaluechange:move|value:String|{selected.set(value.clone());callbacks.write().push(json!({"kind":"value","value":value}));}}
        }
       }
      }
      input {class:"group-consumer-state",r#type:"hidden",value:json!({"checked":checked(),"value":selected(),"callbacks":callbacks()}).to_string()}
     }
    }
}
const STYLES: &str = include_str!("styles/group_review.css");
