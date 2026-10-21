//! Excluded actual-gallery simultaneous Pane opening focus and local workflows.
use super::pane_layers_review::gallery_application;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn PaneOpeningPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    let resize_controls = {
        #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
        {
            rsx! {div {
                for width in [1280,360] {
                    Button { id:format!("pane-opening-review-width-{width}"), variant:ButtonVariant::Secondary,
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
        IxTheme { class:"pane-opening-review",
            {resize_controls}
            gallery_application::ApplicationExamples {}
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=crate::dom_query::eval(NATIVE_PANE_OPENING_CHECK);let _=query.send("paneopening");
                    match query.join::<serde_json::Value>().await {
                        Ok(value)if value["ok"]==true=>{println!("Dioxus native Pane opening: {value}");window.close();}
                        value=>{eprintln!("Dioxus native Pane opening failed: {value:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(60)).await;eprintln!("Pane opening native replay timed out");std::process::exit(1);});}
            } }
        }
    }
}
const STYLES: &str = "body{margin:0}.pane-opening-review{box-sizing:border-box;min-height:100vh;padding:24px}@media(max-width:600px){.pane-opening-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_PANE_OPENING_CHECK: &str = r#"
let snapshots={},keys=[];
try{
 await dioxus.recv();
 const get=id=>document.getElementById('pane-layers-'+id),status=()=>get('status'),frame=()=>get('frame'),layout=()=>frame()?.querySelector(':scope>.ix-pane-layout'),pane=slot=>layout()?.querySelector(':scope>.ix-pane-slot-'+slot+'>.ix-pane'),header=slot=>pane(slot)?.querySelector('.ix-pane-header>button'),all=value=>['left','right','top','bottom'].every(slot=>status()?.dataset[slot]===String(value));
 const assert=(value,label)=>{if(!value)throw Error(label)},wait=async(label,predicate)=>{for(let i=0;i<250;i++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+'; active='+document.activeElement?.id+'; state='+status()?.outerHTML)};
 const click=node=>{node.focus();node.click()},key=(value,shiftKey=false)=>{const event=new KeyboardEvent('keydown',{key:value,code:value,bubbles:true,cancelable:true,shiftKey});document.activeElement.dispatchEvent(event);return event};
 const slots=['left','right','top','bottom'],actual=(expanded,floating,mobile)=>slots.every(slot=>pane(slot)?.dataset.expanded===String(expanded)&&pane(slot)?.dataset.floating===String(floating)&&pane(slot)?.dataset.mobile===String(mobile)),metadata=()=>({layout:layout()?.dataset.layout,status:{...status()?.dataset},panes:Object.fromEntries(slots.map(slot=>[slot,{...pane(slot)?.dataset}]))});
 const prepare=async order=>{const mobile=innerWidth<=400;click(get('collapse-all'));await wait('owner and all actual panes collapsed at current viewport',()=>all(false)&&slots.every(slot=>pane(slot)?.dataset.expanded==='false'&&pane(slot)?.dataset.mobile===String(mobile)));if(status().dataset.layout!==order)click(get('layout'));if(status().dataset.variant!=='floating')click(get('variant'));await wait('actual layout/variant/mobile accepted before co-opening',()=>status().dataset.layout===order&&status().dataset.variant==='floating'&&layout()?.dataset.layout===order&&actual(false,true,mobile));const before=metadata();click(get('expand-all'));await wait('owner and all actual panes expanded',()=>all(true)&&actual(true,true,mobile));frame().scrollIntoView({block:'center'});snapshots[(mobile?'mobile':'desktop')+'-'+order+'-metadata']={before,after:metadata()}};
 const focusFacts=()=>{const a=document.activeElement,r=a.getBoundingClientRect(),hit=document.elementFromPoint(r.x+r.width/2,r.y+r.height/2);return{id:a.id,pane:a.closest('.ix-pane')?.getAttribute('aria-label'),hitPane:hit?.closest('.ix-pane')?.getAttribute('aria-label'),hitIsFocused:!!hit&&(a===hit||a.contains(hit)),rect:{x:r.x,y:r.y,width:r.width,height:r.height},viewport:{width:innerWidth,height:innerHeight}}};
 const focused=async slot=>{await wait('exposed '+slot+' opening destination',()=>{const a=document.activeElement;if(!pane(slot)?.contains(a))return false;return focusFacts().hitIsFocused});return focusFacts()};
 const localKeys=async slot=>{await wait('native focus observation is current',()=>pane(slot).dataset.paneFocusCurrent===document.activeElement.id);const forward=key('Tab');await focused(slot);const reverse=key('Tab',true);await focused(slot);keys.push({slot,synthetic:true,forwardDefaultPrevented:forward.defaultPrevented,reverseDefaultPrevented:reverse.defaultPrevented});key('Escape');await wait('Escape closes own '+slot,()=>status().dataset[slot]==='false')};
 await wait('actual application controls ready',()=>get('expand-all')&&header('left'));
 click(document.getElementById('pane-opening-review-width-1280'));await wait('actual wide native viewport/all Pane facts',()=>innerWidth>=1100&&slots.every(slot=>pane(slot)?.dataset.mobile==='false'));
 await prepare('full-vertical');snapshots.vertical=await focused('right');await localKeys('right');
 await prepare('full-horizontal');snapshots.horizontal=await focused('bottom');await localKeys('bottom');
 click(get('collapse-all'));await wait('collapsed for single Left',()=>all(false));click(header('left'));await wait('single Left expanded',()=>status().dataset.left==='true');snapshots.singleLeft=await focused('left');
 const note=get('note');note.focus();Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(note,'Inspection stays local');note.dispatchEvent(new Event('input',{bubbles:true}));await wait('actual note input accepted',()=>note.value==='Inspection stays local');click(get('protect'));await wait('protected',()=>get('protect').getAttribute('aria-checked')==='true');note.focus();key('Escape');await wait('protected Escape veto',()=>status().dataset.left==='true'&&document.querySelector('#pane-layout .application-pane-events').textContent.includes('Closing prevented'));
 const changedLayout=status().dataset.layout==='full-horizontal'?'full-vertical':'full-horizontal';click(get('layout'));await wait('layout action retains independent toolbar focus',()=>status().dataset.layout===changedLayout&&document.activeElement===get('layout')&&status().dataset.left==='true');click(get('variant'));await wait('floating-to-inline retains current toolbar focus',()=>status().dataset.variant==='inline'&&document.activeElement===get('variant'));assert(note.value==='Inspection stays local','actual note retained through layout/variant');
 click(get('protect'));await wait('unprotected',()=>get('protect').getAttribute('aria-checked')==='false');click(header('left'));await wait('normal Left close',()=>status().dataset.left==='false');click(header('left'));await wait('normal Left reopen',()=>status().dataset.left==='true');assert(note.value==='Inspection stays local','close/reopen retains note');snapshots.note={value:note.value,state:{...status().dataset}};
 click(document.getElementById('pane-opening-review-width-360'));await wait('real narrow native viewport/all Pane facts',()=>innerWidth<=400&&slots.every(slot=>pane(slot)?.dataset.mobile==='true'));
 for(const order of ['full-vertical','full-horizontal']){await prepare(order);snapshots['mobile-'+order]=await focused('bottom');await localKeys('bottom')}
 assert(note.value==='Inspection stays local','actual resize/Expand all retains note');
 return{ok:true,snapshots,keys,checks:['desktop FullVertical simultaneous opening admits exposed Right registered control; synthetic boundary requests retain own focus and Escape closes','desktop FullHorizontal simultaneous opening admits exposed Bottom registered control; synthetic boundary requests retain own focus and Escape closes','single Left registered fallback, native note input, Keep open veto, outside toolbar focus/layout and floating-to-inline cancellation, ordinary close/reopen retain note','actual desktop window360 opens exposed Bottom in both layouts and preserves own synthetic boundary/dismissal and note'],input:'Excluded synthetic DOM focus/click/input and keydown events invoke actual unchanged ApplicationExamples; single-control synthetic Tab requests preserve current focus and do not establish native default navigation; unchanged paneequipment separately proves registered boundary transfer, and Chromium proves trusted Tab/ShiftTab. Native window is really resized1280/360 using excluded Rust buttons; native rectangles/elementFromPoint establish exposed destination. No application/controller/callback/CSS injection. Raw note has no mounted registration, so registered header fallback is expected; unchanged equipment/layout regressions cover registered field autofocus. Trusted automated Chromium keys/pointer are separate evidence; no physical OS/AT claim.',renderer:navigator.userAgent};
}catch(error){return{ok:false,error:String(error),snapshots,focus:document.activeElement?.id,viewport:{width:innerWidth,height:innerHeight},status:document.getElementById('pane-layers-status')?.outerHTML}}
"#;
