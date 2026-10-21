//! Excluded native ToggleButton computed-token runner; application intent stays Rust.
use dioxus::prelude::*;
#[cfg(any(
    target_arch = "wasm32",
    all(feature = "desktop", not(target_arch = "wasm32"))
))]
use serde_json::Value;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "toggle_button_review.rs"]
mod toggle_button_review;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    let source = std::env::var("IX_TOGGLE_BUTTON_SOURCE").is_ok_and(|value| value == "1");
    let index = if source {
        include_str!("upstream_toggle_button_token_review.html").replace(
            "</body>",
            "<div id=\"main\" hidden></div><!-- MODULE LOADER --></body>",
        )
    } else {
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>ToggleButton sizing and states</title></head><body style=\"margin:0\"><div id=\"main\"></div><!-- MODULE LOADER --></body></html>".to_string()
    };
    let config = dioxus::desktop::Config::new()
        .with_custom_index(index)
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title(if source {
                    "Pinned iX ToggleButton"
                } else {
                    "Dioxus iX ToggleButton"
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
    eprintln!("Run the ToggleButton review with desktop or web features");
}

#[component]
fn Probe() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let source = std::env::var("IX_TOGGLE_BUTTON_SOURCE").is_ok_and(|value| value == "1");
    #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
    let source = false;
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(150)).await;
            eprintln!("ToggleButton token native timeout");
            std::process::exit(1);
        });
    });
    rsx! {div {
        onmounted:move |_| {
            #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
            {let window=window.clone();spawn(async move{
                let query=dom_query::eval(VERIFY);let _=query.send(source);
                match query.join::<Value>().await {
                    Ok(value) if value["ok"]==true=>{println!("ToggleButton token native observations: {value}");window.close();}
                    Ok(value)=>{eprintln!("ToggleButton token native failure JSON: {value}");std::process::exit(1);}
                    Err(error)=>{eprintln!("ToggleButton token native transport failure: {error:?}");std::process::exit(1);}
                }
            });}
        },
        {web_controls()}
        if !source {toggle_button_review::ToggleButtonReview {}}
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
        span {role:"status",style:"margin-left:12px",if status()=="done" {"Completed: 894 ToggleButton token/state observations."} else {"{status()}"}}
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
const fields=['display','width','height','minWidth','boxSizing','paddingTop','paddingRight','paddingBottom','paddingLeft','borderTopWidth','borderTopColor','borderTopStyle','borderTopLeftRadius','borderTopRightRadius','borderBottomLeftRadius','borderBottomRightRadius','backgroundColor','color','fontSize','lineHeight','fontWeight','fontFamily','fontStyle','letterSpacing','fontFeatureSettings','outlineColor','outlineWidth','outlineStyle','outlineOffset','opacity','pointerEvents','justifyContent','textAlign','marginLeft','marginRight','minHeight'];
const metrics=node=>{if(!(node instanceof Element))return null;const cs=getComputedStyle(node),r=node.getBoundingClientRect();return{object:object(node),tag:node.tagName,id:node.id,class:node.getAttribute('class'),attrs:Object.fromEntries(Array.from(node.attributes,a=>[a.name,a.value])),computed:Object.fromEntries(fields.map(key=>[key,cs[key]])),rect:{x:r.x,y:r.y,width:r.width,height:r.height},filter:cs.filter};};
const control=wrap=>wrap.querySelector('.ix-toggle-button-native');
const capture=(wrap,visualState)=>{
 const node=control(wrap),scope=wrap.closest('[data-toggle-scope]'),key=wrap.dataset.key,extra=wrap.dataset.toggleRow==='extra';
 if(!(node instanceof HTMLElement)||!scope)throw Error('Missing actual ToggleButton '+wrap.id);
 const state=extra?(key==='host-active-selected'?'host-active':key==='loading-icons'?'loading':'rest'):visualState;
 const leading=node.querySelector(':scope > .ix-button-icon-start'),trailing=node.querySelector(':scope > .ix-button-icon-end'),loadingWrapper=node.querySelector(':scope > .ix-button-loading'),spinner=loadingWrapper?.querySelector('.ix-spinner');
 return{id:wrap.id,scope:scope.dataset.toggleScope,scopeIndex:Number(scope.dataset.scopeIndex),revision:Number(scope.dataset.revision),variant:wrap.dataset.variant,pressed:wrap.dataset.pressed==='true',state,kind:extra?'extra':'core',key,host:metrics(node),control:metrics(node),icons:[[leading,'start'],[trailing,'end']].filter(([part])=>part).map(([part,side])=>({side,size:'20',node:metrics(part),svg:metrics(part.querySelector('.ix-toggle-button-glyph'))})),spinner:spinner?[{host:metrics(loadingWrapper),size:'sm',inner:metrics(spinner.querySelector('svg')),actualSpan:metrics(spinner),hideTrack:spinner.classList.contains('ix-spinner-no-track')}]:[],content:metrics(node.querySelector('.ix-button-content')),nativeFacts:{disabledProperty:node.disabled,ariaDisabled:node.getAttribute('aria-disabled'),ariaPressed:node.getAttribute('aria-pressed'),hostTabIndex:node.tabIndex,controlTabIndex:node.tabIndex,focused:node===document.activeElement,matchesFocus:node.matches(':focus'),matchesFocusVisible:node.matches(':focus-visible'),matchesHover:node.matches(':hover'),matchesActive:node.matches(':active'),hostActive:node.classList.contains('active'),hostFocused:node.classList.contains('ix-focused'),loading:wrap.dataset.loading==='true',disabled:wrap.dataset.disabled==='true',href:node.getAttribute('href'),target:node.getAttribute('target'),rel:node.getAttribute('rel')},wrapper:{object:object(wrap),attrs:Object.fromEntries(Array.from(wrap.attributes,a=>[a.name,a.value]))}};
};
const core=()=>Array.from(document.querySelectorAll('[data-toggle-row="core"]')),extras=()=>Array.from(document.querySelectorAll('[data-toggle-row="extra"]'));
const settle=async()=>{await document.fonts.ready;for(let index=1;index<=2;index++)await new Promise(resolve=>requestAnimationFrame(timestamp=>queueMicrotask(()=>{log('native-frame',{index,timestamp});resolve();})));};
const setRevision=async id=>{const button=document.getElementById('toggle-'+id);check(button instanceof HTMLButtonElement,'real Rust revision control '+id);log('public-revision-input',{id});button.click();await wait(()=>document.getElementById('toggle-review')?.dataset.phase===id&&document.getElementById('toggle-review')?.dataset.state==='rest','Rust revision/state commit '+id);await settle();};
const setState=async value=>{const select=document.getElementById('toggle-state');check(select instanceof HTMLSelectElement,'real Rust state selector');document.getElementById('toggle-outside').focus();select.value=value;select.dispatchEvent(new Event('change',{bubbles:true,composed:true}));log('public-state-input',{value});await wait(()=>document.getElementById('toggle-review')?.dataset.state===value,'Rust state commit '+value);await settle();};
const seedKeyboard=()=>{const node=document.getElementById('toggle-outside');node.focus();const event=new KeyboardEvent('keydown',{key:'Tab',bubbles:true,composed:true,cancelable:true});node.dispatchEvent(event);log('native-keyboard-seed',{defaultPrevented:event.defaultPrevented});};
const nativeClick=async node=>{const events=[];let original=null;const capture=event=>{if(event.composedPath()[0]!==node)return;original=event;events.push({site:'document-capture',defaultPrevented:event.defaultPrevented,trusted:event.isTrusted,targetTag:event.target.tagName,originTag:event.composedPath()[0].tagName});},bubble=event=>{if(event.composedPath()[0]!==node)return;events.push({site:'window-bubble',defaultPrevented:event.defaultPrevented,trusted:event.isTrusted,targetTag:event.target.tagName,originTag:event.composedPath()[0].tagName});};document.addEventListener('click',capture,true);window.addEventListener('click',bubble);node.click();document.removeEventListener('click',capture,true);window.removeEventListener('click',bubble);return{eventObserved:!!original,defaultPrevented:original?.defaultPrevented??null,events};};
const controlledState=()=>JSON.parse(document.getElementById('toggle-controlled-json').value);
const consumerCommand=async(id,checked)=>{
 const node=document.getElementById('toggle-'+id),root=document.getElementById('toggle-review'),before=Number(root.dataset.consumerCommand);
 check(node instanceof HTMLElement,'real Rust consumer control '+id);
 if(typeof checked==='boolean'){check(node instanceof HTMLInputElement,'native consumer checkbox '+id);node.checked=checked;node.dispatchEvent(new Event('change',{bubbles:true,composed:true}));}else node.click();
 log('public-consumer-input',{id,checked:checked??null});
 await wait(()=>Number(root.dataset.consumerCommand)>before,'Rust consumer command commit '+id);await settle();
};
const controlledHistory=async()=>{
 await setRevision('restored');await consumerCommand('reset');const steps=[];
 for(const name of ['reject-click-1','reject-click-2','accept-click-true','reject-click-false','accept-click-false','external-true-click-false','disabled-click','loading-click','restore']){
   if(name.startsWith('reject-'))await consumerCommand('reject');
   if(name.startsWith('accept-'))await consumerCommand('accept');
   if(name==='external-true-click-false')await consumerCommand('external-true');
   if(name==='disabled-click'||name==='loading-click')for(const key of ['disabled','loading'])await consumerCommand('controlled-'+key,name===key+'-click');
   await settle();const before=controlledState();let original=null;
   if(name==='restore')await consumerCommand('reset');else original=await nativeClick(control(document.getElementById('toggle-extra-controlled')));
   await settle();const after=controlledState();
   steps.push({name,before,after,original,requestsDelta:name==='restore'?[]:after.requests.slice(before.requests.length),row:capture(document.getElementById('toggle-extra-controlled'),'rest')});
 }
 return{steps,state:controlledState()};
};
try {
 if(source){
   await wait(()=>window.upstreamToggleButtonTokenReview&&document.getElementById('toggle-status')?.dataset.ready==='true','pinned ToggleButton readiness');
   const result=await window.upstreamToggleButtonTokenReview.run();check(result?.ok===true,'source run completed');check(result.phases.length===18,'source eighteen phase observations');
   check(result.phases.reduce((sum,p)=>sum+p.rows.length+p.extras.length,0)===894,'source 894 actual computed rows');
   return{...result,ok:result.ok&&errors.length===0&&(result.errors??[]).length===0,runnerAssertions:assertions,runnerErrors:errors,renderer:'Linux desktop WebView pinned source'};
 }
 await wait(()=>core().length===48&&extras().length===10,'Rust 48 core and 10 supplemental controls committed');
 for(const revisionID of ['baseline','revised','restored']){
   await setRevision(revisionID);
   for(const state of ['rest','hover','active','disabled','loading','focus']){
      await setState(state);const rows=[];
      if(state==='focus'){seedKeyboard();for(const wrap of core()){const node=control(wrap);log('native-focus-input-before',{id:wrap.id});node.focus();await settle();rows.push(capture(wrap,state));log('native-focus-input-observed',{id:wrap.id,focused:document.activeElement===node,focusVisible:node.matches(':focus-visible')});}}
      else rows.push(...core().map(wrap=>capture(wrap,state)));
      check(rows.length===48,revisionID+':'+state+' 48 actual computed ToggleButtons');
      const supplemental=state==='rest'?extras().map(wrap=>capture(wrap,'rest')):[];
      if(state==='rest')check(supplemental.length===10,revisionID+' 10 actual supplemental controls');
      phases.push({id:revisionID+':'+state,revision:Number(document.getElementById('toggle-review').dataset.revision),state,rows,extras:supplemental,settleFrames:2,settlement:{publicCommit:'actual Rust phase/state DOM attribute',finalObservationFrames:2,...(state==='focus'?{additionalPerRowNativeFocusFrames:2}:{})}});
   }
 }
 const controlled=await controlledHistory();
 const fallback=Array.from(document.querySelectorAll('#toggle-brand-fallback .ix-toggle-button-native'),metrics);
 return{ok:errors.length===0,revision:'8bf599bc68365d7752e12832e71b625883b93d7a',fixture:'Developer-authored system-token ToggleButton adapter, not full Brand metrics',phases,controlled,errors,assertions,journal,fallback,renderer:'Linux desktop WebView Rust',limits:['Core state aliases come from Rust public class/disabled/loading props; selected values remain consumer-controlled','Rust active denotes source host active; ix-toggle-active denotes the separate shipped source inner active alias','Focus uses actual sequential .focus after a synthetic Tab input; observed focus does not imply :focus-visible','Observed public Rust phase/state or consumer-command commit precedes two fixed native RAF plus font readiness; callbacks are observed after fixed frames without waiting for expected requests','Source final two frames additionally follow parent frames/current custom-child readiness; no identical total IPC/publication chronology claim','Source host and shadow icon map to flattened native root and glyph wrapper; actual SVG and Spinner span facts remain separate','Actual native .click and Rust callback values do not prove trusted input, form behavior, fallback naming or all source events','No-token normal/compact previews remain separate provisional Brand diagnostics']};
}catch(error){errors.push(String(error));return{ok:false,phases,assertions,controlled:null,journal,errors};}finally{abort.abort();}
"#;
