//! Excluded actual-gallery dropdown button preview and bounded native DOM replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/composition.rs"]
mod gallery_composition;

#[component]
pub fn DropdownButtonPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_composition::STYLES} }
        IxTheme { class: "dropdown-button-review",
            h1 { "Dropdown button" }
            Button { id: "dropdown-button-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "dropdown-button-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_composition::DropdownTaskActionsExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_DROPDOWN_BUTTON_CHECK);
                        if let Err(error) = query.send("dropdownbutton") {
                            eprintln!("Dropdown button native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native dropdown button: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native dropdown button failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Dropdown button native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.dropdown-button-review{box-sizing:border-box;min-height:100vh;padding:24px}.dropdown-button-review>h1{font-size:28px;margin:0 0 24px}.dropdown-button-review h2{font-size:22px;margin:24px 0 16px}.dropdown-button-review p{max-width:72ch;line-height:1.5}#dropdown-button-frame{margin-top:24px}@media(max-width:600px){.dropdown-button-review{padding:16px}}";

// Excluded native replay; trusted browser input/default activation is separate lead evidence.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_DROPDOWN_BUTTON_CHECK: &str = r#"
let snapshots={},layout={},keyPermissions={},hitFacts={},failureGeometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement,result=()=>get('task-actions-result'),host=()=>get('task-actions-button')?.querySelector('.ix-dropdown[role="button"]');
 const menu=()=>get(host()?.getAttribute('aria-controls')),presentation=()=>host()?.querySelector('button.ix-dropdown-trigger'),opened=()=>host()?.getAttribute('aria-expanded')==='true'&&menu()&&!menu().hidden;
 const title=()=>get('task-action-title'),note=()=>get('task-action-note'),preparation=()=>get('task-actions-preparation'),add=()=>get('task-action-add'),count=()=>Number(result()?.dataset.count),requests=()=>Number(result()?.dataset.changeRequests);
 const record=id=>get('task-record-task-'+id),field=(id,name)=>record(id)?.querySelector('[data-field="'+name+'"]')?.textContent,badge=()=>get('task-actions-badge');
 const assert=(condition,label)=>{if(!condition)throw Error(label)},hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{let stable=0;for(let attempt=0;attempt<150;attempt++){stable=predicate()?stable+1:0;if(stable===3)return;await hold(20)}throw Error(label+'; state='+JSON.stringify(result()?.dataset)+'; focus='+(active()?.id||active()?.className))};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' native focus',()=>active()===node)};
 const input=async(node,value,label)=>{Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait(label+' controlled draft',()=>node.isConnected&&node.value===value)};
 const key=(node,value)=>{const event=new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const select=async(id,value)=>{const node=get(id);node.value=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait(id+' applied',()=>node.value===value&&result()?.dataset[id==='task-actions-mode'?'mode':'placement']===value)};
 const checked=async(id,value)=>{const node=get(id);node.checked=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait(id+' applied',()=>result()?.dataset[id.slice(13).replace(/-([a-z])/g,(_,letter)=>letter.toUpperCase())]===String(value))};
 const open=async()=>{if(!opened()){await focus(host(),'task actions owning trigger');host().click()}await wait('actual task action menu open',()=>opened()&&result().dataset.open==='true')};
 const close=async()=>{if(opened()){await focus(host(),'task actions Escape owner');key(host(),'Escape')}await wait('shared Escape closes and returns owning trigger',()=>!opened()&&active()===host())};
 const rect=node=>{if(!node)return null;const r=node.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height}};
 const hit=async(node,label,insideFrame=true)=>{await focus(node,label);await wait(label+' actual hit and fit',()=>{const r=node.getBoundingClientRect(),frame=get('dropdown-button-frame').getBoundingClientRect(),point=document.elementFromPoint(r.left+r.width/2,r.top+r.height/2);failureGeometry={node:node.id||node.className,nodeRect:rect(node),frameRect:rect(get('dropdown-button-frame')),popup:rect(menu()),hit:point?.id||point?.className||point?.tagName,viewport:{width:innerWidth,height:innerHeight},pageWidth:document.documentElement.scrollWidth};return r.width>0&&r.height>0&&r.left>=0&&r.right<=innerWidth+1&&r.top>=0&&r.bottom<=innerHeight+1&&(!insideFrame||(r.left>=frame.left-1&&r.right<=frame.right+1))&&(point===node||node.contains(point))});hitFacts[label]={...failureGeometry}};
 const snapshot=()=>({state:{...result().dataset},title:title()?.value,note:note()?.value,badge:badge()?.textContent,focus:active()?.id,records:[...get('task-action-records')?.querySelectorAll('tbody tr[data-task-id]')??[]].map(node=>({id:node.dataset.taskId,title:node.querySelector('[data-field="title"]')?.textContent,note:node.querySelector('[data-field="note"]')?.textContent}))});

 await wait('actual task action example mounted',()=>host()&&menu()&&presentation()&&title()&&note()&&result()&&preparation());await wait('native document focused',()=>document.hasFocus());
 assert(count()===0&&result().dataset.mode==='normal'&&result().dataset.placement==='bottom-start'&&!opened()&&title().value==='Inspect Motor 001'&&note().value==='Check drive coupling'&&preparation().value==='Prepare the local task queue','initial actual task queue and private drafts');assert(host().getAttribute('aria-label')==='Task queue actions'&&host().getAttribute('aria-haspopup')==='menu'&&menu().getAttribute('role')==='menu'&&presentation().getAttribute('aria-hidden')==='true'&&presentation().inert&&presentation().tabIndex===-1&&presentation().textContent.includes('Task actions'),'shared owning role button and inert legacy normal presentation');const originalHost=host(),originalMenu=menu(),retainedTitle=title(),retainedNote=note(),retainedPreparation=preparation();await open();await input(title(),'  Inspect Motor 001 bearing  ','actual request title');await input(note(),'Check Motor 001 lubrication.','actual request note');add().click();await wait('Add request stores one actual verbatim record',()=>count()===1&&field(1,'title')==='  Inspect Motor 001 bearing  '&&field(1,'note')==='Check Motor 001 lubrication.'&&!opened());layout.normal=rect(presentation());snapshots.normalRecord=snapshot();

 const options=get('task-actions-options');options.querySelector('summary').click();await wait('native task options expanded',()=>options.open);await select('task-actions-mode','icon');await wait('actual compact named icon-only presentation',()=>presentation().classList.contains('ix-dropdown-trigger-icon-only')&&presentation().querySelector('svg.ix-icon'));assert(host()===originalHost&&menu()===originalMenu&&host().getAttribute('aria-label')==='Task queue actions'&&presentation().inert&&presentation().getAttribute('aria-hidden')==='true'&&!presentation().textContent.includes('Task actions'),'icon-only mode retains one named owner and inert icon presentation');layout.icon=rect(presentation());assert(layout.icon.width<layout.normal.width&&layout.icon.height>=31,'icon-only trigger is visibly compact and usable');await focus(host(),'named icon keyboard owner');const enter=key(host(),'Enter');keyPermissions.iconEnter=enter.defaultPrevented;await wait('owning role-button Enter opens real shared menu',()=>opened());assert(enter.defaultPrevented,'owning role button consumes activation rather than native inert child');await close();snapshots.iconMode=snapshot();

 await select('task-actions-mode','custom');await wait('actual Rust custom label displays current queue count',()=>badge()?.textContent.trim()==='1'&&presentation().textContent.includes('Requests')&&!presentation().textContent.includes('Task actions'));await open();await input(title(),'Prepare Pump 002 inspection','retained custom-label title');await input(note(),'Retained local menu preparation','retained custom-label note');await focus(note(),'live label-mode editing field');for(const mode of ['normal','icon','custom']){await select('task-actions-mode',mode);assert(host()===originalHost&&menu()===originalMenu&&title()===retainedTitle&&note()===retainedNote&&note().value==='Retained local menu preparation'&&title().value==='Prepare Pump 002 inspection'&&active()===retainedNote&&opened(),'mode updates preserve actual menu fields/draft/focus')}add().click();await wait('custom-label action stores real record and reacts count',()=>count()===2&&field(2,'title')==='Prepare Pump 002 inspection'&&field(2,'note')==='Retained local menu preparation'&&badge()?.textContent.trim()==='2'&&!opened());await open();assert(title()===retainedTitle&&note()===retainedNote&&note().value==='Retained local menu preparation','custom label action close/reopen retains menu-owned draft');await close();await input(preparation(),'Prepare retained task actions','independent preparation');get('task-actions-clear').click();await wait('Clear requests updates actual model and reactive badge only',()=>count()===0&&!record(1)&&!record(2)&&badge()?.textContent.trim()==='0');assert(title()===retainedTitle&&note()===retainedNote&&preparation()===retainedPreparation&&note().value==='Retained local menu preparation'&&preparation().value==='Prepare retained task actions','record clear preserves menu and preparation scopes');snapshots.customLabel=snapshot();

 await select('task-actions-placement','top-end');await checked('task-actions-top-layer',true);await open();await wait('forwarded placement reaches actual native top-layer surface',()=>menu().dataset.configuredPlacement==='top-end'&&menu().matches(':popover-open'));assert(host()===originalHost&&menu()===originalMenu&&note()===retainedNote&&note().value==='Retained local menu preparation','placement/top-layer updates retain shared runtime and local draft');await select('task-actions-placement','bottom-end');await wait('live placement metadata updates same popup',()=>menu()===originalMenu&&menu().dataset.configuredPlacement==='bottom-end'&&menu().matches(':popover-open'));await focus(note(),'top-layer native draft');const nativeHome=key(note(),'Home');keyPermissions.noteHome=!nativeHome.defaultPrevented;assert(!nativeHome.defaultPrevented,'composed native input keeps its editing key');await close();snapshots.placementTopLayer=snapshot();

 for(const mode of ['disabled','loading']){await checked('task-actions-'+mode,true);const before=requests();assert(host().getAttribute('aria-disabled')==='true'&&host().tabIndex===-1&&presentation().disabled,mode+' protects the owning trigger and inert presentation');if(mode==='loading')assert(host().getAttribute('aria-busy')==='true'&&presentation().querySelector('.ix-spinner'),'loading keeps a meaningful spinner/busy state');host().click();key(host(),'Enter');await wait(mode+' cannot open or issue change requests',()=>!opened()&&requests()===before&&count()===0);await checked('task-actions-'+mode,false)}await checked('task-actions-veto',true);const beforeVeto=requests(),changes=Number(result().dataset.changes);host().click();await wait('opening veto records one request without accepted open',()=>requests()===beforeVeto+1&&!opened()&&Number(result().dataset.changes)===changes);await checked('task-actions-veto',false);await open();await input(title(),'   ','blank title guard');add().click();await wait('invalid title keeps real menu open with validation and no request record',()=>opened()&&count()===0&&get('task-action-error')?.getAttribute('role')==='alert'&&title().getAttribute('aria-invalid')==='true');assert(note()===retainedNote&&note().value==='Retained local menu preparation','guarded actions retain actual local draft');await close();snapshots.guards=snapshot();

 const reset=get('task-actions-reset');await focus(reset,'Restore task actions');reset.click();await wait('Restore replaces only owned menu/drafts/model and options values',()=>host()&&host()!==originalHost&&menu()!==originalMenu&&title()!==retainedTitle&&note()!==retainedNote&&preparation()!==retainedPreparation&&!opened()&&count()===0&&result().dataset.mode==='normal'&&result().dataset.placement==='bottom-start'&&title().value==='Inspect Motor 001'&&note().value==='Check drive coupling'&&preparation().value==='Prepare the local task queue'&&['task-actions-disabled','task-actions-loading','task-actions-veto','task-actions-top-layer'].every(id=>!get(id).checked));assert(!originalHost.isConnected&&!originalMenu.isConnected&&!retainedTitle.isConnected&&!retainedNote.isConnected&&!retainedPreparation.isConnected&&get('task-actions-reset')===reset&&get('task-actions-options')===options&&active()===reset,'scoped reset removes stale native nodes and keeps toolbar/options focus');get('dropdown-button-narrow').click();await wait('actual 340px task action frame',()=>get('dropdown-button-frame').getBoundingClientRect().width<=341);await select('task-actions-mode','custom');await checked('task-actions-top-layer',true);await hit(preparation(),'narrow independent preparation');await hit(host(),'narrow owning trigger');await open();await hit(title(),'narrow actual request title',false);await input(title(),'Narrow maintenance request','narrow title');await hit(note(),'narrow actual request note',false);await input(note(),'Confirm access before inspection','narrow note');await hit(add(),'narrow actual Add request',false);add().click();await wait('narrow action creates exact real record and reactive count',()=>count()===1&&field(1,'title')==='Narrow maintenance request'&&field(1,'note')==='Confirm access before inspection'&&badge()?.textContent.trim()==='1'&&!opened());await select('task-actions-mode','icon');await hit(host(),'narrow named compact trigger');await open();layout.narrow={frame:rect(get('dropdown-button-frame')),trigger:rect(host()),presentation:rect(presentation()),popup:rect(menu()),field:rect(title()),viewport:{width:innerWidth,height:innerHeight},pageWidth:document.documentElement.scrollWidth};assert(layout.narrow.popup.left>=0&&layout.narrow.popup.right<=innerWidth+1&&layout.narrow.field.height>=31&&layout.narrow.pageWidth<=innerWidth+1&&note().value==='Confirm access before inspection','narrow popup/fields/actions fit actual viewport and retain saved task workflow');snapshots.resetNarrow=snapshot();
 return {ok:true,snapshots,layout,keyPermissions,hitFacts,checks:['legacy normal owning role-button/inert presentation and exact real request','compact accessible icon-only owner and shared Enter/Escape','reactive Rust custom label and persistent menu drafts through modes/record clear','forwarded live placement and native top-layer on same shared popup','disabled/loading/opening veto and meaningful invalid-title guard','scoped Restore and340px actual input/action hits producing exact records'],input:'excluded synthetic input/change/keydown, DOM clicks and explicit native focus/normal page scrolling; owning role-button keys execute Rust handlers, inert native child defaults are not simulated',trustedInput:'separate lead Chromium trusted Tab/typing/Enter/Space/pointer and paired appearance evidence; no broad shared-dropdown chronology audit',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,keyPermissions,hitFacts,failureGeometry}}
"#;
