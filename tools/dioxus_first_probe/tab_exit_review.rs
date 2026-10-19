//! Excluded outside-Tab workflow preview and synthetic Linux driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn ExitTrigger(dropdown: DropdownHandle) -> Element {
    let binding = use_dropdown_trigger_binding(dropdown);
    rsx! { button {
        id:"r0210-custom-trigger", r#type:"button", class:"ix-button ix-button-secondary",
        onmounted:binding.onmounted,onvisible:binding.onvisible,onkeydown:binding.onkeydown,onclick:binding.onclick, "aria-haspopup":"menu", "aria-expanded":dropdown.is_open().to_string(),
        ..binding.attributes(), "Custom equipment actions"
    } }
}

#[component]
pub fn TabExitPreview() -> Element {
    let custom = use_dropdown();
    let popover = use_popover();
    let field_mount = use_focus_mount("r0210-field");
    let button_mount = use_focus_mount("r0210-native-after");
    let parent_mount = use_focus_mount("r0210-parent-after");
    let mut destination = use_signal(|| 0_usize);
    let mut veto = use_signal(|| false);
    let mut value = use_signal(String::new);
    let mut actions = use_signal(|| 0_usize);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {"body{margin:0}.tab-exit-review{min-height:100vh;box-sizing:border-box;padding:24px}.tab-exit-review h1{font-size:28px;margin:0 0 16px}.tab-exit-review p{max-width:65ch;line-height:1.5}.tab-exit-review section{max-width:780px;margin:28px 0;padding-bottom:24px;border-bottom:1px solid var(--ix-color-border)}.tab-exit-review h2{font-size:20px}.exit-controls{display:flex;flex-wrap:wrap;align-items:center;gap:12px;margin:16px 0}.exit-field{box-sizing:border-box;min-height:32px;max-width:100%;padding:4px 8px;background:var(--ix-color-component-1);color:inherit;border:1px solid var(--ix-color-border);font:inherit}.exit-field:focus-visible{outline:2px solid var(--ix-color-primary);outline-offset:2px}.tab-exit-review output{display:block;margin-top:12px}.exit-parent-content{display:flex;flex-wrap:wrap;gap:12px;align-items:center}@media(max-width:600px){.tab-exit-review{padding:16px}}"} }
        IxTheme { class:"tab-exit-review",
            h1 { "Dioxus-owned Tab exits" }
            p { "Tab leaves a roving menu for the adjacent registered control. Shift+Tab returns to its trigger, including when a close request is vetoed." }
            section {
                h2 { "Registered outside destinations" }
                div { class:"exit-controls",
                    label { r#for:"r0210-destination", "Destination" }
                    select { id:"r0210-destination", class:"exit-field", value:destination().to_string(),
                        onchange:move |event|destination.set(event.value().parse().unwrap_or(0)),
                        option { value:"0", "Native input" }
                        option { value:"1", "Native button" }
                        option { value:"2", "Library Button" }
                        option { value:"3", "Library IconButton" }
                        option { value:"4", "Unregistered reference" }
                    }
                    label { input { id:"r0210-veto", r#type:"checkbox", checked:veto(), onchange:move |_|veto.set(!veto()) } " Veto menu close" }
                }
                ExitTrigger { dropdown:custom }
                Dropdown { controller:custom, trigger:Some("r0210-custom-trigger".into()), label:"Custom Tab exit actions", class:"r0210-custom",
                    navigation_mode:DropdownNavigationMode::RovingTabindex,
                    onbeforechange:move |event:DropdownChangeEvent|if !event.open&&veto(){event.prevent_default();},
                    DropdownItem { id:"r0210-custom-first", label:"Inspect equipment", keep_open:true }
                    DropdownItem { id:"r0210-custom-last", label:"Export equipment", keep_open:true }
                }
                div { class:"exit-controls",
                    input { id:"r0210-field", class:"exit-field", "aria-label":"Equipment note", placeholder:"Equipment note", disabled:destination()!=0,
                        value:value(), onmounted:field_mount, oninput:move |event|value.set(event.value()) }
                    button { id:"r0210-native-after", class:"ix-button ix-button-secondary", r#type:"button", disabled:destination()!=1,
                        onmounted:button_mount, onclick:move |_|actions.set(actions()+1), "Native action" }
                    Button { id:"r0210-library-after", variant:ButtonVariant::Secondary, disabled:destination()!=2,
                        onclick:move |_|actions.set(actions()+1), "Library action" }
                    IconButton { id:"r0210-icon-after", icon:"refresh", label:"Refresh equipment", disabled:destination()!=3,
                        onclick:move |_|actions.set(actions()+1) }
                    button { id:"r0210-unregistered", class:"ix-button ix-button-secondary", r#type:"button", disabled:destination()!=4, "Unregistered reference" }
                }
                output { id:"r0210-state", "data-veto":veto().to_string(), "Note: {value()}; actions: {actions()}" }
            }
            section {
                h2 { "Built-in trigger return" }
                Dropdown { label:"Built-in Tab exit actions", class:"r0210-builtin", navigation_mode:DropdownNavigationMode::RovingTabindex,
                    DropdownItem { id:"r0210-builtin-first", label:"Built-in first action", keep_open:true }
                    DropdownItem { label:"Built-in last action", keep_open:true }
                }
            }
            section {
                h2 { "Header overflow return" }
                ApplicationHeader { name:"Equipment", aria_label_more_menu_icon_button:"Equipment overflow actions",
                    overflow:rsx! { Dropdown { label:"Header Tab exit actions", class:"r0210-header-child", navigation_mode:DropdownNavigationMode::RovingTabindex,
                        DropdownItem { id:"r0210-header-first", label:"Header equipment action", keep_open:true }
                    } }
                }
            }
            section {
                h2 { "Exit to the parent Popover" }
                div { id:"r0210-popover-trigger", PopoverTrigger { popover, "Open equipment panel" } }
                Popover { popover, label:"Parent equipment panel",
                    div { class:"exit-parent-content",
                        Dropdown { label:"Nested Tab exit actions", class:"r0210-nested", navigation_mode:DropdownNavigationMode::RovingTabindex,
                            DropdownItem { id:"r0210-nested-first", label:"Nested first action", keep_open:true }
                            DropdownItem { id:"r0210-nested-last", label:"Nested last action", keep_open:true }
                        }
                        button { id:"r0210-parent-after", class:"ix-button ix-button-secondary", r#type:"button", onmounted:parent_mount,
                            onclick:move |_|actions.set(actions()+1), "Parent panel action" }
                    }
                }
            }
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=super::dom_query::eval(NATIVE_TAB_EXIT_CHECK);let _=query.send("tab-exit-review");
                    match query.join::<serde_json::Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dioxus Tab exit native: {value}");window.close();}
                        other=>{eprintln!("Dioxus Tab exit native failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus Tab exit native timed out");std::process::exit(1);});}
            } }
        }
    }
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_TAB_EXIT_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),surface=()=>document.querySelector('.r0210-custom');
// Separate synthetic actions from asynchronous typed observation delivery.
const settleInput=()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));
const wait=async(label,predicate)=>{for(let n=0;n<200;n++){if(predicate()){await settleInput();return;}await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' focus='+document.activeElement?.id);};
const key=(node,value,shiftKey=false)=>{const event=new KeyboardEvent('keydown',{key:value,shiftKey,bubbles:true,cancelable:true});node.dispatchEvent(event);return event;};
const enter=async(root,first)=>{root.focus();key(root,'Home');await wait('menu item '+first,()=>document.activeElement===byId(first));};
const choose=async(index,id)=>{const select=byId('r0210-destination');select.value=String(index);select.dispatchEvent(new Event('change',{bubbles:true}));await wait('destination '+id,()=>byId(id)&&!byId(id).disabled);};
await wait('custom trigger and menu mounted',()=>byId('r0210-custom-trigger')?.getBoundingClientRect().width>0&&surface()?.querySelector('[role=menu]')?.isConnected);
const destinations=['r0210-field','r0210-native-after','r0210-library-after','r0210-icon-after'];
for(const [index,id]of destinations.entries()){
 await choose(index,id);byId('r0210-custom-trigger').scrollIntoView({block:'center'});
 await enter(byId('r0210-custom-trigger'),'r0210-custom-first');
 const scroll=window.scrollY,event=key(document.activeElement,'Tab');if(!event.defaultPrevented)throw new Error('registered Tab exit not prevented: '+id);
 await wait('registered forward '+id,()=>surface().getAttribute('data-ix-overlay-open')==='false'&&document.activeElement===byId(id));
 if(window.scrollY!==scroll)throw new Error('visible outside exit changed document scroll: '+id);
}
await choose(0,'r0210-field');await enter(byId('r0210-custom-trigger'),'r0210-custom-first');
const reverse=key(document.activeElement,'Tab',true);if(!reverse.defaultPrevented)throw new Error('custom reverse Tab not prevented');
await wait('custom reverse trigger',()=>surface().getAttribute('data-ix-overlay-open')==='false'&&document.activeElement===byId('r0210-custom-trigger'));
byId('r0210-veto').click();await wait('veto enabled',()=>byId('r0210-state').getAttribute('data-veto')==='true');
await enter(byId('r0210-custom-trigger'),'r0210-custom-first');
const vetoExit=key(document.activeElement,'Tab');if(!vetoExit.defaultPrevented)throw new Error('veto exit not prevented');
await wait('veto destination preserved',()=>surface().getAttribute('data-ix-overlay-open')==='true'&&document.activeElement===byId('r0210-field'));
byId('r0210-field').value='Pump';byId('r0210-field').dispatchEvent(new Event('input',{bubbles:true}));await wait('outside input usable',()=>byId('r0210-state').textContent.includes('Note: Pump'));
byId('r0210-veto').click();await wait('veto disabled',()=>byId('r0210-state').getAttribute('data-veto')==='false');
key(byId('r0210-custom-trigger'),'Escape');await wait('veto cleanup',()=>surface().getAttribute('data-ix-overlay-open')==='false');
await choose(4,'r0210-unregistered');await enter(byId('r0210-custom-trigger'),'r0210-custom-first');
const unsupported=key(document.activeElement,'Tab');if(unsupported.defaultPrevented)throw new Error('unregistered destination claimed programmatic focus');
await wait('unregistered exit closes',()=>surface().getAttribute('data-ix-overlay-open')==='false');
const builtin=document.querySelector('.r0210-builtin');await enter(builtin,'r0210-builtin-first');
const builtinReverse=key(document.activeElement,'Tab',true);if(!builtinReverse.defaultPrevented)throw new Error('built-in reverse Tab not prevented');
await wait('built-in reverse trigger',()=>builtin.getAttribute('data-ix-overlay-open')==='false'&&document.activeElement===builtin);
const more=document.querySelector('[aria-label="Equipment overflow actions"]');more.click();
await wait('header overflow open',()=>more.getAttribute('aria-expanded')==='true');
const headerChild=document.querySelector('.r0210-header-child');await enter(headerChild,'r0210-header-first');
const headerExit=key(document.activeElement,'Tab',true);if(!headerExit.defaultPrevented)throw new Error('Header reverse Tab not prevented');
await wait('registered Header More return',()=>more.getAttribute('aria-expanded')==='false'&&document.activeElement===more);
byId('r0210-popover-trigger').querySelector('button').click();const panel=document.querySelector('[aria-label="Parent equipment panel"]');
await wait('parent Popover open',()=>panel?.matches(':popover-open'));
const nested=document.querySelector('.r0210-nested');await enter(nested,'r0210-nested-first');
const nestedExit=key(document.activeElement,'Tab');if(!nestedExit.defaultPrevented)throw new Error('parent destination Tab not prevented');
await wait('parent registered destination',()=>nested.getAttribute('data-ix-overlay-open')==='false'&&document.activeElement===byId('r0210-parent-after'));
if(!panel.matches(':popover-open'))throw new Error('Tab exit closed parent Popover');
return {ok:true,forwardDestinations:destinations.length,customReverse:true,builtinReverse:true,headerMoreReverse:true,closeVeto:true,outsideInput:true,visibleExitScrollPreserved:true,unregisteredDefault:true,parentPopover:true};
"#;
