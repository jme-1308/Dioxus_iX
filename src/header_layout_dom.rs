//! Primary viewport/global invalidation adapter for Header mounted geometry.
//! Pinned MountedData has no viewport-width query and Document has no typed
//! global scroll/resize subscriptions. Shared overlay input owns pointer facts.
//! Dioxus owns slot presence, lifecycle, control groups and measurements; the shared
//! overlay observation supplies eligible focus facts. No local DOM enumeration.

pub(super) const BRIDGE: &str = r#"
const owner=__OWNER__,root=document.getElementById(owner);
const transport=document.getElementById(owner+'-transport');
if(!root||!transport)return;
const listeners=new AbortController(),signal=listeners.signal;
let alive=true,sequence=0;
function report(kind){
  if(!alive||!root.isConnected)return;
  transport.value=JSON.stringify({owner,generation:Number(root.getAttribute('data-generation')||'0'),
    sequence:++sequence,kind,viewport_width:document.documentElement.clientWidth});
  transport.dispatchEvent(new Event('input',{bubbles:true,cancelable:false}));
}
window.addEventListener('resize',()=>report('layout'),{passive:true,signal});
window.addEventListener('scroll',()=>report('layout'),{capture:true,passive:true,signal});
root.dataset.headerLayoutNative='true';
try{report('layout');await dioxus.recv();}finally{
  alive=false;listeners.abort();delete root.dataset.headerLayoutNative;
}
"#;
