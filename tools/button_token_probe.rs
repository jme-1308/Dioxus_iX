//! Excluded native Button computed-token runner; application intent stays Rust.
use dioxus::prelude::*;
#[cfg(any(
    target_arch = "wasm32",
    all(feature = "desktop", not(target_arch = "wasm32"))
))]
use serde_json::Value;
#[path = "../examples/gallery/button_review.rs"]
mod button_review;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    let source = std::env::var("IX_BUTTON_SOURCE").is_ok_and(|value| value == "1");
    let index = if source {
        include_str!("upstream_button_token_review.html").replace(
            "</body>",
            "<div id=\"main\" hidden></div><!-- MODULE LOADER --></body>",
        )
    } else {
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Button sizing and states</title></head><body style=\"margin:0\"><div id=\"main\"></div><!-- MODULE LOADER --></body></html>".to_string()
    };
    let config = dioxus::desktop::Config::new()
        .with_custom_index(index)
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title(if source {
                    "Pinned iX Button"
                } else {
                    "Dioxus iX Button"
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
    eprintln!("Run the Button review with desktop or web features");
}

#[component]
fn Probe() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let source = std::env::var("IX_BUTTON_SOURCE").is_ok_and(|value| value == "1");
    #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
    let source = false;
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(90)).await;
            eprintln!("Button token native timeout");
            std::process::exit(1);
        });
    });
    rsx! {div {
        onmounted:move |_| {
            #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
            {let window=window.clone();spawn(async move{
                let query=dom_query::eval(VERIFY);let _=query.send(source);
                match query.join::<Value>().await {
                    Ok(value) if value["ok"]==true=>{println!("Button token native observations: {value}");window.close();}
                    Ok(value)=>{eprintln!("Button token native failure JSON: {value}");std::process::exit(1);}
                    Err(error)=>{eprintln!("Button token native transport failure: {error:?}");std::process::exit(1);}
                }
            });}
        },
        {web_controls()}
        if !source {button_review::ButtonReview {}}
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
        button {id:"button-run",r#type:"button",disabled:busy(),aria_busy:busy(),
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
        span {role:"status",style:"margin-left:12px","{status()}"}
        pre {id:"button-web-result","data-run":status(),"aria-label":"Browser Button token observations",style:"max-height:180px;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere;font-size:12px","{result()}"}
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
const fields=['display','width','height','minWidth','boxSizing','paddingTop','paddingRight','paddingBottom','paddingLeft','borderTopWidth','borderTopColor','borderTopStyle','borderTopLeftRadius','borderTopRightRadius','borderBottomLeftRadius','borderBottomRightRadius','backgroundColor','color','fontSize','lineHeight','fontWeight','fontFamily','fontStyle','letterSpacing','fontFeatureSettings','outlineColor','outlineWidth','outlineStyle','outlineOffset','opacity','pointerEvents','justifyContent','textAlign','marginLeft','marginRight'];
const metrics=node=>{if(!(node instanceof Element))return null;const cs=getComputedStyle(node),r=node.getBoundingClientRect();return{object:object(node),tag:node.tagName,id:node.id,class:node.getAttribute('class'),attrs:Object.fromEntries(Array.from(node.attributes,a=>[a.name,a.value])),computed:Object.fromEntries(fields.map(key=>[key,cs[key]])),rect:{x:r.x,y:r.y,width:r.width,height:r.height},filter:cs.filter};};
const control=wrap=>wrap.querySelector('.ix-button-standard');
const capture=(wrap,visualState)=>{
 const node=control(wrap),scope=wrap.closest('[data-button-scope]'),key=wrap.dataset.key,extra=wrap.dataset.buttonRow==='extra';
 if(!(node instanceof HTMLElement)||!scope)throw Error('Missing actual Button '+wrap.id);
 const leading=node.querySelector('.ix-button-icon-start'),trailing=node.querySelector('.ix-button-icon-end'),loading=node.querySelector('.ix-button-loading'),spinner=node.querySelector('.ix-spinner');
 const descriptorState=extra?(key==='host-active'?'host-active':key.startsWith('loading')?'loading':'rest'):visualState;
 return{id:wrap.id,scope:scope.dataset.buttonScope,scopeIndex:Number(scope.dataset.scopeIndex),revision:Number(scope.dataset.revision),variant:wrap.dataset.variant,state:descriptorState,kind:extra?'extra':'core',key,host:metrics(node),control:metrics(node),icons:[[leading,'start'],[trailing,'end']].filter(([el])=>el).map(([el,side])=>({side,node:metrics(el),glyph:metrics(el.querySelector('.ix-icon,svg'))})),spinner:spinner?[{host:metrics(loading??spinner),size:key==='loading12'?'xxs':key==='loading16'?'xs':'sm',inner:metrics(spinner),glyph:metrics(spinner.querySelector('svg')),hideTrack:spinner.classList.contains('ix-spinner-no-track')}]:[],content:metrics(node.querySelector('.ix-button-content')),nativeFacts:{disabledProperty:typeof node.disabled==='boolean'?node.disabled:null,ariaDisabled:node.getAttribute('aria-disabled'),hostTabIndex:node.tabIndex,controlTabIndex:node.tabIndex,focused:node===document.activeElement,matchesFocus:node.matches(':focus'),matchesFocusVisible:node.matches(':focus-visible'),matchesHover:node.matches(':hover'),matchesActive:node.matches(':active'),hostActive:node.classList.contains('active'),hostFocused:node.classList.contains('ix-focused'),loading:descriptorState==='loading',disabled:!extra&&descriptorState==='disabled',href:node.getAttribute('href'),target:node.getAttribute('target'),rel:node.getAttribute('rel')},wrapper:{object:object(wrap),attrs:Object.fromEntries(Array.from(wrap.attributes,a=>[a.name,a.value]))}};
};
const core=()=>Array.from(document.querySelectorAll('[data-button-row="core"]')),extras=()=>Array.from(document.querySelectorAll('[data-button-row="extra"]'));
const settle=async()=>{await document.fonts.ready;for(let index=1;index<=2;index++)await new Promise(resolve=>requestAnimationFrame(timestamp=>queueMicrotask(()=>{log('native-frame',{index,timestamp});resolve();})));};
const setRevision=async id=>{const button=document.getElementById('button-'+id);check(button instanceof HTMLButtonElement,'real Rust revision control '+id);log('public-revision-input',{id});button.click();await wait(()=>document.getElementById('button-review')?.dataset.phase===id&&document.getElementById('button-review')?.dataset.state==='rest','Rust revision/state commit '+id);await settle();};
const setState=async value=>{const select=document.getElementById('button-state');check(select instanceof HTMLSelectElement,'real Rust state selector');document.getElementById('button-outside').focus();select.value=value;select.dispatchEvent(new Event('change',{bubbles:true,composed:true}));log('public-state-input',{value});await wait(()=>document.getElementById('button-review')?.dataset.state===value,'Rust state commit '+value);await settle();};
const seedKeyboard=()=>{const node=document.getElementById('button-outside');node.focus();const event=new KeyboardEvent('keydown',{key:'Tab',bubbles:true,composed:true,cancelable:true});node.dispatchEvent(event);log('native-keyboard-seed',{defaultPrevented:event.defaultPrevented});};
const nativeClick=async node=>{const events=[];let original=null;const capture=event=>{if(event.composedPath()[0]!==node)return;original=event;events.push({site:'document-capture',defaultPrevented:event.defaultPrevented,trusted:event.isTrusted,targetTag:event.target.tagName,originTag:event.composedPath()[0].tagName});},bubble=event=>{if(event.composedPath()[0]!==node)return;events.push({site:'window-bubble',defaultPrevented:event.defaultPrevented,trusted:event.isTrusted,targetTag:event.target.tagName,originTag:event.composedPath()[0].tagName});};document.addEventListener('click',capture,true);window.addEventListener('click',bubble);node.click();document.removeEventListener('click',capture,true);window.removeEventListener('click',bubble);await settle();return{eventObserved:!!original,defaultPrevented:original?.defaultPrevented??null,events};};
try {
 if(source){
   await wait(()=>window.upstreamButtonTokenReview&&document.getElementById('button-status')?.dataset.ready==='true','pinned Button readiness');
   const result=await window.upstreamButtonTokenReview.run();check(result?.ok===true,'source run completed');check(result.phases.length===18,'source eighteen phase observations');
   check(result.phases.reduce((sum,p)=>sum+p.rows.length+p.extras.length,0)===684,'source 684 actual computed rows');
   return{...result,ok:result.ok&&errors.length===0,runnerAssertions:assertions,runnerErrors:errors,renderer:'Linux desktop WebView pinned source'};
 }
 await wait(()=>core().length===36&&extras().length===12,'Rust 36 core and 12 supplemental controls committed');
 for(const revisionID of ['baseline','revised','restored']){
   await setRevision(revisionID);
   for(const state of ['rest','hover','active','disabled','loading','focus']){
      await setState(state);const rows=[];
      if(state==='focus'){seedKeyboard();for(const wrap of core()){const node=control(wrap);log('native-focus-input-before',{id:wrap.id});node.focus();await settle();rows.push(capture(wrap,state));log('native-focus-input-observed',{id:wrap.id,focused:document.activeElement===node});}}
      else rows.push(...core().map(wrap=>capture(wrap,state)));
      check(rows.length===36,revisionID+':'+state+' 36 actual computed buttons');
      const supplemental=state==='rest'?extras().map(wrap=>capture(wrap,'rest')):[];
      if(state==='rest')check(supplemental.length===12,revisionID+' 12 actual supplemental controls');
      phases.push({id:revisionID+':'+state,revision:Number(document.getElementById('button-review').dataset.revision),state,rows,extras:supplemental,settleFrames:2});
   }
 }
 const activation=[];await setRevision('restored');
 const main=document.getElementById('button-review'),first=core()[0];
 for(const scenario of ['enabled','disabled','loading']){
    await setState(scenario==='enabled'?'rest':scenario);const node=control(first),before=Number(main.dataset.activations),observed=await nativeClick(node);
    activation.push({scenario,consumerDelta:Number(main.dataset.activations)-before,...observed,host:metrics(node),control:metrics(node)});
 }
 await setState('rest');const anchor=control(document.getElementById('button-extra-anchor')),before=Number(main.dataset.activations),observed=await nativeClick(anchor);
 activation.push({scenario:'anchor-consumer-cancel',consumerDelta:Number(main.dataset.activations)-before,...observed,host:metrics(anchor),control:metrics(anchor),consumerPrevented:main.dataset.consumerPrevented==='true'});
 const fallback=Array.from(document.querySelectorAll('#button-brand-fallback .ix-button-standard'),metrics);
 return{ok:errors.length===0,revision:'8bf599bc68365d7752e12832e71b625883b93d7a',fixture:'Developer-authored system-token Button adapter, not full Brand metrics',phases,errors,assertions,activation,journal,fallback,renderer:'Linux desktop WebView Rust',limits:['Core state aliases come from Rust public class/disabled/loading props; focus uses sequential actual native .focus after a synthetic Tab modality input','Fixed two native RAF plus font readiness follow observed public Rust control-state commits; no desired computed-style wait','Source host and inner control are separate; the flattened Rust Button uses the same actual native node for both fields','Icon/loading wrapper margins project source icon/spinner host margins; raw actual SVG/Spinner facts are retained separately','Actual .click smoke and consumer counters do not prove trusted physical input, submit or navigation parity','No-token normal/compact fallbacks remain separate provisional Brand diagnostics']};
}catch(error){errors.push(String(error));return{ok:false,phases,assertions,activation:[],journal,errors};}finally{abort.abort();}
"#;
