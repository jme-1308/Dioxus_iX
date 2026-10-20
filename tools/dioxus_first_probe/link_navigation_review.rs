//! Excluded actual-gallery link navigation preview and bounded native DOM replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/link_navigation.rs"]
mod gallery_links;

#[component]
pub fn NativeLinkNavigationPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_links::STYLES} }
        IxTheme { class: "link-navigation-review",
            h1 { "Maintenance references" }
            Button { id: "link-navigation-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "link-navigation-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_links::MaintenanceReferencesExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_LINK_NAVIGATION_CHECK);
                        if let Err(error) = query.send("linknavigation") {
                            eprintln!("Maintenance references native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native link navigation: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native link navigation failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Maintenance references native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.link-navigation-review{box-sizing:border-box;min-height:100vh;padding:24px}.link-navigation-review>h1{font-size:28px;margin:0 0 24px}.link-navigation-review h2{font-size:22px;margin:24px 0 16px}.link-navigation-review p{max-width:72ch;line-height:1.5}#link-navigation-frame{margin-top:24px}@media(max-width:600px){.link-navigation-review{padding:16px}}";

// Excluded native replay. Pinned dioxus-interpreter-js native.ts intercepts
// desktop anchor defaults (intercept_link_redirects/handleClickNavigate), even
// without href. The gallery uses mounted scroll/focus for its desktop checklist;
// web retains ordinary fragment navigation. Trusted Enter/Tab is separate evidence.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_LINK_NAVIGATION_CHECK: &str = r#"
let snapshots={},layout={},hitFacts={},defaultFacts={},failureGeometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement,result=()=>get('references-result');
 const link=kind=>get('references-link-'+kind)?.querySelector('a'),note=()=>get('references-note'),article=()=>get('references-article');
 const count=name=>Number(result()?.dataset[name]),selected=()=>result()?.dataset.selected;
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{let stable=0;for(let attempt=0;attempt<150;attempt++){stable=predicate()?stable+1:0;if(stable===3)return;await hold(20)}throw Error(label+'; state='+JSON.stringify(result()?.dataset)+'; hash='+location.hash+'; focus='+(active()?.id||active()?.tagName))};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' actual native focus',()=>active()===node)};
 const setNote=async text=>{const node=note();Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,text);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait('actual retained preparation changed',()=>note()===node&&node.value===text)};
 const hitTestable=node=>{
  const box=node?.getBoundingClientRect(),frame=get('link-navigation-frame')?.getBoundingClientRect(),hit=box?document.elementFromPoint(box.left+box.width/2,box.top+box.height/2):null;
  failureGeometry={node:node?.id||node?.textContent,nodeRect:box?{left:box.left,right:box.right,top:box.top,bottom:box.bottom,width:box.width,height:box.height}:null,frameRect:frame?{left:frame.left,right:frame.right,width:frame.width}:null,innerWidth,innerHeight,scrollTop:document.documentElement.scrollTop,hit:hit?{id:hit.id,tag:hit.tagName,className:hit.className}:null};
  return !!box&&box.width>0&&box.height>0&&box.left>=frame.left-1&&box.right<=frame.right+1&&box.left>=0&&box.right<=innerWidth+1&&box.top>=0&&box.bottom<=innerHeight+1&&(hit===node||node.contains(hit));
 };
 const clickObserved=node=>{let event;node.addEventListener('click',value=>{event=value},{once:true});node.click();return ()=>event?{defaultPrevented:event.defaultPrevented,isTrusted:event.isTrusted}:null};
 const navigate=async kind=>{const before=count('navigations'),hash=location.hash,observe=clickObserved(link(kind));await wait('consumer-cancelled '+kind+' link changes actual article without native navigation',()=>selected()===kind&&article().dataset.reference===kind&&get('references-heading').textContent===(kind==='motor'?'Motor 001 reference':'Pump 002 reference')&&count('navigations')===before+1&&location.hash===hash);defaultFacts[kind]=observe();assert(defaultFacts[kind]?.defaultPrevented===true,'consumer-cancelled '+kind+' native click default prevented')};
 const snapshot=()=>({selected:selected(),navigations:count('navigations'),disabledRequests:count('disabledRequests'),disabled:result()?.dataset.disabled,heading:get('references-heading')?.textContent,body:get('references-body')?.textContent,note:note()?.value,hash:location.hash});

 await wait('actual reference links/article/preparation mounted',()=>result()&&article()&&note()&&['motor','pump','disabled','native','missing','blank'].every(kind=>link(kind)));
 await wait('native WebView ready for focus',()=>document.hasFocus());
 assert(selected()==='motor'&&article().dataset.reference==='motor'&&count('navigations')===0&&count('disabledRequests')===0&&note().value==='Prepare Motor 001 review','initial real reference article and preparation');
 const options=get('references-options');options.querySelector('summary').click();await wait('native reference options expanded',()=>options.open);
 for(const kind of ['motor','pump','native']){const node=link(kind);assert(node.tagName==='A'&&node.hasAttribute('href')&&!node.hasAttribute('tabindex')&&!node.hasAttribute('title')&&node.getAttribute('aria-disabled')==='false'&&node.getAttribute('target')==='_self','enabled '+kind+' preserves ordinary native anchor semantics');assert(node.querySelector('svg[aria-hidden="true"]')&&node.firstElementChild.tagName.toLowerCase()==='svg'&&node.textContent.trim().length>0,'leading decorative glyph and caller link content '+kind)}
 assert(link('motor').getAttribute('href')==='#references-motor'&&link('pump').getAttribute('href')==='#references-pump'&&link('native').getAttribute('href')==='#references-checklist','actual supplied native destinations');
 assert(!link('missing').hasAttribute('href')&&!link('missing').hasAttribute('tabindex')&&!link('missing').hasAttribute('role')&&link('blank').getAttribute('target')==='_blank'&&['noopener','noreferrer'].every(value=>link('blank').rel.split(/\s+/).includes(value)),'missing destination remains inert native anchor and blank target retains safe rel');
 const absentHash=location.hash,absentNavigations=count('navigations');const missingEvent=clickObserved(link('missing'));await wait('missing URL does not fabricate navigation or callbacks',()=>location.hash===absentHash&&count('navigations')===absentNavigations&&selected()==='motor');defaultFacts.missing=missingEvent();snapshots.semantics=snapshot();

 const retainedNote=note(),initialBody=get('references-body').textContent;await focus(note(),'native reference preparation');await wait('actual preparation input receives pointer hit',()=>hitTestable(note()));hitFacts.note=failureGeometry.hit;await setNote('Prepare pump seal review');
 await focus(link('pump'),'native Pump reference');await wait('actual Pump link receives pointer hit',()=>hitTestable(link('pump')));hitFacts.pump=failureGeometry.hit;await navigate('pump');
 assert(get('references-body').textContent!==initialBody&&note()===retainedNote&&note().value==='Prepare pump seal review','actual article body changes while independent preparation node/draft persist');await navigate('motor');assert(get('references-body').textContent===initialBody&&note()===retainedNote,'return navigation restores real Motor article and retained preparation');snapshots.articles=snapshot();

 get('references-disable').click();await wait('live disabled option removes native destinations',()=>result().dataset.disabled==='true'&&['motor','pump','disabled'].every(kind=>!link(kind).hasAttribute('href')&&link(kind).getAttribute('role')==='link'&&link(kind).getAttribute('aria-disabled')==='true'&&link(kind).getAttribute('tabindex')==='-1'));
 const blocked={hash:location.hash,navigations:count('navigations')};for(const kind of ['motor','pump','disabled']){const observe=clickObserved(link(kind));await wait('disabled '+kind+' has no callback or native navigation',()=>count('navigations')===blocked.navigations&&count('disabledRequests')===0&&location.hash===blocked.hash&&selected()==='motor');defaultFacts['disabled '+kind]=observe();assert(defaultFacts['disabled '+kind]?.defaultPrevented===true,'disabled '+kind+' actual native click default prevented')}
 get('references-disable').click();await wait('enabled native destinations restored',()=>result().dataset.disabled==='false'&&link('pump').getAttribute('href')==='#references-pump'&&!link('pump').hasAttribute('tabindex'));await navigate('pump');assert(note()===retainedNote&&note().value==='Prepare pump seal review','disabled option changes retain independent preparation');snapshots.disabled=snapshot();

 const target=get('references-checklist'),beforeNative={navigations:count('navigations'),selected:selected(),hash:location.hash};await focus(link('native'),'desktop checklist link');await wait('actual checklist link receives pointer hit',()=>hitTestable(link('native')));const nativeEvent=clickObserved(link('native'));
 await wait('Dioxus mounted scroll/focus reveals desktop checklist',()=>target&&active()===target&&target.getBoundingClientRect().bottom>0&&target.getBoundingClientRect().top<innerHeight);defaultFacts.native=nativeEvent();
 assert(defaultFacts.native?.defaultPrevented===true&&active()===target&&location.hash===beforeNative.hash&&count('navigations')===beforeNative.navigations&&selected()===beforeNative.selected&&note()===retainedNote&&note().value==='Prepare pump seal review','desktop framework default prevented; mounted target revealed/focused with hash/article/preparation unchanged');snapshots.nativeFragment=snapshot();

 const reset=get('references-reset'),oldNote=note();await focus(reset,'Restore maintenance references');reset.click();await wait('Restore resets actual selected article/counters and owned preparation',()=>selected()==='motor'&&article().dataset.reference==='motor'&&count('navigations')===0&&count('disabledRequests')===0&&result().dataset.disabled==='false'&&note()?.value==='Prepare Motor 001 review');assert(!oldNote.isConnected&&note()!==oldNote&&get('references-reset')===reset&&get('references-options')===options&&active()===reset,'Restore replaces owned note while preserving toolbar/options/focus');
 get('link-navigation-narrow').click();await wait('340px reference preview without horizontal page overflow',()=>get('link-navigation-frame').getBoundingClientRect().width<=341&&document.documentElement.scrollWidth<=document.documentElement.clientWidth+1);
 for(const kind of ['motor','pump','native']){await focus(link(kind),'narrow native '+kind+' link');await wait('narrow '+kind+' native link fits and receives pointer hit',()=>hitTestable(link(kind)));hitFacts['narrow '+kind]=failureGeometry.hit}
 await focus(note(),'narrow actual preparation');await wait('narrow native preparation fits and receives pointer hit',()=>hitTestable(note()));await setNote('Narrow pump reference review');await navigate('pump');
 const frame=get('link-navigation-frame').getBoundingClientRect(),input=note().getBoundingClientRect();layout={frameWidth:frame.width,inputWidth:input.width,inputHeight:input.height,links:['motor','pump','native'].map(kind=>({kind,width:link(kind).getBoundingClientRect().width,height:link(kind).getBoundingClientRect().height})),pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};assert(note().value==='Narrow pump reference review'&&count('navigations')===1&&selected()==='pump'&&count('disabledRequests')===0&&layout.inputHeight>=31&&layout.pageScroll<=layout.pageClient+1,'narrow article navigation changes actual content and retains native preparation');snapshots.resetNarrow=snapshot();
 return {ok:true,snapshots,layout,hitFacts,defaultFacts,checks:['enabled/missing destination/target/rel and leading decorative icon preserve native anchor semantics','consumer-cancelled link clicks change actual article and retain independent native note','disabled links remove href/Tab admission and suppress callback/default navigation','desktop checklist click uses Dioxus mounted scroll/focus with native default prevented and hash/model unchanged','scoped reset preserves toolbar focus;340px native links/note hit-fit and actual navigation'],input:'actual DOM click activation, synthetic input events, explicit native focus and normal page scroll setup; no synthetic key events',frameworkBoundary:'Pinned dioxus-interpreter-js native.ts intercepts all desktop anchor defaults, including missing href; desktop gallery checklist uses mounted scroll/focus. Ordinary web fragment hash navigation is verified separately in Chromium.',trustedInput:'separate Chromium trusted Enter/Tab/pointer/typing, native fragment hash navigation and visual evidence; native driver does not certify trusted keyboard defaults',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,hitFacts,defaultFacts,failureGeometry,documentFocusedNow:document.hasFocus()}}
"#;
