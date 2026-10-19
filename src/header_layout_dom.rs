//! Native DOM observations and execution for the private Rust Header controller.
//! `__OWNER__` is replaced by a serialized root ID. Rust owns all policy and
//! attributes and Dioxus key handlers. Input carries read-only observations only.

pub(super) const BRIDGE: &str = r#"
const owner=__OWNER__,root=document.getElementById(owner);
const transport=root?.querySelector('input[data-header-layout-observer]');
if(!root||!transport)return;
const listeners=new AbortController();
const signal=listeners.signal;
let alive=true,sequence=0,lastReport='';
// Pinned MountedData has no active-control, descendant-focus-order or CSS/inert
// visibility query. Geometry measurements alone cannot report those facts.
// Observe existing IDs only; Rust resolves registered mounted handles.
const focusableSelector='a[href],area[href],button,input:not([type="hidden"]),select,textarea,iframe,object,embed,[tabindex],[contenteditable]';
const groups=['secondary','default','overflow','avatar'].map(name=>root.querySelector('[data-header-group="'+name+'"]'));
const hosts=['small','large'].map(name=>root.querySelector('[data-header-popup-host="'+name+'"]'));
const more=()=>root.querySelector('[data-header-more-button]');
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
function focusGroup(node,trigger){
  if(node===trigger)return 4;
  const index=groups.findIndex(group=>group&&contains(group,node));
  return index<0?null:index;
}
function facts(kind){
  const host=activeHost(),active=deepActive(),trigger=more();
  // MountedData cannot enumerate presented descendants or identify the actual
  // active element. One read-only scan supplies these facts; Rust derives all
  // popup membership/count/index and programmatic-focus policy projections.
  const controls=Array.from(root.querySelectorAll(focusableSelector)).filter(presented).map(node=>({
    id:node.id,tab_index:node.tabIndex,active:node===active,
    group:focusGroup(node,trigger),in_popup:!!host&&contains(host,node)
  }));
  return {
    owner,generation:Number(generation()),sequence:++sequence,kind,
    presence:groups.map(group=>!!group&&group.children.length>0),controls,
    active_default:active===document.body||active===document.documentElement,
    viewport_width:document.documentElement.clientWidth
  };
}
function send(report){
  if(!alive||!root.isConnected)return;
  transport.value=JSON.stringify(report);
  transport.dispatchEvent(new Event('input',{bubbles:true,cancelable:false}));
}
function reportPresence(){
  const report=facts('presence');
  const signature=JSON.stringify({generation:report.generation,
    small:root.getAttribute('data-small'),open:root.getAttribute('data-open'),top:root.getAttribute('data-top-layer'),
    presence:report.presence,controls:report.controls,viewport_width:report.viewport_width,active_default:report.active_default});
  if(signature!==lastReport){lastReport=signature;send(report);}
}
// Dioxus mounted handles own geometry. The pinned MountedData API has no
// viewport query or global scroll/resize listeners, so report only invalidation
// and document width; no element measurements or presentation writes occur here.
function sync(){if(!alive||!root.isConnected)return;reportPresence();}
function reportLayout(){if(alive&&root.isConnected)send(facts('layout'));}
document.addEventListener('pointerdown',event=>{
  if(open()||event.composedPath().includes(root))send(facts('pointer'));
},{capture:true,signal});
document.addEventListener('focusin',reportPresence,{capture:true,passive:true,signal});
root.addEventListener('focusout',()=>queueMicrotask(()=>{if(alive)reportPresence();}),{passive:true,signal});
window.addEventListener('resize',reportLayout,{passive:true,signal});
window.addEventListener('scroll',reportLayout,{capture:true,passive:true,signal});
// Observe Rust state, element eligibility and child topology. Native popup
// geometry and readiness writes do not trigger another observation.
const observer=new MutationObserver(sync);
observer.observe(root,{subtree:true,childList:true,attributes:true,attributeFilter:[
  'data-generation','data-small','data-open','data-top-layer',
  'hidden','disabled','tabindex','inert','class'
]});
root.dataset.headerLayoutNative='true';
try{sync();await dioxus.recv();}finally{
  alive=false;listeners.abort();observer.disconnect();
  delete root.dataset.headerLayoutNative;
}
"#;
