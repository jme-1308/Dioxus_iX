//! Excluded pinned-source Popover controller oracle. Public component methods,
//! properties and synthetic Escape drive real cached upstream components.
//! Source owns controller policy, native top-layer state and focus.
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
                .with_title("Pinned source Popover controller contracts")
                .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0, 1000.0))
                .with_visible(true),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}

const VERIFY: &str = r#"
const journal=[],observations=[],requests=[],completions=[],assignments=[],focusEvents=[];
try {
 await dioxus.recv();
 const revision='8bf599bc68365d7752e12832e71b625883b93d7a',sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const globalAbort=new AbortController(),eventIds=new WeakMap();let seq=0,eventSeq=0,generation=0,scenario='bootstrap',current=null,stage=null;
 const simple=node=>node instanceof Element?{id:node.id,tag:node.tagName}:null;
 const deepActive=()=>{let node=document.activeElement;while(node?.shadowRoot?.activeElement)node=node.shadowRoot.activeElement;return node;};
 const focus=()=>{const chain=[];let node=document.activeElement;while(node){chain.push(simple(node));node=node.shadowRoot?.activeElement;}return chain;};
 const record=(kind,data={})=>{const row={seq:++seq,generation,scenario,kind,time:performance.now(),...data};journal.push(row);return row;};
 const eventId=event=>{if(!eventIds.has(event))eventIds.set(event,++eventSeq);return eventIds.get(event);};
 const native=node=>{const dialog=node.shadowRoot?.querySelector('dialog');let popoverOpen=false;try{popoverOpen=!!dialog?.matches(':popover-open');}catch{}const rect=dialog?.getBoundingClientRect();return{show:node.show,connected:node.isConnected,dialog:dialog?{id:dialog.id,popover:dialog.getAttribute('popover'),popoverOpen,open:dialog.open,rect:rect?{x:rect.x,y:rect.y,width:rect.width,height:rect.height}:null}:null};};
 const state=()=>({generation,overlays:current?Object.fromEntries(Object.entries(current.nodes).map(([name,node])=>[name,native(node)])):null,focus:focus()});
 const assert=(ok,label)=>{if(!ok)throw Error(label+' '+JSON.stringify({state:state(),tail:journal.slice(-28)}));};
 const wait=async(predicate,label,timeout=5000)=>{const end=performance.now()+timeout;while(performance.now()<end){if(predicate())return;await sleep(5);}throw Error('Timeout '+label+' '+JSON.stringify({state:state(),tail:journal.slice(-28)}));};
 const frames=async(count=4)=>{for(let i=0;i<count;i++)await new Promise(requestAnimationFrame);await sleep(0);};
 const settle=async()=>{await frames();await sleep(25);};
 const rows=(list,ctx,name,detail)=>list.filter(row=>row.generation===ctx.generation&&row.overlay===name&&row.detail===detail);
 const since=(list,ctx,after)=>list.filter(row=>row.generation===ctx.generation&&row.seq>after);
 const presented=(ctx,name)=>{const value=native(ctx.nodes[name]);return value.show===true&&value.dialog?.popoverOpen===true;};
 const save=(ctx,extra={})=>observations.push({label:scenario,generation:ctx.generation,...extra,state:state(),requests:requests.filter(row=>row.generation===ctx.generation).map(row=>row.seq),completions:completions.filter(row=>row.generation===ctx.generation).map(row=>row.seq)});
 document.addEventListener('focusin',event=>focusEvents.push(record('native-focusin',{event:eventId(event),target:simple(event.target),origin:simple(event.composedPath()[0]),trusted:event.isTrusted,focus:focus()})),{capture:true,passive:true,signal:globalAbort.signal});
 function make(ctx,name,trigger){
  const node=document.createElement('ix-popover'),content=document.createElement('div'),button=document.createElement('button');
  node.id='popover-controller-'+generation+'-'+name;node.trigger=trigger;node.closeOnClickOutside=true;node.placement='bottom';content.style.cssText='padding:16px;background:white;color:#202932;min-width:180px';button.id=node.id+'-focus';button.textContent=name+' content action';content.append(button);node.append(content);ctx.nodes[name]=node;ctx.contents[name]=content;ctx.buttons[name]=button;ctx.triggers[name]=trigger;
  for(const type of ['showChange','showChanged'])node.addEventListener(type,event=>{if(event.composedPath()[0]!==node)return;const row=record(type==='showChange'?'source-showChange':'source-showChanged',{event:eventId(event),overlay:name,detail:event.detail,showAtCapture:node.show,nativeAtCapture:native(node),focus:focus(),trusted:event.isTrusted,cancelable:event.cancelable,defaultPreventedAtCapture:event.defaultPrevented});(type==='showChange'?requests:completions).push(row);queueMicrotask(()=>row.defaultPreventedAfterDispatch=event.defaultPrevented);},{capture:true,passive:true,signal:ctx.abort.signal});
  node.addEventListener('showChange',event=>{if(event.composedPath()[0]!==node||!ctx.veto[name]?.[String(event.detail)])return;event.preventDefault();record('consumer-showChange-veto',{overlay:name,event:eventId(event),detail:event.detail,defaultPrevented:event.defaultPrevented,show:node.show,focus:focus()});},{signal:ctx.abort.signal});
  node.addEventListener('ix-assign-sub-popover',event=>{assignments.push(record('source-child-assignment-capture',{overlay:name,event:eventId(event),detail:event.detail,origin:simple(event.composedPath()[0]),target:simple(event.target),bubbles:event.bubbles,composed:event.composed,cancelable:event.cancelable,trusted:event.isTrusted}));},{capture:true,passive:true,signal:ctx.abort.signal});
  return node;
 }
 async function fresh(label,{nestedHosts=false}={}){
  current?.abort.abort();stage?.remove();await frames();generation++;scenario=label;
  stage=document.createElement('section');stage.id='popover-controller-stage';stage.style.cssText='position:fixed;inset:16px;z-index:1000;padding:24px;overflow:auto;background:#eef1f4;color:#202932;font:16px/1.4 system-ui';
  const title=document.createElement('p');title.textContent='Pinned source Popover controller: public methods and actual native top layer. Diagnostic layout.';
  const rack=document.createElement('div');rack.style.cssText='display:flex;gap:40px;align-items:center;min-height:60px';
  const ctx={generation,nestedHosts,nodes:{},triggers:{},contents:{},buttons:{},abort:new AbortController(),veto:{}};current=ctx;
  const trigger=name=>{const el=document.createElement('button');el.id='popover-controller-'+generation+'-'+name+'-trigger';el.textContent='Open '+name;return el;};
  const pt=trigger('parent'),ct=trigger('child'),st=trigger('sibling'),ut=trigger('unrelated'),outside=trigger('outside');ctx.outside=outside;rack.append(pt,ut,outside);
  const parent=make(ctx,'parent',pt);ctx.contents.parent.append(ct,st);const child=make(ctx,'child',ct),sibling=make(ctx,'sibling',st),unrelated=make(ctx,'unrelated',ut);
  if(nestedHosts){ctx.contents.parent.append(child,sibling);stage.append(title,rack,parent,unrelated);}else{stage.append(title,rack,parent,child,sibling,unrelated);}document.body.append(stage);
  await Promise.all(Object.values(ctx.nodes).map(node=>node.componentOnReady()));
  await wait(()=>Object.values(ctx.triggers).every(node=>node.hasAttribute('data-ix-popover-trigger')),'all actual source trigger registration markers');await frames();
  assert(assignments.filter(row=>row.generation===generation&&row.overlay==='parent').some(row=>row.detail===child.id),'actual child registration event reaches source parent');assert(assignments.filter(row=>row.generation===generation&&row.overlay==='parent').some(row=>row.detail===sibling.id),'actual sibling registration event reaches source parent');
  assert(Object.values(ctx.nodes).every(node=>node.show===false&&native(node).dialog?.popover=== 'manual'),'all four source hosts initially closed native manual popovers');record('driver-fresh-source-hosts',{topology:nestedHosts?'parent with child and sibling triggers AND hosts nested in parent light content; unrelated host is stage sibling':'parent with child and sibling triggers in light content; child/sibling/unrelated hosts are stage siblings; source assignment events, no fabricated private hierarchy',state:state()});return ctx;
 }
 async function call(ctx,name,method){const node=ctx.nodes[name],start=record('driver-public-method-call',{overlay:name,method,show:node.show,focus:focus()});await node[method]();const end=record('driver-public-method-promise-resolved',{overlay:name,method,show:node.show,native:native(node),focus:focus(),completionSequencesSinceCall:since(completions,ctx,start.seq).map(row=>row.seq)});return{start:start.seq,end:end.seq};}
 async function open(ctx,name){await call(ctx,name,'showPopover');await wait(()=>presented(ctx,name)&&rows(completions,ctx,name,true).length>0,name+' real open and separate RAF completion');await settle();}
 async function pair(ctx){await open(ctx,'parent');await open(ctx,'child');assert(presented(ctx,'parent')&&presented(ctx,'child'),'source nested pair concurrently presented');}
 function escape(ctx){const event=new KeyboardEvent('keydown',{key:'Escape',code:'Escape',bubbles:true,composed:true,cancelable:true});const row=record('driver-native-Escape',{target:simple(ctx.outside),trusted:event.isTrusted,focus:focus()});ctx.outside.dispatchEvent(event);row.defaultPreventedAfterDispatch=event.defaultPrevented;return row;}
 await wait(()=>window.upstreamMenuHoverReview?.snapshot()?.ready,'unchanged cached source fixture ready',20000);assert(window.upstreamMenuHoverReview.snapshot().revision===revision,'fixture revision matches pin');document.getElementById('upstream-menu-hover-application').remove();await frames();await customElements.whenDefined('ix-popover');
 {
  const ctx=await fresh('public show promise and separate RAF completion');const result=await call(ctx,'parent','showPopover');assert(presented(ctx,'parent'),'showPopover promise resolves with actual native popover presented');await wait(()=>rows(completions,ctx,'parent',true).length===1,'separate true completion');await settle();assert(rows(requests,ctx,'parent',true).length===1,'one accepted public opening request');save(ctx,{call:result,classification:'promise resolution and showChanged RAF are separately observed; promise is not a completion-event contract'});
 }
 {
  const ctx=await fresh('already-present show is request-free');await open(ctx,'parent');const before=seq;const result=await call(ctx,'parent','showPopover');await settle();assert(since(requests,ctx,before).length===0&&since(completions,ctx,before).length===0&&presented(ctx,'parent'),'already-present public show emits no duplicate request or completion');save(ctx,{call:result});
 }
 {
  const ctx=await fresh('already-closed hide still requests false without completion');const before=seq;const result=await call(ctx,'parent','hidePopover');await settle();const actual=since(requests,ctx,before);assert(actual.length===1&&actual[0].overlay==='parent'&&actual[0].detail===false&&actual[0].showAtCapture===false&&since(completions,ctx,before).length===0,'closed host still invokes willDismiss false then exits');save(ctx,{call:result});
 }
 {
  const ctx=await fresh('parent public hide requests parent then cached child then closes both');await pair(ctx);const before=seq;const result=await call(ctx,'parent','hidePopover');await wait(()=>!presented(ctx,'parent')&&!presented(ctx,'child'),'parent and child native closed');await settle();const actual=since(requests,ctx,before);assert(actual[0]?.overlay==='parent'&&actual[0]?.detail===false&&actual[1]?.overlay==='child'&&actual[1]?.detail===false,'parent request precedes cached child request');assert(actual.some(row=>row.overlay==='sibling'&&row.detail===false&&row.showAtCapture===false),'closed cached sibling is still requested during cascade');save(ctx,{call:result,classification:'request order asserted; completion order and final native focus recorded separately'});
 }
 {
  const ctx=await fresh('parent false veto prevents cached child dismissal requests');await pair(ctx);ctx.veto.parent={'false':true};const before=seq;await call(ctx,'parent','hidePopover');await settle();const actual=since(requests,ctx,before);assert(actual.length===1&&actual[0].overlay==='parent'&&actual[0].defaultPreventedAfterDispatch&&presented(ctx,'parent')&&presented(ctx,'child'),'parent veto returns before child request traversal');assert(since(completions,ctx,before).length===0,'veto has no completion');save(ctx);
 }
 {
  const ctx=await fresh('child false veto retains child without rolling back accepted parent');await pair(ctx);ctx.veto.child={'false':true};const before=seq;await call(ctx,'parent','hidePopover');await wait(()=>!presented(ctx,'parent')&&presented(ctx,'child'),'accepted parent closed but vetoed child native remains');await settle();assert(since(requests,ctx,before).some(row=>row.overlay==='child'&&row.detail===false&&row.defaultPreventedAfterDispatch),'child veto observed');assert(since(completions,ctx,before).some(row=>row.overlay==='parent'&&row.detail===false)&&!since(completions,ctx,before).some(row=>row.overlay==='child'&&row.detail===false),'accepted parent completion independent from child veto');save(ctx);
 }
 {
  const ctx=await fresh('closed cached child receives false request during parent hide');await open(ctx,'parent');const before=seq;await call(ctx,'parent','hidePopover');await wait(()=>!presented(ctx,'parent'),'parent native closed');await settle();assert(since(requests,ctx,before).some(row=>row.overlay==='child'&&row.detail===false&&row.showAtCapture===false),'registered cached child requested even never shown');assert(!since(completions,ctx,before).some(row=>row.overlay==='child'),'closed child has no transition completion');save(ctx);
 }
 {
  const ctx=await fresh('Window Escape selects child only while parent remains');await pair(ctx);const before=seq;const input=escape(ctx);await wait(()=>presented(ctx,'parent')&&!presented(ctx,'child'),'Escape closes selected child native surface only');await settle();const actual=since(requests,ctx,before);assert(actual.length===1&&actual[0].overlay==='child'&&actual[0].detail===false,'Popover Escape does not resolve same-type root');save(ctx,{input:input.seq});
 }
 {
  const ctx=await fresh('accepted unrelated opening requests true before dismissing nested pair');await pair(ctx);const before=seq;await open(ctx,'unrelated');await wait(()=>!presented(ctx,'parent')&&!presented(ctx,'child')&&presented(ctx,'unrelated'),'accepted unrelated dismisses source pair');await settle();const actual=since(requests,ctx,before);assert(actual[0]?.overlay==='unrelated'&&actual[0]?.detail===true&&actual.some(row=>row.overlay==='parent'&&row.detail===false)&&actual.some(row=>row.overlay==='child'&&row.detail===false),'new true callback occurs before unrelated parent/child close callbacks');save(ctx,{classification:'registry can request already-closing children again before async show flips; complete callback journal retained'});
 }
 {
  const ctx=await fresh('vetoed unrelated true request leaves existing pair untouched');await pair(ctx);ctx.veto.unrelated={'true':true};const before=seq;await call(ctx,'unrelated','showPopover');await settle();const actual=since(requests,ctx,before);assert(actual.length===1&&actual[0].overlay==='unrelated'&&actual[0].detail===true&&actual[0].defaultPreventedAfterDispatch&&presented(ctx,'parent')&&presented(ctx,'child')&&!presented(ctx,'unrelated'),'true veto precedes dismissOthers and retains old pair');save(ctx);
 }
 {
  const ctx=await fresh('same-type opening preserves unrelated sticky popover');ctx.nodes.unrelated.closeOnClickOutside=false;await open(ctx,'unrelated');const before=seq;await open(ctx,'parent');await settle();assert(presented(ctx,'parent')&&presented(ctx,'unrelated')&&!since(requests,ctx,before).some(row=>row.overlay==='unrelated'&&row.detail===false),'closeOnClickOutside=false blocks opening-time unrelated dismissal');save(ctx);
 }
 {
  const ctx=await fresh('sibling opening dismisses child while keeping cached parent');await pair(ctx);const before=seq;await open(ctx,'sibling');await wait(()=>presented(ctx,'parent')&&presented(ctx,'sibling')&&!presented(ctx,'child'),'source sibling replaces prior child');await settle();const actual=since(requests,ctx,before);assert(actual[0]?.overlay==='sibling'&&actual[0]?.detail===true&&!actual.some(row=>row.overlay==='parent'&&row.detail===false)&&actual.some(row=>row.overlay==='child'&&row.detail===false),'cached parent excluded while unrelated sibling is dismissed');save(ctx);
 }
 {
  const ctx=await fresh('direct public show=false watcher and later child visibility observation');await pair(ctx);const before=seq;const childTriggerRect=()=>{const rect=ctx.triggers.child.getBoundingClientRect();return{x:rect.x,y:rect.y,width:rect.width,height:rect.height};};const write=record('driver-public-property-write',{overlay:'parent',property:'show',value:false,childTriggerRect:childTriggerRect(),classification:'public property watcher, distinct from hidePopover controller method'});ctx.nodes.parent.show=false;record('driver-property-write-immediate',{state:state(),childTriggerRect:childTriggerRect()});await wait(()=>!presented(ctx,'parent'),'direct parent property hide closes native parent');await settle();assert(!since(requests,ctx,before).some(row=>row.overlay==='parent'&&row.detail===false)&&since(completions,ctx,before).some(row=>row.overlay==='parent'&&row.detail===false),'property watcher bypasses parent cancelable request but emits parent completion');save(ctx,{write:write.seq,childTriggerRect:childTriggerRect(),childRequestsAfterWrite:since(requests,ctx,before).filter(row=>row.overlay==='child').map(row=>row.seq),classification:'child outcome observational: hiding parent also hides nested child trigger; source Floating UI referenceHidden can separately request child dismissal. No assertion of child survival or absence of later native/position-driven requests.'});
 }
 {
  const ctx=await fresh('direct public show=true bypasses requests and dismissOthers');await open(ctx,'unrelated');const before=seq;const write=record('driver-public-property-write',{overlay:'parent',property:'show',value:true,classification:'public property watcher, distinct from showPopover controller method'});ctx.nodes.parent.show=true;await wait(()=>presented(ctx,'parent')&&presented(ctx,'unrelated'),'direct property opening retains unrelated nonsticky popover');await settle();assert(since(requests,ctx,before).length===0&&since(completions,ctx,before).some(row=>row.overlay==='parent'&&row.detail===true),'property opening bypasses cancelable requests but emits completion');save(ctx,{write:write.seq});
 }
 {
  const ctx=await fresh('unrelated opening separately visits child after parent dismissal veto');await pair(ctx);ctx.veto.parent={'false':true};const before=seq;await open(ctx,'unrelated');await wait(()=>presented(ctx,'parent')&&!presented(ctx,'child')&&presented(ctx,'unrelated'),'registry independent child visit still closes child after parent veto');await settle();const actual=since(requests,ctx,before);assert(actual.some(row=>row.overlay==='parent'&&row.detail===false&&row.defaultPreventedAfterDispatch)&&actual.some(row=>row.overlay==='child'&&row.detail===false&&!row.defaultPreventedAfterDispatch),'dismissOthers visits instances independently of parent veto');save(ctx);
 }
 // Compare actual host topology without assuming that native membership
 // guarantees a rendered child box under a closed native ancestor.
 const rect=node=>{const box=node?.getBoundingClientRect();return box?{x:box.x,y:box.y,width:box.width,height:box.height}:null;};
 const layoutNode=node=>{if(!(node instanceof Element))return null;const style=getComputedStyle(node);let popoverOpen=null;try{if(node.hasAttribute('popover'))popoverOpen=node.matches(':popover-open');}catch{}return{...simple(node),connected:node.isConnected,rect:rect(node),display:style.display,visibility:style.visibility,contentVisibility:style.contentVisibility,hidden:node.hasAttribute('hidden'),inert:node.hasAttribute('inert'),popoverOpen};};
 function ancestry(node,composed){const path=[];for(let current=node;current&&path.length<24;){if(current instanceof Element)path.push(layoutNode(current));if(composed&&current.assignedSlot){current=current.assignedSlot;continue;}const parent=current.parentNode;current=parent instanceof ShadowRoot?parent.host:parent;}return path;}
 function topologyPhase(ctx,phase,retained,input){const child=ctx.nodes.child,dialog=child.shadowRoot?.querySelector('dialog');const value={phase,topology:ctx.nestedHosts?'DOM-nested-hosts':'sibling-hosts',state:state(),childHost:layoutNode(child),childDialog:layoutNode(dialog),parentHost:layoutNode(ctx.nodes.parent),parentDialog:layoutNode(ctx.nodes.parent.shadowRoot?.querySelector('dialog')),childHostDomAncestry:ancestry(child,false),childDialogComposedAncestry:ancestry(dialog,true),childTrigger:layoutNode(ctx.triggers.child),identity:{sameParentHost:ctx.nodes.parent===retained.parent,sameChildHost:child===retained.child,sameChildDialog:dialog===retained.dialog,sameChildButton:ctx.buttons.child===retained.button,sameInput:input===retained.input&&ctx.contents.child.contains(input)},nativeInputValue:input.value,note:'rectangles/computed ancestry are observations, never inferred from show or :popover-open'};record('driver-topology-phase',value);return value;}
 for(const veto of [true,false])for(const nestedHosts of [false,true]){
  const topology=nestedHosts?'DOM-nested-hosts':'sibling-hosts';
  const ctx=await fresh('matching topology '+topology+' child '+(veto?'veto':'accept')+' parent hide/reopen',{nestedHosts});
  const input=document.createElement('input');input.id='popover-controller-'+generation+'-child-retained-input';input.setAttribute('aria-label','Source retained child native input');ctx.contents.child.append(input);
  await pair(ctx);input.value='source-native-retained-'+topology+'-'+String(veto);
  const retained={parent:ctx.nodes.parent,child:ctx.nodes.child,dialog:ctx.nodes.child.shadowRoot.querySelector('dialog'),button:ctx.buttons.child,input},seededValue=input.value,phases=[];
  assert(ctx.nodes.parent.contains(ctx.nodes.child)===nestedHosts,'actual child host topology matches driver option');assert(ctx.contents.parent.contains(ctx.triggers.child),'actual child trigger remains nested in both topologies');
  phases.push(topologyPhase(ctx,'before-parent-hide',retained,input));ctx.veto.child={'false':veto};const before=seq;const close=await call(ctx,'parent','hidePopover');await wait(()=>!ctx.nodes.parent.show&&!native(ctx.nodes.parent).dialog?.popoverOpen,'accepted source parent is actually closed');await settle();
  const closeRequests=since(requests,ctx,before);assert(closeRequests[0]?.overlay==='parent'&&closeRequests[0]?.detail===false,'public parent false request precedes descendants in matching topology');assert(closeRequests.some(row=>row.overlay==='child'&&row.detail===false&&row.defaultPreventedAfterDispatch===veto),'actual source child close request records selected veto policy');
  if(veto)assert(ctx.nodes.child.show===true,'source child visibility state remains true after veto; native geometry independently recorded');else await wait(()=>ctx.nodes.child.show===false&&rows(completions,ctx,'child',false).length>0,'accepted source child close state/completion');
  phases.push(topologyPhase(ctx,'after-parent-hide',retained,input));assert(input.value===seededValue&&ctx.nodes.child===retained.child&&ctx.nodes.child.shadowRoot.querySelector('dialog')===retained.dialog&&ctx.contents.child.contains(input),'source child host/dialog/input identity and native input value survive parent close');
  const reopen=await call(ctx,'parent','showPopover');await wait(()=>presented(ctx,'parent'),'source parent reopens by public method');await settle();phases.push(topologyPhase(ctx,'after-parent-reopen',retained,input));
  if(!veto){await open(ctx,'child');phases.push(topologyPhase(ctx,'after-explicit-child-reopen',retained,input));}
  assert(input.value===seededValue&&ctx.nodes.parent===retained.parent&&ctx.nodes.child===retained.child&&ctx.nodes.child.shadowRoot.querySelector('dialog')===retained.dialog&&ctx.buttons.child===retained.button&&input.isConnected,'same source native objects and input value survive reopen');
  save(ctx,{topology,nestedHosts,childVeto:veto,close,reopen,phases,seededNativeInputValue:seededValue,classification:'actual source child HOST topology paired with identical nested triggers; accepted/vetoed public controller requests and native identity checked; child rectangles/native membership/computed ancestry/focus observed without assuming visible outcome; native input value is not Rust component-state proof'});
 }
 {
  const ctx=await fresh('hover restored already-focused trigger permits later fresh focus activation');
  await customElements.whenDefined('ix-popover-header');const header=document.createElement('ix-popover-header');header.id='popover-controller-'+generation+'-hover-header';header.textContent='Actual source hover header';ctx.contents.parent.prepend(header);await header.componentOnReady();const closeButton=header.shadowRoot.querySelector('ix-icon-button');if(closeButton)await closeButton.componentOnReady();await frames();const nativeHeaderButtons=[];const gatherHeaderButtons=root=>{for(const node of root.querySelectorAll('*')){if(node.tagName==='BUTTON')nativeHeaderButtons.push({...simple(node),autofocus:node.autofocus,autofocusAttribute:node.hasAttribute('autofocus')});if(node.shadowRoot)gatherHeaderButtons(node.shadowRoot);}};gatherHeaderButtons(header.shadowRoot);assert(nativeHeaderButtons.length>0,'actual source header close native button hydrated');record('driver-actual-source-hover-header',{header:simple(header),nativeButtons:nativeHeaderButtons,note:'actual upstream header component included before hover focus; no fabricated autofocus policy'});
  ctx.nodes.parent.triggerMode='hover';await frames();await wait(()=>ctx.triggers.parent.hasAttribute('data-ix-popover-trigger'),'actual source hover trigger listeners registered');
  const stages=[];const capture=phase=>{const value={phase,state:state(),activeIsTrigger:deepActive()===ctx.triggers.parent,trigger:layoutNode(ctx.triggers.parent),parentDialog:layoutNode(ctx.nodes.parent.shadowRoot.querySelector('dialog'))};stages.push(value);record('driver-hover-focus-phase',value);};
  record('driver-programmatic-trigger-focus',{target:simple(ctx.triggers.parent),mode:'actual public triggerMode hover'});ctx.triggers.parent.focus();await wait(()=>presented(ctx,'parent')&&rows(completions,ctx,'parent',true).length>0,'source hover focus actually opens parent');await settle();assert(deepActive()===ctx.triggers.parent,'source hover opening preserves focus on its configured trigger');capture('hover-open-trigger-still-focused');
  const repeatStart=seq;await call(ctx,'parent','showPopover');await settle();assert(since(requests,ctx,repeatStart).length===0&&deepActive()===ctx.triggers.parent,'already-open public show does not move hover trigger focus or emit a new request');
  const close=await call(ctx,'parent','hidePopover');await wait(()=>!ctx.nodes.parent.show&&!native(ctx.nodes.parent).dialog?.popoverOpen&&rows(completions,ctx,'parent',false).length>0,'source hover public hide completes');await frames(3);await settle();assert(deepActive()===ctx.triggers.parent,'public restore keeps the already-focused configured hover trigger');capture('closed-after-restore-and-following-frames');
  record('driver-programmatic-outside-focus',{target:simple(ctx.outside)});ctx.outside.focus();await frames();assert(deepActive()===ctx.outside,'native focus leaves hover trigger');capture('outside-focus-before-refocus');
  const refocusStart=seq;record('driver-programmatic-trigger-refocus',{target:simple(ctx.triggers.parent)});ctx.triggers.parent.focus();await wait(()=>presented(ctx,'parent')&&since(completions,ctx,refocusStart).some(row=>row.overlay==='parent'&&row.detail===true),'later source fresh trigger focus reopens hover Popover');await settle();assert(since(requests,ctx,refocusStart).some(row=>row.overlay==='parent'&&row.detail===true),'fresh focus emits a real source opening request');assert(deepActive()===ctx.triggers.parent,'fresh hover opening retains configured trigger focus');capture('fresh-focus-reopened');
  save(ctx,{close,stages,classification:'actual public hover mode and native programmatic focus; already-focused trigger close/restore observed across multiple RAFs, then fresh focus reopens. No private suppression flag read, physical input or timer precision claim'});
 }
 current?.abort.abort();globalAbort.abort();
 return{ok:true,revision,caseCount:observations.length,observations,sourceRequests:requests,sourceCompletions:completions,actualChildAssignmentObservations:assignments,nativeFocusEvents:focusEvents,journal,input:'public showPopover/hidePopover promises and show property writes; one synthetic native Window-bubbling Escape; real pinned source components/controllers/native popover API/RAF own policy',limits:['one source module and native document; no Rust parity, independent VDOM or multi-window proof','all initiating methods/properties and Escape are programmatic/synthetic; no physical pointer, keyboard activation, hover, platform matrix or AT proof','native dialog :popover-open, show property, request, separate RAF completion and native focus endpoints are distinct observations','source internal cached graph is inferred from actual assignment event and public behavior; no private registry calls or fabricated component events','public property writes are bounded diagnostics and bypass the written host controller method by source design; subsequent child anchor-visibility/native/focus effects can still cause separate controller requests','manual native top layer and final focus observed; completion/event timing is not promised by public showPopover/hidePopover resolution','first15 scenarios use sibling child hosts with nested triggers; next4 explicitly compare sibling versus actual DOM-nested child hosts with geometry/computed ancestry observations; controller native membership is not itself visible-box proof','final hover case uses programmatic native focus with actual source hover listeners/public methods and observes fresh focus after restore; no private flag or physical input proof','native input value and object identity are source DOM retention only, not Rust component state','fixed diagnostic stage and foundation cache styles/system fonts establish no Brand geometry/placement parity','source runtime bytes/cache provenance must be verified externally; original source fixture remains unchanged']};
} catch(error){return{ok:false,error:String(error),observations,sourceRequests:requests,sourceCompletions:completions,actualChildAssignmentObservations:assignments,nativeFocusEvents:focusEvents,journal};}
"#;

#[component]
fn Probe() -> Element {
    let window = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(120)).await;
            eprintln!("Pinned source Popover controller timeout");
            std::process::exit(1);
        });
    });
    rsx! { div { onmounted: move |_| {
        let window = window.clone();
        spawn(async move {
            let query = dom_query::eval(VERIFY);
            let _ = query.send(());
            match query.join::<serde_json::Value>().await {
                Ok(result) if result["ok"] == true => { println!("Pinned source Popover controller checks: {result}"); window.close(); }
                Ok(result) => { eprintln!("Pinned source Popover controller failure JSON: {result}"); std::process::exit(1); }
                Err(error) => { eprintln!("Pinned source Popover controller eval failure: {error:?}"); std::process::exit(1); }
            }
        });
    } } }
}
