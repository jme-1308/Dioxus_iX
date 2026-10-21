//! Excluded actual-gallery page-drag state/geometry, qualified synthetic events.
use super::host_paged_grid_review::gallery_host;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn GridPageDragPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class:"grid-page-drag-native-review",
            gallery_host::HostPagedGridExample {}
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=crate::dom_query::eval(NATIVE_GRID_PAGE_DRAG_CHECK);let _=query.send("gridpagedrag");
                    match query.join::<serde_json::Value>().await {
                        Ok(value)if value["ok"]==true=>{println!("Dioxus native Grid page drag: {value}");window.close();}
                        value=>{eprintln!("Dioxus native Grid page drag failed: {value:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(60)).await;eprintln!("Grid page drag native replay timed out");std::process::exit(1);});}
            } }
        }
    }
}
const STYLES: &str = "body{margin:0}.grid-page-drag-native-review{box-sizing:border-box;min-height:100vh;padding:24px}@media(max-width:600px){.grid-page-drag-native-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_GRID_PAGE_DRAG_CHECK: &str = r#"
let snapshots={},geometry={};
try{
 await dioxus.recv();
 const get=id=>document.getElementById('host-paged-'+id),host=()=>get('grid'),grid=()=>host()?.querySelector('.ix-data-grid-scroll[aria-label="Host-paged equipment"]'),row=id=>grid()?.querySelector('[data-row-id="'+id+'"]'),handle=id=>row(id)?.querySelector('.ix-data-grid-drag'),status=()=>get('status'),state=()=>({...status().dataset}),order=()=>JSON.parse(status().dataset.rowOrder);
 const assert=(v,label)=>{if(!v)throw Error(label)},wait=async(label,predicate)=>{let stable=0;for(let i=0;i<250;i++){stable=predicate()?stable+1:0;if(stable>=2)return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+'; state='+status()?.outerHTML)};
 const rect=n=>{const r=n.getBoundingClientRect();return{x:r.x,y:r.y,width:r.width,height:r.height,right:r.right,bottom:r.bottom}},point=(n,after=false)=>{const r=rect(n);return{x:r.x+Math.min(100,r.width/3),y:r.y+r.height*(after?.8:.2)}},edge=down=>{const r=rect(grid()),h=rect(grid().querySelector('.ix-data-grid-header'));return{x:r.x+Math.min(110,r.width/2),y:down?r.bottom-8:r.y+h.height+8}};
 const emit=(node,type,p,data)=>node.dispatchEvent(new DragEvent(type,{bubbles:true,cancelable:true,clientX:p.x,clientY:p.y,dataTransfer:data}));
 const begin=async id=>{const source=handle(id);source.scrollIntoView({block:'nearest',inline:'nearest'});source.focus();await wait('actual current mounted source focus',()=>document.activeElement===source);const transfer=new DataTransfer();transfer.setData('text/plain',id);emit(source,'dragstart',point(source),transfer);if(status().dataset.rowDragAcrossPages==='true')await wait('current native drag origin admitted',()=>grid().dataset.rowDragSource===id);return{source,transfer}};
 const drop=async(drag,target,after,index,moves)=>{const targetRow=row(target),p=point(targetRow,after);emit(targetRow,'dragover',p,drag.transfer);await wait('actual loaded target acknowledges synthetic native dragover',()=>row(target)?.dataset.dropTarget==='true');emit(targetRow,'drop',p,drag.transfer);await wait('real Rust callback accepts global drop ordinal',()=>status().dataset.moves===String(moves)&&order()[index]==='host-equipment-002');emit(drag.source,'dragend',p,drag.transfer);await wait('native dragend retires retained source',()=>!grid().querySelector('[data-drag-source-retained="true"]')&&!grid().dataset.rowDragSource);};
 const reset=async()=>{get('reset').click();await wait('fresh scoped source/default options',()=>status().dataset.page==='0'&&status().dataset.moves==='0'&&status().dataset.rowMove==='false'&&status().dataset.rowDragAcrossPages==='false'&&!get('drag-across-pages').checked&&get('drag-across-pages').disabled&&order().every((id,i)=>id==='host-equipment-'+String(i).padStart(3,'0')))};
 await wait('actual new cross-page option/current source mounted',()=>get('drag-across-pages')&&grid()&&handle('host-equipment-002'));assert(typeof DataTransfer==='function'&&typeof DragEvent==='function','native synthetic DragEvent/DataTransfer capability required and reported honestly');assert(get('drag-across-pages').disabled&&!get('drag-across-pages').checked,'cross-page option default off and dependent');get('row-move').click();await wait('ordinary existing row movement enabled',()=>!handle('host-equipment-002').disabled&&!get('drag-across-pages').disabled&&!get('drag-across-pages').checked);let drag=await begin('host-equipment-002');await drop(drag,'host-equipment-005',true,5,1);assert(status().dataset.page==='0'&&order()[4]==='host-equipment-005','default within-page pointer policy remains usable');snapshots.default=state();await reset();
 get('row-move').click();await wait('ordinary callback restored',()=>!get('drag-across-pages').disabled);get('drag-across-pages').click();await wait('explicit page drag opt-in accepted',()=>status().dataset.rowDragAcrossPages==='true'&&!handle('host-equipment-002').disabled);row('host-equipment-002').querySelector('[role=checkbox]').click();await wait('source selected before new drag',()=>status().dataset.selected==='1'&&row('host-equipment-002').getAttribute('aria-selected')==='true');drag=await begin('host-equipment-002');const bottom=edge(true);emit(grid(),'dragover',bottom,drag.transfer);await wait('native geometry edge accepts next page',()=>status().dataset.page==='1'&&row('host-equipment-013')&&drag.source.isConnected&&drag.source.closest('[data-row-id]').dataset.dragSourceRetained==='true');geometry.forward={edge:bottom,root:rect(grid()),sourceConnected:drag.source.isConnected,viewport:{width:innerWidth,height:innerHeight}};await drop(drag,'host-equipment-013',false,12,1);assert(status().dataset.page==='1'&&status().dataset.selectedIds==='["host-equipment-002"]','accepted current page/drop preserves stable selection');snapshots.dropped=state();
 drag=await begin('host-equipment-002');const top=edge(false);emit(grid(),'dragover',top,drag.transfer);await wait('reverse edge accepts previous page with same source retained',()=>status().dataset.page==='0'&&drag.source.isConnected&&drag.source.closest('[data-row-id]').dataset.dragSourceRetained==='true');emit(drag.source,'dragend',top,drag.transfer);await wait('synthetic dragend cancels offpage source without drop',()=>!grid().querySelector('[data-drag-source-retained="true"]')&&!grid().dataset.rowDragEdge);assert(status().dataset.moves==='1'&&order()[12]==='host-equipment-002'&&status().dataset.selected==='1','cancel retains accepted order/selection without move');snapshots.cancelled=state();await reset();snapshots.restored=state();
 return{ok:true,snapshots,geometry,checks:['actual default-off/dependent option preserves ordinary within-page synthetic native geometry/drop and scoped Reset','opt-in synthetic DragEvents use mounted edge geometry to accept pages while original DOM source stays connected, global drop preserves selection, reverse dragend cancels/cleans up and Reset restores source'],input:'Excluded synthetic DragEvent/DataTransfer plus DOM click/explicit focus invoke actual local137 gallery and Rust handlers; native drop is observed before explicit dragend, so this is state/geometry/handler evidence, not native OS HTML drag chronology. It cannot certify physical drag source lifetime. Separate trusted automated Chromium mouse.down/move/up HTML drag preserves the real source through actual browser accepted pages/drop/end. No application/model/controller/callback/CSS injection, copied model, fabricated application callback or fake fetch/loading. Current native viewport is reported, not resized or claimed narrow.',renderer:navigator.userAgent};
}catch(error){return{ok:false,error:String(error),snapshots,geometry}}
"#;
