//! Excluded actual-gallery page recovery and bounded synthetic native replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/page_recovery.rs"]
mod gallery_recovery;

#[component]
pub fn PageRecoveryPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class: "page-recovery-native-review",
            gallery_recovery::PageRecoveryExample {}
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_PAGE_RECOVERY_CHECK);
                        if let Err(error) = query.send("pagerecovery") {
                            eprintln!("Page recovery native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native page recovery: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native page recovery failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Page recovery native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.page-recovery-native-review{box-sizing:border-box;min-height:100vh;padding:24px}@media(max-width:600px){.page-recovery-native-review{padding:16px}}";

// Development replay only. The actual gallery owns navigation, fields, native
// forms and mounted-root/input focus in Rust; this script is not a fallback.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_PAGE_RECOVERY_CHECK: &str = r#"
let snapshots={},roots={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById('page-recovery-'+id),status=()=>get('status'),active=()=>document.activeElement;
 const assert=(condition,label)=>{if(!condition)throw Error(label)},hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{for(let n=0;n<180;n++){if(predicate())return;await hold(20)}throw Error(label+'; status='+status()?.outerHTML+'; focus='+(active()?.id||active()?.tagName))};
 const count=()=>Number(status()?.dataset.savedCount),capture=()=>({generation:get('owner')?.dataset.generation,...status()?.dataset,title:get('title')?.value,note:get('note')?.value,preparation:get('preparation')?.value,focus:active()?.id||active()?.tagName});
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' actual native focus',()=>active()===node)};
 const click=id=>{assert(get(id),id+' exists');get(id).click()};
 const input=async(id,value)=>{const node=get(id);await focus(node,id);const proto=node instanceof HTMLTextAreaElement?HTMLTextAreaElement.prototype:HTMLInputElement.prototype;Object.getOwnPropertyDescriptor(proto,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait(id+' controlled native value',()=>node.value===value)};
 const lookup=async identifier=>{await input('query',identifier);const payload=Object.fromEntries(new FormData(get('lookup-form')).entries());assert(payload['equipment-query']===identifier,'native lookup form owns exact equipment query');click('lookup');await wait('lookup route '+identifier,()=>status()?.dataset.route===identifier);return payload};
 const checkRoot=(name,tag,label)=>{const node=get(name);assert(node?.tagName===tag&&node.getAttribute('tabindex')==='-1'&&node.getAttribute('aria-label')===label,'actual '+tag+' receives consumer root metadata and focus target');roots[name]={tag:node.tagName,id:node.id,tabindex:node.getAttribute('tabindex'),ariaLabel:node.getAttribute('aria-label'),classes:node.className}};
 const save=async(equipment,title,note,expected)=>{await input('title',title);await input('note',note);const payload=Object.fromEntries(new FormData(get('editor-form')).entries());assert(payload.equipment===equipment&&payload.title===title&&payload.note===note&&!Object.values(payload).includes('Native retained preparation'),'actual native successful controls contain exact equipment/title/note and exclude independent preparation');click('save');await wait('native FormEvent saves current '+equipment,()=>count()===expected);const actual=JSON.parse(status().dataset.payload);assert(JSON.stringify(actual)===JSON.stringify(payload)||Object.keys(payload).every(key=>actual[key]===payload[key]),'native FormEvent payload equals actual successful controls');const row=get('saved-records').lastElementChild;assert(row?.dataset.equipment===equipment&&row.dataset.title===title&&row.dataset.note===note,'saved record retains exact native form snapshot');return payload};
 await wait('actual known-equipment page mounted',()=>status()?.dataset.state==='ready'&&status().dataset.equipment==='pump'&&get('content')&&get('title')&&get('preparation'));
 await wait('native WebView focus available',()=>document.hasFocus());checkRoot('content','MAIN','Equipment editor');const originalContent=get('content'),preparation=get('preparation');click('review');await wait('actual mounted Content root review focus',()=>active()===originalContent);snapshots.initial=capture();
 await input('preparation','Native retained preparation');await input('title','Retained pump draft');await input('note','Pump draft before navigation');await lookup('fan');await wait('known fan has current editor input focus',()=>status().dataset.state==='ready'&&status().dataset.equipment==='fan'&&active()===get('title')&&get('title').value==='Check fan guard');assert(get('preparation')===preparation,'known route change retains independent preparation');const fanPayload=await save('fan','Native fan inspection','Saved fan guard observations',1);snapshots.knownLookupAndSave={...capture(),payload:fanPayload};
 await input('title','Retained unsaved fan title');await input('note','Unsaved fan draft through missing page');const oldFanTitle=get('title');const missingQuery=await lookup('missing-unit');await wait('missing local asset shows actual InfoPage and receives mounted root focus',()=>status().dataset.state==='missing'&&get('missing')&&active()===get('missing')&&!get('content')&&!get('title'));checkRoot('missing','SECTION','Equipment unavailable');assert(!oldFanTitle.isConnected&&get('preparation')===preparation&&preparation.value==='Native retained preparation'&&count()===1,'missing route retains prior saved record and outside preparation while editor is absent');const missingRoot=get('missing');snapshots.missing={...capture(),lookup:missingQuery};
 click('return');await wait('useful Return restores retained current fan draft and focus',()=>status().dataset.state==='ready'&&status().dataset.equipment==='fan'&&get('title')?.value==='Retained unsaved fan title'&&get('note')?.value==='Unsaved fan draft through missing page'&&active()===get('title'));assert(!missingRoot.isConnected&&get('title')!==oldFanTitle&&get('preparation')===preparation&&count()===1,'Return owns fresh current editor while retaining draft, preparation and immutable save');snapshots.returned=capture();
 await lookup('unavailable-pump');await wait('second missing root mounted and focused',()=>get('missing')&&active()===get('missing'));click('open-pump');await wait('Open known Pump restores earlier retained pump draft with current focus',()=>status().dataset.equipment==='pump'&&status().dataset.state==='ready'&&get('title')?.value==='Retained pump draft'&&get('note')?.value==='Pump draft before navigation'&&active()===get('title'));const pumpPayload=await save('pump','Native pump inspection','Saved inlet seal observations',2);snapshots.openKnownAndSave={...capture(),payload:pumpPayload};
 click('disabled');await wait('native editing paused',()=>get('title').disabled&&get('note').disabled&&get('save').disabled);click('save');assert(count()===2&&get('preparation')===preparation&&preparation.value==='Native retained preparation','disabled native Save preserves immutable records and independent preparation');click('disabled');await wait('native editing resumed',()=>!get('title').disabled&&!get('save').disabled);
 const generation=Number(get('owner').dataset.generation),beforeReset=get('title'),beforeRoot=get('content');click('reset');await wait('keyed owner Reset mounts fresh initial current root/input',()=>Number(get('owner')?.dataset.generation)>generation&&get('title')!==beforeReset&&get('title')?.value==='Inspect inlet seals'&&count()===0);assert(!beforeReset.isConnected&&!beforeRoot.isConnected&&!preparation.isConnected&&get('preparation').value==='Bring the inspection kit'&&status().dataset.equipment==='pump'&&get('note').value==='','Reset retires old root/input/preparation and restores initial local page');checkRoot('content','MAIN','Equipment editor');const fresh=get('title');await lookup('missing-after-reset');await wait('fresh missing current root owns native focus',()=>get('missing')&&active()===get('missing'));click('return');await wait('fresh Return uses current post-reset title handle',()=>get('title')&&get('title')!==fresh&&active()===get('title')&&get('title').value==='Inspect inlet seals');await save('pump','Fresh native recovered inspection','Current owner save',1);assert(document.documentElement.scrollWidth<=innerWidth+1,'native page and recovery actions fit viewport without horizontal overflow');snapshots.reset=capture();
 return{ok:true,snapshots,roots,checks:['actual Content main root receives native metadata; known local lookup replaces editor and focuses current mounted input','actual native FormData/FormEvent saves immutable equipment/title/note snapshots while independent preparation stays outside form','missing local asset mounts actual InfoPage section with metadata and native root focus','useful Return/open-known actions restore retained per-asset drafts and current editor focus without losing saves or preparation','disabled native Save is guarded; keyed Reset retires old roots/fields and fresh recovery/focus/native Save remain usable'],input:'Excluded synthetic native focus, input events, explicit DOM clicks and actual native form defaults. Trusted Enter/Space and physical input are established separately in Chromium, not by these synthesized events.',renderer:navigator.userAgent};
} catch(error){return{ok:false,error:String(error),snapshots,roots,status:document.getElementById('page-recovery-status')?.outerHTML,focus:document.activeElement?.id||document.activeElement?.tagName}}

"#;
