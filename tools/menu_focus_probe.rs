//! Excluded paired mixed keyboard/hover focus replay. Synthetic initiating
//! events exercise actual Rust policy/native execution; no consumer veto API is
//! invented. Source requests and this renderer's commits are distinct evidence.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::time::Duration;

#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "menu_hover_review.rs"]
mod menu_hover_review;

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("iX mixed keyboard and hover focus probe")
                .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0, 1400.0))
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
 const get=id=>document.getElementById('g04-hover-'+id),sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const journal=[],observations=[],globalListeners=new AbortController();let scenario='bootstrap',sequence=0,watch=null;
 const stage=()=>get('stage'),menu=()=>stage()?.querySelector('nav.ix-menu');
 const category=(root,label)=>[...root?.querySelectorAll('.ix-menu-category')??[]].find(host=>host.querySelector(':scope > [role=menuitem]')?.getAttribute('aria-label')===label);
 const a=()=>category(stage(),'Category A'),b=()=>category(stage(),'Category B'),peer=()=>category(document.getElementById('g04-focus-peer-stage'),'Peer category');
 const parent=host=>host?.querySelector(':scope > [role=menuitem]'),child=host=>host?.querySelector('[data-menu-category-child]'),popupHost=host=>host?.querySelector(':scope > [data-menu-category-popup]');
 const popup=host=>host?.getAttribute('data-menu-category-popup-open')==='true',mode=host=>host?.getAttribute('data-menu-category-keyboard-mode'),jobs=host=>{try{const value=JSON.parse(host?.getAttribute('data-menu-category-return-jobs')??'[]');return Array.isArray(value)?value:[];}catch{return[];}};
 const nativeOpen=host=>{try{return!!popupHost(host)?.matches(':popover-open');}catch{return false;}};
 const presented=host=>{const p=popupHost(host),r=p?.getBoundingClientRect();return!!host?.isConnected&&popup(host)&&!!p&&!p.hidden&&!!r?.width&&!!r?.height&&nativeOpen(host);};
 const closed=host=>!!host?.isConnected&&!popup(host)&&!nativeOpen(host);
 const node=el=>el?{id:el.id,tag:el.tagName,label:el.getAttribute?.('aria-label')}:null;
 const deepActive=()=>{let el=document.activeElement;while(el?.shadowRoot?.activeElement)el=el.shadowRoot.activeElement;return el;};
 const focusChain=()=>{const out=[];let el=document.activeElement;while(el){out.push(node(el));el=el.shadowRoot?.activeElement;}return out;};
 const record=(kind,data={})=>{const row={seq:++sequence,scenario,kind,time:performance.now(),...data};journal.push(row);if(journal.length>900)journal.shift();return row;};
 const hostState=host=>({node:node(host),connected:!!host?.isConnected,popup:popup(host),nativePopover:nativeOpen(host),mode:mode(host),jobs:jobs(host),parent:node(parent(host)),parentFocused:!!host&&deepActive()===parent(host)});
 const snapshot=()=>({a:hostState(a()),b:hostState(b()),peer:hostState(peer()),focus:focusChain()});
 const assert=(ok,label)=>{if(!ok)throw Error(label+' '+JSON.stringify({scenario,snapshot:snapshot(),tail:journal.slice(-18)}));};
 const wait=async(predicate,label,timeout=4000)=>{const end=performance.now()+timeout;while(performance.now()<end){if(predicate())return;await sleep(5);}throw Error('Timeout '+label+' '+JSON.stringify({scenario,snapshot:snapshot(),watch:watch?{triggered:watch.triggered,executed:watch.executed,error:watch.error}:null,tail:journal.slice(-18)}));};
 const frames=async(count=3)=>{for(let i=0;i<count;i++)await new Promise(requestAnimationFrame);await sleep(0);};
 const elapsed=async(start,delay)=>{const left=start+delay-performance.now();if(left>0)await sleep(left);return performance.now()-start;};
 const readyHost=host=>host?.dataset.categoryNative==='true'&&host?.dataset.categoryHoverNative==='true'&&host.hasAttribute('data-menu-category-return-jobs')&&['true','false'].includes(mode(host));
 const ready=()=>menu()?.dataset.menuNative==='true'&&readyHost(a())&&readyHost(b())&&readyHost(peer())&&[...document.querySelectorAll('nav.ix-menu input[data-menu-hover-observer]')].every(input=>input.dataset.menuDocumentReady==='true');
 document.addEventListener('input',event=>{if(!event.target.matches?.('input[data-menu-hover-observer],input[data-menu-observer],input[data-ix-overlay-transport]'))return;try{record('raw-private-transport',{transport:event.target.hasAttribute('data-ix-overlay-transport')?'overlay-document/window':event.target.hasAttribute('data-menu-hover-observer')?'hover/modality/capture':'navigation/outside',detail:JSON.parse(event.target.value),focus:focusChain()});}catch{}},{capture:true,passive:true,signal:globalListeners.signal});
 document.addEventListener('keydown',event=>record('actual-document-keydown-bubble',{key:event.key,trusted:event.isTrusted,defaultPrevented:event.defaultPrevented}),{passive:true,signal:globalListeners.signal});
 document.addEventListener('focusin',event=>record('native-focusin',{target:node(event.target),origin:node(event.composedPath()[0]),trusted:event.isTrusted,focus:focusChain()}),{capture:true,passive:true,signal:globalListeners.signal});
 const mutations=new MutationObserver(records=>{
  for(const mutation of records){const host=mutation.target;if(!host.matches?.('.ix-menu-category'))continue;
   record('dom-state-commit',{attribute:mutation.attributeName,previous:mutation.oldValue,value:host.getAttribute(mutation.attributeName),category:host.getAttribute('data-menu-category-root'),label:parent(host)?.getAttribute('aria-label'),focus:focusChain()});
   if(mutation.attributeName==='data-menu-category-return-jobs'&&watch&&watch.host===host&&!watch.triggered){
    const tokens=jobs(host);if(!tokens.length)continue;watch.triggered=true;watch.tokens=tokens;watch.marker=record('observed-return-authorization-marker',{tokens,focus:focusChain(),boundary:'Rust authorization attr/VDOM commit; native Eval may already have begun its frames'});
    if(deepActive()===parent(host)){watch.error='PREREQUISITE_MISSED: marker already observes parent return; cannot place consumer action before it';continue;}
    const selected=watch;selected.frame=requestAnimationFrame(()=>{
     selected.frame=0;if(deepActive()===parent(host)){selected.error='PREREQUISITE_MISSED: parent return preceded consumer first-RAF action';return;}
     selected.executed=true;selected.action();
    });
   }
  }
 });
 mutations.observe(document.body,{subtree:true,attributes:true,attributeOldValue:true,attributeFilter:['data-menu-category-popup-open','data-menu-category-keyboard-mode','data-menu-category-return-jobs']});
 function pointer(target,type,relatedTarget){assert(target?.isConnected,'connected synthetic pointer target');const event=new PointerEvent(type,{pointerType:'mouse',pointerId:1,isPrimary:true,button:-1,buttons:0,relatedTarget,bubbles:type==='pointermove',composed:type==='pointermove',cancelable:false}),time=performance.now();record('driver-pointer',{type,target:node(target),relatedTarget:node(relatedTarget),pointerType:event.pointerType,buttons:event.buttons,trusted:event.isTrusted});target.dispatchEvent(event);return time;}
 function key(target,value){const event=new KeyboardEvent('keydown',{key:value,code:value,bubbles:true,cancelable:true,composed:true}),time=performance.now();record('driver-key',{key:value,target:node(target),trusted:event.isTrusted});target.dispatchEvent(event);record('driver-key-result',{key:value,defaultPrevented:event.defaultPrevented,focus:focusChain()});return{event,time};}
 function resetModality(type='mousedown'){
  const event=type==='touchstart'?(typeof TouchEvent==='function'?new TouchEvent(type,{bubbles:true,cancelable:true,composed:true}):new Event(type,{bubbles:true,cancelable:true,composed:true})):new MouseEvent(type,{bubbles:true,cancelable:true,composed:true,button:0,buttons:1});
  const time=performance.now();record('driver-modality',{type,eventClass:event.constructor.name,trusted:event.isTrusted});get('after').dispatchEvent(event);return time;
 }
 const rawModality=(after,kind,keyName)=>journal.some(row=>row.scenario===scenario&&row.time>=after&&row.kind==='raw-private-transport'&&row.detail.entries?.at(-1)?.kind===kind&&(!keyName||row.detail.entries.at(-1).key===keyName));
 const overlayWindowReports=(after,kind,host,keyName)=>journal.filter(row=>row.scenario===scenario&&row.time>=after&&row.kind==='raw-private-transport'&&row.transport==='overlay-document/window'&&row.detail.kind===kind&&row.detail.phase==='window'&&typeof row.detail.document==='string'&&row.detail.document.length>0&&row.detail.window_sequence>0&&(!keyName||row.detail.key===keyName)&&row.detail.nodes?.some(node=>node.id===host.getAttribute('data-ix-overlay-root')&&node.owner===host.getAttribute('data-ix-overlay-owner')&&node.connected));
 function control(id,value){const el=get(id),time=performance.now();if(el instanceof HTMLInputElement&&el.type==='checkbox')el.checked=value;else el.value=value;record('fixture-control-change',{id,value});el.dispatchEvent(new Event('change',{bubbles:true}));return time;}
 function clearWatch(){if(watch?.frame)cancelAnimationFrame(watch.frame);watch=null;}
 async function fresh(label){
  clearWatch();scenario=label;const old=menu();get('fresh').click();await wait(()=>menu()&&menu()!==old&&ready()&&closed(a())&&closed(b()),'fresh collapsed owner/registered modality+return protocol');
  const reset=resetModality();await wait(()=>rawModality(reset,'mousedown')&&mode(a())==='false'&&mode(peer())==='false','actual raw mousedown resets root Rust mode');
  if(popup(peer())){pointer(peer(),'pointerleave',get('body'));await wait(()=>closed(peer()),'old peer closes');}
 }
 async function keyboardOpen(){
  parent(a()).focus();const opened=key(parent(a()),'Enter');assert(opened.event.defaultPrevented,'parent Enter default prevented');
  await wait(()=>journal.some(row=>row.scenario===scenario&&row.kind==='actual-document-keydown-bubble'&&row.key==='Enter'&&row.time>=opened.time)&&rawModality(opened.time,'keydown','Enter')&&mode(a())==='true','actual Enter document-bubble/raw observation sets Rust mode');
  await wait(()=>presented(a())&&deepActive()===child(a()),'keyboard-open actual child focus');child(a()).focus();await frames(2);assert(deepActive()===child(a()),'seeded actual child focus');record('seeded-child-focus',{focus:focusChain()});return child(a());
 }
 const exposedJobs=(host,after=0)=>journal.filter(row=>row.scenario===scenario&&row.kind==='dom-state-commit'&&row.category===host.getAttribute('data-menu-category-root')&&row.attribute==='data-menu-category-return-jobs'&&row.time>=after).filter(row=>{try{return JSON.parse(row.value).length>0;}catch{return false;}});
 const closeCommit=(host,start)=>journal.find(row=>row.scenario===scenario&&row.kind==='dom-state-commit'&&row.category===host.getAttribute('data-menu-category-root')&&row.attribute==='data-menu-category-popup-open'&&row.value==='false'&&row.time>=start);
 async function settleParent(host=a()){await wait(()=>closed(host)&&deepActive()===parent(host)&&jobs(host).length===0,'closed/settled category-parent focus/consumed jobs');await frames(3);assert(deepActive()===parent(host),'parent remains after paint');}
 function save(extra={}){observations.push({label:scenario,...extra,snapshot:snapshot()});}
 function arm(host,action){watch={host,action,triggered:false,executed:false,error:null,frame:0,tokens:[]};}
 async function requireAction(){try{await wait(()=>watch?.error||watch?.executed,'return marker and first-RAF consumer action');}catch(error){throw Error('AUTHORIZATION_MARKER_MISSING: cannot distinguish absent authorization from unobserved delivery interval; '+String(error));}assert(!watch.error,watch.error);}
 await wait(ready,'initial actual hover/modality/return protocol');
 await fresh('keyboard Enter then hover grace returns A parent');await keyboardOpen();{
  const leave=pointer(a(),'pointerleave',get('body'));await settleParent();const close=closeCommit(a(),leave);assert(close&&close.time-leave>=248,'actual popup close commit after250ms grace with2ms clock tolerance');assert(exposedJobs(a(),leave).length>0,'pre-dismiss return authorization exposed');save({leave,closeCommitDelta:close.time-leave,timingBoundary:'DOM commit, not source showChange request/completion'});
 }
 await fresh('pointermove/enter preserve document-bubble keyboard mode');await keyboardOpen();{
  const move=pointer(get('body'),'pointermove',a());pointer(a(),'pointerenter',get('body'));assert(mode(a())==='true','pointer movement leaves Rust keyboard mode true');const leave=pointer(a(),'pointerleave',get('body'));await settleParent();save({move,leave});
 }
 for(const type of ['mousedown','touchstart']){
  await fresh(type+' resets mode without native focus default');await keyboardOpen();const reset=resetModality(type);await wait(()=>rawModality(reset,type)&&mode(a())==='false','actual raw '+type+' updates mode');assert(deepActive()===child(a()),'synthetic modality event leaves child focus inside');const leave=pointer(a(),'pointerleave',get('body'));await wait(()=>closed(a()),'mouse-mode closure');await frames(4);assert(deepActive()===document.body,'reset mode closes to BODY without category return');assert(exposedJobs(a(),reset).length===0,'reset mode exposes no return authorization');save({reset,leave});
 }
 await fresh('keyboard mode B entry closes A yet returns A parent');await keyboardOpen();{
  const enter=pointer(b(),'pointerenter',a());await settleParent();await wait(()=>presented(b()),'B remains actually presented');save({enter});
 }
 await fresh('same VDOM peer Menu preserves A captured return');await keyboardOpen();{
  const enter=pointer(peer(),'pointerenter',a());await settleParent();await wait(()=>presented(peer()),'separate Menu peer stays presented');save({enter,scope:'separate Menu in same VDOM, not Window/process-global equivalence'});
 }
 await fresh('outside focus before deadline uses property hide and stays outside');await keyboardOpen();{
  const leave=pointer(a(),'pointerleave',get('body'));await elapsed(leave,70);get('after').focus();const moved=record('driver-outside-focus',{boundary:'before deadline',focus:focusChain()});await wait(()=>journal.some(row=>row.scenario===scenario&&row.kind==='raw-private-transport'&&row.detail.kind==='category-outside'&&row.detail.outside_cause==='focusout'&&row.detail.inside===false&&row.detail.category===Number(a().getAttribute('data-menu-category-root'))),'actual focusout property-path report');await wait(()=>closed(a()),'focusout/property hides popup');await elapsed(leave,450);await frames(3);assert(deepActive()===get('after'),'outside-before focus is not stolen');assert(exposedJobs(a(),leave).length===0,'property hide exposes no controller return job');const propertyReport=journal.find(row=>row.scenario===scenario&&row.kind==='raw-private-transport'&&row.detail.kind==='category-outside'&&row.detail.outside_cause==='focusout'&&row.detail.inside===false);save({leave,outsideFocus:moved.time,propertyReport,closeCommit:closeCommit(a(),leave),sourceComparison:'source has0false requests/1completion; this port reports typed focusout property facts/commits/job authorization, not source callbacks'});
 }
 await fresh('outside focus after return authorization is followed by A parent');await keyboardOpen();{
  const old=a();let shifted=null;arm(old,()=>{get('after').focus();shifted=record('consumer-first-RAF-outside-focus',{tokens:watch.tokens,actualOutside:deepActive()===get('after'),focus:focusChain()});});
  const leave=pointer(old,'pointerleave',get('body'));await requireAction();assert(shifted?.actualOutside,'actual consumer outside focus observed');await settleParent(old);const parentAfter=journal.find(row=>row.scenario===scenario&&row.kind==='native-focusin'&&row.seq>shifted.seq&&row.origin?.id===parent(old).id);assert(parentAfter,'actual native A-parent focus follows consumer outside focus');save({leave,authorizationMarker:watch.marker,consumerOutside:shifted,parentAfter,adapter:'Rust capture/authorization/VDOM/native Eval boundary, not source synchronous request or exact frame alignment'});clearWatch();
 }
 for(const value of ['Enter',' ']){
  await fresh('direct child '+(value===' '?'Space':'Enter')+' keeps original ACK without extra return');await keyboardOpen();const old=a(),begin=performance.now(),result=key(child(old),value);assert(result.event.defaultPrevented,'collapsed direct child key original default prevented');assert(deepActive()===parent(old),'original child ACK focuses parent before dispatch returns');await wait(()=>closed(old),'direct property child closure');await frames(4);assert(exposedJobs(old,begin).length===0&&jobs(old).length===0,'direct property ACK does not schedule controller return jobs');save({key:value,sourceBoundary:'existing immediate child ACK; no fabricated native click or new veto API'});
 }
 await fresh('category-only removal before timer then fresh category');await keyboardOpen();{
  const old=a(),leave=pointer(old,'pointerleave',get('body'));await elapsed(leave,70);control('category-toggle',false);await wait(()=>!a()&&!old.isConnected,'category dropped before timer');get('after').focus();await elapsed(leave,450);await frames(3);assert(deepActive()===get('after')&&!nativeOpen(old),'dropped category has no later focus/native revival');control('category-toggle',true);await wait(()=>readyHost(a())&&closed(a()),'new category protocol registered');await frames(3);assert(a()!==old&&deepActive()===get('after'),'new category is not old queued target');save({leave,lifetime:'owned Rust category-drop guard, not source resource-disposal equivalence'});
 }
 for(const scope of ['category','menu']){
  await fresh(scope+' removal after authorization cancels owned return/new target');await keyboardOpen();const old=a(),oldParent=parent(old);arm(old,()=>{control(scope==='category'?'category-toggle':'mount',false);record('consumer-unmount-after-authorization',{scope,tokens:watch.tokens});});
  const leave=pointer(old,'pointerleave',get('body'));await requireAction();await wait(()=>!old.isConnected&&(scope==='category'?!a():!menu()),'actual owned scope dropped');get('after').focus();await frames(4);assert(deepActive()===get('after')&&!oldParent.isConnected&&!nativeOpen(old),'no detached late return after observed drop');control(scope==='category'?'category-toggle':'mount',true);await wait(()=>ready()&&closed(a())&&closed(b()),'new owner/category registered');await frames(3);assert(parent(a())!==oldParent&&deepActive()===get('after'),'old authorized return cannot focus replacement');save({leave,scope,authorizationMarker:watch.marker,lifetime:'explicit Rust resource/identity adapter; source disconnected callbacks are observational, not generic safety guarantees'});clearWatch();
 }
 for(const consumer of ['none','focus-outside','prevent-default','stop-propagation']){
  await fresh('outside click Window bubble with '+consumer);await keyboardOpen();const host=a(),target=get('after'),begin=performance.now();let reachedWindow=false;
  const event=new MouseEvent('click',{bubbles:true,cancelable:true,composed:true,button:0,buttons:0}),listeners=new AbortController();
  target.addEventListener('click',actual=>{
   if(consumer==='focus-outside')target.focus();
   if(consumer==='prevent-default')actual.preventDefault();
   if(consumer==='stop-propagation')actual.stopPropagation();
   record('consumer-outside-click-target',{consumer,defaultPrevented:actual.defaultPrevented,cancelBubbleDuringDispatch:actual.cancelBubble,focus:focusChain(),mode:mode(host)});
  },{once:true,signal:listeners.signal});
  window.addEventListener('click',actual=>{if(actual!==event)return;reachedWindow=true;record('actual-window-click-bubble',{consumer,trusted:actual.isTrusted,defaultPrevented:actual.defaultPrevented,focus:focusChain()});},{once:true,signal:listeners.signal});
  record('driver-click-only',{consumer,target:node(target),trusted:event.isTrusted,buttons:event.buttons,inputClass:event.constructor.name,classification:'synthetic click only; no mousedown/native focus default'});target.dispatchEvent(event);
  assert(mode(host)==='true','click-only consumer preserves Rust keyboard mode');
  const clickReports=()=>overlayWindowReports(begin,'click',host);
  if(consumer==='stop-propagation'){
   await frames(6);assert(!reachedWindow&&presented(host)&&deepActive()===child(host),'stopped target click never reaches Window and retains popup/inside focus');assert(clickReports().length===0&&exposedJobs(host,begin).length===0,'stopped click sends no outside report or return job');
  }else{
   assert(reachedWindow,'outside click reaches actual Window bubble after target consumer');await wait(()=>clickReports().length>0,'actual shared overlay Window click transport with connected category owner');
   if(consumer==='focus-outside'){
    await wait(()=>closed(host),'target-focused outside closure');await frames(4);assert(deepActive()===target&&exposedJobs(host,begin).length===0&&jobs(host).length===0,'target consumer focus precedes capture; no return authorization or later steal');assert(clickReports().every(row=>!row.detail.active_categories?.includes(Number(host.getAttribute('data-menu-category-root')))),'Window click raw active membership reflects target focus before capture');
   }else{await settleParent(host);assert(exposedJobs(host,begin).length>0,'click-only controller closure authorizes category-parent return');}
   if(consumer==='prevent-default')assert(event.defaultPrevented,'defaultPrevented alone does not block Window outside closure');
  }
  listeners.abort();save({click:begin,consumer,reachedWindow,originalDefaultPrevented:event.defaultPrevented,clickReports:clickReports(),classification:'one-shot target consumer; port raw/commit/authorization/native focus, not source synchronous request/completion'});
 }
 {
  await fresh('click-only B parent closes A with captured parent return');await keyboardOpen();const host=a(),target=parent(b()),begin=performance.now(),listeners=new AbortController();let reachedWindow=false;
  const event=new MouseEvent('click',{bubbles:true,cancelable:true,composed:true,button:0,buttons:0});
  window.addEventListener('click',actual=>{if(actual===event){reachedWindow=true;record('actual-window-click-bubble',{target:node(target)});}},{signal:listeners.signal});
  record('driver-click-only',{target:node(target),trusted:event.isTrusted,buttons:event.buttons,classification:'synthetic B parent click only; no mousedown, no B focus before dispatch',focus:focusChain()});target.dispatchEvent(event);
  const preflight=journal.find(row=>row.scenario===scenario&&row.time>=begin&&row.kind==='raw-private-transport'&&row.detail.kind==='category-click'&&row.detail.category===Number(b().getAttribute('data-menu-category-root')));assert(preflight?.detail.category_parent===true&&preflight.detail.active_categories?.includes(Number(host.getAttribute('data-menu-category-root'))),'actual B click preflight captures A active ancestry before Rust sibling command');assert(!reachedWindow,'category parent stops original click before Window');await settleParent(host);await wait(()=>presented(b()),'B parent click actually presents B');assert(mode(host)==='true','click-only B activation preserves keyboard mode');assert(exposedJobs(host,begin).length>0,'B activation authorizes captured A-parent return');listeners.abort();save({click:begin,reachedWindow,preflight,originalDefaultPrevented:event.defaultPrevented,comparison:'source stopped click closes sibling via closeOtherCategories; port raw preflight and Rust same-VDOM coordinator'});
 }
 for(const priorKeyboardMode of [false,true]){
  await fresh('collapsed Escape document mode then Window close with prior mode '+priorKeyboardMode);
  if(priorKeyboardMode)await keyboardOpen();
  else{pointer(a(),'pointerenter',get('body'));await wait(()=>presented(a()),'pointer-open actual native popup');child(a()).focus();await frames();assert(deepActive()===child(a()),'pointer-mode Escape actual child focus');}
  const host=a(),begin=performance.now();assert(mode(host)===String(priorKeyboardMode),'Rust Escape prior mode observed');const result=key(child(host),'Escape');
  const documentEscapes=()=>journal.filter(row=>row.scenario===scenario&&row.time>=begin&&row.kind==='actual-document-keydown-bubble'&&row.key==='Escape');
  assert(!result.event.defaultPrevented&&documentEscapes().length>0,'Escape keeps original default enabled and reaches actual document bubble');
  const windowKey=()=>overlayWindowReports(begin,'key',host,'Escape')[0];
  await wait(()=>rawModality(begin,'keydown','Escape')&&windowKey(),'document modality then actual Window Escape transport');
  const modality=journal.find(row=>row.scenario===scenario&&row.time>=begin&&row.kind==='raw-private-transport'&&row.detail.entries?.at(-1)?.kind==='keydown'&&row.detail.entries.at(-1).key==='Escape'),windowCapture=windowKey();
  assert(modality.seq<windowCapture.seq,'document raw Escape modality precedes Window controller report');assert(windowCapture.detail.active_categories?.includes(Number(host.getAttribute('data-menu-category-root')))&&windowCapture.focus.at(-1)?.id===child(host).id,'Window Escape captures actual A-child focus and category ancestry');
  await settleParent(host);assert(exposedJobs(host,begin).length>0&&mode(host)==='true','Escape authorizes captured parent return with keyboard mode true regardless prior mode');save({escape:begin,priorKeyboardMode,documentEscapes:documentEscapes(),modality,windowCapture,originalDefaultPrevented:result.event.defaultPrevented,comparison:'measured source anchor-only route: document mode before Window request; source callbacks vs port authorization/commit/native focus are distinct boundaries'});
 }
 mutations.disconnect();globalListeners.abort();clearWatch();
 return{ok:true,observations,journal,input:'synthetic KeyboardEvent/PointerEvent/mousedown/touchstart/click-only MouseEvent plus programmatic focus and public fixture control changes; real Rust/native execution',limits:['not physical/normal hover, touch hardware, AT/Brand/platform or G04.6 nearest-scroll proof','port commits, raw captures, authorization markers and native focus differ from source showChange/showChanged boundaries','after-authorization consumer action uses exposed Rust attr plus firstRAF; only actual child/outside/parent order asserted, no private/exact source frame claim','source canceled-close veto has no matching public Category API and is not faked here','same VDOM Menu coordination is not source Window/cross-VDOM equivalence','owned drop guard is an explicit adapter; no universal stale-work or private resource-disposal certification'],renderer:navigator.userAgent};
}catch(error){return{ok:false,error:String(error)}}
"#;

#[component]
fn PeerMenu() -> Element {
    rsx! {
        div { id: "g04-focus-peer-stage", style: "height:12rem;position:relative;min-width:0",
            Application { start_expanded: false, pinned_menu: true, force_breakpoint: ApplicationBreakpoint::Large,
                menu: rsx! { Menu { label: "Focus peer navigation", application_name: "Peer", start_expanded: false, pinned: true,
                    MenuCategory { label: "Peer category", icon: "rocket", MenuItem { label: "Peer child" } }
                } },
                p { "Separate Menu owner in the same VDOM." }
            }
        }
    }
}

#[component]
fn Probe() -> Element {
    let window = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(90)).await;
            eprintln!("Menu mixed focus native global timeout");
            std::process::exit(1);
        });
    });
    rsx! { IxTheme { scheme: ColorScheme::Dark,
        div { onmounted: move |_| {
            let window = window.clone();
            spawn(async move {
                let query = dom_query::eval(VERIFY);
                let _ = query.send(());
                match query.join::<serde_json::Value>().await {
                    Ok(result) if result["ok"] == true => { println!("Menu mixed focus native checks: {result}"); window.close(); }
                    result => { eprintln!("Menu mixed focus native failure: {result:?}"); std::process::exit(1); }
                }
            });
        }, menu_hover_review::MenuHoverReview {} PeerMenu {} }
    } }
}
