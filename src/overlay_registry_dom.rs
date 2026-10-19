// SPDX-FileCopyrightText: 2026 Siemens AG
// SPDX-License-Identifier: MIT

//! Native DOM observations and execution for the per-root overlay runtime.
//! Document identity and original-event identity are shared across roots. Local
//! reports retain the root partition; Window reports expose every native overlay
//! in that document so Rust can coordinate them without native policy.
//! `__OWNER__` is replaced by a serialized runtime owner ID. Rust owns all
//! hierarchy, discovery caches, menu/focus policy, and open/close decisions;
//! `input` dispatches a raw fact report (`prevent` ACK) and `change` queries
//! either the saved `stop` ACK (identical report JSON) or consume-once native
//! click target authorization (`activate_target` with matching `sequence`).

pub(super) const BRIDGE: &str = r#"
const owner=__OWNER__,listeners=new AbortController(),signal=listeners.signal;
const documentSlot=Symbol.for('dioxus-ix.document-native-domain.v1');
const nativeDocument=document[documentSlot]||(document[documentSlot]={identity:owner,eventSequence:0,eventIds:new WeakMap()});
nativeDocument.inventoryReaders||=new Map();
nativeDocument.triggerReaders||=new Map();
nativeDocument.focusReaders||=new Map();
function windowSequence(event){if(!event)return 0;let id=nativeDocument.eventIds.get(event);if(!id){id=++nativeDocument.eventSequence;nativeDocument.eventIds.set(event,id);}return id;}
const focusableSelector='a[href],area[href],audio,button,input:not([type="hidden"]),select,textarea,iframe,object,embed,video,[tabindex],[contenteditable],ix-dropdown-item,ix-select-item';
let alive=true,sequence=0,lastSignature='';
const keyTargets=new Map(),clickTargets=new Map(),triggerKeyTargets=new Map(),navigationTargets=new Map(),handledBubbleKeys=new WeakSet();
const navigationBindings=new Map();
const anchorObservers=new Map();
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
function findElement(id){const el=registryElement(id);return el instanceof HTMLElement&&el.isConnected?el:null;}
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
// Rust publishes committed declaration/disposal versions. Each attempt retains
// its own ID and may resolve later; a newer attempt never cancels an older wait.
// Current resolution, installed trigger handlers and AD navigation are distinct.
const declaredTriggers=new Map();
function triggerBindingAlive(binding){
  return alive&&binding.root.isConnected&&binding.root.getAttribute('data-ix-overlay-owner')===owner&&binding.root.getAttribute('data-ix-overlay-root')===binding.id&&binding.root.getAttribute('data-ix-overlay-generation')===binding.generation;
}
function resolveTriggerAttempt(binding,attempt){
  if(!triggerBindingAlive(binding))return;
  // None is the source undefined reference. Empty/invalid CSS-selector error
  // behavior is an API adapter; it must not clear a previous actual binding.
  const element=document.getElementById(attempt.declaration===null?'undefined':attempt.declaration);
  if(!(element instanceof HTMLElement))return;
  binding.pending.delete(attempt.version);
  binding.current=element;
  if(!binding.installed)binding.installed=element;
  element.setAttribute('data-ix-overlay-trigger',binding.id);
  const anchor=binding.root.getAttribute('data-ix-overlay-anchor-name');
  if(anchor)element.style.setProperty('anchor-name',anchor);
  // Capture the actual assignment path ONCE at this resolution. Later movement
  // changes live ancestry, never this historical event or its queue order.
  binding.assignments.push(Object.freeze({sequence:++binding.assignmentSequence,ancestors:Object.freeze(triggerAncestors(element,binding.id))}));
}
function configuredTriggerBinding(id,root){
  const version=root?.getAttribute('data-ix-overlay-trigger-version');
  if(version===null||version===undefined)return null;
  const generation=root.getAttribute('data-ix-overlay-generation')||'0';
  let binding=declaredTriggers.get(root);
  if(!binding?.pending||binding.id!==id||binding.generation!==generation){
    binding={root,id,generation,version:null,disposeVersion:null,current:null,installed:null,pending:new Map(),assignmentSequence:0,assignments:[]};
    declaredTriggers.set(root,binding);
  }
  const disposeVersion=root.getAttribute('data-ix-overlay-trigger-dispose-version')||'0';
  if(binding.disposeVersion!==null&&binding.disposeVersion!==disposeVersion)binding.installed=null;
  binding.disposeVersion=disposeVersion;
  if(binding.version!==version){
    binding.version=version;
    const attempt=Object.freeze({version,declaration:root.getAttribute('data-ix-overlay-trigger-id')});
    binding.pending.set(version,attempt);resolveTriggerAttempt(binding,attempt);
  }
  return binding;
}
function retryTriggerAttempts(){
  for(const binding of declaredTriggers.values())if(binding.pending&&triggerBindingAlive(binding))for(const attempt of Array.from(binding.pending.values()))resolveTriggerAttempt(binding,attempt);
}
function acknowledgeTriggerAssignment(command){
  for(const binding of declaredTriggers.values())if(binding.assignments&&binding.id===command.root&&Number(binding.generation)===command.generation){
    binding.assignments=binding.assignments.filter(emission=>emission.sequence>command.sequence);
  }
}
function findTrigger(id,root){
  const rootOwner=root?.getAttribute('data-ix-overlay-owner');
  if(rootOwner&&rootOwner!==owner){const reader=nativeDocument.triggerReaders.get(rootOwner);return reader?reader(id,root):null;}
  const binding=configuredTriggerBinding(id,root);
  if(binding)return binding.current;
  // Compatibility for renderers/owners without committed-binding metadata.
  const declared=root?.getAttribute('data-ix-overlay-trigger-id');
  if(declared){
    let legacy=declaredTriggers.get(root);
    const generation=root.getAttribute('data-ix-overlay-generation')||'0',key=id+'@'+generation+'@'+declared;
    if(!legacy||legacy.key!==key){legacy={key,root,id,generation,current:null,installed:null};declaredTriggers.set(root,legacy);}
    if(!legacy.current){const element=document.getElementById(declared);if(element instanceof HTMLElement){legacy.current=element;legacy.installed=element;element.setAttribute('data-ix-overlay-trigger',id);const anchor=root.getAttribute('data-ix-overlay-anchor-name');if(anchor)element.style.setProperty('anchor-name',anchor);}}
    return legacy.current;
  }
  for(const n of registryElements('[data-ix-overlay-trigger]'))if(n instanceof HTMLElement&&n.isConnected&&n.getAttribute('data-ix-overlay-trigger')===id)return n;
  const tid=root?.getAttribute('data-ix-overlay-trigger-id'),el=tid?document.getElementById(tid):null;
  return el instanceof HTMLElement&&el.isConnected?el:null;
}
const triggerReader=(id,root)=>findTrigger(id,root);nativeDocument.triggerReaders.set(owner,triggerReader);
function navigationBindingFor(root){
  const state=navigationBindings.get(root.getAttribute('data-ix-overlay-root'));
  return state&&state.root===root&&state.generation===Number(root.getAttribute('data-ix-overlay-generation'))?state:null;
}
function bindNavigationConfigurations(){
  for(const state of navigationBindings.values()){
    const root=state.root||Array.from(registryElements('[data-ix-overlay-root]')).find(element=>element instanceof HTMLElement&&element.isConnected&&element.getAttribute('data-ix-overlay-owner')===owner&&element.getAttribute('data-ix-overlay-root')===state.id&&Number(element.getAttribute('data-ix-overlay-generation'))===state.generation);
    if(!root||!root.isConnected||root.getAttribute('data-ix-overlay-owner')!==owner||Number(root.getAttribute('data-ix-overlay-generation'))!==state.generation)continue;
    state.root=root;
    // Even disabled/legacy configurations retain their actual registration for
    // resource cleanup, without acquiring this configured-string event route.
    if(!state.enabled||state.mode!=='active-descendant'||state.element||!root.hasAttribute('data-ix-overlay-trigger-version'))continue;
    const element=findTrigger(state.id,root)||findAnchor(root),captured=captureItem(element);
    if(captured){state.element=element;state.capture=captured.capture;}
  }
}
function configureNavigation(command){
  const previous=navigationBindings.get(command.root);
  if(previous&&previous.generation===command.generation&&previous.version===command.version)return;
  navigationBindings.set(command.root,{id:command.root,generation:command.generation,version:command.version,mode:command.mode,enabled:command.enabled,root:null,element:null,capture:null});
  bindNavigationConfigurations();syncKeyTargets();reportObserve();
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
  const enclosing_triggers=triggers.filter(id=>{
    const triggerIndex=path.findIndex(node=>node instanceof HTMLElement&&node.getAttribute('data-ix-overlay-trigger')===id);
    const hostIndex=path.findIndex(node=>node instanceof HTMLElement&&isActiveHost(node)&&node.getAttribute('data-ix-overlay-host')===id);
    return hostIndex>=0&&triggerIndex>hostIndex;
  });
  // Preserve actual receiver order and identity. Rust decides which physical
  // root/host sites correspond to the source component's click handlers.
  const click_sites=[];
  if(kind==='click'&&phase==='bubble')for(const receiver of path){
    if(!(receiver instanceof HTMLElement))continue;
    const host=isActiveHost(receiver),id=receiver.getAttribute(host?'data-ix-overlay-host':'data-ix-overlay-root');
    if(!id)continue;
    const registration=path.find(root=>root instanceof HTMLElement&&root.getAttribute('data-ix-overlay-root')===id&&root.getAttribute('data-ix-overlay-owner')===owner);
    if(!registration)continue;
    click_sites.push({root:id,generation:Number(registration.getAttribute('data-ix-overlay-generation')||'0'),connected:registration.isConnected&&receiver.isConnected,receiver:host?'host':'root',popover:receiver.hasAttribute('popover'),target_is_host:host&&target===receiver});
  }
  const candidate_ids=[],addCand=el=>{if(el instanceof HTMLElement&&el.isConnected&&el.id&&!candidate_ids.includes(el.id))candidate_ids.push(el.id);};
  if(target)addCand(target);
  if(active)addCand(active);
  for(const p of path)if(p instanceof HTMLElement)addCand(p);
  const active_categories=[];
  for(let c=active;c;c=stepParent(c))if(c instanceof Element&&c.hasAttribute('data-menu-category-root')){const id=Number(c.getAttribute('data-menu-category-root'));if(Number.isSafeInteger(id)&&id>0&&!active_categories.includes(id))active_categories.push(id);}
  const nodes=[],anchors=[];
  for(const root of registryElements('[data-ix-overlay-root]')){
    if(!(root instanceof HTMLElement)||!root.isConnected)continue;
    const ro=root.getAttribute('data-ix-overlay-owner'),id=root.getAttribute('data-ix-overlay-root')||'';
    if(!id||(!['window','trap'].includes(phase)&&event?.key!=='Tab'&&ro!==owner))continue;
    const generation=Number(root.getAttribute('data-ix-overlay-generation')||root.getAttribute('data-generation')||'0')||0;
    const declared_ready=root.getAttribute('data-ix-overlay-ready')!=='false';
    const open=root.hasAttribute('data-ix-overlay-open')?root.getAttribute('data-ix-overlay-open')==='true':root.getAttribute('data-open')==='true';
    const host=findHost(id),itemsHost=findItemsHost(root,id),trigger=findTrigger(id,root),triggerBinding=declaredTriggers.get(root),installedTrigger=triggerBinding?.installed||(!root.hasAttribute('data-ix-overlay-trigger-version')?trigger:null),navigationBinding=navigationBindingFor(root),anchor=findAnchor(root),itemsWithEl=hostItems(itemsHost),focusables=hostFocusables(itemsHost);
    const discovery_ready=declared_ready&&(!root.hasAttribute('data-ix-overlay-trigger-id')||!!trigger);
    if(anchor&&path.some(node=>node instanceof Node&&composedContains(anchor,node)))anchors.push(id);
    if(trigger)addCand(trigger);
    for(const e of itemsWithEl)addCand(e.el);
    for(const el of focusables)addCand(el);
    let focused_item=null;
    const probe=target&&itemsHost&&composedContains(itemsHost,target)?target:active;
    if(probe)for(const e of itemsWithEl)if(composedContains(e.el,probe)){focused_item=e.num;break;}
    const focused_index=findFocusedIndex(active,focusables);
    const on_trigger=!!trigger&&(event?target===trigger:active===trigger);
    const inside=!!host&&(hosts.includes(id)||(target?composedContains(host,target):false)||(active?composedContains(host,active):false));
    const tab_candidates=event?.key==='Tab'&&root.getAttribute('data-ix-overlay-native-surface')==='popover'?popoverTabFacts(host,active):[];
    const inventory=inventoryFacts(itemsHost),own_inventory=itemsHost===host?null:inventoryFacts(host);
    for(const item of inventory?.hosts||[])if(item.target)addCand(findElement(item.target));
    nodes.push({
      id,owner:ro||'',generation,root_capture:captureItem(root)?.capture,trigger_capture:captureItem(trigger)?.capture,connected:root.isConnected,discovery_ready,open,mode:root.getAttribute('data-ix-overlay-mode')||'',trigger_id:nativeId(trigger),anchor_id:nativeId(anchor),
      anchor_in_dropdown_item:!!(anchor||trigger)?.closest('[data-ix-dropdown-item]'),
      trigger_ancestors:trigger||anchor?triggerAncestors(trigger||anchor,id):[],
      assignment_ancestors:trigger?triggerAncestors(trigger,id):[],
      header_controls:isHeaderPopupHost(host)?headerControls(root):[],
      items:itemsWithEl.map(e=>e.num),focused_item,
      inventory,own_inventory,trigger_on_path:!!trigger&&path.includes(trigger),
      assignment_emissions:triggerBinding?.assignments??null,
      trigger_listener_capture:captureItem(installedTrigger)?.capture??null,
      configured_trigger_listener:(root.hasAttribute('data-ix-overlay-trigger-version')||root.hasAttribute('data-ix-overlay-trigger-id'))&&!!installedTrigger&&event?.currentTarget===installedTrigger?captureItem(event.currentTarget)?.capture:null,
      navigation_binding:navigationBinding?.enabled&&navigationBinding.element?{version:navigationBinding.version,capture:captureItem(navigationBinding.element)?.capture}:null,
      navigation_listener:navigationBinding?.enabled&&navigationBinding.element&&event?.currentTarget===navigationBinding.element?{version:navigationBinding.version,capture:captureItem(event.currentTarget)?.capture}:null,
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
    hosts,triggers,enclosing_triggers,anchors,nodes,click_sites,active_categories,document_tab_candidates,tab_order:document_tab_candidates.map(candidate=>candidate.target),candidate_ids,
    active_hosts:hostAncestors(active),active_popover:nearestPopover(active)?.getAttribute('data-ix-overlay-host')||'',
    item_focus_host:kind==='item-focusin'?(()=>{
      const hosts=path.filter(element=>element instanceof HTMLElement&&element.matches(itemSelector));
      const element=hosts.find(element=>element.getAttribute('data-ix-item-focusable')==='true'||element.classList.contains('ix-focusable'))||hosts[0];
      return element?itemHostFacts(element):null;
    })():null
  };
}
function send(report,event){
  const input=transport();
  if(!alive||!input)return;
  // Bind authorized activation to the element observed before Rust callbacks.
  const activationExecution=new Map(report.candidate_ids.map(id=>[id,findElement(id)]));
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
  if(report.kind==='key'&&report.candidate_ids.length){
    for(const id of report.candidate_ids){
      const el=activationExecution.get(id);
      if(!(el instanceof HTMLElement)||!el.isConnected)continue;
      const clickInput=transport();if(!clickInput)return;
      clickInput.value=JSON.stringify({owner,sequence:report.sequence,activate_target:id});
      if(!clickInput.dispatchEvent(new Event('change',{bubbles:true,cancelable:true}))){
        if(el.isConnected)el.click();
        break;
      }
    }
  }
  pruneItemCaptures();
}

// The source's global focus patch reports its exact attempted target after
// native focus, including a no-op. Document listeners instead see the retargeted
// target. Project only renderer-owned control/part boundaries into source hosts.
function itemFocusStart(target,documentEvent){
  if(!(target instanceof HTMLElement))return null;
  const control=target.closest('[data-ix-item-control]');
  if(control){
    const host=control.closest('[data-ix-item-kind]');
    if(host){
      if(control.getAttribute('data-ix-item-part')==='category-parent')return documentEvent?host:control;
      return host;
    }
  }
  return target;
}
function reportItemFocus(target,documentEvent=false){
  if(!alive)return;
  const start=itemFocusStart(target,documentEvent);if(!start)return;
  const path=[],hosts=[];
  // Pinned getComposedPath jumps directly to an actual shadow host; it never
  // flattens assigned slots or substitutes the deepest active element.
  for(let node=start;node instanceof Element;){
    if(node instanceof HTMLElement){
      const part=node.hasAttribute('data-ix-item-part')?node:null;
      const host=node.matches(itemSelector)?node:part?.closest(itemSelector);
      if(host?.getAttribute('data-ix-item-runtime')===owner){
        const binding=captureItem(node);if(binding)path.push(binding.capture);
        if(!hosts.some(facts=>facts.registration_key===host.getAttribute('data-ix-item-key')))hosts.push(itemHostFacts(host));
      }
    }
    const root=node.getRootNode();node=root instanceof ShadowRoot?root.host:node.parentElement;
  }
  const input=transport();if(!input)return;
  input.value=JSON.stringify({owner,sequence:0,item_focus_observation:{path,hosts}});
  input.dispatchEvent(new Event('change',{bubbles:true,cancelable:true}));
}
nativeDocument.focusReaders.set(owner,reportItemFocus);
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
function syncKeyTargets(){
  const wanted=new Set(),wantedClicks=new Set(),wantedTriggerKeys=new Set(),wantedNavigation=new Set();
  bindNavigationConfigurations();
  for(const root of registryElements('[data-ix-overlay-root]')){
    if(!(root instanceof HTMLElement)||!root.isConnected||root.getAttribute('data-ix-overlay-owner')!==owner)continue;
    const current=findTrigger(root.getAttribute('data-ix-overlay-root'),root),binding=declaredTriggers.get(root);
    const managed=root.hasAttribute('data-ix-overlay-trigger-version');
    const target=managed?binding?.installed:current;
    if(!target)continue;
    wantedClicks.add(target);
    if(!clickTargets.has(target)){
      const listener=event=>send(collectFacts('click',event,'trigger'),null);
      clickTargets.set(target,listener);target.addEventListener('click',listener,{signal});
    }
    if(root.getAttribute('data-ix-overlay-mode')!=='active-descendant')continue;
    if(managed){
      wantedTriggerKeys.add(target);
      if(!triggerKeyTargets.has(target)){
        const listener=event=>{handledBubbleKeys.add(event);send(collectFacts('key',event,'trigger-key'),event);};
        triggerKeyTargets.set(target,listener);target.addEventListener('keydown',listener,{signal});
      }
      continue;
    }
    wanted.add(target);
    if(!keyTargets.has(target)){
      const listener=event=>{if(handledBubbleKeys.has(event))return;handledBubbleKeys.add(event);send(collectFacts('key',event,'bubble'),event);};
      keyTargets.set(target,listener);target.addEventListener('keydown',listener,{signal});
    }
  }
  bindNavigationConfigurations();
  for(const [id,state]of navigationBindings){
    if(state.root&&(!state.root.isConnected||state.root.getAttribute('data-ix-overlay-owner')!==owner||Number(state.root.getAttribute('data-ix-overlay-generation'))!==state.generation)){navigationBindings.delete(id);continue;}
    if(!state.enabled||state.mode!=='active-descendant'||!state.element)continue;
    wantedNavigation.add(state.element);
    if(!navigationTargets.has(state.element)){
      // Both roles may run on the same event. The WeakSet only suppresses the
      // document fallback; it must never swallow another installed role.
      const listener=event=>{handledBubbleKeys.add(event);send(collectFacts('key',event,'navigation'),event);};
      navigationTargets.set(state.element,listener);state.element.addEventListener('keydown',listener,{signal});
    }
  }
  for(const [root,binding]of declaredTriggers)if(!triggerBindingAlive(binding))declaredTriggers.delete(root);
  for(const [id,lease]of itemInventoryLeases){
    const current=Array.from(registryElements('[data-ix-overlay-root]')).find(root=>root.getAttribute('data-ix-overlay-owner')===owner&&root.getAttribute('data-ix-overlay-root')===id);
    if(!current||Number(current.getAttribute('data-ix-overlay-generation'))!==lease.generation)itemInventoryLeases.delete(id);
  }
  for(const [target,listener]of keyTargets)if(!wanted.has(target)){target.removeEventListener('keydown',listener);keyTargets.delete(target);}
  for(const [target,listener]of clickTargets)if(!wantedClicks.has(target)){target.removeEventListener('click',listener);clickTargets.delete(target);}
  for(const [target,listener]of triggerKeyTargets)if(!wantedTriggerKeys.has(target)){target.removeEventListener('keydown',listener);triggerKeyTargets.delete(target);}
  for(const [target,listener]of navigationTargets)if(!wantedNavigation.has(target)){target.removeEventListener('keydown',listener);navigationTargets.delete(target);}
}
function syncAnchorObservers(){
  const wanted=new Set();
  for(const root of registryElements('[data-ix-overlay-root]')){
    if(!(root instanceof HTMLElement)||!root.isConnected||root.getAttribute('data-ix-overlay-owner')!==owner)continue;
    if(!root.hasAttribute('data-ix-overlay-mode')||root.getAttribute('data-ix-overlay-open')!=='true'||root.getAttribute('data-ix-overlay-trigger-visibility-check')==='false')continue;
    const id=root.getAttribute('data-ix-overlay-root'),target=findAnchor(root)||findTrigger(id,root),generation=root.getAttribute('data-ix-overlay-generation')||'0';
    if(!id||!(target instanceof HTMLElement)||typeof IntersectionObserver==='undefined')continue;
    wanted.add(id);const previous=anchorObservers.get(id);
    if(previous?.root===root&&previous.target===target&&previous.generation===generation)continue;
    previous?.observer.disconnect();
    const binding={root,target,generation,observer:null};
    binding.observer=new IntersectionObserver(entries=>{
      if(!alive||anchorObservers.get(id)!==binding||!root.isConnected||!target.isConnected)return;
      if(root.getAttribute('data-ix-overlay-open')!=='true'||root.getAttribute('data-ix-overlay-trigger-visibility-check')==='false'||root.getAttribute('data-ix-overlay-generation')!==generation||(findAnchor(root)||findTrigger(id,root))!==target)return;
      for(const entry of entries){
        if(entry.target!==target)continue;
        const report=collectFacts('anchor-visibility',null,'visibility');
        const rect=r=>({top:r.top,bottom:r.bottom,left:r.left,right:r.right});
        report.anchor_visibility={root:id,generation:Number(generation),target:nativeId(target),intersecting:entry.isIntersecting,
          geometry:{bounding:rect(entry.boundingClientRect),intersection:rect(entry.intersectionRect),anchor_in_dropdown_item:!!target.closest('[data-ix-dropdown-item]')}};
        send(report,null);
      }
    },{threshold:[0,0.5,1]});
    anchorObservers.set(id,binding);binding.observer.observe(target);
  }
  for(const [id,binding]of anchorObservers)if(!wanted.has(id)){binding.observer.disconnect();anchorObservers.delete(id);}
}
function reportObserve(){
  if(!alive)return;
  syncReflectedCheckedAttributes();
  syncKeyTargets();
  if(!transport())return;
  for(const input of registryElements('input[data-ix-overlay-transport]'))if(input.getAttribute('data-ix-overlay-transport')===owner)input.setAttribute('data-ix-overlay-native','true');
  syncNativeSurfaces();
  syncAnchorObservers();
  const report=collectFacts('observe',null);
  const sig=JSON.stringify({nodes:report.nodes,active_id:report.active_id,document_tab_candidates:report.document_tab_candidates});
  if(sig!==lastSignature){lastSignature=sig;send(report,null);}
  pruneItemCaptures();
}
document.addEventListener('keydown',e=>send(collectFacts('key',e,'capture'),e),{capture:true,signal});
document.addEventListener('keydown',e=>{if(!handledBubbleKeys.has(e))send(collectFacts('key',e,'fallback'),e);},{signal});
document.addEventListener('keydown',e=>{if(e.key==='Tab')send(collectFacts('key',e,'trap'),e);},{signal});
for(const kind of ['keydown','mousedown','touchstart'])document.addEventListener(kind,e=>send(collectFacts('modality',e,'document'),null),{passive:true,signal});
document.addEventListener('click',e=>send(collectFacts('click',e,'capture'),e),{capture:true,signal});
document.addEventListener('click',e=>send(collectFacts('click',e,'bubble'),e),{signal});
window.addEventListener('keydown',e=>send(collectFacts('key',e,'window'),e),{signal});
window.addEventListener('click',e=>send(collectFacts('click',e,'window'),e),{signal});
document.addEventListener('pointerdown',e=>send(collectFacts('pointer',e),e),{capture:true,signal});
document.addEventListener('focusin',event=>reportItemFocus(event.target,true),{capture:true,signal});
document.addEventListener('focusin',reportObserve,{signal});
document.addEventListener('focusout',event=>{reportItemFocus(event.target,true);queueMicrotask(()=>{if(alive)reportObserve();});},{passive:true,signal});
const observer=new MutationObserver(changes=>{
  // Each outstanding source ID wait observes real child-list changes. Resolve
  // all retained attempts in their admission order, never only the latest ID.
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
    retryTriggerAttempts();
  }
  reportObserve();
});
function observeRegistryScope(scope){observer.observe(scope===document?document.documentElement:scope,{subtree:true,childList:true,attributes:true,attributeFilter:[
  'data-ix-overlay-root','data-ix-overlay-owner','data-ix-overlay-host','data-ix-overlay-trigger','data-ix-overlay-trigger-id','data-ix-overlay-trigger-version','data-ix-overlay-trigger-dispose-version','data-ix-overlay-anchor-id','data-ix-overlay-generation','data-ix-overlay-open',
  'data-ix-overlay-mode','data-ix-overlay-trigger-visibility-check','data-ix-overlay-native-surface','data-ix-overlay-items-owner','data-ix-overlay-ready','data-ix-overlay-revision','data-ix-overlay-top-layer','data-small','data-open','hidden','disabled','tabindex','inert','class'
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
    if(command?.assignment_ack){acknowledgeTriggerAssignment(command.assignment_ack);reportObserve();}
    if(command?.navigation_config)configureNavigation(command.navigation_config);
  }
}finally{
  alive=false;listeners.abort();keyTargets.clear();clickTargets.clear();triggerKeyTargets.clear();navigationTargets.clear();navigationBindings.clear();declaredTriggers.clear();observer.disconnect();
  for(const binding of anchorObservers.values())binding.observer.disconnect();anchorObservers.clear();
  itemCaptureNodes.clear();
  itemInventoryLeases.clear();if(nativeDocument.inventoryReaders.get(owner)===inventoryReader)nativeDocument.inventoryReaders.delete(owner);
  if(nativeDocument.triggerReaders.get(owner)===triggerReader)nativeDocument.triggerReaders.delete(owner);
  if(nativeDocument.focusReaders.get(owner)===reportItemFocus)nativeDocument.focusReaders.delete(owner);
  for(const input of registryElements('input[data-ix-overlay-transport]'))if(input.getAttribute('data-ix-overlay-transport')===owner)input.removeAttribute('data-ix-overlay-native');
  registryScopes.clear();
  if(document.documentElement.getAttribute('data-ix-overlay-registry-native')===owner)document.documentElement.removeAttribute('data-ix-overlay-registry-native');
}
"#;
