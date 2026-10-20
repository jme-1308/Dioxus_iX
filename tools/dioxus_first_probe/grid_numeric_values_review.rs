//! Excluded actual-gallery numeric Grid and bounded Linux DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/grid_numeric_values.rs"]
mod gallery_numeric;

#[component]
pub fn GridNumericValuesPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_numeric::STYLES} }
        IxTheme { class: "grid-numeric-values-review",
            h1 { "Numeric stock planning" }
            Button { id: "grid-numeric-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "grid-numeric-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_numeric::GridNumericValuesExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_GRID_NUMERIC_VALUES_CHECK);
                        if let Err(error) = query.send("gridnumeric") {
                            eprintln!("Numeric Grid native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native numeric Grid: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native numeric Grid failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Numeric Grid native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.grid-numeric-values-review{box-sizing:border-box;min-height:100vh;padding:24px}.grid-numeric-values-review h1{font-size:28px;margin:0 0 24px}.grid-numeric-values-review h2{font-size:22px;margin:24px 0 16px}.grid-numeric-values-review h3{font-size:18px}.grid-numeric-values-review p{max-width:72ch;line-height:1.5}#grid-numeric-frame{margin-top:24px}@media(max-width:600px){.grid-numeric-values-review{padding:16px}}";

// Excluded driver JS sends synthetic form/click/key events and explicitly requests
// native focus. The actual Rust/Dioxus Grid owns projection, filters and editing.
// Synthetic Tab is checked only where Grid handles it. Native number sanitization
// is observed directly; raw invalid/nonfinite queries remain Rust unit coverage.
// Chromium separately checks trusted keyboard input and native select defaults.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_GRID_NUMERIC_VALUES_CHECK: &str = r#"
let snapshots={},layout={},nativeInputFacts={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const host=()=>get('grid-numeric-values'),result=()=>get('numeric-result');
 const grid=()=>host()?.querySelector('[role="grid"][aria-label="Stock planning"]');
 const rows=()=>[...grid()?.querySelectorAll('.ix-data-grid-body [data-row-id]')??[]];
 const row=id=>rows().find(node=>node.dataset.rowId===id);
 const cell=(id,column)=>row(id)?.querySelector('[role="gridcell"][data-column-id="'+column+'"]');
 const heading=column=>grid()?.querySelector('[role="columnheader"][data-column-id="'+column+'"]');
 const sort=column=>heading(column)?.querySelector('.ix-data-grid-sort');
 const trigger=column=>heading(column)?.querySelector('.ix-data-grid-filter-trigger');
 const popup=column=>get(trigger(column)?.getAttribute('popovertarget'));
 const operator=(column,n=1)=>popup(column)?.querySelector('select[aria-label="'+({units:'Units',cost:'Unit cost',asset:'Asset'}[column])+' filter condition '+n+'"]');
 const value=(column,n=1)=>popup(column)?.querySelector('input[aria-label="'+({units:'Units',cost:'Unit cost',asset:'Asset'}[column])+' filter value '+n+'"]');
 const join=column=>popup(column)?.querySelector('select[aria-label="Combine filter conditions"]');
 const editor=()=>host()?.querySelector('.ix-data-grid-editor[data-retiring="false"]');
 const editorAt=(id,column)=>cell(id,column)?.querySelector('.ix-data-grid-editor[data-retiring="false"]');
 const data=name=>result()?.dataset[name],number=name=>Number(data(name));
 const order=()=>data('order')?.split(',').filter(Boolean)??[];
 const selected=()=>rows().filter(node=>node.getAttribute('aria-selected')==='true').map(node=>node.dataset.rowId);
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
  throw Error(label+'; result='+result()?.textContent+'; order='+order()+'; focus='+(active()?.getAttribute('aria-label')||active()?.dataset?.columnId||active()?.id||active()?.tagName));
 };
 const expectOrder=async(ids,label)=>wait(label,()=>order().join(',')===ids.map(id=>'stock-'+id).join(',')&&rows().map(node=>node.dataset.rowId).join(',')===order().join(',')&&number('visible')===ids.length);
 const key=(node,key,shiftKey=false)=>{const event=new KeyboardEvent('keydown',{key,code:key===' '?'Space':key,shiftKey,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus();await wait(label+' native focus established',()=>active()===node)};
 const setInput=async(node,text,label)=>{
  assert(node,label+' input exists');Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,text);
  node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));
  await wait(label+' native input value applied',()=>node.isConnected&&node.value===text);
 };
 const choice=async(node,text,label)=>{
  assert(node&&[...node.options].some(option=>option.value===text),label+' option available');
  node.value=text;node.dispatchEvent(new Event('change',{bubbles:true}));
  await wait(label+' actual choice applied',()=>node.isConnected&&node.value===text);
 };
 const reset=async()=>{
  get('numeric-reset').focus();get('numeric-reset').click();
  await expectOrder([1,2,3,4,5,6,7],'Restore stock restores original stable order');
  await wait('Restore stock clears selection, edits, readonly and owned popup/editor',()=>number('selected')===0&&number('edits')===0&&data('qty1')==='2.5'&&data('sort')==='none'&&!checked(get('numeric-readonly'))&&!editor()&&!host().querySelector('.ix-data-grid-filter:popover-open'));
 };
 const open=async column=>{
  trigger(column).click();await wait(column+' actual native filter popup opened',()=>popup(column)?.matches(':popover-open')&&operator(column));
 };
 const close=async column=>{if(popup(column)?.matches(':popover-open'))trigger(column).click();await wait(column+' popup closed',()=>!popup(column)?.matches(':popover-open'))};
 const clear=async column=>{
  popup(column).querySelector('.ix-data-grid-filter-clear').click();
  await wait(column+' filter reset exposes inactive first condition',()=>operator(column)?.value===(column==='asset'?'contains':'equals')&&value(column)?.value===''&&!join(column));
 };
 const condition=async(column,n,op,text)=>{
  await choice(operator(column,n),op,column+' condition '+n);
  if(text!==undefined){await wait(column+' numeric value '+n+' mounted',()=>value(column,n));await setInput(value(column,n),text,column+' value '+n)}
 };
 const begin=async(id,column)=>{
  await focus(cell(id,column),id+' '+column);key(active(),'F2');
  await wait(id+' '+column+' numeric editor focused',()=>editorAt(id,column)&&active()===editorAt(id,column));
  assert(editor().type==='number'&&!editor().hasAttribute('data-select-all'),'numeric editor uses native number without text selection bridge');return editor();
 };
 const cancel=async(id,column)=>{key(editor(),'Escape');await wait('Escape cancels numeric editor',()=>!editor()&&active()===cell(id,column))};
 const snapshot=()=>({order:order(),visible:number('visible'),selected:number('selected'),selectedIds:selected(),edits:number('edits'),qty1:data('qty1'),sort:data('sort'),readonly:checked(get('numeric-readonly'))});

 await wait('actual seven-row numeric stock gallery mounted',()=>host()&&grid()&&result()&&rows().length===7&&get('numeric-reset')&&get('numeric-low-stock')&&get('numeric-clear-filters'));
 await wait('WebView ready for native focus',()=>document.hasFocus());
 await expectOrder([1,2,3,4,5,6,7],'initial host and Grid source order');
 assert(number('edits')===0&&number('selected')===0&&data('qty1')==='2.5'&&get('numeric-clear-filters').disabled,'initial counters and inactive filter clearing');
 assert(cell('stock-6','units').getAttribute('aria-readonly')==='true'&&cell('stock-7','cost').getAttribute('aria-readonly')==='true','blank and unknown source numeric values remain explicit readonly cells');
 sort('units').click();await expectOrder([4,3,1,2,5,6,7],'numeric ascending differs from lexical order and keeps unknowns last');
 assert(heading('units').getAttribute('aria-sort')==='ascending'&&data('sort')==='asc','actual header and host state report ascending Units');snapshots.ascending=snapshot();
 sort('units').click();await expectOrder([5,2,1,3,4,6,7],'numeric descending retains blank/unknown tail');
 assert(data('sort')==='desc','actual header descending reaches controlled host');snapshots.descending=snapshot();
 sort('units').click();await expectOrder([1,2,3,4,5,6,7],'third header activation restores source order');
 sort('asset').click();await expectOrder([1,7,5,4,3,6,2],'ordinary text column keeps case-insensitive alphabetical sorting');snapshots.textSort=snapshot();
 await reset();

 await open('units');
 const numericOptions=[...operator('units').options].map(option=>option.value);
 assert(numericOptions.length===8&&['equals','notEqual','lessThan','lessThanOrEqual','greaterThan','greaterThanOrEqual','blank','notBlank'].every(token=>numericOptions.includes(token))&&!numericOptions.includes('contains'),'numeric popup exposes only its eight supported operators');
 assert(value('units').type==='number'&&value('units').step==='any'&&operator('units').value==='equals','inactive numeric popup starts with native number/any-step Equals');
 await condition('units',1,'equals','2.5');await expectOrder([1],'decimal equality filters actual displayed numeric rows');
 await setInput(value('units'),'2.5e0','scientific decimal query');await expectOrder([1],'scientific query matches equivalent finite decimal');
 const invalid=value('units');Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(invalid,'NaN');
 nativeInputFacts={type:invalid.type,step:invalid.step,invalidAssignmentValue:invalid.value};
 assert(invalid.value==='','native number input sanitizes raw NaN instead of exposing an invalid query');invalid.dispatchEvent(new Event('input',{bubbles:true,composed:true}));
 await expectOrder([1,2,3,4,5,6,7],'sanitized empty numeric query remains inactive safely');
 await condition('units',1,'greaterThanOrEqual','2');
 await wait('second numeric condition appears',()=>operator('units',2)&&join('units'));
 await condition('units',2,'lessThanOrEqual','10');await expectOrder([1,2,3],'inclusive two-condition AND retains both bounds');snapshots.inclusiveAnd=snapshot();
 await condition('units',1,'lessThan','2');await condition('units',2,'greaterThanOrEqual','100');
 await choice(join('units'),'or','numeric OR');await expectOrder([4,5],'OR combines separated finite ranges and excludes unknowns');snapshots.rangeOr=snapshot();
 await clear('units');await condition('units',1,'blank');await expectOrder([6],'Blank selects only empty numeric source');
 assert(!value('units'),'Blank removes irrelevant numeric value input');snapshots.blank=snapshot();
 await clear('units');await condition('units',1,'notBlank');await expectOrder([1,2,3,4,5,7],'Not blank preserves nonempty unknown source as distinct from numeric comparisons');snapshots.notBlank=snapshot();
 await clear('units');await close('units');
 await open('asset');
 assert([...operator('asset').options].some(option=>option.value==='contains')&&value('asset').type==='text','ordinary text popup keeps text operators/input');
 await close('asset');

 get('numeric-low-stock').click();await expectOrder([1,3,4],'Low stock toolbar uses typed LessThan 10 rather than lexical comparison');
 grid().querySelector('.ix-data-grid-header [role="checkbox"]').click();
 await wait('header selects exactly displayed numeric filtered rows',()=>number('selected')===3&&selected().join(',')==='stock-1,stock-3,stock-4');
 get('numeric-clear-filters').click();await expectOrder([1,2,3,4,5,6,7],'Clear filters restores all rows preserving selected IDs');
 assert(number('selected')===3,'clear filter preserves hidden selection');
 await open('units');await condition('units',1,'greaterThanOrEqual','10');await expectOrder([2,5],'second typed filter exposes other numeric rows');
 grid().querySelector('.ix-data-grid-header [role="checkbox"]').click();await wait('filtered header adds new IDs while retaining hidden selected IDs',()=>number('selected')===5&&selected().length===2);
 grid().querySelector('.ix-data-grid-header [role="checkbox"]').click();await wait('filtered deselect leaves hidden selected IDs intact',()=>number('selected')===3&&selected().length===0);snapshots.hiddenSelection=snapshot();
 await reset();sort('units').click();await expectOrder([4,3,1,2,5,6,7],'numeric order ready for range selection');
 row('stock-4').querySelector('[role="checkbox"]').click();await wait('first displayed zero row establishes range anchor',()=>number('selected')===1);
 row('stock-1').querySelector('[role="checkbox"]').dispatchEvent(new MouseEvent('click',{bubbles:true,cancelable:true,shiftKey:true}));
 await wait('Shift range follows numeric displayed order',()=>number('selected')===3&&selected().join(',')==='stock-4,stock-3,stock-1');snapshots.numericRange=snapshot();

 await reset();sort('units').click();await expectOrder([4,3,1,2,5,6,7],'numeric order ready for edited reprojection');
 const quantity=await begin('stock-1','units');
 assert(quantity.value==='2.5'&&quantity.min==='0'&&quantity.max==='1000'&&quantity.step==='0.5','numeric value metadata remains independent of native editor limits');
 await setInput(quantity,'20','numeric quantity draft');key(editor(),'Enter');
 await expectOrder([4,3,2,1,5,6,7],'accepted numeric edit reprojects sorted rows by new numeric value');
 await wait('controlled numeric edit counted once',()=>!editor()&&number('edits')===1&&data('qty1')==='20');
 await begin('stock-2','cost');await setInput(editor(),'80','unit cost draft');key(editor(),'Tab');
 await wait('Tab commits and enters next numeric displayed row',()=>editorAt('stock-1','units')&&active()===editorAt('stock-1','units')&&number('edits')===2&&cell('stock-2','cost').textContent.includes('80'));
 key(editor(),'Tab',true);await wait('Shift Tab returns to previous displayed numeric row',()=>editorAt('stock-2','cost')&&active()===editorAt('stock-2','cost'));
 await cancel('stock-2','cost');assert(number('edits')===2,'unchanged traversal and cancellation do not invent edit callbacks');snapshots.editedSort=snapshot();
 await reset();get('numeric-low-stock').click();await expectOrder([1,3,4],'numeric filter ready for edit leaving projection');
 row('stock-1').querySelector('[role="checkbox"]').click();await wait('selected source row ready for filtered edit',()=>number('selected')===1);
 await begin('stock-1','units');await setInput(editor(),'20','quantity leaving Low stock');key(editor(),'Enter');
 await expectOrder([3,4],'saved quantity leaves filtered displayed rows');
 await wait('filtered edit preserves stable hidden selection and single callback',()=>!editor()&&number('selected')===1&&number('edits')===1&&data('qty1')==='20');snapshots.editedFilter=snapshot();

 await reset();get('numeric-readonly').click();
 await wait('whole Grid readonly reaches numeric cells',()=>checked(get('numeric-readonly'))&&cell('stock-1','units').getAttribute('aria-readonly')==='true');
 await focus(cell('stock-1','units'),'readonly numeric cell');key(active(),'F2');
 await wait('readonly numeric cell does not open editor',()=>!editor()&&number('edits')===0&&active()===cell('stock-1','units'));
 key(active(),' ');await wait('readonly mode preserves normal row selection',()=>number('selected')===1&&row('stock-1').getAttribute('aria-selected')==='true');
 await open('units');await condition('units',1,'lessThan','10');await expectOrder([1,3,4],'readonly Grid still permits numeric filtering');
 await reset();assert(active()===get('numeric-reset'),'external Restore stock remains focused after open popup cleanup');snapshots.reset=snapshot();

 get('grid-numeric-narrow').click();
 await wait('340px numeric Grid keeps horizontal scrolling inside viewport',()=>get('grid-numeric-frame').getBoundingClientRect().width<=341&&grid().scrollWidth>grid().clientWidth);
 await open('units');await focus(value('units'),'narrow numeric filter input');
 await wait('narrow native filter input is visible and hit-testable',()=>{
  const node=value('units'),box=node?.getBoundingClientRect();
  if(!box||box.width<=0||box.height<=0||box.left<0||box.top<0||box.right>innerWidth+1||box.bottom>innerHeight+1)return false;
  const hit=document.elementFromPoint(box.left+box.width/2,box.top+box.height/2);return hit===node||node.contains(hit);
 });
 await setInput(value('units'),'2.5','narrow actual decimal filter');await expectOrder([1],'narrow native numeric filter remains usable');
 const box=grid().closest('.ix-data-grid').getBoundingClientRect(),pop=popup('units').getBoundingClientRect();
 layout={width:box.width,height:box.height,rowHeight:row('stock-1').getBoundingClientRect().height,clientWidth:grid().clientWidth,scrollWidth:grid().scrollWidth,popupWidth:pop.width,popupHeight:pop.height,pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};
 assert(Math.abs(layout.height-300)<=1&&Math.abs(layout.rowHeight-42)<=1&&layout.pageScroll<=layout.pageClient+1,'typed numeric values preserve Grid geometry and narrow page containment');
 await close('units');
 return {ok:true,snapshots,layout,nativeInputFacts,
  checks:['actual numeric asc/desc/unsorted headers with blank and unknown tail','unchanged ordinary text sorting and filter input','native numeric popup operators, decimals and scientific equality','safe native invalid-query sanitation without raw-invalid parsing claims','inclusive two-condition AND and separated OR numeric ranges','Blank versus nonempty unknown Not blank','filtered header selection preserves hidden stable IDs','Shift selection follows numeric displayed order','accepted numeric edit reprojects sorted rows','Tab and ShiftTab editing follow typed displayed row order','edited numeric value leaves filter without losing selected identity','readonly keeps selection/filtering while blocking editing','Restore stock clears owned popup/editor and controlled state','340px numeric filter hit testing and internal horizontal overflow'],
  input:'synthetic click/input/change/key events and explicit native focus; raw invalid/nonfinite query semantics are covered by Rust tests',
  trustedInput:'separate Chromium actual number typing/select keyboard defaults; no physical-device certification',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,nativeInputFacts,documentFocusedNow:document.hasFocus()}}
"#;
