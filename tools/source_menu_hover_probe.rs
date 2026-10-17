//! Excluded pinned-source handler/timer probe. PointerEvents are synthetic;
//! source components and their timers run unchanged in the real Linux WebView.
use dioxus::prelude::*;
use std::time::Duration;

#[path = "../src/dom_query.rs"]
mod dom_query;

fn main() {
    // alpha.2 protocol.rs injects its loader before </body>; the default root
    // name is main. Only this hidden root belongs to Dioxus, not the source UI.
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
                .with_title("Pinned source category hover handler probe")
                .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0, 1000.0))
                .with_visible(true),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}

const VERIFY: &str = r#"
try {
 await dioxus.recv();
 const prefix='upstream-menu-hover-',get=id=>document.getElementById(prefix+id),sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const observations=[],journal=[];let scenario='bootstrap',abort=null,dispatch=0;
 const snapshot=()=>window.upstreamMenuHoverReview?.snapshot();
 const simple=node=>node instanceof Element?{tag:node.tagName,id:node.id}:node instanceof ShadowRoot?{shadow:node.host.id}:null;
 const deepActive=()=>{let node=document.activeElement;while(node?.shadowRoot?.activeElement)node=node.shadowRoot.activeElement;return node;};
 const focusChain=()=>{const chain=[];let node=document.activeElement;while(node){chain.push(simple(node));node=node.shadowRoot?.activeElement;}return chain;};
 const observeFocus=stage=>journal.push({scenario,kind:'observed-focus',stage,time:performance.now(),chain:focusChain()});
 const assert=(condition,message)=>{if(!condition)throw Error(message+' '+JSON.stringify({scenario,state:snapshot(),tail:journal.slice(-12)}));};
 const wait=async(predicate,label,timeout=2500)=>{const end=performance.now()+timeout;while(performance.now()<end){if(predicate())return;await sleep(5);}throw Error('Timeout '+label+' '+JSON.stringify({scenario,state:snapshot(),tail:journal.slice(-12)}));};
 const dropdown=category=>category.shadowRoot?.querySelector('ix-dropdown');
 const presented=category=>{const d=dropdown(category),r=d?.getBoundingClientRect();return category.isConnected&&d?.show===true&&d.classList.contains('show')&&!!r?.width&&!!r?.height&&getComputedStyle(d).display!=='none';};
 const sourceEvents=(category,type='showChange',detail=false)=>journal.filter(row=>row.scenario===scenario&&row.kind==='source'&&row.category===category.id&&row.type===type&&row.detail===detail);
 function attach(categories){
  abort?.abort();abort=new AbortController();
  for(const category of categories){const d=dropdown(category);
   for(const type of ['showChange','showChanged'])d.addEventListener(type,event=>{
    if(event.target!==d)return;
    journal.push({scenario,kind:'source',category:category.id,type,time:performance.now(),eventTimeStamp:event.timeStamp,detail:event.detail,trusted:event.isTrusted,connected:category.isConnected,cancelable:event.cancelable,defaultPrevented:event.defaultPrevented});
   },{capture:true,passive:true,signal:abort.signal});
   category.addEventListener('closeOtherCategories',event=>journal.push({scenario,kind:'sibling-source',category:category.id,type:event.type,time:performance.now(),detail:event.detail,trusted:event.isTrusted}),{capture:true,passive:true,signal:abort.signal});
  }
 }
 function pointer(target,type,relatedTarget,pointerType='mouse'){
  const event=new PointerEvent(type,{pointerType,pointerId:1,isPrimary:true,buttons:0,button:-1,relatedTarget,bubbles:false,cancelable:false,composed:false});
  const row={scenario,kind:'driver-pointer',dispatch:++dispatch,type,time:performance.now(),pointerType,buttons:event.buttons,button:event.button,trusted:event.isTrusted,target:simple(target),relatedTarget:simple(relatedTarget)};
  journal.push(row);target.dispatchEvent(event);row.end=performance.now();return row.time;
 }
 async function fresh(label){
  scenario=label;abort?.abort();const prior=snapshot()?.generation;get('fresh').click();
  // Fresh replaces nodes synchronously, then marks readiness after hydration.
  await wait(()=>snapshot()?.generation!==prior&&snapshot()?.ready&&snapshot()?.menu?.expand===false&&document.querySelector('ix-menu-category')?.isConnected,'fresh collapsed source ready',10000);
  const a=get('a'),b=get('b');assert(a&&b&&a!==b,'two source categories');
  await Promise.all([dropdown(a),dropdown(b)].map(node=>node.componentOnReady()));
  attach([a,b]);
  await wait(()=>!presented(a)&&!presented(b),'fresh panels closed');
  return {a,b,outside:get('body'),generation:snapshot().generation,initial:snapshot()};
 }
 async function open(category,outside){
  const before=deepActive();observeFocus('before first entry');pointer(category,'pointerenter',outside);observeFocus('synchronous after first entry');
  assert(deepActive()===before,'first synthetic entry does not synchronously move deep native focus');
  await wait(()=>presented(category),'synthetic source entry presents');await wait(()=>sourceEvents(category,'showChanged',true).length>0,'source open completion');
  observeFocus('after first source open completion');assert(deepActive()===before,'first synthetic entry/source completion preserves deep native focus');
 }
 function publicExpand(value){const node=get('source'),time=performance.now();journal.push({scenario,kind:'driver-public-property',time,target:simple(node),property:'expand',value});node.expand=value;return time;}
 async function atElapsed(start,elapsed){const rest=start+elapsed-performance.now();if(rest>0)await sleep(rest);return performance.now()-start;}
 async function closing(category,start,label){
  await wait(()=>sourceEvents(category).length>0,label+' own close request',2000);
  const request=sourceEvents(category)[0],delta=request.time-start;
  assert(delta>=248,label+' close request not before250ms (2ms clock tolerance)');
  await wait(()=>!presented(category)&&sourceEvents(category,'showChanged',false).length>0,label+' settled close completion',2000);
  const completion=sourceEvents(category,'showChanged',false)[0];
  return{leave:start,request:request.time,requestDelta:delta,completion:completion.time,completionDelta:completion.time-start};
 }
 function retain(category,label){assert(presented(category),label+' stays presented');assert(sourceEvents(category).length===0,label+' has no close request');}
 function save(label,data){observations.push({label,generation:snapshot()?.generation,...data,state:snapshot()});}
 await wait(()=>window.upstreamMenuHoverReview&&snapshot()?.ready,'initial fixture bootstrap',20000);
 {
  const {a,outside}=await fresh('outside leave250ms');await open(a,outside);const leave=pointer(a,'pointerleave',outside);
  const earlyElapsed=await atElapsed(leave,100);assert(earlyElapsed<200,'actual early sample before200ms');retain(a,'early outside leave');
  const timing=await closing(a,leave,'outside mouse');save(scenario,{earlyElapsed,...timing});
 }
 {
  const {a,outside}=await fresh('host enter cancels pending leave');await open(a,outside);const leave=pointer(a,'pointerleave',outside);
  await atElapsed(leave,80);assert(performance.now()-leave<200,'host reentry delivered before deadline');const enter=pointer(a,'pointerenter',outside);
  await atElapsed(leave,450);retain(a,'host reentry after original deadline');save(scenario,{leave,enter,elapsed:performance.now()-leave});
 }
 {
  const {a,outside}=await fresh('leave enter leave restarts deadline');await open(a,outside);const first=pointer(a,'pointerleave',outside);
  await atElapsed(first,70);pointer(a,'pointerenter',outside);await atElapsed(first,130);assert(performance.now()-first<200,'restart setup fits original deadline');const last=pointer(a,'pointerleave',outside);
  const oldDeadlineSample=await atElapsed(first,290);assert(performance.now()-last<200,'restart sample before latest deadline');retain(a,'past original deadline');
  save(scenario,{first,last,oldDeadlineSample,...await closing(a,last,'restarted mouse')});
 }
 for(const targetKind of ['light-child','category-shadow','dropdown-host','dropdown-shadow']){
  const {a,outside}=await fresh('accepted relatedTarget '+targetKind);await open(a,outside);const d=dropdown(a);
  const target=targetKind==='light-child'?a.querySelector('ix-menu-item'):targetKind==='category-shadow'?a.shadowRoot.querySelector('.category-parent'):targetKind==='dropdown-host'?d:d.shadowRoot.querySelector('[role="presentation"]');
  assert(target instanceof Node,'actual internal related Node '+targetKind);const leave=pointer(a,'pointerleave',target);
  await atElapsed(leave,450);retain(a,'accepted internal target without pending timer');save(scenario,{leave,targetKind,target:simple(target)});
 }
 {
  const {a,outside}=await fresh('internal leave does not clear old timeout');await open(a,outside);const leave=pointer(a,'pointerleave',outside);
  await atElapsed(leave,80);const internal=pointer(a,'pointerleave',dropdown(a));assert(internal-leave<200,'internal leave before old deadline');
  save(scenario,{internal,...await closing(a,leave,'accepted internal leave with preexisting timer')});
 }
 {
  const {a,outside}=await fresh('own-ID sibling event does not clear old timeout');await open(a,outside);const leave=pointer(a,'pointerleave',outside);
  await atElapsed(leave,80);assert(performance.now()-leave<200,'own-ID injection before old deadline');
  const ownId=a.shadowRoot.querySelector('.category-parent')?.id;assert(ownId&&journal.some(row=>row.scenario===scenario&&row.kind==='sibling-source'&&row.detail===ownId),'actual generated own ID from source entry');
  const event=new CustomEvent('closeOtherCategories',{detail:ownId,bubbles:true,cancelable:true,composed:true}),injected=performance.now();
  journal.push({scenario,kind:'driver-synthetic-custom-event',type:event.type,time:injected,target:simple(a),detail:ownId,trusted:event.isTrusted,bubbles:event.bubbles,cancelable:event.cancelable,composed:event.composed});a.dispatchEvent(event);
  save(scenario,{injected,ownId,...await closing(a,leave,'own-ID sibling event with pending timer')});
 }
 {
  const {a,outside}=await fresh('flyout enter clears old timeout');await open(a,outside);const leave=pointer(a,'pointerleave',outside);
  await atElapsed(leave,80);const enter=pointer(dropdown(a),'pointerenter',a);assert(enter-leave<200,'flyout enter before old deadline');
  await atElapsed(leave,450);retain(a,'flyout enter');save(scenario,{leave,enter,elapsed:performance.now()-leave});
 }
 {
  const {a,outside}=await fresh('touch leave exemption');await open(a,outside);const leave=pointer(a,'pointerleave',outside,'touch');
  await atElapsed(leave,450);retain(a,'touch leave');save(scenario,{leave,elapsed:performance.now()-leave});
 }
 {
  const {a,b,outside}=await fresh('sibling ownership');await open(a,outside);const leave=pointer(a,'pointerleave',outside);await atElapsed(leave,70);const enter=pointer(b,'pointerenter',a);
  await wait(()=>presented(b)&&!presented(a),'B owns flyout and A closes');await atElapsed(leave,450);retain(b,'B survives old A deadline');
  const sibling=journal.filter(row=>row.scenario===scenario&&row.kind==='sibling-source');assert(new Set(sibling.map(row=>row.detail)).size===2,'distinct source sibling category IDs');save(scenario,{leave,enter,sibling});
 }
 {
  const {a,outside}=await fresh('disconnect after pending leave');await open(a,outside);const leave=pointer(a,'pointerleave',outside);
  await atElapsed(leave,70);assert(performance.now()-leave<200,'disconnect before deadline');a.remove();const removed=performance.now(),focusAfterRemoval=document.activeElement;
  await atElapsed(leave,450);const d=dropdown(a),dialog=d?.shadowRoot?.querySelector('dialog');assert(!a.isConnected&&d.getClientRects().length===0&&!dialog?.matches(':popover-open'),'disconnected category has no visible/native popup revival');assert(document.activeElement===focusAfterRemoval,'no delayed focus steal after disconnected grace');
  save(scenario,{leave,removed,elapsed:performance.now()-leave,resourceLimit:'no private timer resource introspection'});
  const old=a;await fresh('fresh generation after disconnected timer');await sleep(300);assert(get('a')!==old&&!presented(get('a'))&&!presented(get('b')),'fresh source generation remains closed');save(scenario,{});
 }
 {
  const {a,outside}=await fresh('pending leave then public expanded mode observation');await open(a,outside);const leave=pointer(a,'pointerleave',outside);
  await atElapsed(leave,70);assert(performance.now()-leave<200,'public mode change before old deadline');const modeWrite=publicExpand(true);
  const earlyElapsed=await atElapsed(leave,150),early=snapshot();const lateElapsed=await atElapsed(leave,450),late=snapshot();
  save(scenario,{leave,modeWrite,samples:[{elapsed:earlyElapsed,state:early},{elapsed:lateElapsed,state:late}],closeRequests:sourceEvents(a),interpretation:'public mode/timer interaction is observed, no prescribed popup/panel visibility outcome'});
 }
 {
  const {a,outside}=await fresh('expanded host entry clears pending timeout before guard');await open(a,outside);const leave=pointer(a,'pointerleave',outside);
  await atElapsed(leave,70);assert(performance.now()-leave<200,'expanded reentry setup before old deadline');const modeWrite=publicExpand(true);
  const siblingsBefore=journal.filter(row=>row.scenario===scenario&&row.kind==='sibling-source').length,openRequestsBefore=sourceEvents(a,'showChange',true).length;
  const enter=pointer(a,'pointerenter',outside);assert(journal.filter(row=>row.scenario===scenario&&row.kind==='sibling-source').length===siblingsBefore&&sourceEvents(a,'showChange',true).length===openRequestsBefore,'expanded guard issues no sibling/open request');
  const earlyElapsed=await atElapsed(leave,150),early=snapshot();const lateElapsed=await atElapsed(leave,450),late=snapshot(),closeRequests=sourceEvents(a);
  save(scenario,{leave,modeWrite,enter,samples:[{elapsed:earlyElapsed,state:early},{elapsed:lateElapsed,state:late}],closeRequests,sourceExpected:'host entry clears timeout before the expanded guard; timer handle is not inspected',interpretation:closeRequests.length?'mode/other closure observed; timeout-cancellation causality not isolated':'no close request through old deadline after expanded enter; popup/panel visibility recorded independently'});
 }
 {
  const {a,outside}=await fresh('expanded hover guard');get('expand').click();await wait(()=>snapshot()?.menu?.expand===true,'public expanded Menu');pointer(a,'pointerenter',outside);await sleep(350);assert(!presented(a)&&sourceEvents(a,'showChange',true).length===0,'expanded entry does not present collapsed popup');save(scenario,{});
 }
 {
  const {a,outside}=await fresh('active child dual surface projection and focus observation');
  const child=a.querySelector('ix-menu-item'),popup=dropdown(a);child.active=true;
  journal.push({scenario,kind:'driver-public-property',time:performance.now(),target:simple(child),property:'active',value:true});
  await wait(()=>child.classList.contains('active'),'source active child committed');await open(a,outside);
  child.focus();await sleep(30);const focused=deepActive(),beforeFocus=focusChain(),beforeSlot=child.assignedSlot;
  const modeWrite=publicExpand(true);await wait(()=>snapshot()?.categories?.a?.expandedPanel===true,'source expanded active panel');
  await sleep(60);const afterFocus=focusChain(),afterSlot=child.assignedSlot,panel=a.shadowRoot.querySelector('.menu-items');
  const inPanel=afterSlot?.parentElement===panel;
  assert(a.querySelector('ix-menu-item')===child&&dropdown(a)===popup,'source child and popup native references retained');
  assert(presented(a)&&snapshot().categories.a.expandedPanel&&inPanel,'source simultaneously open shell with first-slot child projection to expanded panel');
  save(scenario,{modeWrite,beforeFocus,afterFocus,sameDeepFocus:deepActive()===focused,sameNativeChild:true,sameNativePopup:true,childInPanel:inPanel,slotChanged:afterSlot!==beforeSlot,focusClassification:'programmatic native HTMLElement.focus; source public active/expand properties, not physical input'});
 }
 abort?.abort();
 return{ok:true,observations,journal,nativeCapabilities:{moveBefore:typeof Element.prototype.moveBefore==='function'},input:'synthetic native PointerEvent objects delivered to exact pinned source hosts; buttons0, isTrustedfalse; synthetic own-ID CustomEvent and direct public Menu.expand writes explicitly journaled; ordinary synthetic .click only for untimed fixture controls',limits:['entry focus assertions begin at actual deep native focus (BODY in this harness), not physical hover input','not normal/physical hover, real touch, or held-button browser drag','no native hit-strip/diagonal path proof from PointerEvent dispatch','2ms lower-bound timing tolerance; no source maximum completion latency claim','no private timeout resource introspection; disconnect checks visible/native state only','source framework/global-window scope and all mode/veto/pen/platform races not exhausted','G04.6 nearest scrolling and Brand/AT parity remain separate']};
}catch(error){return{ok:false,error:String(error)}}
"#;

#[component]
fn Probe() -> Element {
    let window = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(90)).await;
            eprintln!("Pinned source hover probe global timeout");
            std::process::exit(1);
        });
    });
    rsx! { div { onmounted: move |_| {
        let window = window.clone();
        spawn(async move {
            let query = dom_query::eval(VERIFY);
            let _ = query.send(());
            match query.join::<serde_json::Value>().await {
                Ok(result) if result["ok"] == true => {
                    println!("Pinned source hover handler checks: {result}");
                    window.close();
                }
                result => {
                    eprintln!("Pinned source hover handler failure: {result:?}");
                    std::process::exit(1);
                }
            }
        });
    } } }
}
