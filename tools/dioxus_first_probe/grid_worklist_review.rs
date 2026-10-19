//! Excluded actual-gallery maintenance worklist and bounded Linux DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/grid_worklist.rs"]
mod gallery_worklist;

#[component]
pub fn GridWorklistPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_worklist::STYLES} }
        IxTheme { class: "grid-worklist-review",
            h1 { "Maintenance worklist" }
            gallery_worklist::GridWorklistExample {}
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_GRID_WORKLIST_CHECK);
                        if let Err(error) = query.send("gridworklist") {
                            eprintln!("Worklist native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native maintenance worklist: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native maintenance worklist failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Worklist native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.grid-worklist-review{box-sizing:border-box;min-height:100vh;padding:24px}.grid-worklist-review h1{font-size:28px;margin:0 0 24px}.grid-worklist-review h2{font-size:22px;margin:24px 0 16px}.grid-worklist-review h3{font-size:18px}.grid-worklist-review p{max-width:72ch;line-height:1.5}.grid-worklist-review .demo-section{margin-bottom:32px;padding-bottom:24px;border-bottom:1px solid var(--ix-color-border)}@media(max-width:600px){.grid-worklist-review{padding:16px}}";

// Driver-only JS synthesizes click/form/keydown events and requests explicit
// native focus. It observes the actual controlled gallery and real native
// popovertarget behavior; it does not implement Grid state or emulate trusted
// typing/button defaults. Chromium separately verifies ordinary keyboard input.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_GRID_WORKLIST_CHECK: &str = r#"
let layout={},snapshots={},inputFocusFacts={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const host=()=>get('grid-worklist'),status=()=>get('worklist-status');
 const grid=()=>host()?.querySelector('[role="grid"][aria-label="Maintenance worklist"]');
 const outer=()=>grid()?.closest('.ix-data-grid');
 const rows=()=>[...grid()?.querySelectorAll('.ix-data-grid-body [data-row-id]')??[]];
 const row=id=>rows().find(node=>node.dataset.rowId===id);
 const cell=(id,column)=>row(id)?.querySelector('[role="gridcell"][data-column-id="'+column+'"]');
 const rowCheck=id=>row(id)?.querySelector('[role="checkbox"]');
 const headerCheck=()=>grid()?.querySelector('[role="checkbox"][aria-label="Select all filtered rows"]');
 const sortButton=()=>grid()?.querySelector('button[aria-label="Sort Asset"]');
 const assetHeader=()=>grid()?.querySelector('[role="columnheader"][data-column-id="asset"]');
 const filterTrigger=()=>grid()?.querySelector('button[aria-label="Filter Status"]');
 const dialog=()=>get(filterTrigger()?.getAttribute('popovertarget'));
 const filterInput=()=>dialog()?.querySelector('input[aria-label="Status filter value 1"]');
 const editor=()=>host()?.querySelector('.ix-data-grid-editor:not([readonly])');
 const alert=()=>host()?.querySelector('.ix-data-grid-edit-error[role="alert"]');
 const data=name=>status()?.dataset[name],count=name=>Number(data(name));
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const wait=async(label,predicate)=>{
  let stable=0;
  for(let attempt=0;attempt<150;attempt++){
   stable=predicate()?stable+1:0;
   if(stable===3)return;
   await hold(20);
  }
  throw Error(label+'; status='+status()?.textContent+'; focus='+(active()?.getAttribute('aria-label')||active()?.dataset?.columnId||active()?.id||active()?.tagName));
 };
 const key=(node,value)=>{
  const event=new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,bubbles:true,cancelable:true});
  node.dispatchEvent(event);return event;
 };
 const focus=async(node,label)=>{
  assert(node,label+' exists');node.focus();await wait(label+' focused',()=>active()===node);
 };
 const setInput=async(node,value,label)=>{
  Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,value);
  node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));
  await wait(label+' draft applied',()=>node.isConnected&&node.value===value);
 };
 const opened=()=>!!dialog()?.matches(':popover-open');
 const openFilter=async()=>{
  if(!opened()){await focus(filterTrigger(),'Status filter trigger');filterTrigger().click();}
  await wait('native Status popovertarget opens actual filter dialog',()=>opened()&&filterInput());
 };
 const closeFilter=async()=>{
  if(opened()){filterTrigger().click();await wait('native Status popovertarget closes dialog',()=>!opened());}
 };
 const setFilter=async(value,visible)=>{
  await openFilter();await setInput(filterInput(),value,'Status filter');
  await wait('Status filter '+value+' changes actual result set',()=>count('visible')===visible&&filterTrigger()?.dataset.active==='true');
 };
 const beginEdit=async(id,column)=>{
  await focus(cell(id,column),id+' '+column);key(active(),'F2');
  await wait(id+' '+column+' F2 editor mounted and focused',()=>editor()&&active()===editor()&&cell(id,column)?.contains(editor()));
  assert(editor().getAttribute('aria-label')?.endsWith(' for '+id),'editor identifies stable row ID');
 };
 const snapshot=()=>({visible:count('visible'),selected:count('selected'),saved:count('saved'),held:count('held'),
  first:data('firstRow'),sort:data('sort'),lastRow:data('lastRow'),lastColumn:data('lastColumn')});

 await wait('actual controlled worklist mounted',()=>status()&&grid()&&row('worklist-000')&&row('worklist-001')&&get('worklist-restore')&&filterTrigger());
 await wait('WebView document ready for focus input',()=>document.hasFocus());
 assert(count('total')===96&&count('visible')===96&&count('selected')===0&&count('saved')===0&&count('held')===0,'fresh worklist owns96 rows and zero counters');
 assert(data('firstRow')==='worklist-000'&&data('sort')==='none','initial stable identity and unsorted order');
 assert(get('worklist-mark-service').disabled&&get('worklist-clear-selection').disabled&&get('worklist-clear-filters').disabled,'empty selection and filter actions start disabled');
 assert(cell('worklist-000','asset').textContent.includes('Asset 000')&&cell('worklist-000','status').textContent.includes('Normal')&&cell('worklist-001','status').textContent.includes('Warning'),'illustrative dataset uses declared asset/status values');
 assert(outer().dataset.virtualized==='true'&&outer().dataset.striped==='true'&&rows().length>0&&rows().length<96,'actual worklist is striped with a bounded mounted row window');
 await focus(rowCheck('worklist-001'),'stable row selection');rowCheck('worklist-001').click();
 await wait('Warning row selected by stable ID',()=>count('selected')===1&&row('worklist-001')?.getAttribute('aria-selected')==='true');
 inputFocusFacts={documentFocused:document.hasFocus(),activeLabel:active()?.getAttribute('aria-label')};
 sortButton().click();
 await wait('Asset ascending sort',()=>data('sort')==='asset-ascending'&&data('firstRow')==='worklist-000'&&assetHeader()?.getAttribute('aria-sort')==='ascending');
 assert(count('selected')===1,'ascending sort retains selection');
 sortButton().click();
 await wait('Asset descending sort changes actual first row',()=>data('sort')==='asset-descending'&&data('firstRow')==='worklist-095'&&row('worklist-095')&&assetHeader()?.getAttribute('aria-sort')==='descending');
 assert(count('selected')===1&&cell('worklist-095','asset').textContent.includes('Asset 095'),'descending order reflects actual dataset without changing selection');
 sortButton().click();
 await wait('third sort click restores unsorted stable order',()=>data('sort')==='none'&&data('firstRow')==='worklist-000'&&assetHeader()?.getAttribute('aria-sort')==='none'&&row('worklist-001')?.getAttribute('aria-selected')==='true');
 snapshots.sortCycle=snapshot();

 await setFilter('Normal',32);
 assert(!row('worklist-001')&&count('selected')===1&&grid().getAttribute('aria-rowcount')==='33','Normal filter hides selected Warning row while retaining its identity');
 await setInput(filterInput(),'No matching status xyz','empty Status filter');
 await wait('empty filter has truthful empty rows and disabled header selection',()=>count('visible')===0&&rows().length===0&&host().querySelector('.ix-data-grid-empty')?.textContent.includes('No Rows To Show')&&headerCheck()?.disabled);
 assert(count('selected')===1,'empty result preserves hidden selection');
 dialog().querySelector('.ix-data-grid-filter-clear').click();
 await wait('native filter Reset restores96 rows and clears active indicator',()=>count('visible')===96&&filterTrigger()?.dataset.active==='false'&&get('worklist-clear-filters').disabled);
 await closeFilter();
 await setFilter('Normal',32);await closeFilter();
 assert(headerCheck().getAttribute('aria-checked')==='false','header selection reflects filtered results only');
 headerCheck().click();
 await wait('header selects32 filtered rows plus one hidden selection',()=>count('selected')===33&&headerCheck()?.getAttribute('aria-checked')==='true');
 headerCheck().click();
 await wait('header deselects filtered rows while preserving hidden selection',()=>count('selected')===1&&headerCheck()?.getAttribute('aria-checked')==='false');
 headerCheck().click();
 await wait('filtered rows selected again for bulk action',()=>count('selected')===33&&headerCheck()?.getAttribute('aria-checked')==='true');
 sortButton().click();await wait('sort applied inside filtered worklist',()=>data('sort')==='asset-ascending'&&count('visible')===32);
 assert(!get('worklist-mark-service').disabled,'bulk action enables with actual selected rows');
 get('worklist-mark-service').click();
 await wait('bulk action changes all selected Normal rows while retaining filter/sort/selection',()=>count('visible')===0&&count('selected')===33&&data('sort')==='asset-ascending'&&filterTrigger()?.dataset.active==='true'&&headerCheck()?.disabled);
 assert(get('worklist-feedback').textContent.includes('33'),'bulk feedback reports all selected rows including hidden row');
 snapshots.bulkFiltered=snapshot();
 get('worklist-clear-filters').click();
 await wait('clear filters reveals bulk changes and preserves sort/selection',()=>count('visible')===96&&count('selected')===33&&data('sort')==='asset-ascending'&&row('worklist-001')&&get('worklist-clear-filters').disabled);
 assert(cell('worklist-000','status').textContent.includes('Maintenance')&&cell('worklist-001','status').textContent.includes('Maintenance'),'bulk changes both filtered Normal row and hidden selected Warning row');
 assert(row('worklist-001').getAttribute('aria-selected')==='true'&&cell('worklist-004','status').textContent.includes('Warning'),'selection identity remains stable and unselected Warning sibling stays unchanged');
 get('worklist-clear-selection').click();
 await wait('clear selection preserves data and sort',()=>count('selected')===0&&get('worklist-mark-service').disabled&&get('worklist-clear-selection').disabled&&data('sort')==='asset-ascending');
 get('worklist-restore').click();
 await wait('restore remounts grid with fresh controlled data/state',()=>!editor()&&count('visible')===96&&count('selected')===0&&data('sort')==='none'&&cell('worklist-000','status')?.textContent.includes('Normal')&&cell('worklist-001','status')?.textContent.includes('Warning'));

 await beginEdit('worklist-000','asset');
 assert(editor().value==='Asset 000','F2 starts with actual controlled cell value');
 await setInput(editor(),'Asset 000 serviced','asset editor');key(editor(),'Enter');
 await wait('Enter accepts stable row/column payload and renders host-applied value',()=>!editor()&&count('saved')===1&&data('lastRow')==='worklist-000'&&data('lastColumn')==='asset'&&cell('worklist-000','asset')?.textContent.includes('Asset 000 serviced')&&active()===cell('worklist-000','asset'));
 snapshots.savedEdit=snapshot();
 await beginEdit('worklist-000','asset');await setInput(editor(),'','required asset editor');key(editor(),'Enter');
 await wait('blank required asset remains in editor with validation alert',()=>editor()?.getAttribute('aria-invalid')==='true'&&alert()?.textContent.trim().length>0);
 assert(count('saved')===1&&count('held')===0,'invalid draft never reaches host acceptance');
 key(editor(),'Escape');
 await wait('Escape discards invalid draft and restores cell focus/value',()=>!editor()&&active()===cell('worklist-000','asset')&&cell('worklist-000','asset').textContent.includes('Asset 000 serviced'));
 assert(!alert()&&count('saved')===1,'cancellation clears validation without saving');

 get('worklist-hold-edits').click();await wait('host preview-only mode enabled',()=>get('worklist-hold-edits').checked);
 await beginEdit('worklist-000','owner');assert(editor().value==='Team 1','owner editor starts with controlled source value');
 await setInput(editor(),'Team 9 preview','held owner editor');key(editor(),'Enter');
 await wait('host holds accepted payload without applying rows',()=>!editor()&&count('held')===1&&count('saved')===1&&data('lastRow')==='worklist-000'&&data('lastColumn')==='owner'&&cell('worklist-000','owner')?.textContent.includes('Team 1'));
 assert(!cell('worklist-000','owner').textContent.includes('Team 9'),'held edit leaves actual controlled value unchanged');
 snapshots.heldEdit=snapshot();
 get('worklist-hold-edits').click();await wait('host preview-only mode disabled',()=>!get('worklist-hold-edits').checked);
 for(const [id,column] of [['worklist-001','owner'],['worklist-000','ticket']]){
  await focus(cell(id,column),id+' readonly '+column);assert(active().getAttribute('aria-readonly')==='true','declared read-only cell reports its state');
  key(active(),'F2');await wait('read-only '+column+' blocks editing',()=>!editor()&&active()===cell(id,column));
 }
 get('worklist-readonly').click();
 await wait('whole worklist becomes read-only',()=>get('worklist-readonly').checked&&cell('worklist-000','asset')?.getAttribute('aria-readonly')==='true');
 await focus(cell('worklist-000','asset'),'whole-grid readonly asset');key(active(),'F2');
 await wait('whole-grid readonly blocks F2 editor',()=>!editor()&&active()===cell('worklist-000','asset'));
 rowCheck('worklist-002').click();
 await wait('read-only worklist still supports selection',()=>count('selected')===1&&row('worklist-002')?.getAttribute('aria-selected')==='true');
 assert(count('saved')===1&&count('held')===1,'readonly attempts do not emit accepted edit callbacks');
 get('worklist-readonly').click();await wait('editable mode restored',()=>!get('worklist-readonly').checked&&cell('worklist-000','asset')?.getAttribute('aria-readonly')==='false');
 await beginEdit('worklist-000','asset');await setInput(editor(),'Discard pending edit on restore','pending asset editor');
 get('worklist-restore').click();
 await wait('restore discards pending editor and resets actual data/counters/state',()=>!editor()&&count('visible')===96&&count('selected')===0&&count('saved')===0&&count('held')===0&&data('sort')==='none'&&data('firstRow')==='worklist-000'&&data('lastRow')==='none'&&data('lastColumn')==='none'&&cell('worklist-000','asset')?.textContent.trim()==='Asset 000');
 assert(!get('worklist-hold-edits').checked&&!get('worklist-readonly').checked&&cell('worklist-000','owner').textContent.includes('Team 1')&&cell('worklist-000','status').textContent.includes('Normal'),'restore resets mode controls and original row values');
 const box=outer().getBoundingClientRect();
 layout={width:box.width,height:box.height,gridClientWidth:grid().clientWidth,gridScrollWidth:grid().scrollWidth,
  gridClientHeight:grid().clientHeight,gridScrollHeight:grid().scrollHeight,mountedRows:rows().length,
  pageClientWidth:document.documentElement.clientWidth,pageScrollWidth:document.documentElement.scrollWidth};
 assert(Math.abs(layout.height-300)<=1&&layout.gridScrollHeight>layout.gridClientHeight&&layout.mountedRows>0&&layout.mountedRows<96,'300px actual grid viewport retains vertical scrolling and bounded row mounting');
 assert(layout.pageScrollWidth<=layout.pageClientWidth+1,'grid overflow remains inside worklist rather than page');
 snapshots.restored=snapshot();
 return {ok:true,layout,snapshots,inputFocusFacts,
  checks:['actual96-row striped virtualized dataset','ascending/descending/none sort and stable selection','native Status popovertarget filter Normal/empty/Reset','filtered select-all/deselect preserves hidden selection','bulk Maintenance updates selected hidden rows and retains sort/filter/selection','clear selection and filters preserve other state','F2/input/Enter controlled edit payload and focus','required blank validation and Escape discard','host-held edit preserves original data','readonly cells/wholegrid block editing while selection works','restore remount discards pending editor and resets data/state','actual300px viewport and page overflow containment'],
  input:'synthetic click/input/keydown events with explicit native focus and real native popovertarget defaults',
  trustedInput:'separate Chromium keyboard activation, typing and editor input defaults',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),layout,snapshots,inputFocusFacts,documentFocusedNow:document.hasFocus()}}
"#;
