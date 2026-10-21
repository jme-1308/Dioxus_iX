//! Excluded actual-gallery shared cell help and ordinary editing workflow.
use super::grid_worklist_review::gallery_worklist;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn GridHelpPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_worklist::STYLES} }
        IxTheme { class:"grid-help-review",
            gallery_worklist::GridWorklistExample {}
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=crate::dom_query::eval(NATIVE_GRID_HELP_CHECK);let _=query.send("gridhelp");
                    match query.join::<serde_json::Value>().await {
                        Ok(value)if value["ok"]==true=>{println!("Dioxus native Grid help: {value}");window.close();}
                        value=>{eprintln!("Dioxus native Grid help failed: {value:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(40)).await;eprintln!("Grid help native replay timed out");std::process::exit(1);});}
            } }
        }
    }
}
const STYLES: &str = "body{margin:0}.grid-help-review{box-sizing:border-box;min-height:100vh;padding:24px}@media(max-width:600px){.grid-help-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_GRID_HELP_CHECK: &str = r#"
let snapshots={};
try{
 await dioxus.recv();
 const get=id=>document.getElementById('worklist-'+id),host=()=>document.getElementById('grid-worklist'),grid=()=>host()?.querySelector('[role="grid"][aria-label="Maintenance worklist"]'),row=id=>grid()?.querySelector('[data-row-id="'+id+'"]'),cell=(id,col)=>row(id)?.querySelector('[role="gridcell"][data-column-id="'+col+'"]'),editor=()=>host()?.querySelector('.ix-data-grid-editor:not([readonly])'),panels=()=>[...document.querySelectorAll('.ix-data-grid-cell-help')],open=()=>panels().filter(n=>n.dataset.open==='true');
 const assert=(v,label)=>{if(!v)throw Error(label)},hold=ms=>new Promise(r=>setTimeout(r,ms)),wait=async(label,predicate)=>{for(let i=0;i<200;i++){if(predicate())return;await hold(20)}throw Error(label+'; state='+get('status')?.outerHTML)};
 const key=(n,k,ctrl=false)=>n.dispatchEvent(new KeyboardEvent('keydown',{key:k,code:k,ctrlKey:ctrl,bubbles:true,cancelable:true})),hover=n=>n.dispatchEvent(new MouseEvent('mouseenter',{bubbles:false})),leave=n=>n.dispatchEvent(new MouseEvent('mouseleave',{bubbles:false}));
 const panel=n=>(n?.getAttribute('aria-describedby')||'').split(/\s+/).map(id=>document.getElementById(id)).find(n=>n?.classList.contains('ix-data-grid-cell-help'));
 const help=async(n,text)=>{await wait('current mounted shared help '+text,()=>panel(n)?.dataset.open==='true'&&panel(n)?.dataset.positioned==='true'&&panel(n)?.matches(':popover-open')&&panel(n)?.textContent.trim()===text);assert(!n.hasAttribute('title'),'no title fallback');return{id:panel(n).id,text:panel(n).textContent.trim(),placement:panel(n).dataset.placement}};
 const focus=async n=>{n.focus();await wait('current grid cell native focus',()=>document.activeElement===n)};
 const setValue=async(n,text)=>{Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(n,text);n.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait('current editor draft',()=>n.value===text)};
 await wait('actual worklist mounted',()=>cell('worklist-000','asset'));grid().scrollIntoView({block:'center'});hover(cell('worklist-000','asset'));snapshots.hover=await help(cell('worklist-000','asset'),'Asset 000');leave(cell('worklist-000','asset'));await wait('pointer leave retires visible help',()=>open().length===0);
 await focus(cell('worklist-000','asset'));key(cell('worklist-000','asset'),'ArrowRight');await wait('ArrowRight focuses actual Status cell',()=>document.activeElement===cell('worklist-000','status'));snapshots.status=await help(cell('worklist-000','status'),'Normal operation. No maintenance action is marked for this asset.');const statusPanel=panel(cell('worklist-000','status')),r=statusPanel.getBoundingClientRect(),g=grid().getBoundingClientRect();snapshots.edge={panel:{left:r.left,right:r.right,top:r.top,bottom:r.bottom},grid:{left:g.left,right:g.right,top:g.top,bottom:g.bottom},viewport:{width:innerWidth,height:innerHeight},nativeTopLayer:statusPanel.matches(':popover-open')};assert(r.left>=-1&&r.right<=innerWidth+1&&r.top>=-1&&r.bottom<=innerHeight+1,'shared help fits actual viewport');assert(r.top<g.top||r.bottom>g.bottom||r.left<g.left||r.right>g.right,'near-edge help escapes grid scroll clipping in native top layer');key(cell('worklist-000','status'),'Escape');await wait('Escape dismisses focused help',()=>statusPanel.dataset.open==='false');await focus(cell('worklist-001','owner'));snapshots.readonly=await help(cell('worklist-001','owner'),'Team 2 owns this asset. Its assignment is locked in this worklist.');assert(cell('worklist-001','owner').getAttribute('aria-readonly')==='true','readonly owner semantics retained');
 await focus(cell('worklist-000','asset'));key(cell('worklist-000','asset'),'F2');await wait('actual F2 editor active without grid help panels',()=>editor()&&document.activeElement===editor()&&panels().length===0);await setValue(editor(),'Inspection asset 000');hover(cell('worklist-001','status'));await hold(750);assert(editor()?.value==='Inspection asset 000'&&panels().length===0,'other-cell hover cannot show help during active editor');leave(cell('worklist-001','status'));key(editor(),'Enter');await wait('Enter commits and returns real cell focus',()=>!editor()&&get('status').dataset.saved==='1'&&document.activeElement===cell('worklist-000','asset'));snapshots.committed=await help(cell('worklist-000','asset'),'Inspection asset 000');key(cell('worklist-000','asset'),'F2');await wait('fresh F2 editor',()=>editor()&&panels().length===0);await setValue(editor(),'Cancelled draft');key(editor(),'Escape');await wait('Escape retires editor preserving saved count',()=>!editor()&&get('status').dataset.saved==='1');snapshots.cancelled=await help(cell('worklist-000','asset'),'Inspection asset 000');
 key(cell('worklist-000','asset'),'End',true);await wait('CtrlEnd mounted last actual row and reveals scroll',()=>document.activeElement===cell('worklist-095','ticket')&&grid().scrollTop>0);snapshots.revealed=await help(cell('worklist-095','ticket'),'IX-00095');assert(grid().getBoundingClientRect().bottom>=cell('worklist-095','ticket').getBoundingClientRect().bottom-1,'native keyboard reveal keeps current cell visible');
 get('pagination').click();await wait('actual first12-row page',()=>get('pagination-status').dataset.page==='0'&&get('pagination-status').dataset.pageSize==='12'&&cell('worklist-000','asset'));row('worklist-001').querySelector('[role="checkbox"]').click();await wait('actual row selected',()=>get('status').dataset.selected==='1');await focus(cell('worklist-000','asset'));await help(cell('worklist-000','asset'),'Inspection asset 000');const retired=panel(cell('worklist-000','asset')).id;const next=host().querySelector('nav[aria-label="Maintenance worklist pages"] button[aria-label="Next page"]');next.focus();next.click();await wait('new page retires previous cell and owned panel',()=>get('pagination-status').dataset.page==='1'&&!cell('worklist-000','asset')&&!document.getElementById(retired)&&cell('worklist-012','asset'));await focus(cell('worklist-012','asset'));snapshots.paged=await help(cell('worklist-012','asset'),'Asset 012');get('csv-generate').click();await wait('CSV full96 and cross-page selection retained',()=>get('csv-status').dataset.rows==='96'&&get('status').dataset.selected==='1');
 return{ok:true,snapshots,checks:['shared native top-layer help follows actual pointer hover/leave and committed fallback text','ArrowRight focuses Status override; near-edge panel escapes grid clipping within viewport; Escape dismisses and readonly owner reason is explicit','any active editor removes all help panels and other-cell hover cannot reopen them; Enter restores current committed help','Escape cancels current draft while retaining committed value/current focus/help','actual CtrlEnd keyboard navigation reveals last native row and supplies matching Ticket help','pagination retires prior cell-owned panel without orphan; current page help/navigation, selection and CSV full96 remain intact'],input:'Excluded synthetic mouseenter/mouseleave, DOM focus/click/key/input exercise actual shared gallery handlers and native mounted layout/popovers. A settled observation follows ordinary shared hover delay; no exact deadline/event chronology is asserted. Trusted keyboard/pointer input is separate Chromium evidence; no application state/controller/callback/CSS injection.',renderer:navigator.userAgent};
}catch(error){return{ok:false,error:String(error),snapshots,state:document.getElementById('worklist-status')?.outerHTML}}
"#;
