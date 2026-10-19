//! Excluded representative Settings/About and Pane mounted-focus workspace.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn WorkspaceField(
    id: String,
    label: String,
    value: Signal<String>,
    #[props(default)] autofocus: bool,
) -> Element {
    let mounted = use_focus_mount(id.clone());
    let mut value = value;
    rsx! {
        label {r#for:id.clone(),"{label}"}
        input {id,class:"application-focus-field",value:value(),autofocus,onmounted:mounted,
            oninput:move |event|value.set(event.value())}
    }
}

#[component]
pub fn ApplicationFocusPreview() -> Element {
    let mut overlay = use_signal(|| None::<MenuOverlayKind>);
    let mut veto = use_signal(|| false);
    let mut settings_requests = use_signal(|| 0_usize);
    let mut close_events = use_signal(|| 0_usize);
    let settings_note = use_signal(|| "Station A".to_string());
    let pane_note = use_signal(|| "Pump P-101".to_string());
    let mut pane_expanded = use_signal(|| false);
    let mut floating = use_signal(|| true);
    let mut pane_generation = use_signal(|| 1_usize);
    let mut applied = use_signal(|| 0_usize);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class:"application-focus-review",
            h1 { "Dioxus-owned application focus" }
            p { "Open Settings or About from the navigation rail, or inspect equipment in the details pane. Registered controls own focus and return behavior." }
            div { class:"application-focus-controls",
                Button { id:"r0216-veto", variant:ButtonVariant::Secondary, onclick:move |_|veto.toggle(),
                    if veto(){"Allow Settings opening"}else{"Veto Settings opening"} }
                Button { id:"r0216-overlay-outside-close", variant:ButtonVariant::Secondary,
                    onclick:move |_|overlay.set(None), "Close overlay from outside" }
                Button { id:"r0216-pane-open", onclick:move |event:MouseEvent|{event.stop_propagation();pane_expanded.set(true);}, "Open equipment details" }
                Button { id:"r0216-pane-mode", variant:ButtonVariant::Secondary, onclick:move |event:MouseEvent|{event.stop_propagation();floating.toggle();},
                    if floating(){"Use inline pane"}else{"Use floating pane"} }
                Button { id:"r0216-pane-remount", variant:ButtonVariant::Secondary,
                    onclick:move |event:MouseEvent|{event.stop_propagation();pane_expanded.set(false);pane_generation.set(pane_generation()+1);}, "Reset details pane" }
            }
            section {
                h2 { "Equipment workspace" }
                div { id:"r0216-stage",
                    Application { class:"r0216-app", pinned_menu:true, start_expanded:false,
                        header:rsx! {ApplicationHeader {name:"Equipment",show_menu:false}},
                        menu:rsx! {Menu {application_name:"Equipment",active_overlay:overlay,start_expanded:false,pinned:true,
                            onbeforeopensettings:move |event:MenuOpenEvent|{settings_requests.set(settings_requests()+1);if veto(){event.prevent_default();}},
                            settings:rsx! {MenuSettings {onclose:move |_|close_events.set(close_events()+1),
                                WorkspaceField {id:"r0216-settings-note",label:"Station name",value:settings_note}
                                div {class:"application-focus-controls",
                                    Button {id:"r0216-switch-about",variant:ButtonVariant::Secondary,onclick:move |_|overlay.set(Some(MenuOverlayKind::About)),"Show About without closing"}
                                }
                            }},
                            about:rsx! {MenuAbout {onclose:move |_|close_events.set(close_events()+1),
                                h3 {"Equipment workspace"}
                                p {"A small application workspace built with registered Dioxus controls."}
                                Button {id:"r0216-about-action",variant:ButtonVariant::Secondary,onclick:move |_|applied.set(applied()+1),"Inspect application information"}
                            }},
                            MenuItem {label:"Equipment overview",icon:"home",close_on_select:false}
                        }},
                        PaneLayout {variant:if floating(){PaneVariant::Floating}else{PaneVariant::Inline},
                            right:rsx! {for generation in [pane_generation()] {
                                Pane {key:"{generation}",heading:"Equipment details",expanded:pane_expanded(),size:"320px",hide_on_collapse:true,
                                    close_on_click_outside:true,aria_label_collapse_close_button:"Close equipment details",
                                    onexpandedchange:move |open|pane_expanded.set(open),
                                    WorkspaceField {id:"r0216-pane-note",label:"Equipment name",value:pane_note,autofocus:true}
                                    p {"Edit this name, then use Escape or the close control to return to the opener."}
                                    Button {id:"r0216-pane-action",onclick:move |_|applied.set(applied()+1),"Apply equipment name"}
                                }
                            }},
                            div {class:"application-focus-content",
                                h3 {"Pump station overview"}
                                p {"Settings and About leave the workspace intact. The floating pane wraps only its keyboard boundaries; inline panes keep native Tab behavior."}
                                Button {id:"r0216-independent",variant:ButtonVariant::Secondary,"Independent destination"}
                            }
                        }
                    }
                }
                output {id:"r0216-state","data-overlay":format!("{:?}", overlay()),"data-veto":veto().to_string(),
                    "data-settings-requests":settings_requests(),"data-close-events":close_events(),
                    "data-pane-open":pane_expanded().to_string(),"data-floating":floating().to_string(),"data-pane-generation":pane_generation(),
                    "Station: {settings_note()}; equipment: {pane_note()}; actions: {applied()}"}
            }
            div {onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=super::dom_query::eval(NATIVE_APPLICATION_FOCUS_CHECK);let _=query.send("application-focus-review");
                    match query.join::<serde_json::Value>().await {
                        Ok(value)if value["ok"]==true=>{println!("Dioxus application mounted focus native: {value}");window.close();}
                        other=>{eprintln!("Dioxus application mounted focus native failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus application mounted focus native timed out");std::process::exit(1);});}
            }}
        }
    }
}

const STYLES: &str = "body{margin:0}.application-focus-review{min-height:100vh;box-sizing:border-box;padding:24px}.application-focus-review>h1{font-size:28px;margin:0 0 16px}.application-focus-review p{max-width:65ch;line-height:1.5}.application-focus-review section{max-width:1080px;margin:28px 0}.application-focus-review h2{font-size:20px}.application-focus-controls{display:flex;flex-wrap:wrap;align-items:center;gap:12px;margin:16px 0}.r0216-app{height:460px;min-height:460px;border:1px solid var(--ix-color-border)}.r0216-app .ix-application-main{padding:0;min-width:0}.r0216-app .ix-pane-layout{height:100%;min-width:0}.application-focus-content{padding:20px;min-width:0;overflow-wrap:anywhere}.application-focus-content h3{margin:0 0 12px;font-size:18px}.application-focus-review label{display:block;margin-bottom:8px}.application-focus-field{box-sizing:border-box;width:100%;max-width:280px;min-height:32px;padding:4px 8px;background:var(--ix-color-component-1);color:inherit;border:1px solid var(--ix-color-border);font:inherit}.application-focus-field:focus-visible{outline:2px solid var(--ix-color-primary);outline-offset:2px}.application-focus-review output{display:block;margin-top:16px;overflow-wrap:anywhere}.r0216-app .ix-menu-overlay-content{min-width:0;max-width:100%;box-sizing:border-box}@media(max-width:600px){.application-focus-review{padding:16px}.application-focus-content{padding:12px}.r0216-app{height:480px;min-height:480px}}";

// Synthetic Linux events and focus reads only. Input values may be edited;
// production Dioxus handlers own all state, cue and ARIA updates. Physical Tab
// traversal and pointer interactions are verified in Chromium by the lead.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_APPLICATION_FOCUS_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),active=()=>document.activeElement;
const wait=async(label,predicate)=>{for(let n=0;n<220;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' focus='+active()?.id);};
const key=(node,value,shiftKey=false)=>{const event=new KeyboardEvent('keydown',{key:value,code:value,shiftKey,bubbles:true,cancelable:true});node.dispatchEvent(event);return event;};
const app=()=>document.querySelector('.r0216-app'),menu=()=>app().querySelector('[data-menu-root]'),utility=label=>menu().querySelector('[aria-label="'+label+'"]'),overlay=()=>app().querySelector('.ix-menu-overlay-panel');
const settings=()=>utility('Settings'),about=()=>utility('About & legal information'),overlayClose=()=>overlay()?.querySelector('.ix-menu-overlay-close');
const pane=()=>app().querySelector('.ix-pane'),paneClose=()=>pane()?.querySelector('.ix-pane-header button'),paneOpen=()=>pane()?.getAttribute('data-expanded')==='true';
const paneFocusReady=()=>wait('Pane focus facts committed',()=>pane()?.getAttribute('data-pane-focus-current')===active()?.id);
await wait('application controls mounted',()=>settings()&&about()&&pane());
settings().focus();settings().click();await wait('Settings opening focus',()=>overlay()?.getAttribute('aria-label')==='Settings'&&active()===overlayClose());
const field=byId('r0216-settings-note');field.focus();field.value='Station revised';field.dispatchEvent(new Event('input',{bubbles:true}));
await wait('Settings editing',()=>byId('r0216-state').textContent.includes('Station revised'));
if(key(field,'Tab').defaultPrevented)throw new Error('Settings interior Tab was intercepted');
overlayClose().click();await wait('Settings close returns registered invoker',()=>!overlay()&&active()===settings());
about().focus();about().click();await wait('About opening focus',()=>overlay()?.getAttribute('aria-label')==='About & legal information'&&active()===overlayClose());
key(active(),'Escape');await wait('About Escape return',()=>!overlay()&&active()===about());
settings().focus();settings().click();await wait('direct switch setup',()=>overlay()&&active()===overlayClose());
byId('r0216-switch-about').click();await wait('direct switch keeps usable About focus',()=>overlay()?.getAttribute('aria-label')==='About & legal information'&&active()===overlayClose());
overlayClose().click();await wait('direct switch returns original Settings invoker',()=>!overlay()&&active()===settings());
settings().focus();settings().click();await wait('outside-close setup',()=>overlay()&&active()===overlayClose());
const outsideClose=byId('r0216-overlay-outside-close');outsideClose.focus();outsideClose.click();await wait('external close preserves outside focus',()=>!overlay()&&active()===outsideClose);
byId('r0216-veto').click();await wait('Settings veto enabled',()=>byId('r0216-state').dataset.veto==='true');
const requests=+byId('r0216-state').dataset.settingsRequests;settings().focus();settings().click();
await wait('vetoed Settings does not open',()=>+byId('r0216-state').dataset.settingsRequests>requests&&!overlay()&&active()===settings());
byId('r0216-veto').click();await wait('Settings veto released',()=>byId('r0216-state').dataset.veto==='false');
const opener=byId('r0216-pane-open');opener.focus();opener.click();
await wait('floating Pane registered autofocus',()=>paneOpen()&&active()===byId('r0216-pane-note'));
const paneField=byId('r0216-pane-note');paneField.value='Pump revised';paneField.dispatchEvent(new Event('input',{bubbles:true}));
await wait('Pane editing',()=>byId('r0216-state').textContent.includes('Pump revised'));
await paneFocusReady();if(key(paneField,'Tab').defaultPrevented)throw new Error('Pane interior Tab was intercepted');
byId('r0216-pane-action').focus();await paneFocusReady();if(!key(active(),'Tab').defaultPrevented)throw new Error('Pane forward boundary was not trapped');
await wait('Pane forward wrap',()=>active()===paneClose());
await paneFocusReady();if(!key(active(),'Tab',true).defaultPrevented)throw new Error('Pane reverse boundary was not trapped');
await wait('Pane reverse wrap',()=>active()===byId('r0216-pane-action'));
key(active(),'Escape');await wait('Pane Escape return',()=>!paneOpen()&&active()===opener);
opener.focus();opener.click();await wait('Pane close setup',()=>paneOpen()&&active()===byId('r0216-pane-note'));
paneClose().click();await wait('Pane built-in close return',()=>!paneOpen()&&active()===opener);
opener.focus();opener.click();await wait('Pane outside setup',()=>paneOpen()&&active()===byId('r0216-pane-note'));
const outside=byId('r0216-independent');outside.focus();outside.click();await wait('Pane outside close preserves destination',()=>!paneOpen()&&active()===outside);
byId('r0216-pane-mode').click();await wait('inline Pane selected',()=>pane().getAttribute('data-floating')==='false');
opener.focus();opener.click();await wait('inline Pane opens without focus trap',()=>paneOpen()&&active()===opener);
byId('r0216-pane-action').focus();if(key(active(),'Tab').defaultPrevented)throw new Error('inline Pane intercepted Tab');
paneClose().click();await wait('inline Pane closed',()=>!paneOpen());
byId('r0216-pane-mode').click();await wait('floating Pane restored',()=>pane().getAttribute('data-floating')==='true');
opener.focus();opener.click();await wait('Pane remount setup',()=>paneOpen()&&active()===byId('r0216-pane-note'));
const oldPane=pane(),reset=byId('r0216-pane-remount');reset.focus();reset.click();
await wait('closed remount rejects stale Pane focus',()=>pane()!==oldPane&&!paneOpen()&&active()===reset);
return {ok:true,menu:{settings:true,about:true,closeReturn:true,escapeReturn:true,directSwitch:true,outsidePreserved:true,veto:true,editing:true},pane:{autofocus:true,editing:true,boundaries:2,interiorDefault:true,escapeReturn:true,closeReturn:true,outsidePreserved:true,inlineNoTrap:true,remount:true},stateWrites:0,nativeTabTraversal:'Chromium only'};
"#;
