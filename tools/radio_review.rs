//! Shared accepting Radio consumer and semantic-token review fixture.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};

pub const STATES: [&str; 3] = ["rest", "disabled", "focus"];
pub const VARIANTS: [(&str, FieldState); 5] = [
    ("default", FieldState::Default),
    ("valid", FieldState::Valid),
    ("info", FieldState::Info),
    ("warning", FieldState::Warning),
    ("invalid", FieldState::Invalid),
];
pub const COMBINATIONS: [(&str, bool); 2] = [("off", false), ("checked", true)];
const COLORS: [&str; 27] = [
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
];
pub fn sentinel_style(scope: usize, revision: usize) -> String {
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
        ("size-60", size),
        ("size-70", size + 4),
        ("size-30", 8 + scope + revision),
        ("spacing-y-10", 2 + scope + revision),
        ("spacing-x-40", 10 + scope + revision),
        ("border-radius-full", 99 + scope + revision),
        ("border-width-default", 1 + revision),
        ("focus-ring-offset", 3 + revision),
    ] {
        style.push_str(&format!("--si-sys-sizing-{token}:{value}px;"));
    }
    style.push_str(&format!("font:400 16px/24px system-ui,sans-serif;color:var(--si-sys-color-text-primary);--si-sys-typography-body:400 {}px/{}px system-ui,sans-serif;--si-ref-typography-letter-spacing-normal:{}px;",14+scope+revision,20+scope+revision,0.25+0.25*revision as f64));
    style
}
fn override_rules(revision: usize) -> String {
    format!(
        "#radio-review .radio-component-override{{--ix-radio-button--width:{}px;--ix-radio-button--height:{}px;--ix-radio-checkmark--width:{}px;--ix-radio-checkmark--min-width:{}px;--ix-radio-checkmark--max-width:{}px;--ix-radio-checkmark--height:{}px;--ix-radio-checkmark--min-height:{}px;--ix-radio-checkmark--max-height:{}px;--ix-radio--target-size:{}px;--ix-radio-selection-indicator--width:{}px;--ix-radio-selection-indicator--height:{}px;--ix-radio-checkmark--border-radius:{}px;--ix-radio--margin:3px 0 5px 14px;--ix-radio--border-width:2px;--ix-radio--outline-width--focus:3px;--ix-radio--focus--outline-offset:6px;--ix-radio-checkmark--background:rgb(245,247,249);}}",
        30 + 2 * revision,
        28 + 2 * revision,
        26 + 2 * revision,
        26 + 2 * revision,
        26 + 2 * revision,
        24 + 2 * revision,
        24 + 2 * revision,
        24 + 2 * revision,
        36 + 2 * revision,
        11 + revision,
        7 + revision,
        13 + revision
    )
}

#[derive(Clone, Copy, PartialEq)]
struct Consumer {
    checked: Signal<bool>,
    disabled: Signal<bool>,
    requests: Signal<Vec<Value>>,
    values: Signal<Vec<Value>>,
    commands: Signal<usize>,
}
impl Consumer {
    fn command(mut self) {
        self.commands += 1;
    }
    fn request(mut self, value: bool) {
        let before = (self.checked)();
        self.checked.set(value);
        let mut rows = self.requests.write();
        let sequence = rows.len() + 1;
        rows.push(json!({"sequence":sequence,"before":before,"value":value,"after":value,"disabledAtEvent":(self.disabled)()}));
    }
    fn reset(mut self) {
        self.checked.set(false);
        self.disabled.set(false);
        self.requests.set(Vec::new());
        self.values.set(Vec::new());
        self.command();
    }
    fn snapshot(self) -> String {
        json!({"checked":(self.checked)(),"disabled":(self.disabled)(),"requests":(self.requests)(),"values":(self.values)()}).to_string()
    }
}

#[component]
pub fn RadioReview() -> Element {
    let mut revision = use_signal(|| 0_usize);
    let mut phase = use_signal(|| "baseline".to_string());
    let mut state = use_signal(|| "rest".to_string());
    let mut supplied = use_signal(|| true);
    let mut consumer = Consumer {
        checked: use_signal(|| false),
        disabled: use_signal(|| false),
        requests: use_signal(Vec::<Value>::new),
        values: use_signal(Vec::<Value>::new),
        commands: use_signal(|| 0_usize),
    };
    let mut pointer = Consumer {
        checked: use_signal(|| false),
        disabled: use_signal(|| false),
        requests: use_signal(Vec::<Value>::new),
        values: use_signal(Vec::<Value>::new),
        commands: use_signal(|| 0_usize),
    };
    let mut pointer_case = use_signal(|| "default-off".to_string());
    rsx! {
     style {{STYLES}} style {{override_rules(revision())}}
     IxTheme {scheme:ColorScheme::Light,class:"radio-review-shell",
      section {id:"radio-review","data-phase":phase(),"data-revision":revision().to_string(),"data-state":state(),"data-supplied":supplied().to_string(),"data-consumer-command":(consumer.commands)().to_string(),
       onmounted:move |_|{spawn(async move{let query=dioxus::document::eval(POINTER);let _=query.join::<Value>().await;});},
       header {h2 {"Radio selection and target geometry"} p {"Inspect selected and unselected controls across validation states. Supplied test tokens exercise integration; they are not Brand measurements or accessibility claims."}}
       section {class:"radio-pointer-panel",h3 {"Pointer interaction"} p {"Choose a case, then press and hold the radio. Captures record actual active and hover states."}
        label {"Pointer case",select {id:"radio-pointer-case",value:pointer_case(),onchange:move|event|{let next=event.value();pointer.checked.set(next.ends_with("checked"));pointer.requests.set(Vec::new());pointer_case.set(next);},for key in["default-off","default-checked","info-off","info-checked","warning-off","warning-checked","invalid-off"]{option {value:key,"{key}"}}}}
        IxTheme {scheme:ColorScheme::Light,style:sentinel_style(0,0),div {id:"radio-pointer-target","data-pointer-case":pointer_case(),Radio {label:"Select",value:"consent",class:"radio-pointer-control",checked:(pointer.checked)(),state:if pointer_case().starts_with("info"){FieldState::Info}else if pointer_case().starts_with("warning"){FieldState::Warning}else if pointer_case().starts_with("invalid"){FieldState::Invalid}else{FieldState::Default},oncheckedchange:move|value|pointer.request(value)}}}
        output {id:"radio-pointer-status",role:"status","Awaiting pointer input"} pre {id:"radio-pointer-result","aria-hidden":"true"}
       }
       fieldset {class:"radio-review-controls",legend {"Shared preview controls"}
        label {"Visual state",select {id:"radio-state",value:state(),onchange:move|event|state.set(event.value()),for value in STATES{option {value,"{value}"}}}}
        label {input {id:"radio-supplied",r#type:"checkbox",checked:supplied(),onchange:move|event|supplied.set(event.checked())} "Supply test tokens"}
        for next in["baseline","revised","restored"]{button {id:format!("radio-{next}"),r#type:"button",onclick:move|_|{revision.set(usize::from(next=="revised"));phase.set(next.into());state.set("rest".into());supplied.set(true);},match next{"baseline"=>"Baseline","revised"=>"Revise tokens",_=>"Restore tokens"}}}
        output {id:"radio-status",role:"status","{phase()} / {state()} — 40 radio combinations"} button {id:"radio-outside",r#type:"button","Keyboard target"}
       }
       div {class:"radio-review-matrix",
        for(scope,index,scheme,density)in[("light-normal",0,ColorScheme::Light,Density::Normal),("light-compact",1,ColorScheme::Light,Density::Compact),("dark-normal",2,ColorScheme::Dark,Density::Normal),("dark-compact",3,ColorScheme::Dark,Density::Compact)]{
         section {id:format!("radio-scope-{scope}"),"data-radio-scope":scope,"data-scope-index":index.to_string(),"data-revision":revision().to_string(),h3 {"{scheme.as_str()} / {density.as_str()}"}
          IxTheme {scheme,density,style:if supplied(){sentinel_style(index,revision())}else{String::new()},div {class:"radio-review-samples",for(name,variant)in VARIANTS{for(key,checked)in COMBINATIONS{
           Sample {row_id:format!("radio-{scope}-{name}-{key}"),kind:"core",sample_key:format!("{name}-{key}"),boolean_key:key,variant_name:name,variant,checked,visual_state:state()}
          }}}}
         }
        }
       }
       section {class:"radio-review-extras","data-radio-scope":"light-normal","data-scope-index":"0","data-revision":revision().to_string(),h3 {"Defaults, overrides and an accepting consumer"}
        IxTheme {scheme:ColorScheme::Light,style:if supplied(){sentinel_style(0,revision())}else{String::new()},div {class:"radio-review-extra-grid",
         for key in["default","label-less","override","required-invalid"]{Sample {row_id:format!("radio-extra-{key}"),kind:"extra",sample_key:key,boolean_key:if key=="override"{"checked"}else{"off"},variant_name:if key=="override"{"warning"}else if key=="required-invalid"{"invalid"}else{"default"},variant:if key=="override"{FieldState::Warning}else if key=="required-invalid"{FieldState::Invalid}else{FieldState::Default},checked:key=="override"}}
         div {"data-radio-scope":"nested","data-scope-index":"3","data-revision":"0",IxTheme {scheme:ColorScheme::Dark,density:Density::Compact,style:sentinel_style(3,0),Sample {row_id:"radio-extra-nested",kind:"extra",sample_key:"nested",checked:true,boolean_key:"checked"}}}
         form {id:"radio-controlled-form",onsubmit:move|event|event.prevent_default(),Sample {row_id:"radio-extra-controlled",kind:"extra",sample_key:"controlled",consumer:Some(consumer)} div {id:"radio-layout-peer","Layout reference"}}
        }}
       }
       fieldset {class:"radio-review-controls",legend {"Accepting checked-value consumer"} p {"Every checked request is accepted. Clicking an already selected radio keeps it selected."}
        button {id:"radio-reset",r#type:"button",onclick:move|_|consumer.reset(),"Reset"}
        button {id:"radio-off",r#type:"button",onclick:move|_|{consumer.checked.set(false);consumer.command();},"Clear selection externally"}
        button {id:"radio-enable",r#type:"button",onclick:move|_|{consumer.disabled.set(false);consumer.command();},"Enable"}
        button {id:"radio-disable",r#type:"button",onclick:move|_|{consumer.disabled.set(true);consumer.command();},"Disable"}
        button {id:"radio-external-checked",r#type:"button",onclick:move|_|{consumer.checked.set(true);consumer.command();},"Set checked externally"}
        output {role:"status","Checked: {(consumer.checked)()}; {(consumer.requests)().len()} accepted requests"}
        input {id:"radio-controlled-json",r#type:"hidden",value:consumer.snapshot()}
       }
       section {id:"radio-brand-fallback",h3 {"Existing Brand fallback"} p {"No test tokens are supplied here. Remaining Brand metrics are provisional."}
        for density in[Density::Normal,Density::Compact]{h4 {"{density.as_str()}"} IxTheme {scheme:ColorScheme::Light,density,div {class:"radio-review-fallback-row",for(key,checked)in COMBINATIONS{div {span {class:"radio-review-label","{key}"} Radio {label:"Select",value:format!("fallback-{key}"),checked}}}}}}
       }
       pre {id:"radio-result","aria-hidden":"true"}
      }
     }
    }
}
#[component]
fn Sample(
    row_id: String,
    kind: String,
    sample_key: String,
    #[props(default="off".to_string())] boolean_key: String,
    #[props(default="default".to_string())] variant_name: String,
    #[props(default)] variant: FieldState,
    #[props(default)] checked: bool,
    #[props(default="rest".to_string())] visual_state: String,
    #[props(default)] consumer: Option<Consumer>,
) -> Element {
    let actual_checked = consumer.map(|v| (v.checked)()).unwrap_or(checked);
    let disabled = consumer
        .map(|v| (v.disabled)())
        .unwrap_or(visual_state == "disabled");
    let required = sample_key == "required-invalid";
    rsx! {div {id:row_id,class:"radio-review-sample","data-radio-row":kind,"data-key":sample_key.clone(),"data-variant":variant_name,"data-boolean-key":boolean_key,"data-checked":actual_checked.to_string(),"data-disabled":disabled.to_string(),"data-required":required.to_string(),
     span {class:"radio-review-label","{sample_key}"}
     if sample_key=="default" {Radio {label:"Select",value:"consent"}}
     else {Radio {label:if sample_key=="label-less"{String::new()}else{"Select".into()},aria_label:if sample_key=="label-less"{Some("Select".to_string())}else{None},class:if sample_key=="override"{"radio-component-override"}else{""},value:"consent",name:consumer.map(|_|"radio-value".to_string()),checked:actual_checked,disabled,required,state:variant,oncheckedchange:move|value|{if let Some(model)=consumer {model.request(value);}},onvaluechange:move|value|{if let Some(mut model)=consumer{model.values.write().push(json!({"sequence":(model.requests)().len(),"value":value}));}}}}
    }}
}
const STYLES: &str = include_str!("styles/radio_review.css");
pub const POINTER: &str = r#"
const root=document.getElementById('radio-review');
if(!root||root.dataset.pointerObserver==='true')return {installed:false};
root.dataset.pointerObserver='true';
const panel=document.getElementById('radio-pointer-target'),host=panel.querySelector('.ix-radio-native'),rows=[];
const take=(phase,event,before)=>{const input=host?.querySelector('input[type=radio]'),shape=host?.querySelector('.ix-check-shape'),wrapper=host?.querySelector('.ix-radio-button'),dot=host?.querySelector('.ix-radio-dot');if(!host||!input||!shape)return;const fields=['backgroundColor','color','borderTopColor','borderTopWidth','outlineColor','outlineWidth','outlineStyle','outlineOffset','width','height','visibility','pointerEvents'];const facts=node=>{const c=getComputedStyle(node),r=node.getBoundingClientRect();return{tag:node.tagName,attrs:Object.fromEntries(Array.from(node.attributes,a=>[a.name,a.value])),computed:Object.fromEntries(fields.map(k=>[k,c[k]])),rect:{x:r.x,y:r.y,width:r.width,height:r.height},active:node.matches(':active'),hover:node.matches(':hover'),focus:node.matches(':focus'),focusVisible:node.matches(':focus-visible')};};rows.push({phase,case:panel.dataset.pointerCase,event:{type:event.type,trusted:event.isTrusted,pointerType:event.pointerType,clientX:event.clientX,clientY:event.clientY,defaultPrevented:event.defaultPrevented},before,after:{checked:host.dataset.checked==='true'},host:facts(host),control:facts(input),layout:facts(host),shape:facts(shape),wrapper:facts(wrapper),dot:facts(dot)});document.getElementById('radio-pointer-result').textContent=JSON.stringify({captures:rows});document.getElementById('radio-pointer-status').textContent=rows.length+' actual pointer captures';};
let before=null;
host.addEventListener('pointerdown',event=>{before={checked:host.dataset.checked==='true'};requestAnimationFrame(()=>take('pointerdown-frame',event,before));});
host.addEventListener('pointerup',event=>{const previous=before;requestAnimationFrame(()=>requestAnimationFrame(()=>take('pointerup-second-frame',event,previous)));});
return {installed:true};
"#;
