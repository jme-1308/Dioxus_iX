//! Excluded actual-gallery wide equipment Grid preview and bounded native DOM replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/wide_equipment_grid.rs"]
mod gallery_wide;

#[component]
pub fn WideEquipmentGridPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_wide::STYLES} }
        IxTheme { class: "wide-equipment-grid-review",
            h1 { "Wide equipment Grid" }
            Button { id: "wide-equipment-grid-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "wide-equipment-grid-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_wide::WideEquipmentGridExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_WIDE_GRID_CHECK);
                        if let Err(error) = query.send("widegrid") {
                            eprintln!("Wide equipment Grid native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native wide equipment Grid: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native wide equipment Grid failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Wide equipment Grid native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.wide-equipment-grid-review{box-sizing:border-box;min-height:100vh;padding:24px}.wide-equipment-grid-review>h1{font-size:28px;margin:0 0 24px}.wide-equipment-grid-review h2{font-size:22px;margin:24px 0 16px}.wide-equipment-grid-review p{max-width:72ch;line-height:1.5}#wide-equipment-grid-frame{margin-top:24px}@media(max-width:600px){.wide-equipment-grid-review{padding:16px}}";

// Excluded actual-gallery replay: synthetic Grid keys/form events and native scroll/focus.
// Trusted keyboard defaults and pointer interactions are separate lead browser evidence.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_WIDE_GRID_CHECK: &str = r#"
let snapshots={},layout={},hitFacts={},failureGeometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement,host=()=>get('wide-grid-host'),result=()=>get('wide-grid-result');
 const root=()=>host()?.querySelector('.ix-data-grid'),grid=()=>host()?.querySelector('.ix-data-grid-scroll[role="grid"]'),note=()=>get('wide-grid-note');
 const rows=()=>[...grid()?.querySelectorAll('.ix-data-grid-body .ix-data-grid-row[data-row-id]')??[]],row=id=>rows().find(node=>node.dataset.rowId===id);
 const headings=()=>[...grid()?.querySelectorAll('.ix-data-grid-heading[data-column-id]')??[]],heading=column=>headings().find(node=>node.dataset.columnId===column);
 const cells=id=>[...row(id)?.querySelectorAll('[role="gridcell"][data-column-id]')??[]],cell=(id,column)=>cells(id).find(node=>node.dataset.columnId===column);
 const editor=()=>host()?.querySelector('.ix-data-grid-editor[data-retiring="false"]'),editorAt=(id,column)=>cell(id,column)?.querySelector('.ix-data-grid-editor[data-retiring="false"]');
 const trigger=column=>heading(column)?.querySelector('.ix-data-grid-filter-trigger'),popup=column=>get(trigger(column)?.getAttribute('popovertarget'));
 const count=name=>Number(result()?.dataset[name]),order=()=>result()?.dataset.columnOrder?.split(',')??[];
 const stored=(id,column)=>get('wide-grid-record-'+id)?.querySelector('[data-field="'+column+'"]')?.textContent.trim();
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{let stable=0;for(let attempt=0;attempt<150;attempt++){stable=predicate()?stable+1:0;if(stable===3)return;await hold(20)}throw Error(label+'; result='+JSON.stringify(result()?.dataset)+'; scroll='+grid()?.scrollLeft+'; headers='+headings().map(node=>node.dataset.columnId)+'; focus='+(active()?.id||active()?.dataset?.columnId||active()?.tagName))};
 const key=(node,value,ctrlKey=false,shiftKey=false)=>{const event=new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,ctrlKey,shiftKey,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' actual native focus',()=>active()===node)};
 const input=async(node,text,label)=>{assert(node,label+' exists');Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,text);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait(label+' native draft updated',()=>node.isConnected&&node.value===text)};
 const scroll=async side=>{const node=grid(),position=side==='end'?node.scrollWidth-node.clientWidth:0;node.scrollLeft=position;node.dispatchEvent(new Event('scroll'));await wait('native horizontal scroll reaches '+side,()=>Math.abs(node.scrollLeft-position)<2)};
 const inPort=node=>{const box=node?.getBoundingClientRect(),port=grid()?.getBoundingClientRect();return !!box&&box.width>0&&box.left>=port.left-1&&box.right<=port.left+grid().clientWidth+1};
 const hitTestable=(node,insideGrid=false)=>{
  const box=node?.getBoundingClientRect(),frame=get('wide-equipment-grid-frame')?.getBoundingClientRect(),port=grid()?.getBoundingClientRect(),hit=box?document.elementFromPoint(box.left+box.width/2,box.top+box.height/2):null;
  failureGeometry={node:node?.id||node?.dataset?.columnId,nodeRect:box?{left:box.left,right:box.right,top:box.top,bottom:box.bottom,width:box.width,height:box.height}:null,portRect:port?{left:port.left,right:port.right,top:port.top,bottom:port.bottom,width:port.width}:null,scrollLeft:grid()?.scrollLeft,frameWidth:frame?.width,innerWidth,innerHeight,hit:hit?{id:hit.id,tag:hit.tagName,className:hit.className}:null};
  return !!box&&box.width>0&&box.height>0&&box.left>=frame.left-1&&box.right<=frame.right+1&&box.top>=0&&box.bottom<=innerHeight+1&&(!insideGrid||inPort(node))&&(hit===node||node.contains(hit));
 };
 const verifyPositions=()=>{const ordered=order();assert(ordered.length===36,'full model has36 columns');for(const node of [...headings(),...rows().flatMap(node=>cells(node.dataset.rowId))]){const expected=ordered.indexOf(node.dataset.columnId)+2;assert(expected>=2&&node.getAttribute('aria-colindex')===String(expected)&&getComputedStyle(node).gridColumnStart===String(expected),'actual original CSS track and ARIA column index '+node.dataset.columnId)}};
 const begin=async(id,column)=>{await focus(cell(id,column),id+' '+column);key(active(),'F2');await wait('actual '+id+' '+column+' editor focused',()=>editorAt(id,column)&&active()===editorAt(id,column));return editor()};
 const cancel=async(id,column)=>{key(editor(),'Escape');await wait('Escape cancels draft and restores actual visible cell',()=>!editor()&&active()===cell(id,column)&&inPort(cell(id,column)))};
 const snapshot=()=>({mountedHeaders:headings().map(node=>node.dataset.columnId),mountedRows:rows().length,mountedCells:rows().reduce((sum,node)=>sum+cells(node.dataset.rowId).length,0),scrollLeft:grid()?.scrollLeft,visible:count('visible'),selected:count('selected'),edits:count('edits'),fullColumns:result()?.dataset.fullColumns,order:order(),first:{asset:stored('equipment-001','asset'),check32:stored('equipment-001','check-32'),review:stored('equipment-001','review')},last:{asset:stored('equipment-064','asset'),check32:stored('equipment-064','check-32')},note:note()?.value});

 await wait('actual36-column64-row inspection sheet mounted',()=>root()&&grid()&&result()&&note()&&row('equipment-001')&&heading('asset')&&order().length===36);await wait('native WebView ready for focus',()=>document.hasFocus());
 assert(count('total')===64&&count('columns')===36&&count('visible')===64&&count('selected')===0&&count('edits')===0&&root().dataset.columnVirtualized==='true'&&grid().getAttribute('aria-colcount')==='37'&&grid().getAttribute('aria-rowcount')==='65'&&rows().length>0&&rows().length<64&&headings().length>0&&headings().length<=18&&rows().every(node=>cells(node.dataset.rowId).length<=18),'actual row/column mounted work bounded while full model/ARIA stays complete');
 assert(stored('equipment-001','asset')==='Motor 001'&&stored('equipment-001','check-32')==='35'&&stored('equipment-001','review')==='Pending review'&&stored('equipment-064','asset')==='Motor 064'&&stored('equipment-064','check-32')==='22'&&cell('equipment-001','asset').textContent.trim()==='Motor 001'&&note().value==='Prepare inspection sheet','actual first/last records and early displayed cell');verifyPositions();const retainedNote=note();await focus(note(),'independent sheet preparation');await input(note(),'Prepare wide equipment inspection','preparation');snapshots.boundedMount=snapshot();

 await scroll('end');await wait('native horizontal movement renders actual far columns and releases unretained early headers',()=>heading('review')&&heading('check-32')&&cell('equipment-001','review')&&!heading('owner')&&inPort(cell('equipment-001','review')));
 assert(cell('equipment-001','review').textContent.trim()==='Pending review'&&cell('equipment-001','check-32').textContent.trim()==='35'&&headings().length<=18,'native far viewport contains correct actual record values with bounded mounts');verifyPositions();const farHeader=heading('review').getBoundingClientRect(),priorHeader=heading('check-32').getBoundingClientRect(),farCell=cell('equipment-001','review').getBoundingClientRect();assert(Math.abs(priorHeader.right-farHeader.left)<2&&Math.abs(farCell.left-farHeader.left)<2&&Math.abs(farHeader.width-220)<2,'unmounted columns leave original full-track positions and220px review width');snapshots.horizontal=snapshot();

 await scroll('start');await wait('early cell remounts after native scroll',()=>cell('equipment-001','asset'));await focus(cell('equipment-001','asset'),'first cell for Ctrl+Right');key(active(),'ArrowRight',true);await wait('Ctrl+Right pending far target mounts and Dioxus reveals native focused cell',()=>cell('equipment-001','review')&&active()===cell('equipment-001','review')&&inPort(active())&&grid().scrollLeft>0);verifyPositions();snapshots.pendingFocus=snapshot();

 await begin('equipment-001','review');await input(editor(),'Reviewed in native preview','late Review');key(editor(),'Tab');await wait('late editor Tab saves actual row and advances to next row first editor',()=>count('edits')===1&&stored('equipment-001','review')==='Reviewed in native preview'&&editorAt('equipment-002','asset')&&active()===editorAt('equipment-002','asset')&&inPort(editorAt('equipment-002','asset')));assert(editor().value==='Pump 002','Tab editor has actual next row source value');await cancel('equipment-002','asset');snapshots.lateEditTab=snapshot();

 await scroll('start');await wait('early owner remounted for retained draft',()=>cell('equipment-001','owner'));const retainedEditor=await begin('equipment-001','owner');await input(retainedEditor,'Uncommitted team draft','retained owner editor');await scroll('end');await wait('horizontal scroll keeps actual native editor/draft alive offscreen',()=>heading('review')&&retainedEditor.isConnected&&editor()===retainedEditor&&active()===retainedEditor&&retainedEditor.value==='Uncommitted team draft'&&cell('equipment-001','owner'));
 assert(stored('equipment-001','owner')==='Team A'&&count('edits')===1,'horizontal scroll does not commit or lose pending actual edit');await cancel('equipment-001','owner');assert(stored('equipment-001','owner')==='Team A'&&count('edits')===1,'Cancel preserves saved owner value and edit count');snapshots.editorRetention=snapshot();

 await scroll('start');await wait('asset header available for logical navigation',()=>heading('asset'));await focus(heading('asset').querySelector('.ix-data-grid-sort'),'native first header action');key(active(),'ArrowRight',true);await wait('header Ctrl+Right mounts/reveals actual last header focus lane',()=>active()?.closest('.ix-data-grid-heading')?.dataset.columnId==='review'&&heading('review')?.getAttribute('aria-colindex')==='37'&&inPort(heading('review')));key(active(),'ArrowLeft',true);await wait('header Ctrl+Left restores first logical header focus',()=>active()?.closest('.ix-data-grid-heading')?.dataset.columnId==='asset'&&inPort(heading('asset')));key(active(),'ArrowRight');await wait('header ArrowRight reaches adjacent logical Owner header',()=>active()?.closest('.ix-data-grid-heading')?.dataset.columnId==='owner'&&inPort(heading('owner')));
 await wait('owner header available',()=>heading('owner'));const focusedHeader=heading('owner'),sortButton=focusedHeader.querySelector('.ix-data-grid-sort');await focus(sortButton,'actual header sort button');await scroll('end');await wait('focused header survives native horizontal scroll',()=>heading('owner')===focusedHeader&&focusedHeader.isConnected&&active()===sortButton&&heading('review'));
 await scroll('start');trigger('owner').click();await wait('actual native Owner filter opens',()=>popup('owner')?.matches(':popover-open')&&popup('owner').querySelector('input'));
 const filterPopup=popup('owner'),filterInput=filterPopup.querySelector('input');await focus(filterInput,'actual native Owner filter');await scroll('end');await wait('open native filter and its header survive horizontal scroll',()=>heading('owner')===focusedHeader&&filterPopup.isConnected&&filterPopup.matches(':popover-open')&&active()===filterInput&&heading('review'));
 await input(filterInput,'Team A','actual Owner filter');await wait('retained native filter changes real projected rows',()=>count('visible')===32&&row('equipment-001')&&rows().every(node=>cell(node.dataset.rowId,'owner')?.textContent.trim()==='Team A')&&count('edits')===1);filterPopup.querySelector('.ix-data-grid-filter-clear').click();await wait('Reset filter restores real64-row projection',()=>count('visible')===64&&rows().length>1);trigger('owner').click();await wait('native Owner filter closed',()=>!filterPopup.matches(':popover-open'));assert(note()===retainedNote&&note().value==='Prepare wide equipment inspection','header/filter work retains independent preparation');snapshots.headerFilterRetention=snapshot();

 const options=get('wide-grid-options');options.querySelector('summary').click();await wait('native sheet options expanded',()=>options.open);get('wide-grid-full-columns').click();await wait('opt-out mounts all36 actual headers/cells without disabling row virtualization',()=>result().dataset.fullColumns==='true'&&root().dataset.columnVirtualized==='false'&&headings().length===36&&rows().every(node=>cells(node.dataset.rowId).length===36)&&rows().length<64&&stored('equipment-001','review')==='Reviewed in native preview'&&count('edits')===1);verifyPositions();get('wide-grid-full-columns').click();await wait('ordinary bounded column rendering restored',()=>result().dataset.fullColumns==='false'&&root().dataset.columnVirtualized==='true'&&headings().length<=18);
 get('wide-grid-team').click();await wait('actual Team A owner filter projects32 records',()=>count('visible')===32);get('wide-grid-select').click();await wait('actual Select visible records selection across full columns',()=>count('selected')===32);get('wide-grid-team').click();await wait('actual full owner projection restored without losing selected identities',()=>count('visible')===64&&count('selected')===32);
 await scroll('start');await wait('asset header before width update',()=>heading('asset'));get('wide-grid-width').click();await wait('controlled actual asset width reaches260px track',()=>Math.abs(heading('asset').getBoundingClientRect().width-260)<2);get('wide-grid-reverse').click();await wait('actual full column order reversed',()=>order()[0]==='review'&&order()[35]==='asset');verifyPositions();assert(stored('equipment-001','review')==='Reviewed in native preview'&&count('edits')===1&&note()===retainedNote&&note().value==='Prepare wide equipment inspection','opt-out/selection/filter/width/reorder preserve actual edited model and independent note');snapshots.optOutModelOptions=snapshot();

 const reset=get('wide-grid-reset'),oldRoot=root(),oldNote=note();await focus(reset,'Restore inspection sheet');reset.click();await wait('Restore actual sheet/records/options/counts and owned preparation',()=>root()&&root()!==oldRoot&&count('visible')===64&&count('selected')===0&&count('edits')===0&&stored('equipment-001','asset')==='Motor 001'&&stored('equipment-001','review')==='Pending review'&&order()[0]==='asset'&&order()[35]==='review'&&note()?.value==='Prepare inspection sheet'&&!get('wide-grid-full-columns').checked&&!get('wide-grid-readonly').checked&&headings().length<=18);assert(!oldRoot.isConnected&&!oldNote.isConnected&&note()!==oldNote&&get('wide-grid-reset')===reset&&get('wide-grid-options')===options&&active()===reset,'Restore replaces owned Grid/preparation and preserves toolbar/options focus');
 get('wide-equipment-grid-narrow').click();await wait('340px grid remains internally scrollable without page overflow',()=>get('wide-equipment-grid-frame').getBoundingClientRect().width<=341&&grid().scrollWidth>grid().clientWidth&&headings().length<=18&&document.documentElement.scrollWidth<=document.documentElement.clientWidth+1);await focus(note(),'narrow preparation');await wait('narrow native note receives pointer hit',()=>hitTestable(note()));hitFacts.narrowNote=failureGeometry.hit;await input(note(),'Narrow inspection preparation','narrow preparation');await scroll('start');await wait('narrow first cell available',()=>cell('equipment-001','asset'));await focus(cell('equipment-001','asset'),'narrow first native cell');key(active(),'ArrowRight',true);await wait('narrow offscreen last cell mounts/reveals/focuses',()=>cell('equipment-001','review')&&active()===cell('equipment-001','review')&&inPort(active()));await wait('narrow late native cell receives pointer hit',()=>hitTestable(cell('equipment-001','review'),true));hitFacts.narrowLateCell=failureGeometry.hit;
 await begin('equipment-001','review');await input(editor(),'Narrow review saved','narrow late Review');key(editor(),'Enter');await wait('narrow actual late edit saves to controlled record',()=>count('edits')===1&&stored('equipment-001','review')==='Narrow review saved'&&!editor()&&active()===cell('equipment-001','review')&&inPort(active()));verifyPositions();const frame=get('wide-equipment-grid-frame').getBoundingClientRect();layout={frameWidth:frame.width,scrollClient:grid().clientWidth,scrollWidth:grid().scrollWidth,mountedHeaders:headings().length,mountedRows:rows().length,headerHeight:grid().querySelector('.ix-data-grid-header')?.getBoundingClientRect().height,rowHeight:row('equipment-001').getBoundingClientRect().height,pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};assert(layout.frameWidth<=341&&layout.mountedHeaders<=18&&layout.mountedRows<64&&layout.pageScroll<=layout.pageClient+1&&note().value==='Narrow inspection preparation'&&count('selected')===0,'narrow bounded mount/edit and retained preparation produce real contained workflow');snapshots.resetNarrow=snapshot();
 return {ok:true,snapshots,layout,hitFacts,checks:['bounded actual wide headers/cells and row virtualization preserve complete model/ARIA','native horizontal scroll remounts correct far values at original CSS/ARIA positions','Ctrl+Right pending far target mounts/reveals actual native focused cell','late native editor saves actual record and Tab starts next-row editor','offscreen horizontal scroll retains native focused editor/draft until Cancel','logical header keyboard reachability plus focused header/open native filter retention and real projection','opt-out mounts all columns; actual owner filter/selection/width/order preserve edits','scoped Restore and340px internal scrolling/offscreen edit/native note hit workflow'],input:'native scrollLeft plus typed scroll event, synthetic Grid keys/input events and explicit native focus; Grid handles editor Tab/Ctrl+Arrow, trusted native defaults are separate',trustedInput:'separate Chromium trusted wheel/scroll/pointer/typing/Tab and legacy small-grid regressions by lead; no physical-device certification',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,hitFacts,failureGeometry,documentFocusedNow:document.hasFocus()}}
"#;
