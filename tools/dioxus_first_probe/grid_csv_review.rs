//! Excluded actual-gallery worklist CSV and native download-transport replay.
use super::grid_worklist_review::gallery_worklist;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn GridCsvPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_worklist::STYLES} }
        IxTheme { class:"grid-csv-review",
            gallery_worklist::GridWorklistExample {}
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=crate::dom_query::eval(NATIVE_GRID_CSV_CHECK);let _=query.send("gridcsv");
                    match query.join::<serde_json::Value>().await {
                        Ok(value)if value["ok"]==true=>{println!("Dioxus native Grid CSV: {value}");window.close();}
                        value=>{eprintln!("Dioxus native Grid CSV failed: {value:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(35)).await;eprintln!("Grid CSV native replay timed out");std::process::exit(1);});}
            } }
        }
    }
}
const STYLES: &str = "body{margin:0}.grid-csv-review{box-sizing:border-box;min-height:100vh;padding:24px}@media(max-width:600px){.grid-csv-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_GRID_CSV_CHECK: &str = r#"
let snapshots={},downloads=[];
const originalClick=HTMLAnchorElement.prototype.click;
try {
 await dioxus.recv();
 // Spy only on the final native anchor activation; this verifies dispatch bytes
 // without claiming an OS save dialog or replacing application download logic.
 HTMLAnchorElement.prototype.click=function(){if(this.href.startsWith('data:text/csv;')){const raw=atob(this.href.split(',')[1]),bytes=Uint8Array.from(raw,c=>c.charCodeAt(0));downloads.push({filename:this.download,prefix:Array.from(bytes.slice(0,3)),text:new TextDecoder().decode(bytes),bytes:bytes.length});return;}return originalClick.call(this)};
 const get=id=>document.getElementById('worklist-'+id),csv=()=>get('csv-status'),status=()=>get('status'),preview=()=>get('csv-preview'),host=()=>document.getElementById('grid-worklist'),grid=()=>host()?.querySelector('[role="grid"][aria-label="Maintenance worklist"]');
 const assert=(condition,label)=>{if(!condition)throw Error(label)},wait=async(label,predicate)=>{for(let i=0;i<180;i++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+'; csv='+csv()?.outerHTML+'; grid='+status()?.outerHTML)};
 const rows=()=>JSON.parse(csv().dataset.rowIds),columns=()=>JSON.parse(csv().dataset.columnIds),row=id=>grid()?.querySelector('[data-row-id="'+id+'"]'),cell=(id,column)=>row(id)?.querySelector('[role="gridcell"][data-column-id="'+column+'"]');
 const click=id=>{assert(get(id),id+' exists');get(id).click()},key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,code:value,bubbles:true,cancelable:true}));
 const input=async(node,value)=>{Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait('actual controlled value',()=>node.value===value)};
 const generate=async(scope,count)=>{get('csv-scope').value=scope;get('csv-scope').dispatchEvent(new Event('change',{bubbles:true}));await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));click('csv-generate');await wait('actual '+scope+' CSV snapshot',()=>csv().dataset.state==='ready'&&csv().dataset.scope===scope&&Number(csv().dataset.rows)===count);return preview().value};
 const download=async()=>{const expected=preview().value,n=downloads.length;click('csv-download');await wait('actual CSV native dispatch',()=>downloads.length===n+1&&csv().dataset.state==='dispatched');const actual=downloads.at(-1);assert(actual.filename==='maintenance-worklist.csv'&&actual.prefix.join(',')==='239,187,191'&&actual.text.replaceAll('\r\n','\n')===expected.replaceAll('\r\n','\n'),'shared native dispatch includes exact UTF8 BOM/content and CSV filename');return actual};
 await wait('actual worklist CSV controls mounted',()=>csv()&&get('csv-scope')&&get('csv-generate')&&status()?.dataset.total==='96');assert(get('csv-download').disabled&&csv().dataset.state==='empty','no implicit CSV snapshot/download before Generate');const initial=await generate('displayed',96);assert(initial.startsWith('Asset,Status,Owner,Ticket\n')&&initial.includes('Asset 000,Normal,Team 1,IX-00000\n')&&columns().join(',')==='asset,status,owner,ticket'&&rows()[0]==='worklist-000','initial actual export uses logical columns/all96records beyond mounted window');await download();snapshots.initial={rows:rows(),columns:columns(),content:initial};
 row('worklist-001').querySelector('[role="checkbox"]').click();await wait('actual selected warning row',()=>status().dataset.selected==='1');const target=cell('worklist-000','asset');target.focus();key(target,'F2');await wait('actual grid native editor',()=>host().querySelector('.ix-data-grid-editor'));const editor=host().querySelector('.ix-data-grid-editor');await input(editor,'=SUM("Å",1)');key(editor,'Enter');await wait('actual committed formula-like quoted asset',()=>status().dataset.saved==='1'&&cell('worklist-000','asset')?.textContent.includes('=SUM("Å",1)'));assert(preview().value===initial,'committed edit leaves captured CSV snapshot immutable');await download();
 const trigger=grid().querySelector('button[aria-label="Filter Status"]');trigger.click();await wait('actual Status native filter',()=>document.getElementById(trigger.getAttribute('popovertarget'))?.matches(':popover-open'));const panel=document.getElementById(trigger.getAttribute('popovertarget'));await input(panel.querySelector('input[aria-label="Status filter value 1"]'),'Normal');await wait('actual32Normal rows retain hidden selected Warning',()=>status().dataset.visible==='32'&&status().dataset.selected==='1');trigger.click();await generate('selected-displayed',0);assert(preview().value==='Asset,Status,Owner,Ticket\n'&&rows().length===0,'selected displayed scope excludes hidden selected record');await generate('selected-all',1);assert(rows().join(',')==='worklist-001'&&preview().value.includes('Asset 001,Warning'),'all selected scope includes hidden selected Warning record');await download();snapshots.hiddenSelection={rows:rows(),content:preview().value};
 const sort=grid().querySelector('button[aria-label="Sort Asset"]');sort.click();await wait('Asset ascending',()=>status().dataset.sort==='asset-ascending');sort.click();await wait('Asset descending',()=>status().dataset.sort==='asset-descending');const current=await generate('displayed',32);assert(rows()[0]==='worklist-093'&&rows().at(-1)==='worklist-000','displayed CSV follows actual filtered descending order');assert(current.includes('"\'=SUM(""Å"",1)"'),'default formula escaping and CSV comma/quote escaping use actual committed edit');await download();snapshots.current={rows:rows(),content:current};
 click('restore');await wait('actual worklist reset clears CSV and gridstate',()=>csv().dataset.state==='empty'&&get('csv-download').disabled&&status().dataset.saved==='0'&&status().dataset.selected==='0'&&status().dataset.visible==='96'&&get('csv-scope').value==='displayed');const fresh=await generate('displayed',96);assert(fresh===initial,'fresh reset snapshot restores actual initial data');await download();snapshots.reset={rows:rows(),content:fresh};
 return{ok:true,snapshots,downloads,checks:['actual snapshot exports all logical96rows/currentcolumns beyond mounted virtual row window; shared native CSV dispatch has exact BOM/filename/bytes','committed edit cannot mutate previous snapshot; selected displayed vs selected all handle hidden selection honestly','actual sorted/filtered current export includes committed formula/comma/quote escaping in current row order','Restore clears snapshot/download ownership and fresh actual snapshot dispatch remains usable'],input:'Excluded synthetic native key/input/click dispatch exercises actual gallery Grid/CSV state. Final native anchor activation is spied and suppressed: bytes/filename/dispatch are verified; OS save completion is not claimed.',renderer:navigator.userAgent};
} catch(error){return{ok:false,error:String(error),snapshots,downloads,csv:document.getElementById('worklist-csv-status')?.outerHTML,grid:document.getElementById('worklist-status')?.outerHTML}}
finally{HTMLAnchorElement.prototype.click=originalClick}
"#;
