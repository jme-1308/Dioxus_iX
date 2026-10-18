// SPDX-FileCopyrightText: 2026 Siemens AG
// SPDX-License-Identifier: MIT

//! Raw source topology and exact-object execution for opening focus.
//! This fragment shares the overlay bridge's lexical scope. Rust selects the
//! candidate and target; native code never filters for eligibility or retries
//! after a granted native focus attempt.

pub(super) const PLUMBING: &str = r#"
const descendantObjectTokens=new WeakMap();let descendantObjectSequence=0;
function descendantCapture(element){
  if(!(element instanceof Element))return null;
  if(element instanceof HTMLElement)return captureItem(element)?.capture||null;
  let capture=descendantObjectTokens.get(element);
  if(!capture){capture=owner+'-descendant-object-'+(++descendantObjectSequence);descendantObjectTokens.set(element,capture);}
  return capture;
}
function descendantQuery(command,reference,phase,extra={}){
  const input=transport();if(!alive||!input)return false;
  input.value=JSON.stringify({owner,sequence:0,descendant_focus:{sequence:command.sequence,root:command.root,generation:command.generation,reference,phase,...extra}});
  return !input.dispatchEvent(new Event('change',{bubbles:true,cancelable:true}));
}
function descendantRootMatches(root,command,reference){
  return root instanceof HTMLElement&&root.getAttribute('data-ix-overlay-owner')===owner&&root.getAttribute('data-ix-overlay-root')===command.root&&Number(root.getAttribute('data-ix-overlay-generation'))===command.generation&&descendantCapture(root)===reference;
}
function descendantOwnContent(root,id){
  return Array.from(root.querySelectorAll('[data-ix-overlay-host]')).find(element=>element instanceof HTMLElement&&element.getAttribute('data-ix-overlay-host')===id&&element.closest('[data-ix-overlay-root]')===root&&isActiveHost(element))||null;
}
function collectDescendantFocus(root,content,extra=[]){
  const nodes=new Map(),elements=new Map(),typedHosts=new Set(),light=[],slots=[],shadowOrders=new Map(),outerElements=new Set();
  const internal=element=>element.matches('input[data-ix-overlay-transport]');
  const remember=element=>{
    if(!(element instanceof Element)||internal(element))return null;
    const capture=descendantCapture(element);if(!capture)return null;
    if(!nodes.has(capture)){
      const rawKind=element.getAttribute('data-ix-descendant-kind')||element.getAttribute('data-ix-item-kind');
      const sourceKind=['dropdown','select','menu','category','native-opt-in'].includes(rawKind)?rawKind:null;
      nodes.set(capture,{capture,tag:element.localName,is_html:element instanceof HTMLElement,...(sourceKind?{source_kind:sourceKind}:{}),tabindex:element.getAttribute('tabindex'),href:element.hasAttribute('href'),contenteditable:element.hasAttribute('contenteditable'),hidden:element.hasAttribute('hidden'),disabled:element.hasAttribute('disabled'),checked:element.hasAttribute('checked')});
      elements.set(capture,element);
      if(element instanceof HTMLElement&&element.hasAttribute('data-ix-item-kind'))typedHosts.add(element);
    }
    return capture;
  };
  const shadowOrder=host=>{
    remember(host);let order=shadowOrders.get(host);
    if(!order){order=[];shadowOrders.set(host,order);}return order;
  };
  // Only structural generated boundaries are partitioned here. Supplied
  // Category body children remain light descendants regardless of overlays.
  const walk=(element,order,domain=null,collectSlots=false)=>{
    if(!(element instanceof Element)||internal(element))return;
    const typed=element.hasAttribute('data-ix-item-kind');
    if(typed)domain=null;
    const host=element.parentElement?.closest('[data-ix-item-kind]');
    const generated=element.hasAttribute('data-ix-item-part')||element.hasAttribute('data-ix-item-control')||element.hasAttribute('data-ix-descendant-kind');
    const categoryHost=host?.getAttribute('data-ix-item-kind')==='category'?host:null;
    if(generated&&host)domain=host;
    if(categoryHost&&(element.hasAttribute('data-menu-category-panel')||element.hasAttribute('data-menu-category-empty-popup')))domain=categoryHost;
    const capture=remember(element);if(!capture)return;
    (domain?shadowOrder(domain):order).push(capture);
    if(!domain){outerElements.add(element);if(collectSlots&&element instanceof HTMLSlotElement)slots.push(element);}
    const suppliedCategoryBody=categoryHost&&element.hasAttribute('data-menu-category-body');
    for(const child of element.children)walk(child,order,suppliedCategoryBody?null:domain,collectSlots);
  };
  const fallback=remember(root);
  if(content instanceof Element)for(const child of content.children)walk(child,light,null,true);
  for(const element of extra)walk(element,[],null,false);
  const assignments=[];
  for(const slot of slots){
    const supplied=[];
    for(const assigned of slot.assignedElements({flatten:true})){
      const capture=remember(assigned);if(!capture)continue;
      outerElements.add(assigned);
      const descendants=[];
      for(const child of assigned.children)walk(child,descendants,null,false);
      supplied.push({root:capture,descendants});
    }
    assignments.push({slot,assignments:supplied});
  }
  // Own shadow querySelectorAll is intentionally not recursive and does not
  // expand assigned slots. Rust applies its first literal matching descendant.
  for(const element of outerElements){
    if(!element.shadowRoot)continue;
    const descendants=[];
    for(const inner of element.shadowRoot.querySelectorAll('*')){
      const capture=remember(inner);if(capture)descendants.push(capture);
    }
    shadowOrders.set(element,descendants);
  }
  const facts={nodes:Array.from(nodes.values()),light,
    slots:assignments.map(entry=>({assignments:entry.assignments,following:Array.from(elements,([capture,element])=>({capture,element})).filter(binding=>entry.slot.compareDocumentPosition(binding.element)&Node.DOCUMENT_POSITION_FOLLOWING).map(binding=>binding.capture)})),
    shadows:Array.from(shadowOrders,([host,descendants])=>({host:descendantCapture(host),descendants})),fallback};
  return {facts,elements,typedHosts};
}
function queueDescendantFocus(command){
  let reference=command.reference,root=reference?itemCaptureNodes.get(reference)?.element:null;
  if(!reference){
    // Only an unbound bootstrap admission can find its first registered root.
    // A known reference never falls back to another element with that ID.
    root=Array.from(document.querySelectorAll('[data-ix-overlay-root]')).find(element=>element instanceof HTMLElement&&element.isConnected&&element.getAttribute('data-ix-overlay-owner')===owner&&element.getAttribute('data-ix-overlay-root')===command.root&&Number(element.getAttribute('data-ix-overlay-generation'))===command.generation);
    reference=descendantCapture(root);
  }
  const discard=()=>{descendantQuery(command,reference||command.reference||'', 'discard');};
  if(!reference||!descendantRootMatches(root,command,reference)){discard();return;}
  const content=descendantOwnContent(root,command.root);
  const returned=command.purpose?.kind==='return'?itemCaptureNodes.get(command.purpose.capture)?.element:null;
  if(command.purpose?.kind==='return'&&!(returned instanceof Element)){discard();return;}
  if(!descendantQuery(command,reference,'arm')){discard();return;}
  nativeFocusCommands.push({kind:'descendant',command,root,content,reference,returned});scheduleNativeFocus();
}
function executeDescendantFocus(job){
  const {command,root,reference,returned}=job;
  const content=command.purpose?.kind==='submenu'?findItemsHost(root,command.root):job.content;
  const cancel=()=>{descendantQuery(command,reference,'cancel');};
  if(!descendantRootMatches(root,command,reference)){cancel();return;}
  syncReflectedCheckedAttributes();
  let fallback=root;
  if(command.purpose?.kind==='submenu'&&content instanceof HTMLElement){
    const hostId=content.getAttribute('data-ix-overlay-host');
    fallback=Array.from(document.querySelectorAll('[data-ix-overlay-root]')).find(candidate=>candidate instanceof HTMLElement&&candidate.getAttribute('data-ix-overlay-owner')===owner&&candidate.getAttribute('data-ix-overlay-root')===hostId)||root;
  }
  const held=collectDescendantFocus(fallback,content,returned?[returned]:[]);
  const currentHosts=(path=[])=>Array.from(new Set([...held.typedHosts,...path.filter(element=>element instanceof HTMLElement&&element.hasAttribute('data-ix-item-kind'))]),host=>itemHostFacts(host)).filter(Boolean);
  if(!descendantQuery(command,reference,'prepare',{facts:held.facts,inventory:inventoryFacts(content),hosts:currentHosts()})){cancel();return;}
  for(const [target,element]of held.elements){
    if(!descendantQuery(command,reference,'focus',{target}))continue;
    // Source tryFocusElement ignores a selected non-HTMLElement. The selected
    // object is still the only offer granted; no later candidate is retried.
    // Source Menu/Category hosts delegate native focus into their own control.
    // Keep the logical attempted host for the post-focus source patch path.
    const kind=element.getAttribute?.('data-ix-item-kind');
    const delegated=['menu','category'].includes(kind)?Array.from(element.querySelectorAll('[data-ix-item-control]')).find(control=>control.closest('[data-ix-item-kind]')===element):null;
    if(element instanceof HTMLElement)try{(delegated||element).focus();}catch{}
    // SetupMixin reads the attempted object's path AFTER original focus. Its
    // own-shadow step jumps to that root's host; it never follows assignedSlot.
    const path=[];
    if(element instanceof HTMLElement)for(let current=element;current instanceof Element;){
      path.push(current);
      const currentRoot=current.getRootNode();
      current=currentRoot instanceof ShadowRoot?currentRoot.host:current.parentElement;
    }
    const hosts=currentHosts(path),attempt_path=path.map(descendantCapture).filter(Boolean);
    descendantQuery(command,reference,'visual',{target,hosts,attempt_path});
    for(const host of new Set([...held.typedHosts,...path.filter(element=>element instanceof HTMLElement&&element.hasAttribute('data-ix-item-kind'))]))syncItemCues(host);
    return;
  }
  cancel();
}
"#;
