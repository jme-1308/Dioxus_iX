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
function windowSequence(event){if(!event)return 0;let id=nativeDocument.eventIds.get(event);if(!id){id=++nativeDocument.eventSequence;nativeDocument.eventIds.set(event,id);}return id;}
const focusableSelector='a[href],area[href],audio,button,input:not([type="hidden"]),select,textarea,iframe,object,embed,video,[tabindex],[contenteditable],ix-dropdown-item,ix-select-item';
let alive=true,sequence=0,lastSignature='',focusFrame=0;
const keyTargets=new Map(),clickTargets=new Map(),handledBubbleKeys=new WeakSet();
const tabFocusCaptures=new Map(),nativeFocusCommands=[],immediateTabFrames=new Set();let nativeFocusFrame=0;
const anchorObservers=new Map();
const itemObjectTokens=new WeakMap(),itemCaptureNodes=new Map(),itemScrollFrames=new Set();let itemObjectSequence=0;
const itemInventoryLeases=new Map();
let reconcilingItemCues=false,itemCueReconcileQueued=false,itemCueRetainedRequested=false,itemCueForcedRequested=false;
const itemCueCapturedPending=new Set();
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
    input.value=JSON.stringify({owner,sequence:0,item_focus_phase:'retain',item_focus_capture:capture});
    if(input.dispatchEvent(new Event('change',{bubbles:true,cancelable:true})))itemCaptureNodes.delete(capture);
  }
}
const lastFocus=new Map(),nativeIds=new WeakMap(),nativeNodes=new Map();let nativeSequence=0;
function nativeId(el){if(!(el instanceof HTMLElement))return '';if(el.id)return el.id;let id=nativeIds.get(el);if(!id){id=owner+'-native-'+(++nativeSequence);nativeIds.set(el,id);}nativeNodes.set(id,el);return id;}
function findElement(id){const el=document.getElementById(id)||nativeNodes.get(id);return el instanceof HTMLElement&&el.isConnected?el:null;}
function transport(){
  for(const n of document.querySelectorAll('input[data-ix-overlay-transport]'))if(n instanceof HTMLInputElement&&n.isConnected&&n.getAttribute('data-ix-overlay-transport')===owner)return n;
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
function findTrigger(id,root){
  for(const n of document.querySelectorAll('[data-ix-overlay-trigger]'))if(n instanceof HTMLElement&&n.isConnected&&n.getAttribute('data-ix-overlay-trigger')===id)return n;
  const tid=root?.getAttribute('data-ix-overlay-trigger-id'),el=tid?document.getElementById(tid):null;
  return el instanceof HTMLElement&&el.isConnected?el:null;
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
function popoverAncestors(element){
  const result=[];
  for(let n=element;n;n=stepParent(n))if(n instanceof HTMLElement&&n.getAttribute('data-ix-overlay-native-surface')==='popover'){
    const id=n.getAttribute('data-ix-overlay-host');if(id&&!result.includes(id))result.push(id);
  }
  return result;
}
function popoverTabFacts(panel,active){
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
function itemHostFacts(element){
  const binding=captureItem(element);if(!binding)return null;
  const rawKind=element.getAttribute('data-ix-item-kind');
  const kind=['dropdown','select','menu','category'].includes(rawKind)?rawKind:'native-opt-in';
  return {item:binding.capture,kind,target:binding.target,capture:binding.capture,
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
  for(const root of document.querySelectorAll('[data-ix-overlay-root]')){
    if(!(root instanceof HTMLElement)||!root.isConnected)continue;
    const ro=root.getAttribute('data-ix-overlay-owner'),id=root.getAttribute('data-ix-overlay-root')||'';
    if(!id||(!['window','trap'].includes(phase)&&event?.key!=='Tab'&&ro!==owner))continue;
    const generation=Number(root.getAttribute('data-ix-overlay-generation')||root.getAttribute('data-generation')||'0')||0;
    const discovery_ready=root.getAttribute('data-ix-overlay-ready')!=='false';
    const open=root.hasAttribute('data-ix-overlay-open')?root.getAttribute('data-ix-overlay-open')==='true':root.getAttribute('data-open')==='true';
    const host=findHost(id),itemsHost=findItemsHost(root,id),trigger=findTrigger(id,root),anchor=findAnchor(root),itemsWithEl=hostItems(itemsHost),focusables=hostFocusables(itemsHost);
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
    const item_focus_captures=ro===owner?[...new Set([...itemsWithEl.map(item=>item.el),...focusables])].map(captureItem).filter(Boolean):[];
    const inventory=inventoryFacts(itemsHost);
    for(const item of inventory?.hosts||[])if(item.target)addCand(findElement(item.target));
    nodes.push({
      id,owner:ro||'',generation,root_capture:captureItem(root)?.capture,trigger_capture:captureItem(trigger)?.capture,connected:root.isConnected,discovery_ready,open,mode:root.getAttribute('data-ix-overlay-mode')||'',trigger_id:nativeId(trigger),anchor_id:nativeId(anchor),
      anchor_in_dropdown_item:!!(anchor||trigger)?.closest('[data-ix-dropdown-item]'),
      trigger_ancestors:trigger||anchor?triggerAncestors(trigger||anchor,id):[],
      header_controls:isHeaderPopupHost(host)?headerControls(root):[],
      items:itemsWithEl.map(e=>e.num),focused_item,
      item_focus_captures,
      inventory,
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
    focus_targets:nodes.flatMap(node=>node.tab_candidates.map(candidate=>candidate.target)),
    item_focus_host:kind==='item-focusin'?(()=>{
      const hosts=path.filter(element=>element instanceof HTMLElement&&element.matches(itemSelector));
      const element=hosts.find(element=>element.getAttribute('data-ix-item-focusable')==='true'||element.classList.contains('ix-focusable'))||hosts[0];
      return element?itemHostFacts(element):null;
    })():null
  };
}
function applyRovingWrite(element,value){
  if(!(element instanceof HTMLElement))return;
  if(value===null)element.removeAttribute('tabindex');else element.setAttribute('tabindex',value);
}
function drainRovingWrites(report){
  const pending=transport();if(!pending)return;
  pending.value=JSON.stringify({owner,sequence:report.sequence,roving_pending:report.sequence});
  if(pending.dispatchEvent(new Event('change',{bubbles:true,cancelable:true})))return;
  const tokens=[...new Set(report.nodes.flatMap(node=>node.inventory?.query_order||[]))];
  for(const token of itemCaptureNodes.keys())if(!tokens.includes(token))tokens.push(token);
  let progress;
  do{
    progress=false;
    for(const value of ['-1','0'])for(const item of tokens){
      const input=transport();if(!input)return;
      input.value=JSON.stringify({owner,sequence:report.sequence,roving_write:true,
        roving_event_sequence:report.sequence,roving_item:item,roving_tabindex:value});
      if(!input.dispatchEvent(new Event('change',{bubbles:true,cancelable:true}))){
        applyRovingWrite(itemCaptureNodes.get(item)?.element,value);progress=true;
      }
    }
  }while(progress);
}
function send(report,event){
  const input=transport();
  if(!alive||!input)return;
  // Keep the actual observed objects through Rust callbacks. A replacement
  // with the same public ID must not inherit this event's focus permission.
  const focusExecution=new Map([...report.tab_order,...report.focus_targets,...report.nodes.map(node=>node.trigger_id)].filter(Boolean).map(id=>{
    const element=findElement(id),panel=element?nearestPopover(element):null;
    return [id,{element,panel,generation:panel?.getAttribute('data-surface-generation')}];
  }));
  const activationExecution=new Map(report.candidate_ids.map(id=>[id,findElement(id)]));
  const value=JSON.stringify(report);
  input.value=value;
  const prevent=!input.dispatchEvent(new Event('input',{bubbles:true,cancelable:true}));
  if(report.kind==='key')drainRovingWrites(report);
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
  if(report.kind==='key'){
    for(const [id,facts]of focusExecution){
      const el=facts.element;
      if(report.key==='Tab'&&['capture','bubble','fallback'].includes(report.phase)){
        const scheduleInput=transport();if(!scheduleInput)return;
        tabFocusCaptures.set(report.sequence,{target:id,element:el});
        scheduleInput.value=JSON.stringify({owner,sequence:report.sequence,schedule_focus_target:id});
        if(!scheduleInput.dispatchEvent(new Event('change',{bubbles:true,cancelable:true}))){
          scheduleInput.value=JSON.stringify({owner,sequence:report.sequence,dispatch_focus_target:id});
          if(!scheduleInput.dispatchEvent(new Event('change',{bubbles:true,cancelable:true}))){
            const command={sequence:report.sequence,target:id};
            const frame=requestAnimationFrame(()=>{
              immediateTabFrames.delete(frame);if(alive)executeCapturedTab(command);
            });
            immediateTabFrames.add(frame);
          }
          // A deferred command arrives on the ordered cleanup channel. Direct
          // jobs retain the original event's first native animation frame.
          break;
        }
        tabFocusCaptures.delete(report.sequence);
      }
      const valid=()=>el?.isConnected&&(!facts.panel||(facts.panel.isConnected&&facts.panel.matches(':popover-open')&&facts.panel.getAttribute('data-surface-generation')===facts.generation));
      if(!valid())continue;
      const focusInput=transport();if(!focusInput)return;
      focusInput.value=JSON.stringify({owner,sequence:report.sequence,focus_target:id});
      if(!focusInput.dispatchEvent(new Event('change',{bubbles:true,cancelable:true}))){if(valid())el.focus();break;}
    }
  }
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
  syncItemCues();
}

// Each Boolean or nullable string is an offer. Only Rust's current model may
// grant it; DOM values are never a saved desired state or a navigation policy.
function itemCueQuery(cue){
  const input=transport();if(!alive||!input)return false;
  input.value=JSON.stringify({owner,sequence:0,item_cue:cue});
  return !input.dispatchEvent(new Event('change',{bubbles:true,cancelable:true}));
}
function liveItemCueBinding(host,key,capture,part,partCapture,partKind,allowDetached){
  if(!alive||(!allowDetached&&!host.isConnected)||host.getAttribute('data-ix-item-runtime')!==owner||host.getAttribute('data-ix-item-key')!==key||itemCaptureNodes.get(capture)?.element!==host)return false;
  return !part||((allowDetached||part.isConnected)&&part.closest('[data-ix-item-kind]')===host&&part.getAttribute('data-ix-item-part')===partKind&&itemCaptureNodes.get(partCapture)?.element===part);
}
function syncOneItemCue(host,part=null,allowDetached=false){
  const key=host.getAttribute('data-ix-item-key');if(!key)return;
  const binding=captureItem(host),partBinding=part?captureItem(part):null;
  if(!binding||(part&&!partBinding))return;
  const element=part||host,partKind=part?.getAttribute('data-ix-item-part');
  const valid=()=>liveItemCueBinding(host,key,binding.capture,part,partBinding?.capture,partKind,allowDetached);
  if(!valid())return;
  // Agreement between class and data-active can still be stale. Ask current
  // Rust even when both DOM fields already agree; never infer policy from them.
  const current=element.getAttribute('data-active')==='true';
  for(const active of [current,!current]){
    const cue={key,capture:binding.capture,active};if(partBinding)cue.part=partBinding.capture;
    if(!itemCueQuery(cue))continue;
    if(!valid())return;
    const value=active?'true':'false';
    if(element.getAttribute('data-active')!==value)element.setAttribute('data-active',value);
    if(element.classList.contains('ix-focused')!==active)element.classList.toggle('ix-focused',active);
    return;
  }
}
function rootCueQuery(cue){
  const input=transport();if(!alive||!input)return false;
  input.value=JSON.stringify({owner,sequence:0,item_root_cue:cue});
  return !input.dispatchEvent(new Event('change',{bubbles:true,cancelable:true}));
}
function syncOneRootCue(root,trigger,field,attribute,candidates){
  const id=root.getAttribute('data-ix-overlay-root'),generation=Number(root.getAttribute('data-ix-overlay-generation'));
  const rootBinding=captureItem(root),triggerBinding=trigger?captureItem(trigger):null;
  const element=field==='trigger-aria'?trigger:root;
  if(!id||!Number.isSafeInteger(generation)||!rootBinding||!(element instanceof HTMLElement)||(field==='trigger-aria'&&!triggerBinding))return;
  const valid=()=>alive&&root.isConnected&&root.getAttribute('data-ix-overlay-owner')===owner&&root.getAttribute('data-ix-overlay-root')===id&&Number(root.getAttribute('data-ix-overlay-generation'))===generation&&itemCaptureNodes.get(rootBinding.capture)?.element===root&&(field!=='trigger-aria'||(trigger.isConnected&&findTrigger(id,root)===trigger&&itemCaptureNodes.get(triggerBinding.capture)?.element===trigger));
  const offered=Array.from(new Set([element.getAttribute(attribute),null,...candidates]));
  for(const value of offered){
    if(!valid())return;
    const cue={root:id,generation,capture:rootBinding.capture,field,value};
    if(field==='trigger-aria')cue.trigger_capture=triggerBinding.capture;
    if(!rootCueQuery(cue))continue;
    if(!valid())return;
    if(value===null){if(element.hasAttribute(attribute))element.removeAttribute(attribute);}
    else if(element.getAttribute(attribute)!==value)element.setAttribute(attribute,value);
    return;
  }
}
function queueItemCueReconcile(){
  if(itemCueReconcileQueued||!alive)return;
  itemCueReconcileQueued=true;
  queueMicrotask(()=>{itemCueReconcileQueued=false;if(alive)syncItemCues();});
}
function itemCuesPending(){
  const input=transport();if(!alive||!input)return false;
  input.value=JSON.stringify({owner,sequence:0,item_cues_pending:true});
  return !input.dispatchEvent(new Event('change',{bubbles:true,cancelable:true}));
}
function syncItemCues(captured=null,retained=false,force=false){
  if(!alive)return;
  if(captured instanceof HTMLElement)itemCueCapturedPending.add(captured);
  itemCueRetainedRequested||=retained;
  itemCueForcedRequested||=force;
  if(reconcilingItemCues){queueItemCueReconcile();return;}
  reconcilingItemCues=true;
  try{
    // Consume only a work notification, before projection. A producer during
    // this pass sets new Rust work and keeps its independent wake command.
    const pending=itemCuesPending(),forced=itemCueForcedRequested;
    itemCueForcedRequested=false;
    const capturedHosts=new Set(itemCueCapturedPending);itemCueCapturedPending.clear();
    const includeRetained=itemCueRetainedRequested;itemCueRetainedRequested=false;
    if(!pending&&!forced)return;
    // Any entry can consume the dirty flag before its queued wake arrives.
    // Therefore that entry must also offer retained detached original hosts.
    if(includeRetained||pending)for(const binding of itemCaptureNodes.values())if(binding.element instanceof HTMLElement&&binding.element.hasAttribute('data-ix-item-kind'))capturedHosts.add(binding.element);
    const roots=Array.from(document.querySelectorAll('[data-ix-overlay-root]')).filter(root=>root instanceof HTMLElement&&root.isConnected&&root.getAttribute('data-ix-overlay-owner')===owner&&root.hasAttribute('data-ix-overlay-mode'));
    const candidates=new Set();
    // Offers cover every captured inventory in this owner, including a linked
    // child whose physical host is elsewhere. Rust selects each root's value.
    for(const root of roots){
      const id=root.getAttribute('data-ix-overlay-root'),facts=inventoryFacts(findItemsHost(root,id));
      for(const host of facts?.hosts||[])if(host.target)candidates.add(host.target);
    }
    const seen=new Set();
    const projectHost=(host,allowDetached)=>{
      if(!(host instanceof HTMLElement)||(!allowDetached&&!host.isConnected)||host.getAttribute('data-ix-item-runtime')!==owner||seen.has(host))return;
      seen.add(host);
      if(['dropdown','select','menu'].includes(host.getAttribute('data-ix-item-kind')))syncOneItemCue(host,null,allowDetached);
      for(const part of itemParts(host))syncOneItemCue(host,part,allowDetached);
    };
    // An admitted attempt retains its original object even after DOM removal.
    // No ID lookup may substitute a replacement for that detached object.
    for(const host of capturedHosts)projectHost(host,true);
    for(const host of document.querySelectorAll('[data-ix-item-kind][data-ix-item-runtime]'))projectHost(host,false);
    for(const root of roots){
      const trigger=findTrigger(root.getAttribute('data-ix-overlay-root'),root);
      syncOneRootCue(root,trigger,'active','data-ix-overlay-active-item',candidates);
      syncOneRootCue(root,trigger,'aria','aria-activedescendant',candidates);
      if(trigger)syncOneRootCue(root,trigger,'trigger-aria','aria-activedescendant',candidates);
    }
  }finally{reconcilingItemCues=false;}
}

function syncExecution(){
  let needsFocusPass=false;
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
    const ver=root.getAttribute('data-ix-overlay-focus-version')||'0',gen=root.getAttribute('data-ix-overlay-generation')||'0';
    if(lastFocus.get(id)!==gen+':'+ver)needsFocusPass=true;
  }
  if(needsFocusPass&&!focusFrame){
    focusFrame=requestAnimationFrame(()=>{
      focusFrame=0;if(!alive)return;
      for(const root of document.querySelectorAll('[data-ix-overlay-root]')){
        if(!(root instanceof HTMLElement)||!root.isConnected)continue;
        const ro=root.getAttribute('data-ix-overlay-owner'),id=root.getAttribute('data-ix-overlay-root')||'';
        if(!id||ro!==owner)continue;
        if(root.hasAttribute('data-ix-overlay-native-surface'))continue;
        const ver=root.getAttribute('data-ix-overlay-focus-version')||'0',gen=root.getAttribute('data-ix-overlay-generation')||'0';
        const token=gen+':'+ver,spec=root.getAttribute('data-ix-overlay-focus-target')||'none';
        if(lastFocus.get(id)===token)continue;
        lastFocus.set(id,token);if(spec==='none')continue;
        const list=hostFocusables(findHost(id));
        const dest=spec==='trigger'?findTrigger(id,root):spec==='first-native'?list[0]:spec==='last-native'?list[list.length-1]:
          spec.startsWith('index:')?list[Number(spec.slice(6))]:document.getElementById(spec.startsWith('id:')?spec.slice(3):spec);
        if(dest instanceof HTMLElement&&dest.isConnected)try{dest.focus();}catch{}
      }
      reportObserve();
    });
  }
}
function syncKeyTargets(){
  const wanted=new Set(),wantedClicks=new Set(),currentIds=new Set();
  for(const root of document.querySelectorAll('[data-ix-overlay-root]')){
    if(!(root instanceof HTMLElement)||!root.isConnected||root.getAttribute('data-ix-overlay-owner')!==owner)continue;
    currentIds.add(root.getAttribute('data-ix-overlay-root'));
    const target=findTrigger(root.getAttribute('data-ix-overlay-root'),root);
    if(!target)continue;
    wantedClicks.add(target);
    if(!clickTargets.has(target)){
      const listener=event=>send(collectFacts('click',event,'trigger'),null);
      clickTargets.set(target,listener);target.addEventListener('click',listener,{signal});
    }
    if(root.getAttribute('data-ix-overlay-mode')!=='active-descendant')continue;
    wanted.add(target);
    if(!keyTargets.has(target)){
      const listener=event=>{if(handledBubbleKeys.has(event))return;handledBubbleKeys.add(event);send(collectFacts('key',event,'bubble'),event);};
      keyTargets.set(target,listener);target.addEventListener('keydown',listener,{signal});
    }
  }
  for(const id of lastFocus.keys())if(!currentIds.has(id))lastFocus.delete(id);
  for(const [id,lease]of itemInventoryLeases){
    const current=Array.from(document.querySelectorAll('[data-ix-overlay-root]')).find(root=>root.getAttribute('data-ix-overlay-owner')===owner&&root.getAttribute('data-ix-overlay-root')===id);
    if(!current||Number(current.getAttribute('data-ix-overlay-generation'))!==lease.generation)itemInventoryLeases.delete(id);
  }
  for(const [target,listener]of keyTargets)if(!wanted.has(target)){target.removeEventListener('keydown',listener);keyTargets.delete(target);}
  for(const [target,listener]of clickTargets)if(!wantedClicks.has(target)){target.removeEventListener('click',listener);clickTargets.delete(target);}
}
function syncAnchorObservers(){
  const wanted=new Set();
  for(const root of document.querySelectorAll('[data-ix-overlay-root]')){
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
  for(const [id,node] of nativeNodes)if(!node.isConnected)nativeNodes.delete(id);
  syncKeyTargets();
  if(!transport())return;
  for(const input of document.querySelectorAll('input[data-ix-overlay-transport]'))if(input.getAttribute('data-ix-overlay-transport')===owner)input.setAttribute('data-ix-overlay-native','true');
  syncExecution();
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
document.addEventListener('focusin',event=>{
  const item=event.composedPath().find(n=>n instanceof HTMLElement&&n.hasAttribute('data-ix-dropdown-item'));
  if(item){
    const id=item.getAttribute('data-ix-dropdown-owner');
    const root=Array.from(document.querySelectorAll('[data-ix-overlay-root]')).find(n=>n.getAttribute('data-ix-overlay-root')===id&&n.getAttribute('data-ix-overlay-owner')===owner);
    if(!item.hasAttribute('data-ix-item-kind')&&root?.getAttribute('data-ix-overlay-focus-proxy')==='true'){
      const trigger=findTrigger(id,root);
      if(trigger&&trigger!==deepActive()&&visibleFocusable(trigger))trigger.focus();
    }
  }
  send(collectFacts('item-focusin',event,'focus'),null);
  reportObserve();
},{signal});
document.addEventListener('focusout',()=>queueMicrotask(()=>{if(alive)reportObserve();}),{passive:true,signal});
function itemCueMutation(change){
  if(change.type!=='attributes'||!(change.target instanceof HTMLElement))return false;
  const element=change.target,name=change.attributeName;
  const host=element.matches('[data-ix-item-kind]')?element:element.closest('[data-ix-item-kind]');
  if(host?.getAttribute('data-ix-item-runtime')===owner&&
    (element===host||element.hasAttribute('data-ix-item-part'))&&
    ['class','data-active','data-ix-item-kind','data-ix-item-key','data-ix-item-runtime','data-ix-item-part','data-ix-item-focusable'].includes(name))return true;
  if(element.hasAttribute('data-ix-overlay-mode')&&element.getAttribute('data-ix-overlay-owner')===owner&&
    ['aria-activedescendant','data-ix-overlay-active-item','data-ix-overlay-mode'].includes(name))return true;
  if(name==='aria-activedescendant')for(const root of document.querySelectorAll('[data-ix-overlay-root][data-ix-overlay-mode]')){
    if(root.getAttribute('data-ix-overlay-owner')===owner&&findTrigger(root.getAttribute('data-ix-overlay-root'),root)===element)return true;
  }
  return false;
}
const observer=new MutationObserver(changes=>{
  reportObserve();
  // Child-list/log/geometry work is still observed for raw discovery, but only
  // writes to actual cue fields force projection without a Rust work signal.
  // Otherwise a consumer logging query events could recursively request more
  // queries merely by appending ordinary text to the document.
  if(changes.some(itemCueMutation))syncItemCues(null,false,true);
});
observer.observe(document.documentElement,{subtree:true,childList:true,attributes:true,attributeFilter:[
  'data-ix-overlay-root','data-ix-overlay-owner','data-ix-overlay-host','data-ix-overlay-trigger','data-ix-overlay-trigger-id','data-ix-overlay-anchor-id','data-ix-overlay-generation','data-ix-overlay-open',
  'data-ix-overlay-mode','data-ix-overlay-trigger-visibility-check','data-ix-overlay-native-surface','data-ix-overlay-items-owner','data-ix-overlay-ready','data-ix-overlay-top-layer','data-ix-overlay-focus-version','data-ix-overlay-focus-target','data-small','data-open','hidden','disabled','tabindex','inert','class'
  ,'data-ix-item-kind','data-ix-item-key','data-ix-item-focusable','data-active','data-ix-roving-item','checked'
  ,'data-ix-overlay-items-host-id','data-ix-item-runtime','data-ix-item-part','aria-activedescendant','data-ix-overlay-active-item'
]});
document.documentElement.setAttribute('data-ix-overlay-registry-native',owner);
function queueTriggerRestore(command){
  const root=Array.from(document.querySelectorAll('[data-ix-overlay-root]')).find(node=>
    node instanceof HTMLElement&&node.getAttribute('data-ix-overlay-owner')===owner&&node.getAttribute('data-ix-overlay-root')===command.root&&Number(node.getAttribute('data-ix-overlay-generation'))===command.generation);
  nativeFocusCommands.push({kind:'trigger',command,root});scheduleNativeFocus();
}
function executeCapturedTab(command){
  const capture=tabFocusCaptures.get(command.sequence);tabFocusCaptures.delete(command.sequence);
  if(!capture||capture.target!==command.target)return;
  const input=transport();if(!input)return;
  const el=capture.element;
  input.value=JSON.stringify({owner,sequence:command.sequence,pending_focus_target:command.target,focus_hosts:popoverAncestors(el)});
  const allowed=!input.dispatchEvent(new Event('change',{bubbles:true,cancelable:true}));
  if(allowed&&el?.isConnected)el.focus();
}
function itemFocusQuery(command,capture,phase,element=null){
  const input=transport();if(!input)return false;
  input.value=JSON.stringify({owner,sequence:0,item_focus_sequence:command.sequence,
    item_focus_root:command.root,item_focus_generation:command.generation,
    item_focus_target:command.target,item_focus_capture:capture,item_focus_phase:phase,
    item_focus_host:(phase==='focus'||phase==='visual')&&element instanceof HTMLElement?itemHostFacts(element):undefined});
  return !input.dispatchEvent(new Event('change',{bubbles:true,cancelable:true}));
}
function queueItemFocus(command){
  const discard=()=>{
    const input=transport();if(input){input.value=JSON.stringify({owner,sequence:0,item_focus_phase:'discard',
      item_focus_sequence:command.sequence,item_focus_root:command.root,item_focus_generation:command.generation,
      item_focus_target:command.target,item_focus_capture:command.capture});input.dispatchEvent(new Event('change',{bubbles:true,cancelable:true}));}
  };
  let capture=command.capture,element=capture?itemCaptureNodes.get(capture)?.element:null;
  // Only a pre-bootstrap/async admission without a capture may bind late. An
  // existing object token never falls back to a replacement's public ID.
  if(!capture){
    element=findElement(command.target);
    const binding=captureItem(element);capture=binding?.capture;
  }
  if(!(element instanceof HTMLElement)){
    discard();return;
  }
  if(!capture||!itemFocusQuery(command,capture,'arm')){discard();return;}
  nativeFocusCommands.push({kind:'item',command,capture,element});scheduleNativeFocus();
}
function executeItemFocus(job){
  const {command,capture}=job;
  // Source captures a Category host at admission, then resolves its current
  // shadow parent in frame one. Frame two retains that resolved object.
  const markedKind=job.element?.getAttribute('data-ix-item-kind');
  const control=['menu','category'].includes(markedKind)?Array.from(job.element.querySelectorAll('[data-ix-item-control]')).find(control=>control.closest('[data-ix-item-kind]')===job.element):null;
  const element=control||(job.element?.matches('ix-menu-category')?job.element.shadowRoot?.querySelector('.category-parent'):job.element);
  if(!itemFocusQuery(command,capture,'focus',job.element))return;
  if(element instanceof HTMLElement&&command.focus_native)try{element.focus();}catch{}
  // Reentrant focusin may change Rust's current cue. Source patches publish
  // the attempted host after original focus, even when native focus is a no-op.
  itemFocusQuery(command,capture,'visual',job.element);
  syncItemCues(job.element);
  const frame=requestAnimationFrame(()=>{
    itemScrollFrames.delete(frame);if(!alive)return;
    const allowed=itemFocusQuery(command,capture,'scroll');
    if(allowed&&element instanceof HTMLElement)try{element.scrollIntoView({block:'nearest'});}catch{}
    pruneItemCaptures();
  });
  itemScrollFrames.add(frame);
}
function executeRovingAttributes(command){
  for(const write of command.writes||[]){
    const input=transport();if(!input)return;
    input.value=JSON.stringify({owner,sequence:0,roving_write:true,roving_sequence:command.sequence,
      roving_root:command.root,roving_generation:command.generation,roving_item:write.item,roving_tabindex:write.tabindex});
    if(!input.dispatchEvent(new Event('change',{bubbles:true,cancelable:true})))applyRovingWrite(itemCaptureNodes.get(write.item)?.element,write.tabindex);
  }
  pruneItemCaptures();
}
const inventoryReader=(id,generation)=>{
  const root=Array.from(document.querySelectorAll('[data-ix-overlay-root]')).find(root=>root instanceof HTMLElement&&root.isConnected&&root.getAttribute('data-ix-overlay-owner')===owner&&root.getAttribute('data-ix-overlay-root')===id&&Number(root.getAttribute('data-ix-overlay-generation'))===generation);
  if(!alive||!root)return null;
  const facts=inventoryFacts(findItemsHost(root,id));
  if(facts)itemInventoryLeases.set(id,{generation,tokens:new Set([facts.items_host,...facts.hosts.flatMap(host=>[host.capture,...host.parts.map(part=>part.capture)])])});
  return facts;
};
nativeDocument.inventoryReaders.set(owner,inventoryReader);
function scheduleNativeFocus(){
  if(nativeFocusFrame||!nativeFocusCommands.length)return;
  nativeFocusFrame=requestAnimationFrame(()=>{
    nativeFocusFrame=0;if(!alive)return;
    // Only commands admitted before this frame belong to its batch. A Rust
    // completion callback can enqueue a new transition for a later frame.
    let remaining=nativeFocusCommands.length;
    while(nativeFocusCommands.length&&remaining--){
      const job=nativeFocusCommands[0],command=job.command;
      if(job.kind==='trigger'&&job.root?.isConnected&&job.root.getAttribute('data-ix-overlay-owner')===owner&&job.root.getAttribute('data-ix-overlay-root')===command.root&&Number(job.root.getAttribute('data-ix-overlay-generation'))===command.generation&&
        Number(job.root.getAttribute('data-ix-overlay-focus-version')||'0')<command.focus_version)break;
      // Waiting for one renderer commit must not let a later captured Tab exit
      // overtake an earlier cleanup. Rust's channel supplies causal order.
      nativeFocusCommands.shift();syncExecution();
      const input=transport();if(!input)continue;
      if(job.kind==='completion'){
        input.value=JSON.stringify({owner,sequence:0,completion_sequence:command.sequence,completion_root:command.root,completion_generation:command.generation});
        input.dispatchEvent(new Event('change',{bubbles:true,cancelable:true}));
      }else if(job.kind==='item'){
        executeItemFocus(job);
      }else if(job.kind==='trigger'){
        input.value=JSON.stringify({owner,sequence:0,trigger_restore_sequence:command.sequence,trigger_restore_root:command.root,trigger_restore_generation:command.generation});
        const allowed=!input.dispatchEvent(new Event('change',{bubbles:true,cancelable:true})),root=job.root;
        if(allowed&&root?.isConnected&&root.getAttribute('data-ix-overlay-owner')===owner&&root.getAttribute('data-ix-overlay-root')===command.root&&Number(root.getAttribute('data-ix-overlay-generation'))===command.generation){
          const trigger=findTrigger(command.root,root);
          if(trigger?.isConnected)try{trigger.focus();}catch{}
        }
      }else{
        executeCapturedTab(command);
      }
    }
    reportObserve();scheduleNativeFocus();
  });
}
try{
  reportObserve();
  while(alive){const command=await dioxus.recv();if(command?.finish)break;
    if(command?.restore_trigger)queueTriggerRestore(command.restore_trigger);
    if(command?.tab_focus){nativeFocusCommands.push({kind:'tab',command:command.tab_focus});scheduleNativeFocus();}
    if(command?.completion_frame){nativeFocusCommands.push({kind:'completion',command:command.completion_frame});scheduleNativeFocus();}
    if(command?.item_focus)queueItemFocus(command.item_focus);
    if(command?.item_cues)syncItemCues(null,true);
    if(command?.roving_attributes)executeRovingAttributes(command.roving_attributes);
  }
}finally{
  alive=false;listeners.abort();keyTargets.clear();clickTargets.clear();nativeNodes.clear();observer.disconnect();if(focusFrame)cancelAnimationFrame(focusFrame);
  for(const binding of anchorObservers.values())binding.observer.disconnect();anchorObservers.clear();
  if(nativeFocusFrame)cancelAnimationFrame(nativeFocusFrame);nativeFocusCommands.length=0;tabFocusCaptures.clear();
  for(const frame of itemScrollFrames)cancelAnimationFrame(frame);itemScrollFrames.clear();itemCaptureNodes.clear();
  itemInventoryLeases.clear();itemCueCapturedPending.clear();if(nativeDocument.inventoryReaders.get(owner)===inventoryReader)nativeDocument.inventoryReaders.delete(owner);
  for(const frame of immediateTabFrames)cancelAnimationFrame(frame);immediateTabFrames.clear();
  for(const input of document.querySelectorAll('input[data-ix-overlay-transport]'))if(input.getAttribute('data-ix-overlay-transport')===owner)input.removeAttribute('data-ix-overlay-native');
  if(document.documentElement.getAttribute('data-ix-overlay-registry-native')===owner)document.documentElement.removeAttribute('data-ix-overlay-registry-native');
}
"#;
