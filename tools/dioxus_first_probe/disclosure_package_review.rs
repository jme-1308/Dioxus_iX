//! Excluded actual-gallery retained disclosures and bounded Linux DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/service_package.rs"]
mod gallery_package;

#[component]
pub fn DisclosurePackagePreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_package::STYLES} }
        IxTheme { class: "disclosure-package-review",
            h1 { "Retained service package" }
            Button { id: "disclosure-package-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            // The actual gallery lives in an ancestor scrollport so mounted
            // nearest-edge reveal can be observed without timing instrumentation.
            div { id: "disclosure-package-scroll",
                div { id: "disclosure-package-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                    gallery_package::ServicePackageExample {}
                }
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_DISCLOSURE_PACKAGE_CHECK);
                        if let Err(error) = query.send("disclosurepackage") {
                            eprintln!("Disclosure package native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native disclosure package: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native disclosure package failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Disclosure package native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.disclosure-package-review{box-sizing:border-box;min-height:100vh;padding:24px}.disclosure-package-review h1{font-size:28px;margin:0 0 24px}.disclosure-package-review h2{font-size:22px;margin:24px 0 16px}.disclosure-package-review h3{font-size:18px}.disclosure-package-review p{max-width:72ch;line-height:1.5}#disclosure-package-scroll{box-sizing:border-box;height:260px;max-width:100%;min-width:0;overflow:auto;margin-top:24px}@media(max-width:600px){.disclosure-package-review{padding:16px}}";

// Excluded driver JS sends synthetic form/click/key events and requests native
// focus. Actual Dioxus components own controlled/uncontrolled expansion, retained
// children, ARIA links and mounted reveal. Untrusted native Enter/Space do not
// activate a button: permission is checked, then one explicit click is used.
// Trusted Chromium separately verifies native keyboard defaults and Tab order.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_DISCLOSURE_PACKAGE_CHECK: &str = r#"
let snapshots={},layout={},keyPermissions={},revealFacts={},failureGeometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const host=()=>get('service-package'),result=()=>get('service-package-result'),port=()=>get('disclosure-package-scroll');
 const parts=()=>get('package-parts'),readiness=()=>get('package-readiness');
 const toggle=root=>root()?.querySelector('button[aria-controls][aria-expanded]');
 const partsToggle=()=>toggle(parts),readinessToggle=()=>toggle(readiness);
 const panel=button=>get(button?.getAttribute('aria-controls'));
 const partsPanel=()=>panel(partsToggle()),readinessPanel=()=>panel(readinessToggle());
 const open=button=>button?.getAttribute('aria-expanded')==='true';
 const partsOpen=()=>open(partsToggle()),readinessOpen=()=>open(readinessToggle());
 const selection=()=>get('service-package-card')?.querySelector('button[aria-label="Select Motor 001 package"]');
 const note=()=>get('package-parts-note'),readinessNote=()=>get('package-readiness-note');
 const count=name=>Number(result()?.dataset[name]);
 const checked=node=>node instanceof HTMLInputElement?node.checked:node.getAttribute('aria-checked')==='true';
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const wait=async(label,predicate)=>{
  let stable=0;
  for(let attempt=0;attempt<150;attempt++){
   stable=predicate()?stable+1:0;
   if(stable===3)return;
   await hold(20);
  }
  throw Error(label+'; result='+result()?.textContent+'; parts='+partsToggle()?.getAttribute('aria-expanded')+'; readiness='+readinessToggle()?.getAttribute('aria-expanded')+'; focus='+(active()?.getAttribute('aria-label')||active()?.id||active()?.tagName));
 };
 const key=(node,key)=>{const event=new KeyboardEvent('keydown',{key,code:key===' '?'Space':key,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus();await wait(label+' native focus established',()=>active()===node)};
 const setInput=async(node,value,label)=>{
  Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));
  await wait(label+' actual native input applied',()=>node.isConnected&&node.value===value);
 };
 const setParts=async desired=>{if(partsOpen()!==desired)partsToggle().click();await wait('controlled parts '+desired+' reflected in actual panel',()=>partsOpen()===desired&&partsPanel().hidden===!desired)};
 const setReadiness=async desired=>{if(readinessOpen()!==desired)readinessToggle().click();await wait('uncontrolled readiness '+desired+' reflected in actual panel',()=>readinessOpen()===desired&&readinessPanel().hidden===!desired)};
 const linked=button=>{
  const region=panel(button);
  return button?.id&&region?.id&&region.getAttribute('role')==='region'&&region.getAttribute('aria-labelledby')===button.id&&button.getAttribute('aria-controls')===region.id;
 };
 const snapshot=()=>({partsOpen:partsOpen(),readinessOpen:readinessOpen(),requests:count('partsRequests'),selections:count('cardSelections'),inspections:count('inspections'),selected:selection()?.getAttribute('aria-pressed'),partsNote:note()?.value,readinessNote:readinessNote()?.value,seal:checked(get('package-seal-kit')),permit:checked(get('package-permit')),held:checked(get('package-hold')),custom:checked(get('package-custom-header')),reveal:checked(get('package-reveal')),variant:get('package-variant')?.value});
 const hitTestable=node=>{
  const box=node?.getBoundingClientRect(),viewport=port().getBoundingClientRect();
  const rect=element=>{const value=element?.getBoundingClientRect();return value?{left:value.left,top:value.top,right:value.right,bottom:value.bottom,width:value.width,height:value.height}:null};
  const hit=box?document.elementFromPoint(box.left+box.width/2,box.top+box.height/2):null;
  const ancestry=[];
  for(let element=node; element&&ancestry.length<8; element=element.parentElement){
   const style=getComputedStyle(element);ancestry.push({tag:element.tagName,id:element.id,className:element.className,rect:rect(element),pointerEvents:style.pointerEvents,zIndex:style.zIndex,overflow:style.overflow,display:style.display,visibility:style.visibility});
  }
  failureGeometry={node:node?.id,active:active()?.id,nodeRect:rect(node),panelRect:rect(node?.closest('[role="region"]')),scrollport:rect(port()),frame:rect(get('disclosure-package-frame')),portClientWidth:port().clientWidth,portClientHeight:port().clientHeight,scrollTop:port().scrollTop,scrollLeft:port().scrollLeft,scrollHeight:port().scrollHeight,scrollWidth:port().scrollWidth,innerWidth,innerHeight,documentScrollTop:document.documentElement.scrollTop,centerHit:hit?{tag:hit.tagName,id:hit.id,className:hit.className}:null,ancestry};
  if(!box||box.width<=0||box.height<=0||box.left<viewport.left-1||box.right>viewport.left+port().clientWidth+1||box.top<viewport.top-1||box.bottom>viewport.top+port().clientHeight+1||box.top<0||box.bottom>innerHeight+1)return false;
  return hit===node||node.contains(hit);
 };

 await wait('actual retained service-package gallery mounted',()=>host()&&partsToggle()&&readinessToggle()&&note()&&readinessNote()&&selection()&&result());
 await wait('native WebView ready for focus',()=>document.hasFocus());
 const initialPartsNote=note().value,initialReadinessNote=readinessNote().value,initialHeader=readinessToggle().textContent.trim();
 assert(initialPartsNote==='Stage seal kit beside Motor 001'&&initialReadinessNote==='Review permit before service','initial local note fields match actual service example');
 assert(linked(partsToggle())&&linked(readinessToggle())&&partsPanel().id!==readinessPanel().id&&partsToggle().id!==readinessToggle().id,'stable component-owned toggle/panel IDs connect both actual regions');
 assert(partsToggle().getAttribute('aria-label')==='Show planned maintenance parts'&&readinessToggle().getAttribute('aria-label')==='Service readiness for Motor 001','explicit native disclosure button names retained');
 assert(!partsOpen()&&!readinessOpen()&&partsPanel().hidden&&readinessPanel().hidden&&note().isConnected&&readinessNote().isConnected&&note().getClientRects().length===0&&readinessNote().getClientRects().length===0,'closed actual panels retain children while native hidden excludes their layout');
 const originalIds={partsToggle:partsToggle().id,partsPanel:partsPanel().id,readinessToggle:readinessToggle().id,readinessPanel:readinessPanel().id};
 await focus(selection(),'independent Card selection button');note().focus();assert(active()===selection(),'closed retained parts input cannot take native focus');snapshots.relationships=snapshot();

 await setParts(true);await setParts(false);
 for(const value of ['Enter',' ']){
  await focus(partsToggle(),'native parts disclosure toggle');const before=count('partsRequests'),wasOpen=partsOpen(),event=key(partsToggle(),value);keyPermissions['parts'+value]=!event.defaultPrevented;
  assert(!event.defaultPrevented,'native parts '+value+' default is preserved');
  await wait('untrusted parts key alone does not fabricate a request',()=>count('partsRequests')===before&&partsOpen()===wasOpen);
  partsToggle().click();await wait('explicit native click toggles controlled parts once',()=>count('partsRequests')===before+1&&partsOpen()===!wasOpen&&partsPanel().hidden===wasOpen);
 }
 await setReadiness(true);await focus(readinessToggle(),'native readiness disclosure toggle');const readinessKey=key(readinessToggle(),' ');keyPermissions.readinessSpace=!readinessKey.defaultPrevented;
 assert(!readinessKey.defaultPrevented&&readinessOpen(),'readiness preserves native Space default without fabricated activation');
 readinessToggle().click();await wait('explicit native readiness click closes uncontrolled panel',()=>!readinessOpen()&&readinessPanel().hidden);
 assert(count('cardSelections')===0,'disclosure activation never selects Card');snapshots.activation=snapshot();

 await setParts(true);const retainedParts=note(),retainedSeal=get('package-seal-kit');await focus(note(),'actual staging note');await setInput(note(),'Stage seals by Motor 001','staging note');
 get('package-seal-kit').click();await wait('actual retained seal checkbox checked',()=>get('package-seal-kit').checked);
 await setParts(false);await setParts(true);assert(note()===retainedParts&&get('package-seal-kit')===retainedSeal&&note().value==='Stage seals by Motor 001'&&get('package-seal-kit').checked,'parts close/reopen retains native nodes, local note and checkbox state');
 await setReadiness(true);const retainedReadiness=readinessNote(),retainedPermit=get('package-permit');await focus(readinessNote(),'actual readiness note');await setInput(readinessNote(),'Permit reviewed by service team','readiness note');
 get('package-permit').click();await wait('actual retained permit checkbox checked',()=>get('package-permit').checked);
 await setReadiness(false);await setReadiness(true);assert(readinessNote()===retainedReadiness&&get('package-permit')===retainedPermit&&readinessNote().value==='Permit reviewed by service team'&&get('package-permit').checked,'uncontrolled readiness retains independent local content');
 assert(partsToggle().id===originalIds.partsToggle&&partsPanel().id===originalIds.partsPanel&&readinessToggle().id===originalIds.readinessToggle&&readinessPanel().id===originalIds.readinessPanel&&linked(partsToggle())&&linked(readinessToggle()),'ordinary state changes preserve actual ARIA identities');snapshots.retention=snapshot();

 const beforeInspect=count('inspections'),beforeRequests=count('partsRequests');get('package-inspect').click();
 await wait('independent header action records one real inspection',()=>count('inspections')===beforeInspect+1);
 assert(readinessOpen()&&partsOpen()&&count('partsRequests')===beforeRequests&&count('cardSelections')===0,'header action neither expands/collapses nor selects Card');
 selection().click();await wait('actual Card primary selection records only its own action',()=>count('cardSelections')===1&&selection().getAttribute('aria-pressed')==='true');
 await setParts(false);assert(count('cardSelections')===1&&readinessOpen(),'controlled disclosure request leaves Card selection independent');snapshots.independentActions=snapshot();

 get('package-hold').click();await wait('controlled parts requests held by host',()=>checked(get('package-hold')));
 const heldBefore=count('partsRequests');partsToggle().click();await wait('held closed request is recorded without opening actual panel',()=>count('partsRequests')===heldBefore+1&&!partsOpen()&&partsPanel().hidden);
 const externalBefore=count('partsRequests');get('package-toggle-parts').click();await wait('external host update opens held controlled panel without another request',()=>partsOpen()&&!partsPanel().hidden&&count('partsRequests')===externalBefore);
 partsToggle().click();await wait('held close request leaves actual controlled panel open',()=>count('partsRequests')===externalBefore+1&&partsOpen());
 const closedExternally=count('partsRequests');get('package-toggle-parts').click();await wait('external close updates actual panel without inventing request',()=>!partsOpen()&&partsPanel().hidden&&count('partsRequests')===closedExternally);
 assert(note()===retainedParts&&note().value==='Stage seals by Motor 001'&&get('package-seal-kit').checked,'controlled holding/external updates preserve local child state');
 get('package-hold').click();await wait('host resumes normal controlled requests',()=>!checked(get('package-hold')));snapshots.heldControlled=snapshot();

 get('package-custom-header').click();await wait('fallback readiness heading replaces custom presentation',()=>!checked(get('package-custom-header'))&&readinessToggle().textContent.includes('Service readiness')&&readinessToggle().textContent.trim()!==initialHeader);
 assert(readinessOpen()&&readinessNote()===retainedReadiness&&readinessNote().value==='Permit reviewed by service team'&&get('package-permit').checked&&readinessToggle().id===originalIds.readinessToggle&&linked(readinessToggle()),'header replacement retains uncontrolled expansion/body and ARIA links');
 get('package-custom-header').click();await wait('custom heading presentation restored',()=>checked(get('package-custom-header'))&&readinessToggle().textContent.trim()===initialHeader);
 assert(!readinessToggle().querySelector('button,input,select,a'),'custom header contains no nested interactive controls');snapshots.customHeader=snapshot();

 for(const variant of ['filled','warning','primary']){
  const node=get('package-variant');node.value=variant;node.dispatchEvent(new Event('change',{bubbles:true}));
  await wait('semantic '+variant+' surfaces reflect host appearance',()=>node.value===variant&&parts().classList.contains('ix-card-'+variant)&&readiness().classList.contains('ix-card-'+variant));
 }
 assert(note()===retainedParts&&readinessNote()===retainedReadiness,'semantic appearance changes preserve retained bodies');
 get('package-reveal').click();await wait('optional reveal disabled without remounting body',()=>!checked(get('package-reveal'))&&note()===retainedParts);await setParts(true);await setParts(false);
 get('package-reveal').click();await wait('mounted opening reveal option enabled',()=>checked(get('package-reveal')));
 await focus(partsToggle(),'closed parts toggle before real mounted reveal');
 const beforePort=port().getBoundingClientRect(),beforeToggle=partsToggle().getBoundingClientRect(),beforeScroll=port().scrollTop;
 partsToggle().click();await wait('actual mounted parts reveal makes the real region visible inside ancestor scrollport',()=>{
  if(!partsOpen()||partsPanel().hidden)return false;
  const box=partsPanel().getBoundingClientRect(),viewport=port().getBoundingClientRect();return box.height>0&&box.top>=viewport.top-1&&box.bottom<=viewport.top+port().clientHeight+1;
 });
 const afterPanel=partsPanel().getBoundingClientRect();
 revealFacts={scrollBefore:beforeScroll,scrollAfter:port().scrollTop,closedToggleBottom:beforeToggle.bottom,viewportTop:beforePort.top,viewportBottom:beforePort.top+port().clientHeight,panelTop:afterPanel.top,panelBottom:afterPanel.bottom,panelHeight:afterPanel.height};snapshots.variantsReveal=snapshot();

 const resetButton=get('package-reset'),oldParts=note(),oldReadiness=readinessNote();resetButton.focus();resetButton.click();
 await wait('Restore service package clears owned fields/options/counts and closes both disclosures',()=>!partsOpen()&&!readinessOpen()&&partsPanel().hidden&&readinessPanel().hidden&&note()?.value===initialPartsNote&&readinessNote()?.value===initialReadinessNote&&!get('package-seal-kit').checked&&!get('package-permit').checked&&count('partsRequests')===0&&count('cardSelections')===0&&count('inspections')===0&&selection().getAttribute('aria-pressed')==='false'&&!checked(get('package-hold'))&&checked(get('package-custom-header'))&&checked(get('package-reveal'))&&get('package-variant').value==='outline');
 assert(!oldParts.isConnected&&!oldReadiness.isConnected&&note()!==oldParts&&readinessNote()!==oldReadiness&&get('package-reset')===resetButton&&active()===resetButton&&linked(partsToggle())&&linked(readinessToggle()),'reset replaces only owned disclosure scopes and preserves toolbar native focus');snapshots.reset=snapshot();

 get('disclosure-package-narrow').click();await wait('340px disclosure frame applied without horizontal page overflow',()=>get('disclosure-package-frame').getBoundingClientRect().width<=341&&document.documentElement.scrollWidth<=document.documentElement.clientWidth+1);
 await setParts(true);
 // Wait for the actual opening reveal to expose its region before requesting
 // a second native focus scroll. This is a geometry condition, not a delay.
 await wait('narrow opening reveal exposes the actual retained parts region',()=>{
  hitTestable(note());
  if(!partsOpen()||partsPanel().hidden)return false;
  const box=partsPanel().getBoundingClientRect(),viewport=port().getBoundingClientRect();return box.height>0&&box.top>=viewport.top-1&&box.bottom<=viewport.top+port().clientHeight+1;
 });
 await focus(note(),'narrow open staging note');await wait('narrow actual staging input visible and hit-testable',()=>hitTestable(note()));
 await setInput(note(),'Narrow staging note','narrow staging note');get('package-seal-kit').click();await wait('narrow native seal checkbox remains usable',()=>get('package-seal-kit').checked);
 await setReadiness(true);
 // Probe-only viewport setup: WebKit focus can expose the caret while leaving
 // the field's outer edge clipped. A user can scroll the retained Blind body;
 // position that native viewport before testing full-box input usability.
 readinessNote().scrollIntoView({block:'nearest',inline:'nearest'});
 await focus(readinessNote(),'narrow open readiness note');await wait('narrow actual readiness input visible and hit-testable',()=>hitTestable(readinessNote()));
 await setInput(readinessNote(),'Narrow permit note','narrow readiness note');get('package-inspect').click();await wait('narrow sibling header action remains usable',()=>count('inspections')===1&&readinessOpen());
 const portBox=port().getBoundingClientRect(),inputBox=readinessNote().getBoundingClientRect();
 layout={frameWidth:get('disclosure-package-frame').getBoundingClientRect().width,scrollportWidth:portBox.width,scrollportHeight:portBox.height,clientWidth:port().clientWidth,scrollWidth:port().scrollWidth,inputWidth:inputBox.width,inputHeight:inputBox.height,pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};
 assert(note().value==='Narrow staging note'&&readinessNote().value==='Narrow permit note'&&get('package-seal-kit').checked&&count('cardSelections')===0&&layout.pageScroll<=layout.pageClient+1,'narrow retained fields and independent actions preserve usable contained layout');snapshots.narrow=snapshot();
 return {ok:true,snapshots,layout,keyPermissions,revealFacts,
  checks:['stable actual native toggle/region ARIA links and hidden retained fields','native Enter/Space permission plus explicit pointer activation without Card selection','actual local notes/checks and native identities retained across close/open','independent header inspection and Card primary selection','held controlled requests and external updates preserve accepted state/content','custom/fallback readiness header replacement retains uncontrolled body','semantic variants and actual mounted reveal into native ancestor scrollport','Restore resets only owned bodies/options/counts and preserves toolbar focus','340px native open field hit testing and independent actions'],
  input:'synthetic form/click/key events, explicit native focus and readiness viewport setup; native button Enter/Space and Tab defaults are not simulated',
  trustedInput:'separate Chromium native Enter/Space/Tab/input and visual checks; reveal call suppression/lifecycle guards covered by Rust tests; no physical-device certification',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,keyPermissions,revealFacts,failureGeometry,documentFocusedNow:document.hasFocus()}}
"#;
