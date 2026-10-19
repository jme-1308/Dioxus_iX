// SPDX-FileCopyrightText: 2026 Siemens AG
// SPDX-License-Identifier: MIT

//! Native DOM observations and execution for the per-root overlay runtime.
//! Document identity and original-event identity are shared across roots. Local
//! reports retain the root partition; Window reports expose every native overlay
//! in that document so Rust can coordinate them without native policy.
//! `__OWNER__` is replaced by a serialized runtime owner ID. Rust owns all
//! hierarchy, discovery caches, menu/focus policy, and open/close decisions;
//! `input` dispatches a raw fact report (`prevent` ACK) and `change` queries
//! the saved `stop` ACK with identical report JSON. Dropdown keyboard handling
//! and activation run through typed Dioxus controls.

pub(super) const BRIDGE: &str = r#"
const owner=__OWNER__,listeners=new AbortController(),signal=listeners.signal;
const documentSlot=Symbol.for('dioxus-ix.document-native-domain.v1');
const nativeDocument=document[documentSlot]||(document[documentSlot]={identity:owner,eventSequence:0,eventIds:new WeakMap()});
nativeDocument.inventoryReaders||=new Map();
function windowSequence(event){if(!event)return 0;let id=nativeDocument.eventIds.get(event);if(!id){id=++nativeDocument.eventSequence;nativeDocument.eventIds.set(event,id);}return id;}
const focusableSelector='a[href],area[href],audio,button,input:not([type="hidden"]),select,textarea,iframe,object,embed,video,[tabindex],[contenteditable],ix-dropdown-item,ix-select-item';
let alive=true,sequence=0,lastSignature='';
const itemObjectTokens=new WeakMap(),itemCaptureNodes=new Map();let itemObjectSequence=0;
const itemInventoryLeases=new Map();
function captureItem(element){
  if(!(element instanceof HTMLElement))return null;
  const target=nativeId(element);if(!target)return null;
  let capture=itemObjectTokens.get(element);
  if(!capture){capture=owner+'-item-object-'+(++itemObjectSequence);itemObjectTokens.set(element,capture);}
  itemCaptureNodes.set(capture,{element,current:true});return {target,capture};
}
function pruneItemCaptures(){
  for(const [capture,binding]of itemCaptureNodes){
    if(binding.current||Array.from(itemInventoryLeases.values()).some(lease=>lease.tokens.has(capture)))continue;
    const input=transport();if(!input)continue;
    input.value=JSON.stringify({owner,sequence:0,capture_retained:capture});
    if(input.dispatchEvent(new Event('change',{bubbles:true,cancelable:true})))itemCaptureNodes.delete(capture);
  }
}
// Mounted focus registration uses Rust-authored IDs; unidentified DOM controls
// have no programmatic focus fallback and need no native identity registry.
function nativeId(el){return el instanceof HTMLElement?el.id:'';}
// Registry reachability is separate from source-selected light-DOM queries.
// Scan a subtree only at discovery/mutation boundaries, never for every query.
const registryScopes=new Set([document]);
function registryElements(selector){
  const result=[];
  for(const scope of registryScopes)if(scope===document||scope.host.isConnected)result.push(...scope.querySelectorAll(selector));
  return result;
}
function registryElement(id){
  for(const scope of registryScopes){
    if(scope!==document&&!scope.host.isConnected)continue;
    const element=scope.getElementById(id);if(element instanceof HTMLElement)return element;
  }
  return null;
}
function discoverRegistryScopes(node){
  if(!(node instanceof Element||node instanceof Document||node instanceof ShadowRoot))return;
  const add=scope=>{
    if(registryScopes.has(scope))return;
    registryScopes.add(scope);observeRegistryScope(scope);
    scope.addEventListener('slotchange',reportObserve,{signal});
    discoverRegistryScopes(scope);
  };
  if(node instanceof Element&&node.shadowRoot)add(node.shadowRoot);
  const containing=node.getRootNode();
  if(containing instanceof ShadowRoot&&containing.host.isConnected)add(containing);
  for(const element of node.querySelectorAll('*'))if(element.shadowRoot)add(element.shadowRoot);
}
function transport(){
  let fallback=null,owned=null;
  for(const n of registryElements('input[data-ix-overlay-transport]')){
    if(!(n instanceof HTMLInputElement)||!n.isConnected||n.getAttribute('data-ix-overlay-transport')!==owner)continue;
    // A root-owned transport remains on the renderer's real delegation path,
    // including when assigned light items are currently outside that path.
    if(n.closest('[data-ix-overlay-root]')){
      if(n.getRootNode() instanceof ShadowRoot)return n;
      owned||=n;
    }
    fallback||=n;
  }
  return owned||fallback;
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
  for(const n of registryElements('[data-ix-overlay-host]'))if(n instanceof HTMLElement&&n.isConnected&&n.getAttribute('data-ix-overlay-host')===id&&isActiveHost(n))return n;
  return null;
}
function findItemsHost(root,id){
  const configured=root.getAttribute('data-ix-overlay-items-host-id'),element=configured?registryElement(configured):null;
  return element instanceof HTMLElement&&element.isConnected?element:findHost(root.getAttribute('data-ix-overlay-items-owner')||id);
}
// Trigger metadata and anchor style are rendered by the owning Dioxus
// component. This lookup observes current markers, including cross-owner
// controls; an ID alone never installs behavior or supplies a fallback.
function findTrigger(id){
  for(const element of registryElements('[data-ix-overlay-trigger]'))if(element instanceof HTMLElement&&element.isConnected&&element.getAttribute('data-ix-overlay-trigger')===id)return element;
  return null;
}
function findAnchor(root){
  const id=root?.getAttribute('data-ix-overlay-anchor-id'),el=id?document.getElementById(id):null;
  return el instanceof HTMLElement&&el.isConnected?el:null;
}
function triggerAncestors(trigger,selfId){
  const out=[];
  for(let c=stepParent(trigger);c;c=stepParent(c))if(c instanceof HTMLElement&&isActiveHost(c)){
    const hid=c.getAttribute('data-ix-overlay-host');
    if(hid&&hid!==selfId&&!out.includes(hid))out.push(hid);
  }
  return out;
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
function hostItems(host){
  const out=[];if(!host)return out;
  for(const el of host.querySelectorAll('[data-ix-dropdown-item]')){
    if(!(el instanceof HTMLElement)||nearestHost(el)!==host||el.getAttribute('data-ix-dropdown-owner')!==host.getAttribute('data-ix-overlay-host')||el.matches('[hidden],[disabled],[aria-disabled="true"]'))continue;
    const num=Number(el.getAttribute('data-ix-dropdown-item'));
    if(Number.isInteger(num)&&num>=0)out.push({num,el});
  }
  return out;
}
const itemSelector='[data-ix-item-kind],[data-ix-roving-item]';
function itemParts(element){
  return Array.from(element.querySelectorAll('[data-ix-item-part]')).filter(part=>part instanceof HTMLElement&&part.closest('[data-ix-item-kind]')===element&&['select-option','category-parent'].includes(part.getAttribute('data-ix-item-part')));
}
// The alpha interpreter writes checked as a property even on component DIV
// hosts. Copy the literal Rust declaration once per changed marker. Remembering
// copied bytes avoids overwriting independent consumer DOM edits on every
// observation; eligibility remains the resulting actual own attribute in Rust.
const reflectedCheckedAttributes=new WeakMap();
function syncReflectedCheckedAttributes(){
  for(const element of registryElements('[data-ix-reflect-checked]')){
    if(!(element instanceof HTMLElement)||element.closest('[data-ix-item-kind][data-ix-item-runtime]')?.getAttribute('data-ix-item-runtime')!==owner)continue;
    const marker=element.getAttribute('data-ix-reflect-checked');
    if(reflectedCheckedAttributes.get(element)===marker)continue;
    let value;try{value=JSON.parse(marker);}catch{continue;}
    if(value!==null&&typeof value!=='string')continue;
    reflectedCheckedAttributes.set(element,marker);
    if(value===null){if(element.hasAttribute('checked'))element.removeAttribute('checked');}
    else if(element.getAttribute('checked')!==value)element.setAttribute('checked',value);
  }
}
function itemRootPath(element){
  const roots=[];
  for(let current=element;current;current=stepParent(current))if(current instanceof HTMLElement&&current.hasAttribute('data-ix-overlay-root')&&current.getAttribute('data-ix-overlay-owner')===owner){
    roots.push({id:current.getAttribute('data-ix-overlay-root'),generation:Number(current.getAttribute('data-ix-overlay-generation')),is_host:current===element});
  }
  return roots;
}
function itemHostFacts(element){
  const binding=captureItem(element);if(!binding)return null;
  const rawKind=element.getAttribute('data-ix-item-kind');
  const kind=['dropdown','select','menu','category'].includes(rawKind)?rawKind:'native-opt-in';
  return {item:binding.capture,kind,target:binding.target,capture:binding.capture,root_path:itemRootPath(element),
    registration_key:element.getAttribute('data-ix-item-key')||'',
    source_focusable:element.hasAttribute('data-ix-item-focusable')?element.getAttribute('data-ix-item-focusable')==='true':element.classList.contains('ix-focusable'),
    disabled_attribute:element.hasAttribute('disabled'),hidden_attribute:element.hasAttribute('hidden'),
    tabindex_attribute:element.getAttribute('tabindex'),checked_attribute:element.hasAttribute('checked'),
    visible_focus:element.getAttribute('data-active')==='true'||element.classList.contains('ix-focused'),
    parts:itemParts(element).flatMap(part=>{const binding=captureItem(part);return binding?[{kind:part.getAttribute('data-ix-item-part'),capture:binding.capture,source_focusable:part.hasAttribute('data-ix-item-focusable')?part.getAttribute('data-ix-item-focusable')==='true':part.classList.contains('ix-focusable')}]:[];}),
    roving_opt_in:element.hasAttribute('data-ix-roving-item'),linked_trigger:element.hasAttribute('data-ix-overlay-trigger')?element.getAttribute('data-ix-overlay-trigger'):null};
}
function inventoryFacts(host){
  if(!(host instanceof HTMLElement))return null;
  const identity=captureItem(host),direct=Array.from(host.querySelectorAll(itemSelector)).filter(element=>element instanceof HTMLElement);
  const elements=new Set(direct),rawSlots=[];
  for(const slot of host.querySelectorAll('slot')){
    const assignments=[];
    for(const assigned of slot.assignedElements({flatten:true})){
      const root=assigned.matches(itemSelector)?assigned:null;
      const descendants=Array.from(assigned.querySelectorAll(itemSelector)).filter(element=>element instanceof HTMLElement);
      if(root)elements.add(root);for(const element of descendants)elements.add(element);
      assignments.push({root,descendants});
    }
    rawSlots.push({slot,assignments});
  }
  const facts=new Map(Array.from(elements).map(element=>[element,itemHostFacts(element)]));
  const token=element=>facts.get(element)?.item||null,order=direct.map(token).filter(Boolean);
  return {items_host:identity.capture,hosts:Array.from(facts.values()).filter(Boolean),navigation_order:order,query_order:order,
    slots:rawSlots.map(({slot,assignments})=>({
      assignments:assignments.map(assignment=>({root:token(assignment.root),descendants:assignment.descendants.map(token).filter(Boolean)})),
      following:Array.from(elements).filter(element=>slot.compareDocumentPosition(element)&Node.DOCUMENT_POSITION_FOLLOWING).map(token).filter(Boolean)
    }))};
}
function findFocusedIndex(active,list){
  if(!active)return null;
  const idx=list.indexOf(active);if(idx!==-1)return idx;
  for(let i=0;i<list.length;i++)if(composedContains(list[i],active))return i;
  return null;
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
function collectFacts(kind,event,phase=''){
  for(const site of event?.composedPath?.()||[])if(site instanceof ShadowRoot&&!registryScopes.has(site))discoverRegistryScopes(site);
  for(const binding of itemCaptureNodes.values())binding.current=false;
  const active=deepActive(),path=event?.composedPath?.()||[];
  const target=path[0] instanceof HTMLElement?path[0]:(event?.target instanceof HTMLElement?event.target:null);
  const hosts=pathIds(path,'data-ix-overlay-host',isActiveHost),triggers=pathIds(path,'data-ix-overlay-trigger');
  const nodes=[],anchors=[];
  for(const root of registryElements('[data-ix-overlay-root]')){
    if(!(root instanceof HTMLElement)||!root.isConnected)continue;
    const ro=root.getAttribute('data-ix-overlay-owner'),id=root.getAttribute('data-ix-overlay-root')||'';
    if(!id||(!['window','trap'].includes(phase)&&event?.key!=='Tab'&&ro!==owner))continue;
    const generation=Number(root.getAttribute('data-ix-overlay-generation')||root.getAttribute('data-generation')||'0')||0;
    const declared_ready=root.getAttribute('data-ix-overlay-ready')!=='false';
    const open=root.hasAttribute('data-ix-overlay-open')?root.getAttribute('data-ix-overlay-open')==='true':root.getAttribute('data-open')==='true';
    const host=findHost(id),itemsHost=findItemsHost(root,id),trigger=findTrigger(id),anchor=findAnchor(root),itemsWithEl=hostItems(itemsHost),focusables=hostFocusables(itemsHost);
    // Dioxus renders trigger metadata before MountedEvent notifications.
    // Current marked-control presence, not a later mounted-handle render,
    // admits Dropdown ancestry. An ID-only control never supplies this marker.
    const discovery_ready=declared_ready&&(!root.hasAttribute('data-ix-dropdown-focus-owned')||!!trigger);
    if(anchor&&path.some(node=>node instanceof Node&&composedContains(anchor,node)))anchors.push(id);
    let focused_item=null;
    const probe=target&&itemsHost&&composedContains(itemsHost,target)?target:active;
    if(probe)for(const e of itemsWithEl)if(composedContains(e.el,probe)){focused_item=e.num;break;}
    const focused_index=findFocusedIndex(active,focusables);
    const on_trigger=!!trigger&&(event?target===trigger:active===trigger);
    const inside=!!host&&(hosts.includes(id)||(target?composedContains(host,target):false)||(active?composedContains(host,active):false));
    // Cache current candidates on ordinary observations as well: typed Dioxus
    // Tab handlers consume these facts without a native key transport.
    const tab_candidates=root.getAttribute('data-ix-overlay-native-surface')==='popover'?popoverTabFacts(host,active):[];
    const inventory=inventoryFacts(itemsHost),own_inventory=itemsHost===host?null:inventoryFacts(host);
    nodes.push({
      id,owner:ro||'',generation,root_capture:captureItem(root)?.capture,trigger_capture:captureItem(trigger)?.capture,connected:root.isConnected,discovery_ready,open,mode:root.getAttribute('data-ix-overlay-mode')||'',trigger_id:nativeId(trigger),
      anchor_in_dropdown_item:!!(anchor||trigger)?.closest('[data-ix-dropdown-item]'),
      trigger_ancestors:trigger||anchor?triggerAncestors(trigger||anchor,id):[],
      assignment_ancestors:trigger?triggerAncestors(trigger,id):[],
      header_controls:isHeaderPopupHost(host)?headerControls(root):[],
      items:itemsWithEl.map(e=>e.num),focused_item,
      inventory,own_inventory,trigger_on_path:!!trigger&&path.includes(trigger),
      // Pinned MountedData can focus a registered handle, but cannot enumerate
      // current native Tab order. Report IDs only; Dropdown never executes focus here.
      focusable_ids:focusables.map(nativeId).filter(Boolean),
      focus_count:focusables.length,focused_index,on_trigger,inside,
      trap_enabled:root.getAttribute('data-ix-popover-focus-trap')==='true',tab_candidates
    });
  }
  const document_tab_candidates=documentTabCandidates();
  return {
    owner,document:nativeDocument.identity,window_sequence:phase==='window'?windowSequence(event):0,sequence:++sequence,kind,phase,input_kind:event?.type||'',
    key:event?.key||'',shift:!!event?.shiftKey,alt:!!event?.altKey,ctrl:!!event?.ctrlKey,meta:!!event?.metaKey,
    default_prevented:!!event?.defaultPrevented,
    target_id:nativeId(target),active_id:nativeId(active),
    hosts,triggers,anchors,nodes,document_tab_candidates,tab_order:document_tab_candidates.map(candidate=>candidate.target),
    active_hosts:hostAncestors(active),active_popover:nearestPopover(active)?.getAttribute('data-ix-overlay-host')||''
  };
}
function send(report,event){
  const input=transport();
  if(!alive||!input)return;
  const value=JSON.stringify(report);
  input.value=value;
  const prevent=!input.dispatchEvent(new Event('input',{bubbles:true,cancelable:true}));
  const ackInput=transport();if(!ackInput)return;
  ackInput.value=value;
  const stop=!ackInput.dispatchEvent(new Event('change',{bubbles:true,cancelable:true}));
  let immediate=false;
  if(report.phase==='trap'){
    const immediateInput=transport();if(!immediateInput)return;
    immediateInput.value=JSON.stringify({owner,sequence:report.sequence,immediate_stop:true});
    immediate=!immediateInput.dispatchEvent(new Event('change',{bubbles:true,cancelable:true}));
  }
  if(event){if(prevent&&event.cancelable)event.preventDefault();if(immediate)event.stopImmediatePropagation();else if(stop)event.stopPropagation();}
  pruneItemCaptures();
}

function syncNativeSurfaces(){
  for(const root of registryElements('[data-ix-overlay-root]')){
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
function pruneInventoryLeases(){
  for(const [id,lease]of itemInventoryLeases){
    const current=Array.from(registryElements('[data-ix-overlay-root]')).find(root=>root.getAttribute('data-ix-overlay-owner')===owner&&root.getAttribute('data-ix-overlay-root')===id);
    if(!current||Number(current.getAttribute('data-ix-overlay-generation'))!==lease.generation)itemInventoryLeases.delete(id);
  }
}
function reportObserve(){
  if(!alive)return;
  syncReflectedCheckedAttributes();
  pruneInventoryLeases();
  if(!transport())return;
  for(const input of registryElements('input[data-ix-overlay-transport]'))if(input.getAttribute('data-ix-overlay-transport')===owner)input.setAttribute('data-ix-overlay-native','true');
  syncNativeSurfaces();
  const report=collectFacts('observe',null);
  const sig=JSON.stringify({nodes:report.nodes,active_id:report.active_id,document_tab_candidates:report.document_tab_candidates});
  if(sig!==lastSignature){lastSignature=sig;send(report,null);}
  pruneItemCaptures();
}
// Generic Header/Popover/global coordination retains current key/Tab facts.
// Dropdown/Header local clicks and Dropdown keys use Dioxus handlers only.
// The remaining window outside listener observes document events beyond any
// component root. Pinned Document has no typed global-event registration; this
// legacy integration is the sole outside path, never an activation fallback.
document.addEventListener('keydown',e=>send(collectFacts('key',e,'capture'),e),{capture:true,signal});
document.addEventListener('keydown',e=>{if(e.key==='Tab')send(collectFacts('key',e,'trap'),e);},{signal});
for(const kind of ['keydown','mousedown','touchstart'])document.addEventListener(kind,e=>send(collectFacts('modality',e,'document'),null),{passive:true,signal});
window.addEventListener('keydown',e=>send(collectFacts('key',e,'window'),e),{signal});
window.addEventListener('click',e=>send(collectFacts('click',e,'window'),e),{signal});
document.addEventListener('pointerdown',e=>send(collectFacts('pointer',e),e),{capture:true,signal});
document.addEventListener('focusin',reportObserve,{signal});
// Generic overlay discovery/dismissal still observes current focus until its
// R02 event migration. Typed item cues are owned by Dioxus focus/blur handlers.
document.addEventListener('focusout',()=>{queueMicrotask(()=>{if(alive)reportObserve();});},{passive:true,signal});
const observer=new MutationObserver(changes=>{
  // Shared read-only discovery remains a separate later R02 boundary.
  if(changes.some(change=>change.type==='childList')){
    for(const change of changes)if(change.type==='childList'){
      for(const node of [...change.addedNodes,...change.removedNodes])discoverRegistryScopes(node);
    }
    // Stop observing detached consumer trees; reattachment is discovered from
    // its real insertion or event path, with no retained closed-root access.
    if(Array.from(registryScopes).some(scope=>scope!==document&&!scope.host.isConnected)){
      observer.disconnect();
      for(const scope of Array.from(registryScopes)){
        if(scope!==document&&!scope.host.isConnected){scope.removeEventListener('slotchange',reportObserve);registryScopes.delete(scope);}
        else observeRegistryScope(scope);
      }
    }
  }
  reportObserve();
});
function observeRegistryScope(scope){observer.observe(scope===document?document.documentElement:scope,{subtree:true,childList:true,attributes:true,attributeFilter:[
  'data-ix-overlay-root','data-ix-overlay-owner','data-ix-overlay-host','data-ix-overlay-trigger','data-ix-overlay-anchor-id','data-ix-overlay-generation','data-ix-overlay-open',
  'data-ix-overlay-mode','data-ix-overlay-native-surface','data-ix-overlay-items-owner','data-ix-overlay-ready','data-ix-overlay-revision','data-ix-overlay-top-layer','data-small','data-open','hidden','disabled','tabindex','inert','class'
  ,'data-ix-item-kind','data-ix-item-key','data-ix-item-focusable','data-active','data-ix-roving-item','checked'
  ,'data-ix-overlay-items-host-id','data-ix-item-runtime','data-ix-item-part','aria-activedescendant','data-ix-overlay-active-item','data-ix-reflect-checked','slot','name'
]});}
observeRegistryScope(document);discoverRegistryScopes(document);
document.documentElement.setAttribute('data-ix-overlay-registry-native',owner);
const inventoryReader=(id,generation)=>{
  const root=Array.from(registryElements('[data-ix-overlay-root]')).find(root=>root instanceof HTMLElement&&root.isConnected&&root.getAttribute('data-ix-overlay-owner')===owner&&root.getAttribute('data-ix-overlay-root')===id&&Number(root.getAttribute('data-ix-overlay-generation'))===generation);
  if(!alive||!root)return null;
  const facts=inventoryFacts(findItemsHost(root,id));
  if(facts)itemInventoryLeases.set(id,{generation,tokens:new Set([facts.items_host,...facts.hosts.flatMap(host=>[host.capture,...host.parts.map(part=>part.capture)])])});
  return facts;
};
nativeDocument.inventoryReaders.set(owner,inventoryReader);
try{
  reportObserve();
  while(alive){const command=await dioxus.recv();if(command?.finish)break;
  }
}finally{
  alive=false;listeners.abort();observer.disconnect();
  itemCaptureNodes.clear();
  itemInventoryLeases.clear();if(nativeDocument.inventoryReaders.get(owner)===inventoryReader)nativeDocument.inventoryReaders.delete(owner);
  for(const input of registryElements('input[data-ix-overlay-transport]'))if(input.getAttribute('data-ix-overlay-transport')===owner)input.removeAttribute('data-ix-overlay-native');
  registryScopes.clear();
  if(document.documentElement.getAttribute('data-ix-overlay-registry-native')===owner)document.documentElement.removeAttribute('data-ix-overlay-registry-native');
}
"#;
