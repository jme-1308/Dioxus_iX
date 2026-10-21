//! Excluded native Toggle computed-token runner; application intent stays Rust.
use dioxus::prelude::*;
#[cfg(any(
    target_arch = "wasm32",
    all(feature = "desktop", not(target_arch = "wasm32"))
))]
use serde_json::Value;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "toggle_review.rs"]
mod toggle_review;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    let source = std::env::var("IX_TOGGLE_SOURCE").is_ok_and(|value| value == "1");
    let index = if source {
        include_str!("upstream_toggle_token_review.html").replace(
            "</body>",
            "<div id=\"main\" hidden></div><!-- MODULE LOADER --></body>",
        )
    } else {
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Toggle sizing and states</title></head><body style=\"margin:0\"><div id=\"main\"></div><!-- MODULE LOADER --></body></html>".to_string()
    };
    let config = dioxus::desktop::Config::new()
        .with_custom_index(index)
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title(if source {
                    "Pinned iX Toggle"
                } else {
                    "Dioxus iX Toggle"
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
    eprintln!("Run the Toggle review with desktop or web features");
}

#[component]
fn Probe() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let source = std::env::var("IX_TOGGLE_SOURCE").is_ok_and(|value| value == "1");
    #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
    let source = false;
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(180)).await;
            eprintln!("Toggle token native timeout");
            std::process::exit(1);
        });
    });
    rsx! {div {
        onmounted:move |_| {
            #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
            {let window=window.clone();spawn(async move{
                let query=dom_query::eval(VERIFY);let _=query.send(source);
                match query.join::<Value>().await {
                    Ok(value) if value["ok"]==true=>{println!("Toggle token native observations: {value}");window.close();}
                    Ok(value)=>{eprintln!("Toggle token native failure JSON: {value}");std::process::exit(1);}
                    Err(error)=>{eprintln!("Toggle token native transport failure: {error:?}");std::process::exit(1);}
                }
            });}
        },
        {web_controls()}
        if !source {toggle_review::ToggleReview {}}
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
        button {id:"toggle-run",r#type:"button",disabled:busy(),aria_busy:busy(),
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
        span {role:"status",style:"margin-left:12px",if status()=="done" {"Completed: 744 Toggle token/state observations."} else {"{status()}"}}
        pre {id:"toggle-web-result","data-run":status(),"aria-hidden":"true",style:"max-height:180px;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere;font-size:12px","{result()}"}
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
const fields=["display","width","height","minWidth","boxSizing","paddingTop","paddingRight","paddingBottom","paddingLeft","borderTopWidth","borderTopColor","borderTopStyle","borderTopLeftRadius","borderTopRightRadius","borderBottomLeftRadius","borderBottomRightRadius","backgroundColor","color","fontSize","lineHeight","fontWeight","fontFamily","fontStyle","letterSpacing","fontFeatureSettings","outlineColor","outlineWidth","outlineStyle","outlineOffset","opacity","pointerEvents","justifyContent","textAlign","marginLeft","marginRight","maxWidth","minHeight","maxHeight","position","flexShrink","gap","top","right","bottom","left","content","fill","stroke","strokeWidth","cursor","visibility","alignItems","marginTop","marginBottom","borderRightWidth","borderBottomWidth","borderLeftWidth","transform","transitionProperty","transitionDuration","transitionTimingFunction","transitionDelay","flexDirection","wordBreak","whiteSpace","overflow","marginBlockStart","marginBlockEnd","marginInlineStart"];
const metrics=(node,pseudo=null)=>{if(!(node instanceof Element))return null;const cs=getComputedStyle(node,pseudo),r=node.getBoundingClientRect();return{object:object(node),tag:node.tagName,id:node.id,class:node.getAttribute('class'),attrs:Object.fromEntries(Array.from(node.attributes,a=>[a.name,a.value])),computed:Object.fromEntries(fields.map(key=>[key,cs[key]])),rect:{x:r.x,y:r.y,width:r.width,height:r.height},pseudo};};
const outer=wrap=>wrap.querySelector('.ix-toggle-native');
const control=wrap=>outer(wrap)?.querySelector('button.ix-toggle-control');
const capture=(wrap,visualState)=>{
 const host=control(wrap),scope=wrap.closest('[data-toggle-scope]'),key=wrap.dataset.key,extra=wrap.dataset.toggleRow==='extra';
 if(!(host instanceof HTMLButtonElement)||!scope)throw Error('Missing actual Toggle '+wrap.id);
 const wrapper=host.querySelector('.ix-toggle-wrapper'),switchNode=host.querySelector('.ix-toggle-switch'),slider=host.querySelector('.ix-toggle-slider'),label=host.querySelector('.ix-toggle-label'),input=outer(wrap).querySelector('input');
 return{id:wrap.id,scope:scope.dataset.toggleScope,scopeIndex:Number(scope.dataset.scopeIndex),revision:Number(scope.dataset.revision),variant:wrap.dataset.variant,booleanKey:wrap.dataset.booleanKey,checked:host.dataset.checked==='true',indeterminate:host.dataset.mixed==='true',disabled:host.disabled,required:host.getAttribute('aria-required')==='true',hideText:!label,state:extra?'rest':visualState,kind:extra?'extra':'core',key,host:metrics(host),control:metrics(host),outer:metrics(outer(wrap)),wrapper:metrics(wrapper),switch:metrics(switchNode),slider:metrics(slider),thumb:metrics(slider,'::before'),label:metrics(label),labelText:label?.textContent??null,input:metrics(input),nativeFacts:{disabledProperty:host.disabled,ariaDisabled:host.getAttribute('aria-disabled'),ariaChecked:host.getAttribute('aria-checked'),ariaRequired:host.getAttribute('aria-required'),ariaInvalid:host.getAttribute('aria-invalid'),hostRole:host.getAttribute('role'),controlRole:host.getAttribute('role'),controlTabIndex:host.tabIndex,hostTabIndex:host.tabIndex,focused:host===document.activeElement,matchesFocus:host.matches(':focus'),matchesFocusVisible:host.matches(':focus-visible'),matchesHover:host.matches(':hover'),matchesActive:host.matches(':active'),switchHover:switchNode.matches(':hover'),switchActive:switchNode.matches(':active'),labelHover:label?.matches(':hover')??false,checkedProperty:input?.checked,indeterminateProperty:input?.indeterminate,labelText:label?.textContent??null,accessibleLabel:host.getAttribute('aria-label'),ariaLabelledby:host.getAttribute('aria-labelledby')},rowWrapper:metrics(wrap)};
};
const core=()=>Array.from(document.querySelectorAll('[data-toggle-row="core"]')),extras=()=>Array.from(document.querySelectorAll('[data-toggle-row="extra"]'));
const settle=async()=>{await document.fonts.ready;for(let index=1;index<=2;index++)await new Promise(resolve=>requestAnimationFrame(timestamp=>queueMicrotask(()=>{log('native-frame',{index,timestamp});resolve();})));};
const setRevision=async id=>{const button=document.getElementById('toggle-'+id);check(button instanceof HTMLButtonElement,'real Rust revision control '+id);log('public-revision-input',{id});button.click();await wait(()=>document.getElementById('toggle-review')?.dataset.phase===id&&document.getElementById('toggle-review')?.dataset.state==='rest','Rust revision/state commit '+id);await settle();};
const setState=async value=>{const select=document.getElementById('toggle-state');check(select instanceof HTMLSelectElement,'real Rust state selector');document.getElementById('toggle-outside').focus();select.value=value;select.dispatchEvent(new Event('change',{bubbles:true,composed:true}));log('public-state-input',{value});await wait(()=>document.getElementById('toggle-review')?.dataset.state===value,'Rust state commit '+value);await settle();};
const seedKeyboard=()=>{const node=document.getElementById('toggle-outside');node.focus();const event=new KeyboardEvent('keydown',{key:'Tab',bubbles:true,composed:true,cancelable:true});node.dispatchEvent(event);log('native-keyboard-seed',{defaultPrevented:event.defaultPrevented});};
const nativeClick=async node=>{const events=[];let original=null;const capture=event=>{if(event.composedPath()[0]!==node)return;original=event;events.push({site:'document-capture',defaultPrevented:event.defaultPrevented,trusted:event.isTrusted,targetTag:event.target.tagName,originTag:event.composedPath()[0].tagName});},bubble=event=>{if(event.composedPath()[0]!==node)return;events.push({site:'window-bubble',defaultPrevented:event.defaultPrevented,trusted:event.isTrusted,targetTag:event.target.tagName,originTag:event.composedPath()[0].tagName});};document.addEventListener('click',capture,true);window.addEventListener('click',bubble);node.click();document.removeEventListener('click',capture,true);window.removeEventListener('click',bubble);return{eventObserved:!!original,defaultPrevented:original?.defaultPrevented??null,events};};
const controlledState=()=>({...JSON.parse(document.getElementById('toggle-controlled-json').value),formData:Array.from(new FormData(document.getElementById('toggle-controlled-form')).entries())});
const consumerCommand=async(id,checked)=>{const node=document.getElementById('toggle-'+id),root=document.getElementById('toggle-review'),before=Number(root.dataset.consumerCommand);check(node instanceof HTMLElement,'real Rust consumer control '+id);if(typeof checked==='boolean'){check(node instanceof HTMLInputElement,'real native consumer checkbox '+id);node.checked=checked;node.dispatchEvent(new Event('change',{bubbles:true,composed:true}));}else node.click();log('public-consumer-input',{id});await wait(()=>Number(root.dataset.consumerCommand)>before,'Rust consumer command commit '+id);await settle();};
const controlledHistory=async()=>{
 await setRevision('restored');await consumerCommand('reset');const steps=[],submitEvents=[];const form=document.getElementById('toggle-controlled-form');form.addEventListener('submit',event=>{submitEvents.push({defaultPrevented:event.defaultPrevented,targetTag:event.target.tagName});},{signal:abort.signal});
 for(const name of ['click-on','click-off','mixed-click','disabled-click','enabled-click','external-checked','reset']){
  if(name==='mixed-click')await consumerCommand('mixed');
  if(name==='disabled-click')await consumerCommand('disable');
  if(name==='enabled-click')await consumerCommand('enable');
  await settle();const before=controlledState();let original=null;
  if(name==='external-checked')await consumerCommand('external-checked');else if(name==='reset')await consumerCommand('reset');else original=await nativeClick(control(document.getElementById('toggle-extra-controlled')));
  await settle();const after=controlledState();steps.push({name,before,after,original,requestsDelta:name==='reset'?[]:after.requests.slice(before.requests.length),row:capture(document.getElementById('toggle-extra-controlled'),'rest')});
 }
 return{steps,state:controlledState(),submitEvents};
};
try{
 if(source){await wait(()=>window.upstreamToggleTokenReview&&document.getElementById('toggle-status')?.dataset.ready==='true','pinned Toggle readiness');const result=await window.upstreamToggleTokenReview.run();check(result?.ok===true,'source run completed');check(result.phases.length===9,'source nine phase observations');check(result.phases.reduce((sum,p)=>sum+p.rows.length+p.extras.length,0)===744,'source 744 actual computed rows');return{...result,ok:result.ok&&errors.length===0&&(result.errors??[]).length===0,runnerAssertions:assertions,runnerErrors:errors,renderer:'Linux desktop WebView pinned source'};}
 await wait(()=>core().length===80&&extras().length===8,'Rust 80 core and eight supplemental controls committed');
 for(const revisionID of ['baseline','revised','restored']){
  await setRevision(revisionID);
  for(const state of ['rest','disabled','focus']){
   await setState(state);const rows=[];
   if(state==='focus'){seedKeyboard();for(const wrap of core()){const node=control(wrap);log('native-focus-input-before',{id:wrap.id});node.focus();await settle();rows.push(capture(wrap,state));log('native-focus-input-observed',{id:wrap.id,focused:document.activeElement===node,focusVisible:node.matches(':focus-visible')});}}
   else rows.push(...core().map(wrap=>capture(wrap,state)));
   check(rows.length===80,revisionID+':'+state+' 80 actual computed Toggles');const supplemental=state==='rest'?extras().map(wrap=>capture(wrap,'rest')):[];if(state==='rest')check(supplemental.length===8,revisionID+' eight actual supplemental controls');
   phases.push({id:revisionID+':'+state,revision:Number(document.getElementById('toggle-review').dataset.revision),state,rows,extras:supplemental,settleFrames:2,settlement:{publicCommit:'actual Rust phase/state DOM attribute',finalObservationFrames:2,...(state==='focus'?{additionalPerRowNativeFocusFrames:2}:{})}});
  }
 }
 const defaultWrap=document.getElementById('toggle-extra-default'),directBefore=capture(defaultWrap,'rest'),directOriginal=await nativeClick(control(defaultWrap));await settle();const noHandler={before:directBefore,original:directOriginal,after:capture(defaultWrap,'rest'),adapter:'Source auto-mutates; Rust no-handler controlled props do not mutate, explicit unpaired API diagnostic'};
 const controlled=await controlledHistory(),fallback=Array.from(document.querySelectorAll('#toggle-brand-fallback .ix-toggle-control'),metrics);
 window.ixToggleTokenReview={snapshot:()=>({phase:document.getElementById('toggle-review').dataset.phase,state:document.getElementById('toggle-review').dataset.state,controlled:controlledState()}),pointer:()=>JSON.parse(document.getElementById('toggle-pointer-result').textContent||'null')};
 return{ok:errors.length===0,revision:'8bf599bc68365d7752e12832e71b625883b93d7a',fixture:'Developer-authored system-token Toggle anatomy adapter; not full Brand metrics',phases,controlled,noHandler,errors,assertions,journal,fallback,renderer:'Linux desktop WebView Rust',limits:['Source auto-mutates checked and clears mixed before emitting, then restores on cancellation; Rust accepting consumer updates checked and clears mixed in its callback, without source cancellation equivalence','No-handler actual .click diagnostic is explicitly unpaired source automatic state versus Rust controlled props','Source role-switch focusable customhost maps to native button typebutton and hidden form mirror; raw ARIA/required/name chronology is retained','Actual sequential .focus after synthetic Tab is observed, :focus-visible measured rather than assumed','Public Rust commit precedes fonts and two fixed RAF; source additional child readiness and IPC/publication timing differ','Authored 0s/1ms durations verify transition recipe at fixed frames, not full motion/interruption/reduced-motion parity','Pointer paint requires actual switch hover/active, not label/host alone; qualified browser pointer captures remain separate from synthetic .click','Standalone accepting consumer FormData does not certify touched/reset/validation/ElementInternals/group or platform/AT behavior','No-token normal/compact fallback remains separate provisional Brand evidence']};

}catch(error){errors.push(String(error));return{ok:false,phases,controlled:null,assertions,journal,errors};}finally{abort.abort();}
"#;
