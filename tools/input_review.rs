//! Compact shared Input/Textarea token fixture; public values and revisions stay Rust-owned.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};

#[derive(Clone, PartialEq)]
struct Case {
    key: String,
    kind: &'static str,
    state: FieldState,
    condition: &'static str,
    label: Option<String>,
    placeholder: String,
    value: String,
    helper: Option<String>,
    valid: Option<String>,
    info: Option<String>,
    warning: Option<String>,
    invalid: Option<String>,
    scope: usize,
    extra: &'static str,
    required: bool,
    max_length: Option<u32>,
    rows: Option<u32>,
    cols: Option<u32>,
    height: Option<String>,
    width: Option<String>,
    host_width: Option<u32>,
    resize: &'static str,
    input_type: &'static str,
    start: &'static str,
    end: &'static str,
    align_end: bool,
}
const STATES: [(&str, FieldState); 5] = [
    ("default", FieldState::Default),
    ("valid", FieldState::Valid),
    ("info", FieldState::Info),
    ("warning", FieldState::Warning),
    ("invalid", FieldState::Invalid),
];
fn base_case(kind: &'static str, scope: usize) -> Case {
    Case {
        key: String::new(),
        kind,
        state: FieldState::Default,
        condition: "rest",
        label: Some("Field label".into()),
        placeholder: "Enter a value".into(),
        value: String::new(),
        helper: Some("Helpful guidance".into()),
        valid: Some("Value accepted".into()),
        info: Some("Value information".into()),
        warning: Some("Value warning".into()),
        invalid: Some("Value invalid".into()),
        scope,
        extra: "",
        required: false,
        max_length: None,
        rows: None,
        cols: None,
        height: None,
        width: None,
        host_width: None,
        resize: "both",
        input_type: "text",
        start: "",
        end: "",
        align_end: false,
    }
}
fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for kind in ["input", "textarea"] {
        for (name, state) in STATES {
            for condition in ["rest", "disabled", "readonly", "focus"] {
                let mut case = base_case(kind, cases.len() % 4);
                case.key = format!("{kind}-{name}-{condition}");
                case.state = state;
                case.condition = condition;
                cases.push(case);
            }
        }
    }
    for extra in [
        "start",
        "end",
        "both",
        "password",
        "password-readonly",
        "password-disabled",
        "required-counter",
        "override",
        "live-props",
    ] {
        let mut case = base_case("input", cases.len() % 4);
        case.extra = extra;
        case.key = format!("input-{extra}");
        match extra {
            "start" => case.start = "span",
            "end" => case.end = "span",
            "both" => {
                case.start = "icon24";
                case.end = "span";
            }
            "password" | "password-readonly" | "password-disabled" => {
                case.input_type = "password";
                case.value = "secret".into();
                case.condition = match extra {
                    "password-readonly" => "readonly",
                    "password-disabled" => "disabled",
                    _ => "rest",
                };
            }
            "required-counter" => {
                case.required = true;
                case.max_length = Some(12);
                case.value = "abc".into();
                case.state = FieldState::Invalid;
                case.invalid = Some(" \t ".into());
                case.helper = Some("Fallback helper".into());
            }
            "live-props" => {
                case.state = FieldState::Info;
                case.helper = Some(" \t ".into());
                case.info = Some(String::new());
                case.start = "span";
                case.end = "span";
            }
            _ => {}
        }
        cases.push(case);
    }
    for extra in [
        "rows-cols",
        "explicit-dims",
        "resize-none",
        "counter-nolabel",
        "override",
        "narrow-live",
    ] {
        let mut case = base_case("textarea", cases.len() % 4);
        case.extra = extra;
        case.key = format!("textarea-{extra}");
        match extra {
            "rows-cols" => {
                case.rows = Some(4);
                case.cols = Some(24);
                case.resize = "vertical";
                case.value = "First line\nSecond line".into();
            }
            "explicit-dims" => {
                case.height = Some("96px".into());
                case.width = Some("240px".into());
                case.rows = Some(2);
                case.cols = Some(10);
            }
            "resize-none" => case.resize = "none",
            "counter-nolabel" => {
                case.label = None;
                case.value = "Counted text".into();
                case.max_length = Some(20);
            }
            "narrow-live" => {
                case.host_width = Some(200);
                case.label = Some("A longer field label that wraps".into());
                case.value = "Text inside the field".into();
            }
            _ => {}
        }
        cases.push(case);
    }
    cases
}
fn current_case(mut case: Case, revision: usize) -> Case {
    if revision == 1 {
        if case.extra == "live-props" {
            case.state = FieldState::Invalid;
            case.label = Some("After update".into());
            case.value = "updated".into();
            case.helper = Some("Updated helper".into());
            case.invalid = Some("After invalid".into());
            case.align_end = true;
        }
        if case.extra == "narrow-live" {
            case.state = FieldState::Warning;
            case.warning = Some("A longer warning that wraps across several lines".into());
            case.label = Some("After narrow update".into());
            case.rows = Some(3);
            case.cols = Some(12);
            case.value = "Updated multiline\nvalue".into();
        }
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
fn pointer_cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for kind in ["input", "textarea"] {
        for (name, state) in STATES {
            let mut case = base_case(kind, 0);
            case.key = format!("{kind}-{name}");
            case.state = state;
            case.placeholder = "Pointer field".into();
            case.extra = "pointer";
            cases.push(case);
        }
    }
    cases
}
const COLORS: [&str; 37] = [
    "background-1",
    "background-4",
    "border-2",
    "border-4",
    "border-1",
    "border-information",
    "border-danger",
    "border-warning",
    "text-primary",
    "text-disabled",
    "effects-focus",
    "text-danger",
    "text-secondary",
    "text-accent",
    "text-accent-hover",
    "text-information",
    "text-success",
    "text-warning",
    "background-danger",
    "background-danger-active",
    "background-danger-hover",
    "text-on-danger",
    "border-3",
    "background-accent",
    "background-accent-active",
    "background-accent-hover",
    "text-on-accent",
    "background-accent-secondary",
    "background-accent-secondary-active",
    "background-accent-secondary-hover",
    "border-accent",
    "border-accent-active",
    "border-accent-hover",
    "text-accent-active",
    "background-2",
    "background-selected",
    "background-hover",
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
        ("spacing-x-10", 2 + scope + revision),
        ("spacing-x-20", 4 + scope + revision),
        ("spacing-x-30", 6 + scope + revision),
        ("spacing-x-40", 10 + scope + revision),
        ("spacing-x-60", 14 + scope + 2 * revision),
        ("size-60", size),
        ("size-70", 24),
        ("size-30", 8 + scope + revision),
        ("size-50", 16 + scope + revision),
        ("size-80", 32 - 8 * (scope % 2) + 2 * revision),
        ("size-100", 48 + 2 * scope + 4 * revision),
        ("size-130", 96 + 2 * scope + 4 * revision),
        ("size-40", 12 + scope + revision),
        ("border-width-default", 1 + revision),
        ("border-width-selected", 2 + revision),
        ("border-width-emphasis", 3 + revision),
        ("border-radius-full", 99 + scope + revision),
        ("border-radius-xs", 3 + scope + revision),
        ("focus-ring-offset", 3 + scope + revision),
    ] {
        style.push_str(&format!("--si-sys-sizing-{token}:{value}px;"));
    }
    style.push_str(&format!("font:400 16px/24px system-ui,sans-serif;color:var(--si-sys-color-text-primary);--si-sys-typography-body:400 {}px/{}px system-ui,sans-serif;--si-sys-typography-body-paragraph:400 {}px/{}px system-ui,sans-serif;--si-sys-typography-body-sbold:600 {}px/{}px system-ui,sans-serif;--si-ref-typography-letter-spacing-normal:{}px;",14+scope+revision,20+scope+revision,15+scope+revision,23+scope+revision,14+scope+revision,20+scope+revision,0.25+0.25*revision as f64));
    style
}
fn override_style(revision: usize) -> String {
    format!(
        "#input-review .input-component-override{{--ix-input--min-height:{}px;--ix-input--border-radius:{}px;--ix-input--border-width:2px;--ix-input--outline-width--focus:3px;--ix-input--focus--outline-offset:5px;}}#input-review .ix-input-native.input-component-override{{--ix-input--padding:5px 17px;}}#input-review .ix-textarea-native.input-component-override{{--ix-input-textarea--padding:7px 19px;--ix-input-textarea--height:80px;}}",
        40 + 2 * revision,
        9 + revision
    )
}
#[component]
pub fn InputReview() -> Element {
    let mut revision = use_signal(|| 0_usize);
    let mut phase = use_signal(|| "baseline".to_string());
    let mut supplied = use_signal(|| true);
    let mut pointer_key = use_signal(|| "input-default".to_string());
    let pointer = pointer_cases()
        .into_iter()
        .find(|case| case.key == pointer_key())
        .unwrap_or_else(|| pointer_cases().remove(0));
    rsx! {style {{STYLES}} style {{override_style(revision())}}
     IxTheme {scheme:ColorScheme::Light,class:"input-review-shell",section {id:"input-review","data-phase":phase(),"data-revision":revision().to_string(),"data-supplied":supplied().to_string(),
      onmounted:move |_|{spawn(async move{let query=dioxus::document::eval(POINTER);let _=query.join::<Value>().await;});},
      header {h2 {"Input and textarea anatomy"} p {"Inspect native text controls, semantic feedback, adornments and dimensions under authored test tokens. These inputs exercise integration and do not claim complete Brand or form behavior."}}
      section {class:"input-pointer-panel",h3 {"Pointer states"} p {"Select a field state, then press and hold inside the native text control."}
       label {"Pointer case",select {id:"input-pointer-case",value:pointer_key(),onchange:move|event|pointer_key.set(event.value()),for case in pointer_cases(){option {value:case.key.clone(),"{case.key}"}}}}
       div {id:"input-pointer-target","data-pointer-case":pointer_key(),FieldCase {case:pointer,revision:0,supplied:true,measured:false}}
       output {id:"input-pointer-status",role:"status","Awaiting pointer input"} pre {id:"input-pointer-result","aria-hidden":"true"}
      }
      fieldset {class:"input-review-controls",legend {"Token and prop revisions"}
       for next in["baseline","revised"]{button {id:format!("input-{next}"),r#type:"button",onclick:move|_|{revision.set(usize::from(next=="revised"));phase.set(next.into());supplied.set(true);},if next=="baseline"{"Baseline"}else{"Revise tokens and props"}}}
       label {input {id:"input-supplied",r#type:"checkbox",checked:supplied(),onchange:move|event|supplied.set(event.checked())} "Supply test tokens"}
       button {id:"input-outside",r#type:"button","Keyboard target"} output {id:"input-status",role:"status","{phase()} — 55 cases"}
      }
      div {class:"input-review-matrix",for case in cases(){FieldCase {key:"{case.key}",case:case.clone(),revision:revision(),supplied:supplied(),measured:true}}}
      section {class:"input-review-fallback",h3 {"Existing Brand fallback"} p {"No authored test tokens are supplied in these provisional previews."}
       IxTheme {scheme:ColorScheme::Light,Input {label:"Field label",placeholder:"Enter a value"} Textarea {label:"Field label",placeholder:"Enter multiline text"}}
      }
      pre {id:"input-result","aria-hidden":"true"}
     }}
    }
}
fn adornment(kind: &str, side: &str, revision: usize) -> Option<Element> {
    match kind {
        "span" => {
            let width = if side == "start" {
                18 + 2 * revision
            } else {
                20 + 3 * revision
            };
            Some(
                rsx! {span {class:"input-review-adornment","data-adornment-side":side,style:format!("display:inline-block;width:{width}px;height:18px"),if side=="start"{"S"}else{"E"}}},
            )
        }
        "icon24" => Some(rsx! {Icon {name:"add",size:24,class:"input-review-adornment-icon"}}),
        _ => None,
    }
}
#[component]
fn FieldCase(case: Case, revision: usize, supplied: bool, measured: bool) -> Element {
    let case = current_case(case, revision);
    let mut edited = use_signal(|| None::<(usize, String)>);
    let mut callbacks = use_signal(Vec::<Value>::new);
    let value = edited()
        .filter(|(epoch, _)| *epoch == revision)
        .map(|(_, value)| value)
        .unwrap_or_else(|| case.value.clone());
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
    let class = if case.extra == "override" {
        "input-component-override"
    } else if case.extra == "narrow-live" {
        "input-narrow-control"
    } else {
        ""
    };
    let native_id = format!("input-native-{}", case.key);
    let props = json!({"kind":case.kind,"state":state_name(case.state),"condition":case.condition,"label":case.label,"placeholder":case.placeholder,"value":case.value,"helperText":case.helper,"validText":case.valid,"infoText":case.info,"warningText":case.warning,"invalidText":case.invalid,"extra":case.extra,"required":case.required,"maxLength":case.max_length,"textareaRows":case.rows,"textareaCols":case.cols,"textareaHeight":case.height,"textareaWidth":case.width,"resizeBehavior":case.resize,"type":case.input_type,"start":case.start,"end":case.end,"textAlignment":if case.align_end{"end"}else{"start"},"width":case.host_width});
    rsx! {article {id:if measured{format!("input-{}",case.key)}else{"input-pointer-sample".to_string()},class:"input-review-case","data-input-row":if measured{Some("true")}else{None},"data-key":case.key.clone(),"data-kind":case.kind,"data-condition":case.condition,"data-scope-index":case.scope.to_string(),"data-revision":revision.to_string(),"data-props":props.to_string(),
     if measured{h3 {class:"input-review-case-title","{case.key}"}}
     IxTheme {scheme,density,style:if supplied{sentinel_style(case.scope,revision)}else{String::new()},
      if case.kind=="input"{Input {id:native_id,label:case.label.clone(),value:value.clone(),placeholder:case.placeholder.clone(),disabled:case.condition=="disabled",readonly:case.condition=="readonly",state:case.state,required:case.required,max_length:case.max_length,input_type:case.input_type,class,text_alignment_end:case.align_end,helper_text:case.helper.clone(),valid_text:case.valid.clone(),info_text:case.info.clone(),warning_text:case.warning.clone(),invalid_text:case.invalid.clone(),start:adornment(case.start,"start",revision),end:adornment(case.end,"end",revision),onvaluechange:move|value:String|{edited.set(Some((revision,value.clone())));callbacks.write().push(json!({"kind":"value","value":value}));}}}
      else{Textarea {id:native_id,label:case.label.clone(),value,placeholder:case.placeholder.clone(),disabled:case.condition=="disabled",readonly:case.condition=="readonly",state:case.state,required:case.required,max_length:case.max_length,rows:case.rows,cols:case.cols,width:case.width.clone(),height:case.height.clone(),resize:case.resize,class,helper_text:case.helper.clone(),valid_text:case.valid.clone(),info_text:case.info.clone(),warning_text:case.warning.clone(),invalid_text:case.invalid.clone(),onvaluechange:move|value:String|{edited.set(Some((revision,value.clone())));callbacks.write().push(json!({"kind":"value","value":value}));}}}
     }
     input {class:"input-review-consumer",r#type:"hidden",value:json!({"callbacks":callbacks()}).to_string()}
    }}
}
const STYLES: &str = include_str!("styles/input_review.css");
pub const POINTER: &str = r#"
const root=document.getElementById('input-review');if(!root||root.dataset.pointerObserver==='true')return{installed:false};root.dataset.pointerObserver='true';
const panel=document.getElementById('input-pointer-target'),rows=[],fields=["display","width","height","minWidth","boxSizing","paddingTop","paddingRight","paddingBottom","paddingLeft","borderTopWidth","borderTopColor","borderTopStyle","borderTopLeftRadius","borderTopRightRadius","borderBottomLeftRadius","borderBottomRightRadius","backgroundColor","color","fontSize","lineHeight","fontWeight","fontFamily","fontStyle","letterSpacing","fontFeatureSettings","outlineColor","outlineWidth","outlineStyle","outlineOffset","opacity","pointerEvents","justifyContent","textAlign","marginLeft","marginRight","maxWidth","minHeight","maxHeight","position","flexShrink","gap","top","right","bottom","left","content","fill","stroke","strokeWidth","cursor","visibility","alignItems","marginTop","marginBottom","borderRightWidth","borderBottomWidth","borderLeftWidth","transform","transitionProperty","transitionDuration","transitionTimingFunction","transitionDelay","flexDirection","wordBreak","whiteSpace","overflow","marginBlockStart","marginBlockEnd","marginInlineStart","rowGap","columnGap","flexWrap","marginInlineEnd","minInlineSize","appearance","textOverflow","resize","caretColor","outline","userSelect"];
const metrics=(node,pseudo=null)=>{if(!node)return null;const c=getComputedStyle(node,pseudo),r=node.getBoundingClientRect();return{tag:node.tagName,attrs:Object.fromEntries(Array.from(node.attributes,a=>[a.name,a.value])),computed:Object.fromEntries(fields.map(k=>[k,c[k]])),rect:{x:r.x,y:r.y,width:r.width,height:r.height},pseudo,active:node.matches(':active'),hover:node.matches(':hover'),focus:node.matches(':focus'),focusVisible:node.matches(':focus-visible')};};
const take=(phase,event,control,before)=>{if(!control)return;const host=control.closest('.ix-text-field-native'),wrap=control.closest('.ix-input-wrap');rows.push({phase,case:panel.dataset.pointerCase,event:{type:event.type,trusted:event.isTrusted,pointerType:event.pointerType,clientX:event.clientX,clientY:event.clientY,defaultPrevented:event.defaultPrevented},before,after:{value:control.value,disabled:control.disabled,readonly:control.readOnly},host:metrics(host),control:metrics(control),placeholder:metrics(control,'::placeholder'),inputWrap:metrics(wrap),nativeFacts:{connected:control.isConnected,focused:document.activeElement===control,hover:control.matches(':hover'),active:control.matches(':active'),paddingReady:wrap?.dataset.paddingReady??null}});if(rows.length>64)rows.shift();document.getElementById('input-pointer-result').textContent=JSON.stringify({captures:rows});document.getElementById('input-pointer-status').textContent=rows.length+' actual pointer captures';};
let before=null;const actual=event=>event.composedPath().find(node=>node instanceof HTMLInputElement||node instanceof HTMLTextAreaElement);
panel.addEventListener('pointerdown',event=>{const control=actual(event);if(!control)return;before={value:control.value,disabled:control.disabled,readonly:control.readOnly};const previous=before;requestAnimationFrame(()=>take('pointerdown-frame',event,control,previous));});
panel.addEventListener('pointerup',event=>{const control=actual(event);if(!control)return;const previous=before;requestAnimationFrame(()=>requestAnimationFrame(()=>take('pointerup-second-frame',event,control,previous)));});
window.ixInputPointerReview={snapshot:()=>({case:panel.dataset.pointerCase,captures:rows.slice(),control:metrics(panel.querySelector('.ix-input-wrap input,.ix-input-wrap textarea'))})};return{installed:true};
"#;
