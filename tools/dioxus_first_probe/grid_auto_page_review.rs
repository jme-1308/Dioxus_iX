//! Excluded actual-gallery automatic page-capacity and retained-edit workflow.
use super::grid_worklist_review::gallery_worklist;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn GridAutoPagePreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_worklist::STYLES} }
        IxTheme { class:"grid-auto-page-review",
            gallery_worklist::GridWorklistExample {}
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=crate::dom_query::eval(NATIVE_GRID_AUTO_PAGE_CHECK);let _=query.send("gridautopage");
                    match query.join::<serde_json::Value>().await {
                        Ok(value)if value["ok"]==true=>{println!("Dioxus native Grid auto page: {value}");window.close();}
                        value=>{eprintln!("Dioxus native Grid auto page failed: {value:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(40)).await;eprintln!("Grid auto page native replay timed out");std::process::exit(1);});}
            } }
        }
    }
}
const STYLES: &str = "body{margin:0}.grid-auto-page-review{box-sizing:border-box;min-height:100vh;padding:24px}@media(max-width:600px){.grid-auto-page-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_GRID_AUTO_PAGE_CHECK: &str = r#"
let snapshots={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById('worklist-'+id),host=()=>document.getElementById('grid-worklist'),pg=()=>get('pagination-status'),state=()=>get('status'),grid=()=>host()?.querySelector('[role="grid"][aria-label="Maintenance worklist"]'),pager=()=>host()?.querySelector('nav[aria-label="Maintenance worklist pages"]'),control=label=>pager()?.querySelector('[aria-label="'+label+'"]'),row=id=>grid()?.querySelector('[data-row-id="'+id+'"]'),cell=id=>row(id)?.querySelector('[role="gridcell"][data-column-id="asset"]'),editor=()=>host()?.querySelector('.ix-data-grid-editor:not([readonly])');
 const assert=(v,label)=>{if(!v)throw Error(label)},wait=async(label,predicate)=>{for(let i=0;i<200;i++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+'; status='+pg()?.outerHTML)};
 const click=id=>{assert(get(id),id+' exists');get(id).click()},key=(n,k)=>n.dispatchEvent(new KeyboardEvent('keydown',{key:k,code:k,bubbles:true,cancelable:true}));
 const setInput=async(n,value)=>{Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(n,String(value));n.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))};
 const geom=()=>{const g=grid(),h=Number.parseFloat(getComputedStyle(g).getPropertyValue('--ix-grid-row-height'));return{height:g.clientHeight,rect:g.getBoundingClientRect().height,rowHeight:h,capacity:Math.max(1,Math.floor((g.clientHeight-48)/Math.max(24,h))),body:g.querySelector('.ix-data-grid-body').getBoundingClientRect().height,scrollHeight:g.scrollHeight}};
 const fitting=async label=>{await wait(label,()=>Number(pg().dataset.pageSize)===geom().capacity);const v=geom();assert(v.body<=v.height-48+1,'all accepted rows fit actual scroll content height');return{page:{...pg().dataset},geometry:v}};
 const height=async value=>{get('grid-height').value=String(value);get('grid-height').dispatchEvent(new Event('change',{bubbles:true}));await wait('actual root height '+value,()=>pg().dataset.gridHeight===String(value)&&Math.abs(grid().parentElement.getBoundingClientRect().height-Number(value))<3)};
 await wait('actual worklist mounted',()=>pg()&&row('worklist-000')&&get('auto-page-size'));assert(pg().dataset.enabled==='false'&&!pager()&&!get('auto-page-size').checked&&get('auto-page-size').disabled,'default remains unpaged and automatic option disabled');click('pagination');await wait('manual12 unchanged',()=>pg().dataset.pageSize==='12'&&control('Items per page'));assert(!get('auto-page-size').checked&&control('Items per page').value==='12','manual selector retained');row('worklist-001').querySelector('[role="checkbox"]').click();await wait('selected actual row',()=>state().dataset.selected==='1');snapshots.manual={...pg().dataset};
 click('auto-page-size');snapshots.automatic=await fitting('actual measured automatic fitting size');assert(!control('Items per page')&&host().querySelector('.ix-data-grid-page-capacity')?.textContent.includes(pg().dataset.pageSize),'automatic capacity readout replaces manual chooser');control('Next page').click();await wait('automatic Next works',()=>pg().dataset.page==='1');const first=Number(pg().dataset.start);await height(560);snapshots.tall=await fitting('tall actual capacity accepted');assert(Number(pg().dataset.start)<=first&&first<Number(pg().dataset.end),'resize keeps previous first logical row within new page');assert(state().dataset.selected==='1','resize retains selection');click('csv-generate');await wait('CSV full logical96 despite automatic page',()=>get('csv-status').dataset.rows==='96');snapshots.tall.oldFirst=first;
 const targetId='worklist-'+String(pg().dataset.start).padStart(3,'0');cell(targetId).focus();key(cell(targetId),'F2');await wait('actual editor active',()=>editor()&&host().querySelector('fieldset.ix-data-grid-pagination').disabled);await setInput(editor(),'Auto-sized inspection');const accepted=pg().dataset.pageSize;await height(300);assert(editor()?.value==='Auto-sized inspection'&&pg().dataset.pageSize===accepted&&Number(accepted)!==geom().capacity,'actual height change defers size and retains private editor draft');snapshots.deferred={page:{...pg().dataset},geometry:geom(),draft:editor().value};key(editor(),'Enter');await wait('Enter commits actual edited row',()=>!editor()&&state().dataset.saved==='1');snapshots.committed=await fitting('commit accepts latest compact capacity');assert(cell(targetId)?.textContent.includes('Auto-sized inspection'),'committed current record retained after automatic resize');
 cell(targetId).focus();key(cell(targetId),'F2');await wait('fresh cancel editor',()=>editor());await setInput(editor(),'Discarded auto-page draft');const compact=pg().dataset.pageSize;await height(560);assert(editor()?.value==='Discarded auto-page draft'&&pg().dataset.pageSize===compact,'second current editor preserves draft across resize');key(editor(),'Escape');await wait('Escape cancels editor',()=>!editor());snapshots.cancelled=await fitting('cancel accepts latest tall capacity');assert(cell(targetId)?.textContent.includes('Auto-sized inspection')&&state().dataset.saved==='1','Escape preserves committed record without extra save');
 const acceptedAuto=pg().dataset.pageSize;click('auto-page-size');await wait('manual chooser returns',()=>control('Items per page')&&!host().querySelector('.ix-data-grid-page-capacity'));assert(pg().dataset.pageSize===acceptedAuto&&control('Items per page').value===acceptedAuto,'auto-off retains authoritative accepted size');click('restore');await wait('Restore original defaults',()=>pg().dataset.enabled==='false'&&!pager()&&state().dataset.selected==='0'&&state().dataset.saved==='0');assert(!get('auto-page-size').checked&&get('auto-page-size').disabled&&pg().dataset.gridHeight==='300'&&get('csv-status').dataset.state==='empty','Restore clears automatic preference/model/CSV');snapshots.restored={...pg().dataset};
 return{ok:true,snapshots,checks:['manual default and12-row pagination unchanged; actual row selection retained','auto capacity derives actual measured viewport; chooser replaced/readout and Next works','height resize preserves old first logical record; rows fit actual viewport; selected IDs and CSV96 remain logical','active edit retains draft/accepted capacity during actual height change; Enter commits then accepts latest measurement','Escape preserves committed value and accepts deferred latest measurement','auto-off restores manual chooser with accepted size; Restore returns unpaged model/selection/CSV defaults'],input:'Excluded synthetic native DOM input/change/click/key dispatch invokes actual shared gallery controls. No injected Rust state/controllers/callbacks. Trusted input is separate Chromium evidence.',renderer:navigator.userAgent};
} catch(error){return{ok:false,error:String(error),snapshots,page:document.getElementById('worklist-pagination-status')?.outerHTML,state:document.getElementById('worklist-status')?.outerHTML}}
"#;
