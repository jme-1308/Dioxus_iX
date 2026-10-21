//! Excluded actual-gallery Content body scrolling and bounded synthetic native replay.
use dioxus::prelude::*;
use dioxus_ix::*;

use super::page_recovery_review::gallery_recovery;

#[component]
pub fn ContentScrollPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class: "content-scroll-native-review",
            gallery_recovery::PageRecoveryExample {}
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_CONTENT_SCROLL_CHECK);
                        if let Err(error) = query.send("contentscroll") {
                            eprintln!("Content scroll native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native Content scroll: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native Content scroll failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Content scroll native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.content-scroll-native-review{box-sizing:border-box;min-height:100vh;padding:24px}@media(max-width:600px){.content-scroll-native-review{padding:16px}}";

// Development replay only. The actual gallery owns navigation, fields, native
// forms and mounted-root/input focus in Rust; this script is not a fallback.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_CONTENT_SCROLL_CHECK: &str = r#"
let snapshots={},layout={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById('page-recovery-'+id),root=()=>get('content'),body=()=>root()?.querySelector('.ix-content-body'),header=()=>root()?.querySelector('.ix-content-header-region'),status=()=>get('status'),active=()=>document.activeElement;
 const assert=(condition,label)=>{if(!condition)throw Error(label)},hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{for(let n=0;n<180;n++){if(predicate())return;await hold(20)}throw Error(label+'; status='+status()?.outerHTML+'; focus='+(active()?.id||active()?.tagName))};
 const rect=node=>{const r=node.getBoundingClientRect();return{left:r.left,top:r.top,right:r.right,bottom:r.bottom,width:r.width,height:r.height}};
 const geometry=()=>({root:rect(root()),body:{...rect(body()),scrollTop:body().scrollTop,scrollHeight:body().scrollHeight,clientHeight:body().clientHeight,overflow:getComputedStyle(body()).overflowY},header:rect(header()),save:rect(get('save'))});
 const capture=()=>({...status()?.dataset,title:get('title')?.value,note:get('note')?.value,preparation:get('preparation')?.value,focus:active()?.id||active()?.tagName});
 const click=id=>{assert(get(id),id+' exists');get(id).click()};
 const input=async(id,value)=>{const node=get(id);node.focus();const proto=node instanceof HTMLTextAreaElement?HTMLTextAreaElement.prototype:HTMLInputElement.prototype;Object.getOwnPropertyDescriptor(proto,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait(id+' controlled value',()=>node.value===value)};
 const mode=async value=>{if(get('scrollable').checked!==value)click('scrollable');await wait('Content bounded scroll option '+value,()=>status().dataset.scrollable===String(value)&&root().classList.contains('ix-content-scrollable')===value)};
 const scroll=async()=>{body().scrollTop=body().scrollHeight;await wait('real native Content body scroll',()=>body().scrollTop>0)};
 const start=async()=>{click('form-start');await wait('Dioxus current body scrolls to form start and current title receives focus',()=>body().scrollTop<=1&&active()===get('title'))};
 const lookup=async value=>{await input('query',value);click('lookup');await wait('actual local lookup '+value,()=>status().dataset.route===value)};
 const payload=()=>Object.fromEntries(new FormData(get('editor-form')).entries());
 await wait('actual default page/Content body mounted',()=>root()&&body()&&header()&&get('scrollable')&&get('title'));
 const initial={root:root(),body:body(),header:header(),title:get('title'),note:get('note'),preparation:get('preparation')};
 assert(!root().classList.contains('ix-content-scrollable')&&getComputedStyle(body()).display==='contents','default Content retains ordinary flow');
 await input('title','Native bounded inspection');await input('note','Keep native inspection observations through scrolling');await input('preparation','Keep native scrolling preparation');await mode(true);await wait('bounded Content body has real overflow',()=>body().scrollHeight>body().clientHeight+4);
 assert(root()===initial.root&&body()===initial.body&&header()===initial.header&&get('title')===initial.title&&get('note')===initial.note,'mode option retains actual Content root/header/body and editor controls');assert(get('save').form===get('editor-form')&&header().contains(get('save')),'native header Save belongs actual body form');
 layout.before=geometry();await scroll();layout.scrolled=geometry();assert(Math.abs(layout.before.header.top-layout.scrolled.header.top)<=1&&Math.abs(layout.before.save.top-layout.scrolled.save.top)<=1&&layout.scrolled.body.overflow==='auto','native body scroll keeps header and Save stationary');snapshots.scrolled=capture();
 const submitted=payload();assert(submitted.equipment==='pump'&&submitted.title==='Native bounded inspection'&&submitted.note==='Keep native inspection observations through scrolling'&&!Object.values(submitted).includes('Keep native scrolling preparation'),'native successful controls contain exact editor fields and exclude preparation');click('save');await wait('associated header Submit saves actual current native form',()=>status().dataset.savedCount==='1');const saved=JSON.parse(status().dataset.payload);assert(Object.keys(submitted).every(key=>submitted[key]===saved[key]),'header FormEvent payload matches exact successful controls');await start();assert(get('title').value===submitted.title&&get('note').value===submitted.note&&get('preparation')===initial.preparation,'mounted Form start keeps drafts and independent preparation');layout.formStart=geometry();snapshots.savedAndFormStart=capture();
 await mode(false);assert(body()===initial.body&&get('title')===initial.title&&get('note')===initial.note&&getComputedStyle(body()).display==='contents','returning to normal flow retains body and editor controls');await mode(true);await scroll();click('disabled');await wait('paused editing disables native Save and Form start',()=>get('save').disabled&&get('form-start').disabled);assert(status().dataset.savedCount==='1'&&get('title').value===submitted.title,'paused editor retains saved snapshot and draft');click('disabled');await wait('editing restored',()=>!get('save').disabled);await start();snapshots.liveModeAndDisabled=capture();
 await lookup('missing-scroll');await wait('existing error InfoPage mounted/focused',()=>get('missing')&&active()===get('missing'));assert(!initial.body.isConnected&&!initial.title.isConnected&&get('preparation')===initial.preparation,'recovery retires old Content body while preparation stays');click('return');await wait('recovery restores retained bounded draft with fresh mounted title',()=>root()?.classList.contains('ix-content-scrollable')&&get('title')?.value===submitted.title&&active()===get('title'));const restoredBody=body();await scroll();await start();assert(body()===restoredBody&&get('note').value===submitted.note,'recovery uses actual current body handle');const generation=Number(get('owner').dataset.generation),beforeReset=body();click('reset');await wait('keyed Reset restores fresh default Content body',()=>Number(get('owner')?.dataset.generation)>generation&&body()&&body()!==beforeReset&&!root().classList.contains('ix-content-scrollable')&&get('title')?.value==='Inspect inlet seals');assert(!beforeReset.isConnected&&get('preparation').value==='Bring the inspection kit'&&status().dataset.savedCount==='0','Reset retires old body and restores owned defaults');await mode(true);await wait('fresh bounded body overflows',()=>body().scrollHeight>body().clientHeight+4);await scroll();await start();assert(document.documentElement.scrollWidth<=innerWidth+1,'native bounded header/body fit page without horizontal overflow');layout.fresh=geometry();snapshots.recoveryAndReset=capture();
 return{ok:true,snapshots,layout,checks:['default and bounded Content modes retain actual root/header/body and current editor fields; bounded body has real native overflow','native body scroll keeps header actions stationary; associated header Submit saves exact body FormData/FormEvent values','Dioxus current mounted body Form start restores scroll offset zero and actual title focus; drafts/preparation survive mode toggles and paused editing','existing error recovery and keyed Reset retire old body handles; fresh body scrolling and mounted Form start remain usable'],input:'Excluded synthetic native DOM input/clicks and direct body scroll assignments exercise real gallery handlers, native form defaults and Dioxus mounted-body scroll/focus. Trusted wheel/keyboard evidence is recorded separately in Chromium.',renderer:navigator.userAgent};
} catch(error){return{ok:false,error:String(error),snapshots,layout,status:document.getElementById('page-recovery-status')?.outerHTML,focus:document.activeElement?.id||document.activeElement?.tagName}}
"#;
