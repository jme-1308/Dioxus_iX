//! Shared accepting Checkbox consumer and semantic-token review fixture.
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
pub const COMBINATIONS: [(&str, bool, bool); 4] = [
    ("off", false, false),
    ("checked", true, false),
    ("mixed", false, true),
    ("both", true, true),
];
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
        ("spacing-x-30", 8 + scope + revision),
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
        "#checkbox-review .checkbox-component-override{{--ix-checkbox-button--width:{}px;--ix-checkbox-button--height:{}px;--ix-checkbox--width:{}px;--ix-checkbox--min-width:{}px;--ix-checkbox--max-width:{}px;--ix-checkbox--height:{}px;--ix-checkbox--min-height:{}px;--ix-checkbox--max-height:{}px;--ix-checkbox--target-size:{}px;--ix-checkbox-label--gap:{}px;--ix-checkbox--border-width:2px;--ix-checkbox--outline-width--focus:3px;--ix-checkbox--focus--outline-offset:6px;--ix-checkbox-checkmark--color:rgb(245,247,249);}}",
        30 + 2 * revision,
        28 + 2 * revision,
        26 + 2 * revision,
        26 + 2 * revision,
        26 + 2 * revision,
        24 + 2 * revision,
        24 + 2 * revision,
        24 + 2 * revision,
        36 + 2 * revision,
        12 + revision
    )
}

#[derive(Clone, Copy, PartialEq)]
struct Consumer {
    checked: Signal<bool>,
    mixed: Signal<bool>,
    disabled: Signal<bool>,
    requests: Signal<Vec<Value>>,
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
        rows.push(json!({"sequence":sequence,"before":before,"value":value,"after":value,"mixedAtEvent":(self.mixed)(),"disabledAtEvent":(self.disabled)()}));
    }
    fn reset(mut self) {
        self.checked.set(false);
        self.mixed.set(false);
        self.disabled.set(false);
        self.requests.set(Vec::new());
        self.command();
    }
    fn snapshot(self) -> String {
        json!({"checked":(self.checked)(),"mixed":(self.mixed)(),"disabled":(self.disabled)(),"requests":(self.requests)()}).to_string()
    }
}

#[component]
pub fn CheckboxReview() -> Element {
    let mut revision = use_signal(|| 0_usize);
    let mut phase = use_signal(|| "baseline".to_string());
    let mut state = use_signal(|| "rest".to_string());
    let mut supplied = use_signal(|| true);
    let mut consumer = Consumer {
        checked: use_signal(|| false),
        mixed: use_signal(|| false),
        disabled: use_signal(|| false),
        requests: use_signal(Vec::<Value>::new),
        commands: use_signal(|| 0_usize),
    };
    let mut pointer = Consumer {
        checked: use_signal(|| false),
        mixed: use_signal(|| false),
        disabled: use_signal(|| false),
        requests: use_signal(Vec::<Value>::new),
        commands: use_signal(|| 0_usize),
    };
    let mut pointer_case = use_signal(|| "default-off".to_string());
    rsx! {
     style {{STYLES}} style {{override_rules(revision())}}
     IxTheme {scheme:ColorScheme::Light,class:"checkbox-review-shell",
      section {id:"checkbox-review","data-phase":phase(),"data-revision":revision().to_string(),"data-state":state(),"data-supplied":supplied().to_string(),"data-consumer-command":(consumer.commands)().to_string(),
       onmounted:move |_|{spawn(async move{let query=dioxus::document::eval(POINTER);let _=query.join::<Value>().await;});},
       header {h2 {"Checkbox states and target geometry"} p {"Inspect checked and mixed values independently across validation states. Supplied test tokens exercise integration; they are not Brand measurements or accessibility claims."}}
       section {class:"checkbox-pointer-panel",h3 {"Pointer interaction"} p {"Choose a case, then press and hold the checkbox. Captures record actual active and hover states."}
        label {"Pointer case",select {id:"checkbox-pointer-case",value:pointer_case(),onchange:move|event|{let next=event.value();pointer.checked.set(next.ends_with("checked")||next.ends_with("both"));pointer.mixed.set(next.ends_with("mixed")||next.ends_with("both"));pointer.requests.set(Vec::new());pointer_case.set(next);},for key in["default-off","default-checked","default-mixed","default-both","info-checked","warning-mixed","invalid-off"]{option {value:key,"{key}"}}}}
        IxTheme {scheme:ColorScheme::Light,style:sentinel_style(0,0),div {id:"checkbox-pointer-target","data-pointer-case":pointer_case(),Checkbox {label:"Check",class:"checkbox-pointer-control",checked:(pointer.checked)(),indeterminate:(pointer.mixed)(),state:if pointer_case().starts_with("info"){FieldState::Info}else if pointer_case().starts_with("warning"){FieldState::Warning}else if pointer_case().starts_with("invalid"){FieldState::Invalid}else{FieldState::Default},oncheckedchange:move|value|pointer.request(value)}}}
        output {id:"checkbox-pointer-status",role:"status","Awaiting pointer input"} pre {id:"checkbox-pointer-result","aria-hidden":"true"}
       }
       fieldset {class:"checkbox-review-controls",legend {"Shared preview controls"}
        label {"Visual state",select {id:"checkbox-state",value:state(),onchange:move|event|state.set(event.value()),for value in STATES{option {value,"{value}"}}}}
        label {input {id:"checkbox-supplied",r#type:"checkbox",checked:supplied(),onchange:move|event|supplied.set(event.checked())} "Supply test tokens"}
        for next in["baseline","revised","restored"]{button {id:format!("checkbox-{next}"),r#type:"button",onclick:move|_|{revision.set(usize::from(next=="revised"));phase.set(next.into());state.set("rest".into());supplied.set(true);},match next{"baseline"=>"Baseline","revised"=>"Revise tokens",_=>"Restore tokens"}}}
        output {id:"checkbox-status",role:"status","{phase()} / {state()} — 80 checkbox combinations"} button {id:"checkbox-outside",r#type:"button","Keyboard target"}
       }
       div {class:"checkbox-review-matrix",
        for(scope,index,scheme,density)in[("light-normal",0,ColorScheme::Light,Density::Normal),("light-compact",1,ColorScheme::Light,Density::Compact),("dark-normal",2,ColorScheme::Dark,Density::Normal),("dark-compact",3,ColorScheme::Dark,Density::Compact)]{
         section {id:format!("checkbox-scope-{scope}"),"data-checkbox-scope":scope,"data-scope-index":index.to_string(),"data-revision":revision().to_string(),h3 {"{scheme.as_str()} / {density.as_str()}"}
          IxTheme {scheme,density,style:if supplied(){sentinel_style(index,revision())}else{String::new()},div {class:"checkbox-review-samples",for(name,variant)in VARIANTS{for(key,checked,mixed)in COMBINATIONS{
           Sample {row_id:format!("checkbox-{scope}-{name}-{key}"),kind:"core",sample_key:format!("{name}-{key}"),boolean_key:key,variant_name:name,variant,checked,mixed,visual_state:state()}
          }}}}
         }
        }
       }
       section {class:"checkbox-review-extras","data-checkbox-scope":"light-normal","data-scope-index":"0","data-revision":revision().to_string(),h3 {"Defaults, overrides and an accepting consumer"}
        IxTheme {scheme:ColorScheme::Light,style:if supplied(){sentinel_style(0,revision())}else{String::new()},div {class:"checkbox-review-extra-grid",
         for key in["default","label-less","override","required-invalid"]{Sample {row_id:format!("checkbox-extra-{key}"),kind:"extra",sample_key:key,boolean_key:if key=="override"{"both"}else{"off"},variant_name:if key=="override"{"warning"}else if key=="required-invalid"{"invalid"}else{"default"},variant:if key=="override"{FieldState::Warning}else if key=="required-invalid"{FieldState::Invalid}else{FieldState::Default},checked:key=="override",mixed:key=="override"}}
         div {"data-checkbox-scope":"nested","data-scope-index":"3","data-revision":"0",IxTheme {scheme:ColorScheme::Dark,density:Density::Compact,style:sentinel_style(3,0),Sample {row_id:"checkbox-extra-nested",kind:"extra",sample_key:"nested",checked:true,boolean_key:"checked"}}}
         form {id:"checkbox-controlled-form",onsubmit:move|event|event.prevent_default(),Sample {row_id:"checkbox-extra-controlled",kind:"extra",sample_key:"controlled",consumer:Some(consumer)}}
        }}
       }
       fieldset {class:"checkbox-review-controls",legend {"Accepting checked-value consumer"} p {"Every checked request is accepted. Mixed remains independent until changed by these controls."}
        button {id:"checkbox-reset",r#type:"button",onclick:move|_|consumer.reset(),"Reset"}
        button {id:"checkbox-mixed",r#type:"button",onclick:move|_|{consumer.checked.set(false);consumer.mixed.set(true);consumer.command();},"Set mixed, unchecked"}
        button {id:"checkbox-enable",r#type:"button",onclick:move|_|{consumer.disabled.set(false);consumer.command();},"Enable"}
        button {id:"checkbox-disable",r#type:"button",onclick:move|_|{consumer.disabled.set(true);consumer.command();},"Disable"}
        button {id:"checkbox-external-checked",r#type:"button",onclick:move|_|{consumer.checked.set(true);consumer.mixed.set(false);consumer.command();},"Set checked externally"}
        output {role:"status","Checked: {(consumer.checked)()}; mixed: {(consumer.mixed)()}; {(consumer.requests)().len()} accepted requests"}
        input {id:"checkbox-controlled-json",r#type:"hidden",value:consumer.snapshot()}
       }
       section {id:"checkbox-brand-fallback",h3 {"Existing Brand fallback"} p {"No test tokens are supplied here. Remaining Brand metrics are provisional."}
        for density in[Density::Normal,Density::Compact]{h4 {"{density.as_str()}"} IxTheme {scheme:ColorScheme::Light,density,div {class:"checkbox-review-fallback-row",for(key,checked,mixed)in COMBINATIONS{div {span {class:"checkbox-review-label","{key}"} Checkbox {label:"Check",checked,indeterminate:mixed}}}}}}
       }
       pre {id:"checkbox-result","aria-hidden":"true"}
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
    #[props(default)] mixed: bool,
    #[props(default="rest".to_string())] visual_state: String,
    #[props(default)] consumer: Option<Consumer>,
) -> Element {
    let actual_checked = consumer.map(|v| (v.checked)()).unwrap_or(checked);
    let actual_mixed = consumer.map(|v| (v.mixed)()).unwrap_or(mixed);
    let disabled = consumer
        .map(|v| (v.disabled)())
        .unwrap_or(visual_state == "disabled");
    let required = sample_key == "required-invalid";
    rsx! {div {id:row_id,class:"checkbox-review-sample","data-checkbox-row":kind,"data-key":sample_key.clone(),"data-variant":variant_name,"data-boolean-key":boolean_key,"data-checked":actual_checked.to_string(),"data-mixed":actual_mixed.to_string(),"data-disabled":disabled.to_string(),"data-required":required.to_string(),
     span {class:"checkbox-review-label","{sample_key}"}
     if sample_key=="default"{Checkbox {label:"Check"}}
     else{Checkbox {label:if sample_key=="label-less"{String::new()}else{"Check".into()},aria_label:if sample_key=="label-less"{Some("Check".to_string())}else{None},class:if sample_key=="override"{"checkbox-component-override"}else{""},checked:actual_checked,indeterminate:actual_mixed,disabled,required,invalid:required,state:variant,name:consumer.map(|_|"checkbox-value".to_string()),value:"consent",oncheckedchange:move|value|{if let Some(model)=consumer{model.request(value);}}}}
    }}
}
const STYLES: &str = r#"
#checkbox-review{font:400 15px/1.5 system-ui,sans-serif;padding:24px;color:#24333b;max-width:1500px;margin:auto}#checkbox-review .ix-theme{min-height:0}#checkbox-review h2,#checkbox-review h3,#checkbox-review h4,#checkbox-review p,#checkbox-review legend,#checkbox-review .checkbox-review-label{color:#24333b}#checkbox-review h2{font-size:26px;margin:0 0 12px}#checkbox-review h3{font-size:18px;margin:26px 0 12px}#checkbox-review p{max-width:72ch;margin:0 0 16px}.checkbox-review-controls{display:flex;align-items:center;flex-wrap:wrap;gap:12px;margin:24px 0;padding:16px;border:1px solid #a9b8c2}.checkbox-review-controls label{display:flex;gap:8px;align-items:center}.checkbox-review-controls button,.checkbox-review-controls select,#checkbox-pointer-case{font:inherit;padding:6px 10px}.checkbox-review-matrix{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:24px}.checkbox-review-matrix>section{min-width:0}.checkbox-review-samples,.checkbox-review-extra-grid{padding:16px;display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:18px 12px}.checkbox-review-sample{min-width:0;display:flex;flex-direction:column;align-items:flex-start;gap:8px}.checkbox-review-label{font-size:12px;line-height:18px}.checkbox-review-fallback-row{display:flex;flex-wrap:wrap;gap:24px}.checkbox-review-fallback-row>div{display:flex;flex-direction:column;gap:8px}.checkbox-pointer-panel{padding:16px;background:#eef3f5}.checkbox-pointer-panel label{display:flex;gap:12px;flex-wrap:wrap;margin:12px 0}#checkbox-pointer-target{display:inline-block;margin:24px;padding:8px}#checkbox-review pre{max-height:150px;overflow:auto;white-space:pre-wrap;font-size:12px}@media(max-width:760px){#checkbox-review{padding:16px}.checkbox-review-matrix{grid-template-columns:1fr}.checkbox-review-samples,.checkbox-review-extra-grid{grid-template-columns:1fr}}
"#;
pub const POINTER: &str = r#"
const root=document.getElementById('checkbox-review');
if(!root||root.dataset.pointerObserver==='true')return {installed:false};
root.dataset.pointerObserver='true';
const panel=document.getElementById('checkbox-pointer-target'),rows=[];
const take=(phase,event,before)=>{const host=panel.querySelector('.ix-checkbox-native'),button=host?.querySelector('button'),shape=host?.querySelector('.ix-check-shape'),wrapper=host?.querySelector('.ix-checkbox-button');if(!host||!button||!shape)return;const fields=['backgroundColor','color','borderTopColor','borderTopWidth','outlineColor','outlineWidth','outlineStyle','outlineOffset','width','height','pointerEvents'];const facts=node=>{const c=getComputedStyle(node),r=node.getBoundingClientRect();return{tag:node.tagName,attrs:Object.fromEntries(Array.from(node.attributes,a=>[a.name,a.value])),computed:Object.fromEntries(fields.map(k=>[k,c[k]])),rect:{x:r.x,y:r.y,width:r.width,height:r.height},active:node.matches(':active'),hover:node.matches(':hover'),focus:node.matches(':focus'),focusVisible:node.matches(':focus-visible')};};const after={checked:host.dataset.checked==='true',mixed:host.dataset.mixed==='true'};rows.push({phase,case:panel.dataset.pointerCase,event:{type:event.type,trusted:event.isTrusted,pointerType:event.pointerType,clientX:event.clientX,clientY:event.clientY,defaultPrevented:event.defaultPrevented},before,after,host:facts(host),control:facts(button),shape:facts(shape),wrapper:facts(wrapper)});document.getElementById('checkbox-pointer-result').textContent=JSON.stringify({captures:rows});document.getElementById('checkbox-pointer-status').textContent=rows.length+' actual pointer captures';};
let before=null;
panel.querySelector('.ix-checkbox-native').addEventListener('pointerdown',event=>{const host=panel.querySelector('.ix-checkbox-native');before={checked:host?.dataset.checked==='true',mixed:host?.dataset.mixed==='true'};requestAnimationFrame(()=>take('pointerdown-frame',event,before));});
panel.querySelector('.ix-checkbox-native').addEventListener('pointerup',event=>{const previous=before;requestAnimationFrame(()=>requestAnimationFrame(()=>take('pointerup-second-frame',event,previous)));});
return {installed:true};
"#;
