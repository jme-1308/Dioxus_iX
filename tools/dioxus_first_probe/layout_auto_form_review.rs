//! Excluded actual-gallery LayoutAuto form and bounded native mounted workflow.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/layout_auto_form.rs"]
mod gallery_form;

#[component]
pub fn LayoutAutoFormPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class:"layout-auto-form-native-review",
            gallery_form::LayoutAutoFormExample {}
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=crate::dom_query::eval(NATIVE_LAYOUT_AUTO_FORM_CHECK);let _=query.send("layoutautoform");
                    match query.join::<serde_json::Value>().await {
                        Ok(value)if value["ok"]==true=>{println!("Dioxus native layout auto form: {value}");window.close();}
                        value=>{eprintln!("Dioxus native layout auto form failed: {value:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(60)).await;eprintln!("Layout auto form native replay timed out");std::process::exit(1);});}
            } }
        }
    }
}
const STYLES: &str = "body{margin:0}.layout-auto-form-native-review{box-sizing:border-box;min-height:100vh;padding:24px}@media(max-width:600px){.layout-auto-form-native-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_LAYOUT_AUTO_FORM_CHECK: &str = r#"
let snapshots={},geometry={};
try{
 await dioxus.recv();
 const get=id=>document.getElementById('layout-auto-'+id),root=()=>get('inspection')||get('inspection-alt'),owner=()=>get('owner'),result=()=>get('result'),form=()=>get('inspection-form'),fields=()=>['equipment','title','date','note'].map(get),payload=()=>JSON.parse(result().dataset.payload),count=()=>Number(result().dataset.count);
 const assert=(v,label)=>{if(!v)throw Error(label)},wait=async(label,predicate)=>{let stable=0;for(let i=0;i<220;i++){stable=predicate()?stable+1:0;if(stable>=2)return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+'; result='+result()?.outerHTML+'; focus='+document.activeElement?.id)};
 const focus=async node=>{assert(node,'current native focus target exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait('native actual focus',()=>document.activeElement===node)};
 const input=async(id,value)=>{const node=get(id);await focus(node);const proto=node instanceof HTMLTextAreaElement?HTMLTextAreaElement.prototype:HTMLInputElement.prototype;Object.getOwnPropertyDescriptor(proto,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait('native controlled '+id,()=>node.value===value)};
 const check=async(id,value)=>{const n=get(id);if(n.checked!==value)n.click();await wait('native preference '+id,()=>get(id).checked===value)};
 const rect=n=>{const r=n.getBoundingClientRect();return{x:r.x,y:r.y,width:r.width,height:r.height,right:r.right,bottom:r.bottom}};
 const measure=async()=>{get('measure').click();await wait('current mounted root measured',()=>get('measure-status').dataset.state==='measured'&&Number(get('measure-status').dataset.width)>0&&Math.abs(Number(get('measure-status').dataset.width)-root().getBoundingClientRect().width)<2);return{reported:{...get('measure-status').dataset},actual:rect(root()),root:root().id}};
 const capture=()=>({generation:owner().dataset.generation,root:root().id,scope:root().dataset.ixLayoutScope,columns:getComputedStyle(root()).getPropertyValue('--ix-layout-columns').trim(),draft:Object.fromEntries(new FormData(form())),count:count(),payload:result().dataset.payload,preparation:get('preparation').value});
 await wait('actual LayoutAuto form ready',()=>root()&&owner()&&form()&&result()&&get('note')&&get('save')&&get('measure')&&get('show-notes'));const original={root:root(),items:fields().map(n=>n.closest('.ix-layout-auto-item')),fields:fields(),preparation:get('preparation'),scope:root().dataset.ixLayoutScope};
 await input('preparation','Retained independent preparation');get('equipment').value='fan';get('equipment').dispatchEvent(new Event('change',{bubbles:true}));await input('title','Inspect fan coupling');await input('date','2026-11-05');await input('note','Record exact local inspection notes');const exact={equipment:'fan',title:'Inspect fan coupling',date:'2026-11-05',note:'Record exact local inspection notes'};assert(JSON.stringify(Object.fromEntries(new FormData(form())))===JSON.stringify(exact),'actual native successful form controls contain exact current fields only');get('save').click();await wait('real associated native Save commits exact snapshot',()=>count()===1&&JSON.stringify(payload())===JSON.stringify(exact));for(const [name,value]of Object.entries(exact))assert(get('records').querySelector('li').getAttribute('data-'+name)===value,'saved visible '+name);snapshots.saved=capture();
 await input('title','Retained uncommitted follow-up');await input('note','Retained draft across live composition');const immutable=result().dataset.payload;await check('custom',true);await check('wide',false);await check('root-id',true);await wait('actual root ID/template/span change accepted',()=>root().id==='layout-auto-inspection-alt'&&root()===original.root&&root().dataset.ixLayoutScope===original.scope&&get('note').closest('.ix-layout-auto-item').dataset.colspan==='1');assert(fields().every((n,i)=>n===original.fields[i])&&original.items.every(n=>n.isConnected),'live props retain current fields/items');assert(get('title').value==='Retained uncommitted follow-up'&&get('note').value==='Retained draft across live composition'&&result().dataset.payload===immutable,'live layout preserves draft and immutable saved record');snapshots.retained=capture();
 geometry.measured=await measure();get('show-notes').click();await wait('actual mounted item scroll and current note focus',()=>document.activeElement===get('note'));const oldNote=get('note');await check('notes-visible',false);await wait('optional notes removed and current action guarded',()=>!get('note')&&get('show-notes').disabled);assert(!oldNote.isConnected&&root()===original.root&&get('title')===original.fields[1],'optional removal preserves root and remaining child identity');await check('notes-visible',true);await wait('optional notes remount from retained Rust draft',()=>get('note')&&get('note')!==oldNote&&get('note').value==='Retained draft across live composition'&&!get('show-notes').disabled);get('show-notes').click();await wait('current remounted note handle focuses',()=>document.activeElement===get('note'));assert(result().dataset.payload===immutable,'optional content leaves saved snapshot unchanged');snapshots.optional=capture();
 const prior={root:root(),title:get('title'),note:get('note'),generation:owner().dataset.generation};get('reset').click();await wait('owned draft Reset remounts fresh current handles',()=>owner().dataset.generation!==prior.generation&&root()!==prior.root&&get('title').value==='Inspect drive coupling'&&get('date').value==='2026-11-03'&&get('equipment').value==='pump'&&get('note').value==='');assert(!prior.root.isConnected&&!prior.title.isConnected&&!prior.note.isConnected,'old owned draft nodes retired');assert(get('custom').checked&&!get('wide').checked&&get('root-id').checked&&get('notes-visible').checked&&root().id==='layout-auto-inspection-alt','Reset preserves independent layout preferences');assert(get('preparation')===original.preparation&&get('preparation').value==='Retained independent preparation'&&count()===1&&result().dataset.payload===immutable,'Reset preserves preparation and immutable records');geometry.fresh=await measure();get('show-notes').click();await wait('fresh current note handle works after Reset',()=>document.activeElement===get('note'));geometry.layout={root:rect(root()),items:fields().map(n=>rect(n.closest('.ix-layout-auto-item'))),viewport:{width:innerWidth,height:innerHeight},pageWidth:document.documentElement.scrollWidth};assert(geometry.layout.root.width>0&&geometry.layout.items.every(r=>r.width>0&&r.x>=geometry.layout.root.x-1&&r.right<=geometry.layout.root.right+1)&&geometry.layout.pageWidth<=innerWidth+1,'ordinary native form items stay within actual root/viewport');snapshots.reset=capture();
 return{ok:true,snapshots,geometry,checks:['actual native associated form Save commits exact immutable equipment/title/date/note snapshot without independent preparation','live template/span/public root ID changes preserve the same private scope, mounted root/items/fields and uncommitted drafts','current mounted root measurement and item scroll/note focus survive optional content removal/recovery with retained Rust draft','owned draft Reset retires old handles and restores defaults while preferences/preparation/records remain; fresh measurement/scroll/current focus and native layout work'],input:'Excluded synthetic DOM input/change/click and explicit native focus exercise actual unchanged gallery form and Dioxus mounted methods. Read-only FormData, node identities, native geometry/current IDs and saved values only; no application/model/controller/callback/CSS injection or copied model. Optional notes deliberately unmount/remount; other retained children keep identity. Native viewport reported as observed, not resized or claimed narrow. Trusted browser Tab/typing/Save at1280/360 is separate evidence; no physical OS/AT or exact chronology claim.',renderer:navigator.userAgent};
}catch(error){return{ok:false,error:String(error),snapshots,geometry}}
"#;
