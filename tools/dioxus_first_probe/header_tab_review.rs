//! Excluded Header sequential-focus preview and synthetic Linux boundary driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn FocusHeader(
    prefix: String,
    mode: usize,
    show_menu: bool,
    menu_actions: Signal<usize>,
    apps_actions: Signal<usize>,
    work_actions: Signal<usize>,
    overflow_changes: Signal<usize>,
) -> Element {
    let label = format!("{prefix} overflow actions");
    rsx! {
        ApplicationHeader {
            name:"Equipment", show_menu:show_menu && mode==0, show_app_switch:mode<2,
            aria_label_app_switch_icon_button:format!("{prefix} applications"),
            aria_label_more_menu_icon_button:label,
            onmenutoggle:move |_|menu_actions+=1,
            onopenappswitch:move |_|apps_actions+=1,
            onoverflowchange:move |_|overflow_changes+=1,
            actions:if mode<2 {Some(rsx! {Button {id:format!("{prefix}-default"),variant:ButtonVariant::Secondary,onclick:move |_|work_actions+=1,"Refresh"}})} else {None},
            overflow:rsx! {
                Dropdown {label:"Equipment actions",class:format!("{prefix}-child"),navigation_mode:DropdownNavigationMode::RovingTabindex,
                    DropdownItem {id:format!("{prefix}-child-first"),label:"Inspect equipment",keep_open:true,onitemclick:move |_|work_actions+=1}
                    DropdownItem {id:format!("{prefix}-child-last"),label:"Export equipment",keep_open:true,onitemclick:move |_|work_actions+=1}
                }
            },
        }
    }
}

#[component]
pub fn HeaderTabReview() -> Element {
    let popover = use_popover();
    let mut mode = use_signal(|| 0_usize);
    let mut mounted = use_signal(|| true);
    let mut show_menu = use_signal(|| true);
    let mut epoch = use_signal(|| 0_u64);
    let menu_actions = use_signal(|| 0_usize);
    let apps_actions = use_signal(|| 0_usize);
    let mut work_actions = use_signal(|| 0_usize);
    let overflow_changes = use_signal(|| 0_usize);
    let tail = use_focus_mount("r0244-panel-tail");
    let after = use_focus_mount("r0244-workspace-after");
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style {{STYLES}}
        IxTheme {class:"header-tab-review",
            h1 {"Header controls in the keyboard path"}
            p {"Menu, Apps and More are ordinary Dioxus buttons. Use Tab and Shift+Tab across the workspace, or open the panel to check its focus boundary. On narrow screens Apps disappears and the default action moves into More."}
            div {class:"header-tab-controls",
                label {r#for:"r0244-boundary","First panel control"}
                select {id:"r0244-boundary",value:mode().to_string(),onchange:move |event|mode.set(event.value().parse().unwrap_or(0)),
                    option {value:"0","Menu"} option {value:"1","Apps"} option {value:"2","More"}
                }
                button {id:"r0244-menu-toggle",r#type:"button",class:"ix-button ix-button-secondary","aria-pressed":show_menu().to_string(),onclick:move |_|show_menu.toggle(),"Show / hide Menu"}
                button {id:"r0244-mounted",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|mounted.toggle(),"Show / hide headers"}
                button {id:"r0244-remount",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|{epoch+=1;mounted.set(true);show_menu.set(true);},"Remount headers"}
            }
            section {
                h2 {"Workspace navigation"}
                div {id:"r0244-workspace",class:"header-tab-stage",
                    if mounted() {FocusHeader {key:"workspace-{epoch()}",prefix:"r0244-workspace",mode:0,show_menu:show_menu(),menu_actions,apps_actions,work_actions,overflow_changes}}
                    div {class:"header-tab-body",p {"Equipment is ready. Header actions update the activity counts below."}
                        button {id:"r0244-workspace-after",r#type:"button",class:"ix-button ix-button-secondary",onmounted:after,onclick:move |_|work_actions+=1,"Workspace action"}
                    }
                }
            }
            section {
                h2 {"Header at a Popover boundary"}
                div {id:"r0244-panel-trigger",PopoverTrigger {popover,"Open equipment panel"}}
                Popover {popover,label:"Header keyboard panel",
                    div {id:"r0244-panel",class:"header-tab-panel",
                        if mounted() {FocusHeader {key:"panel-{epoch()}",prefix:"r0244-panel",mode:mode(),show_menu:show_menu(),menu_actions,apps_actions,work_actions,overflow_changes}}
                        div {class:"header-tab-body",p {"Tab from the last action wraps to the first visible Header control. Shift+Tab returns to the last action."}
                            button {id:"r0244-panel-tail",r#type:"button",class:"ix-button ix-button-secondary",onmounted:tail,onclick:move |_|work_actions+=1,"Last panel action"}
                        }
                    }
                }
            }
            output {id:"r0244-state",role:"status","aria-live":"polite","data-mode":mode().to_string(),"data-mounted":mounted().to_string(),"data-menu":show_menu().to_string(),"data-epoch":epoch().to_string(),"data-menu-actions":menu_actions().to_string(),"data-apps-actions":apps_actions().to_string(),"data-work-actions":work_actions().to_string(),"data-overflow-changes":overflow_changes().to_string(),
                "Menu actions: {menu_actions()}; Apps requests: {apps_actions()}; workspace actions: {work_actions()}; overflow changes: {overflow_changes()}; generation: {epoch()}"
            }
            p {class:"header-tab-note","The Linux driver dispatches synthetic boundary keys and pointer actions. Synthetic interior Tab does not perform browser traversal; Chromium checks use actual keyboard input."}
            div {onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=super::dom_query::eval(NATIVE_HEADER_TAB_CHECK);let _=query.send("header-tab-review");
                    match query.join::<serde_json::Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dioxus Header Tab native: {value}");window.close();}
                        other=>{eprintln!("Dioxus Header Tab native failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus Header Tab native timed out");std::process::exit(1);});}
            }}
        }
    }
}

const STYLES: &str = "body{margin:0}.header-tab-review{box-sizing:border-box;min-height:100vh;padding:24px}.header-tab-review>h1{font-size:28px;margin:0 0 16px}.header-tab-review h2{font-size:20px;margin:0 0 16px}.header-tab-review p{max-width:70ch;line-height:1.5}.header-tab-controls{display:flex;flex-wrap:wrap;align-items:center;gap:10px;margin:20px 0}.header-tab-controls select{max-width:100%;min-height:32px;background:var(--ix-color-component-1);color:inherit;font:inherit;border:1px solid var(--ix-color-border)}.header-tab-review section{margin:28px 0}.header-tab-stage{max-width:1000px;border:1px solid var(--ix-color-border)}.header-tab-body{padding:16px}.header-tab-body p{margin-top:0}.header-tab-panel{width:min(800px,calc(100vw - 40px));max-width:100%;box-sizing:border-box}.header-tab-review output{display:block;max-width:70ch;line-height:1.6;overflow-wrap:anywhere;font-variant-numeric:tabular-nums}.header-tab-note{color:var(--ix-color-soft-text);font-size:14px}@media(max-width:600px){.header-tab-review{padding:16px}.header-tab-body{padding:12px}}";

// Development-only synthetic event driver. This does not certify interior native
// Tab traversal, trusted click defaults or OS accessibility behavior.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_HEADER_TAB_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),state=()=>byId('r0244-state');
const panel=()=>document.querySelector('[aria-label="Header keyboard panel"]'),header=()=>byId('r0244-panel')?.querySelector('header');
const menu=()=>header()?.querySelector('.ix-header-menu-toggle'),apps=()=>header()?.querySelector('[aria-label="r0244-panel applications"]'),more=()=>header()?.querySelector('[aria-label="r0244-panel overflow actions"]');
const child=()=>document.querySelector('.r0244-panel-child');
const wait=async(label,predicate)=>{for(let n=0;n<220;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' focus='+document.activeElement?.outerHTML+' state='+state()?.textContent);};
const key=(node,value,shiftKey=false)=>{const event=new KeyboardEvent('keydown',{key:value,shiftKey,bubbles:true,cancelable:true});node.dispatchEvent(event);return event;};
const open=async()=>{byId('r0244-panel-trigger').querySelector('button').click();await wait('parent panel opening focus',()=>panel()?.matches(':popover-open')&&document.activeElement===(menu()||apps()||more()));};
const close=async()=>{key(document.activeElement,'Escape');await wait('parent panel closed',()=>!panel()?.matches(':popover-open'));};
const choose=async(mode)=>{const control=byId('r0244-boundary');control.value=String(mode);control.dispatchEvent(new Event('change',{bubbles:true}));await wait('boundary mode '+mode,()=>state().dataset.mode===String(mode));};
const wrap=async(target,label)=>{const tail=byId('r0244-panel-tail');tail.focus();const forward=key(tail,'Tab');if(!forward.defaultPrevented)throw new Error(label+' forward boundary not claimed');await wait(label+' forward boundary',()=>document.activeElement===target());const reverse=key(document.activeElement,'Tab',true);if(!reverse.defaultPrevented)throw new Error(label+' reverse boundary not claimed');await wait(label+' reverse boundary',()=>document.activeElement===tail);};
await wait('workspace Header mounted',()=>byId('r0244-workspace')?.querySelector('.ix-header-menu-toggle')&&more());
await open();await wrap(menu,'Menu');
const beforeMenu=Number(state().dataset.menuActions);menu().click();await wait('Menu pointer action',()=>Number(state().dataset.menuActions)===beforeMenu+1);
if(apps()){const beforeApps=Number(state().dataset.appsActions);apps().click();await wait('Apps pointer action',()=>Number(state().dataset.appsActions)===beforeApps+1);}
more().click();await wait('More opened',()=>more().getAttribute('aria-expanded')==='true');key(more(),'Escape');await wait('More Escape return',()=>more().getAttribute('aria-expanded')==='false'&&document.activeElement===more());
more().click();await wait('More reopened',()=>more().getAttribute('aria-expanded')==='true');const nested=child();nested.focus();key(nested,'Home');await wait('overflow child item focused',()=>document.activeElement===byId('r0244-panel-child-first'));key(document.activeElement,'Tab',true);await wait('nested reverse exits to More',()=>nested.getAttribute('data-ix-overlay-open')==='false'&&more().getAttribute('aria-expanded')==='false'&&document.activeElement===more());
if(!panel().matches(':popover-open'))throw new Error('nested reverse closed parent');
more().click();await wait('More opens for forward exit',()=>more().getAttribute('aria-expanded')==='true');nested.focus();key(nested,'Home');await wait('nested forward start',()=>document.activeElement===byId('r0244-panel-child-first'));key(document.activeElement,'Tab');await wait('nested forward exits to trailing action',()=>nested.getAttribute('data-ix-overlay-open')==='false'&&document.activeElement===byId('r0244-panel-tail'));if(!panel().matches(':popover-open'))throw new Error('nested forward closed parent');await close();
await choose(1);await open();const appsAvailable=Boolean(apps());if(appsAvailable)await wrap(apps,'Apps');await close();
await choose(2);await open();await wrap(more,'More');await close();
byId('r0244-mounted').click();await wait('headers removed',()=>state().dataset.mounted==='false'&&!byId('r0244-workspace').querySelector('header')&&!header());byId('r0244-remount').click();await wait('fresh headers registered',()=>state().dataset.mounted==='true'&&state().dataset.epoch==='1'&&more());await choose(0);await open();await wrap(menu,'remounted Menu');await close();
byId('r0244-menu-toggle').click();await wait('Menu hidden',()=>state().dataset.menu==='false'&&!header()?.querySelector('.ix-header-menu-toggle'));byId('r0244-menu-toggle').click();await wait('Menu restored',()=>state().dataset.menu==='true'&&menu());
return {ok:true,menuBoundary:true,appsBoundary:appsAvailable,moreBoundary:true,moreEscapeReturn:true,nestedReverseExit:true,nestedForwardExit:true,pointerActions:true,ownerRemount:true,menuVisibility:true,syntheticInteriorTab:false,responsiveViewport:innerWidth};
"#;
