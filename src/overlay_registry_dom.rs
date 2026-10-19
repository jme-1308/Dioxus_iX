// SPDX-FileCopyrightText: 2026 Siemens AG
// SPDX-License-Identifier: MIT

//! Native DOM observations and execution for the per-root overlay runtime.
//! Document identity and minimal input delivery sequence are shared across roots.
//! Read-only focus candidates span renderer owners; Dioxus registrations supply
//! current surface descriptors for Tab/Header projections. The shared hub
//! delivers global input, while Rust alone authorizes ownership and actions.
//! `__OWNER__` is replaced by a serialized runtime owner ID. Rust owns all
//! hierarchy, item membership/metadata, menu/focus policy, and open/close decisions;
//! Typed Eval channels deliver read-only reports; Dioxus controls own keyboard
//! cancellation, activation and boundary focus. No raw native Tab dispatch or
//! prevent/stop acknowledgement transport is retained.

pub(super) const BRIDGE: &str = r#"
const owner=__OWNER__,listeners=new AbortController(),signal=listeners.signal;
const documentSlot=Symbol.for('dioxus-ix.document-native-domain.v1');
const nativeDocument=document[documentSlot]||(document[documentSlot]={identity:owner,eventSequence:0});
const focusableSelector='a[href],area[href],audio,button,input:not([type="hidden"]),select,textarea,iframe,object,embed,video,[tabindex],[contenteditable],ix-dropdown-item,ix-select-item';
let alive=true,sequence=__START_SEQUENCE__,lastSignature='',documentEvents=null,lastFocus=null,movedSequence=0;
// Mounted focus registration uses Rust-authored IDs; unidentified DOM controls
// have no programmatic focus fallback and need no native identity registry.
function nativeId(el){return el instanceof HTMLElement?el.id:'';}
// Dioxus owns overlay lifetimes, relationships and surface descriptors. The
// remaining native lookups provide current focus/eligibility, keyed-move and
// global outside-event facts. Foreign shadow-tree discovery is unsupported.
function deepActive(){
  let a=document.activeElement;
  while(a){
    if(a instanceof HTMLElement&&a.shadowRoot?.activeElement&&a.shadowRoot.activeElement!==a){a=a.shadowRoot.activeElement;continue;}
    const r=a.getRootNode();
    if(r instanceof ShadowRoot&&r.activeElement&&r.activeElement!==a){a=r.activeElement;continue;}
    break;
  }
  return a instanceof HTMLElement?a:null;
}
function stepParent(n){
  if(!n)return null;
  if(n instanceof Element&&n.assignedSlot)return n.assignedSlot;
  if(n.parentNode)return n.parentNode;
  const r=n.getRootNode();
  return r instanceof ShadowRoot?r.host:null;
}
function composedContains(container,node){for(let c=node;c;c=stepParent(c))if(c===container)return true;return false;}
function isHeaderPopupHost(n){return n instanceof HTMLElement&&n.hasAttribute('data-header-popup-host');}
function isActiveHost(n){
  if(!(n instanceof HTMLElement)||!n.getAttribute('data-ix-overlay-host'))return false;
  if(!isHeaderPopupHost(n))return true;
  const hr=n.closest('[data-small],[data-ix-overlay-root]');
  return n.getAttribute('data-header-popup-host')===(hr?.getAttribute('data-small')==='true'?'small':'large');
}
function findAnchor(root){
  const id=root?.getAttribute('data-ix-overlay-anchor-id'),el=id?document.getElementById(id):null;
  return el instanceof HTMLElement&&el.isConnected?el:null;
}
function nearestHost(n){for(let c=n;c;c=stepParent(c))if(c instanceof HTMLElement&&isActiveHost(c))return c;return null;}
function isHiddenOrInert(n){
  if(!(n instanceof HTMLElement)||n.matches(':disabled,[disabled],[hidden],[aria-disabled="true"]')||!n.getClientRects().length)return true;
  const s=getComputedStyle(n);
  if(s.display==='none'||s.visibility==='hidden'||s.visibility==='collapse')return true;
  for(let c=n;c;c=stepParent(c)){
    if(c instanceof HTMLElement&&(c.inert||c.hidden||c.getAttribute('aria-hidden')==='true'))return true;
    if(c instanceof Element&&getComputedStyle(c).display==='none')return true;
  }
  return false;
}
function hostAncestors(element){
  const result=[];
  for(let n=element;n;n=stepParent(n))if(n instanceof HTMLElement&&isActiveHost(n)){
    const id=n.getAttribute('data-ix-overlay-host');if(id&&!result.includes(id))result.push(id);
  }
  return result;
}
function nearestPopover(element){
  for(let n=element;n;n=stepParent(n))if(n instanceof HTMLElement&&n.getAttribute('data-ix-overlay-native-surface')==='popover')return n;
  return null;
}
function pathIds(path,attr,pred){
  const out=[];
  for(const item of path)if(item instanceof HTMLElement&&(!pred||pred(item))){
    const id=item.getAttribute(attr);if(id&&!out.includes(id))out.push(id);
  }
  return out;
}
// MountedData has no descendant/document enumeration, current-active query or
// computed visibility. One document-order scan supplies those native facts;
// Rust derives host/Popover projections, active selection and Tab ordering.
function focusCandidates(active){
  const eligible=document.body?Array.from(document.body.querySelectorAll(focusableSelector)).filter(element=>!isHiddenOrInert(element)):[];
  return {
    // Programmatic return may target a registered tabindex=-1 opener. This is
    // the same visibility pass, not another discovery or native focus path.
    eligible_focus_ids:eligible.map(nativeId).filter(Boolean),
    focus_candidates:eligible.filter(element=>element.tabIndex>=0).map(element=>({
      target:nativeId(element),tab_index:element.tabIndex,autofocus:element.hasAttribute('autofocus'),hosts:hostAncestors(element),
      nearest_host:nativeId(nearestHost(element)),popover:nativeId(nearestPopover(element)),
      exact_active:element===active,contains_active:!!active&&composedContains(element,active)
    }))
  };
}
// The pinned interpreter uses insertBefore for keyed moves. MountedData has
// no move notification or activeElement query, so observe the existing native
// records. Only key/ID strings survive this callback; Rust performs any repair.
function registeredFocus(element){
  if(!(element instanceof HTMLElement)||!element.id)return null;
  const item=element.closest('[data-ix-item-key]'),host=nearestHost(element);
  if(!item||!host)return null;
  const key=item.getAttribute('data-ix-item-key'),rootId=host.getAttribute('data-ix-overlay-host');
  const root=Array.from(document.querySelectorAll('[data-ix-overlay-root]')).find(root=>root.getAttribute('data-ix-overlay-root')===rootId&&root.getAttribute('data-ix-overlay-owner')===owner);
  if(!key||!root||root.getAttribute('data-ix-overlay-mode')!=='roving-tabindex'||root.getAttribute('data-ix-overlay-open')!=='true')return null;
  return {key,focus_id:element.id};
}
function movedRegisteredFocus(records,active){
  if(!lastFocus||!document.hasFocus())return null;
  const target=document.getElementById(lastFocus.focus_id),current=registeredFocus(target);
  if(!target?.isConnected||!current||current.key!==lastFocus.key)return null;
  const host=nearestHost(target);
  if(active&&active!==document.body&&active!==document.documentElement&&active!==host)return null;
  // Compare the actual current control against both mutation sides. A new
  // same-ID/key element or a changed label cannot masquerade as a moved node.
  const includes=node=>node===target||(node instanceof Element&&node.contains(target));
  const removed=records.some(record=>record.type==='childList'&&Array.from(record.removedNodes).some(includes));
  const added=records.some(record=>record.type==='childList'&&Array.from(record.addedNodes).some(includes));
  return removed&&added?current:null;
}
function collectFacts(){
  const active=deepActive();
  return {
    owner,document:nativeDocument.identity,sequence:++sequence,kind:'observe',active_id:nativeId(active),
    active_default:!active||active===document.body||active===document.documentElement||active.matches('[data-ix-overlay-host]'),
    active_nested:!!active?.closest?.('dialog[open],[data-ix-overlay-open="true"]')||(()=>{try{return !!active?.closest?.('[popover]:popover-open');}catch{return false;}})(),
    ...focusCandidates(active),
    active_hosts:hostAncestors(active),active_popover:nearestPopover(active)?.getAttribute('data-ix-overlay-host')||''
  };
}
function send(report){if(alive)dioxus.send(report);}

function reportObserve(records=[],focusChanged=false,focusoutMove=null,focusRequest=null){
  if(!alive)return;
  const pending=observer.takeRecords();
  records=Array.from(records).concat(pending);
  const active=deepActive(),moved=movedRegisteredFocus(records,active);
  if(moved)movedSequence++;
  const explicitBlur=focusoutMove!==null&&!moved&&focusoutMove===movedSequence;
  if(explicitBlur||!document.hasFocus())lastFocus=null;
  else if(active&&active!==document.body&&active!==document.documentElement&&!active.matches('[data-ix-overlay-host]'))lastFocus=registeredFocus(active);
  else if(lastFocus&&!document.getElementById(lastFocus.focus_id)?.isConnected)lastFocus=null;
  const report=collectFacts();
  report.focus_changed=focusChanged||explicitBlur;
  if(focusRequest)report.focus_request=focusRequest;
  if(moved)report.moved_focus=moved;
  const sig=JSON.stringify({active_id:report.active_id,active_default:report.active_default,focus_candidates:report.focus_candidates,active_hosts:report.active_hosts,active_nested:report.active_nested,eligible_focus_ids:report.eligible_focus_ids});
  if(focusRequest||moved||report.focus_changed||sig!==lastSignature){lastSignature=sig;send(report);}
  if(!documentEvents)documentEvents=subscribeDocumentEvents();
}
// Pinned Document has no typed document/window input listener API. This sole
// shared adapter reports input facts; Dioxus and Rust own decisions and actions.
// Only callbacks are retained; each active stream uses its own typed channel.
function sendDocumentEvent(action,event,deliverySequence){
  if(action==='blur')lastFocus=null;
  if(!alive)return false;
  const document_event={sequence:deliverySequence,action};
  if(action==='input'){
    document_event.input_kind=event.type;
    if(event.type==='keydown')document_event.key=event.key;
  }else if(action==='outside'){
    const path=event.composedPath();
    document_event.hosts=pathIds(path,'data-ix-overlay-host',isActiveHost);
    document_event.triggers=pathIds(path,'data-ix-overlay-trigger');
    document_event.anchors=[];
    for(const root of document.querySelectorAll('[data-ix-overlay-root][data-ix-overlay-anchor-id]')){
      if(!(root instanceof HTMLElement)||!root.isConnected)continue;
      const id=root.getAttribute('data-ix-overlay-root'),anchor=findAnchor(root);
      if(id&&anchor&&path.some(node=>node instanceof Node&&composedContains(anchor,node))&&!document_event.anchors.includes(id))document_event.anchors.push(id);
    }
  }
  send({owner,document:nativeDocument.identity,sequence:++sequence,kind:'document',document_event});
  return true;
}
function subscribeDocumentEvents(){
  let events=nativeDocument.events;
  if(!events){
    events={controller:new AbortController(),receivers:new Map()};
    nativeDocument.events=events;
    const sharedSignal=events.controller.signal;
    const deliver=(action,event)=>{
      const deliverySequence=++nativeDocument.eventSequence;
      for(const sender of Array.from(events.receivers.values()))if(sender(action,event,deliverySequence)&&action!=='input'&&action!=='blur')break;
    };
    for(const kind of ['keydown','mousedown','touchstart'])document.addEventListener(kind,event=>deliver('input',event),{passive:true,signal:sharedSignal});
    window.addEventListener('click',event=>deliver('outside',event),{signal:sharedSignal});
    // Element FocusData has no window-blur notification. Broadcast this small
    // fact so every renderer can cancel only its pending Rust move repairs.
    window.addEventListener('blur',event=>deliver('blur',event),{signal:sharedSignal});
    // Dioxus web forwards preventDefault but not virtual stopPropagation to
    // the original event. A handled child/Header Escape must not dismiss a
    // second overlay when that original event reaches the shared window hub.
    window.addEventListener('keydown',event=>{if(event.key==='Escape'&&!event.defaultPrevented)deliver('escape',event);},{signal:sharedSignal});
  }
  events.receivers.set(owner,sendDocumentEvent);
  return events;
}
function unsubscribeDocumentEvents(){
  if(documentEvents?.receivers.get(owner)!==sendDocumentEvent)return;
  documentEvents.receivers.delete(owner);
  if(!documentEvents.receivers.size){
    documentEvents.controller.abort();
    if(nativeDocument.events===documentEvents)delete nativeDocument.events;
  }
}
document.addEventListener('focusin',()=>reportObserve([],true),{signal});
// Remaining generic Tab facts observe current focus. Typed item cues are
// owned by Dioxus focus/blur handlers.
document.addEventListener('focusout',()=>{const observedMoves=movedSequence;queueMicrotask(()=>{if(alive)reportObserve([],false,observedMoves);});},{passive:true,signal});
// Remaining native facts track focus candidates and presentation;
// this observer never enrolls roots or changes ownership/lifecycle in Rust.
const observer=new MutationObserver(records=>reportObserve(records));
observer.observe(document.documentElement,{subtree:true,childList:true,attributes:true,attributeFilter:[
  'data-ix-overlay-root','data-ix-overlay-owner','data-ix-overlay-host','data-ix-overlay-trigger','data-ix-overlay-anchor-id','data-ix-overlay-generation','data-ix-overlay-open',
  'data-ix-overlay-mode','data-ix-overlay-native-surface','data-ix-overlay-items-owner','data-ix-overlay-revision','data-ix-overlay-top-layer','data-small','data-open','open','hidden','disabled','tabindex','autofocus','inert','class',
  'data-ix-item-key','data-ix-overlay-items-host-id','aria-activedescendant','data-ix-overlay-active-item'
]});
try{
  reportObserve();
  while(alive){const command=await dioxus.recv();if(command?.finish)break;
    // MountedData cannot query eligibility/document order. Reuse the canonical
    // scan for opening without inventing focus changes or retaining requests.
    if(command?.focus_request)reportObserve([],false,null,command.focus_request);
  }
}finally{
  alive=false;listeners.abort();observer.disconnect();
  unsubscribeDocumentEvents();
}
"#;
