// SPDX-FileCopyrightText: 2026 Siemens AG
// SPDX-License-Identifier: MIT

//! Native DOM observations and execution for the per-root overlay runtime.
//! Document identity and minimal input delivery sequence are shared across roots.
//! Read-only observations include marked roots across renderer owners so cached
//! Tab/Header projections preserve cross-owner composition. Rust registrations
//! alone authorize ownership and actions; the shared hub delivers global input.
//! `__OWNER__` is replaced by a serialized runtime owner ID. Rust owns all
//! hierarchy, item membership/metadata, menu/focus policy, and open/close decisions;
//! All deliveries use noncancelable `input`; typed Dioxus controls own keyboard
//! cancellation, activation and boundary focus. No raw native Tab dispatch or
//! prevent/stop acknowledgement transport is retained.

pub(super) const BRIDGE: &str = r#"
const owner=__OWNER__,listeners=new AbortController(),signal=listeners.signal;
const documentSlot=Symbol.for('dioxus-ix.document-native-domain.v1');
const nativeDocument=document[documentSlot]||(document[documentSlot]={identity:owner,eventSequence:0});
const focusableSelector='a[href],area[href],audio,button,input:not([type="hidden"]),select,textarea,iframe,object,embed,video,[tabindex],[contenteditable],ix-dropdown-item,ix-select-item';
let alive=true,sequence=0,lastSignature='',documentEvents=null;
// Mounted focus registration uses Rust-authored IDs; unidentified DOM controls
// have no programmatic focus fallback and need no native identity registry.
function nativeId(el){return el instanceof HTMLElement?el.id:'';}
// Dioxus owns overlay lifetimes and relationships. These document lookups
// supply only remaining native order/geometry/presentation facts for its marked
// light-DOM surfaces. Foreign shadow-tree discovery is unsupported.
function transport(){
  for(const input of document.querySelectorAll('input[data-ix-overlay-transport]'))if(input instanceof HTMLInputElement&&input.isConnected&&input.getAttribute('data-ix-overlay-transport')===owner)return input;
  return null;
}
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
function findHost(id){
  for(const n of document.querySelectorAll('[data-ix-overlay-host]'))if(n instanceof HTMLElement&&n.isConnected&&n.getAttribute('data-ix-overlay-host')===id&&isActiveHost(n))return n;
  return null;
}
function findItemsHost(root,id){
  const configured=root.getAttribute('data-ix-overlay-items-host-id'),element=configured?document.getElementById(configured):null;
  return element instanceof HTMLElement&&element.isConnected?element:findHost(root.getAttribute('data-ix-overlay-items-owner')||id);
}
// Trigger metadata and anchor style are rendered by the owning Dioxus
// component. This lookup observes current markers, including cross-owner
// controls; an ID alone never installs behavior or supplies a fallback.
function findTrigger(id){
  for(const element of document.querySelectorAll('[data-ix-overlay-trigger]'))if(element instanceof HTMLElement&&element.isConnected&&element.getAttribute('data-ix-overlay-trigger')===id)return element;
  return null;
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
function visibleFocusable(n){return n instanceof HTMLElement&&n.tabIndex>=0&&!isHiddenOrInert(n);}
function sortByTabOrder(els){
  return els.map((el,i)=>({i,el})).sort((a,b)=>{
    const ao=a.el.tabIndex>0?a.el.tabIndex:Number.MAX_SAFE_INTEGER,bo=b.el.tabIndex>0?b.el.tabIndex:Number.MAX_SAFE_INTEGER;
    return ao-bo||a.i-b.i;
  }).map(x=>x.el);
}
function hostFocusables(host){
  return host?sortByTabOrder(Array.from(host.querySelectorAll(focusableSelector)).filter(el=>el instanceof HTMLElement&&visibleFocusable(el)&&nearestHost(el)===host)):[];
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
function popoverTabFacts(panel,active){
  // Pinned dioxus-html 0.8 alpha MountedData has focus/scroll methods but no
  // document tab-order enumeration. Observe candidates only; Dropdown exits
  // execute through registered mounted handles in Rust, with no JS fallback.
  if(!panel)return [];
  const raw=Array.from(panel.querySelectorAll(focusableSelector)).filter(element=>{
    if(!(element instanceof HTMLElement)||element.matches('[tabindex^="-"],[hidden],[disabled],:disabled')||nearestPopover(element)!==panel)return false;
    for(let n=element;n;n=stepParent(n))if(n instanceof HTMLElement&&n.inert)return false;
    const style=getComputedStyle(element);if(style.visibility==='hidden'||style.visibility==='collapse')return false;
    if(typeof element.checkVisibility==='function')return element.checkVisibility();
    for(let n=element;n;n=stepParent(n))if(n instanceof Element&&getComputedStyle(n).display==='none')return false;
    return element.getClientRects().length>0;
  });
  let activeIndex=raw.indexOf(active);
  if(activeIndex===-1)activeIndex=raw.findIndex(element=>active&&composedContains(element,active));
  return raw.map((element,index)=>({target:nativeId(element),tab_index:element.tabIndex,hosts:hostAncestors(element),active:index===activeIndex}));
}
// Pinned MountedData exposes mounted focus/measurement, but no child or
// document-position enumeration. This primary adapter reports only Rust-rendered
// registration keys in current DOM order. Rust owns membership, current owner
// generation and eligibility; an arbitrary DOM element cannot join an inventory.
function registeredItemOrder(host){
  return host instanceof HTMLElement?Array.from(host.querySelectorAll('[data-ix-item-key]')).map(element=>element.getAttribute('data-ix-item-key')).filter(Boolean):[];
}
function pathIds(path,attr,pred){
  const out=[];
  for(const item of path)if(item instanceof HTMLElement&&(!pred||pred(item))){
    const id=item.getAttribute(attr);if(id&&!out.includes(id))out.push(id);
  }
  return out;
}
function documentTabCandidates(){
  if(!document.body)return [];
  const raw=Array.from(document.body.querySelectorAll(focusableSelector)).filter(el=>{
    return el instanceof HTMLElement&&visibleFocusable(el);
  });
  return raw.map(el=>({target:nativeId(el),tab_index:el.tabIndex,hosts:hostAncestors(el),active:false}));
}
function headerControls(root){
  const header=root.closest('header.ix-application-header');if(!header)return [];
  return Array.from(header.querySelectorAll('[data-header-more-button],[data-header-menu-toggle],[data-app-switch-opener],.ix-application-header-leading > .ix-app-icon-button'))
    .filter(el=>el instanceof HTMLElement&&el.closest('header.ix-application-header')===header).map(nativeId);
}
function collectFacts(){
  const active=deepActive(),nodes=[];
  // Candidate metadata spans renderer owners; it never creates graph members.
  // Rust validates the current registered generation/kind before projection.
  for(const root of document.querySelectorAll('[data-ix-overlay-root]')){
    if(!(root instanceof HTMLElement)||!root.isConnected)continue;
    const id=root.getAttribute('data-ix-overlay-root')||'';if(!id)continue;
    const generation=Number(root.getAttribute('data-ix-overlay-generation')||root.getAttribute('data-generation')||'0')||0;
    const host=findHost(id),itemsHost=findItemsHost(root,id),trigger=findTrigger(id),anchor=findAnchor(root),focusables=hostFocusables(itemsHost);
    nodes.push({
      id,generation,trigger_id:nativeId(trigger),
      anchor_in_dropdown_item:!!(anchor||trigger)?.closest('[data-ix-dropdown-item]'),
      header_controls:isHeaderPopupHost(host)?headerControls(root):[],
      item_order:registeredItemOrder(itemsHost),
      // MountedData cannot enumerate document Tab order. Rust owns the direct
      // key handler and executes focus only through registered mounted handles.
      focusable_ids:focusables.map(nativeId).filter(Boolean),
      tab_candidates:root.getAttribute('data-ix-overlay-native-surface')==='popover'?popoverTabFacts(host,active):[]
    });
  }
  const document_tab_candidates=documentTabCandidates();
  return {
    owner,document:nativeDocument.identity,sequence:++sequence,kind:'observe',active_id:nativeId(active),
    nodes,document_tab_candidates,tab_order:document_tab_candidates.map(candidate=>candidate.target),
    active_hosts:hostAncestors(active),active_popover:nearestPopover(active)?.getAttribute('data-ix-overlay-host')||''
  };
}
function send(report){
  const input=transport();if(!alive||!input)return;
  input.value=JSON.stringify(report);
  input.dispatchEvent(new Event('input',{bubbles:true,cancelable:false}));
}

function syncNativeSurfaces(){
  for(const root of document.querySelectorAll('[data-ix-overlay-root]')){
    if(!(root instanceof HTMLElement)||!root.isConnected)continue;
    const ro=root.getAttribute('data-ix-overlay-owner'),id=root.getAttribute('data-ix-overlay-root')||'';
    if(!id||ro!==owner)continue;
    if(root.hasAttribute('data-ix-overlay-native-surface'))continue;
    const host=findHost(id);
    if(host&&!isHeaderPopupHost(host)){
      const open=root.getAttribute('data-ix-overlay-open')==='true',top=root.getAttribute('data-ix-overlay-top-layer')==='true';
      const dialog=host.matches('dialog[popover]')?host:host.querySelector('dialog[popover]');
      if(dialog instanceof HTMLElement){
        let isOpen=false;try{isOpen=dialog.matches(':popover-open');}catch{}
        if(open&&top&&!isOpen&&typeof dialog.showPopover==='function'){
          const prev=dialog.inert;dialog.inert=true;
          try{dialog.showPopover();}catch{}finally{dialog.inert=prev;}
        }else if((!open||!top)&&isOpen&&typeof dialog.hidePopover==='function'){
          try{dialog.hidePopover();}catch{}
        }
      }
    }
  }
}
function reportObserve(){
  if(!alive)return;
  if(!transport())return;
  for(const input of document.querySelectorAll('input[data-ix-overlay-transport]'))if(input.getAttribute('data-ix-overlay-transport')===owner)input.setAttribute('data-ix-overlay-native','true');
  syncNativeSurfaces();
  const report=collectFacts();
  const sig=JSON.stringify({nodes:report.nodes,active_id:report.active_id,document_tab_candidates:report.document_tab_candidates});
  if(sig!==lastSignature){lastSignature=sig;send(report);}
  if(!documentEvents)documentEvents=subscribeDocumentEvents();
}
// Pinned Document has no typed document/window input listener API. This sole
// shared adapter reports input facts; Dioxus and Rust own decisions and actions.
// Only callbacks are retained: each delivery resolves its owner's live transport.
function sendDocumentEvent(action,event,deliverySequence){
  if(!alive)return false;
  const input=transport();if(!input)return false;
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
  input.value=JSON.stringify({owner,document:nativeDocument.identity,sequence:++sequence,kind:'document',document_event});
  input.dispatchEvent(new Event('input',{bubbles:true,cancelable:false}));
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
      for(const sender of Array.from(events.receivers.values()))if(sender(action,event,deliverySequence)&&action!=='input')break;
    };
    for(const kind of ['keydown','mousedown','touchstart'])document.addEventListener(kind,event=>deliver('input',event),{passive:true,signal:sharedSignal});
    window.addEventListener('click',event=>deliver('outside',event),{signal:sharedSignal});
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
document.addEventListener('focusin',reportObserve,{signal});
// Remaining generic Tab facts observe current focus. Typed item cues are
// owned by Dioxus focus/blur handlers.
document.addEventListener('focusout',()=>{queueMicrotask(()=>{if(alive)reportObserve();});},{passive:true,signal});
// Remaining native facts track order, focus candidates and presentation;
// this observer never enrolls roots or changes ownership/lifecycle in Rust.
const observer=new MutationObserver(reportObserve);
observer.observe(document.documentElement,{subtree:true,childList:true,attributes:true,attributeFilter:[
  'data-ix-overlay-root','data-ix-overlay-owner','data-ix-overlay-host','data-ix-overlay-trigger','data-ix-overlay-anchor-id','data-ix-overlay-generation','data-ix-overlay-open',
  'data-ix-overlay-mode','data-ix-overlay-native-surface','data-ix-overlay-items-owner','data-ix-overlay-revision','data-ix-overlay-top-layer','data-small','data-open','hidden','disabled','tabindex','inert','class',
  'data-ix-item-key','data-ix-overlay-items-host-id','aria-activedescendant','data-ix-overlay-active-item'
]});
document.documentElement.setAttribute('data-ix-overlay-registry-native',owner);
try{
  reportObserve();
  while(alive){const command=await dioxus.recv();if(command?.finish)break;
  }
}finally{
  alive=false;listeners.abort();observer.disconnect();
  unsubscribeDocumentEvents();
  for(const input of document.querySelectorAll('input[data-ix-overlay-transport]'))if(input.getAttribute('data-ix-overlay-transport')===owner)input.removeAttribute('data-ix-overlay-native');
  if(document.documentElement.getAttribute('data-ix-overlay-registry-native')===owner)document.documentElement.removeAttribute('data-ix-overlay-registry-native');
}
"#;
