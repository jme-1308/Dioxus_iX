//! Excluded native Radio computed-token runner; application intent stays Rust.
use dioxus::prelude::*;
#[cfg(any(
    target_arch = "wasm32",
    all(feature = "desktop", not(target_arch = "wasm32"))
))]
use serde_json::Value;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "radio_review.rs"]
mod radio_review;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    let source = std::env::var("IX_RADIO_SOURCE").is_ok_and(|value| value == "1");
    let index = if source {
        include_str!("upstream_radio_token_review.html").replace(
            "</body>",
            "<div id=\"main\" hidden></div><!-- MODULE LOADER --></body>",
        )
    } else {
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Radio sizing and states</title></head><body style=\"margin:0\"><div id=\"main\"></div><!-- MODULE LOADER --></body></html>".to_string()
    };
    let config = dioxus::desktop::Config::new()
        .with_custom_index(index)
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title(if source {
                    "Pinned iX Radio"
                } else {
                    "Dioxus iX Radio"
                })
                .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0, 1000.0)),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}
#[cfg(target_arch = "wasm32")]
fn main() {
    dioxus::launch(Probe);
}
#[cfg(all(not(target_arch = "wasm32"), not(feature = "desktop")))]
fn main() {
    let _fixture = Probe;
    eprintln!("Run the Radio review with desktop or web features");
}

#[component]
fn Probe() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let source = std::env::var("IX_RADIO_SOURCE").is_ok_and(|value| value == "1");
    #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
    let source = false;
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(180)).await;
            eprintln!("Radio token native timeout");
            std::process::exit(1);
        });
    });
    rsx! {div {
        onmounted:move |_| {
            #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
            {let window=window.clone();spawn(async move{
                let query=dom_query::eval(VERIFY);let _=query.send(source);
                match query.join::<Value>().await {
                    Ok(value) if value["ok"]==true=>{println!("Radio token native observations: {value}");window.close();}
                    Ok(value)=>{eprintln!("Radio token native failure JSON: {value}");std::process::exit(1);}
                    Err(error)=>{eprintln!("Radio token native transport failure: {error:?}");std::process::exit(1);}
                }
            });}
        },
        {web_controls()}
        if !source {radio_review::RadioReview {}}
    }}
}

#[cfg(target_arch = "wasm32")]
fn web_controls() -> Element {
    rsx! {WebRunner {}}
}
#[cfg(not(target_arch = "wasm32"))]
fn web_controls() -> Element {
    VNode::empty()
}

#[cfg(target_arch = "wasm32")]
#[component]
fn WebRunner() -> Element {
    let mut busy = use_signal(|| false);
    let mut result = use_signal(String::new);
    let mut status = use_signal(|| "idle".to_string());
    rsx! {section {style:"padding:16px 24px;font:14px/1.5 system-ui,sans-serif",
        button {id:"radio-run",r#type:"button",disabled:busy(),aria_busy:busy(),
            onclick:move |_| {
                if busy() {return;}
                busy.set(true);status.set("running".into());
                spawn(async move {
                    let query=dioxus::document::eval(VERIFY);
                    if let Err(error)=query.send(false) {
                        result.set(serde_json::json!({"ok":false,"error":format!("{error:?}")}).to_string());
                        status.set("error".into());busy.set(false);return;
                    }
                    match query.join::<Value>().await {
                        Ok(mut value)=>{
                            let ok=value["ok"]==true;
                            if let Some(object)=value.as_object_mut() {
                                object.insert("renderer".to_string(),Value::String("web browser Rust".to_string()));
                            }
                            result.set(value.to_string());status.set(if ok{"done"}else{"error"}.to_string());
                        }
                        Err(error)=>{
                            result.set(serde_json::json!({"ok":false,"error":format!("{error:?}")}).to_string());
                            status.set("error".into());
                        }
                    }
                    busy.set(false);
                });
            },
            "Run token states"
        }
        span {role:"status",style:"margin-left:12px",if status()=="done" {"Completed: 378 Radio token/state observations."} else {"{status()}"}}
        pre {id:"radio-web-result","data-run":status(),"aria-hidden":"true",style:"max-height:180px;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere;font-size:12px","{result()}"}
    }}
}

#[cfg(any(
    target_arch = "wasm32",
    all(feature = "desktop", not(target_arch = "wasm32"))
))]
const VERIFY: &str = r#"
const source=await dioxus.recv(),errors=[],phases=[],journal=[],assertions=[],objects=new WeakMap();let nextObject=0,sequence=0;
const abort=new AbortController(),sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const log=(kind,details={})=>journal.push({seq:++sequence,kind,time:performance.now(),...details});
const object=node=>{if(!objects.has(node))objects.set(node,++nextObject);return objects.get(node);};
window.addEventListener('error',event=>errors.push(event.message),{signal:abort.signal});window.addEventListener('unhandledrejection',event=>errors.push(String(event.reason)),{signal:abort.signal});
const wait=async(predicate,label)=>{const until=performance.now()+25000;while(performance.now()<until){if(predicate())return;await sleep(5);}throw Error('Timeout '+label);};
const check=(condition,label)=>{assertions.push({ok:!!condition,label});if(!condition)throw Error(label);};
const fields=['flexShrink','visibility','textAlign','display','position','width','height','minWidth','maxWidth','minHeight','maxHeight','boxSizing','paddingTop','paddingRight','paddingBottom','paddingLeft','borderTopWidth','borderRightWidth','borderBottomWidth','borderLeftWidth','borderTopColor','borderTopStyle','borderTopLeftRadius','borderTopRightRadius','borderBottomLeftRadius','borderBottomRightRadius','backgroundColor','color','fontSize','lineHeight','fontWeight','fontFamily','fontStyle','letterSpacing','fontFeatureSettings','outlineColor','outlineWidth','outlineStyle','outlineOffset','opacity','pointerEvents','cursor','gap','alignItems','justifyContent','marginTop','marginRight','marginBottom','marginLeft','top','right','bottom','left','content','fill','stroke','strokeWidth'];
const metrics=(node,pseudo=null)=>{if(!(node instanceof Element))return null;const cs=getComputedStyle(node,pseudo),r=node.getBoundingClientRect();return{object:object(node),tag:node.tagName,id:node.id,class:node.getAttribute('class'),attrs:Object.fromEntries(Array.from(node.attributes,a=>[a.name,a.value])),computed:Object.fromEntries(fields.map(key=>[key,cs[key]])),rect:{x:r.x,y:r.y,width:r.width,height:r.height},pseudo};};
const actualHost=wrap=>wrap.querySelector('.ix-radio-native');
const control=wrap=>actualHost(wrap)?.querySelector('input[type="radio"]');
const capture=(wrap,visualState)=>{
 const host=actualHost(wrap),node=control(wrap),scope=wrap.closest('[data-radio-scope]'),key=wrap.dataset.key,extra=wrap.dataset.radioRow==='extra';
 if(!(node instanceof HTMLInputElement)||node.type!=='radio'||!scope||!host)throw Error('Missing actual Radio '+wrap.id);
 const wrapper=host.querySelector('.ix-radio-button'),shape=host.querySelector('.ix-check-shape'),label=host.querySelector('.ix-radio-label'),dot=host.querySelector('.ix-radio-dot');
 return{id:wrap.id,scope:scope.dataset.radioScope,scopeIndex:Number(scope.dataset.scopeIndex),revision:Number(scope.dataset.revision),variant:wrap.dataset.variant,booleanKey:wrap.dataset.booleanKey,checked:node.checked,disabled:node.disabled,required:node.required,state:extra?'rest':visualState,kind:extra?'extra':'core',key,host:metrics(host),control:metrics(node),layout:metrics(host),wrapper:metrics(wrapper),shape:metrics(shape),dot:metrics(dot),label:metrics(label),before:metrics(wrapper,'::before'),nativeFacts:{disabledProperty:node.disabled,checkedProperty:node.checked,requiredProperty:node.required,ariaDisabled:node.getAttribute('aria-disabled'),ariaChecked:node.getAttribute('aria-checked'),ariaRequired:node.getAttribute('aria-required'),ariaInvalid:node.getAttribute('aria-invalid'),hostRole:host.getAttribute('role'),hostAriaChecked:host.getAttribute('aria-checked'),hostAriaDisabled:host.getAttribute('aria-disabled'),controlRole:node.getAttribute('role'),controlTabIndex:node.tabIndex,hostTabIndex:host.tabIndex,focused:node===document.activeElement,matchesFocus:node.matches(':focus'),matchesFocusVisible:node.matches(':focus-visible'),matchesHover:host.matches(':hover'),matchesActive:host.matches(':active'),controlHover:node.matches(':hover'),controlActive:node.matches(':active'),labelText:label?.textContent??null,accessibleLabel:node.getAttribute('aria-label'),name:node.name,value:node.value},rowWrapper:metrics(wrap)};
};
const core=()=>Array.from(document.querySelectorAll('[data-radio-row="core"]')),extras=()=>Array.from(document.querySelectorAll('[data-radio-row="extra"]'));
const settle=async()=>{await document.fonts.ready;for(let index=1;index<=2;index++)await new Promise(resolve=>requestAnimationFrame(timestamp=>queueMicrotask(()=>{log('native-frame',{index,timestamp});resolve();})));};
const setRevision=async id=>{const button=document.getElementById('radio-'+id);check(button instanceof HTMLButtonElement,'real Rust revision control '+id);log('public-revision-input',{id});button.click();await wait(()=>document.getElementById('radio-review')?.dataset.phase===id&&document.getElementById('radio-review')?.dataset.state==='rest','Rust revision/state commit '+id);await settle();};
const setState=async value=>{const select=document.getElementById('radio-state');check(select instanceof HTMLSelectElement,'real Rust state selector');document.getElementById('radio-outside').focus();select.value=value;select.dispatchEvent(new Event('change',{bubbles:true,composed:true}));log('public-state-input',{value});await wait(()=>document.getElementById('radio-review')?.dataset.state===value,'Rust state commit '+value);await settle();};
const seedKeyboard=()=>{const node=document.getElementById('radio-outside');node.focus();const event=new KeyboardEvent('keydown',{key:'Tab',bubbles:true,composed:true,cancelable:true});node.dispatchEvent(event);log('native-keyboard-seed',{defaultPrevented:event.defaultPrevented});};
const nativeClick=async node=>{const events=[];let original=null;const capture=event=>{if(event.composedPath()[0]!==node)return;original=event;events.push({site:'document-capture',defaultPrevented:event.defaultPrevented,trusted:event.isTrusted,targetTag:event.target.tagName,originTag:event.composedPath()[0].tagName});},bubble=event=>{if(event.composedPath()[0]!==node)return;events.push({site:'window-bubble',defaultPrevented:event.defaultPrevented,trusted:event.isTrusted,targetTag:event.target.tagName,originTag:event.composedPath()[0].tagName});};document.addEventListener('click',capture,true);window.addEventListener('click',bubble);node.click();document.removeEventListener('click',capture,true);window.removeEventListener('click',bubble);return{eventObserved:!!original,defaultPrevented:original?.defaultPrevented??null,events};};
const controlledState=()=>({...JSON.parse(document.getElementById('radio-controlled-json').value),formData:Array.from(new FormData(document.getElementById('radio-controlled-form')).entries())});
const consumerCommand=async id=>{const node=document.getElementById('radio-'+id),root=document.getElementById('radio-review'),before=Number(root.dataset.consumerCommand);check(node instanceof HTMLButtonElement,'real Rust consumer control '+id);node.click();log('public-consumer-input',{id});await wait(()=>Number(root.dataset.consumerCommand)>before,'Rust consumer command commit '+id);await settle();};
const controlledHistory=async()=>{
 await setRevision('restored');await consumerCommand('reset');const steps=[],mutations=[],form=document.getElementById('radio-controlled-form'),wrap=document.getElementById('radio-extra-controlled'),held=actualHost(wrap),heldDot=held.querySelector('.ix-radio-dot');
 const geometry=()=>Object.fromEntries([['host',held],['wrapper',held.querySelector('.ix-radio-button')],['shape',held.querySelector('.ix-check-shape')],['dot',heldDot],['label',held.querySelector('.ix-radio-label')],['peer',document.getElementById('radio-layout-peer')]].map(([key,node])=>{const r=node?.getBoundingClientRect();return[key,r?{x:r.x,y:r.y,width:r.width,height:r.height}:null];}));
 const observer=new MutationObserver(changes=>mutations.push({sequence:mutations.length+1,time:performance.now(),changes:changes.map(change=>({target:object(change.target),tag:change.target.tagName,attribute:change.attributeName,oldValue:change.oldValue,newValue:change.target.getAttribute(change.attributeName)})),geometry:geometry(),checked:control(wrap).checked,dotVisibility:getComputedStyle(heldDot).visibility}));
 observer.observe(held,{attributes:true,attributeOldValue:true,subtree:true});
 try{
 for(const name of ['click-on','click-again','external-off-click','disabled-click','enabled-click','external-checked','reset']){
  if(['external-off-click','disabled-click','external-checked'].includes(name))await consumerCommand('off');
  if(name==='disabled-click')await consumerCommand('disable');
  if(name==='enabled-click')await consumerCommand('enable');
  await settle();const before=controlledState(),beforeGeometry=geometry(),from=mutations.length;let original=null;
  if(name==='external-checked')await consumerCommand('external-checked');else if(name==='reset')await consumerCommand('reset');else original=await nativeClick(control(wrap));
  await settle();const after=controlledState(),afterGeometry=geometry();steps.push({name,before,after,original,requestsDelta:name==='reset'?[]:after.requests.slice(before.requests.length),row:capture(wrap,'rest'),layout:{before:beforeGeometry,after:afterGeometry,sameHost:held===actualHost(wrap),sameDot:heldDot===actualHost(wrap).querySelector('.ix-radio-dot'),mutations:mutations.slice(from)}});
 }
 return{steps,state:controlledState(),layoutMutations:mutations};
 }finally{observer.disconnect();}
};

try{
 if(source){await wait(()=>window.upstreamRadioTokenReview&&document.getElementById('radio-status')?.dataset.ready==='true','pinned Radio readiness');const result=await window.upstreamRadioTokenReview.run();check(result?.ok===true,'source run completed');check(result.phases.length===9,'source nine phase observations');check(result.phases.reduce((sum,p)=>sum+p.rows.length+p.extras.length,0)===378,'source 378 actual computed rows');return{...result,ok:result.ok&&errors.length===0&&(result.errors??[]).length===0,runnerAssertions:assertions,runnerErrors:errors,renderer:'Linux desktop WebView pinned source'};}
 await wait(()=>core().length===40&&extras().length===6,'Rust 40 core and six supplemental controls committed');
 for(const revisionID of ['baseline','revised','restored']){
  await setRevision(revisionID);
  for(const state of ['rest','disabled','focus']){
   await setState(state);const rows=[];
   if(state==='focus'){seedKeyboard();for(const wrap of core()){const node=control(wrap);log('native-focus-input-before',{id:wrap.id});node.focus();await settle();rows.push(capture(wrap,state));log('native-focus-input-observed',{id:wrap.id,focused:document.activeElement===node,focusVisible:node.matches(':focus-visible')});}}
   else rows.push(...core().map(wrap=>capture(wrap,state)));
   check(rows.length===40,revisionID+':'+state+' 40 actual computed Radios');const supplemental=state==='rest'?extras().map(wrap=>capture(wrap,'rest')):[];if(state==='rest')check(supplemental.length===6,revisionID+' six actual supplemental controls');
   phases.push({id:revisionID+':'+state,revision:Number(document.getElementById('radio-review').dataset.revision),state,rows,extras:supplemental,settleFrames:2,settlement:{publicCommit:'actual Rust phase/state DOM attribute',finalObservationFrames:2,...(state==='focus'?{additionalPerRowNativeFocusFrames:2}:{})}});
  }
 }
 const controlled=await controlledHistory(),fallback=Array.from(document.querySelectorAll('#radio-brand-fallback .ix-radio-native'),metrics);
 window.ixRadioTokenReview={snapshot:()=>({phase:document.getElementById('radio-review').dataset.phase,state:document.getElementById('radio-review').dataset.state,controlled:controlledState()}),pointer:()=>JSON.parse(document.getElementById('radio-pointer-result').textContent||'null')};
 return{ok:errors.length===0,revision:'8bf599bc68365d7752e12832e71b625883b93d7a',fixture:'Developer-authored system-token Radio anatomy adapter; not full Brand metrics',phases,controlled,errors,assertions,journal,fallback,renderer:'Linux desktop WebView Rust',limits:['Source checkedChange is cancellable before source auto-mutation; Rust native controlled accepting consumer has distinct callback timing/cancellation and emits checked then value callbacks','Source focusable role-radio customhost maps to real native input, while whole label receives projected focus outline; raw role/tabindex/ARIA remain structural adapters','Single standalone accepting consumer and FormData do not certify all group observers, reset/validation/touched/ElementInternals or platform/AT semantics','Actual sequential input.focus after synthetic Tab is observed; :focus-visible is measured rather than assumed','Public Rust commit precedes two fixed RAF plus fonts; source additionally settles custom-child readiness, no equal publication chronology claim','Native .click does not certify trusted pointer or active/hover; separate panel records qualified real browser pointer input','Always-mounted dot identity, visibility and passive mutation/rect observations are kept raw; source and native absolute geometry is not normalized','No-token normal/compact fallback stays separate provisional Brand evidence']};

}catch(error){errors.push(String(error));return{ok:false,phases,controlled:null,assertions,journal,errors};}finally{abort.abort();}
"#;
