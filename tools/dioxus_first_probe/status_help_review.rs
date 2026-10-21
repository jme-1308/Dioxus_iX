//! Excluded actual-gallery styled status help and ordinary form/Chip workflows.
use super::chip_workflows_review::gallery_chips;
use super::service_request_review::gallery_request;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn StatusHelpPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_request::STYLES} }
        style { {gallery_chips::STYLES} }
        IxTheme { class:"status-help-review",
            gallery_request::ServiceRequestExample {}
            div { id:"status-help-default", Pill {variant:StatusVariant::Neutral,outline:true,icon:"info","Draft"} }
            gallery_chips::ChipWorkflowsExample {}
            div {onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=crate::dom_query::eval(NATIVE_STATUS_HELP_CHECK);let _=query.send("statushelp");
                    match query.join::<serde_json::Value>().await {
                        Ok(value)if value["ok"]==true=>{println!("Dioxus native Status help: {value}");window.close();}
                        value=>{eprintln!("Dioxus native Status help failed: {value:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(40)).await;eprintln!("Status help native replay timed out");std::process::exit(1);});}
            }}
        }
    }
}
const STYLES: &str = "body{margin:0}.status-help-review{box-sizing:border-box;min-height:100vh;padding:24px}@media(max-width:600px){.status-help-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_STATUS_HELP_CHECK: &str = r#"
let snapshots={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),pill=()=>get('service-status-pill'),result=()=>get('service-result'),main=id=>get(id)?.querySelector('.ix-chip-main'),close=id=>get(id)?.querySelector('.ix-chip-close'),chips=()=>get('chip-workflow-status');
 const assert=(v,label)=>{if(!v)throw Error(label)},wait=async(label,predicate)=>{for(let i=0;i<180;i++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+'; result='+result()?.outerHTML+'; chips='+chips()?.outerHTML)};
 const key=(n,k)=>n.dispatchEvent(new KeyboardEvent('keydown',{key:k,code:k,bubbles:true,cancelable:true}));
 const setValue=async(n,value)=>{const proto=n instanceof HTMLTextAreaElement?HTMLTextAreaElement.prototype:HTMLInputElement.prototype;Object.getOwnPropertyDescriptor(proto,'value').set.call(n,value);n.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait('controlled field value',()=>n.value===value)};
 const panel=n=>(n?.getAttribute('aria-describedby')||'').split(/\s+/).map(get).find(p=>p?.getAttribute('role')==='tooltip');
 const help=async(n,text)=>{n.scrollIntoView({block:'center'});n.focus();await wait('actual focus and styled help '+text,()=>document.activeElement===n&&panel(n)?.dataset.open==='true'&&panel(n)?.dataset.positioned==='true'&&panel(n).textContent.includes(text));assert(!n.hasAttribute('title'),'no browser title fallback');return{id:panel(n).id,text:panel(n).textContent.trim(),anchor:n.id,placement:panel(n).dataset.placement}};
 await wait('actual composed service and assets',()=>pill()&&main('asset-a')&&main('asset-b')&&main('asset-c'));
 const plain=get('status-help-default').querySelector('.ix-pill'),a=pill().getBoundingClientRect(),b=plain.getBoundingClientRect();assert(pill().tabIndex===0&&!plain.hasAttribute('tabindex')&&!plain.hasAttribute('aria-describedby')&&!plain.hasAttribute('title')&&Math.abs(a.height-b.height)<=1&&Math.abs(a.width-b.width)<=1&&Math.abs(a.height-20)<=1,'help leaves default Pill geometry and no-help tab semantics unchanged');snapshots.draft=await help(pill(),'Complete the fields to preview a request. Nothing is sent.');const draftPanel=panel(pill());key(pill(),'Escape');await wait('Escape dismisses styled Pill help',()=>draftPanel.dataset.open==='false');
 get('service-form').requestSubmit(get('service-submit'));await wait('actual host invalid validation',()=>result().dataset.status==='invalid'&&document.activeElement===get('service-asset'));snapshots.invalid=await help(pill(),'Check the highlighted fields and try again.');await setValue(get('service-asset'),'MTR-42');await setValue(get('service-notes'),'Inspect bearings before restart.');get('service-form').requestSubmit(get('service-submit'));await wait('real native form accepted preview',()=>result().dataset.status==='ready'&&result().dataset.submissions==='1'&&get('service-preview'));const payload=Object.fromEntries(new FormData(get('service-form')).entries());assert(payload.asset==='MTR-42'&&payload.priority==='normal'&&payload.technicians==='2'&&payload.notes==='Inspect bearings before restart.'&&get('service-preview').dataset.asset===payload.asset&&get('service-preview').dataset.notes===payload.notes,'native FormData and actual accepted preview preserved');snapshots.ready={help:await help(pill(),'Review the request details below. Nothing has been sent.'),payload};await setValue(get('service-notes'),'Changed preparation');await wait('edit retires preview and returns Draft',()=>result().dataset.status==='draft'&&!get('service-preview'));await wait('currently open help reacts to Draft text',()=>panel(pill()).textContent.includes('Complete the fields'));
 snapshots.motor=await help(main('asset-a'),'Motor A is ready for the next service window.');assert(main('asset-a').tagName==='BUTTON'&&!get('asset-a').querySelector('.ix-chip').hasAttribute('tabindex'),'help adds no chip-wrapper tab stop');assert(panel(close('asset-a'))===panel(main('asset-a')),'actual main and close buttons own same live description');main('asset-a').click();await wait('ordinary inspection callback',()=>chips().dataset.inspections==='1');get('chip-keep-closed').click();await wait('current live help reactive host suffix',()=>panel(main('asset-a')).textContent.includes('The host currently keeps this pin after a remove request.'));close('asset-a').click();await wait('independent retained pin close callback',()=>chips().dataset.removals==='1'&&chips().dataset.count==='3');assert(chips().dataset.inspections==='1','close does not invoke inspect');
 assert(main('asset-b').disabled,'inactive main remains native-disabled');main('asset-b').click();assert(chips().dataset.inspections==='1','inactive cannot invoke inspect');snapshots.inactive=await help(close('asset-b'),'Compressor B is unavailable for inspection. Its pin can still be removed.');get('chip-keep-closed').click();const retired=panel(close('asset-b')).id;close('asset-b').click();await wait('inactive pin independent removal retires owned help',()=>!get('asset-b')&&!get(retired)&&chips().dataset.count==='2'&&chips().dataset.removals==='2');assert(chips().dataset.inspections==='1','inactive close does not invoke main');
 snapshots.long=await help(main('asset-c'),'Commissioning and safety review notes for Compressor B.');const longPanel=panel(main('asset-c'));key(main('asset-c'),'Escape');await wait('Escape closes current Chip help',()=>longPanel.dataset.open==='false');get('chip-restore').click();await wait('actual host Restore remounts missing pin',()=>chips().dataset.count==='3'&&main('asset-b'));snapshots.restored=await help(close('asset-b'),'Remove only the pin; its asset details are unchanged.');
 return{ok:true,snapshots,checks:['default no-help Pill compact geometry/tab semantics retained; focusable help Pill has styled accessible description and Escape dismissal','actual host invalid validation then native named FormData/accepted Ready preview preserved with current reactive Draft/Invalid/Ready help','Chip help belongs to real native main and close buttons without added wrapper tab stop; ordinary inspection works','host keep-pin state updates current help; close remains independent of inspection','inactive native main blocks inspection while close removes pin and retires owned help without orphan','long help uses current Chip anchor/Escape and restored inactive pin gets fresh working description'],input:'Excluded synthetic DOM input/click/keydown and explicit focus invoke actual shared gallery components. Real requestSubmit/FormData is exercised. Native trusted typing/button default activation is separate Chromium evidence; no application state/controller/callback injection.',renderer:navigator.userAgent};
}catch(error){return{ok:false,error:String(error),snapshots,result:document.getElementById('service-result')?.outerHTML,chips:document.getElementById('chip-workflow-status')?.outerHTML}}
"#;
