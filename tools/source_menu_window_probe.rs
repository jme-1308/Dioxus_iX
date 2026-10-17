//! Excluded pinned-source Window broadcast/overlay oracle. Independent native
//! Application/Menu groups share one source module and document, not Rust VDOMs.
//! Native events/focus seeding are synthetic; source owns all overlay policy.
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
                .with_title("Pinned source Window menu and overlay contracts")
                .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0, 1000.0))
                .with_visible(true),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}

const VERIFY: &str = r#"
const journal=[],observations=[],requests=[],completions=[],broadcasts=[],modeEvents=[],focusEvents=[];
try {
 await dioxus.recv();
 const revision='8bf599bc68365d7752e12832e71b625883b93d7a',sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const eventIds=new WeakMap(),driverBroadcasts=new WeakSet();
 const globalListeners=new AbortController();let sequence=0,eventSequence=0,generation=0,scenario='bootstrap',observedMode=null,current=null,stage=null;
 const simple=node=>node instanceof Element?{id:node.id,tag:node.tagName}:node===window?{tag:'WINDOW'}:node===document?{tag:'DOCUMENT'}:null;
 const deepActive=()=>{let node=document.activeElement;while(node?.shadowRoot?.activeElement)node=node.shadowRoot.activeElement;return node;};
 const focus=()=>{const chain=[];let node=document.activeElement;while(node){chain.push(simple(node));node=node.shadowRoot?.activeElement;}return chain;};
 const within=(ancestor,target=deepActive())=>{for(let node=target;node;){if(node===ancestor)return true;if(node.assignedSlot){node=node.assignedSlot;continue;}const parent=node.parentNode;node=parent instanceof ShadowRoot?parent.host:parent;}return false;};
 const dropdown=group=>group.category.shadowRoot?.querySelector('ix-dropdown'),parent=group=>group.category.shadowRoot?.querySelector('.category-parent'),child=group=>group.category.querySelector('ix-menu-item');
 const shown=node=>{const rect=node?.getBoundingClientRect();return!!node?.isConnected&&node.show===true&&!!rect?.width&&!!rect?.height;};
 const presented=group=>shown(dropdown(group))&&dropdown(group).classList.contains('show');
 const id=event=>{if(!eventIds.has(event))eventIds.set(event,++eventSequence);return eventIds.get(event);};
 const record=(kind,data={})=>{const row={seq:++sequence,scenario,generation,kind,time:performance.now(),...data};journal.push(row);return row;};
 const groupState=group=>({application:simple(group.application),connected:group.application.isConnected,categoryConnected:group.category.isConnected,expanded:group.menu.expand,dropdown:simple(dropdown(group)),show:dropdown(group)?.show,presented:presented(group),parentId:parent(group)?.id,parentFocused:within(parent(group)),childFocused:within(child(group))});
 const state=()=>({generation,mode:observedMode,a:current?groupState(current.a):null,b:current?groupState(current.b):null,plain:current?.plain?{show:current.plain.show,connected:current.plain.isConnected,trigger:simple(current.plainTrigger),triggerParent:simple(current.plainTrigger.parentElement)}:null,popover:current?.popover?{show:current.popover.show,connected:current.popover.isConnected}:null,focus:focus()});
 const assert=(ok,label)=>{if(!ok)throw Error(label+' '+JSON.stringify({state:state(),tail:journal.slice(-22)}));};
 const wait=async(predicate,label,timeout=4000)=>{const until=performance.now()+timeout;while(performance.now()<until){if(predicate())return;await sleep(5);}throw Error('Timeout '+label+' '+JSON.stringify({state:state(),tail:journal.slice(-22)}));};
 const frames=async(count=3)=>{for(let index=0;index<count;index++)await new Promise(requestAnimationFrame);await sleep(0);};
 const until=async(time,delay)=>{const remaining=time+delay-performance.now();if(remaining>0)await sleep(remaining);};
 document.addEventListener('keyboardmodechange',event=>{if(typeof event.detail?.keyboardMode!=='boolean')return;observedMode=event.detail.keyboardMode;modeEvents.push(record('source-keyboardmodechange',{detail:event.detail,trusted:event.isTrusted,focus:focus()}));},{passive:true,signal:globalListeners.signal});
 document.addEventListener('focusin',event=>focusEvents.push(record('native-focusin',{event:id(event),target:simple(event.target),origin:simple(event.composedPath()[0]),trusted:event.isTrusted,focus:focus()})),{capture:true,passive:true,signal:globalListeners.signal});
 for(const site of ['document-capture','window-capture','window-bubble']){
  const target=site==='document-capture'?document:window,capture=site!=='window-bubble';
  target.addEventListener('closeOtherCategories',event=>{
   const row=record('actual-broadcast-observer',{event:id(event),site,detail:event.detail,target:simple(event.target),currentTarget:simple(event.currentTarget),origin:simple(event.composedPath()[0]),path:event.composedPath().map(simple),driverCreated:driverBroadcasts.has(event),bubbles:event.bubbles,composed:event.composed,cancelable:event.cancelable,defaultPrevented:event.defaultPrevented,cancelBubbleDuringDispatch:event.cancelBubble,trusted:event.isTrusted,focus:focus()});broadcasts.push(row);
   queueMicrotask(()=>{row.defaultPreventedAfterDispatch=event.defaultPrevented;});
  },{capture,passive:true,signal:globalListeners.signal});
 }
 function observeOverlay(ctx,node,name){
  assert(node instanceof HTMLElement,'actual overlay host exists for '+name);
  for(const type of ['showChange','showChanged'])node.addEventListener(type,event=>{
   if(event.composedPath()[0]!==node)return;
   const row=record(type==='showChange'?'source-showChange':'source-showChanged',{event:id(event),generation:ctx.generation,overlay:name,detail:event.detail,showAtCapture:node.show,trusted:event.isTrusted,cancelable:event.cancelable,defaultPreventedAtCapture:event.defaultPrevented,connected:node.isConnected,mode:observedMode,focus:focus()});
   (type==='showChange'?requests:completions).push(row);queueMicrotask(()=>{row.defaultPreventedAfterDispatch=event.defaultPrevented;});
  },{capture:true,passive:true,signal:ctx.abort.signal});
 }
 function makeGroup(name){
  const group=document.createElement('div'),application=document.createElement('ix-application'),menu=document.createElement('ix-menu'),category=document.createElement('ix-menu-category'),body=document.createElement('div');
  group.style.cssText='position:relative;min-width:0;height:512px;border:1px solid #bdc6ce';application.id='window-probe-'+generation+'-'+name+'-application';application.style.cssText='height:100%;width:100%';application.breakpoints=['md'];menu.id='window-probe-'+generation+'-'+name+'-menu';menu.style.cssText='position:relative;height:100%;--ix-menu--width:52px;--ix-menu--expanded-width:256px';menu.pinned=false;menu.startExpanded=false;menu.applicationName='Group '+name.toUpperCase();category.id='window-probe-'+generation+'-'+name;category.label='Group '+name.toUpperCase();
  for(let index=1;index<=2;index++){const item=document.createElement('ix-menu-item');item.id=category.id+'-child-'+index;item.label='Group '+name.toUpperCase()+' child '+index;item.style.setProperty('--ix-menu-item-height','40px');category.append(item);}
  body.id=category.id+'-body';body.textContent='Independent source Application/Menu group '+name.toUpperCase();body.style.cssText='padding:12px';menu.append(category);application.append(menu,body);group.append(application);return{group,application,menu,category,body};
 }
 async function fresh(label){
  current?.abort.abort();stage?.remove();await frames();scenario=label;generation++;
  stage=document.createElement('section');stage.id='window-probe-stage';stage.style.cssText='position:fixed;inset:16px;z-index:1000;background:#fff;color:#202932;padding:16px;box-sizing:border-box;overflow:auto;font:16px/1.4 system-ui';
  const title=document.createElement('p');title.textContent='Pinned source Window oracle: two native source Application/Menu groups; diagnostic geometry only.';
  const outside=document.createElement('button');outside.id='window-probe-outside';outside.textContent='Ordinary outside native target';
  const rack=document.createElement('div');rack.style.cssText='display:flex;gap:16px;min-height:42px;align-items:center';rack.append(outside);
  const grid=document.createElement('div');grid.style.cssText='display:grid;grid-template-columns:1fr 1fr;gap:48px;margin-top:16px';
  const ctx={generation,a:makeGroup('a'),b:makeGroup('b'),outside,rack,abort:new AbortController(),veto:{a:false,b:false},broadcastConsumer:'none'};current=ctx;grid.append(ctx.a.group,ctx.b.group);stage.append(title,rack,grid);document.body.append(stage);
  await Promise.all([ctx.a,ctx.b].flatMap(group=>[group.application,group.menu,group.category,...group.category.querySelectorAll('ix-menu-item')]).map(node=>node.componentOnReady()));
  await Promise.all([ctx.a,ctx.b].map(group=>dropdown(group)?.componentOnReady()));await frames();
  assert(document.querySelectorAll('ix-application').length===2&&document.querySelectorAll('ix-menu').length===2,'exactly two independent source Application/Menu groups');
  for(const [name,group]of[['a',ctx.a],['b',ctx.b]]){
   observeOverlay(ctx,dropdown(group),name);
   group.category.addEventListener('showChange',event=>{if(event.composedPath()[0]!==dropdown(group)||event.detail!==false||!ctx.veto[name])return;event.preventDefault();record('consumer-false-showChange-veto',{event:id(event),overlay:name,defaultPrevented:event.defaultPrevented,focus:focus()});},{signal:ctx.abort.signal});
   const rect=parent(group)?.getBoundingClientRect();assert(group.menu.expand===false&&rect?.width&&rect?.height&&rect.left>=0&&rect.top>=0&&rect.right<=innerWidth&&rect.bottom<=innerHeight,'collapsed actual category anchor within viewport');
  }
  ctx.b.category.addEventListener('closeOtherCategories',event=>{if(event.composedPath()[0]!==ctx.b.category)return;if(ctx.broadcastConsumer==='prevent-default')event.preventDefault();if(ctx.broadcastConsumer==='stop-propagation')event.stopPropagation();record('consumer-broadcast-target',{event:id(event),consumer:ctx.broadcastConsumer,defaultPrevented:event.defaultPrevented,cancelBubbleDuringDispatch:event.cancelBubble});},{signal:ctx.abort.signal});
  raw(ctx,'mousedown');await wait(()=>!presented(ctx.a)&&!presented(ctx.b),'closed initial source groups');record('driver-fresh-native-groups',{generation,topology:'two independent DOM Application/Menu groups, one loaded source module/document',a:groupState(ctx.a),b:groupState(ctx.b)});return ctx;
 }
 function raw(ctx,type,key=null,consumer='none',target=ctx.outside){
  const event=type==='keydown'?new KeyboardEvent(type,{key,code:key,bubbles:true,cancelable:true,composed:true}):new MouseEvent(type,{bubbles:true,cancelable:true,composed:true,button:0,buttons:type==='mousedown'?1:0});let reachedWindow=false,reachedDocument=false;
  const abort=new AbortController();target.addEventListener(type,actual=>{if(actual!==event)return;if(consumer==='prevent-default')actual.preventDefault();if(consumer==='stop-propagation')actual.stopPropagation();record('consumer-native-target',{event:id(event),consumer,defaultPrevented:actual.defaultPrevented,cancelBubbleDuringDispatch:actual.cancelBubble});},{signal:abort.signal});
  document.addEventListener(type,actual=>{if(actual===event){reachedDocument=true;record('actual-document-bubble',{event:id(event),type,key,defaultPrevented:actual.defaultPrevented,mode:observedMode});}},{passive:true,signal:abort.signal});
  window.addEventListener(type,actual=>{if(actual===event){reachedWindow=true;record('actual-window-bubble',{event:id(event),type,key,defaultPrevented:actual.defaultPrevented,mode:observedMode});}},{passive:true,signal:abort.signal});
  const row=record('driver-native-event',{event:id(event),type,key,consumer,target:simple(target),trusted:event.isTrusted,focus:focus()});target.dispatchEvent(event);abort.abort();row.reachedDocument=reachedDocument;row.reachedWindow=reachedWindow;row.defaultPrevented=event.defaultPrevented;row.focusAfterDispatch=focus();return row;
 }
 function keyboard(ctx){const start=modeEvents.length;raw(ctx,'keydown','Tab');assert(modeEvents.slice(start).some(row=>row.detail.keyboardMode===true)&&observedMode===true,'actual source document detector establishes true mode');}
 function pointer(group,type){const event=new PointerEvent(type,{pointerType:'mouse',pointerId:1,isPrimary:true,button:-1,buttons:0,relatedTarget:group.body,bubbles:false,cancelable:false,composed:false});const row=record('driver-category-pointer',{event:id(event),type,target:simple(group.category),relatedTarget:simple(group.body),trusted:event.isTrusted});group.category.dispatchEvent(event);return row;}
 async function open(group){pointer(group,'pointerenter');await wait(()=>presented(group),'source category hover actually presented');await frames();}
 async function seed(group){child(group).focus();await frames();assert(within(child(group)),'actual source deep child focus');record('driver-seeded-child-focus',{category:simple(group.category),focus:focus(),mode:observedMode});}
 const rows=(list,ctx,name,detail)=>list.filter(row=>row.generation===ctx.generation&&row.overlay===name&&row.detail===detail);
 async function settle(){await frames(4);await sleep(25);}
 function save(ctx,extra={}){observations.push({label:scenario,generation:ctx.generation,...extra,state:state(),requests:requests.filter(row=>row.generation===ctx.generation).map(row=>row.seq),completions:completions.filter(row=>row.generation===ctx.generation).map(row=>row.seq)});}
 function broadcast(ctx,detail){const event=new CustomEvent('closeOtherCategories',{detail,bubbles:true,composed:true,cancelable:true});driverBroadcasts.add(event);const row=record('driver-explicit-window-broadcast',{event:id(event),detail,trusted:event.isTrusted,note:'external native CustomEvent; not a source category emission'});window.dispatchEvent(event);return row;}
 async function twoOpen(ctx,focused=false){keyboard(ctx);await open(ctx.a);if(focused)await seed(ctx.a);ctx.broadcastConsumer='stop-propagation';await open(ctx.b);assert(presented(ctx.a)&&presented(ctx.b),'stopped B broadcast leaves two unrelated categories open');}
 async function addPlain(ctx,{nested=false}={}){
  await customElements.whenDefined('ix-dropdown');const trigger=document.createElement('button'),node=document.createElement('ix-dropdown');trigger.id='window-probe-'+ctx.generation+'-plain-trigger';trigger.textContent='Plain Dropdown trigger';node.id='window-probe-'+ctx.generation+'-plain';node.trigger=trigger;node.closeBehavior='both';node.disableFocusHandling=true;node.disableFocusTrap=true;node.suppressTriggerVisibilityCheck=true;node.placement='bottom-start';
  const item=document.createElement('ix-dropdown-item');item.textContent='Plain Dropdown content';node.append(item);if(nested)ctx.a.category.append(trigger);else ctx.rack.append(trigger);stage.append(node);ctx.plain=node;ctx.plainTrigger=trigger;await Promise.all([node,item].map(value=>value.componentOnReady()));await wait(()=>!!trigger.dataset.ixDropdownTrigger,'plain source trigger listener registration');observeOverlay(ctx,node,'plain');await frames();record('driver-public-plain-dropdown-ready',{nestedTrigger:nested,trigger:simple(trigger),triggerParent:simple(trigger.parentElement),publicDiagnosticProps:{disableFocusHandling:true,disableFocusTrap:true,suppressTriggerVisibilityCheck:true}});return node;
 }
 await wait(()=>window.upstreamMenuHoverReview?.snapshot()?.ready,'unchanged cached source fixture ready',20000);assert(window.upstreamMenuHoverReview.snapshot().revision===revision,'fixture source revision');
 document.getElementById('upstream-menu-hover-application').remove();await frames();
 for(const consumer of ['none','prevent-default','stop-propagation']){
  const ctx=await fresh('cross-group source broadcast '+consumer);keyboard(ctx);await open(ctx.a);await seed(ctx.a);ctx.broadcastConsumer=consumer;const before=broadcasts.length;await open(ctx.b);await settle();
  const actual=broadcasts.slice(before).filter(row=>row.origin?.id===ctx.b.category.id&&!row.driverCreated);assert(actual.length>0&&actual.every(row=>row.bubbles&&row.composed&&row.cancelable&&!row.trusted),'actual source broadcast flags observed');
  assert(actual.some(row=>row.site==='window-bubble')===(consumer!=='stop-propagation'),'target stopPropagation controls Window bubble reach');
  if(consumer==='stop-propagation'){assert(presented(ctx.a)&&presented(ctx.b)&&rows(requests,ctx,'a',false).length===0,'stopped broadcast keeps both groups open with no A close request');assert(within(child(ctx.a)),'A child focus retained while B opens');}
  else{await wait(()=>!presented(ctx.a)&&rows(completions,ctx,'a',false).length>0,'A source cross-group close completion');await settle();assert(rows(requests,ctx,'a',false).some(row=>row.mode===true&&row.focus.some(node=>node?.id===child(ctx.a).id)),'source close request captures true mode and actual A child focus');assert(within(parent(ctx.a))&&presented(ctx.b),'captured A child returns A parent while B stays open');if(consumer==='prevent-default')assert(actual.some(row=>row.defaultPreventedAfterDispatch),'original source broadcast default was prevented');}
  save(ctx,{consumer,sourceBroadcastEvents:[...new Set(actual.map(row=>row.event))]});
 }
 {
  const ctx=await fresh('own-ID broadcast preserves pending 250ms grace');keyboard(ctx);await open(ctx.a);await seed(ctx.a);const leave=pointer(ctx.a,'pointerleave');await until(leave.time,70);const own=broadcast(ctx,parent(ctx.a).id);await wait(()=>!presented(ctx.a)&&rows(completions,ctx,'a',false).length>0,'own-ID did not cancel pending close');await settle();const close=rows(requests,ctx,'a',false);assert(close.some(row=>row.time-leave.time>=248)&&close.every(row=>row.time>own.time),'own-ID skips dismissal and pending timeout still requests later close');assert(within(parent(ctx.a)),'own-ID retained keyboard close return');save(ctx,{leave:leave.seq,broadcast:own.seq});
 }
 {
  const ctx=await fresh('foreign-ID close veto clears pending grace despite retained popup');keyboard(ctx);await open(ctx.a);await seed(ctx.a);ctx.veto.a=true;const leave=pointer(ctx.a,'pointerleave');await until(leave.time,70);const foreign=broadcast(ctx,parent(ctx.b).id);await wait(()=>rows(requests,ctx,'a',false).some(row=>row.defaultPreventedAfterDispatch),'foreign broadcast reaches cancelable A close request');await until(leave.time,500);await settle();const close=rows(requests,ctx,'a',false);assert(close.length===1&&presented(ctx.a)&&rows(completions,ctx,'a',false).length===0,'foreign receiver clears timer before one vetoed dismissal; no later grace request');assert(within(parent(ctx.a)),'source queues parent return before false-showChange veto');save(ctx,{leave:leave.seq,broadcast:foreign.seq,veto:true});
 }
 {
  const ctx=await fresh('category removal during pending grace observational');keyboard(ctx);await open(ctx.a);await seed(ctx.a);const leave=pointer(ctx.a,'pointerleave');await until(leave.time,70);ctx.a.category.remove();const removal=record('driver-category-remove',{category:simple(ctx.a.category),focus:focus()});await until(leave.time,500);await settle();assert(!ctx.a.category.isConnected,'actual removed source category remains disconnected');save(ctx,{leave:leave.seq,removal:removal.seq,lateRequestSequences:rows(requests,ctx,'a',false).filter(row=>row.time>removal.time+80).map(row=>row.seq),classification:'disconnectedCallback clears category timer; actual disconnect requests and focus recorded without assuming private safety or counts'});
 }
 {
  const ctx=await fresh('global Escape closes latest unrelated category then earlier category');await twoOpen(ctx);const first=raw(ctx,'keydown','Escape');await wait(()=>!presented(ctx.b)&&presented(ctx.a),'latest B closes while unrelated A remains');await settle();assert(rows(requests,ctx,'b',false).length>0&&rows(requests,ctx,'a',false).length===0,'first Escape requests only actual topmost B');const second=raw(ctx,'keydown','Escape');await wait(()=>!presented(ctx.a),'second Escape closes earlier A');await settle();save(ctx,{first:first.seq,second:second.seq});
 }
 {
  const ctx=await fresh('global Escape false-showChange veto leaves earlier category untouched');await twoOpen(ctx);ctx.veto.b=true;const escape=raw(ctx,'keydown','Escape');await wait(()=>rows(requests,ctx,'b',false).some(row=>row.defaultPreventedAfterDispatch),'topmost B receives vetoed Escape close');await settle();assert(presented(ctx.a)&&presented(ctx.b)&&rows(requests,ctx,'a',false).length===0,'topmost veto does not cascade Escape to earlier unrelated A');save(ctx,{escape:escape.seq,veto:'b'});
 }
 for(const consumer of ['none','prevent-default','stop-propagation']){
  const ctx=await fresh('outside click across two unrelated categories '+consumer);await twoOpen(ctx,true);const click=raw(ctx,'click',null,consumer);assert(observedMode===true,'click-only does not reset keyboard mode');
  if(consumer==='stop-propagation'){await settle();assert(!click.reachedWindow&&presented(ctx.a)&&presented(ctx.b)&&rows(requests,ctx,'a',false).length===0&&rows(requests,ctx,'b',false).length===0,'stopped click leaves both source overlays open');}
  else{assert(click.reachedWindow,'outside click reaches actual Window');await wait(()=>!presented(ctx.a)&&!presented(ctx.b),'Window outside click closes both unrelated overlays');await settle();assert(within(parent(ctx.a)),'outside click captures focused A and returns its parent');}
  save(ctx,{consumer,click:click.seq});
 }
 {
  const ctx=await fresh('outside click closes B but A veto retains popup and late parent focus');await twoOpen(ctx,true);ctx.veto.a=true;const click=raw(ctx,'click');await wait(()=>!presented(ctx.b)&&rows(requests,ctx,'a',false).some(row=>row.defaultPreventedAfterDispatch),'outside click closes B and receives veto from A');await settle();assert(presented(ctx.a)&&within(parent(ctx.a))&&rows(completions,ctx,'a',false).length===0,'A remains open while its pre-veto queued parent return executes');save(ctx,{click:click.seq,veto:'a'});
 }
 {
  const ctx=await fresh('mixed category and unrelated plain Dropdown Escape observation');keyboard(ctx);const plain=await addPlain(ctx);await open(ctx.a);record('driver-public-show-write',{overlay:'plain',value:true,classification:'public property opening; no controller present request is implied'});plain.show=true;await wait(()=>shown(plain)&&presented(ctx.a),'category and unrelated plain Dropdown both presented');await frames();const escape=raw(ctx,'keydown','Escape');await sleep(180);await settle();save(ctx,{escape:escape.seq,classification:'observational first: global latest plain Dropdown versus category; focus handling/trap disabled only on plain diagnostic overlay'});
 }
 {
  const ctx=await fresh('mixed category and unrelated Popover Escape observation');await customElements.whenDefined('ix-popover');const trigger=document.createElement('button'),popover=document.createElement('ix-popover');trigger.textContent='Popover trigger';trigger.id='window-probe-'+ctx.generation+'-popover-trigger';popover.id='window-probe-'+ctx.generation+'-popover';popover.trigger=trigger;popover.closeOnClickOutside=true;popover.textContent='Static noninteractive Popover content';ctx.rack.append(trigger);stage.append(popover);ctx.popover=popover;await popover.componentOnReady();observeOverlay(ctx,popover,'popover');keyboard(ctx);await open(ctx.a);record('driver-public-popover-open',{method:'showPopover',trigger:simple(trigger)});await popover.showPopover();await wait(()=>popover.show===true&&presented(ctx.a),'category and unrelated Popover both presented');await frames();const escape=raw(ctx,'keydown','Escape');await sleep(180);await settle();save(ctx,{escape:escape.seq,classification:'observational first: mixed controller types share source overlay coordinator; no focusable Popover content'});
 }
 for(const relocate of [false,true]){
  const ctx=await fresh('cached dropdown ancestry versus live trigger '+(relocate?'relocation':'retention')+' observation');const plain=await addPlain(ctx,{nested:true});keyboard(ctx);await open(ctx.a);const trigger=ctx.plainTrigger;const triggerClick=raw(ctx,'click',null,'stop-propagation',trigger);await wait(()=>shown(plain)&&presented(ctx.a),'nested-trigger plain Dropdown and category both presented');await frames();
  const before=record('hierarchy-native-before',{trigger:simple(trigger),parent:simple(trigger.parentElement),insideCategory:ctx.a.category.contains(trigger),sourceTriggerMarker:trigger.dataset.ixDropdownTrigger});
  if(relocate){ctx.rack.append(trigger);record('driver-native-trigger-relocate',{trigger:simple(trigger),parent:simple(trigger.parentElement),insideCategory:ctx.a.category.contains(trigger),note:'actual public native trigger node moved after parent/child presentation; no private registry inspection'});await frames();}
  assert(shown(plain)&&presented(ctx.a),'both actual source overlays remain presented immediately before hierarchy probe');const escape=raw(ctx,'keydown','Escape');await sleep(180);await settle();save(ctx,{triggerClick:triggerClick.seq,before:before.seq,relocate,escape:escape.seq,classification:'observational first: dropdown controller uses captured assigned submenu IDs for Escape root; overlay coordinator recomputes ancestry from live trigger containment; no private hierarchy access'});
 }
 current?.abort.abort();globalListeners.abort();
 return{ok:true,revision,caseCount:observations.length,observations,sourceRequests:requests,sourceCompletions:completions,actualBroadcastObservations:broadcasts,sourceModeEvents:modeEvents,nativeFocusEvents:focusEvents,journal,input:'synthetic native PointerEvent/KeyboardEvent/MouseEvent/explicit external CustomEvent and programmatic child.focus; real pinned source categories/controllers/listeners/timers/RAF own policy',limits:['two independent native source Application/Menu DOM groups share one module/document; these are not Rust VDOM roots and establish no Rust parity','all initiating input synthetic; no physical hover/default activation/touch hardware/AT/platform matrix','request captures, post-change completions, broadcast capture/bubble observations and native focus are separate journals; passive observer order can follow reentrant requests and is not emission chronology','explicit own/foreign-ID CustomEvents are driver events, separately labeled from actual source category broadcasts','mixed overlay and trigger-relocation combinations are observations for parent adjudication; passing prerequisites does not assert their final policy or infer private cached hierarchy','plain diagnostic Dropdown disables focus handling/trap and trigger visibility checks; property opening is distinct from controller present requests','diagnostic fixed stage only keeps native anchors within viewport; foundation CSS/system fonts, no Brand geometry or placement parity','timeout/removal observations expose actual source requests/focus without inferring disconnected resource safety; source runtime/cache provenance remains external prerequisite']};
} catch(error) { return {ok:false,error:String(error),observations,journal,sourceRequests:requests,sourceCompletions:completions,actualBroadcastObservations:broadcasts,sourceModeEvents:modeEvents,nativeFocusEvents:focusEvents}; }
"#;

#[component]
fn Probe() -> Element {
    let window = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(120)).await;
            eprintln!("Pinned source menu Window global timeout");
            std::process::exit(1);
        });
    });
    rsx! { div { onmounted: move |_| {
        let window = window.clone();
        spawn(async move {
            let query = dom_query::eval(VERIFY);
            let _ = query.send(());
            match query.join::<serde_json::Value>().await {
                Ok(result) if result["ok"] == true => { println!("Pinned source menu Window checks: {result}"); window.close(); }
                result => { eprintln!("Pinned source menu Window failure: {result:?}"); std::process::exit(1); }
            }
        });
    } } }
}
