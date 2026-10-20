//! Excluded actual-gallery pagination workflow and bounded Linux DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/paged_equipment.rs"]
mod gallery_pages;

#[component]
pub fn PaginationWorkflowPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_pages::STYLES} }
        IxTheme { class: "pagination-workflow-review",
            h1 { "Dioxus equipment register pagination" }
            Button { id: "pagination-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "pagination-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_pages::PagedEquipmentExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_PAGINATION_WORKFLOW_CHECK);
                        if let Err(error) = query.send("paginationworkflow") {
                            eprintln!("Pagination workflow native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native pagination workflow: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native pagination workflow failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Pagination workflow native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.pagination-workflow-review{box-sizing:border-box;min-height:100vh;padding:24px}.pagination-workflow-review h1{font-size:28px;margin:0 0 24px}.pagination-workflow-review h2{font-size:22px;margin:24px 0 16px}.pagination-workflow-review h3{font-size:18px}.pagination-workflow-review p{max-width:72ch;line-height:1.5}#pagination-frame{margin-top:24px}@media(max-width:600px){.pagination-workflow-review{padding:16px}}";

// Excluded native driver: actual gallery owns rows, state and all handlers. DOM
// click executes native button activation; synthetic Enter/change/input events
// exercise the handler contract. They do not fabricate trusted Enter/Space/Tab,
// native blur-to-change behavior, number editing or select keyboard defaults.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_PAGINATION_WORKFLOW_CHECK: &str = r#"
let snapshots={},layout={},nativeInputFacts={},keyPermissions={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const host=()=>get('paged-equipment'),result=()=>get('paged-result');
 const pager=()=>get('paged-navigation')?.querySelector('nav.ix-pagination');
 const table=()=>host()?.querySelector('table');
 const rows=()=>[...table()?.querySelectorAll('tbody [data-equipment-id]')??[]];
 const ids=()=>rows().map(node=>node.dataset.equipmentId);
 const pageInput=()=>pager()?.querySelector('input.ix-pagination-input');
 const sizeInput=()=>pager()?.querySelector('select');
 const previous=()=>pager()?.querySelector('button.ix-layout-icon-button');
 const next=()=>[...pager()?.querySelectorAll('button.ix-layout-icon-button')??[]].at(-1);
 const pageButtons=()=>[...pager()?.querySelectorAll('button.ix-page-button')??[]];
 const current=()=>pager()?.querySelector('button[aria-current="page"]');
 const jump=()=>pageButtons().find(node=>/Jump to page|Zu Seite/.test(node.getAttribute('aria-label')||''));
 const data=name=>result()?.dataset[name],number=name=>Number(data(name));
 const rendered=node=>!!node&&node.isConnected&&node.getClientRects().length>0&&!node.closest('[hidden]')&&getComputedStyle(node).visibility!=='hidden';
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const wait=async(label,predicate)=>{
  let stable=0;
  for(let attempt=0;attempt<150;attempt++){
   stable=predicate()?stable+1:0;if(stable===3)return;await hold(20);
  }
  throw Error(label+'; rows='+JSON.stringify(ids())+'; host='+result()?.textContent+'; input='+pageInput()?.value+'; focus='+(active()?.getAttribute('aria-label')||active()?.id||active()?.tagName));
 };
 const focus=async(node,label)=>{assert(node&&rendered(node),label+' exists and is rendered');node.focus();await wait(label+' native focus',()=>active()===node)};
 const click=async(node,label)=>{assert(node&&rendered(node)&&!node.disabled,label+' is an available rendered control');node.click();await hold(30)};
 const key=(node,value)=>{const event=new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const draft=async value=>{const node=pageInput();assert(node&&!node.disabled,'direct page input available');Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait('native page draft applied',()=>pageInput()?.value===value);return node};
 const change=()=>pageInput().dispatchEvent(new Event('change',{bubbles:true}));
 const choice=async(node,value,label)=>{assert(node&&rendered(node)&&[...node.options].some(option=>option.value===value),label+' native option available');node.value=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait(label+' applied',()=>node.isConnected&&node.value===value)};
 const option=async(id,value)=>{if(get(id).checked!==value)await click(get(id),id);await wait(id+' checkbox applied',()=>get(id).checked===value)};
 const expected=(start,count,step=1)=>Array.from({length:count},(_,i)=>'asset-'+(start+i*step));
 const expectRows=async(values,label)=>wait(label,()=>ids().join(',')===values.join(','));
 const snapshot=()=>({page:number('page'),count:number('count'),size:number('size'),total:number('total'),requests:number('requests'),accepted:number('accepted'),rows:ids(),input:pageInput()?.value,landmark:pager()?.getAttribute('aria-label'),current:current()?.getAttribute('aria-label')});
 const reset=async()=>{await click(get('paged-reset'),'Restore register');await expectRows(expected(1,5),'Restore register restores actual first five assets');await wait('Restore register clears modes and request counts',()=>number('page')===0&&number('count')===17&&number('size')===5&&number('total')===83&&number('requests')===0&&number('accepted')===0&&pageInput()?.value==='1'&&get('paged-filter').value==='all'&&get('paged-advanced').checked&&!get('paged-veto').checked&&!get('paged-hold').checked&&!get('paged-german').checked&&!get('paged-hide-size').checked)};

 await wait('actual equipment gallery and advanced pager mounted',()=>host()&&table()&&pager()&&result()&&pageInput()&&sizeInput()&&rows().length===5);
 await wait('WebView ready for native focus',()=>document.hasFocus());
 await expectRows(expected(1,5),'initial table projects actual assets 1 through 5');
 assert(number('page')===0&&number('count')===17&&number('size')===5&&number('total')===83&&pageInput().value==='1','initial zero-based host and one-based input agree');
 assert(table().querySelector('caption')?.textContent.includes('Equipment register')&&rows()[0].textContent.includes('Asset 001')&&rows()[3].querySelector('.ix-pill')?.textContent.includes('Needs service'),'semantic table and actual status content rendered');
 assert(previous().disabled&&!next().disabled&&pageInput().min==='1'&&pageInput().max==='17'&&pageInput().step==='1'&&sizeInput().value==='5','initial native boundaries, integer constraints and page size');
 snapshots.initial=snapshot();

 await focus(next(),'Next page');const enter=key(active(),'Enter');keyPermissions.nextEnter=!enter.defaultPrevented;
 assert(!enter.defaultPrevented,'native Next button retains Enter default permission');await click(next(),'Next page activation');
 await expectRows(expected(6,5),'next projects actual assets 6 through 10');await wait('next accepted once without stealing button focus',()=>number('page')===1&&number('requests')===1&&number('accepted')===1&&active()===next());
 await click(previous(),'Previous page');await expectRows(expected(1,5),'previous returns first five assets');
 const options=get('paged-options');assert(options?.tagName==='DETAILS','pagination options use native details');
 if(!options.open)await click(options.querySelector('summary'),'Pagination options summary');await wait('actual native options open',()=>options.open&&rendered(get('paged-last')));
 await click(get('paged-last'),'Last page');await expectRows(expected(81,3),'external last page shows actual final three assets');
 assert(number('page')===16&&pageInput().value==='17'&&next().disabled&&!previous().disabled,'last-page boundary matches actual remaining data');
 const boundaryRequests=number('requests');next().click();await hold(80);assert(number('requests')===boundaryRequests,'disabled next never requests navigation');
 snapshots.boundaries=snapshot();await reset();

 await focus(pageInput(),'direct page entry');await draft('5');change();
 await expectRows(expected(21,5),'native change commits one-based page 5');await wait('direct change accepted once',()=>number('page')===4&&number('requests')===1&&number('accepted')===1&&pageInput().value==='5');
 key(pageInput(),'Enter');await hold(80);assert(number('requests')===1&&number('accepted')===1,'Enter after change does not duplicate accepted request');
 await draft('7');key(pageInput(),'Enter');await expectRows(expected(31,5),'direct Enter commits page 7');await wait('Enter commits exactly once',()=>number('page')===6&&number('requests')===2&&number('accepted')===2&&pageInput().value==='7');
 snapshots.directEntry=snapshot();
 await draft('999');change();await expectRows(expected(81,3),'high integer clamps actual final page');assert(pageInput().value==='17','high clamp displays authoritative final page');
 await draft('-50');change();await expectRows(expected(1,5),'negative integer clamps actual first page');assert(pageInput().value==='1','negative clamp displays one-based first page');
 snapshots.clamping=snapshot();

 const invalidRequests=number('requests');await draft('');change();await wait('empty draft restores authoritative input without request',()=>pageInput().value==='1'&&number('requests')===invalidRequests);
 await draft('1.5');change();await wait('fractional draft restores authoritative input without request',()=>pageInput().value==='1'&&number('requests')===invalidRequests);
 const invalid=pageInput();Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(invalid,'NaN');nativeInputFacts.nonfiniteSanitized=invalid.value==='';invalid.dispatchEvent(new Event('input',{bubbles:true}));change();await wait('native-sanitized nonfinite draft restores authoritative input',()=>pageInput().value==='1'&&number('requests')===invalidRequests);
 await draft('12');key(pageInput(),'Escape');await wait('Escape cancels direct draft without request',()=>pageInput().value==='1'&&number('requests')===invalidRequests);
 await draft('3');change();await focus(get('paged-filter'),'outside filter after native change');await expectRows(expected(11,5),'explicit native change then blur retains accepted third page');
 assert(pageInput().value==='3'&&number('requests')===invalidRequests+1,'blur after accepted change does not duplicate navigation');
 snapshots.invalidAndEscape=snapshot();

 await option('paged-veto',true);let before=number('requests'),accepted=number('accepted');await focus(pageInput(),'veto direct input');await draft('8');change();
 await wait('veto direct request restores authoritative input and actual rows',()=>number('requests')===before+1&&number('accepted')===accepted&&pageInput().value==='3'&&number('page')===2&&ids().join(',')===expected(11,5).join(','));
 await option('paged-veto',false);await option('paged-hold',true);before=number('requests');accepted=number('accepted');await draft('9');key(pageInput(),'Enter');
 await wait('accepted controlled hold restores authoritative input without changing rows',()=>number('requests')===before+1&&number('accepted')===accepted+1&&pageInput().value==='3'&&number('page')===2&&ids().join(',')===expected(11,5).join(','));
 snapshots.vetoAndHeld=snapshot();await option('paged-hold',false);

 assert([...sizeInput().options].map(node=>node.value).join(',')==='5,10,20','actual native size choices are positive and useful');
 await choice(sizeInput(),'10','ten items per page');await expectRows(expected(1,10),'native page size resets actual projection to first ten');assert(number('page')===0&&number('size')===10&&number('count')===9&&pageInput().value==='1','host owns size and recalculated page count');
 await choice(sizeInput(),'20','twenty items per page');await expectRows(expected(1,20),'larger page size exposes actual first twenty assets');
 snapshots.pageSizes=snapshot();
 await choice(get('paged-filter'),'service','service filter');await expectRows(expected(4,20,4),'service filter projects twenty matching actual IDs');assert(number('total')===20&&number('count')===1&&number('page')===0&&previous().disabled&&next().disabled,'service filter recalculates total and boundary');
 await choice(get('paged-filter'),'empty','no matches filter');await expectRows([],'no matches leaves no actual table data rows');await wait('zero count disables direct/previous/next controls',()=>number('total')===0&&number('count')===0&&pageInput().value==='0'&&pageInput().disabled&&previous().disabled&&next().disabled);
 before=number('requests');previous().click();next().click();await hold(80);assert(number('requests')===before,'zero-count native disabled controls never request');
 snapshots.filtersAndEmpty=snapshot();await reset();

 await option('paged-advanced',false);await wait('basic pager shows actual current page and jump window',()=>!pageInput()&&!sizeInput()&&pageButtons().length>0&&current()?.textContent==='1'&&jump());
 const destination=Number(jump().getAttribute('aria-label').match(/\d+/)?.[0])-1;assert(Number.isInteger(destination)&&destination>0&&destination<17,'jump label provides an actual destination');await click(jump(),'basic jump');
 await expectRows(expected(destination*5+1,Math.min(5,83-destination*5)),'jump label destination matches actual equipment projection');await wait('basic current page reflects accepted jump',()=>number('page')===destination&&current()?.getAttribute('aria-label')==='Page '+(destination+1));
 snapshots.basicJump=snapshot();

 await option('paged-german',true);await wait('basic accessible labels localized',()=>pager().getAttribute('aria-label')==='Geräteseiten'&&previous().getAttribute('aria-label')==='Vorherige Seite'&&next().getAttribute('aria-label')==='Nächste Seite'&&current()?.getAttribute('aria-label')==='Seite '+(number('page')+1)&&jump()?.getAttribute('aria-label').startsWith('Zu Seite '));
 await option('paged-advanced',true);await wait('advanced direct and native size labels localized',()=>pageInput()?.getAttribute('aria-label')==='Seitenauswahl'&&sizeInput()?.getAttribute('aria-label')==='Einträge pro Seite'&&pager().textContent.includes('von'));
 await option('paged-hide-size',true);await wait('hide size removes visible native size control without changing projection',()=>!rendered(sizeInput())&&number('size')===5&&number('page')===destination);await option('paged-hide-size',false);
 snapshots.localizedAndHidden=snapshot();

 await focus(pageInput(),'stale page draft');await draft('2');before=number('requests');await click(get('paged-last'),'external Last page discards draft');
 await expectRows(expected(81,3),'external selected authority reaches actual final data');await wait('external page update replaces stale direct draft without request',()=>pageInput().value==='17'&&number('requests')===before);
 snapshots.externalAuthority=snapshot();await click(get('paged-prune'),'Keep first 12 assets');await expectRows(expected(11,2),'pruning from last page clamps actual projection to final retained assets');
 await wait('external count change clamps selected page and direct constraints',()=>number('total')===12&&number('count')===3&&number('page')===2&&pageInput().value==='3'&&pageInput().max==='3'&&next().disabled&&number('requests')===before);
 snapshots.pruned=snapshot();

 const oldInput=pageInput(),toolbar=get('paged-reset'),optionsNode=get('paged-options');await draft('1');await reset();
 assert(pageInput()!==oldInput&&get('paged-reset')===toolbar&&get('paged-options')===optionsNode&&optionsNode.open,'Restore register remounts pager only and preserves actual options element');snapshots.reset=snapshot();
 await click(get('pagination-narrow'),'narrow preview');await wait('narrow actual register fits frame',()=>get('pagination-frame').getBoundingClientRect().width<=341&&table().getBoundingClientRect().width<=341);
 await focus(pageInput(),'narrow direct page input');const tab=key(active(),'Tab');keyPermissions.directTab=!tab.defaultPrevented;assert(!tab.defaultPrevented,'native direct input retains Tab default permission');
 next().scrollIntoView({block:'center',inline:'nearest'});await hold(50);const box=next().getBoundingClientRect(),hit=document.elementFromPoint(box.left+box.width/2,box.top+box.height/2);assert(hit===next()||next().contains(hit),'narrow Next control is visible and hit-testable');
 await click(next(),'narrow Next page');await expectRows(expected(6,5),'narrow navigation projects actual second page');
 layout={frameWidth:get('pagination-frame').getBoundingClientRect().width,tableWidth:table().getBoundingClientRect().width,pagerWidth:pager().getBoundingClientRect().width,pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};assert(layout.pageScroll<=layout.pageClient+1,'narrow pagination and table remain within page width');snapshots.narrow=snapshot();
 return {ok:true,snapshots,layout,nativeInputFacts,keyPermissions,
  checks:['actual first-five rows, semantic status table and native boundaries','Next/Previous/Last projection and disabled boundary','direct change/Enter commit exactly once','high and negative integer clamping','empty/fractional/sanitized nonfinite/Escape restoration and change then blur','veto and controlled-held authoritative rows/input','native page-size choices reset actual projection','service and empty filtering with zero-count disabled controls','basic current/jump destination matches actual rows','localized accessible labels and hidden size controls','external selected authority discards stale draft','prune clamps final retained rows and page constraints','Restore register clears modes and remounts pager only','narrow native focus/Tab permission and reachable Next navigation'],
  input:'native DOM click, synthetic input/change/key events and explicit native focus; trusted Enter/Space/Tab, number editing, blur-to-change and select defaults are not simulated',
  trustedInput:'separate Chromium actual keyboard activation/traversal, number entry/blur and native select; no physical-device certification',renderer:navigator.userAgent};
}catch(error){return {ok:false,error:String(error),snapshots,layout,nativeInputFacts,keyPermissions,documentFocusedNow:document.hasFocus()}}
"#;
