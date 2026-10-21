//! Excluded actual-gallery pane layering and ordinary retained-note workflows.
#[path = "../../examples/gallery/application.rs"]
pub(super) mod gallery_application;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn PaneLayersPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    let resize_controls = {
        #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
        {
            rsx! {div {
                for width in [1280,360] {
                    Button { id:format!("pane-layers-review-width-{width}"), variant:ButtonVariant::Secondary,
                        onclick:move |_|dioxus::desktop::window().set_inner_size(dioxus::desktop::LogicalSize::new(f64::from(width),1000.0)),
                        "Native window {width}px"
                    }
                }
            }}
        }
        #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
        {
            VNode::empty()
        }
    };
    rsx! {
        style { {STYLES} }
        style { {gallery_application::STYLES} }
        IxTheme { class:"pane-layers-review",
            {resize_controls}
            gallery_application::ApplicationExamples {}
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=crate::dom_query::eval(NATIVE_PANE_LAYERS_CHECK);let _=query.send("panelayers");
                    match query.join::<serde_json::Value>().await {
                        Ok(value)if value["ok"]==true=>{println!("Dioxus native Pane layers: {value}");window.close();}
                        value=>{eprintln!("Dioxus native Pane layers failed: {value:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(45)).await;eprintln!("Pane layers native replay timed out");std::process::exit(1);});}
            } }
        }
    }
}
const STYLES: &str = "body{margin:0}.pane-layers-review{box-sizing:border-box;min-height:100vh;padding:24px}@media(max-width:600px){.pane-layers-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_PANE_LAYERS_CHECK: &str = r#"
let snapshots={};
try{
 await dioxus.recv();
 const get=id=>document.getElementById('pane-layers-'+id),status=()=>get('status'),frame=()=>get('frame'),layout=()=>frame()?.querySelector(':scope>.ix-pane-layout'),pane=slot=>layout()?.querySelector(':scope>.ix-pane-slot-'+slot+'>.ix-pane'),header=slot=>pane(slot)?.querySelector('.ix-pane-header>button'),all=value=>['left','top','right','bottom'].every(slot=>status()?.dataset[slot]===String(value));
 const assert=(value,label)=>{if(!value)throw Error(label)},wait=async(label,predicate)=>{for(let i=0;i<220;i++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+'; state='+status()?.outerHTML)};
 const prepare=async(order,variant='floating')=>{get('collapse-all').click();await wait('actual four panes collapsed',()=>all(false));if(status().dataset.layout!==order)get('layout').click();if(status().dataset.variant!==variant)get('variant').click();await wait('layout/variant accepted',()=>status().dataset.layout===order&&status().dataset.variant===variant);get('expand-all').click();await wait('actual four panes expanded',()=>all(true));frame().scrollIntoView({block:'center'})};
 const rect=n=>{const r=n.getBoundingClientRect();return{left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height}};
 const hits=async expected=>{await wait('real corner overlap and source-defined hit winner',()=>{const bounds=rect(layout());return[['left','top'],['right','top'],['left','bottom'],['right','bottom']].every(([vertical,horizontal])=>{const a=rect(pane(vertical)),b=rect(pane(horizontal)),left=Math.max(a.left,b.left,bounds.left,0),right=Math.min(a.right,b.right,bounds.right,innerWidth),top=Math.max(a.top,b.top,bounds.top,0),bottom=Math.min(a.bottom,b.bottom,bounds.bottom,innerHeight);if(right-left<2||bottom-top<2)return false;const winner=document.elementFromPoint((left+right)/2,(top+bottom)/2)?.closest('.ix-pane');return expected==='vertical'?[pane('left'),pane('right')].includes(winner):[pane('top'),pane('bottom')].includes(winner)})});const bounds=rect(layout());return[['left','top'],['right','top'],['left','bottom'],['right','bottom']].map(([vertical,horizontal])=>{const a=rect(pane(vertical)),b=rect(pane(horizontal)),x=(Math.max(a.left,b.left,bounds.left,0)+Math.min(a.right,b.right,bounds.right,innerWidth))/2,y=(Math.max(a.top,b.top,bounds.top,0)+Math.min(a.bottom,b.bottom,bounds.bottom,innerHeight))/2,winner=document.elementFromPoint(x,y)?.closest('.ix-pane');return{pair:[vertical,horizontal],point:{x,y},winner:['left','top','right','bottom'].find(slot=>pane(slot)===winner),verticalRect:a,horizontalRect:b}})};
 await wait('actual gallery controls ready',()=>get('expand-all')&&layout());get('review-width-1280').click();await wait('actual wide native viewport',()=>innerWidth>=1100&&pane('left').dataset.mobile==='false');await prepare('full-vertical');snapshots.vertical={viewport:innerWidth,state:{...status().dataset},hits:await hits('vertical')};
 await prepare('full-horizontal');snapshots.horizontal={viewport:innerWidth,state:{...status().dataset},hits:await hits('horizontal')};
 get('collapse-all').click();await wait('collapsed for actual Left note editing',()=>all(false));header('left').click();await wait('only Left opens for retained note workflow',()=>status().dataset.left==='true'&&status().dataset.top==='false'&&status().dataset.bottom==='false');const note=get('note');note.focus();Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(note,'Inspection remains local');note.dispatchEvent(new Event('input',{bubbles:true}));await wait('actual controlled note accepts input',()=>note.value==='Inspection remains local');get('protect').click();await wait('Keep open enabled',()=>get('protect').getAttribute('aria-checked')==='true');header('left').click();await wait('actual close veto keeps Left expanded',()=>status().dataset.left==='true'&&document.querySelector('#pane-layout .application-pane-events').textContent.includes('Closing prevented'));get('protect').click();await wait('protection released',()=>get('protect').getAttribute('aria-checked')==='false');get('collapse-all').click();await wait('normal Collapse all closes Left',()=>all(false));await prepare('full-vertical','inline');assert(note.value==='Inspection remains local','inline/layout changes preserve actual draft');await prepare('full-horizontal','floating');assert(note.value==='Inspection remains local','Collapse/Expand/variant changes retain actual draft');snapshots.note={value:note.value,state:{...status().dataset}};
 get('review-width-360').click();await wait('actual native narrow viewport and mobile pane state',()=>innerWidth<=400&&pane('left').dataset.mobile==='true');get('collapse-all').click();await wait('mobile sibling panes collapsed',()=>all(false));header('left').click();await wait('mobile Left alone opens',()=>status().dataset.left==='true');frame().scrollIntoView({block:'center'});get('note').focus();Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(get('note'),'Inspection remains local / mobile');get('note').dispatchEvent(new Event('input',{bubbles:true}));await wait('mobile note input accepted',()=>get('note').value==='Inspection remains local / mobile');const closeButton=header('left'),closeRect=rect(closeButton),closeTarget=document.elementFromPoint((closeRect.left+closeRect.right)/2,(closeRect.top+closeRect.bottom)/2);snapshots.mobileClose={button:closeRect,reachable:closeButton===closeTarget||closeButton.contains(closeTarget),winnerPane:closeTarget?.closest('.ix-pane')?.querySelector('.ix-pane-heading')?.textContent};assert(snapshots.mobileClose.reachable,'mobile expanded Left close control must remain hit-testable above collapsed sibling headers');closeButton.click();await wait('ordinary mobile Left close accepted',()=>status().dataset.left==='false');await prepare('full-vertical');snapshots.mobileVertical={viewport:innerWidth,state:{...status().dataset},hits:await hits('horizontal')};await prepare('full-horizontal');snapshots.mobileHorizontal={viewport:innerWidth,state:{...status().dataset},hits:await hits('horizontal')};assert(get('note').value==='Inspection remains local / mobile','native responsive workflows retain typed mobile note');
 return{ok:true,snapshots,checks:['actual desktop FullVertical rectangle intersections hit vertical panes at all four corners','actual desktop FullHorizontal intersections hit horizontal panes at all four corners','actual note input, Keep open veto, normal Collapse/Expand and inline/floating/layout changes retain draft','real native window360 permits Left note typing/own close above collapsed sibling headers; both layouts keep expanded horizontal hit priority and note'],input:'Excluded synthetic DOM clicks/input invoke actual unchanged gallery controls, with real desktop window resizing through an excluded Rust button. elementFromPoint and native rectangles establish visible hit ownership; no CSS/application/controller/callback injection. Actual mobile WebKit viewport is observed rather than a CSS-constrained fake viewport. Trusted automated Chromium input is separate evidence; no recent-activation or multi-Pane focus policy is claimed.',renderer:navigator.userAgent};
}catch(error){return{ok:false,error:String(error),snapshots,viewport:innerWidth}}
"#;
