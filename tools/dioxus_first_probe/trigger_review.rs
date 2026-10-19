//! Excluded R02 trigger-focus workflow preview and synthetic Linux driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn CustomTrigger(dropdown: DropdownHandle, revision: usize) -> Element {
    let binding = use_dropdown_trigger_binding(dropdown);
    rsx! { button {
        id:"r02-custom-trigger", r#type:"button", class:"ix-button ix-button-secondary",
        onmounted:binding.onmounted,onvisible:binding.onvisible,onkeydown:binding.onkeydown,onclick:binding.onclick, "aria-haspopup":"menu", "aria-expanded":dropdown.is_open().to_string(), ..binding.attributes(), "Custom equipment actions {revision}"
    } }
}

#[component]
pub fn TriggerPreview() -> Element {
    let custom = use_dropdown();
    let builtin = use_dropdown();
    let child = use_dropdown();
    let second_child = use_dropdown();
    let late_child = use_dropdown();
    let mut tools_loaded = use_signal(|| false);
    let mut revision = use_signal(|| 1_usize);
    let mut roving = use_signal(|| true);
    let mut show = use_signal(|| true);
    let mut count = use_signal(|| 0_usize);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {"body{margin:0}.trigger-review{min-height:100vh;padding:24px;box-sizing:border-box}.trigger-review>h1{font-size:28px;margin:0 0 16px}.trigger-review>p{max-width:65ch;line-height:1.5}.trigger-review section{max-width:780px;margin:32px 0;padding-bottom:24px;border-bottom:1px solid var(--ix-color-border)}.trigger-review h2{font-size:20px}.trigger-actions{display:flex;flex-wrap:wrap;gap:12px;margin:16px 0}.trigger-review output{display:block;margin-top:16px}@media(max-width:600px){.trigger-review{padding:16px}}"} }
        IxTheme { class:"trigger-review",
            h1 { "Dioxus-owned trigger focus" }
            p { "Close an action menu with Escape to return to its trigger. Custom triggers register a mounted Dioxus element." }
            section {
                h2 { "Custom trigger and current ownership" }
                if show() { CustomTrigger { key:"{revision()}", dropdown:custom, revision:revision() } }
                Dropdown { controller:custom, trigger:Some("r02-custom-trigger".into()), label:"Custom equipment actions", class:"r02-custom-surface", enable_top_layer:true,
                    navigation_mode:if roving(){DropdownNavigationMode::RovingTabindex}else{DropdownNavigationMode::ActiveDescendant},
                    DropdownItem { id:"r02-custom-first", label:"Inspect equipment", keep_open:true, onitemclick:move |_|count.set(count()+1) }
                    DropdownItem { id:"r02-custom-mode", label:"Switch navigation mode", keep_open:true, onitemclick:move |_|roving.set(!roving()) }
                    DropdownItem { label:"Unavailable action", disabled:true }
                    label { style:"display:flex;gap:8px;padding:12px;align-items:center",
                        input { id:"r0223-native-check", r#type:"checkbox" } "Include archived equipment"
                    }
                }
                div { class:"trigger-actions",
                    button { id:"r02-replace-trigger", class:"ix-button ix-button-secondary", r#type:"button", onclick:move |_|revision.set(revision()+1), "Replace custom trigger" }
                    button { id:"r02-toggle-trigger", class:"ix-button ix-button-secondary", r#type:"button", onclick:move |_|show.set(!show()), if show(){"Remove custom trigger"}else{"Restore custom trigger"} }
                    button { id:"r02-close-return", class:"ix-button ix-button-secondary", r#type:"button", onclick:move |_|custom.close(true), "Close and return focus" }
                    button { id:"r02-outside-trigger", class:"ix-button ix-button-secondary", r#type:"button", "Independent destination" }
                }
                output { id:"r02-trigger-state", "Inspections: {count()}; navigation: {roving()}; revision: {revision()}" }
            }
            section {
                h2 { "Library trigger" }
                DropdownTrigger { dropdown:builtin, "Library equipment actions" }
                Dropdown { controller:builtin, label:"Library equipment actions", class:"r02-library-surface", navigation_mode:DropdownNavigationMode::RovingTabindex,
                    DropdownItem { id:"r02-library-first", label:"Library action" }
                }
            }
            section {
                h2 { "Linked submenus and loaded actions" }
                Dropdown { label:"Linked navigation", class:"r02-linked-parent", navigation_mode:DropdownNavigationMode::RovingTabindex,
                    DropdownItem { id:"r02-linked-trigger", label:"Open child actions", dropdown:child }
                    DropdownItem { id:"r0222-second-trigger", label:"Open maintenance actions", dropdown:second_child }
                    DropdownItem { id:"r0222-load", label:"Load inspection tools", keep_open:true, onitemclick:move |_|tools_loaded.set(true) }
                    if tools_loaded() { DropdownItem { id:"r0222-late-trigger", label:"Open inspection tools", dropdown:late_child } }
                }
                Dropdown { controller:child, label:"Child equipment actions", class:"r02-linked-child", navigation_mode:DropdownNavigationMode::RovingTabindex,
                    DropdownItem { id:"r02-linked-first", label:"Child action" }
                }
                Dropdown { controller:second_child, label:"Maintenance actions", class:"r0222-second-child", navigation_mode:DropdownNavigationMode::RovingTabindex,
                    DropdownItem { id:"r0222-second-first", label:"Review maintenance history" }
                }
                if tools_loaded() {
                    Dropdown { controller:late_child, label:"Inspection tools", class:"r0222-late-child", navigation_mode:DropdownNavigationMode::RovingTabindex,
                        DropdownItem { id:"r0222-late-first", label:"Read sensor status" }
                    }
                }
                p { "Each row opens its own menu. Load inspection tools while this menu is open to add another group of actions." }
            }
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=super::dom_query::eval(NATIVE_TRIGGER_CHECK);let _=query.send("trigger-review");
                    match query.join::<serde_json::Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dioxus trigger native: {value}");window.close();}
                        other=>{eprintln!("Dioxus trigger native failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus trigger native timed out");std::process::exit(1);});}
            } }
        }
    }
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_TRIGGER_CHECK: &str = r#"
await dioxus.recv();
const wait=async(label,predicate)=>{for(let n=0;n<200;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' focus='+document.activeElement?.id);};
const byId=id=>document.getElementById(id),key=value=>document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:value,bubbles:true,cancelable:true}));
const surface=()=>document.querySelector('.r02-custom-surface'),menu=()=>surface().querySelector('[role=menu]');
await wait('custom trigger mount',()=>byId('r02-custom-trigger'));
byId('r02-custom-trigger').click();await wait('custom open and presented',()=>surface().getAttribute('data-ix-overlay-open')==='true'&&menu()?.matches(':popover-open')&&getComputedStyle(menu()).visibility==='visible');
menu().focus();key('Home');await wait('custom item focus',()=>document.activeElement===byId('r02-custom-first'));
key('Escape');await wait('custom Escape return',()=>surface().getAttribute('data-ix-overlay-open')==='false'&&document.activeElement===byId('r02-custom-trigger'));
byId('r02-custom-trigger').click();await wait('custom reopen',()=>surface().getAttribute('data-ix-overlay-open')==='true');
menu().focus();key('Home');await wait('mode setup',()=>document.activeElement===byId('r02-custom-first'));
byId('r02-custom-mode').click();await wait('live AD trigger focus',()=>surface().getAttribute('data-ix-overlay-mode')==='active-descendant'&&document.activeElement===byId('r02-custom-trigger'));
key('Escape');await wait('AD close',()=>surface().getAttribute('data-ix-overlay-open')==='false');
byId('r02-custom-trigger').click();await wait('outside setup',()=>surface().getAttribute('data-ix-overlay-open')==='true');
byId('r02-outside-trigger').focus();byId('r02-outside-trigger').click();
await wait('outside focus preserved',()=>surface().getAttribute('data-ix-overlay-open')==='false'&&document.activeElement===byId('r02-outside-trigger'));
byId('r02-custom-trigger').click();await wait('native checkbox setup',()=>surface().getAttribute('data-ix-overlay-open')==='true');
byId('r0223-native-check').click();await wait('native checkbox default survives inside dismissal',()=>byId('r0223-native-check').checked&&surface().getAttribute('data-ix-overlay-open')==='false');
byId('r02-custom-trigger').click();await wait('programmatic return setup',()=>surface().getAttribute('data-ix-overlay-open')==='true');
byId('r02-close-return').click();await wait('explicit return',()=>surface().getAttribute('data-ix-overlay-open')==='false'&&document.activeElement===byId('r02-custom-trigger'));
const old=byId('r02-custom-trigger');byId('r02-replace-trigger').click();await wait('replacement mounted',()=>byId('r02-custom-trigger')!==old&&byId('r02-custom-trigger').textContent.includes('2'));
byId('r02-custom-trigger').click();await wait('replacement open',()=>surface().getAttribute('data-ix-overlay-open')==='true');
key('Escape');await wait('replacement current focus',()=>surface().getAttribute('data-ix-overlay-open')==='false'&&document.activeElement===byId('r02-custom-trigger'));
byId('r02-toggle-trigger').click();await wait('removed trigger',()=>!byId('r02-custom-trigger'));
byId('r02-toggle-trigger').click();await wait('remount trigger',()=>byId('r02-custom-trigger'));
const library=document.querySelector('.ix-dropdown-external-trigger');library.click();const libMenu=document.querySelector('.r02-library-surface [role=menu]');
await wait('library open',()=>library.getAttribute('aria-expanded')==='true');libMenu.focus();key('Home');await wait('library item',()=>document.activeElement===byId('r02-library-first'));key('Escape');await wait('library return',()=>document.activeElement===library&&library.getAttribute('aria-expanded')==='false');
const parent=document.querySelector('.r02-linked-parent'),child=document.querySelector('.r02-linked-child');parent.click();
await wait('parent open',()=>parent.getAttribute('aria-expanded')==='true');parent.querySelector('[role=menu]').focus();key('Home');await wait('linked trigger focus',()=>document.activeElement===byId('r02-linked-trigger'));
key('ArrowRight');await wait('child open/focus',()=>child.getAttribute('data-ix-overlay-open')==='true'&&document.activeElement===byId('r02-linked-first'));
key('ArrowLeft');await wait('linked return',()=>child.getAttribute('data-ix-overlay-open')==='false'&&document.activeElement===byId('r02-linked-trigger'));
key('ArrowDown');await wait('second linked row',()=>document.activeElement===byId('r0222-second-trigger'));
key('ArrowRight');await wait('correct second submenu',()=>document.querySelector('.r0222-second-child').getAttribute('data-ix-overlay-open')==='true'&&child.getAttribute('data-ix-overlay-open')==='false'&&document.activeElement===byId('r0222-second-first'));
key('ArrowLeft');await wait('second submenu return',()=>document.querySelector('.r0222-second-child').getAttribute('data-ix-overlay-open')==='false'&&document.activeElement===byId('r0222-second-trigger'));
key('End');await wait('load action focus',()=>document.activeElement===byId('r0222-load'));
key('Enter');await wait('late child mounted while parent open',()=>byId('r0222-late-trigger')&&document.querySelector('.r0222-late-child')&&parent.getAttribute('aria-expanded')==='true');
key('End');await wait('late linked row focus',()=>document.activeElement===byId('r0222-late-trigger'));
key('ArrowRight');await wait('late submenu opens',()=>document.querySelector('.r0222-late-child').getAttribute('data-ix-overlay-open')==='true'&&document.activeElement===byId('r0222-late-first'));
key('ArrowLeft');await wait('late submenu returns to current parent',()=>document.querySelector('.r0222-late-child').getAttribute('data-ix-overlay-open')==='false'&&document.activeElement===byId('r0222-late-trigger'));
key('Escape');await wait('parent return',()=>parent.getAttribute('aria-expanded')==='false'&&document.activeElement===parent);
return {ok:true,customEscape:true,liveMode:true,outsidePreserved:true,explicitReturn:true,replacement:true,remount:true,libraryEscape:true,linkedArrowLeft:true,distinctLinkedTargets:true,lateChildAfterParentOpen:true,nativeCheckboxDefault:true};
"#;
