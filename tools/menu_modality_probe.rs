//! Excluded Linux WebView detector-lifetime replay. The pinned source contract
//! lives in utils/focus/detect-keyboard-mode.ts, focus-utilities.ts, setup.mixin.ts,
//! and MenuCategory.onDropdownShowChange. Rust owns every fixture lifetime.
//! Synthetic raw native events and native focus exercise policy; they do not
//! establish physical-input, cross-window, or source callback timing parity.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::time::Duration;

#[path = "../src/dom_query.rs"]
mod dom_query;

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("iX document modality lifetime probe")
                .with_inner_size(dioxus::desktop::LogicalSize::new(1100.0, 1000.0))
                .with_visible(true),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}

const VERIFY: &str = r#"
let listeners=null,mutations=null;
try{
 await dioxus.recv();
 const get=id=>document.getElementById('modality-'+id),sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const journal=[],observations=[],eventIds=new WeakMap();let scenario='bootstrap',sequence=0,eventSequence=0;
 listeners=new AbortController();
 const host=which=>get(which)?.querySelector('.ix-menu-category'),primary=()=>host('primary'),peer=()=>host('peer');
 const parent=h=>h?.querySelector(':scope > [role=menuitem]'),child=h=>h?.querySelector('[data-menu-category-child]'),popup=h=>h?.querySelector(':scope > [data-menu-category-popup]');
 const mode=h=>h?.getAttribute('data-menu-category-keyboard-mode'),open=h=>h?.getAttribute('data-menu-category-popup-open')==='true';
 const nativeOpen=h=>{try{return!!popup(h)?.matches(':popover-open');}catch{return false;}};
 const presented=h=>{const p=popup(h),r=p?.getBoundingClientRect();return!!h?.isConnected&&open(h)&&!!p&&!p.hidden&&!!r?.width&&!!r?.height&&nativeOpen(h);};
 const closed=h=>!!h?.isConnected&&!open(h)&&!nativeOpen(h);
 const describe=n=>n?{id:n.id,tag:n.tagName,label:n.getAttribute?.('aria-label')}:null;
 const active=()=>{let n=document.activeElement;while(n?.shadowRoot?.activeElement)n=n.shadowRoot.activeElement;return n;};
 const state=h=>({host:describe(h),connected:!!h?.isConnected,mode:mode(h),open:open(h),nativeOpen:nativeOpen(h),parent:describe(parent(h)),child:describe(child(h)),parentFocused:!!h&&active()===parent(h)});
 const snapshot=()=>({menus:document.querySelectorAll('nav.ix-menu').length,primary:state(primary()),peer:state(peer()),focus:describe(active())});
 const record=(kind,data={})=>{const row={seq:++sequence,scenario,kind,time:performance.now(),...data};journal.push(row);if(journal.length>1800)journal.shift();return row;};
 const assert=(ok,label)=>{if(!ok)throw Error(label+' '+JSON.stringify({scenario,snapshot:snapshot(),tail:journal.slice(-18)}));};
 const wait=async(predicate,label,timeout=5000)=>{const until=performance.now()+timeout;while(performance.now()<until){if(predicate())return;await sleep(5);}throw Error('Timeout '+label+' '+JSON.stringify({scenario,snapshot:snapshot(),tail:journal.slice(-18)}));};
 const frames=async(n=3)=>{for(let i=0;i<n;i++)await new Promise(requestAnimationFrame);await sleep(0);};
 const readyHost=h=>h?.dataset.categoryNative==='true'&&h?.dataset.categoryHoverNative==='true'&&['true','false'].includes(mode(h));
 const ready=()=>document.querySelectorAll('nav.ix-menu').length===2&&readyHost(primary())&&readyHost(peer())&&[...document.querySelectorAll('nav.ix-menu')].every(n=>n.dataset.menuNative==='true'&&n.querySelector('input[data-menu-hover-observer]')?.dataset.menuDocumentReady==='true');
 const modes=value=>mode(primary())===String(value)&&mode(peer())===String(value);
 const eventId=e=>{if(!eventIds.has(e))eventIds.set(e,++eventSequence);return eventIds.get(e);};
 for(const type of ['keydown','mousedown','touchstart','pointermove'])document.addEventListener(type,e=>record('actual-document-bubble',{event:eventId(e),type,key:e.key??null,trusted:e.isTrusted,defaultPrevented:e.defaultPrevented,menus:document.querySelectorAll('nav.ix-menu').length}),{passive:true,signal:listeners.signal});
 document.addEventListener('focusin',e=>record('native-focusin',{origin:describe(e.composedPath()[0]),trusted:e.isTrusted}),{capture:true,passive:true,signal:listeners.signal});
 mutations=new MutationObserver(rows=>{for(const row of rows)record('rendered-state-commit',{attribute:row.attributeName,previous:row.oldValue,value:row.target.getAttribute(row.attributeName),category:row.target.getAttribute('data-menu-category-root')});});
 mutations.observe(document.body,{subtree:true,attributes:true,attributeOldValue:true,attributeFilter:['data-menu-category-keyboard-mode','data-menu-category-popup-open']});
 function inputControl(which){
  const control=get(which);assert(control instanceof HTMLInputElement&&control.isConnected,'retained native Rust fixture input '+which);
  control.value=String(++eventSequence);record('driver-rust-fixture-input',{which,classification:'input only; no click/mousedown/touchstart/focus'});control.dispatchEvent(new Event('input',{bubbles:true,composed:true}));
 }
 function emit(type,key=null,consumer='none'){
  const target=get('outside');let event;
  if(type==='keydown')event=new KeyboardEvent(type,{key,code:key,bubbles:true,composed:true,cancelable:true});
  else if(type==='mousedown')event=new MouseEvent(type,{bubbles:true,composed:true,cancelable:true,button:0,buttons:1});
  else if(type==='touchstart')event=typeof TouchEvent==='function'?new TouchEvent(type,{bubbles:true,composed:true,cancelable:true}):new Event(type,{bubbles:true,composed:true,cancelable:true});
  else event=new PointerEvent(type,{bubbles:true,composed:true,cancelable:true,pointerType:'mouse',pointerId:1,isPrimary:true,button:-1,buttons:0});
  const id=eventId(event);let targetReached=false;
  const action=e=>{if(e!==event)return;targetReached=true;if(consumer==='stop-propagation')e.stopPropagation();if(consumer==='prevent-default')e.preventDefault();record('target-consumer',{event:id,consumer,defaultPrevented:e.defaultPrevented,cancelBubble:e.cancelBubble});};
  target.addEventListener(type,action,{once:true});
  const row=record('driver-raw-event',{event:id,type,key,consumer,trusted:event.isTrusted,eventClass:event.constructor.name,menus:document.querySelectorAll('nav.ix-menu').length});
  target.dispatchEvent(event);target.removeEventListener(type,action);
  const bubbles=journal.filter(r=>r.kind==='actual-document-bubble'&&r.event===id);
  assert(targetReached,'ordinary outside target consumer ran');
  assert(bubbles.length===(consumer==='stop-propagation'?0:1),'exact document bubble delivery obeys stopPropagation');
  assert(event.defaultPrevented===(consumer==='prevent-default'),'only configured preventDefault cancels original raw event');
  record('driver-raw-result',{event:id,bubbleCount:bubbles.length,defaultPrevented:event.defaultPrevented});
  return{...row,bubbleCount:bubbles.length,defaultPrevented:event.defaultPrevented};
 }
 function pointer(h,type,relatedTarget){
  const event=new PointerEvent(type,{pointerType:'mouse',pointerId:1,isPrimary:true,button:-1,buttons:0,relatedTarget,bubbles:false,composed:false,cancelable:false});
  const row=record('driver-hover',{type,target:describe(h),relatedTarget:describe(relatedTarget),trusted:event.isTrusted});h.dispatchEvent(event);return row;
 }
 async function hoverReturn(expected){
  const h=primary(),trigger=parent(h);assert(closed(h)&&closed(peer()),'both fresh hover baselines closed');
  const enter=pointer(h,'pointerenter',get('outside'));await wait(()=>presented(h),'native primary hover popup actually presented');
  const item=child(h);assert(item instanceof HTMLElement,'native category child exists');item.focus();await frames(2);assert(active()===item,'programmatic native child focus actually established');assert(modes(expected),'hover and native focus preserve observed modality');
  const seeded=record('seeded-native-child-focus',{focus:describe(active()),mode:mode(h)}),leave=pointer(h,'pointerleave',get('outside'));
  await frames(1);assert(presented(h),'leave retains presented popup during grace');
  await wait(()=>closed(h),'250ms hover leave closes actual popup');
  if(expected)await wait(()=>active()===trigger,'true modality returns native focus to category parent');
  await frames(6);
  assert((active()===trigger)===expected,'native parent return iff rendered keyboard mode is true');assert(modes(expected),'closure preserves both owner modes');
  const close=journal.find(r=>r.scenario===scenario&&r.kind==='rendered-state-commit'&&r.attribute==='data-menu-category-popup-open'&&r.category===h.getAttribute('data-menu-category-root')&&r.value==='false'&&r.time>=leave.time);
  assert(close&&close.time-leave.time>=248,'native popup close commit follows 250ms grace with2ms clock tolerance');
  const returns=journal.filter(r=>r.scenario===scenario&&r.kind==='native-focusin'&&r.time>=leave.time&&r.origin?.id===trigger.id);
  assert(expected?returns.length>0:returns.length===0,'native focusin independently confirms parent-return decision');
  return{enter,seeded,leave,closeCommitDelta:close.time-leave.time,parentReturn:active()===trigger,parentFocusEvents:returns,focusAfter:describe(active()),timingBoundary:'VDOM popup close commit/native focus; not source synchronous showChange/showChanged or exact two-RAF alignment'};
 }
 const cases=[];
 for(const key of ['Tab','ArrowDown','Space','Escape',' ','Shift','Enter','ArrowLeft','ArrowRight','ArrowUp','Home','End'])cases.push({label:'exact focus key '+JSON.stringify(key),type:'keydown',key,before:false,expected:true});
 for(const type of ['mousedown','touchstart'])cases.push({label:type+' resets true',type,before:true,expected:false});
 for(const before of [false,true]){
  cases.push({label:'pointermove preserves '+before,type:'pointermove',before,expected:before});
  for(const key of ['Spacebar','tab','PageDown','PageUp','a','F6','Unidentified'])cases.push({label:'unmatched '+JSON.stringify(key)+' preserves '+before,type:'keydown',key,before,expected:before});
 }
 for(const consumer of ['stop-propagation','prevent-default']){
  cases.push({label:consumer+' Tab',type:'keydown',key:'Tab',before:false,expected:consumer==='prevent-default',consumer});
  for(const type of ['mousedown','touchstart'])cases.push({label:consumer+' '+type,type,before:true,expected:consumer==='stop-propagation',consumer});
 }
 await wait(ready,'initial two collapsed Menu owners/native protocol');
 assert(closed(primary())&&closed(peer()),'initial collapsed fixture');
 for(const phase of ['mounted','all-menus-gap'])for(const test of cases){
  scenario=phase+': '+test.label;
  const seed=emit(test.before?'keydown':'mousedown',test.before?'Tab':null);await wait(()=>modes(test.before),'observed initial rendered mode in both owners');await frames(2);
  const oldPrimary=primary(),oldPeer=peer(),gapStart=performance.now();let zeroMenus=null;
  if(phase==='all-menus-gap'){
   inputControl('remove');await wait(()=>document.querySelectorAll('nav.ix-menu').length===0&&!oldPrimary.isConnected&&!oldPeer.isConnected,'ALL Menu owners actually removed by Rust input');await frames(3);
   assert(get('outside').isConnected&&get('remove').isConnected&&get('restore').isConnected,'ordinary target and Rust controls persist through zero-menu gap');
   zeroMenus=record('observed-all-menus-gap',{menus:document.querySelectorAll('nav.ix-menu').length,oldPrimaryConnected:oldPrimary.isConnected,oldPeerConnected:oldPeer.isConnected});
  }
  const stimulus=emit(test.type,test.key??null,test.consumer??'none');await frames(4);
  if(phase==='all-menus-gap'){
   assert(document.querySelectorAll('nav.ix-menu').length===0,'stimulus and async delivery complete while zero Menus connected');
   inputControl('restore');await wait(()=>ready()&&primary()!==oldPrimary&&peer()!==oldPeer,'fresh Rust Menu owners actually remounted');
  }
  await wait(()=>modes(test.expected),'actual rendered mode equals detector contract');await frames(3);assert(modes(test.expected),'both owners retain expected mode after paint');
  const setupRawEvents=journal.filter(r=>r.scenario===scenario&&r.kind==='actual-document-bubble'&&r.time>=gapStart);
  assert(setupRawEvents.length===(test.consumer==='stop-propagation'?0:1),'no fixture control click/down/key can mask stimulus between seed and rendered mode');
  if(phase==='all-menus-gap'&&setupRawEvents.length)assert(setupRawEvents[0].menus===0,'actual document bubble observes stimulus while ALL Menus absent');
  const rendered=record('rendered-mode-after-stimulus',{expected:test.expected,snapshot:snapshot()});
  const hover=await hoverReturn(test.expected);
  observations.push({...test,label:scenario,phase,seed,zeroMenus,stimulus,rendered,hover,snapshot:snapshot()});
 }
 for(const burst of [
  {label:'Tab then unmatched x then pointermove',events:[['keydown','Tab'],['keydown','x'],['pointermove',null]],expected:true},
  {label:'Tab then touchstart then unmatched x',events:[['keydown','Tab'],['touchstart',null],['keydown','x']],expected:false}
 ]){
  scenario='rapid-all-menus-gap: '+burst.label;
  const seed=emit('mousedown');await wait(()=>modes(false),'both owners observe false before rapid gap');await frames(2);
  const oldPrimary=primary(),oldPeer=peer();inputControl('remove');
  await wait(()=>document.querySelectorAll('nav.ix-menu').length===0&&!oldPrimary.isConnected&&!oldPeer.isConnected,'all owners removed before rapid burst');await frames(3);
  assert(get('outside').isConnected&&get('remove').isConnected&&get('restore').isConnected,'retained controls and ordinary target exist during rapid gap');
  const zeroMenus=record('observed-all-menus-gap',{menus:document.querySelectorAll('nav.ix-menu').length,oldPrimaryConnected:oldPrimary.isConnected,oldPeerConnected:oldPeer.isConnected});
  // Deliberately synchronous: no await/RAF between raw stimuli or before the
  // public Rust restore input. Only native events and fixture state are driven.
  const stimuli=burst.events.map(([type,key])=>emit(type,key));inputControl('restore');
  const restored=record('immediate-restore-input-after-burst',{events:stimuli.map(row=>row.event),boundary:'same JS task; no intervening await/RAF'});
  await wait(()=>ready()&&primary()!==oldPrimary&&peer()!==oldPeer,'fresh owners report document-ready after immediate burst restore');
  await wait(()=>modes(burst.expected),'fresh owners render final detector mode after rapid gap burst');await frames(3);assert(modes(burst.expected),'rapid burst final mode persists in both owners');
  const rawEvents=journal.filter(row=>row.scenario===scenario&&row.kind==='actual-document-bubble'&&row.time>=zeroMenus.time);
  assert(rawEvents.length===burst.events.length&&rawEvents.every((row,index)=>row.event===stimuli[index].event&&row.menus===0),'all rapid stimuli bubble in order while zero Menus exist with no masking key/down/move');
  const rendered=record('rendered-mode-after-burst',{expected:burst.expected,snapshot:snapshot()});
  const hover=await hoverReturn(burst.expected);
  observations.push({label:scenario,phase:'rapid-all-menus-gap',expected:burst.expected,seed,zeroMenus,stimuli,restored,rendered,hover,snapshot:snapshot(),boundary:'observed burst/fresh-owner endpoints; IPC delivery order is not forced'});
 }
 return{ok:true,caseCount:observations.length,observations,journal,renderer:navigator.userAgent,input:'synthetic raw document-bubble KeyboardEvent/MouseEvent/TouchEvent (generic Event fallback)/PointerEvent, native HTMLElement.focus, ordinary Rust state-only input fixture controls',limits:['source proof must be established independently against pinned upstream before implementation parity is claimed','not physical keyboard/mouse/touch, normal native default actions, AT, Brand visual, web-browser, or cross-window/cross-VDOM proof','two owners share one VDOM/document; zero connected Menus preserves the root fixture/VDOM, not a destroyed renderer','rendered keyboard-mode and native hover/focus decisions are public observations; no production private transport/listener lifetime introspection','async waits allow Rust event delivery/VDOM/native Eval adaptation; source synchronous detector and showChange callbacks are distinct evidence','rapid burst cases observe final rendered mode/native focus endpoints after immediate same-task restore, not forced IPC reordering; backlog merging requires independent unit evidence','synthetic touchstart may use Event fallback where TouchEvent construction is unavailable; eventClass is recorded per stimulus']};
}catch(error){return{ok:false,error:String(error)}}finally{mutations?.disconnect();listeners?.abort();}
"#;

#[component]
fn Owner(which: &'static str) -> Element {
    rsx! {
        div { id: "modality-{which}", style: "position:relative;height:22rem;min-width:0",
            Application {
                start_expanded: false,
                pinned_menu: true,
                force_breakpoint: ApplicationBreakpoint::Large,
                menu: rsx! {
                    Menu { label: "{which} navigation", application_name: "Modality probe", start_expanded: false, pinned: true,
                        MenuCategory { label: "{which} category", icon: "folder",
                            MenuItem { label: "{which} child" }
                        }
                    }
                },
                p { "{which} Menu owner" }
            }
        }
    }
}

#[component]
fn Probe() -> Element {
    let mut mounted = use_signal(|| true);
    let window = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(180)).await;
            eprintln!("Menu document modality native global timeout");
            std::process::exit(1);
        });
    });
    rsx! {
        IxTheme { scheme: ColorScheme::Dark,
            div {
                onmounted: move |_| {
                    let window = window.clone();
                    spawn(async move {
                        let query = dom_query::eval(VERIFY);
                        let _ = query.send(());
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Menu document modality native checks: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Menu document modality native failure: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                },
                input { id: "modality-remove", r#type: "hidden", oninput: move |_| mounted.set(false) }
                input { id: "modality-restore", r#type: "hidden", oninput: move |_| mounted.set(true) }
                button { id: "modality-outside", r#type: "button", "Ordinary outside native event target" }
                if mounted() {
                    Owner { which: "primary" }
                    Owner { which: "peer" }
                }
            }
        }
    }
}
