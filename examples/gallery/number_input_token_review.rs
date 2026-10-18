//! Shared NumberInput visual fixture; the existing numeric behavior review remains separate.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};

#[derive(Clone, PartialEq)]
struct Case {
    key: String,
    state: FieldState,
    condition: &'static str,
    steppers: bool,
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
    min: Option<f64>,
    max: Option<f64>,
    step: f64,
    start: &'static str,
    end: &'static str,
    align_end: bool,
    host_width: Option<u32>,
}
const STATES: [(&str, FieldState); 5] = [
    ("default", FieldState::Default),
    ("valid", FieldState::Valid),
    ("info", FieldState::Info),
    ("warning", FieldState::Warning),
    ("invalid", FieldState::Invalid),
];
fn state_name(state: FieldState) -> &'static str {
    match state {
        FieldState::Default => "default",
        FieldState::Valid => "valid",
        FieldState::Info => "info",
        FieldState::Warning => "warning",
        FieldState::Invalid => "invalid",
    }
}
fn base_case(scope: usize) -> Case {
    Case {
        key: String::new(),
        state: FieldState::Default,
        condition: "rest",
        steppers: false,
        label: Some("Number label".into()),
        placeholder: "Enter a number".into(),
        value: "0".into(),
        helper: Some("Helpful guidance".into()),
        valid: Some("Value accepted".into()),
        info: Some("Value information".into()),
        warning: Some("Value warning".into()),
        invalid: Some("Value invalid".into()),
        scope,
        extra: "",
        required: false,
        min: None,
        max: None,
        step: 1.0,
        start: "",
        end: "",
        align_end: true,
        host_width: None,
    }
}
fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for (name, state) in STATES {
        for condition in ["rest", "disabled", "readonly", "focus"] {
            for steppers in [false, true] {
                let mut case = base_case(cases.len() % 4);
                case.key = format!(
                    "number-{name}-{condition}-{}",
                    if steppers { "stepper" } else { "plain" }
                );
                case.state = state;
                case.condition = condition;
                case.steppers = steppers;
                cases.push(case);
            }
        }
    }
    for extra in [
        "defaults",
        "empty",
        "start",
        "end",
        "both",
        "required-blank",
        "override",
        "align-start",
        "bounds",
        "scientific",
        "live-stepper",
        "narrow",
    ] {
        let mut case = base_case(cases.len() % 4);
        case.extra = extra;
        case.key = format!("number-{extra}");
        match extra {
            "empty" => case.value.clear(),
            "start" => {
                case.start = "span";
                case.steppers = true;
            }
            "end" => {
                case.end = "span";
                case.steppers = true;
            }
            "both" => {
                case.start = "icon24";
                case.end = "span";
                case.steppers = true;
            }
            "required-blank" => {
                case.required = true;
                case.value.clear();
                case.state = FieldState::Invalid;
                case.invalid = Some(" \t ".into());
                case.helper = Some("Fallback helper".into());
                case.steppers = true;
            }
            "override" => case.steppers = true,
            "align-start" => {
                case.align_end = false;
                case.value = "12.5".into();
                case.steppers = true;
            }
            "bounds" => {
                case.value = "4".into();
                case.min = Some(0.0);
                case.max = Some(4.0);
                case.step = 0.5;
                case.steppers = true;
            }
            "scientific" => case.value = "1e+21".into(),
            "live-stepper" => {
                case.start = "span";
                case.end = "span";
            }
            "narrow" => {
                case.state = FieldState::Warning;
                case.steppers = true;
                case.label = Some("A longer number label that wraps".into());
                case.warning = Some("A longer warning that wraps across several lines".into());
                case.host_width = Some(200);
            }
            _ => {}
        }
        cases.push(case);
    }
    cases
}
fn current_case(mut case: Case, revision: usize) -> Case {
    if case.extra == "live-stepper" && revision == 1 {
        case.steppers = true;
        case.state = FieldState::Info;
        case.value = "2.5".into();
        case.label = Some("After update".into());
        case.info = Some("Updated information".into());
    }
    case
}
fn pointer_case(name: &str) -> Case {
    let mut case = base_case(0);
    case.key = format!("number-pointer-{name}");
    case.steppers = true;
    case.extra = "pointer";
    case.state = STATES
        .into_iter()
        .find(|(key, _)| *key == name)
        .map(|(_, state)| state)
        .unwrap_or_default();
    case
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
        "#number-input-review .number-input-component-override{{--ix-input--min-height:{}px;--ix-input--border-radius:{}px;--ix-input--border-width:2px;--ix-input--outline-width--focus:3px;--ix-input--focus--outline-offset:5px;--ix-input--padding:5px 17px;--ix-input-show-stepper-buttons--min-width:{}px;}}",
        40 + 2 * revision,
        9 + revision,
        120 + 4 * revision
    )
}
#[derive(Clone, Copy, PartialEq)]
struct Consumer {
    value: Signal<String>,
    disabled: Signal<bool>,
    readonly: Signal<bool>,
    events: Signal<Vec<Value>>,
    commands: Signal<usize>,
}
impl Consumer {
    fn command(mut self) {
        self.commands += 1;
    }
    fn reset(mut self) {
        self.value.set("0".into());
        self.disabled.set(false);
        self.readonly.set(false);
        self.events.set(Vec::new());
        self.command();
    }
    fn record(mut self, kind: &str, value: Value) {
        let before = (self.value)();
        let mut events = self.events.write();
        let sequence = events.len() + 1;
        events.push(json!({"sequence":sequence,"kind":kind,"value":value,"rawAtCallback":before}));
    }
    fn snapshot(self) -> String {
        let events = (self.events)();
        json!({"value":(self.value)().parse::<f64>().ok(),"valueText":(self.value)(),"disabled":(self.disabled)(),"readonly":(self.readonly)(),"events":events,"requests":events.iter().filter(|row|row["kind"]=="numeric").enumerate().map(|(index,row)|json!({"sequence":index+1,"value":row["value"],"eventSequence":row["sequence"]})).collect::<Vec<_>>()}).to_string()
    }
}
#[component]
pub fn NumberInputTokenReview() -> Element {
    let mut revision = use_signal(|| 0_usize);
    let mut phase = use_signal(|| "baseline".to_string());
    let mut supplied = use_signal(|| true);
    let mut pointer_key = use_signal(|| "default".to_string());
    let pointer = Consumer {
        value: use_signal(|| "0".to_string()),
        disabled: use_signal(|| false),
        readonly: use_signal(|| false),
        events: use_signal(Vec::<Value>::new),
        commands: use_signal(|| 0_usize),
    };
    let mut consumer = Consumer {
        value: use_signal(|| "0".to_string()),
        disabled: use_signal(|| false),
        readonly: use_signal(|| false),
        events: use_signal(Vec::<Value>::new),
        commands: use_signal(|| 0_usize),
    };
    let mut consumer_case = base_case(0);
    consumer_case.key = "number-consumer".into();
    consumer_case.extra = "consumer";
    consumer_case.label = Some("Quantity".into());
    consumer_case.steppers = true;
    consumer_case.step = 0.5;
    consumer_case.min = Some(0.0);
    consumer_case.max = Some(4.0);
    rsx! {style {{STYLES}} style {{override_style(revision())}}
     IxTheme {scheme:ColorScheme::Light,class:"number-input-review-shell",section {id:"number-input-review","data-phase":phase(),"data-revision":revision().to_string(),"data-supplied":supplied().to_string(),"data-consumer-command":(consumer.commands)().to_string(),
      onmounted:move |_|{spawn(async move{let query=dioxus::document::eval(POINTER);let _=query.join::<Value>().await;});},
      header {h2 {"Number input tokens and steppers"} p {"Compare native numeric fields, generated controls and feedback under authored test tokens. Numeric parsing, source mutation and complete form semantics remain separate capabilities."}}
      section {class:"number-input-pointer-panel",h3 {"Pointer states"} p {"Choose a state and arm the field, then press and hold in the input or on a stepper."}
       label {"Pointer case",select {id:"number-input-pointer-case",value:pointer_key(),onchange:move|event|{pointer_key.set(event.value());pointer.reset();},for(name,_)in STATES{option {value:name,"{name}"}}}}
       button {id:"number-input-pointer-arm",r#type:"button",onclick:move|_|pointer.reset(),"Arm pointer capture"}
       div {id:"number-input-pointer-target","data-pointer-case":pointer_key(),"data-arm":(pointer.commands)().to_string(),NumberCase {case:pointer_case(&pointer_key()),revision:0,supplied:true,measured:false,consumer:Some(pointer)}}
       output {id:"number-input-pointer-status",role:"status","Awaiting pointer input"} pre {id:"number-input-pointer-result","aria-hidden":"true"}
      }
      fieldset {class:"number-input-review-controls",legend {"Token and prop revisions"}
       for next in["baseline","revised"]{button {id:format!("number-input-{next}"),r#type:"button",onclick:move|_|{revision.set(usize::from(next=="revised"));phase.set(next.into());supplied.set(true);},if next=="baseline"{"Baseline"}else{"Revise tokens and props"}}}
       label {input {id:"number-input-supplied",r#type:"checkbox",checked:supplied(),onchange:move|event|supplied.set(event.checked())} "Supply test tokens"}
       button {id:"number-input-outside",r#type:"button","Keyboard target"} output {id:"number-input-status",role:"status","{phase()} — 52 cases"}
      }
      div {class:"number-input-review-matrix",for case in cases(){NumberCase {key:"{case.key}",case:case.clone(),revision:revision(),supplied:supplied(),measured:true}}}
      section {class:"number-input-consumer-panel",h3 {"Accepting finite-step consumer"} p {"The Rust consumer accepts raw numeric requests. This bounded example uses half steps from zero to four."}
       form {id:"number-input-consumer-form",onsubmit:move|event|event.prevent_default(),NumberCase {case:consumer_case,revision:0,supplied:true,measured:false,consumer:Some(consumer)}}
       div {class:"number-input-review-controls",
        button {id:"number-input-consumer-reset",r#type:"button",onclick:move|_|consumer.reset(),"Reset"}
        button {id:"number-input-consumer-external",r#type:"button",onclick:move|_|{consumer.value.set("1.5".into());consumer.command();},"Set 1.5 externally"}
        button {id:"number-input-consumer-disable",r#type:"button",onclick:move|_|{consumer.disabled.set(true);consumer.readonly.set(false);consumer.command();},"Disable"}
        button {id:"number-input-consumer-readonly",r#type:"button",onclick:move|_|{consumer.disabled.set(false);consumer.readonly.set(true);consumer.command();},"Read only"}
        button {id:"number-input-consumer-enable",r#type:"button",onclick:move|_|{consumer.disabled.set(false);consumer.readonly.set(false);consumer.command();},"Enable editing"}
       }
       input {id:"number-input-consumer-json",r#type:"hidden",value:consumer.snapshot()}
       output {role:"status","Raw value: {(consumer.value)()}; {(consumer.events)().len()} callbacks"}
      }
      section {class:"number-input-review-fallback",h3 {"Existing Brand fallback"} p {"No supplied test tokens in this provisional preview."} IxTheme {scheme:ColorScheme::Light,NumberInput {label:"Number label",show_stepper_buttons:true}}}
      pre {id:"number-input-result","aria-hidden":"true"}
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
                rsx! {span {class:"number-input-review-adornment","data-adornment-side":side,style:format!("display:inline-block;width:{width}px;height:18px"),if side=="start"{"S"}else{"E"}}},
            )
        }
        "icon24" => {
            Some(rsx! {Icon {name:"add",size:24,class:"number-input-review-adornment-icon"}})
        }
        _ => None,
    }
}
#[component]
fn NumberCase(
    case: Case,
    revision: usize,
    supplied: bool,
    measured: bool,
    #[props(default)] consumer: Option<Consumer>,
) -> Element {
    let case = current_case(case, revision);
    let mut edited = use_signal(|| None::<(usize, String)>);
    let mut callbacks = use_signal(Vec::<Value>::new);
    let value = consumer.map(|model| (model.value)()).unwrap_or_else(|| {
        edited()
            .filter(|(epoch, _)| *epoch == revision)
            .map(|(_, value)| value)
            .unwrap_or_else(|| case.value.clone())
    });
    let disabled = consumer
        .map(|model| (model.disabled)())
        .unwrap_or(case.condition == "disabled");
    let readonly = consumer
        .map(|model| (model.readonly)())
        .unwrap_or(case.condition == "readonly");
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
        "number-input-component-override"
    } else if case.extra == "narrow" {
        "number-input-narrow-control"
    } else {
        ""
    };
    let native_id = format!("number-input-native-{}", case.key);
    let props = json!({"kind":"number","state":state_name(case.state),"condition":case.condition,"showStepper":case.steppers,"label":case.label,"placeholder":case.placeholder,"value":case.value.parse::<f64>().ok(),"valueText":case.value,"helperText":case.helper,"validText":case.valid,"infoText":case.info,"warningText":case.warning,"invalidText":case.invalid,"extra":case.extra,"omitDefaults":case.extra=="defaults","required":case.required,"min":case.min,"max":case.max,"step":case.step,"start":case.start,"end":case.end,"textAlignment":if case.align_end{"end"}else{"start"},"width":case.host_width});
    rsx! {article {id:format!("number-input-{}",case.key),class:"number-input-review-case","data-number-input-row":if measured{Some("true")}else{None},"data-key":case.key.clone(),"data-kind":"number","data-condition":case.condition,"data-scope-index":case.scope.to_string(),"data-revision":revision.to_string(),"data-props":props.to_string(),
     if measured{h3 {class:"number-input-review-case-title","{case.key}"}}
     IxTheme {scheme,density,style:if supplied{sentinel_style(case.scope,revision)}else{String::new()},
      if case.extra=="defaults"{NumberInput {id:native_id,label:case.label.clone(),placeholder:case.placeholder.clone(),helper_text:case.helper.clone(),valid_text:case.valid.clone(),info_text:case.info.clone(),warning_text:case.warning.clone(),invalid_text:case.invalid.clone()}}
      else{NumberInput {id:native_id,label:case.label.clone(),value,placeholder:case.placeholder.clone(),disabled,readonly,state:case.state,required:case.required,show_stepper_buttons:case.steppers,min:case.min,max:case.max,step:case.step,class,text_alignment_end:case.align_end,name:if case.extra=="consumer"{Some("quantity".to_string())}else{None},helper_text:case.helper.clone(),valid_text:case.valid.clone(),info_text:case.info.clone(),warning_text:case.warning.clone(),invalid_text:case.invalid.clone(),start:adornment(case.start,"start",revision),end:adornment(case.end,"end",revision),onnumericvaluechange:move|value:Option<f64>|{callbacks.write().push(json!({"kind":"numeric","value":value}));if let Some(model)=consumer{model.record("numeric",json!(value));}},onvaluechange:move|value:String|{edited.set(Some((revision,value.clone())));callbacks.write().push(json!({"kind":"raw","value":value}));if let Some(mut model)=consumer{model.value.set(value.clone());model.record("raw",json!(value));}},oncommit:move|value:Option<f64>|{callbacks.write().push(json!({"kind":"commit","value":value}));if let Some(model)=consumer{model.record("commit",json!(value));}}}}
     }
     input {class:"number-input-review-consumer",r#type:"hidden",value:json!({"callbacks":callbacks()}).to_string()}
    }}
}
const STYLES: &str = r#"
html{font-size:16px}#number-input-review{font:400 15px/1.5 system-ui,sans-serif;padding:24px;color:#24333b;max-width:1500px;margin:auto}#number-input-review .ix-theme{min-height:0}#number-input-review h2,#number-input-review h3,#number-input-review p,#number-input-review legend{color:#24333b}#number-input-review h2{font-size:26px;margin:0 0 12px}#number-input-review p{max-width:72ch;margin:0 0 16px}.number-input-review-controls{display:flex;flex-wrap:wrap;align-items:center;gap:12px;padding:16px;margin:24px 0;border:1px solid #a9b8c2}.number-input-review-controls button,.number-input-pointer-panel select,.number-input-pointer-panel>button{font:inherit;padding:6px 10px}.number-input-review-controls label,.number-input-pointer-panel>label{display:flex;gap:8px;align-items:center}.number-input-review-matrix{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:28px 24px}.number-input-review-case{min-width:0}.number-input-review-case-title{font:600 13px/20px system-ui,sans-serif;margin:0 0 10px;padding:2px 4px;background:#eef2f5}.number-input-review-fallback,.number-input-consumer-panel{margin-top:32px}#number-input-review .number-input-narrow-control{width:200px}.number-input-pointer-panel{padding:16px;background:#eef2f5}.number-input-pointer-panel>label{margin:12px 0}#number-input-pointer-target{padding:16px 0}#number-input-review pre{max-height:150px;overflow:auto;white-space:pre-wrap;font-size:12px}@media(max-width:760px){#number-input-review{padding:16px}.number-input-review-matrix{grid-template-columns:1fr}}
"#;
pub const POINTER: &str = r#"
const root=document.getElementById('number-input-review');if(!root||root.dataset.pointerObserver==='true')return{installed:false};root.dataset.pointerObserver='true';
const panel=document.getElementById('number-input-pointer-target'),rows=[],fields=["display","width","height","minWidth","boxSizing","paddingTop","paddingRight","paddingBottom","paddingLeft","borderTopWidth","borderTopColor","borderTopStyle","borderTopLeftRadius","borderTopRightRadius","borderBottomLeftRadius","borderBottomRightRadius","backgroundColor","color","fontSize","lineHeight","fontWeight","fontFamily","fontStyle","letterSpacing","fontFeatureSettings","outlineColor","outlineWidth","outlineStyle","outlineOffset","opacity","pointerEvents","justifyContent","textAlign","marginLeft","marginRight","maxWidth","minHeight","maxHeight","position","flexShrink","gap","top","right","bottom","left","content","fill","stroke","strokeWidth","cursor","visibility","alignItems","marginTop","marginBottom","borderRightWidth","borderBottomWidth","borderLeftWidth","transform","transitionProperty","transitionDuration","transitionTimingFunction","transitionDelay","flexDirection","wordBreak","whiteSpace","overflow","marginBlockStart","marginBlockEnd","marginInlineStart","rowGap","columnGap","flexWrap","marginInlineEnd","minInlineSize","appearance","textOverflow","resize","caretColor","outline","userSelect"];
const metrics=(node,pseudo=null)=>{if(!node)return null;const c=getComputedStyle(node,pseudo),r=node.getBoundingClientRect();return{tag:node.tagName,attrs:Object.fromEntries(Array.from(node.attributes,a=>[a.name,a.value])),computed:Object.fromEntries(fields.map(k=>[k,c[k]])),rect:{x:r.x,y:r.y,width:r.width,height:r.height},pseudo,active:node.matches(':active'),hover:node.matches(':hover'),focus:node.matches(':focus'),focusVisible:node.matches(':focus-visible')};};
const input=()=>panel.querySelector('input.ix-input');
const take=(phase,event,original,before)=>{const host=panel.querySelector('.ix-number-input-native'),control=input();rows.push({phase,case:panel.dataset.pointerCase,arm:Number(panel.dataset.arm),target:original?.matches('.ix-number-minus')?'minus':original?.matches('.ix-number-plus')?'plus':'control',event:{type:event.type,trusted:event.isTrusted,pointerType:event.pointerType,clientX:event.clientX,clientY:event.clientY,defaultPrevented:event.defaultPrevented},before,after:{value:control.value,disabled:control.disabled,readonly:control.readOnly},host:metrics(host),control:metrics(control),placeholder:metrics(control,'::placeholder'),stepper:metrics(host.querySelector('.ix-number-steppers')),minus:metrics(host.querySelector('.ix-number-minus')),plus:metrics(host.querySelector('.ix-number-plus')),nativeFacts:{focused:document.activeElement===control,hover:control.matches(':hover'),active:control.matches(':active'),paddingReady:host.querySelector('.ix-input-wrap')?.dataset.paddingReady??null}});if(rows.length>64)rows.shift();document.getElementById('number-input-pointer-result').textContent=JSON.stringify({captures:rows});document.getElementById('number-input-pointer-status').textContent=rows.length+' actual pointer captures';};
let before=null;const actual=event=>event.composedPath().find(node=>node instanceof HTMLInputElement||node instanceof HTMLButtonElement);
panel.addEventListener('pointerdown',event=>{const original=actual(event),control=input();if(!original||!control)return;const previous={value:control.value,disabled:control.disabled,readonly:control.readOnly};before=previous;requestAnimationFrame(()=>take('pointerdown-frame',event,original,previous));});
panel.addEventListener('pointerup',event=>{const original=actual(event);if(!original)return;const previous=before;requestAnimationFrame(()=>requestAnimationFrame(()=>take('pointerup-second-frame',event,original,previous)));});
window.ixNumberInputPointerReview={snapshot:()=>({case:panel.dataset.pointerCase,arm:Number(panel.dataset.arm),captures:rows.slice(),control:metrics(input()),minus:metrics(panel.querySelector('.ix-number-minus')),plus:metrics(panel.querySelector('.ix-number-plus'))})};window.ixNumberInputTokenReview={pointer:()=>{const current=window.ixNumberInputPointerReview.snapshot();return{...current,controlRect:current.control?.rect??null,minusRect:current.minus?.rect??null,plusRect:current.plus?.rect??null};}};return{installed:true};
"#;
