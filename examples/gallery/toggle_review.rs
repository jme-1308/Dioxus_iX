//! Shared accepting Toggle consumer and semantic-token review fixture.
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
    "effects-focus",
    "text-disabled",
    "text-primary",
    "background-accent",
    "background-accent-active",
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
    "background-1",
    "background-selected",
    "background-hover",
    "border-2",
    "border-danger",
    "text-on-accent",
    "text-inverse",
    "text-on-information",
    "text-on-danger",
    "text-on-warning",
    "text-secondary",
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
        ("size-100", 44 + 2 * scope + 4 * revision),
        ("spacing-x-10", 2 + scope + revision),
        ("spacing-y-10", 1 + scope + revision),
        ("spacing-x-40", 10 + scope + revision),
        ("spacing-x-50", 11 + scope + revision),
        ("spacing-y-20", 2 + scope + revision),
        ("spacing-y-40", 7 + scope + revision),
        ("border-radius-full", 99 + scope + revision),
        ("border-width-default", 1 + revision),
        ("focus-ring-offset", 3 + revision),
    ] {
        style.push_str(&format!("--si-sys-sizing-{token}:{value}px;"));
    }
    style.push_str(&format!("font:400 16px/24px system-ui,sans-serif;color:var(--si-sys-color-text-primary);--si-sys-typography-body:400 {}px/{}px system-ui,sans-serif;--si-ref-typography-letter-spacing-normal:{}px;",14+scope+revision,20+scope+revision,0.25+0.25*revision as f64));
    style.push_str(if revision == 1 {
        "--theme-default-time:0.001s;"
    } else {
        "--theme-default-time:0s;"
    });
    style
}
fn override_rules(revision: usize) -> String {
    format!(
        "#toggle-review .toggle-component-override{{--ix-toggle--min-height:{}px;--ix-toggle--max-height:100px;--ix-toggle--margin-block-start:4px;--ix-toggle--margin-block-end:6px;--ix-toggle-switch--width:{}px;--ix-toggle-switch--min-width:{}px;--ix-toggle-switch--max-width:{}px;--ix-toggle-switch--height:{}px;--ix-toggle-slider--width:{}px;--ix-toggle-slider--height:{}px;--ix-toggle-slider--left:3px;--ix-toggle-slider--bottom:4px;--ix-toggle-slider-checked--transform:translateX({}px);--ix-toggle-slider--transform:translateX({}px);--ix-toggle-slider--border-radius:{}px;--ix-toggle-slider--border-width:2px;--ix-toggle-switch--border-width:2px;--ix-toggle--outline-width--focus:3px;--ix-toggle-switch--outline-offset:6px;--ix-toggle-label--margin-block-start:5px;--ix-toggle-label--margin-inline-start:13px;--ix-toggle-label--min-height:26px;}}",
        36 + 2 * revision,
        60 + 2 * revision,
        60 + 2 * revision,
        60 + 2 * revision,
        32 + 2 * revision,
        17 + revision,
        13 + revision,
        31 + 2 * revision,
        15 + revision,
        12 + revision
    )
}

#[derive(Clone, Copy, PartialEq)]
struct Consumer {
    checked: Signal<bool>,
    mixed: Signal<bool>,
    disabled: Signal<bool>,
    hide_text: Signal<bool>,
    requests: Signal<Vec<Value>>,
    commands: Signal<usize>,
}
impl Consumer {
    fn command(mut self) {
        self.commands += 1;
    }
    fn request(mut self, value: bool) {
        let before = (self.checked)();
        let before_mixed = (self.mixed)();
        self.checked.set(value);
        self.mixed.set(false);
        let mut rows = self.requests.write();
        let sequence = rows.len() + 1;
        rows.push(json!({"sequence":sequence,"before":before,"beforeMixed":before_mixed,"value":value,"after":value,"mixedAtEvent":(self.mixed)(),"disabledAtEvent":(self.disabled)()}));
    }
    fn reset(mut self) {
        self.checked.set(false);
        self.mixed.set(false);
        self.disabled.set(false);
        self.hide_text.set(false);
        self.requests.set(Vec::new());
        self.command();
    }
    fn snapshot(self) -> String {
        json!({"checked":(self.checked)(),"mixed":(self.mixed)(),"disabled":(self.disabled)(),"hideText":(self.hide_text)(),"requests":(self.requests)()}).to_string()
    }
}

#[component]
pub fn ToggleReview() -> Element {
    let mut revision = use_signal(|| 0_usize);
    let mut phase = use_signal(|| "baseline".to_string());
    let mut state = use_signal(|| "rest".to_string());
    let mut supplied = use_signal(|| true);
    let mut consumer = Consumer {
        checked: use_signal(|| false),
        mixed: use_signal(|| false),
        disabled: use_signal(|| false),
        hide_text: use_signal(|| false),
        requests: use_signal(Vec::<Value>::new),
        commands: use_signal(|| 0_usize),
    };
    let mut pointer = Consumer {
        checked: use_signal(|| false),
        mixed: use_signal(|| false),
        disabled: use_signal(|| false),
        hide_text: use_signal(|| false),
        requests: use_signal(Vec::<Value>::new),
        commands: use_signal(|| 0_usize),
    };
    let mut pointer_case = use_signal(|| "default-off".to_string());
    rsx! {
     style {{STYLES}} style {{override_rules(revision())}}
     IxTheme {scheme:ColorScheme::Light,class:"toggle-review-shell",
      section {id:"toggle-review","data-phase":phase(),"data-revision":revision().to_string(),"data-state":state(),"data-supplied":supplied().to_string(),"data-consumer-command":(consumer.commands)().to_string(),
       onmounted:move |_|{spawn(async move{let query=dioxus::document::eval(POINTER);let _=query.join::<Value>().await;});},
       header {h2 {"Toggle states and target geometry"} p {"Inspect checked and mixed switches independently across validation states. Supplied test tokens exercise integration; they are not Brand measurements or accessibility claims."}}
       section {class:"toggle-pointer-panel",h3 {"Pointer interaction"} p {"Choose a case, then press and hold the toggle. Captures record actual active and hover states."}
        label {"Pointer case",select {id:"toggle-pointer-case",value:pointer_case(),onchange:move|event|{let next=event.value();pointer.checked.set(next.ends_with("checked")||next.ends_with("both"));pointer.mixed.set(next.ends_with("mixed")||next.ends_with("both"));pointer.requests.set(Vec::new());pointer_case.set(next);},for key in["default-off","default-checked","default-mixed","default-both","info-checked","warning-mixed","invalid-off"]{option {value:key,"{key}"}}}}
        IxTheme {scheme:ColorScheme::Light,style:sentinel_style(0,0),div {id:"toggle-pointer-target","data-pointer-case":pointer_case(),Toggle {aria_label:"Setting",class:"toggle-pointer-control",checked:(pointer.checked)(),indeterminate:(pointer.mixed)(),state:if pointer_case().starts_with("info"){FieldState::Info}else if pointer_case().starts_with("warning"){FieldState::Warning}else if pointer_case().starts_with("invalid"){FieldState::Invalid}else{FieldState::Default},oncheckedchange:move|value|pointer.request(value)}}}
        output {id:"toggle-pointer-status",role:"status","Awaiting pointer input"} pre {id:"toggle-pointer-result","aria-hidden":"true"}
       }
       fieldset {class:"toggle-review-controls",legend {"Shared preview controls"}
        label {"Visual state",select {id:"toggle-state",value:state(),onchange:move|event|state.set(event.value()),for value in STATES{option {value,"{value}"}}}}
        label {input {id:"toggle-supplied",r#type:"checkbox",checked:supplied(),onchange:move|event|supplied.set(event.checked())} "Supply test tokens"}
        for next in["baseline","revised","restored"]{button {id:format!("toggle-{next}"),r#type:"button",onclick:move|_|{revision.set(usize::from(next=="revised"));phase.set(next.into());state.set("rest".into());supplied.set(true);},match next{"baseline"=>"Baseline","revised"=>"Revise tokens",_=>"Restore tokens"}}}
        output {id:"toggle-status",role:"status","{phase()} / {state()} — 80 toggle combinations"} button {id:"toggle-outside",r#type:"button","Keyboard target"}
       }
       div {class:"toggle-review-matrix",
        for(scope,index,scheme,density)in[("light-normal",0,ColorScheme::Light,Density::Normal),("light-compact",1,ColorScheme::Light,Density::Compact),("dark-normal",2,ColorScheme::Dark,Density::Normal),("dark-compact",3,ColorScheme::Dark,Density::Compact)]{
         section {id:format!("toggle-scope-{scope}"),"data-toggle-scope":scope,"data-scope-index":index.to_string(),"data-revision":revision().to_string(),h3 {"{scheme.as_str()} / {density.as_str()}"}
          IxTheme {scheme,density,style:if supplied(){sentinel_style(index,revision())}else{String::new()},div {class:"toggle-review-samples",for(name,variant)in VARIANTS{for(key,checked,mixed)in COMBINATIONS{
           Sample {row_id:format!("toggle-{scope}-{name}-{key}"),kind:"core",sample_key:format!("{name}-{key}"),boolean_key:key,variant_name:name,variant,checked,mixed,visual_state:state()}
          }}}}
         }
        }
       }
       section {class:"toggle-review-extras","data-toggle-scope":"light-normal","data-scope-index":"0","data-revision":revision().to_string(),h3 {"Defaults, overrides and an accepting consumer"}
        IxTheme {scheme:ColorScheme::Light,style:if supplied(){sentinel_style(0,revision())}else{String::new()},div {class:"toggle-review-extra-grid",
         for key in["default","hide-text","stable-label","override","required-invalid"] {Sample {row_id:format!("toggle-extra-{key}"),kind:"extra",sample_key:key,boolean_key:if key=="override"{"both"}else if key=="stable-label"{"mixed"}else if key=="hide-text"{"checked"}else{"off"},variant_name:match key{"override"=>"warning","stable-label"=>"info","required-invalid"=>"invalid",_=>"default"},variant:match key{"override"=>FieldState::Warning,"stable-label"=>FieldState::Info,"required-invalid"=>FieldState::Invalid,_=>FieldState::Default},checked:matches!(key,"override"|"hide-text"),mixed:matches!(key,"override"|"stable-label")}}
         div {"data-toggle-scope":"nested","data-scope-index":"3","data-revision":"0",IxTheme {scheme:ColorScheme::Dark,density:Density::Compact,style:sentinel_style(3,0),Sample {row_id:"toggle-extra-nested",kind:"extra",sample_key:"nested",checked:true,boolean_key:"checked",variant_name:"valid",variant:FieldState::Valid}}}
         form {id:"toggle-controlled-form",onsubmit:move|event|event.prevent_default(),Sample {row_id:"toggle-extra-controlled",kind:"extra",sample_key:"controlled",consumer:Some(consumer)}}
         Sample {row_id:"toggle-extra-wrapping",kind:"extra",sample_key:"wrapping"}

        }}
       }
       fieldset {class:"toggle-review-controls",legend {"Accepting checked-value consumer"} p {"Every checked request is accepted and clears mixed, matching the source accepting-state outcome."}
        button {id:"toggle-reset",r#type:"button",onclick:move|_|consumer.reset(),"Reset"}
        button {id:"toggle-mixed",r#type:"button",onclick:move|_|{consumer.checked.set(false);consumer.mixed.set(true);consumer.command();},"Set mixed, unchecked"}
        button {id:"toggle-enable",r#type:"button",onclick:move|_|{consumer.disabled.set(false);consumer.command();},"Enable"}
        button {id:"toggle-disable",r#type:"button",onclick:move|_|{consumer.disabled.set(true);consumer.command();},"Disable"}
        button {id:"toggle-external-checked",r#type:"button",onclick:move|_|{consumer.checked.set(true);consumer.mixed.set(false);consumer.command();},"Set checked externally"}
        output {role:"status","Checked: {(consumer.checked)()}; mixed: {(consumer.mixed)()}; {(consumer.requests)().len()} accepted requests"}
        for flag in["checked","mixed","disabled","hide-text"]{label {input {id:format!("toggle-controlled-{flag}"),r#type:"checkbox",checked:match flag{"checked"=>(consumer.checked)(),"mixed"=>(consumer.mixed)(),"disabled"=>(consumer.disabled)(),_=>(consumer.hide_text)()},onchange:move|event|{match flag{"checked"=>consumer.checked.set(event.checked()),"mixed"=>consumer.mixed.set(event.checked()),"disabled"=>consumer.disabled.set(event.checked()),_=>consumer.hide_text.set(event.checked())};consumer.command();}} "{flag}"}}
        input {id:"toggle-controlled-json",r#type:"hidden",value:consumer.snapshot()}
       }
       section {id:"toggle-brand-fallback",h3 {"Existing Brand fallback"} p {"No test tokens are supplied here. Remaining Brand metrics are provisional."}
        for density in[Density::Normal,Density::Compact]{h4 {"{density.as_str()}"} IxTheme {scheme:ColorScheme::Light,density,div {class:"toggle-review-fallback-row",for(key,checked,mixed)in COMBINATIONS{div {span {class:"toggle-review-label","{key}"} Toggle {aria_label:"Setting",checked,indeterminate:mixed}}}}}}
       }
       pre {id:"toggle-result","aria-hidden":"true"}
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
    let hide_text = consumer
        .map(|v| (v.hide_text)())
        .unwrap_or(sample_key == "hide-text");
    let stable = sample_key == "stable-label";
    let wrapping = sample_key == "wrapping";
    rsx! {div {id:row_id,class:"toggle-review-sample","data-toggle-row":kind,"data-key":sample_key.clone(),"data-variant":variant_name,"data-boolean-key":boolean_key,"data-checked":actual_checked.to_string(),"data-mixed":actual_mixed.to_string(),"data-disabled":disabled.to_string(),"data-required":required.to_string(),"data-hide-text":hide_text.to_string(),
     span {class:"toggle-review-label","{sample_key}"}
     if sample_key=="default" {Toggle {aria_label:"Setting"}}
     else {Toggle {aria_label:"Setting",class:if sample_key=="override"{"toggle-component-override"}else if wrapping{"toggle-wrapping-control"}else{""},checked:actual_checked,indeterminate:actual_mixed,disabled,required,hide_text,state:variant,name:consumer.map(|_|"toggle-value".to_string()),value:"consent",text_on:if stable{"Setting"}else if wrapping{"A longer setting label that wraps across several lines"}else{"On"},text_off:if stable{"Setting"}else if wrapping{"A longer setting label that wraps across several lines"}else{"Off"},text_indeterminate:if stable{"Setting"}else if wrapping{"A longer setting label that wraps across several lines"}else{"Mixed"},oncheckedchange:move|value|{if let Some(model)=consumer{model.request(value);}}}}
    }}
}
const STYLES: &str = r#"
#toggle-review{font:400 15px/1.5 system-ui,sans-serif;padding:24px;color:#24333b;max-width:1500px;margin:auto}#toggle-review .ix-theme{min-height:0}#toggle-review h2,#toggle-review h3,#toggle-review h4,#toggle-review p,#toggle-review legend,#toggle-review .toggle-review-label{color:#24333b}#toggle-review h2{font-size:26px;margin:0 0 12px}#toggle-review h3{font-size:18px;margin:26px 0 12px}#toggle-review p{max-width:72ch;margin:0 0 16px}.toggle-review-controls{display:flex;align-items:center;flex-wrap:wrap;gap:12px;margin:24px 0;padding:16px;border:1px solid #a9b8c2}.toggle-review-controls label{display:flex;gap:8px;align-items:center}.toggle-review-controls button,.toggle-review-controls select,#toggle-pointer-case{font:inherit;padding:6px 10px}.toggle-review-matrix{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:24px}.toggle-review-matrix>section{min-width:0}.toggle-review-samples,.toggle-review-extra-grid{padding:16px;display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:18px 12px}.toggle-review-sample{min-width:0;display:flex;flex-direction:column;align-items:flex-start;gap:8px}.toggle-review-label{font-size:12px;line-height:18px}.toggle-review-fallback-row{display:flex;flex-wrap:wrap;gap:24px}.toggle-review-fallback-row>div{display:flex;flex-direction:column;gap:8px}.toggle-pointer-panel{padding:16px;background:#eef2f5}.toggle-pointer-panel label{display:flex;gap:12px;flex-wrap:wrap;margin:12px 0}#toggle-pointer-target{display:inline-block;margin:24px;padding:8px}#toggle-review .toggle-wrapping-control{width:160px}#toggle-review .toggle-review-label{background:#eef2f5;padding:2px 4px}#toggle-review pre{max-height:150px;overflow:auto;white-space:pre-wrap;font-size:12px}@media(max-width:760px){#toggle-review{padding:16px}.toggle-review-matrix{grid-template-columns:1fr}.toggle-review-samples,.toggle-review-extra-grid{grid-template-columns:1fr}}
"#;
pub const POINTER: &str = r#"
const root=document.getElementById('toggle-review');if(!root||root.dataset.pointerObserver==='true')return {installed:false};root.dataset.pointerObserver='true';
const panel=document.getElementById('toggle-pointer-target'),control=panel.querySelector('.ix-toggle-control'),rows=[],fields=["display","width","height","minWidth","boxSizing","paddingTop","paddingRight","paddingBottom","paddingLeft","borderTopWidth","borderTopColor","borderTopStyle","borderTopLeftRadius","borderTopRightRadius","borderBottomLeftRadius","borderBottomRightRadius","backgroundColor","color","fontSize","lineHeight","fontWeight","fontFamily","fontStyle","letterSpacing","fontFeatureSettings","outlineColor","outlineWidth","outlineStyle","outlineOffset","opacity","pointerEvents","justifyContent","textAlign","marginLeft","marginRight","maxWidth","minHeight","maxHeight","position","flexShrink","gap","top","right","bottom","left","content","fill","stroke","strokeWidth","cursor","visibility","alignItems","marginTop","marginBottom","borderRightWidth","borderBottomWidth","borderLeftWidth","transform","transitionProperty","transitionDuration","transitionTimingFunction","transitionDelay","flexDirection","wordBreak","whiteSpace","overflow","marginBlockStart","marginBlockEnd","marginInlineStart"];
const metrics=(node,pseudo=null)=>{if(!node)return null;const c=getComputedStyle(node,pseudo),r=node.getBoundingClientRect();return{tag:node.tagName,attrs:Object.fromEntries(Array.from(node.attributes,a=>[a.name,a.value])),computed:Object.fromEntries(fields.map(k=>[k,c[k]])),rect:{x:r.x,y:r.y,width:r.width,height:r.height},pseudo,active:node.matches(':active'),hover:node.matches(':hover'),focus:node.matches(':focus'),focusVisible:node.matches(':focus-visible')};};
const state=()=>({checked:control.dataset.checked==='true',mixed:control.dataset.mixed==='true'});
const take=(phase,event,before)=>{const switchNode=control.querySelector('.ix-toggle-switch'),slider=control.querySelector('.ix-toggle-slider'),label=control.querySelector('.ix-toggle-label');rows.push({phase,case:panel.dataset.pointerCase,event:{type:event.type,trusted:event.isTrusted,pointerType:event.pointerType,clientX:event.clientX,clientY:event.clientY,defaultPrevented:event.defaultPrevented},before,after:state(),host:metrics(control),control:metrics(control),outer:metrics(panel.querySelector('.ix-toggle-native')),wrapper:metrics(control.querySelector('.ix-toggle-wrapper')),switch:metrics(switchNode),slider:metrics(slider),thumb:metrics(slider,'::before'),label:metrics(label),nativeFacts:{switchHover:switchNode.matches(':hover'),switchActive:switchNode.matches(':active'),hostHover:control.matches(':hover'),hostActive:control.matches(':active'),labelHover:label?.matches(':hover')??false}});if(rows.length>64)rows.shift();document.getElementById('toggle-pointer-result').textContent=JSON.stringify({captures:rows});document.getElementById('toggle-pointer-status').textContent=rows.length+' actual pointer captures';};
let before=null;control.addEventListener('pointerdown',event=>{before=state();requestAnimationFrame(()=>take('pointerdown-frame',event,before));});control.addEventListener('pointerup',event=>{const previous=before;requestAnimationFrame(()=>requestAnimationFrame(()=>take('pointerup-second-frame',event,previous)));});
control.querySelector('.ix-toggle-label')?.addEventListener('pointerenter',event=>{const previous=state();requestAnimationFrame(()=>take('label-hover-frame',event,previous));});
window.ixTogglePointerReview={snapshot:()=>({state:state(),captures:rows.slice(),switchRect:control.querySelector('.ix-toggle-switch').getBoundingClientRect().toJSON(),labelRect:control.querySelector('.ix-toggle-label')?.getBoundingClientRect().toJSON()??null})};
return {installed:true};
"#;
