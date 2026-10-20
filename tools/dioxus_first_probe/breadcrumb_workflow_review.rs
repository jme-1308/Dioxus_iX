//! Excluded actual-gallery breadcrumb workflow and bounded Linux DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/asset_hierarchy.rs"]
mod gallery_hierarchy;

#[component]
pub fn BreadcrumbWorkflowPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_hierarchy::STYLES} }
        IxTheme { class: "breadcrumb-workflow-review",
            h1 { "Dioxus asset hierarchy navigation" }
            Button { id: "breadcrumb-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "breadcrumb-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_hierarchy::AssetHierarchyExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_BREADCRUMB_WORKFLOW_CHECK);
                        if let Err(error) = query.send("breadcrumbworkflow") {
                            eprintln!("Breadcrumb workflow native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native breadcrumb workflow: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native breadcrumb workflow failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Breadcrumb workflow native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.breadcrumb-workflow-review{box-sizing:border-box;min-height:100vh;padding:24px}.breadcrumb-workflow-review h1{font-size:28px;margin:0 0 24px}.breadcrumb-workflow-review h2{font-size:22px;margin:24px 0 16px}.breadcrumb-workflow-review h3{font-size:18px}.breadcrumb-workflow-review p{max-width:72ch;line-height:1.5}#breadcrumb-frame{margin-top:24px}@media(max-width:600px){.breadcrumb-workflow-review{padding:16px}}";

// Excluded driver only: actual Breadcrumb/Dropdown and host Rust state handle
// navigation. Synthetic keys invoke handlers, not trusted browser defaults;
// native focus and actual rendered menus/panels are checked independently of
// host counters. Chromium separately exercises real Enter/Space/Tab and typing.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_BREADCRUMB_WORKFLOW_CHECK: &str = r#"
let snapshots={},layout={},keyPermissions={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const host=()=>get('asset-hierarchy'),result=()=>get('hierarchy-result');
 const crumb=()=>get('hierarchy-preview')?.querySelector('nav.ix-breadcrumb');
 const heading=()=>get('hierarchy-heading')?.textContent.trim();
 const current=()=>[...crumb()?.querySelectorAll('[aria-current="page"]')??[]];
 const triggers=()=>[...crumb()?.querySelectorAll('[aria-haspopup="menu"][aria-controls]')??[]];
 const trigger=kind=>crumb()?.querySelector('[data-breadcrumb-menu="'+kind+'"][aria-haspopup="menu"][aria-controls]');
 const menu=kind=>get(trigger(kind)?.getAttribute('aria-controls'));
 const opened=kind=>trigger(kind)?.getAttribute('aria-expanded')==='true'&&rendered(menu(kind));
 const item=(kind,label)=>menu(kind)?.querySelector('[role="menuitem"][data-ix-dropdown-label="'+label+'"]');
 const ancestor=label=>[...crumb()?.querySelectorAll('.ix-breadcrumb-item > button,.ix-breadcrumb-item > a')??[]].find(node=>node.textContent.trim()===label);
 const rendered=node=>!!node&&node.isConnected&&node.getClientRects().length>0&&!node.closest('[hidden]')&&getComputedStyle(node).visibility!=='hidden';
 const descendants=()=>[...get('hierarchy-children')?.querySelectorAll('[id^="hierarchy-open-"]')??[]].filter(rendered).map(node=>node.id.slice('hierarchy-open-'.length));
 const data=name=>result()?.dataset[name],number=name=>Number(data(name));
 let labels={previous:'Show previous breadcrumb items',next:'Show next breadcrumb items'};
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const wait=async(label,predicate)=>{
  let stable=0;
  for(let attempt=0;attempt<150;attempt++){
   stable=predicate()?stable+1:0;if(stable===3)return;await hold(20);
  }
  throw Error(label+'; current='+heading()+'; path='+data('path')+'; children='+JSON.stringify(descendants())+'; host='+result()?.textContent+'; focus='+(active()?.getAttribute('aria-label')||active()?.id||active()?.tagName));
 };
 const focus=async(node,label)=>{assert(node&&rendered(node),label+' exists and is rendered');node.focus();await wait(label+' native focus',()=>active()===node)};
 const click=async(node,label)=>{assert(node&&rendered(node)&&!node.disabled&&node.getAttribute('aria-disabled')!=='true',label+' available rendered control');node.click();await hold(30)};
 const key=(node,value)=>{const event=new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const option=async(id,value)=>{assert(rendered(get(id)),id+' option is actually rendered');if(get(id).checked!==value)await click(get(id),id);await wait(id+' checked state',()=>get(id).checked===value)};
 const limit=async value=>{const node=get('hierarchy-limit');assert(rendered(node)&&[...node.options].some(option=>option.value===value),'visible limit native option available');node.value=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait('visible levels option applied',()=>node.value===value)};
 const input=async value=>{const node=get('hierarchy-note');assert(rendered(node),'actual maintenance note rendered');Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait('maintenance note controlled value applied',()=>get('hierarchy-note')?.value===value)};
 const at=async(id,label)=>wait(label,()=>data('current')===id&&heading()===({'cell-a':'Cell A',north:'North plant',root:'Facilities',assembly:'Assembly','line-01':'Line 01','motor-001':'Motor 001','motor-002':'Motor 002'}[id])&&current().length===1&&current()[0].textContent.includes(heading()));
 const open=async kind=>{await focus(trigger(kind),kind+' breadcrumb trigger');key(active(),'ArrowDown');await wait(kind+' real Dropdown opens',()=>opened(kind)&&menu(kind).getAttribute('role')==='menu'&&menu(kind).querySelector('[role="menuitem"]'));return trigger(kind)};
 const choose=async(kind,label)=>{await open(kind);await click(item(kind,label),label+' actual menu action')};
 const dismiss=async kind=>{const owner=trigger(kind);key(active(),'Escape');await wait(kind+' Escape dismisses and returns unchanged owner',()=>!opened(kind)&&active()===owner)};
 const snapshot=()=>({current:data('current'),heading:heading(),path:data('path'),count:number('count'),requests:number('requests'),accepted:number('accepted'),descendants:descendants(),currentFacts:current().length,visibleAncestors:[...crumb()?.querySelectorAll('.ix-breadcrumb-item > button,.ix-breadcrumb-item > a')??[]].filter(rendered).map(node=>node.textContent.trim()),note:get('hierarchy-note')?.value,labels:triggers().map(node=>node.getAttribute('aria-label'))});
 const reset=async()=>{await click(get('hierarchy-reset'),'Restore hierarchy');labels={previous:'Show previous breadcrumb items',next:'Show next breadcrumb items'};await at('cell-a','restored actual Cell A location');await wait('Restore hierarchy clears modes and requests',()=>number('requests')===0&&number('accepted')===0&&!get('hierarchy-veto').checked&&!get('hierarchy-hold').checked&&!get('hierarchy-local').checked&&!get('hierarchy-subtle').checked&&get('hierarchy-top').checked&&!get('hierarchy-empty').checked&&get('hierarchy-limit').value==='3'&&descendants().includes('motor-001')&&descendants().includes('motor-002'))};

 await wait('actual asset hierarchy gallery mounted',()=>host()&&crumb()&&result()&&heading()==='Cell A'&&triggers().length===2);
 await wait('WebView ready for native focus',()=>document.hasFocus());await at('cell-a','initial actual Cell A location');
 assert(data('path')==='root/north/assembly/line-01/cell-a'&&number('requests')===0&&number('accepted')===0&&descendants().includes('motor-001')&&descendants().includes('motor-002'),'initial path and actual descendant row actions agree');
 assert(ancestor('Assembly')&&ancestor('Line 01')&&!ancestor('Facilities')&&!ancestor('North plant')&&trigger('previous')?.getAttribute('aria-label')===labels.previous&&trigger('next')?.getAttribute('aria-label')===labels.next,'default visible-three trail hides exactly previous two ancestors and labels both menus');
 await open('previous');assert(item('previous','Facilities')&&item('previous','North plant')&&menu('previous').matches(':popover-open'),'previous native top-layer menu exposes actual hidden ancestors');await dismiss('previous');
 await open('next');assert(item('next','Motor 001')&&item('next','Motor 002'),'current-location child menu exposes actual children');await dismiss('next');snapshots.initial=snapshot();

 await choose('previous','North plant');await at('north','accepted previous-level navigation changes real location');await wait('previous menu changes actual descendants, focus and request count',()=>descendants().includes('assembly')&&number('requests')===1&&number('accepted')===1&&active()===get('hierarchy-heading'));snapshots.previous=snapshot();await reset();
 await choose('next','Motor 001');await at('motor-001','accepted current-child navigation opens actual motor detail');await wait('leaf has real maintenance note, known heading focus and no next menu',()=>rendered(get('hierarchy-note'))&&!trigger('next')&&number('requests')===1&&number('accepted')===1&&descendants().length===0&&active()===get('hierarchy-heading'));
 await focus(get('hierarchy-note'),'native leaf maintenance note');for(const value of ['Home','End','ArrowLeft',' ','Tab']){const event=key(active(),value);keyPermissions[value]=!event.defaultPrevented;assert(!event.defaultPrevented,'native maintenance note '+value+' remains available')}
 await input('Inspect motor cable');snapshots.nextLeaf=snapshot();

 await click(ancestor('Line 01'),'visible Line 01 ancestor');await at('line-01','visible ancestor request changes actual location');assert(number('requests')===2&&number('accepted')===2&&descendants().includes('cell-a'),'visible item callback accepted once and real child row changes');
 await wait('visible ancestor navigation focuses actual location heading',()=>active()===get('hierarchy-heading'));
 await click(get('hierarchy-open-cell-a'),'actual Cell A descendant row');await at('cell-a','direct child row opens Cell A');await click(get('hierarchy-open-motor-001'),'actual Motor 001 row');await at('motor-001','direct leaf row returns Motor 001');assert(get('hierarchy-note').value==='Inspect motor cable'&&number('requests')===2&&number('accepted')===2,'retained per-asset note survives away/back and direct rows do not invent breadcrumb requests');snapshots.ancestorAndRetainedNote=snapshot();

 await click(get('hierarchy-cell'),'Open Cell A');await at('cell-a','external Cell A reset point');const options=get('hierarchy-options');assert(options?.tagName==='DETAILS','hierarchy options use native details');if(!options.open)await click(options.querySelector('summary'),'Hierarchy options summary');await wait('actual hierarchy options open',()=>options.open&&rendered(get('hierarchy-veto')));
 await option('hierarchy-veto',true);let before=number('requests'),accepted=number('accepted');await choose('next','Motor 002');await wait('veto child request keeps actual location/trail/rows',()=>number('requests')===before+1&&number('accepted')===accepted&&heading()==='Cell A'&&data('current')==='cell-a'&&descendants().includes('motor-001')&&current().length===1);
 assert(active()===trigger('next')||opened('next')&&menu('next').contains(active())&&rendered(active()),'veto retains focus within unchanged available owner/menu');
 if(opened('next')){await focus(trigger('next'),'veto menu owner');await dismiss('next')}
 assert(active()===trigger('next'),'veto navigation retains usable unchanged menu-owner focus');
 await option('hierarchy-veto',false);await option('hierarchy-hold',true);before=number('requests');accepted=number('accepted');await choose('previous','Facilities');await wait('accepted controlled-held request keeps real current location',()=>number('requests')===before+1&&number('accepted')===accepted+1&&heading()==='Cell A'&&data('current')==='cell-a'&&descendants().includes('motor-002')&&current().length===1);
 assert(active()===trigger('previous')||opened('previous')&&menu('previous').contains(active())&&rendered(active()),'held request retains focus within unchanged available owner/menu');
 if(opened('previous')){await focus(trigger('previous'),'held menu owner');await dismiss('previous')}
 assert(active()===trigger('previous'),'held navigation retains usable unchanged menu-owner focus');
 await option('hierarchy-hold',false);snapshots.vetoAndHeld=snapshot();

 await option('hierarchy-local',true);labels={previous:'Vorherige Navigationselemente anzeigen',next:'Nächste Navigationselemente anzeigen'};
 await wait('localized previous and next native menu labels rendered',()=>trigger('previous')?.getAttribute('aria-label')===labels.previous&&trigger('next')?.getAttribute('aria-label')===labels.next);
 await option('hierarchy-subtle',true);await wait('subtle mode reaches actual breadcrumb',()=>crumb().dataset.subtle==='true');
 await option('hierarchy-top',false);await open('next');assert(!menu('next').matches(':popover-open')&&rendered(menu('next')),'same Dropdown menu works outside native top layer');await dismiss('next');
 await limit('1');await wait('one visible level leaves current menu plus previous overflow',()=>!ancestor('Assembly')&&!ancestor('Line 01')&&current().length===1&&trigger('previous')&&trigger('next'));await open('previous');assert(['Facilities','North plant','Assembly','Line 01'].every(label=>item('previous',label)),'runtime one-level limit exposes all previous ancestors in menu');await dismiss('previous');
 await limit('9');await wait('expanded visible limit shows complete ordinary trail',()=>!trigger('previous')&&ancestor('Facilities')&&ancestor('North plant')&&ancestor('Assembly')&&ancestor('Line 01')&&current().length===1);
 snapshots.labelsAndOptions=snapshot();

 await limit('3');await option('hierarchy-top',true);const owner=await open('next');assert(menu('next').matches(':popover-open'),'restored native top layer uses actual open popover');key(owner,'ArrowDown');await dismiss('next');
 await option('hierarchy-empty',true);await wait('empty hierarchy leaves no current fact or actionable breadcrumb menus',()=>current().length===0&&triggers().length===0&&!ancestor('Facilities')&&!ancestor('Line 01'));
 snapshots.empty=snapshot();const toolbar=get('hierarchy-reset'),optionsNode=get('hierarchy-options');await reset();assert(get('hierarchy-reset')===toolbar&&get('hierarchy-options')===optionsNode&&optionsNode.open,'Restore hierarchy preserves toolbar and actual options element');snapshots.reset=snapshot();

 await click(get('breadcrumb-narrow'),'narrow preview');await wait('narrow actual breadcrumb stays in frame',()=>get('breadcrumb-frame').getBoundingClientRect().width<=341&&crumb().getBoundingClientRect().width<=341);
 await open('next');await wait('narrow next menu actual motor choice is visible and hit-testable',()=>{const node=item('next','Motor 001'),box=node?.getBoundingClientRect();if(!box||box.width<=0||box.height<=0||box.left<0||box.top<0||box.right>innerWidth+1||box.bottom>innerHeight+1)return false;const hit=document.elementFromPoint(box.left+box.width/2,box.top+box.height/2);return hit===node||node.contains(hit)});
 await click(item('next','Motor 001'),'narrow actual motor menu choice');await at('motor-001','narrow child navigation opens real motor detail');await focus(get('hierarchy-note'),'narrow maintenance input');await input('Narrow maintenance check');
 layout={frameWidth:get('breadcrumb-frame').getBoundingClientRect().width,breadcrumbWidth:crumb().getBoundingClientRect().width,noteWidth:get('hierarchy-note').getBoundingClientRect().width,pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};assert(layout.noteWidth<=layout.frameWidth+1&&layout.pageScroll<=layout.pageClient+1,'narrow breadcrumb and actual detail input remain within page width');
 snapshots.narrow=snapshot();
 return {ok:true,snapshots,layout,keyPermissions,
  checks:['actual truncated trail/current metadata and both menu roles','previous overflow navigation changes real heading and descendants','current-child navigation opens actual leaf detail','visible ancestor request and retained per-asset note through direct descendant rows','veto and controlled-held breadcrumb requests preserve actual location','localized/subtle/top-layer controls and runtime visible limits','native Dropdown owner focus/Escape dismissal','empty hierarchy and Restore lifecycle','narrow native top-layer menu hit testing and usable leaf input'],
  input:'synthetic ArrowDown/Escape/input/key events, menu action DOM clicks and explicit native focus; trusted Enter/Space/Tab and input defaults are not simulated',
  trustedInput:'separate Chromium actual Enter/Space/ArrowDown/Escape/Tab and native typing; no physical-device certification',renderer:navigator.userAgent};
}catch(error){return {ok:false,error:String(error),snapshots,layout,keyPermissions,documentFocusedNow:document.hasFocus()}}
"#;
