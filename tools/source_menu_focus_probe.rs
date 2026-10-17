//! Excluded mixed-modality source focus probe. Exact cached iX owns policy;
//! native event objects/focus seeding and consumer veto reactions are synthetic.
use dioxus::prelude::*;
use std::time::Duration;

#[path = "../src/dom_query.rs"]
mod dom_query;

fn main() {
    let index = include_str!("upstream_menu_hover_review.html")
        .replace("/www/", "http://127.0.0.1:8089/www/")
        .replace(
            "/keyboard-foundation.css",
            "http://127.0.0.1:8089/keyboard-foundation.css",
        )
        .replace(
            "</body>",
            "<div id=\"main\" hidden></div><!-- MODULE LOADER --></body>",
        );
    let config = dioxus::desktop::Config::new()
        .with_custom_index(index)
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("Pinned source mixed keyboard and hover focus")
                .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0, 1000.0))
                .with_visible(true),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}

const VERIFY: &str = r#"
try{
 await dioxus.recv();
 const get=id=>document.getElementById('upstream-menu-hover-'+id),sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const journal=[],observations=[],modeEvents=[],eventIds=new WeakMap();let scenario='bootstrap',sequence=0,eventSequence=0,latestObservedMode=null,current=null;
 const simple=node=>node instanceof Element?{id:node.id,tag:node.tagName}:null;
 const deepActive=()=>{let node=document.activeElement;while(node?.shadowRoot?.activeElement)node=node.shadowRoot.activeElement;return node;};
 const focusChain=()=>{const chain=[];let node=document.activeElement;while(node){chain.push(simple(node));node=node.shadowRoot?.activeElement;}return chain;};
 const within=(ancestor,target=deepActive())=>{for(let node=target;node;){if(node===ancestor)return true;if(node.assignedSlot){node=node.assignedSlot;continue;}const parent=node.parentNode;node=parent instanceof ShadowRoot?parent.host:parent;}return false;};
 const dropdown=category=>category.shadowRoot?.querySelector('ix-dropdown'),parent=category=>category.shadowRoot?.querySelector('.category-parent'),child=category=>category.querySelector('ix-menu-item');
 const presented=category=>{const d=dropdown(category),r=d?.getBoundingClientRect();return!!category.isConnected&&d?.show===true&&d.classList.contains('show')&&!!r?.width&&!!r?.height;};
 const parentFocused=category=>within(parent(category));
 const record=(kind,data={})=>{const row={seq:++sequence,scenario,kind,time:performance.now(),...data};journal.push(row);if(journal.length>600)journal.shift();return row;};
 const state=()=>({generation:window.upstreamMenuHoverReview?.snapshot()?.generation,a:current?{connected:current.a.isConnected,show:dropdown(current.a)?.show,presented:presented(current.a),parentFocused:parentFocused(current.a)}:null,b:current?{connected:current.b.isConnected,show:dropdown(current.b)?.show,presented:presented(current.b)}:null,focus:focusChain(),observedKeyboardMode:latestObservedMode});
 const assert=(ok,message)=>{if(!ok)throw Error(message+' '+JSON.stringify({state:state(),tail:journal.slice(-16)}));};
 const wait=async(predicate,label,timeout=2500)=>{const until=performance.now()+timeout;while(performance.now()<until){if(predicate())return;await sleep(5);}throw Error('Timeout '+label+' '+JSON.stringify({state:state(),tail:journal.slice(-16)}));};
 const frames=async(count=3)=>{for(let frame=0;frame<count;frame++)await new Promise(requestAnimationFrame);await sleep(0);};
 const elapsed=async(start,delay)=>{const remaining=start+delay-performance.now();if(remaining>0)await sleep(remaining);return performance.now()-start;};
 const globalListeners=new AbortController();
 document.addEventListener('keyboardmodechange',event=>{const mode=event.detail?.keyboardMode;if(typeof mode!=='boolean')return;latestObservedMode=mode;const row=record('source-keyboardmodechange',{detail:event.detail,trusted:event.isTrusted});modeEvents.push(row);},{passive:true,signal:globalListeners.signal});
 document.addEventListener('focusin',event=>record('native-focusin',{target:simple(event.target),origin:simple(event.composedPath()[0]),trusted:event.isTrusted,focus:focusChain()}),{capture:true,passive:true,signal:globalListeners.signal});
 function resetPointerMode(type='mousedown'){
  const target=get('outside'),start=performance.now();let event;
  if(type==='touchstart')event=typeof TouchEvent==='function'?new TouchEvent(type,{bubbles:true,cancelable:true,composed:true}):new Event(type,{bubbles:true,cancelable:true,composed:true});
  else event=new MouseEvent(type,{bubbles:true,cancelable:true,composed:true,button:0,buttons:1});
  record('driver-modality-event',{type,eventClass:event.constructor.name,trusted:event.isTrusted,target:simple(target)});target.dispatchEvent(event);
  assert(modeEvents.some(row=>row.time>=start&&row.detail.keyboardMode===false),'actual pinned detector emits keyboardmodechange(false) on '+type);
 }
 function key(target,value){const event=new KeyboardEvent('keydown',{key:value,code:value,bubbles:true,cancelable:true,composed:true}),start=performance.now();record('driver-key',{key:value,target:simple(target),trusted:event.isTrusted});target.dispatchEvent(event);record('driver-key-result',{key:value,defaultPrevented:event.defaultPrevented,focus:focusChain()});return{event,start};}
 function keyboardMode(){const {start}=key(document,'Tab');assert(modeEvents.some(row=>row.time>=start&&row.detail.keyboardMode===true),'actual pinned detector receives FOCUS_KEYS Tab');}
 function pointer(target,type,relatedTarget){const event=new PointerEvent(type,{pointerType:'mouse',pointerId:1,isPrimary:true,button:-1,buttons:0,relatedTarget,bubbles:type==='pointermove',cancelable:false,composed:type==='pointermove'}),time=performance.now();record('driver-pointer',{type,trusted:event.isTrusted,pointerType:event.pointerType,buttons:event.buttons,target:simple(target),relatedTarget:simple(relatedTarget)});target.dispatchEvent(event);return time;}
 function attach(ctx){
  ctx.abort=new AbortController();ctx.requests=[];ctx.completions=[];ctx.veto=false;ctx.acted=false;ctx.afterRequest=null;
  for(const [name,category]of [['a',ctx.a],['b',ctx.b]]){
   const d=dropdown(category);
   d.addEventListener('showChange',event=>{
    if(event.composedPath()[0]!==d)return;let dispatch=eventIds.get(event);if(!dispatch){dispatch=++eventSequence;eventIds.set(event,dispatch);}
    const row=record('source-showChange',{dispatch,category:name,detail:event.detail,cancelable:event.cancelable,trusted:event.isTrusted,defaultPreventedAtCapture:event.defaultPrevented,connected:category.isConnected,focusAtCapture:focusChain(),observedKeyboardMode:latestObservedMode});ctx.requests.push(row);
    queueMicrotask(()=>{row.defaultPreventedAfterDispatch=event.defaultPrevented;row.focusAfterDispatch=focusChain();});
   },{capture:true,passive:true,signal:ctx.abort.signal});
   d.addEventListener('showChanged',event=>{if(event.composedPath()[0]!==d)return;ctx.completions.push(record('source-showChanged',{category:name,detail:event.detail,trusted:event.isTrusted,connected:category.isConnected,focus:focusChain()}));},{capture:true,passive:true,signal:ctx.abort.signal});
   category.addEventListener('showChange',event=>{
    if(name!=='a'||event.composedPath()[0]!==d||event.detail!==false)return;
    record('consumer-after-target-request',{dispatch:eventIds.get(event),veto:ctx.veto,focus:focusChain(),connected:category.isConnected});
    if(ctx.veto){event.preventDefault();record('consumer-veto',{dispatch:eventIds.get(event),defaultPrevented:event.defaultPrevented});}
    // Source Dropdown target callback has already queued its nested RAF. A
    // removal can reenter willDismiss from disconnectedCallback: consume first.
    if(ctx.afterRequest&&!ctx.acted){ctx.acted=true;ctx.afterRequest(event);}
   },{signal:ctx.abort.signal});
  }
 }
 async function fresh(label){
  current?.abort.abort();scenario=label;const old=window.upstreamMenuHoverReview.snapshot().generation;get('fresh').click();
  await wait(()=>{const s=window.upstreamMenuHoverReview.snapshot();return s.ready&&s.generation!==old&&s.menu.expand===false;},'fresh pinned collapsed owner',10000);
  const ctx={a:get('a'),b:get('b'),outside:get('outside')};await Promise.all([dropdown(ctx.a),dropdown(ctx.b)].map(node=>node.componentOnReady()));attach(ctx);current=ctx;
  await wait(()=>!presented(ctx.a)&&!presented(ctx.b),'closed source baseline');resetPointerMode();return ctx;
 }
 async function keyboardOpen(ctx){
  keyboardMode();parent(ctx.a).focus();const {event}=key(parent(ctx.a),'Enter');assert(event.defaultPrevented,'source category Enter default prevented');
  await wait(()=>presented(ctx.a)&&ctx.completions.some(row=>row.category==='a'&&row.detail===true),'keyboard open source completion');
  await wait(()=>within(child(ctx.a)),'source first-child focus after Enter');child(ctx.a).focus();await frames();assert(within(child(ctx.a)),'actual seeded deep child focus');assert(latestObservedMode===true,'observed keyboard mode retained for close capture');record('seeded-child-focus',{focus:focusChain()});
 }
 const closeRequests=ctx=>ctx.requests.filter(row=>row.category==='a'&&row.detail===false),closeCompletions=ctx=>ctx.completions.filter(row=>row.category==='a'&&row.detail===false);
 async function settleClosed(ctx){await wait(()=>!presented(ctx.a)&&closeCompletions(ctx).length>0,'actual closed/source completion');await frames(3);}
 function save(ctx,data={}){observations.push({label:scenario,generation:window.upstreamMenuHoverReview.snapshot().generation,...data,state:state(),requests:closeRequests(ctx).map(row=>({time:row.time,dispatch:row.dispatch,connected:row.connected,observedKeyboardMode:row.observedKeyboardMode,defaultPreventedAtCapture:row.defaultPreventedAtCapture,defaultPreventedAfterDispatch:row.defaultPreventedAfterDispatch,focusAtCapture:row.focusAtCapture})),completions:closeCompletions(ctx).map(row=>({time:row.time,connected:row.connected,focus:row.focus}))});}
 await wait(()=>window.upstreamMenuHoverReview?.snapshot()?.ready,'source fixture bootstrap',20000);
 {
  const ctx=await fresh('keyboard Enter then hover grace returns category parent');await keyboardOpen(ctx);const leave=pointer(ctx.a,'pointerleave',get('body'));await settleClosed(ctx);assert(parentFocused(ctx.a),'keyboard grace close settles source category parent');assert(closeRequests(ctx).some(row=>row.time-leave>=248),'grace close has a delayed source request');save(ctx,{leave});
 }
 {
  const ctx=await fresh('pointer movement alone retains keyboard mode');await keyboardOpen(ctx);const move=pointer(get('body'),'pointermove',ctx.a);pointer(ctx.a,'pointerenter',get('body'));assert(latestObservedMode===true,'pointermove/enter do not emit pointer-mode reset');const leave=pointer(ctx.a,'pointerleave',get('body'));await settleClosed(ctx);assert(parentFocused(ctx.a),'pointer movement did not suppress keyboard close return');save(ctx,{move,leave});
 }
 for(const type of ['mousedown','touchstart']){
  const ctx=await fresh(type+' resets mode without outside focus');await keyboardOpen(ctx);resetPointerMode(type);assert(within(child(ctx.a)),'synthetic modality reset does not itself move child focus');const leave=pointer(ctx.a,'pointerleave',get('body'));await settleClosed(ctx);assert(!parentFocused(ctx.a),type+' mode reset has no category-parent return');save(ctx,{leave,reset:type});
 }
 {
  const ctx=await fresh('keyboard mode sibling entry closes A with parent return');await keyboardOpen(ctx);const enter=pointer(ctx.b,'pointerenter',ctx.a);await settleClosed(ctx);await wait(()=>presented(ctx.b),'B remains presented');assert(parentFocused(ctx.a),'A close request captured keyboard child focus despite B pointer entry');save(ctx,{enter});
 }
 {
  const ctx=await fresh('outside focus before timer inhibits return capture');await keyboardOpen(ctx);const leave=pointer(ctx.a,'pointerleave',get('body'));await elapsed(leave,70);ctx.outside.focus();const outsideFocus=performance.now();record('driver-outside-focus',{stage:'before timeout request',focus:focusChain()});await wait(()=>!presented(ctx.a)&&closeCompletions(ctx).length>0,'source focusout or timer closure');await elapsed(leave,450);await frames();assert(deepActive()===ctx.outside,'outside focus before request is not stolen');save(ctx,{leave,outsideFocus,classification:'focusout can close by property before timer; request absence is not queued-return cancellation'});
 }
 {
  const ctx=await fresh('veto close retains popup but source still returns parent');await keyboardOpen(ctx);ctx.veto=true;const leave=pointer(ctx.a,'pointerleave',get('body'));await wait(()=>closeRequests(ctx).some(row=>row.defaultPreventedAfterDispatch===true),'own canceled close request');await frames(3);await elapsed(leave,450);assert(presented(ctx.a)&&closeCompletions(ctx).length===0,'veto retains popup with no close completion');assert(parentFocused(ctx.a),'source request callback queued focus before veto outcome');save(ctx,{leave,veto:true});
 }
 {
  const ctx=await fresh('outside focus between queued frames does not cancel return');await keyboardOpen(ctx);ctx.veto=true;let shifted=false;
  ctx.afterRequest=()=>requestAnimationFrame(()=>{ctx.outside.focus();shifted=deepActive()===ctx.outside;record('consumer-first-RAF-outside-focus',{shifted,focus:focusChain()});});
  const leave=pointer(ctx.a,'pointerleave',get('body'));await wait(()=>shifted&&closeRequests(ctx).some(row=>row.defaultPreventedAfterDispatch===true),'consumer shifted focus after request');await frames(3);await elapsed(leave,450);assert(parentFocused(ctx.a),'queued source second-frame return is not canceled by later outside focus');assert(presented(ctx.a)&&closeCompletions(ctx).length===0,'vetoed popup remains open after queued return');save(ctx,{leave,veto:true,consumerOutsideFocusObserved:shifted});
 }
 {
  const ctx=await fresh('category removal before pending hover timeout observation');await keyboardOpen(ctx);const leave=pointer(ctx.a,'pointerleave',get('body'));await elapsed(leave,70);ctx.a.remove();const removed=performance.now();record('driver-category-remove',{stage:'before timer request',focus:focusChain()});await elapsed(leave,450);await frames(3);const d=dropdown(ctx.a),dialog=d?.shadowRoot?.querySelector('dialog');assert(!ctx.a.isConnected,'category remains removed');save(ctx,{leave,removed,detachedNativePopup:!!dialog?.matches(':popover-open'),classification:'actual disconnected requests/focus observed; no private resource/safety inference'});
 }
 {
  const ctx=await fresh('removal during close request before queued focus frames observation');await keyboardOpen(ctx);ctx.afterRequest=()=>{ctx.a.remove();record('consumer-category-remove',{stage:'bubble after source target request callback',focus:focusChain()});};
  const leave=pointer(ctx.a,'pointerleave',get('body'));await wait(()=>ctx.acted&&!ctx.a.isConnected,'category removed after first close request');await elapsed(leave,450);await frames(3);const dialog=dropdown(ctx.a)?.shadowRoot?.querySelector('dialog');save(ctx,{leave,detachedNativePopup:!!dialog?.matches(':popover-open'),classification:'removal can reenter controller dismissal; every request retained; no exact count or disconnected-focus safety assumption'});
 }
 for(const consumer of ['none','focus-outside','prevent-default','stop-propagation']){
  const ctx=await fresh('outside click Window bubble with '+consumer);await keyboardOpen(ctx);const begin=performance.now();let reachedWindow=false;
  const target=ctx.outside,event=new MouseEvent('click',{bubbles:true,cancelable:true,composed:true,button:0,buttons:0}),listeners=new AbortController();
  target.addEventListener('click',actual=>{
   if(consumer==='focus-outside')target.focus();
   if(consumer==='prevent-default')actual.preventDefault();
   if(consumer==='stop-propagation')actual.stopPropagation();
   record('consumer-outside-click-target',{consumer,defaultPrevented:actual.defaultPrevented,cancelBubbleDuringDispatch:actual.cancelBubble,focus:focusChain(),observedKeyboardMode:latestObservedMode});
  },{once:true,signal:listeners.signal});
  window.addEventListener('click',actual=>{if(actual!==event)return;reachedWindow=true;record('actual-window-click-bubble',{consumer,trusted:actual.isTrusted,defaultPrevented:actual.defaultPrevented,focus:focusChain()});},{once:true,signal:listeners.signal});
  record('driver-click-only',{consumer,trusted:event.isTrusted,buttons:event.buttons,target:simple(target),inputClass:event.constructor.name,classification:'synthetic click only; no mousedown/native focus default'});target.dispatchEvent(event);
  assert(latestObservedMode===true,'click-only consumer preserves observed source keyboard mode');
  if(consumer==='stop-propagation'){
   await frames(6);assert(!reachedWindow&&presented(ctx.a)&&within(child(ctx.a)),'stopped target click never reaches Window and retains popup/inside focus');assert(closeRequests(ctx).length===0&&closeCompletions(ctx).length===0,'stopped outside click causes no source controller close');
  }else{
   assert(reachedWindow,'outside click reaches actual Window bubble after target consumer');await settleClosed(ctx);
   assert(closeRequests(ctx).length>0&&closeCompletions(ctx).length>0,'Window outside click actually requests and completes source close');
   if(consumer==='focus-outside')assert(deepActive()===target&&closeRequests(ctx).every(row=>row.focusAtCapture.some(node=>node.id===target.id)),'target consumer focus occurs before source return capture and remains outside');
   else assert(parentFocused(ctx.a),'click-only outside controller closure returns keyboard-focused category parent');
   if(consumer==='prevent-default')assert(event.defaultPrevented,'defaultPrevented alone does not block source Window outside dismissal');
  }
  listeners.abort();save(ctx,{click:begin,consumer,reachedWindow,originalDefaultPrevented:event.defaultPrevented,classification:'one-shot target consumer; source Window bubble policy, no mousedown or trusted native click claim'});
 }
 {
  const ctx=await fresh('click-only B parent closes A with captured parent return');await keyboardOpen(ctx);const begin=performance.now(),target=parent(ctx.b),listeners=new AbortController();let reachedWindow=false,broadcast=null;
  const event=new MouseEvent('click',{bubbles:true,cancelable:true,composed:true,button:0,buttons:0});
  window.addEventListener('click',actual=>{if(actual===event){reachedWindow=true;record('actual-window-click-bubble',{target:simple(target)});}},{signal:listeners.signal});
  window.addEventListener('closeOtherCategories',actual=>{broadcast=record('source-closeOtherCategories',{detail:actual.detail,trusted:actual.isTrusted,origin:simple(actual.composedPath()[0]),focus:focusChain()});},{once:true,passive:true,signal:listeners.signal});
  record('driver-click-only',{target:simple(target),trusted:event.isTrusted,buttons:event.buttons,classification:'synthetic B parent click only; no mousedown, no B focus before dispatch',focus:focusChain()});target.dispatchEvent(event);
  assert(!reachedWindow,'source category parent stops original click before Window');assert(broadcast&&latestObservedMode===true,'source category emits closeOtherCategories without resetting keyboard mode');
  await settleClosed(ctx);await wait(()=>presented(ctx.b),'source B click actually presents B');assert(parentFocused(ctx.a),'source B click dismissal captures A child and returns A parent');listeners.abort();save(ctx,{click:begin,reachedWindow,broadcast,originalDefaultPrevented:event.defaultPrevented});
 }
 for(const priorKeyboardMode of [false,true]){
  const ctx=await fresh('collapsed Escape route observation with prior keyboard mode '+priorKeyboardMode);
  if(priorKeyboardMode)await keyboardOpen(ctx);
  else{pointer(ctx.a,'pointerenter',get('body'));await wait(()=>presented(ctx.a)&&ctx.completions.some(row=>row.category==='a'&&row.detail===true),'pointer-open source completion');child(ctx.a).focus();await frames();assert(within(child(ctx.a)),'pointer-mode Escape actual child focus');}
  assert(latestObservedMode===priorKeyboardMode,'source Escape prior mode observed from detector');const begin=performance.now(),listeners=new AbortController();let reachedDocument=false,documentObservation=null;
  document.addEventListener('keydown',actual=>{if(actual.key==='Escape'){reachedDocument=true;documentObservation=record('actual-document-keydown-bubble',{key:actual.key,trusted:actual.isTrusted,defaultPrevented:actual.defaultPrevented,observedKeyboardMode:latestObservedMode,focus:focusChain()});}},{signal:listeners.signal});
  const result=key(child(ctx.a),'Escape'),modeImmediatelyAfterDispatch=latestObservedMode;await settleClosed(ctx);assert(closeRequests(ctx).length>0,'actual source Escape requests dismissal');
  listeners.abort();save(ctx,{escape:begin,priorKeyboardMode,reachedDocument,documentObservation,originalDefaultPrevented:result.event.defaultPrevented,modeImmediatelyAfterDispatch,modeAtSettledEndpoint:latestObservedMode,classification:'observational routing: category supplies anchor, not trigger; Dropdown trigger-handler stop must not be assumed; document detector/Window controller reach, request capture mode and final deep focus recorded without prescribing route or endpoint'});
 }
 current?.abort.abort();globalListeners.abort();
 return{ok:true,observations,journal,input:'synthetic KeyboardEvent/PointerEvent/mousedown/touchstart/click-only MouseEvent and programmatic HTMLElement.focus; actual pinned source listeners/timers/RAF and consumer veto reactions',limits:['not physical/native hover or touch hardware; touchstart may use generic Event if TouchEvent unavailable','keyboard mode observed only through actual source keyboardmodechange events, no private detector state','BODY/parent endpoints are this Linux WebView source fixture; deep chains retained','source close request, completion, first-frame consumer focus and settled endpoint are separate boundaries','disconnected queued focus/resource behavior recorded without imposing a safety guarantee','no Rust parity, cross-VDOM/global-window proof, all veto/input/platform/AT/Brand or G04.6 nearest-scroll claim']};
}catch(error){return{ok:false,error:String(error)}}
"#;

#[component]
fn Probe() -> Element {
    let window = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(90)).await;
            eprintln!("Pinned source menu focus global timeout");
            std::process::exit(1);
        });
    });
    rsx! { div { onmounted: move |_| {
        let window = window.clone();
        spawn(async move {
            let query = dom_query::eval(VERIFY);
            let _ = query.send(());
            match query.join::<serde_json::Value>().await {
                Ok(result) if result["ok"] == true => { println!("Pinned source menu focus checks: {result}"); window.close(); }
                result => { eprintln!("Pinned source menu focus failure: {result:?}"); std::process::exit(1); }
            }
        });
    } } }
}
