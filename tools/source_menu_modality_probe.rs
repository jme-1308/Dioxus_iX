//! Excluded zero-Menu document-modality source oracle. The cached pinned iX
//! runtime owns detection, hover dismissal, and queued focus policy. Inputs and
//! initial child focus are synthetic; this is not physical-input or port parity.
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
                .with_title("Pinned source zero-Menu document modality")
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
 const revision='8bf599bc68365d7752e12832e71b625883b93d7a',prefix='upstream-menu-hover-',get=id=>document.getElementById(prefix+id),sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 // Exact pinned focus-utilities.ts membership; these are driver inputs, never a
 // second detector. All expected changes must arrive from source events.
 const focusKeys=['Tab','ArrowDown','Space','Escape',' ','Shift','Enter','ArrowLeft','ArrowRight','ArrowUp','Home','End'];
 const journal=[],observations=[],modeEvents=[],focusEvents=[],requests=[],completions=[],eventIds=new WeakMap();
 let sequence=0,dispatchSequence=0,scenario='bootstrap',observedMode=null,current=null;
 const listeners=new AbortController();
 const simple=node=>node instanceof Element?{id:node.id,tag:node.tagName}:null;
 const focusChain=()=>{const chain=[];let node=document.activeElement;while(node){chain.push(simple(node));node=node.shadowRoot?.activeElement;}return chain;};
 const deepActive=()=>{let node=document.activeElement;while(node?.shadowRoot?.activeElement)node=node.shadowRoot.activeElement;return node;};
 const within=(ancestor,target=deepActive())=>{for(let node=target;node;){if(node===ancestor)return true;if(node.assignedSlot){node=node.assignedSlot;continue;}const parent=node.parentNode;node=parent instanceof ShadowRoot?parent.host:parent;}return false;};
 const dropdown=ctx=>ctx.a.shadowRoot?.querySelector('ix-dropdown'),parent=ctx=>ctx.a.shadowRoot?.querySelector('.category-parent'),child=ctx=>ctx.a.querySelector('ix-menu-item');
 const presented=ctx=>{const node=dropdown(ctx),box=node?.getBoundingClientRect();return ctx.a.isConnected&&node?.show===true&&node.classList.contains('show')&&!!box?.width&&!!box?.height;};
 const connectedCounts=()=>{const counts={applications:0,menus:0,categories:0},queue=[document];while(queue.length){const root=queue.shift();for(const node of root.querySelectorAll('*')){if(node.isConnected){if(node.tagName==='IX-APPLICATION')counts.applications++;if(node.tagName==='IX-MENU')counts.menus++;if(node.tagName==='IX-MENU-CATEGORY')counts.categories++;}if(node.shadowRoot)queue.push(node.shadowRoot);}}return counts;};
 const record=(kind,data={})=>{const row={seq:++sequence,scenario,kind,time:performance.now(),...data};journal.push(row);return row;};
 const state=()=>({connected:connectedCounts(),observedMode,owner:current?{identity:current.identity,connected:current.application.isConnected,expand:current.menu.expand,presented:presented(current),parentFocused:within(parent(current))}:null,focus:focusChain()});
 const assert=(ok,message)=>{if(!ok)throw Error(message+' '+JSON.stringify({state:state(),tail:journal.slice(-20)}));};
 const wait=async(predicate,label,timeout=3000)=>{const until=performance.now()+timeout;while(performance.now()<until){if(predicate())return;await sleep(5);}throw Error('Timeout '+label+' '+JSON.stringify({state:state(),tail:journal.slice(-20)}));};
 const frames=async(count=3)=>{for(let index=0;index<count;index++)await new Promise(requestAnimationFrame);await sleep(0);};
 const assertGap=()=>{const counts=connectedCounts();assert(counts.applications===0&&counts.menus===0&&counts.categories===0,'actual application detached and zero connected source Menu/category hosts');return counts;};
 document.addEventListener('keyboardmodechange',event=>{assert(typeof event.detail?.keyboardMode==='boolean','source mode event has boolean detail');observedMode=event.detail.keyboardMode;modeEvents.push(record('source-keyboardmodechange',{detail:event.detail,trusted:event.isTrusted,connected:connectedCounts(),focus:focusChain()}));},{passive:true,signal:listeners.signal});
 document.addEventListener('focusin',event=>focusEvents.push(record('native-focusin',{target:simple(event.target),origin:simple(event.composedPath()[0]),trusted:event.isTrusted,focus:focusChain()})),{capture:true,passive:true,signal:listeners.signal});
 function attach(ctx){
  const node=dropdown(ctx);assert(node&&parent(ctx)&&child(ctx),'actual hydrated source dropdown, category parent and child exist');
  for(const type of ['showChange','showChanged'])node.addEventListener(type,event=>{
   if(event.composedPath()[0]!==node)return;let dispatch=eventIds.get(event);if(!dispatch){dispatch=++dispatchSequence;eventIds.set(event,dispatch);}
   const row=record(type==='showChange'?'source-showChange':'source-showChanged',{identity:ctx.identity,dispatch,detail:event.detail,cancelable:event.cancelable,defaultPreventedAtCapture:event.defaultPrevented,trusted:event.isTrusted,connected:ctx.a.isConnected,observedMode,focus:focusChain()});
   (type==='showChange'?requests:completions).push(row);queueMicrotask(()=>{row.defaultPreventedAfterDispatch=event.defaultPrevented;});
  },{capture:true,passive:true,signal:listeners.signal});
 }
 async function ready(ctx){
  await Promise.all([ctx.application,ctx.menu,ctx.a,ctx.b,...ctx.menu.querySelectorAll('ix-menu-item')].map(node=>node.componentOnReady()));
  await Promise.all([ctx.a,ctx.b].map(node=>node.shadowRoot?.querySelector('ix-dropdown')?.componentOnReady()));await frames();
  assert(ctx.application.isConnected&&ctx.menu.isConnected&&ctx.a.isConnected&&ctx.b.isConnected,'source application/Menu/categories connected after actual component readiness');
  await wait(()=>ctx.menu.expand===false&&!presented(ctx),'source owner collapsed with closed popup');
 }
 function input(type,{key=null,consumer='none',gap=true}={}){
  const counts=gap?assertGap():connectedCounts(),before=observedMode,begin=modeEvents.length,target=get('outside'),abort=new AbortController();let reachedDocument=false;
  assert(target?.isConnected,'native outside dispatch target exists');
  const options={bubbles:true,cancelable:true,composed:true};let event;
  if(type==='keydown')event=new KeyboardEvent(type,{...options,key,code:key===' '?'Space':key});
  else if(type==='mousedown')event=new MouseEvent(type,{...options,button:0,buttons:1});
  else if(type==='touchstart')event=typeof TouchEvent==='function'?new TouchEvent(type,options):new Event(type,options);
  else event=new PointerEvent(type,{...options,pointerId:1,pointerType:'mouse',isPrimary:true,button:-1,buttons:0});
  target.addEventListener(type,actual=>{if(actual!==event)return;if(consumer==='stop-propagation')actual.stopPropagation();if(consumer==='prevent-default')actual.preventDefault();record('consumer-input-target',{type,key,consumer,defaultPrevented:actual.defaultPrevented,cancelBubbleDuringDispatch:actual.cancelBubble});},{signal:abort.signal});
  document.addEventListener(type,actual=>{if(actual!==event)return;reachedDocument=true;record('actual-document-input-bubble',{type,key,defaultPrevented:actual.defaultPrevented});},{signal:abort.signal});
  record('driver-native-input',{type,key,consumer,eventClass:event.constructor.name,trusted:event.isTrusted,connected:counts,before});target.dispatchEvent(event);abort.abort();
  const changes=modeEvents.slice(begin),detects=type==='mousedown'||type==='touchstart'||(type==='keydown'&&focusKeys.includes(key)),blocked=consumer==='stop-propagation',expected=detects&&!blocked?(type==='keydown'):before;
  assert(reachedDocument===!blocked,'target propagation controls actual document bubble reach');
  assert(changes.length===(detects&&!blocked?1:0),'only detector-recognized input reaching document emits source keyboardmodechange');
  assert(observedMode===expected,'mode observed from source event agrees with gap input');
  assert(consumer!=='prevent-default'||event.defaultPrevented,'target preventDefault observable on original event');
  const row=record('driver-input-result',{type,key,consumer,before,after:observedMode,reachedDocument,defaultPrevented:event.defaultPrevented,sourceModeEventSequences:changes.map(value=>value.seq),connected:connectedCounts()});
  if(gap)assertGap();return row;
 }
 async function detach(ctx,label){
  scenario=label;assert(ctx.application.isConnected&&!presented(ctx),'remove only a connected owner with closed category');get('outside').focus();await frames();
  ctx.application.remove();await frames();const counts=assertGap();record('driver-application-remove',{identity:ctx.identity,connected:counts,observedMode,method:'actual retained application.remove(); no control click'});
 }
 async function restore(ctx){
  const before=observedMode,modeCount=modeEvents.length;get('mount').append(ctx.application);await ready(ctx);
  assert(observedMode===before&&modeEvents.length===modeCount,'restoring original actual owner has no mode-reset input or source mode change');
  record('driver-original-owner-restore',{identity:ctx.identity,observedMode,connected:connectedCounts(),sameApplication:ctx.application===original.application,sameMenu:ctx.menu===original.menu,sameCategory:ctx.a===original.a});
 }
 function pointer(ctx,type){
  const event=new PointerEvent(type,{pointerId:1,pointerType:'mouse',isPrimary:true,button:-1,buttons:0,relatedTarget:ctx.body,bubbles:false,cancelable:false,composed:false});const row=record('driver-category-pointer',{type,target:simple(ctx.a),relatedTarget:simple(ctx.body),trusted:event.isTrusted,observedMode});ctx.a.dispatchEvent(event);return row;
 }
 async function hoverClose(ctx,label){
  const expectedMode=observedMode,start=requests.length,completionStart=completions.length,modeCount=modeEvents.length;
  assert(typeof expectedMode==='boolean','source mode established before hover cycle');
  pointer(ctx,'pointerenter');await wait(()=>presented(ctx)&&completions.slice(completionStart).some(row=>row.identity===ctx.identity&&row.detail===true),'actual source hover open completion');
  child(ctx).focus();await frames();assert(within(child(ctx))&&!within(parent(ctx)),'actual deep child focus seeded after hover opening');
  assert(observedMode===expectedMode&&modeEvents.length===modeCount,'hover opening and programmatic child focus do not reset source modality');
  record('driver-seeded-child-focus',{identity:ctx.identity,observedMode,focus:focusChain()});const leave=pointer(ctx,'pointerleave');
  await wait(()=>!presented(ctx)&&completions.slice(completionStart).some(row=>row.identity===ctx.identity&&row.detail===false),'actual source delayed hide and close completion');await frames(3);
  const closes=requests.slice(start).filter(row=>row.identity===ctx.identity&&row.detail===false),finished=completions.slice(completionStart).filter(row=>row.identity===ctx.identity&&row.detail===false);
  assert(closes.length>0&&closes.some(row=>row.time-leave.time>=248),'actual source close request follows 250ms grace (2ms clock tolerance)');
  assert(closes.every(row=>row.observedMode===expectedMode),'request-capture mode matches observed gap mode');
  assert(closes.every(row=>row.focus.some(node=>node.id===child(ctx).id)),'source close request captures child focus');
  assert(within(parent(ctx))===expectedMode,'settled source returns category parent exactly for current true keyboard mode');
  assert(observedMode===expectedMode&&modeEvents.length===modeCount,'source closure did not change modality');
  observations.push({label,owner:ctx.identity,expectedMode,leave:leave.seq,closeRequests:closes.map(row=>row.seq),closeCompletions:finished.map(row=>row.seq),settled:state()});
 }
 await wait(()=>window.upstreamMenuHoverReview?.snapshot()?.ready,'unchanged source fixture ready',20000);
 assert(window.upstreamMenuHoverReview.snapshot().revision===revision,'unchanged fixture pins the intended source revision');
 const original={identity:'original',application:get('application'),menu:get('source'),a:get('a'),b:get('b'),body:get('body')};current=original;await ready(original);attach(original);
 assert(connectedCounts().applications===1&&connectedCounts().menus===1&&connectedCounts().categories===2,'initial fixture contains precisely one source application/Menu and two categories');
 input('mousedown',{gap:false});
 await detach(original,'all exact FOCUS_KEYS during zero-Menu gap');
 const exactFocusKeyObservations=[];
 for(const key of focusKeys){input('mousedown');const result=input('keydown',{key});assert(observedMode===true,'exact pinned focus key detected while all source owner nodes disconnected');exactFocusKeyObservations.push(result);}
 await restore(original);await hoverClose(original,scenario);
 for(const mode of [false,true]){
  await detach(original,'pointermove and unrelated keys retain '+mode+' during zero-Menu gap');
  input('mousedown');if(mode)input('keydown',{key:'Tab'});
  for(const type of ['pointermove','keydown']){if(type==='pointermove')input(type);else for(const key of ['a','F1','PageDown'])input(type,{key});}
  assert(observedMode===mode,'pointermove and unrelated keys retain established gap mode');await restore(original);await hoverClose(original,scenario);
 }
 for(const type of ['mousedown','touchstart']){
  await detach(original,type+' resets true mode during zero-Menu gap');input('keydown',{key:'Tab'});input(type);assert(observedMode===false,'gap pointer/touch start resets through source detector');await restore(original);await hoverClose(original,scenario);
 }
 for(const type of ['keydown','mousedown','touchstart'])for(const consumer of ['stop-propagation','prevent-default']){
  await detach(original,type+' '+consumer+' during zero-Menu gap');
  if(type==='keydown')input('mousedown');else input('keydown',{key:'Tab'});
  input(type,{key:type==='keydown'?'Tab':null,consumer});await restore(original);await hoverClose(original,scenario);
 }
 function makeReplacement(identity){
  const application=document.createElement('ix-application'),menu=document.createElement('ix-menu');application.id=prefix+identity+'-application';application.breakpoints=['md'];menu.id=prefix+'source';menu.pinned=false;menu.startExpanded=false;menu.applicationName='Modality replacement';
  const item=(id,label)=>{const node=document.createElement('ix-menu-item');node.id=prefix+identity+'-'+id;node.label=label;return node;};
  const category=name=>{const node=document.createElement('ix-menu-category');node.id=prefix+identity+'-'+name;node.label='Category '+name.toUpperCase();node.append(...[1,2,3].map(index=>item(name+'-'+index,'Child '+index)));return node;};
  const a=category('a'),b=category('b'),body=document.createElement('div');body.className='source-body';body.id=prefix+identity+'-body';body.textContent='Replacement native source application body';menu.append(item('other','Unrelated item'),a,b);application.append(menu,body);return{identity,application,menu,a,b,body};
 }
 for(const mode of [true,false]){
  const previous=current;await detach(previous,'fresh replacement owner inherits gap mode '+mode);input('keydown',{key:'Home'});if(!mode)input('touchstart');
  const replacement=makeReplacement('replacement-'+mode),before=modeEvents.length;current=replacement;get('mount').append(replacement.application);await ready(replacement);attach(replacement);
  assert(observedMode===mode&&modeEvents.length===before,'new source components do not reset singleton initialized by original source owner');
  assert(replacement.application!==original.application&&replacement.menu!==original.menu&&replacement.a!==original.a,'fresh replacement contains actual new source owner nodes');
  assert(connectedCounts().applications===1&&connectedCounts().menus===1&&connectedCounts().categories===2,'replacement is sole connected source owner');
  record('driver-fresh-owner-ready',{identity:replacement.identity,observedMode,connected:connectedCounts(),originalConnected:original.application.isConnected});await hoverClose(replacement,scenario);
 }
 listeners.abort();
 return{ok:true,revision,exactFocusKeys:focusKeys,exactFocusKeyObservations,observations,sourceModeEvents:modeEvents,nativeFocusEvents:focusEvents,sourceRequests:requests,sourceCompletions:completions,journal,input:'synthetic native KeyboardEvent/MouseEvent/TouchEvent-or-Event/PointerEvent objects and programmatic child.focus(); source singleton/event listeners/timers/RAF own policy',limits:['bounded Linux WebView source oracle; no Rust parity or physical keyboard/pointer/touch/AT proof','all source Menu/category hosts and the application removed, but singleton code remains loaded in the same document','mode read only from actual source keyboardmodechange events; no private detector state or fake mode event','document bubble propagation tested at native outside target; separate documents, windows, VDOM roots and global overlays unverified','250ms request grace observed with 2ms clock tolerance; endpoint follows actual completion plus three RAFs, without instrumenting private queued callbacks','unchanged fixture foundation geometry and system fonts; no Brand visual parity; cached runtime provenance is an external prerequisite']};
}catch(error){return{ok:false,error:String(error)}}
"#;

#[component]
fn Probe() -> Element {
    let window = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(90)).await;
            eprintln!("Pinned source menu modality global timeout");
            std::process::exit(1);
        });
    });
    rsx! { div { onmounted: move |_| {
        let window = window.clone();
        spawn(async move {
            let query = dom_query::eval(VERIFY);
            let _ = query.send(());
            match query.join::<serde_json::Value>().await {
                Ok(result) if result["ok"] == true => { println!("Pinned source menu modality checks: {result}"); window.close(); }
                result => { eprintln!("Pinned source menu modality failure: {result:?}"); std::process::exit(1); }
            }
        });
    } } }
}
