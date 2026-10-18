//! Native DOM observations and execution for the private Rust Header controller.
//! `__OWNER__` is replaced by a serialized root ID. Rust owns all policy and
//! attributes; input processes one report, change only queries its saved stop ACK.

pub(super) const BRIDGE: &str = r#"
const owner=__OWNER__,root=document.getElementById(owner);
const transport=root?.querySelector('input[data-header-layout-observer]');
if(!root||!transport)return;
const listeners=new AbortController();
const signal=listeners.signal;
let alive=true,sequence=0,lastReport='',frame=0,lastFocus='';
let previousOpen=false,previousHost=null;
// Retain at most one actual focused object through a presentation handoff.
// Tokens never resolve through public IDs or replacement DOM nodes.
let focusEpoch=0,captureSequence=0,capturedFocus=null;
const focusableSelector='a[href],area[href],button,input:not([type="hidden"]),select,textarea,iframe,object,embed,[tabindex],[contenteditable]';
const groups=['secondary','default','overflow','avatar'].map(name=>root.querySelector('[data-header-group="'+name+'"]'));
const hosts=['small','large'].map(name=>root.querySelector('[data-header-popup-host="'+name+'"]'));
const more=()=>root.querySelector('[data-header-more-button]');
const boundMore=new WeakSet();
const generation=()=>root.getAttribute('data-generation')||'0';
const activeHost=()=>hosts[root.getAttribute('data-small')==='true'?0:1];
const open=()=>root.getAttribute('data-open')==='true';
const topLayer=()=>root.getAttribute('data-top-layer')==='true';
function deepActive(){let node=document.activeElement;while(node?.shadowRoot?.activeElement)node=node.shadowRoot.activeElement;return node;}
function contains(container,node){
  while(node){if(node===container)return true;node=node.parentNode||(node instanceof ShadowRoot?node.host:null);}
  return false;
}
function presented(node){
  if(!(node instanceof HTMLElement)||node.matches(':disabled,[disabled],[hidden],[aria-disabled="true"]'))return false;
  if(node.closest('[hidden],[inert],[aria-hidden="true"],[contenteditable="false"]')||!node.getClientRects().length)return false;
  const style=getComputedStyle(node);
  return style.display!=='none'&&style.visibility!=='hidden'&&style.visibility!=='collapse';
}
function visible(node){return presented(node)&&node.tabIndex>=0;}
function focusGroup(node){
  if(node===more())return 4;
  const index=groups.findIndex(group=>group&&contains(group,node));
  return index<0?null:index;
}
function captureControl(node){
  const group=focusGroup(node);
  if(!(node instanceof HTMLElement)||!node.isConnected||group===null)return null;
  const token=capturedFocus?.element===node?capturedFocus.token:owner+'-focus-'+(++captureSequence);
  capturedFocus={token,element:node,group,epoch:focusEpoch};
  return{token,group,epoch:focusEpoch};
}
function focusables(host){return host?Array.from(host.querySelectorAll(focusableSelector)).filter(visible):[];}
function facts(kind,event){
  const host=activeHost(),items=focusables(host),active=deepActive(),index=items.indexOf(active);
  const path=event?.composedPath()||[];
  const target=path[0]||event?.target;
  return {
    hosts:path.filter(n=>n instanceof Element&&n.hasAttribute("data-ix-overlay-host")).map(n=>n.getAttribute("data-ix-overlay-host")),
    triggers:path.filter(n=>n instanceof Element&&n.hasAttribute("data-ix-overlay-trigger")).map(n=>n.getAttribute("data-ix-overlay-trigger")),
    owner,generation:Number(generation()),sequence:++sequence,connected:root.isConnected,kind,
    key:event?.key||'',shift:!!event?.shiftKey,alt:!!event?.altKey,default_prevented:!!event?.defaultPrevented,
    presence:groups.map(group=>!!group&&group.children.length>0),focus_count:items.length,
    focused_control:captureControl(active),focus_epoch:focusEpoch,
    active_default:active===document.body||active===document.documentElement,
    focused_index:index<0?null:index,more:path.includes(more()),
    inside:!!host&&(path.includes(host)||contains(host,target)),background:!!host&&target===host
  };
}
function send(report,event){
  if(!alive||!root.isConnected)return;
  const value=JSON.stringify(report);
  transport.value=value;
  const prevent=!transport.dispatchEvent(new Event('input',{bubbles:true,cancelable:true}));
  // This second transport asks only for the saved stop flag; it performs no
  // action or sequence processing, and carries the identical observation.
  transport.value=value;
  const stop=!transport.dispatchEvent(new Event('change',{bubbles:true,cancelable:true}));
  if(event){if(prevent&&event.cancelable)event.preventDefault();if(stop)event.stopPropagation();}
}
function bindMoreClick(){
  const trigger=more();if(!trigger||boundMore.has(trigger))return;
  boundMore.add(trigger);
  // Source Dropdown binds its trigger before document-level overlay handling.
  trigger.addEventListener('click',event=>send(facts('click',event),event),{signal});
}
function reportPresence(){
  const report=facts('presence');
  const signature=JSON.stringify({generation:report.generation,connected:report.connected,
    small:root.getAttribute('data-small'),open:root.getAttribute('data-open'),top:root.getAttribute('data-top-layer'),
    presence:report.presence,focus_count:report.focus_count,focused_index:report.focused_index,
    focused_control:report.focused_control,focus_epoch:report.focus_epoch});
  if(signature!==lastReport){lastReport=signature;send(report);}
}
function popoverOpen(host){try{return !!host?.matches(':popover-open');}catch{return false;}}
function blurInside(host){const active=deepActive();if(contains(host,active)&&typeof active?.blur==='function')active.blur();}
function position(host){
  const trigger=more();if(!trigger)return;
  const rect=trigger.getBoundingClientRect();
  const viewportWidth=document.documentElement.clientWidth;
  host.style.maxWidth=Math.max(0,viewportWidth-16)+'px';
  const width=host.getBoundingClientRect().width;
  const left=Math.max(8,Math.min(rect.left,viewportWidth-width-8));
  host.style.position='fixed';host.style.margin='0';host.style.left=left+'px';host.style.top=rect.bottom+'px';
}
function syncPopups(){
  const active=activeHost(),shown=open(),native=topLayer();
  const target=root.getAttribute('data-focus-target')||'none';
  const closing=previousOpen&&!shown;
  if(closing&&target==='none')blurInside(previousHost);
  for(const host of hosts){
    if(!host)continue;
    const show=host===active&&shown&&native&&host.hasAttribute('popover');
    if(host===active&&shown)position(host);
    if(show){
      if(!popoverOpen(host)&&typeof host.showPopover==='function'){
        const inert=host.inert;host.inert=true;
        try{host.showPopover();}finally{host.inert=inert;}
      }
      position(host);
    }else if(popoverOpen(host)){
      if(target==='none'&&!shown)blurInside(host);
      host.hidePopover();
    }
    if(host!==active){
      for(const property of ['position','margin','left','top','max-width'])host.style.removeProperty(property);
    }
  }
  previousOpen=shown;previousHost=active;
}
function requestFocus(){
  const version=root.getAttribute('data-focus-version')||'0';
  const target=root.getAttribute('data-focus-target')||'none';
  const currentGeneration=generation(),token=currentGeneration+':'+version;
  if(token===lastFocus)return;
  if(frame)cancelAnimationFrame(frame);
  frame=requestAnimationFrame(()=>{
    frame=0;
    if(!alive||!root.isConnected||generation()!==currentGeneration||root.getAttribute('data-focus-version')!==version)return;
    lastFocus=token;
    let node;
    if(target==='preserve'){
      const captureToken=root.getAttribute('data-focus-capture')||'';
      const captured=capturedFocus?.token===captureToken?capturedFocus.element:null;
      const epoch=focusEpoch,report=facts('preserve-focus');
      report.focus_check={version:Number(version),token:captureToken,
        target_connected:!!captured?.isConnected,target_owned:!!captured&&contains(root,captured),
        target_visible:presented(captured),target_group:captured?focusGroup(captured):null};
      transport.value=JSON.stringify(report);
      const authorized=!transport.dispatchEvent(new Event('input',{bubbles:true,cancelable:true}));
      if(authorized&&alive&&root.isConnected&&generation()===currentGeneration&&root.getAttribute('data-focus-version')===version&&focusEpoch===epoch&&captured?.isConnected&&capturedFocus?.token===captureToken&&deepActive()!==captured)captured.focus();
    }else if(target==='more')node=more();
    else if(target!=='none'&&open()){
      const items=focusables(activeHost());
      if(target==='first')node=items[0];
      else if(target==='last')node=items[items.length-1];
      else if(target.startsWith('index:')){
        const index=Number(target.slice(6));
        if(Number.isInteger(index)&&index>=0)node=items[index];
      }
    }
    if(node&&visible(node))node.focus();
    reportPresence();
  });
}
function sync(){if(!alive||!root.isConnected)return;bindMoreClick();syncPopups();reportPresence();requestFocus();}
root.addEventListener('keydown',event=>send(facts('key',event),event),{capture:true,signal});
document.addEventListener('click',event=>{
  if(open()&&!event.composedPath().includes(more()))send(facts('click',event),event);
},{signal});
document.addEventListener('pointerdown',event=>{
  if(open()||event.composedPath().includes(root))send(facts('pointer',event),event);
},{capture:true,signal});
document.addEventListener('focusin',()=>{focusEpoch++;reportPresence();},{capture:true,passive:true,signal});
root.addEventListener('focusout',()=>queueMicrotask(()=>{if(alive)reportPresence();}),{passive:true,signal});
window.addEventListener('resize',sync,{passive:true,signal});
window.addEventListener('scroll',sync,{capture:true,passive:true,signal});
// Exclude style, readiness and focus execution attributes written by this
// bridge. Observe only Rust state, user element eligibility and child topology.
const observer=new MutationObserver(sync);
observer.observe(root,{subtree:true,childList:true,attributes:true,attributeFilter:[
  'data-generation','data-small','data-open','data-top-layer','data-focus-version','data-focus-target','data-focus-capture',
  'hidden','disabled','tabindex','inert','class'
]});
root.dataset.headerLayoutNative='true';
try{sync();await dioxus.recv();}finally{
  alive=false;listeners.abort();observer.disconnect();if(frame)cancelAnimationFrame(frame);
  capturedFocus=null;
  for(const host of hosts)if(popoverOpen(host)){
    blurInside(host);try{host.hidePopover();}catch{}
  }
  delete root.dataset.headerLayoutNative;
}
"#;
