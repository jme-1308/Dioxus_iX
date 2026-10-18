//! Excluded native Checkbox computed-token runner; application intent stays Rust.
use dioxus::prelude::*;
#[cfg(any(
    target_arch = "wasm32",
    all(feature = "desktop", not(target_arch = "wasm32"))
))]
use serde_json::Value;
#[path = "../examples/gallery/checkbox_review.rs"]
mod checkbox_review;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    let source = std::env::var("IX_CHECKBOX_SOURCE").is_ok_and(|value| value == "1");
    let index = if source {
        include_str!("upstream_checkbox_token_review.html").replace(
            "</body>",
            "<div id=\"main\" hidden></div><!-- MODULE LOADER --></body>",
        )
    } else {
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Checkbox sizing and states</title></head><body style=\"margin:0\"><div id=\"main\"></div><!-- MODULE LOADER --></body></html>".to_string()
    };
    let config = dioxus::desktop::Config::new()
        .with_custom_index(index)
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title(if source {
                    "Pinned iX Checkbox"
                } else {
                    "Dioxus iX Checkbox"
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
    eprintln!("Run the Checkbox review with desktop or web features");
}

#[component]
fn Probe() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let source = std::env::var("IX_CHECKBOX_SOURCE").is_ok_and(|value| value == "1");
    #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
    let source = false;
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(180)).await;
            eprintln!("Checkbox token native timeout");
            std::process::exit(1);
        });
    });
    rsx! {div {
        onmounted:move |_| {
            #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
            {let window=window.clone();spawn(async move{
                let query=dom_query::eval(VERIFY);let _=query.send(source);
                match query.join::<Value>().await {
                    Ok(value) if value["ok"]==true=>{println!("Checkbox token native observations: {value}");window.close();}
                    Ok(value)=>{eprintln!("Checkbox token native failure JSON: {value}");std::process::exit(1);}
                    Err(error)=>{eprintln!("Checkbox token native transport failure: {error:?}");std::process::exit(1);}
                }
            });}
        },
        {web_controls()}
        if !source {checkbox_review::CheckboxReview {}}
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
        button {id:"checkbox-run",r#type:"button",disabled:busy(),aria_busy:busy(),
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
        span {role:"status",style:"margin-left:12px",if status()=="done" {"Completed: 738 Checkbox token/state observations."} else {"{status()}"}}
        pre {id:"checkbox-web-result","data-run":status(),"aria-hidden":"true",style:"max-height:180px;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere;font-size:12px","{result()}"}
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
const actualHost=wrap=>wrap.querySelector('.ix-checkbox-native');
const control=wrap=>actualHost(wrap)?.querySelector('button.ix-checkbox');
const capture=(wrap,visualState)=>{
 const host=actualHost(wrap),node=control(wrap),scope=wrap.closest('[data-checkbox-scope]'),key=wrap.dataset.key,extra=wrap.dataset.checkboxRow==='extra';
 if(!(node instanceof HTMLButtonElement)||!scope||!host)throw Error('Missing actual Checkbox '+wrap.id);
 const wrapper=host.querySelector('.ix-checkbox-button'),shape=host.querySelector('.ix-check-shape'),label=host.querySelector('.ix-checkbox-label'),svg=shape?.querySelector('svg'),input=host.querySelector('input');
 return{id:wrap.id,scope:scope.dataset.checkboxScope,scopeIndex:Number(scope.dataset.scopeIndex),revision:Number(scope.dataset.revision),variant:wrap.dataset.variant,booleanKey:wrap.dataset.booleanKey,checked:host.dataset.checked==='true',indeterminate:host.dataset.mixed==='true',disabled:node.disabled,required:node.getAttribute('aria-required')==='true',state:extra?'rest':visualState,kind:extra?'extra':'core',key,host:metrics(host),control:metrics(node),layout:metrics(node),wrapper:metrics(wrapper),shape:metrics(shape),label:metrics(label),svg:metrics(svg),primitives:Array.from(svg?.children??[],node=>metrics(node)),before:metrics(wrapper,'::before'),input:metrics(input),nativeFacts:{disabledProperty:node.disabled,ariaDisabled:node.getAttribute('aria-disabled'),ariaChecked:node.getAttribute('aria-checked'),ariaRequired:node.getAttribute('aria-required'),ariaInvalid:node.getAttribute('aria-invalid'),hostRole:host.getAttribute('role'),hostAriaChecked:host.getAttribute('aria-checked'),hostAriaDisabled:host.getAttribute('aria-disabled'),controlRole:node.getAttribute('role'),controlTabIndex:node.tabIndex,hostTabIndex:host.tabIndex,focused:node===document.activeElement,matchesFocus:node.matches(':focus'),matchesFocusVisible:node.matches(':focus-visible'),matchesHover:host.matches(':hover'),matchesActive:host.matches(':active'),controlHover:node.matches(':hover'),controlActive:node.matches(':active'),checkedProperty:input?.checked,indeterminateProperty:input?.indeterminate,labelText:label?.textContent,accessibleLabel:node.getAttribute('aria-label')},rowWrapper:metrics(wrap)};
};
const core=()=>Array.from(document.querySelectorAll('[data-checkbox-row="core"]')),extras=()=>Array.from(document.querySelectorAll('[data-checkbox-row="extra"]'));
const settle=async()=>{await document.fonts.ready;for(let index=1;index<=2;index++)await new Promise(resolve=>requestAnimationFrame(timestamp=>queueMicrotask(()=>{log('native-frame',{index,timestamp});resolve();})));};
const setRevision=async id=>{const button=document.getElementById('checkbox-'+id);check(button instanceof HTMLButtonElement,'real Rust revision control '+id);log('public-revision-input',{id});button.click();await wait(()=>document.getElementById('checkbox-review')?.dataset.phase===id&&document.getElementById('checkbox-review')?.dataset.state==='rest','Rust revision/state commit '+id);await settle();};
const setState=async value=>{const select=document.getElementById('checkbox-state');check(select instanceof HTMLSelectElement,'real Rust state selector');document.getElementById('checkbox-outside').focus();select.value=value;select.dispatchEvent(new Event('change',{bubbles:true,composed:true}));log('public-state-input',{value});await wait(()=>document.getElementById('checkbox-review')?.dataset.state===value,'Rust state commit '+value);await settle();};
const seedKeyboard=()=>{const node=document.getElementById('checkbox-outside');node.focus();const event=new KeyboardEvent('keydown',{key:'Tab',bubbles:true,composed:true,cancelable:true});node.dispatchEvent(event);log('native-keyboard-seed',{defaultPrevented:event.defaultPrevented});};
const nativeClick=async node=>{const events=[];let original=null;const capture=event=>{if(event.composedPath()[0]!==node)return;original=event;events.push({site:'document-capture',defaultPrevented:event.defaultPrevented,trusted:event.isTrusted,targetTag:event.target.tagName,originTag:event.composedPath()[0].tagName});},bubble=event=>{if(event.composedPath()[0]!==node)return;events.push({site:'window-bubble',defaultPrevented:event.defaultPrevented,trusted:event.isTrusted,targetTag:event.target.tagName,originTag:event.composedPath()[0].tagName});};document.addEventListener('click',capture,true);window.addEventListener('click',bubble);node.click();document.removeEventListener('click',capture,true);window.removeEventListener('click',bubble);return{eventObserved:!!original,defaultPrevented:original?.defaultPrevented??null,events};};
const controlledState=()=>({...JSON.parse(document.getElementById('checkbox-controlled-json').value),formData:Array.from(new FormData(document.getElementById('checkbox-controlled-form')).entries())});
const consumerCommand=async id=>{const node=document.getElementById('checkbox-'+id),root=document.getElementById('checkbox-review'),before=Number(root.dataset.consumerCommand);check(node instanceof HTMLButtonElement,'real Rust consumer control '+id);node.click();log('public-consumer-input',{id});await wait(()=>Number(root.dataset.consumerCommand)>before,'Rust consumer command commit '+id);await settle();};
const controlledHistory=async()=>{
 await setRevision('restored');await consumerCommand('reset');const steps=[],submitEvents=[];const form=document.getElementById('checkbox-controlled-form');form.addEventListener('submit',event=>{submitEvents.push({defaultPrevented:event.defaultPrevented,targetTag:event.target.tagName});},{signal:abort.signal});
 for(const name of ['click-on','click-off','mixed-click','disabled-click','enabled-click','external-checked','reset']){
  if(name==='mixed-click')await consumerCommand('mixed');
  if(name==='disabled-click')await consumerCommand('disable');
  if(name==='enabled-click')await consumerCommand('enable');
  await settle();const before=controlledState();let original=null;
  if(name==='external-checked')await consumerCommand('external-checked');else if(name==='reset')await consumerCommand('reset');else original=await nativeClick(control(document.getElementById('checkbox-extra-controlled')));
  await settle();const after=controlledState();steps.push({name,before,after,original,requestsDelta:name==='reset'?[]:after.requests.slice(before.requests.length),row:capture(document.getElementById('checkbox-extra-controlled'),'rest')});
 }
 return{steps,state:controlledState(),submitEvents};
};
try{
 if(source){await wait(()=>window.upstreamCheckboxTokenReview&&document.getElementById('checkbox-status')?.dataset.ready==='true','pinned Checkbox readiness');const result=await window.upstreamCheckboxTokenReview.run();check(result?.ok===true,'source run completed');check(result.phases.length===9,'source nine phase observations');check(result.phases.reduce((sum,p)=>sum+p.rows.length+p.extras.length,0)===738,'source 738 actual computed rows');return{...result,ok:result.ok&&errors.length===0&&(result.errors??[]).length===0,runnerAssertions:assertions,runnerErrors:errors,renderer:'Linux desktop WebView pinned source'};}
 await wait(()=>core().length===80&&extras().length===6,'Rust 80 core and six supplemental controls committed');
 for(const revisionID of ['baseline','revised','restored']){
  await setRevision(revisionID);
  for(const state of ['rest','disabled','focus']){
   await setState(state);const rows=[];
   if(state==='focus'){seedKeyboard();for(const wrap of core()){const node=control(wrap);log('native-focus-input-before',{id:wrap.id});node.focus();await settle();rows.push(capture(wrap,state));log('native-focus-input-observed',{id:wrap.id,focused:document.activeElement===node,focusVisible:node.matches(':focus-visible')});}}
   else rows.push(...core().map(wrap=>capture(wrap,state)));
   check(rows.length===80,revisionID+':'+state+' 80 actual computed Checkboxes');const supplemental=state==='rest'?extras().map(wrap=>capture(wrap,'rest')):[];if(state==='rest')check(supplemental.length===6,revisionID+' six actual supplemental controls');
   phases.push({id:revisionID+':'+state,revision:Number(document.getElementById('checkbox-review').dataset.revision),state,rows,extras:supplemental,settleFrames:2,settlement:{publicCommit:'actual Rust phase/state DOM attribute',finalObservationFrames:2,...(state==='focus'?{additionalPerRowNativeFocusFrames:2}:{})}});
  }
 }
 const controlled=await controlledHistory(),fallback=Array.from(document.querySelectorAll('#checkbox-brand-fallback .ix-checkbox-native'),metrics);
 window.ixCheckboxTokenReview={snapshot:()=>({phase:document.getElementById('checkbox-review').dataset.phase,state:document.getElementById('checkbox-review').dataset.state,controlled:controlledState()}),pointer:()=>JSON.parse(document.getElementById('checkbox-pointer-result').textContent||'null')};
 return{ok:errors.length===0,revision:'8bf599bc68365d7752e12832e71b625883b93d7a',fixture:'Developer-authored system-token Checkbox anatomy adapter; not full Brand metrics',phases,controlled,errors,assertions,journal,fallback,renderer:'Linux desktop WebView Rust',limits:['Source checked is auto-mutated before checkedChange; Rust accepting consumer owns checked; raw callback entry timing is not normalized','Source bool host ARIA versus Rust native mixed ARIA and hidden form mirror remain explicit adapters','Actual source default-submit button and Rust type=button differ; submit diagnostics do not certify full forms/reset/validation/AT','Focus is actual sequential .focus after synthetic Tab; observed focus is not assumed :focus-visible','Public Rust commit precedes two fixed RAF plus fonts; no identical source child readiness/IPC chronology claim','Native .click histories do not certify trusted pointer or active/hover; separate pointer panel requires actually qualified browser input','Sourcehost and shadow controls map to native root/shape/wrapper/label while root/native button label geometry remains an adapter','No-token normal/compact fallback is separate provisional Brand evidence']};
}catch(error){errors.push(String(error));return{ok:false,phases,controlled:null,assertions,journal,errors};}finally{abort.abort();}
"#;
