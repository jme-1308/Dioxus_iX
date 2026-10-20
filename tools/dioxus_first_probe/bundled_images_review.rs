//! Excluded actual-gallery bundled images preview and bounded native DOM replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/action_scenarios.rs"]
mod action_scenarios;
#[path = "../../examples/gallery/application_scenarios.rs"]
mod application_scenarios;
#[path = "../../examples/gallery/demo_images.rs"]
mod demo_images;

#[component]
pub fn BundledImagesPreview() -> Element {
    debug_assert!(action_scenarios::SCENARIOS.contains(&"avatar-image"));
    debug_assert!(application_scenarios::SCENARIOS.contains(&"application-app-switch"));
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {action_scenarios::STYLES} }
        style { {application_scenarios::STYLES} }
        IxTheme { class: "bundled-images-review",
            h1 { "Bundled images" }
            Button { id: "bundled-images-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "bundled-images-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                h2 { "Avatar image" }
                div { id: "bundled-avatar", action_scenarios::ActionScenario { name: "avatar-image" } }
                h2 { "Application switch" }
                div { id: "bundled-application", application_scenarios::ApplicationScenario { name: "application-app-switch" } }
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_BUNDLED_IMAGES_CHECK);
                        if let Err(error) = query.send("bundledimages") {
                            eprintln!("Bundled images native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native bundled images: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native bundled images failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Bundled images native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.bundled-images-review{box-sizing:border-box;min-height:100vh;padding:24px}.bundled-images-review>h1{font-size:28px;margin:0 0 24px}.bundled-images-review #bundled-images-frame>h2{font-size:22px;margin:24px 0 16px}.bundled-images-review p{max-width:72ch;line-height:1.5}#bundled-images-frame{margin-top:24px}@media(max-width:600px){.bundled-images-review{padding:16px}}";

// Excluded native replay; no outbound links are activated and native Escape is represented by a cancel event.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_BUNDLED_IMAGES_CHECK: &str = r#"
let snapshots={},images={},layout={},hitFacts={},failureGeometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement,avatar=()=>get('bundled-avatar')?.querySelector('.ix-avatar'),avatarImage=()=>avatar()?.querySelector('img'),application=()=>get('bundled-application');
 const opener=()=>application()?.querySelector('button[data-app-switch-opener]'),dialog=()=>get(opener()?.getAttribute('aria-controls')),opened=()=>dialog()?.open===true,closeButton=()=>dialog()?.querySelector('button.ix-dialog-close'),entries=()=>[...dialog()?.querySelectorAll('a.ix-application-switch-entry')??[]];
 const config=()=>application()?.querySelector('.ix-dropdown[role="button"][aria-label="Select config"]'),configMenu=()=>get(config()?.getAttribute('aria-controls')),eventOutput=()=>application()?.querySelector('.application-source-events');
 const assert=(condition,label)=>{if(!condition)throw Error(label)},hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const key=(node,value)=>{const event=new KeyboardEvent('keydown',{key:value,code:value,bubbles:true,cancelable:true});node.dispatchEvent(event);return event.defaultPrevented};
 const wait=async(label,predicate)=>{let stable=0;for(let attempt=0;attempt<150;attempt++){stable=predicate()?stable+1:0;if(stable===3)return;await hold(20)}throw Error(label+'; focus='+(active()?.id||active()?.className)+'; dialog='+opened()+'; event='+eventOutput()?.textContent)};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' actual native focus',()=>active()===node)};
 const decoded=node=>!!node&&node.src.startsWith('data:image/svg+xml;')&&node.complete&&node.naturalWidth>0&&node.naturalHeight>0;
 const rect=node=>{if(!node)return null;const r=node.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height}};
 const imageFact=node=>({source:node.getAttribute('src')?.split(',')[0],decoded:decoded(node),naturalWidth:node.naturalWidth,naturalHeight:node.naturalHeight,display:rect(node),alt:node.alt});
 const entryFacts=()=>entries().map(node=>({name:node.querySelector('h3')?.textContent.trim(),description:node.querySelector('p')?.textContent.trim(),href:node.getAttribute('href'),target:node.getAttribute('target'),current:node.getAttribute('aria-current'),selected:node.dataset.selected,image:imageFact(node.querySelector('img'))}));
 const open=async()=>{await focus(opener(),'actual application-switch opener');opener().click();await wait('actual application dialog and bundled images open',()=>opened()&&dialog().matches(':modal')&&entries().length===2&&entries().every(node=>decoded(node.querySelector('img'))))};
 const returned=async label=>wait(label+' closes actual dialog and returns native opener',()=>!opened()&&active()===opener());
 const fit=node=>{const r=node.getBoundingClientRect();return r.width>0&&r.height>0&&r.left>=0&&r.right<=innerWidth+1&&r.top>=0&&r.bottom<=innerHeight+1};
 const hit=async(node,label,insideFrame=true)=>{await focus(node,label);await wait(label+' native hit and fit',()=>{const r=node.getBoundingClientRect(),frame=get('bundled-images-frame').getBoundingClientRect(),point=document.elementFromPoint(r.left+r.width/2,r.top+r.height/2);failureGeometry={node:node.id||node.className,rect:rect(node),frame:rect(get('bundled-images-frame')),dialog:rect(dialog()),hit:point?.id||point?.className||point?.tagName,viewport:{width:innerWidth,height:innerHeight},pageWidth:document.documentElement.scrollWidth};return fit(node)&&(!insideFrame||(r.left>=frame.left-1&&r.right<=frame.right+1))&&(point===node||node.contains(point))});hitFacts[label]={...failureGeometry}};

 await wait('actual avatar/application scenarios mounted with decoded bundled avatar',()=>avatar()&&decoded(avatarImage())&&opener()&&config()&&eventOutput());await wait('native document ready for focus',()=>document.hasFocus());
 assert(Math.abs(avatar().getBoundingClientRect().width-32)<1&&Math.abs(avatar().getBoundingClientRect().height-32)<1&&Math.abs(avatarImage().getBoundingClientRect().width-32)<1&&Math.abs(avatarImage().getBoundingClientRect().height-32)<1,'actual bundled avatar preserves32px face and image geometry');assert(![...get('bundled-avatar').querySelectorAll('img')].some(node=>!node.src.startsWith('data:image/svg+xml;')),'named avatar scenario has no remote image source');images.avatar=imageFact(avatarImage());snapshots.avatar={face:rect(avatar()),event:eventOutput().textContent};

 await open();const firstEntries=entryFacts();assert(dialog().getAttribute('aria-label')==='Switch to Application'&&firstEntries.map(node=>node.name).join('|')==='Floor App|Calculator App'&&firstEntries.map(node=>node.description).join('|')==='Example description for Floor app|Example description for Calculator app','actual dialog names/descriptions unchanged');assert(firstEntries.every(node=>node.href==='https://ix.siemens.io/'&&node.target==='_self'&&node.image.decoded&&Math.abs(node.image.display.width-48)<1&&Math.abs(node.image.display.height-48)<1)&&firstEntries[0].current===null&&firstEntries[0].selected==='false'&&firstEntries[1].current==='page'&&firstEntries[1].selected==='true','actual bundled48px app images retain unchanged href/target/current-app metadata');assert(entries()[0].querySelector('img').src!==entries()[1].querySelector('img').src,'floor/calculator roles use distinct bundled art');images.applications=firstEntries;snapshots.dialog={open:opened(),dialog:rect(dialog()),event:eventOutput().textContent};

 await focus(closeButton(),'native dialog Escape/cancel target');const cancelled=dialog();cancelled.dispatchEvent(new Event('cancel',{bubbles:false,cancelable:true}));await returned('synthetic native cancel representing Escape');await open();assert(entryFacts().every(node=>node.image.decoded)&&eventOutput().textContent.trim()==='Event: No event yet','reopened actual dialog retains bundled sources and local event state');await focus(closeButton(),'actual dialog Close');closeButton().click();await returned('native close button');await focus(config(),'actual source Select config');config().click();await wait('actual shared config dropdown opens',()=>config().getAttribute('aria-expanded')==='true'&&configMenu()&&!configMenu().hidden);const item=[...configMenu().querySelectorAll('[role="menuitem"]')].find(node=>node.textContent.trim()==='Config 2');assert(item,'actual Config2 action exists');item.click();await wait('ordinary source config action changes real local output',()=>eventOutput().textContent.trim()==='Event: Config 2'&&config().getAttribute('aria-expanded')==='false');const pointerEvent=eventOutput().textContent;await focus(config(),'shared config keyboard owner');config().click();await wait('shared config dropdown reopens for semantic keyboard action',()=>config().getAttribute('aria-expanded')==='true'&&configMenu()&&!configMenu().hidden);await focus(config(),'active-descendant config keyboard owner');key(config(),'End');await wait('handled End selects actual Config3 active descendant',()=>get(config().getAttribute('aria-activedescendant'))?.textContent.trim()==='Config 3');const keyboardTarget=config().getAttribute('aria-activedescendant');key(config(),'Enter');await wait('semantic Enter action changes actual output and closes menu without mouse activation',()=>eventOutput().textContent.trim()==='Event: Config 3'&&config().getAttribute('aria-expanded')==='false');snapshots.reopenAndAction={pointerEvent,event:eventOutput().textContent,keyboardTarget,keyboardPath:'synthetic End/Enter dispatched to actual active-descendant owner; no mouse callback for Config3',dialogOpen:opened(),focus:active()?.id};

 get('bundled-images-narrow').click();await wait('actual340px scenario frame and no page overflow',()=>get('bundled-images-frame').getBoundingClientRect().width<=341&&document.documentElement.scrollWidth<=innerWidth+1);assert(decoded(avatarImage())&&Math.abs(avatar().getBoundingClientRect().width-32)<1,'narrow avatar remains decoded at32px');await hit(opener(),'narrow actual application opener');await open();await hit(closeButton(),'actual native dialog close',false);assert(fit(dialog())&&entries().every(node=>fit(node)&&fit(node.querySelector('img'))&&decoded(node.querySelector('img'))&&Math.abs(node.querySelector('img').getBoundingClientRect().width-48)<1),'real dialog/cards/48px bundled images fit actual native viewport');layout={frame:rect(get('bundled-images-frame')),avatar:rect(avatar()),application:rect(application()),dialog:rect(dialog()),entries:entryFacts(),viewport:{width:innerWidth,height:innerHeight},pageWidth:document.documentElement.scrollWidth};closeButton().click();await returned('narrow native close');assert(eventOutput().textContent.trim()==='Event: Config 3'&&document.documentElement.scrollWidth<=innerWidth+1,'narrow open/close preserves actual config output and contained page');snapshots.narrow={event:eventOutput().textContent,dialogOpen:opened(),focus:active()?.id};
 return {ok:true,snapshots,images,layout,hitFacts,checks:['actual32px avatar decodes from bundled data URI','actual app dialog decodes distinct48px images with unchanged names/current/link metadata','native cancel/close/reopen focus, Config2 pointer action and Config3 semantic keyboard action','340px scenario fit/native opener/close hits and real dialog/image containment'],input:'excluded DOM clicks, explicit native focus/normal page positioning and synthetic native cancel event representing Escape; Config3 uses handled synthetic End/Enter on the shared active-descendant owner, without a mouse callback; untrusted native keyboard defaults are not simulated and no application link is activated',viewportBoundary:'340px scenario frame does not resize the native top-layer viewport; dialog/cards/images are checked against reported actual viewport. Lead Chromium supplies true narrow-viewport evidence.',trustedInput:'separate lead Chromium trusted Escape/Tab/pointer and paired desktop-dark/narrow-light inspection',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,images,layout,hitFacts,failureGeometry}}
"#;
