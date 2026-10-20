//! Excluded actual-gallery account menu preview and bounded native DOM replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/account_workspace.rs"]
mod gallery_account;

#[component]
pub fn AccountMenuPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class: "account-menu-review",
            h1 { "Account menu" }
            Button { id: "account-menu-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "account-menu-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_account::AccountWorkspaceExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_ACCOUNT_MENU_CHECK);
                        if let Err(error) = query.send("accountmenu") {
                            eprintln!("Account menu native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native account menu: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native account menu failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Account menu native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.account-menu-review{box-sizing:border-box;min-height:100vh;padding:24px}.account-menu-review>h1{font-size:28px;margin:0 0 24px}.account-menu-review h2{font-size:22px;margin:24px 0 16px}.account-menu-review p{max-width:72ch;line-height:1.5}#account-menu-frame{margin-top:24px}@media(max-width:600px){.account-menu-review{padding:16px}}";

// Excluded native replay; trusted native keyboard defaults are separate lead evidence.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_ACCOUNT_MENU_CHECK: &str = r#"
let snapshots={},layout={},keyPermissions={},hitFacts={},failureGeometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement,result=()=>get('account-result'),owner=()=>get('account-menu');
 const trigger=()=>owner()?.querySelector('button.ix-menu-avatar-trigger'),menu=()=>get(trigger()?.getAttribute('aria-controls')),opened=()=>trigger()?.getAttribute('aria-expanded')==='true'&&menu()&&!menu().hidden;
 const note=()=>get('account-note'),notifications=()=>get('account-notifications'),preparation=()=>get('account-preparation');
 const action=id=>get(id)?.querySelector('button.ix-menu-avatar-item'),apply=()=>action('account-apply'),status=()=>action('account-status'),logout=()=>[...menu()?.querySelectorAll('button.ix-menu-avatar-item')??[]].find(node=>['Logout','Abmelden'].includes(node.textContent.trim()));
 const record=id=>get('account-record-'+id),field=(id,name)=>record(id)?.querySelector('[data-field="'+name+'"]')?.textContent.trim(),count=name=>Number(result()?.dataset[name]);
 const assert=(condition,label)=>{if(!condition)throw Error(label)},hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{let stable=0;for(let attempt=0;attempt<150;attempt++){stable=predicate()?stable+1:0;if(stable===3)return;await hold(20)}throw Error(label+'; state='+JSON.stringify(result()?.dataset)+'; focus='+(active()?.id||active()?.className))};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' native focus',()=>active()===node)};
 const input=async(node,value,label)=>{Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait(label+' controlled input',()=>node.isConnected&&node.value===value)};
 const key=(node,value,extra={})=>{const event=new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,bubbles:true,cancelable:true,...extra});node.dispatchEvent(event);return event};
 const checked=async(id,value)=>{const node=get(id);node.checked=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait(id+' applied',()=>node.checked===value)};
 const selectUser=async(id)=>{const node=get('account-user');await focus(node,'native account selector');node.value=id;node.dispatchEvent(new Event('change',{bubbles:true}));await wait('selected actual '+id+' account',()=>result()?.dataset.user===id);assert(active()===node,'ordinary account switching preserves selector focus')};
 const open=async()=>{if(!opened()){await focus(trigger(),'native account trigger');trigger().click()}await wait('actual shared account menu open',()=>opened()&&result().dataset.menuOpen==='true')};
 const close=async()=>{if(opened()){await focus(trigger(),'account Escape trigger');key(trigger(),'Escape')}await wait('account Escape closes and returns native trigger',()=>!opened()&&active()===trigger())};
 const rect=node=>{if(!node)return null;const r=node.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height}};
 const hit=async(node,label,insideFrame=true)=>{await focus(node,label);await wait(label+' actual hit and fit',()=>{const r=node.getBoundingClientRect(),frame=get('account-menu-frame').getBoundingClientRect(),point=document.elementFromPoint(r.left+r.width/2,r.top+r.height/2);failureGeometry={node:node.id||node.className,nodeRect:rect(node),frameRect:rect(get('account-menu-frame')),popup:rect(menu()),hit:point?.id||point?.className||point?.tagName,active:active()?.id,viewport:{width:innerWidth,height:innerHeight},pageWidth:document.documentElement.scrollWidth};return r.width>0&&r.height>0&&r.left>=0&&r.right<=innerWidth+1&&r.top>=0&&r.bottom<=innerHeight+1&&(!insideFrame||(r.left>=frame.left-1&&r.right<=frame.right+1))&&(point===node||node.contains(point))});hitFacts[label]={...failureGeometry}};
 const snapshot=()=>({state:{...result().dataset},draft:note()?.value,notifications:notifications()?.checked,focus:active()?.id,records:['alex','jo'].map(id=>({id,status:field(id,'status'),notifications:field(id,'notifications'),note:field(id,'note')}))});

 await wait('actual account workspace mounted',()=>result()&&trigger()&&menu()&&note()&&notifications()&&preparation()&&record('alex')&&record('jo'));await wait('native document ready for focus',()=>document.hasFocus());
 assert(result().dataset.user==='alex'&&result().dataset.signedIn==='true'&&!opened()&&['applies','statusActions','notificationChanges','logouts','changeRequests'].every(name=>count(name)===0)&&field('alex','notifications')==='On'&&field('jo','notifications')==='Off'&&field('alex','status')==='Available'&&field('jo','status')==='Away'&&['alex','jo'].every(id=>field(id,'note')==='No applied preparation'),'initial actual account records');
 assert(trigger().tagName==='BUTTON'&&trigger().getAttribute('aria-haspopup')==='menu'&&menu().getAttribute('role')==='menu'&&trigger().getAttribute('aria-controls')===menu().id,'native shared trigger/menu ARIA relationship');await open();assert(note().value==='Prepare Motor 001 review'&&notifications().checked&&apply()&&status()&&logout(),'actual native menu draft, independent preference and account actions');layout.initial={trigger:rect(trigger()),circle:rect(owner().querySelector('.ix-menu-avatar-circle')),popup:rect(menu()),item:rect(apply())};assert(layout.initial.trigger.height>=55&&Math.abs(layout.initial.circle.width-32)<1&&layout.initial.item.height>=39,'recognizable native trigger/circle/item sizing preserved');await close();snapshots.semantics=snapshot();

 await focus(trigger(),'account keyboard opener');key(trigger(),'ArrowDown');await wait('shared ArrowDown opens real menu',()=>opened());await focus(apply(),'account shared navigation action');key(apply(),'ArrowDown');await wait('shared ArrowDown reaches next native action',()=>active()===status());key(status(),'End');await wait('shared End reaches logout',()=>active()===logout());key(logout(),'Home');await wait('shared Home reaches first registered native account field',()=>active()===note());await focus(apply(),'native action default target');
 for(const value of ['Enter',' ']){const event=key(apply(),value);keyPermissions['nativeAction'+value]=!event.defaultPrevented;assert(!event.defaultPrevented,'native account '+value+' default activation remains available')}
 await focus(note(),'native account field');for(const value of ['Home','ArrowLeft',' ']){const event=key(note(),value);keyPermissions['note'+value]=!event.defaultPrevented;assert(!event.defaultPrevented,'native account field '+value+' default available')}const tab=key(note(),'Tab');keyPermissions.noteTab=!tab.defaultPrevented;assert(!tab.defaultPrevented,'native field Tab default remains available');await wait('shared roving Tab dismisses without simulating native focus traversal',()=>!opened());await focus(preparation(),'explicit focus after untrusted Tab');await open();await focus(note(),'native field Escape');key(note(),'Escape');await wait('field Escape closes and restores actual trigger',()=>!opened()&&active()===trigger());await open();await focus(preparation(),'outside account preparation');get('account-workspace').querySelector('h2').click();await wait('ordinary outside click dismisses without external focus theft',()=>!opened()&&active()===preparation());snapshots.keyboard=snapshot();

 await input(preparation(),'Prepare retained account workspace','outside preparation');await open();const retainedNote=note(),retainedPreference=notifications(),retainedPreparation=preparation();await input(note(),'Inspect Motor 001 access','account draft');notifications().click();await wait('native checkbox updates actual Alex preference without closing',()=>opened()&&!notifications().checked&&field('alex','notifications')==='Off'&&count('notificationChanges')===1);apply().click();await wait('Apply saves actual Alex preparation once',()=>count('applies')===1&&field('alex','note')==='Inspect Motor 001 access'&&!opened());await open();assert(note()===retainedNote&&notifications()===retainedPreference&&note().value==='Inspect Motor 001 access','close/reopen retains native custom nodes and draft');snapshots.retainedApply=snapshot();

 await selectUser('jo');await wait('Jo native menu reflects own saved preference and identity',()=>owner().textContent.includes('Jo Rivera')&&!notifications().checked);await open();assert(note()===retainedNote&&note().value==='Inspect Motor 001 access','account switch retains shared menu draft');notifications().click();await wait('Jo preference saves independently',()=>field('jo','notifications')==='On'&&field('alex','notifications')==='Off'&&count('notificationChanges')===2);await input(note(),'Prepare Jo account review','Jo draft');apply().click();await wait('Jo Apply saves actual separate record',()=>count('applies')===2&&field('jo','note')==='Prepare Jo account review'&&!opened());await open();status().click();await wait('native account status changes actual Jo once',()=>field('jo','status')==='Available'&&count('statusActions')===1&&!opened());await selectUser('alex');await open();assert(note()===retainedNote&&note().value==='Prepare Jo account review'&&!notifications().checked&&field('alex','note')==='Inspect Motor 001 access'&&field('jo','note')==='Prepare Jo account review','shared draft and saved per-account records have separate ownership');await close();snapshots.accountSwitch=snapshot();

 const options=get('account-options');options.querySelector('summary').click();await wait('native account options open',()=>options.open);await checked('account-german',true);await open();assert(logout()?.textContent.trim()==='Abmelden','localized actual logout action');const retiredOwner=owner(),retiredMenu=menu(),retiredDraft=note();await focus(logout(),'localized native logout');const logoutSpace=key(logout(),' ');assert(!logoutSpace.defaultPrevented,'native logout Space default available');logout().click();await wait('one logout ends local session and focuses recovery',()=>count('logouts')===1&&result().dataset.signedIn==='false'&&!owner()&&get('account-signed-out')&&active()===get('account-sign-in'));assert(!retiredOwner.isConnected&&!retiredMenu.isConnected&&!retiredDraft.isConnected&&preparation()===retainedPreparation&&preparation().value==='Prepare retained account workspace'&&field('alex','note')==='Inspect Motor 001 access'&&field('jo','note')==='Prepare Jo account review','logout removes owned native menu only and retains actual settings/preparation');get('account-sign-in').click();await wait('local session recovery restores menu and explicit selector focus',()=>result().dataset.signedIn==='true'&&trigger()&&active()===get('account-user'));await open();assert(note()!==retainedNote&&note().value==='Prepare Motor 001 review'&&count('logouts')===1&&field('alex','notifications')==='Off','new session restores fresh menu draft and saved account settings');await close();snapshots.logoutRecovery=snapshot();

 await checked('account-disable-action',true);await open();await wait('native status action disabled',()=>status().disabled);const guarded=count('statusActions');status().click();await wait('disabled native action cannot mutate record',()=>count('statusActions')===guarded&&field('alex','status')==='Available'&&opened());await focus(apply(),'disabled-skip source');key(apply(),'ArrowDown');await wait('shared navigation skips disabled action',()=>active()===logout());await close();await checked('account-disable-action',false);await checked('account-hide-logout',true);await open();assert(!logout()&&apply()&&status(),'hiding logout preserves real custom actions');const removedDraft=note();await checked('account-hide-actions',true);await wait('empty hidden-logout menu closes',()=>!opened()&&!note());trigger().click();await wait('no-action account rejects an empty popup',()=>!opened());assert(!trigger().hasAttribute('aria-haspopup')&&!trigger().hasAttribute('aria-controls')&&!removedDraft.isConnected&&count('logouts')===1,'empty trigger has no actionable ARIA and removed custom scope is cleaned without logout');await checked('account-hide-logout',false);await open();assert(logout()&&!apply()&&!status(),'logout-only account remains usable');await close();await checked('account-hide-actions',false);await open();assert(note()&&note()!==removedDraft&&note().value==='Prepare Motor 001 review'&&apply()&&status(),'restored actual custom membership remounts its owned draft');await close();snapshots.membershipGuards=snapshot();

 await checked('account-top-layer',true);await open();await wait('actual account popup uses native top layer',()=>menu().matches(':popover-open'));const oldOwner=owner(),oldMenu=menu(),oldNote=note(),oldPreparation=preparation(),reset=get('account-reset'),userSelect=get('account-user');await focus(reset,'Restore account workspace');reset.click();await wait('Restore resets actual models/options and remounts owned scopes',()=>owner()&&owner()!==oldOwner&&note()!==oldNote&&preparation()!==oldPreparation&&result().dataset.user==='alex'&&result().dataset.signedIn==='true'&&!opened()&&['applies','statusActions','notificationChanges','logouts','changeRequests'].every(name=>count(name)===0)&&field('alex','notifications')==='On'&&field('jo','notifications')==='Off'&&field('alex','status')==='Available'&&field('jo','status')==='Away'&&['alex','jo'].every(id=>field(id,'note')==='No applied preparation')&&['account-german','account-hide-logout','account-hide-actions','account-disable-action','account-top-layer'].every(id=>!get(id).checked));assert(!oldOwner.isConnected&&!oldMenu.isConnected&&!oldNote.isConnected&&!oldPreparation.isConnected&&get('account-reset')===reset&&get('account-user')===userSelect&&get('account-options')===options&&active()===reset&&preparation().value==='Prepare account settings review','scoped Restore removes former popup/fields and preserves native toolbar/options focus');snapshots.topLayerReset=snapshot();

 get('account-menu-narrow').click();await wait('actual narrow account frame',()=>get('account-menu-frame').getBoundingClientRect().width<=341);await hit(preparation(),'narrow outside preparation');await hit(trigger(),'narrow native account trigger');await checked('account-top-layer',true);await open();await wait('narrow account top layer',()=>menu().matches(':popover-open'));await hit(note(),'narrow actual account note',false);await input(note(),'Narrow account inspection','narrow draft');await hit(notifications(),'narrow actual account preference',false);notifications().click();await wait('narrow actual preference saved',()=>field('alex','notifications')==='Off'&&count('notificationChanges')===1);await hit(apply(),'narrow native Apply',false);apply().click();await wait('narrow actual note saved once',()=>count('applies')===1&&field('alex','note')==='Narrow account inspection'&&!opened());await open();await hit(logout(),'narrow localized native action',false);layout.narrow={frame:rect(get('account-menu-frame')),trigger:rect(trigger()),popup:rect(menu()),field:rect(note()),viewport:{width:innerWidth,height:innerHeight},pageWidth:document.documentElement.scrollWidth};assert(note().value==='Narrow account inspection'&&layout.narrow.field.height>=31&&layout.narrow.popup.left>=0&&layout.narrow.popup.right<=innerWidth+1&&layout.narrow.pageWidth<=innerWidth+1&&field('jo','note')==='No applied preparation','narrow actual menu fields/actions fit viewport and preserve independent records');snapshots.narrow=snapshot();
 return {ok:true,snapshots,layout,keyPermissions,hitFacts,checks:['native shared trigger/menu semantics and recognizable sizing','shared navigation/Escape/outside dismissal plus native field/action default permissions','retained menu draft and actual immediate preference/Apply records','account switching preserves draft and independent saved settings','localized logout once removes session and native focus recovery restores saved settings','hide/custom-membership/empty guard and disabled navigation','native top layer and scoped Restore cleanup/toolbar focus','340px actual native field/action hits and real account records'],input:'excluded synthetic form/key/click events and explicit native focus/normal page positioning; untrusted Enter/Space/Tab do not perform browser default activation',trustedInput:'separate lead Chromium trusted typing/Tab/Enter/Space/pointer and paired visual evidence',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,keyPermissions,hitFacts,failureGeometry}}
"#;
