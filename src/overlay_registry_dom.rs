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
function windowSequence(event){if(!event)return 0;let id=nativeDocument.eventIds.get(event);if(!id){id=++nativeDocument.eventSequence;nativeDocument.eventIds.set(event,id);}return id;}
const focusableSelector='a[href],area[href],audio,button,input:not([type="hidden"]),select,textarea,iframe,object,embed,video,[tabindex],[contenteditable],ix-dropdown-item,ix-select-item';
let alive=true,sequence=0,lastSignature='',focusFrame=0;
const keyTargets=new Map(),clickTargets=new Map(),handledBubbleKeys=new WeakSet();
const tabFocusCaptures=new Map(),nativeFocusCommands=[],immediateTabFrames=new Set();let nativeFocusFrame=0;
const anchorObservers=new Map();
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
  const active=deepActive(),path=event?.composedPath?.()||[];
  const target=path[0] instanceof HTMLElement?path[0]:(event?.target instanceof HTMLElement?event.target:null);
  const hosts=pathIds(path,'data-ix-overlay-host',isActiveHost),triggers=pathIds(path,'data-ix-overlay-trigger');
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
    const host=findHost(id),itemsHost=findHost(root.getAttribute('data-ix-overlay-items-owner')||id),trigger=findTrigger(id,root),anchor=findAnchor(root),itemsWithEl=hostItems(itemsHost),focusables=hostFocusables(itemsHost);
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
    nodes.push({
      id,owner:ro||'',generation,connected:root.isConnected,discovery_ready,open,mode:root.getAttribute('data-ix-overlay-mode')||'',trigger_id:nativeId(trigger),anchor_id:nativeId(anchor),
      anchor_in_dropdown_item:!!(anchor||trigger)?.closest('[data-ix-dropdown-item]'),
      trigger_ancestors:trigger||anchor?triggerAncestors(trigger||anchor,id):[],
      header_controls:isHeaderPopupHost(host)?headerControls(root):[],
      items:itemsWithEl.map(e=>e.num),focused_item,
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
    hosts,triggers,anchors,nodes,active_categories,document_tab_candidates,tab_order:document_tab_candidates.map(candidate=>candidate.target),candidate_ids,
    active_hosts:hostAncestors(active),active_popover:nearestPopover(active)?.getAttribute('data-ix-overlay-host')||'',
    focus_targets:nodes.flatMap(node=>node.tab_candidates.map(candidate=>candidate.target))
  };
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
      const el=document.getElementById(id);
      if(!(el instanceof HTMLElement)||!el.isConnected)continue;
      const clickInput=transport();if(!clickInput)return;
      clickInput.value=JSON.stringify({owner,sequence:report.sequence,activate_target:id});
      if(!clickInput.dispatchEvent(new Event('change',{bubbles:true,cancelable:true}))){
        if(el.isConnected)el.click();
        break;
      }
    }
  }
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
    if(root?.getAttribute('data-ix-overlay-focus-proxy')==='true'){
      const trigger=findTrigger(id,root);
      if(trigger&&trigger!==deepActive()&&visibleFocusable(trigger))trigger.focus();
    }
  }
  reportObserve();
},{signal});
document.addEventListener('focusout',()=>queueMicrotask(()=>{if(alive)reportObserve();}),{passive:true,signal});
const observer=new MutationObserver(reportObserve);
observer.observe(document.documentElement,{subtree:true,childList:true,attributes:true,attributeFilter:[
  'data-ix-overlay-root','data-ix-overlay-owner','data-ix-overlay-host','data-ix-overlay-trigger','data-ix-overlay-trigger-id','data-ix-overlay-anchor-id','data-ix-overlay-generation','data-ix-overlay-open',
  'data-ix-overlay-mode','data-ix-overlay-trigger-visibility-check','data-ix-overlay-native-surface','data-ix-overlay-items-owner','data-ix-overlay-ready','data-ix-overlay-top-layer','data-ix-overlay-focus-version','data-ix-overlay-focus-target','data-small','data-open','hidden','disabled','tabindex','inert','class'
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
  }
}finally{
  alive=false;listeners.abort();keyTargets.clear();clickTargets.clear();nativeNodes.clear();observer.disconnect();if(focusFrame)cancelAnimationFrame(focusFrame);
  for(const binding of anchorObservers.values())binding.observer.disconnect();anchorObservers.clear();
  if(nativeFocusFrame)cancelAnimationFrame(nativeFocusFrame);nativeFocusCommands.length=0;tabFocusCaptures.clear();
  for(const frame of immediateTabFrames)cancelAnimationFrame(frame);immediateTabFrames.clear();
  for(const input of document.querySelectorAll('input[data-ix-overlay-transport]'))if(input.getAttribute('data-ix-overlay-transport')===owner)input.removeAttribute('data-ix-overlay-native');
  if(document.documentElement.getAttribute('data-ix-overlay-registry-native')===owner)document.documentElement.removeAttribute('data-ix-overlay-registry-native');
}
"#;
