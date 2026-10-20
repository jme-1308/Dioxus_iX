//! Excluded actual-gallery managed inspection groups and bounded Linux DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/inspection_groups.rs"]
mod gallery_groups;

#[component]
pub fn InspectionGroupsPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_groups::STYLES} }
        IxTheme { class: "inspection-groups-review",
            h1 { "Linked inspection groups" }
            Button { id: "inspection-groups-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "inspection-groups-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_groups::InspectionGroupsExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_INSPECTION_GROUPS_CHECK);
                        if let Err(error) = query.send("inspectiongroups") {
                            eprintln!("Inspection groups native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native inspection groups: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native inspection groups failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Inspection groups native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.inspection-groups-review{box-sizing:border-box;min-height:100vh;padding:24px}.inspection-groups-review>h1{font-size:28px;margin:0 0 24px}.inspection-groups-review h2{font-size:22px;margin:24px 0 16px}.inspection-groups-review p{max-width:72ch;line-height:1.5}#inspection-groups-frame{margin-top:24px}@media(max-width:600px){.inspection-groups-review{padding:16px}}";

// Excluded driver JS sends synthetic events and observes actual managed rows,
// review state, native controls and retained preparation. Untrusted Enter/Space
// do not activate native button defaults: permission is checked, then one actual
// DOM click is used. Targets use normal page scroll before programmatic focus;
// no artificial scrollport, production JS or source event chronology oracle.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_INSPECTION_GROUPS_CHECK: &str = r#"
let snapshots={},layout={},keyPermissions={},hitFacts={},failureGeometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const host=()=>get('inspection-groups'),group=()=>get('inspection-group'),result=()=>get('inspection-result');
 const header=()=>group()?.querySelector('button.ix-group-header'),headerRow=()=>group()?.querySelector('.ix-group-header-row');
 const expand=()=>group()?.querySelector('button.ix-group-expand'),panel=()=>get(expand()?.getAttribute('aria-controls'));
 const opened=()=>expand()?.getAttribute('aria-expanded')==='true';
 const entry=id=>group()?.querySelector('.ix-group-entry[data-group-item="'+id+'"]');
 const row=id=>entry(id)?.querySelector('button.ix-group-item');
 const secondary=id=>row(id)?.querySelector('.ix-group-secondary')?.textContent.trim();
 const entries=()=>[...group()?.querySelectorAll('.ix-group-entry[data-group-item]')??[]],order=()=>entries().map(node=>node.dataset.groupItem);
 const note=()=>get('inspection-note'),toolkit=()=>get('inspection-toolkit'),selection=()=>get('inspection-selection');
 const trigger=()=>group()?.querySelector('.inspection-actions[role="button"]'),menu=()=>get(trigger()?.getAttribute('aria-controls'));
 const menuOpen=()=>trigger()?.getAttribute('aria-expanded')==='true'&&menu()&&!menu().hidden;
 const count=name=>Number(result()?.dataset[name]),selected=()=>result()?.dataset.selectedItem;
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
  throw Error(label+'; result='+result()?.textContent+'; selected='+selected()+'; expanded='+opened()+'; menu='+menuOpen()+'; focus='+(active()?.getAttribute('aria-label')||active()?.id||active()?.tagName));
 };
 const key=(node,key)=>{const event=new KeyboardEvent('keydown',{key,code:key===' '?'Space':key,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' native focus established',()=>active()===node)};
 const select=async(id,desired=id)=>{
  const before=count('itemRequests');row(id).click();await wait('native managed '+id+' selection updates actual controlled state',()=>count('itemRequests')===before+1&&selected()===desired&&(desired==='none'?!row(id).getAttribute('aria-pressed')||row(id).getAttribute('aria-pressed')==='false':row(desired)?.getAttribute('aria-pressed')==='true')&&headerRow().getAttribute('data-item-selected')===(desired==='none'?null:desired));
 };
 const setExpanded=async desired=>{if(opened()!==desired)expand().click();await wait('actual controlled group expansion '+desired,()=>opened()===desired&&panel().hidden===!desired&&result().dataset.expanded===String(desired))};
 const setNote=async text=>{
  const node=note();Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,text);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));
  await wait('actual local inspection preparation note applied',()=>note()===node&&note().value===text);
 };
 const hitTestable=(node,insideFrame=true)=>{
  const box=node?.getBoundingClientRect(),frame=get('inspection-groups-frame')?.getBoundingClientRect(),hit=box?document.elementFromPoint(box.left+box.width/2,box.top+box.height/2):null;
  failureGeometry={node:node?.id,active:active()?.id,nodeRect:box?{left:box.left,right:box.right,top:box.top,bottom:box.bottom,width:box.width,height:box.height}:null,frameRect:frame?{left:frame.left,right:frame.right,width:frame.width}:null,innerWidth,innerHeight,scrollTop:document.documentElement.scrollTop,hit:hit?{id:hit.id,tag:hit.tagName,className:hit.className,label:hit.getAttribute('aria-label')}:null};
  return !!box&&box.width>0&&box.height>0&&(!insideFrame||(box.left>=frame.left-1&&box.right<=frame.right+1))&&box.left>=0&&box.right<=innerWidth+1&&box.top>=0&&box.bottom<=innerHeight+1&&(hit===node||node.contains(hit));
 };
 const openMenu=async()=>{
  await focus(trigger(),'shared inspection Actions trigger');trigger().click();await wait('actual shared inspection menu opened in native top layer',()=>menuOpen()&&menu().matches(':popover-open'));
 };
 const choose=async(id,reviewedCount)=>{
  const before=count('reviewActions'),requests=count('requests'),itemRequests=count('itemRequests'),headerRequests=count('headerRequests');
  const node=get(id);assert(menuOpen()&&node,'actual shared review menu item exists');node.click();
  await wait('shared menu action updates actual asset review records',()=>!menuOpen()&&count('reviewActions')===before+1&&count('reviewedCount')===reviewedCount);
  assert(count('requests')===requests&&count('itemRequests')===itemRequests&&count('headerRequests')===headerRequests&&opened(),'shared menu review never selects/requests/collapses Group');
 };
 const snapshot=()=>({selected:selected(),headerSelected:result()?.dataset.headerSelected,expanded:opened(),menu:menuOpen(),requests:count('requests'),itemRequests:count('itemRequests'),headerRequests:count('headerRequests'),expandRequests:count('expandRequests'),reviewed:count('reviewedCount'),reviewActions:count('reviewActions'),order:order(),states:entries().map(node=>({id:node.dataset.groupItem,text:secondary(node.dataset.groupItem),pressed:row(node.dataset.groupItem)?.getAttribute('aria-pressed')})),selection:selection()?.textContent.trim(),note:note()?.value,toolkit:toolkit()?.checked,held:checked(get('inspection-hold')),veto:checked(get('inspection-veto')),headerExpand:checked(get('inspection-header-expand')),suppressHeader:checked(get('inspection-suppress-header')),disabledPump:checked(get('inspection-disable-pump'))});

 await wait('actual managed inspection group and retained footer mounted',()=>host()&&group()&&header()&&expand()&&panel()&&note()&&toolkit()&&trigger()&&result()&&entries().length===4);
 await wait('native WebView ready for focus',()=>document.hasFocus());
 assert(order().join(',')==='motor,pump,fan,archive'&&opened()&&!panel().hidden&&selected()==='none'&&result().dataset.headerSelected==='false'&&selection().textContent==='No asset selected'&&['requests','itemRequests','headerRequests','expandRequests','reviewedCount','reviewActions'].every(name=>count(name)===0),'initial actual asset records and accepted/attempted state');
 assert(header().getAttribute('aria-label')==='Select inspection group'&&expand().getAttribute('aria-label')==='Toggle inspection assets'&&header().getAttribute('aria-pressed')==='false'&&panel().getAttribute('role')==='region'&&panel().getAttribute('aria-labelledby')===header().id&&expand().getAttribute('aria-controls')===panel().id,'actual native group labels and stable disclosure/region relationships');
 assert(!header().contains(get('inspection-members'))&&!header().contains(trigger())&&!header().contains(note())&&get('inspection-members').textContent==='3 active'&&entries().every(node=>row(node.dataset.groupItem)?.tagName==='BUTTON'&&secondary(node.dataset.groupItem)==='Needs review'),'managed item and independent header/menu/footer composition');
 assert(row('archive').disabled&&get('inspection-review-selected').disabled&&note().value==='Prepare line inspection'&&!toolkit().checked,'archived managed row guarded and actual preparation defaults');
 row('archive').click();get('inspection-review-selected').click();await wait('disabled archive and missing-selection review cannot mutate requests/model',()=>count('requests')===0&&count('reviewActions')===0&&selected()==='none');snapshots.semantics=snapshot();

 for(const [value,desired] of [['Enter','motor'],[' ','none']]){
  await focus(row('motor'),'native managed Motor row');await wait('actual managed row receives pointer hit',()=>hitTestable(row('motor')));hitFacts.row=failureGeometry.hit;
  const before=count('requests'),event=key(row('motor'),value);keyPermissions['row'+value]=!event.defaultPrevented;assert(!event.defaultPrevented,'native row '+value+' default remains available');await wait('untrusted row key does not fabricate group request',()=>count('requests')===before);await select('motor',desired);
 }
 await select('pump');const reviewBefore=count('reviewActions'),itemBeforeReview=count('itemRequests');get('inspection-review-selected').click();await wait('selected review changes real Pump record and selected detail',()=>count('reviewActions')===reviewBefore+1&&count('reviewedCount')===1&&secondary('pump')==='Reviewed'&&selection().textContent==='Pump 002 · Reviewed');
 assert(count('itemRequests')===itemBeforeReview,'footer review does not select another row');
 await focus(header(),'native group header selection');const headerKey=key(header(),'Enter');keyPermissions.headerEnter=!headerKey.defaultPrevented;assert(!headerKey.defaultPrevented,'native header Enter default remains available');
 const beforeHeader={requests:count('requests'),items:count('itemRequests'),headers:count('headerRequests')};header().click();
 await wait('accepted header selection atomically clears current managed row',()=>result().dataset.headerSelected==='true'&&selected()==='none'&&header().getAttribute('aria-pressed')==='true'&&row('pump').getAttribute('aria-pressed')==='false'&&!headerRow().hasAttribute('data-item-selected')&&count('requests')===beforeHeader.requests+1&&count('itemRequests')===beforeHeader.items+1&&count('headerRequests')===beforeHeader.headers+1);
 await select('fan');assert(result().dataset.headerSelected==='true','item selection does not invent header-selected mutation');header().click();await wait('header deselection clears current Fan selection normally',()=>result().dataset.headerSelected==='false'&&selected()==='none'&&get('inspection-review-selected').disabled);snapshots.selection=snapshot();

 const options=get('inspection-options');if(!options.open)options.querySelector('summary').click();await wait('native Group options opened',()=>options.open);
 get('inspection-hold').click();await wait('controlled item selection held by host',()=>checked(get('inspection-hold')));
 const held={items:count('itemRequests'),requests:count('requests')};row('motor').click();await wait('held accepted item request does not alter authoritative displayed key',()=>count('itemRequests')===held.items+1&&count('requests')===held.requests+1&&selected()==='none'&&row('motor').getAttribute('aria-pressed')==='false'&&selection().textContent==='No asset selected');
 get('inspection-hold').click();await wait('normal controlled item apply restored',()=>!checked(get('inspection-hold')));await select('motor');
 get('inspection-veto').click();await wait('request veto enabled',()=>checked(get('inspection-veto')));
 const vetoed={requests:count('requests'),items:count('itemRequests'),headers:count('headerRequests'),expands:count('expandRequests')};row('pump').click();await wait('vetoed row attempt records without accepted selection callback',()=>count('requests')===vetoed.requests+1&&count('itemRequests')===vetoed.items&&selected()==='motor');
 header().click();await wait('vetoed header attempt has no accepted callback/clear',()=>count('requests')===vetoed.requests+2&&count('headerRequests')===vetoed.headers&&count('itemRequests')===vetoed.items&&selected()==='motor');
 expand().click();await wait('vetoed disclosure attempt preserves actual open region',()=>count('requests')===vetoed.requests+3&&count('expandRequests')===vetoed.expands&&opened());
 await focus(note(),'native footer during veto');key(note(),'Escape');await wait('vetoed Escape leaves actual group and native focus unchanged',()=>count('requests')===vetoed.requests+4&&count('expandRequests')===vetoed.expands&&opened()&&active()===note());
 get('inspection-veto').click();await wait('veto released',()=>!checked(get('inspection-veto')));get('inspection-disable-pump').click();await wait('live Pump disabled option reaches actual row/member count',()=>row('pump').disabled&&get('inspection-members').textContent==='2 active');
 const disabledAttempts=count('requests');row('pump').click();await wait('disabled managed row rejects before change request',()=>count('requests')===disabledAttempts&&selected()==='motor');get('inspection-disable-pump').click();await wait('Pump restored to native managed membership',()=>!row('pump').disabled&&get('inspection-members').textContent==='3 active');snapshots.heldVeto=snapshot();

 const retainedNote=note(),retainedToolkit=toolkit();await focus(note(),'actual inspection preparation note');await setNote('Prepare drive inspection');toolkit().click();await wait('actual local toolkit choice applied',()=>toolkit().checked);await select('pump');
 const beforeReorder=count('itemRequests');get('inspection-reverse').click();await wait('actual reverse preserves stable selected Pump and retained footer',()=>order().join(',')==='archive,fan,pump,motor'&&selected()==='pump'&&row('pump').getAttribute('aria-pressed')==='true'&&count('itemRequests')===beforeReorder&&note()===retainedNote&&note().value==='Prepare drive inspection'&&toolkit().checked);
 const extra=get('inspection-extra');extra.click();await wait('actual new Sensor entry mounted with fresh review state',()=>row('sensor')&&secondary('sensor')==='Needs review'&&get('inspection-members').textContent==='4 active'&&get('inspection-extra')===extra);await select('sensor');
 const removedSensor=entry('sensor'),beforeRemove={items:count('itemRequests'),requests:count('requests')};extra.click();await wait('removing selected Sensor clears host selection without phantom accepted callback',()=>!entry('sensor')&&!removedSensor.isConnected&&selected()==='none'&&count('itemRequests')===beforeRemove.items&&count('requests')===beforeRemove.requests&&selection().textContent==='No asset selected');
 extra.click();await wait('readded Sensor does not resurrect old selection and footer is retained',()=>entry('sensor')&&entry('sensor')!==removedSensor&&selected()==='none'&&row('sensor').getAttribute('aria-pressed')==='false'&&secondary('sensor')==='Needs review'&&note()===retainedNote&&note().value==='Prepare drive inspection'&&toolkit()===retainedToolkit&&toolkit().checked);snapshots.reorderMembership=snapshot();

 await focus(expand(),'native group disclosure');const expandKey=key(expand(),' ');keyPermissions.disclosureSpace=!expandKey.defaultPrevented;assert(!expandKey.defaultPrevented&&opened(),'native disclosure Space default remains available');await setExpanded(false);
 assert(panel().hidden&&note().isConnected&&note().getClientRects().length===0&&note().value==='Prepare drive inspection','closed actual native region hides but retains footer state');note().focus();assert(active()===expand(),'closed retained footer cannot take native focus');
 await setExpanded(true);await focus(note(),'native retained footer before accepted Escape');for(const value of ['Home','ArrowLeft',' ']){const event=key(note(),value);keyPermissions['note'+value]=!event.defaultPrevented;assert(!event.defaultPrevented,'native footer '+value+' default remains available')}
 const escapeBefore=count('expandRequests');key(note(),'Escape');await wait('accepted Group Escape closes actual region and focuses mounted disclosure',()=>!opened()&&panel().hidden&&count('expandRequests')===escapeBefore+1&&active()===expand());await setExpanded(true);
 assert(note()===retainedNote&&toolkit()===retainedToolkit&&note().value==='Prepare drive inspection'&&toolkit().checked,'normal disclosure/Escape retains local footer state');snapshots.disclosure=snapshot();

 await openMenu();await wait('actual shared Review all item receives pointer hit',()=>hitTestable(get('inspection-review-all'),false));hitFacts.menu=failureGeometry.hit;await choose('inspection-review-all',4);
 assert(['motor','pump','fan','sensor'].every(id=>secondary(id)==='Reviewed')&&secondary('archive')==='Needs review','shared Review all changes actual nonarchived records only');await openMenu();await choose('inspection-clear-reviews',0);assert(entries().every(node=>secondary(node.dataset.groupItem)==='Needs review'),'shared Clear reviews restores actual record state');
 await openMenu();const menuEscape={requests:count('requests'),expands:count('expandRequests')};await focus(trigger(),'current owned Actions trigger');key(trigger(),'Escape');
 await wait('first Escape closes only the actual shared menu and returns its trigger',()=>!menuOpen()&&opened()&&count('requests')===menuEscape.requests&&count('expandRequests')===menuEscape.expands&&active()===trigger());
 key(trigger(),'Escape');await wait('second Escape closes Group and returns its disclosure',()=>!opened()&&count('expandRequests')===menuEscape.expands+1&&active()===expand());await setExpanded(true);snapshots.sharedMenuEscape=snapshot();

 get('inspection-header-expand').click();await wait('combined header selection/expansion option enabled',()=>checked(get('inspection-header-expand'))&&header().getAttribute('aria-expanded')==='true');
 const combined={requests:count('requests'),headers:count('headerRequests'),expands:count('expandRequests')};header().click();await wait('combined header requests selection and expansion independently',()=>!opened()&&result().dataset.headerSelected==='true'&&count('requests')===combined.requests+2&&count('headerRequests')===combined.headers+1&&count('expandRequests')===combined.expands+1);
 header().click();await wait('second combined header restores open/deselected state',()=>opened()&&result().dataset.headerSelected==='false');get('inspection-header-expand').click();get('inspection-suppress-header').click();await wait('suppressed selection exposes native header disclosure only',()=>checked(get('inspection-suppress-header'))&&!header().hasAttribute('aria-pressed')&&header().getAttribute('aria-label')==='Toggle inspection assets'&&header().getAttribute('aria-expanded')==='true');
 const suppressed={headers:count('headerRequests'),items:count('itemRequests'),expands:count('expandRequests')};header().click();await wait('header-as-disclosure closes without selection callbacks',()=>!opened()&&count('headerRequests')===suppressed.headers&&count('itemRequests')===suppressed.items&&count('expandRequests')===suppressed.expands+1);header().click();await wait('header-as-disclosure opens actual retained region',()=>opened()&&count('headerRequests')===suppressed.headers);
 get('inspection-suppress-header').click();await wait('normal native header selection restored',()=>!checked(get('inspection-suppress-header'))&&header().getAttribute('aria-label')==='Select inspection group'&&header().getAttribute('aria-pressed')==='false');assert(note()===retainedNote&&note().value==='Prepare drive inspection'&&toolkit().checked,'header options retain footer and selected model boundaries');snapshots.headerOptions=snapshot();

 await openMenu();const resetButton=get('inspection-reset'),oldGroup=group(),oldNote=note();await focus(resetButton,'native Restore inspection group');resetButton.click();
 await wait('Restore resets actual records/accepted states/options/counters and preparation',()=>order().join(',')==='motor,pump,fan,archive'&&opened()&&!menuOpen()&&selected()==='none'&&result().dataset.headerSelected==='false'&&['requests','itemRequests','headerRequests','expandRequests','reviewedCount','reviewActions'].every(name=>count(name)===0)&&entries().every(node=>secondary(node.dataset.groupItem)==='Needs review')&&note()?.value==='Prepare line inspection'&&!toolkit().checked&&get('inspection-review-selected').disabled&&!checked(get('inspection-hold'))&&!checked(get('inspection-veto'))&&!checked(get('inspection-header-expand'))&&!checked(get('inspection-suppress-header'))&&!checked(get('inspection-disable-pump'))&&row('archive').disabled&&!row('pump').disabled);
 assert(!oldGroup.isConnected&&!oldNote.isConnected&&group()!==oldGroup&&note()!==oldNote&&get('inspection-reset')===resetButton&&get('inspection-options')===options&&get('inspection-extra')===extra&&active()===resetButton,'Restore replaces only owned Group/preparation and preserves toolbar/options/native focus');snapshots.reset=snapshot();

 get('inspection-groups-narrow').click();await wait('340px inspection group frame without horizontal page overflow',()=>get('inspection-groups-frame').getBoundingClientRect().width<=341&&document.documentElement.scrollWidth<=document.documentElement.clientWidth+1);
 for(const [label,node] of [['header',header()],['row',row('motor')],['disclosure',expand()],['menu trigger',trigger()]]){await focus(node,'narrow native '+label);await wait('narrow actual '+label+' fits and receives pointer hit',()=>hitTestable(node));hitFacts['narrow '+label]=failureGeometry.hit}
 await select('motor');await focus(note(),'narrow actual inspection preparation');await wait('narrow native preparation input fits and receives pointer hit',()=>hitTestable(note()));await setNote('Narrow inspection preparation');toolkit().click();await wait('narrow local toolkit checkbox updated',()=>toolkit().checked);
 await focus(get('inspection-review-selected'),'narrow native selected review');await wait('narrow footer review action fits and receives pointer hit',()=>hitTestable(get('inspection-review-selected')));get('inspection-review-selected').click();await wait('narrow selected review updates actual Motor record/detail',()=>count('reviewActions')===1&&count('reviewedCount')===1&&secondary('motor')==='Reviewed'&&selection().textContent==='Motor 001 · Reviewed');
 await openMenu();await wait('narrow actual shared menu action is visible and hit-testable',()=>hitTestable(get('inspection-clear-reviews'),false));await choose('inspection-clear-reviews',0);assert(secondary('motor')==='Needs review'&&selection().textContent==='Motor 001 · Needs review','narrow shared action updates actual selected record');
 await setExpanded(false);await setExpanded(true);
 const frame=get('inspection-groups-frame').getBoundingClientRect(),input=note().getBoundingClientRect();layout={frameWidth:frame.width,groupWidth:group().getBoundingClientRect().width,rowHeight:row('motor').getBoundingClientRect().height,inputWidth:input.width,inputHeight:input.height,pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};
 assert(selected()==='motor'&&result().dataset.headerSelected==='false'&&count('itemRequests')===1&&count('headerRequests')===0&&count('expandRequests')===2&&count('reviewActions')===2&&count('reviewedCount')===0&&note().value==='Narrow inspection preparation'&&toolkit().checked&&layout.groupWidth<=341&&layout.pageScroll<=layout.pageClient+1,'narrow managed rows/menu/footer and retained disclosure produce real contained workflow');snapshots.narrow=snapshot();
 return {ok:true,snapshots,layout,keyPermissions,hitFacts,
  checks:['actual managed native rows/labels/ARIA/disabled and independent regions','native key permission plus managed toggle/header-clear and actual selected-record review','held authoritative item state/vetoed attempts/disabled rejection without accepted callbacks','stable-key reorder/add/remove/readd preserve preparation and prevent selection resurrection','native disclosure/Escape focus and hidden retained local footer state','shared review/clear changes actual models and first Escape belongs only to menu','combined header selection/expansion and header-as-disclosure options stay independent','Restore resets owned records/Group/preparation/options/counts and preserves toolbar focus','340px real header/row/menu/disclosure/footer pointer hits and contained retained workflow'],
  input:'synthetic form/click/key events, explicit native focus and normal page scroll setup; native Enter/Space/Tab defaults are not simulated',
  trustedInput:'separate Chromium native Enter/Space/Tab/typing/pointer interactions and visual checks; no physical-device certification',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,keyPermissions,hitFacts,failureGeometry,documentFocusedNow:document.hasFocus()}}
"#;
