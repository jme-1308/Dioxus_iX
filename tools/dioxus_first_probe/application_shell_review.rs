//! Excluded application-switch links, Rust sidebar presence and owner workflows.
use dioxus::prelude::*;
use dioxus_ix::*;

const SYMBOL: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='40' height='40' viewBox='0 0 40 40'%3E%3Crect width='40' height='40' rx='8' fill='%2331566c'/%3E%3Cpath d='M11 12h18v16H11z' fill='white'/%3E%3Cpath d='M16 17h8v6h-8z' fill='%2331566c'/%3E%3C/svg%3E";

fn destination_url(id: &str) -> String {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    {
        format!("about:blank#r0241-destination-{id}")
    }
    #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
    {
        format!("?applicationshell&destination={id}#r0241-destination-{id}")
    }
}

fn configuration(populated: bool, changed: bool, target: &str) -> AppSwitchConfig {
    AppSwitchConfig {
        current_app_id: "alpha".into(),
        label: "Equipment applications".into(),
        loading_label: "Loading equipment applications".into(),
        apps: if populated {
            [
                ("alpha", "Alpha application"),
                (
                    "beta",
                    if changed {
                        "Updated Beta application"
                    } else {
                        "Beta application"
                    },
                ),
            ]
            .into_iter()
            .map(|(id, name)| AppSwitchEntry {
                id: id.into(),
                name: name.into(),
                description: format!("Local {id} workspace"),
                icon_src: SYMBOL.into(),
                url: destination_url(id),
                target: target.into(),
            })
            .collect()
        } else {
            Vec::new()
        },
    }
}

#[component]
fn ShellFocusDestination() -> Element {
    let mounted = use_focus_mount("r0241-content-focus");
    rsx! {button {id:"r0241-content-focus",r#type:"button",class:"ix-button ix-button-secondary",onmounted:mounted,"Workspace action"}}
}

#[component]
pub fn ApplicationShellReview() -> Element {
    let destination = use_hook(|| {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|window| window.location().hash().ok())
                .and_then(|hash| hash.strip_prefix("#r0241-destination-").map(str::to_string))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            None::<String>
        }
    });
    if let Some(name) = destination {
        return rsx! {IxTheme {class:"shell-destination",style {{"body{margin:0}.shell-destination{padding:20px;font:inherit}.shell-destination h1{font-size:24px}"}},h1 {id:"r0241-destination-heading","Local {name} workspace"}p {"This named-frame destination was reached through the application link's browser default."}}};
    }
    let mut config = use_signal(|| Some(configuration(true, false, "r0241-destination")));
    let mut target = use_signal(|| "r0241-destination".to_string());
    let mut queued = use_signal(|| "none".to_string());
    let mut veto = use_signal(|| false);
    let mut sidebar = use_signal(|| true);
    let mut sidebar_note = use_signal(|| "Station A".to_string());
    let mut mounted = use_signal(|| true);
    let mut epoch = use_signal(|| 0_u64);
    let mut requests = use_signal(|| 0_usize);
    let mut applied = use_signal(|| 0_usize);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style {{STYLES}}
        IxTheme {class:"application-shell-review",
            h1 {"Application shell owned by Dioxus"}
            p {"Toggle the sidebar or open Apps in the header. Queue an update before opening to observe loading, replacement, retained configuration or owner removal. Application links target the local frame below; desktop verification checks requested targets and a real existing frame."}
            div {class:"shell-controls",
                button {id:"r0241-sidebar-toggle",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|sidebar.toggle(),"Show / hide sidebar"}
                button {id:"r0241-veto",r#type:"button",class:"ix-button ix-button-secondary","aria-pressed":veto().to_string(),onclick:move |_|veto.toggle(),"Toggle request veto"}
                button {id:"r0241-loading",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|config.set(Some(configuration(false,false,&target()))),"Loading configuration"}
                button {id:"r0241-populate",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|config.set(Some(configuration(true,false,&target()))),"Populate applications"}
                button {id:"r0241-replace",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|config.set(Some(configuration(true,true,&target()))),"Updated applications"}
                button {id:"r0241-remove-config",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|config.set(None),"Remove configuration"}
                label {r#for:"r0241-target","Link target"}
                select {id:"r0241-target",value:target(),onchange:move |event|{let next=event.value();target.set(next.clone());config.set(Some(configuration(true,false,&next)));},option {value:"r0241-destination","Named local frame"}option {value:"_self","Current document"}}
                label {r#for:"r0241-queued","After next accepted opening"}
                select {id:"r0241-queued",value:queued(),onchange:move |event|queued.set(event.value()),option {value:"none","Keep configuration"}option {value:"populate","Populate loading entries"}option {value:"replace","Replace entries"}option {value:"remove","Remove configuration"}option {value:"hide","Remove Application owner"}}
                button {id:"r0241-owner",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|mounted.toggle(),"Hide / show Application"}
                button {id:"r0241-remount",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|{epoch+=1;mounted.set(true);},"Remount Application"}
            }
            div {id:"r0241-stage",
                if mounted() {
                    Application {key:"shell-{epoch()}",class:"r0241-application",sidebar_visible:sidebar(),app_switch_config:config,
                        header:rsx! {ApplicationHeader {name:"Operations workspace",show_menu:false,aria_label_app_switch_icon_button:"Switch application",
                            onopenappswitch:move |event:MouseEvent|{
                                requests+=1;if veto(){event.prevent_default();return;}
                                let action=queued();if action=="none"{return;}queued.set("none".into());
                                let current_epoch=epoch();let current_target=target();
                                spawn(async move {futures_timer::Delay::new(std::time::Duration::from_millis(800)).await;
                                    if !mounted()||epoch()!=current_epoch{return;}
                                    match action.as_str(){"populate"=>config.set(Some(configuration(true,false,&current_target))),"replace"=>config.set(Some(configuration(true,true,&current_target))),"remove"=>config.set(None),"hide"=>mounted.set(false),_=>{}}
                                    applied+=1;
                                });
                            }
                        }},
                        sidebar:rsx! {div {class:"shell-sidebar",h2 {"Station details"}label {r#for:"r0241-sidebar-note","Station name"}input {id:"r0241-sidebar-note",class:"shell-field",value:sidebar_note(),oninput:move |event|sidebar_note.set(event.value())}}},
                        h2 {"Equipment workspace"}
                        p {"Sidebar presence follows the Rust flag. Opening, configuration and navigation use the current Application owner."}
                        ShellFocusDestination {}
                    }
                } else {p {id:"r0241-unmounted","Application owner removed."}}
            }
            section {h2 {"Local application destination"}iframe {id:"r0241-frame",name:"r0241-destination",title:"Application navigation destination",srcdoc:"<h2>Choose an application</h2><p>Alpha or Beta opens here through a real named-target link.</p>"}}
            output {id:"r0241-state",role:"status","aria-live":"polite","data-sidebar":sidebar().to_string(),"data-veto":veto().to_string(),"data-config-present":config().is_some().to_string(),"data-mounted":mounted().to_string(),"data-epoch":epoch().to_string(),"data-requests":requests().to_string(),"data-applied":applied().to_string(),"data-queued":queued(),"data-target":target(),
                "Opening requests: {requests()}; queued updates applied: {applied()}; station: {sidebar_note()}; owner generation: {epoch()}"
            }
            div {onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let task=super::dom_query::eval(NATIVE_APPLICATION_SHELL_CHECK);let _=task.send("application-shell");
                    match task.join::<serde_json::Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dioxus native application shell: {value}");window.close();}
                        other=>{eprintln!("Dioxus native application shell failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus native application shell timed out");std::process::exit(1);});}
            }}
        }
    }
}

const STYLES: &str = "body{margin:0}.application-shell-review{box-sizing:border-box;min-height:100vh;padding:24px}.application-shell-review>h1{font-size:28px;margin:0 0 16px}.application-shell-review h2{font-size:20px;margin:0 0 16px}.application-shell-review p{max-width:70ch;line-height:1.5}.shell-controls{display:flex;flex-wrap:wrap;align-items:center;gap:10px;max-width:960px;margin:20px 0}.r0241-application{height:360px;border:1px solid var(--ix-color-border)}.r0241-application .ix-application-main,.shell-sidebar{padding:16px}.shell-field{box-sizing:border-box;width:100%;min-height:34px;margin:8px 0;padding:6px 8px;background:var(--ix-color-surface);color:inherit;border:1px solid var(--ix-color-border);font:inherit}.application-shell-review section{margin:28px 0}.application-shell-review iframe{box-sizing:border-box;width:100%;max-width:760px;height:160px;border:1px solid var(--ix-color-border);background:var(--ix-color-surface)}.application-shell-review output{display:block;max-width:70ch;line-height:1.6;overflow-wrap:anywhere;font-variant-numeric:tabular-nums}@media(max-width:600px){.application-shell-review{padding:16px}}";

// Synthetic native events and a non-forwarding spy check requested URL/target.
// A final forwarded operation verifies an existing named frame, not new popups.
// Browser verification follows actual href/target defaults.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_APPLICATION_SHELL_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),state=()=>byId('r0241-state'),stage=()=>byId('r0241-stage');
const app=()=>stage()?.querySelector('.ix-application'),dialog=()=>stage()?.querySelector('dialog.ix-modal'),switcher=()=>stage()?.querySelector('.ix-header-app-switch button');
const entry=name=>Array.from(dialog()?.querySelectorAll('a.ix-application-switch-entry')??[]).find(node=>node.textContent.includes(name));
const wait=async(label,predicate)=>{for(let n=0;n<240;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' state='+state()?.textContent+' dialog='+dialog()?.textContent);};
const click=id=>byId(id).click(),choose=(id,value)=>{byId(id).value=value;byId(id).dispatchEvent(new Event('change',{bubbles:true}));};
const open=async()=>{switcher().focus();switcher().click();await wait('Application switch opened',()=>dialog()?.open);};
const close=async()=>{dialog().dispatchEvent(new Event('cancel',{cancelable:true}));await wait('Application switch closed',()=>!dialog()?.open);};
const opens=[],nativeOpen=window.open;window.open=(...args)=>{opens.push(args);return {};};
await wait('Application and sidebar mounted',()=>switcher()&&app()?.querySelector('.ix-application-sidebar'));
const note=byId('r0241-sidebar-note');note.value='Station revised';note.dispatchEvent(new Event('input',{bubbles:true}));await wait('sidebar child edits',()=>state().textContent.includes('Station revised'));
click('r0241-sidebar-toggle');await wait('Rust sidebar flag removes slot',()=>state().dataset.sidebar==='false'&&!app().querySelector('.ix-application-sidebar'));
click('r0241-sidebar-toggle');await wait('Rust sidebar flag restores current content',()=>app().querySelector('.ix-application-sidebar')&&byId('r0241-sidebar-note').value==='Station revised');
click('r0241-veto');await wait('request veto armed',()=>state().dataset.veto==='true');const before=Number(state().dataset.requests);switcher().click();await wait('request veto handled',()=>Number(state().dataset.requests)>before);if(dialog()?.open)throw new Error('veto opened switch');
click('r0241-veto');await wait('request allowed',()=>state().dataset.veto==='false');
click('r0241-loading');choose('r0241-queued','populate');await wait('population queued',()=>state().dataset.queued==='populate');await open();await wait('loading message visible',()=>dialog().textContent.includes('Loading equipment applications'));await wait('live populated links',()=>entry('Alpha application')&&entry('Beta application'));
if(entry('Alpha application').getAttribute('aria-current')!=='page')throw new Error('current application marker missing');
const alpha=entry('Alpha application'),alphaUrl=alpha.getAttribute('href'),alphaTarget=alpha.getAttribute('target');alpha.click();await wait('link requests exact named target and closes',()=>opens.length===1&&!dialog()?.open);
if(JSON.stringify(opens[0])!==JSON.stringify([alphaUrl,alphaTarget])||alphaTarget!=='r0241-destination')throw new Error('named target navigation differs');
choose('r0241-queued','replace');await wait('replacement queued',()=>state().dataset.queued==='replace');await open();await wait('live replacement while shown',()=>entry('Updated Beta application'));await close();
choose('r0241-queued','remove');await wait('removal queued',()=>state().dataset.queued==='remove');await open();await wait('None retains shown entries',()=>state().dataset.configPresent==='false'&&dialog()?.open&&entry('Alpha application')&&!switcher());
entry('Alpha application').click();await wait('retained entry navigates and closes',()=>opens.length===2&&!dialog()?.open);
click('r0241-populate');await wait('configuration restored',()=>switcher());choose('r0241-target','_self');await wait('self target selected',()=>state().dataset.target==='_self');await open();
const beta=entry('Beta application'),betaUrl=beta.getAttribute('href');beta.click();await wait('self target operation and close',()=>opens.length===3&&!dialog()?.open);
if(JSON.stringify(opens[2])!==JSON.stringify([betaUrl,'_self']))throw new Error('self target navigation differs');
choose('r0241-queued','hide');await wait('owner removal queued',()=>state().dataset.queued==='hide');await open();await wait('owner removed with no modal',()=>state().dataset.mounted==='false'&&!app()&&!dialog());
click('r0241-owner');await wait('fresh owner restored',()=>app()&&switcher());await open();await close();
const retired=app();click('r0241-remount');await wait('keyed owner remount',()=>state().dataset.epoch==='1'&&app()!==retired&&!retired.isConnected&&switcher());await open();await close();
window.open=nativeOpen;
choose('r0241-target','r0241-destination');await wait('named target restored for actual navigation',()=>state().dataset.target==='r0241-destination');await open();
const forwarded=entry('Alpha application'),forwardedUrl=forwarded.getAttribute('href'),forwardedTarget=forwarded.getAttribute('target');
if(forwardedUrl!=='about:blank#r0241-destination-alpha'||forwardedTarget!=='r0241-destination')throw new Error('forwarded named-frame link setup differs');
forwarded.click();
await wait('forwarded existing named frame reaches Alpha and modal closes',()=>byId('r0241-frame').contentWindow.location.hash==='#r0241-destination-alpha'&&!dialog()?.open);
return {ok:true,syntheticEvents:true,navigationSpyDoesNotForward:true,exactOperations:opens,forwardedNamedFrame:{url:forwardedUrl,target:forwardedTarget,hash:byId('r0241-frame').contentWindow.location.hash},sidebarRustPresence:true,requestVeto:true,loadingAndLiveReplacement:true,retainedNone:true,ownerRemovalAndRemount:true};
"#;
