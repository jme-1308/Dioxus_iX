//! Excluded actual-gallery release popover viewport/action replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/dialog_scenarios.rs"]
mod dialog_scenarios;

#[component]
pub fn ReleasePopoverPreview() -> Element {
    debug_assert!(dialog_scenarios::SCENARIOS.contains(&"popover"));
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {dialog_scenarios::STYLES} }
        IxTheme { class: "release-popover-review",
            h1 { "Release popover" }
            p { class: "release-popover-intro", "Actual gallery scenario with its opener in the lower half of the viewport." }
            div { class: "release-popover-spacer", "aria-hidden": "true" }
            div { id: "release-popover-stage", dialog_scenarios::DialogScenario { name: "popover" } }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_RELEASE_POPOVER_CHECK);
                        if let Err(error) = query.send(include_str!("../../examples/gallery/assets/release-preview.svg")) {
                            eprintln!("Release popover native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native release popover: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native release popover failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Release popover native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.release-popover-review{box-sizing:border-box;min-height:100vh;padding:24px}.release-popover-review>h1{font-size:28px;line-height:40px;margin:0 0 16px}.release-popover-intro{font-size:14px;line-height:20px;margin:0;max-width:72ch}.release-popover-spacer{height:48vh}";

// This excluded replay does not scroll/focus an offscreen action to hide a layout
// defect. Bounds and native center hits must pass before any action is focused.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_RELEASE_POPOVER_CHECK: &str = r#"
let snapshots={},layout={},imageFacts={},failureGeometry={},escape={};
try {
 const expectedSvg=await dioxus.recv();
 const get=id=>document.getElementById(id),stage=()=>get('release-popover-stage'),active=()=>document.activeElement;
 const opener=()=>stage()?.querySelector('button[popovertarget]'),panel=()=>get(opener()?.getAttribute('aria-controls'));
 const image=()=>panel()?.querySelector('img[alt="Release preview"]'),action=label=>[...panel()?.querySelectorAll('button')??[]].find(node=>node.textContent.trim()===label);
 const feedback=()=>stage()?.querySelector('.dialog-scenario-feedback')?.textContent.trim()??'';
 const opened=()=>panel()?.matches(':popover-open')===true&&panel()?.dataset.ixOverlayOpen==='true';
 const assert=(condition,label)=>{if(!condition)throw Error(label)},hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const rect=node=>{if(!node)return null;const r=node.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height}};
 const viewport=()=>({width:innerWidth,height:innerHeight,pageWidth:document.documentElement.scrollWidth,scrollY});
 const fit=node=>{if(!node)return false;const r=node.getBoundingClientRect();return r.width>0&&r.height>0&&r.left>=-1&&r.right<=innerWidth+1&&r.top>=-1&&r.bottom<=innerHeight+1};
 const hit=node=>{if(!fit(node))return false;const r=node.getBoundingClientRect(),target=document.elementFromPoint(r.left+r.width/2,r.top+r.height/2);return target===node||node.contains(target)};
 const geometry=()=>{
  const node=panel(),style=node?getComputedStyle(node):null;
  return {viewport:viewport(),opener:rect(opener()),panel:rect(node),body:rect(node?.querySelector('.ix-popover-body')),image:rect(image()),save:rect(action('Save')),cancel:rect(action('Cancel')),
   saveHit:hit(action('Save')),cancelHit:hit(action('Cancel')),focus:active()?.id||active()?.tagName,
   supports:{anchorName:CSS.supports('anchor-name:--release-probe'),positionAnchor:CSS.supports('position-anchor:--release-probe'),positionArea:CSS.supports('position-area:bottom'),positionTry:CSS.supports('position-try-fallbacks:flip-block')},
   computed:style?{position:style.position,positionAnchor:style.getPropertyValue('position-anchor'),positionArea:style.getPropertyValue('position-area'),positionTry:style.getPropertyValue('position-try-fallbacks'),maxHeight:style.maxHeight,transform:style.transform}:null};
 };
 const wait=async(label,predicate)=>{for(let attempt=0;attempt<200;attempt++){if(predicate())return;await hold(20)}failureGeometry=geometry();throw Error(label+'; focus='+(active()?.id||active()?.tagName)+'; feedback='+feedback())};
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus({preventScroll:true});await wait(label+' actual native focus',()=>active()===node)};
 const decoded=()=>{const node=image();return !!node&&node.complete&&node.naturalWidth===400&&node.naturalHeight===160};
 const captureImage=()=>{
  const node=image(),source=node?.getAttribute('src')??'';
  const exact=source.startsWith('data:image/svg+xml;base64,')&&atob(source.split(',')[1])===expectedSvg;
  return {decoded:decoded(),naturalWidth:node?.naturalWidth,naturalHeight:node?.naturalHeight,sourcePrefix:source.split(',')[0],svgBytes:expectedSvg.length,exactBundledSvg:exact,alt:node?.alt};
 };
 const open=async label=>{
  await focus(opener(),label+' opener');opener().click();
  await wait(label+' actual native popup and decoded400x160 image',()=>opened()&&decoded()&&action('Save')&&action('Cancel'));
  imageFacts[label]=captureImage();assert(imageFacts[label].exactBundledSvg,label+' preserves exact bundled SVG bytes');
  await wait(label+' popup and actual actions fit native viewport before action focus',()=>fit(panel())&&fit(image())&&fit(action('Save'))&&fit(action('Cancel'))&&hit(action('Save'))&&hit(action('Cancel')));
  layout[label]=geometry();
 };
 const closedAndReturned=async label=>wait(label+' closes native surface and returns actual opener',()=>!opened()&&active()===opener());

 await wait('actual gallery popover trigger/panel mounted',()=>opener()&&panel());
 await wait('native document focus',()=>document.hasFocus());
 const initial=rect(opener());assert(initial.top>=innerHeight/2&&initial.bottom<=innerHeight,'actual opener is in lower half of native viewport');
 snapshots.initial={viewport:viewport(),opener:initial,feedback:feedback()};
 await open('initial');
 await focus(action('Save'),'native visible Save');action('Save').click();
 await wait('actual Save handler records exactly one activation and keeps popup open',()=>feedback()==='Save clicks: 1'&&opened());
 snapshots.save={feedback:feedback(),open:opened(),focus:active()?.id||active()?.textContent.trim()};

 const target=active(),event=new KeyboardEvent('keydown',{key:'Escape',code:'Escape',bubbles:true,cancelable:true});
 target.dispatchEvent(event);escape={synthetic:true,defaultPrevented:event.defaultPrevented,target:target.id||target.textContent.trim()};
 await closedAndReturned('synthetic Escape');snapshots.escape={feedback:feedback(),open:opened(),focus:active()?.id};
 await open('reopen');assert(feedback()==='Save clicks: 1','reopening preserves actual saved action state');
 await focus(action('Cancel'),'native visible Cancel');action('Cancel').click();
 await closedAndReturned('actual Cancel click');
 assert(feedback()==='Save clicks: 1','Cancel does not activate Save');
 snapshots.cancel={feedback:feedback(),open:opened(),focus:active()?.id};
 return {ok:true,snapshots,layout,imageFacts,escape,checks:['actual gallery popup at lower-half opener fits native viewport with reachable Save/Cancel before action focus','exact bundled SVG decodes at400x160 and actual Save records one click','synthetic Escape closes and returns actual opener; reopen preserves image/actions/state and native Cancel closes'],input:'excluded DOM clicks and explicit native focus; Escape is an untrusted synthetic keydown delivered to the actual focused control. No scroll-to-action, fabricated application callback, animation event or trusted keyboard default is used.',trustedInput:'separate lead Chromium verifies trusted pointer/keyboard behavior',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,imageFacts,escape,failureGeometry:Object.keys(failureGeometry).length?failureGeometry:{viewport:{width:innerWidth,height:innerHeight},focus:document.activeElement?.id}}}
"#;
