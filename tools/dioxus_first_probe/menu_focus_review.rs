//! Excluded Menu and Category mounted-focus workflows.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn MenuFocusPreview() -> Element {
    let mut expanded = use_signal(|| false);
    let mut generation = use_signal(|| 1_usize);
    let mut second_child = use_signal(|| true);
    let mut last_action = use_signal(|| "None".to_string());
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class:"menu-focus-review",
            h1 { "Dioxus-owned Menu focus" }
            p { "Navigate the equipment menu with Home, End and the arrow keys. Open a Category with Enter or Space, then return from its children with Escape." }
            div { class:"menu-focus-controls",
                Button { id:"r0215-mode", variant:ButtonVariant::Secondary, onclick:move |_|expanded.toggle(),
                    if expanded(){"Use collapsed flyouts"}else{"Use expanded panels"} }
                Button { id:"r0215-child-toggle", variant:ButtonVariant::Secondary, onclick:move |_|second_child.toggle(),
                    if second_child(){"Remove second child"}else{"Restore second child"} }
                Button { id:"r0215-remount", variant:ButtonVariant::Secondary, onclick:move |_|generation.set(generation()+1), "Remount menu" }
            }
            section {
                h2 { "Equipment navigation" }
                p { "The unavailable and hidden root actions are skipped. Collapsed Categories use a flyout; expanded Categories keep a panel in the navigation tree." }
                div { id:"r0215-stage",
                    onkeydown:move |event:KeyboardEvent|match event.key() {
                        Key::F2=>{event.prevent_default();expanded.toggle();},
                        Key::F3=>{event.prevent_default();second_child.set(false);},
                        Key::F4=>{event.prevent_default();second_child.set(true);},
                        Key::F5=>{event.prevent_default();generation.set(generation()+1);},
                        _=>{},
                    },
                    Application { class:"r0215-app", menu_expanded:expanded, start_expanded:false, pinned_menu:true,
                        force_breakpoint:ApplicationBreakpoint::Large,
                        menu:rsx! {
                            for current in [generation()] {
                                Menu { key:"{current}", label:"Equipment navigation", application_name:"Equipment", start_expanded:expanded(), pinned:true,
                                    MenuItem { id:"r0215-first", label:"Equipment overview", icon:"home", close_on_select:false,
                                        onclick:move |_|last_action.set("Equipment overview".into()) }
                                    MenuItem { id:"r0215-disabled", label:"Unavailable action", icon:"lock", disabled:true }
                                    MenuItem { id:"r0215-hidden", label:"Hidden action", hidden:true }
                                    MenuItem { id:"r0215-history", label:"Service history", icon:"calendar", close_on_select:false,
                                        onclick:move |_|last_action.set("Service history".into()) }
                                    MenuCategory { id:"r0215-category", label:"Equipment group", icon:"folder",
                                        MenuItem { id:"r0215-child-one", label:"Inspect equipment", close_on_select:false,
                                            onclick:move |_|last_action.set("Inspect equipment".into()) }
                                        MenuItem { id:"r0215-child-disabled", label:"Unavailable child", disabled:true }
                                        if second_child() { MenuItem { id:"r0215-child-two", label:"Export equipment", close_on_select:false,
                                            onclick:move |_|last_action.set("Export equipment".into()) } }
                                    }
                                    MenuItem { id:"r0215-last", label:"Equipment settings", icon:"cogwheel", close_on_select:false,
                                        onclick:move |_|last_action.set("Equipment settings".into()) }
                                }
                            }
                        },
                        div { id:"r0215-content",
                            h3 { "Equipment workspace" }
                            p { "Keep this independent destination focused when changing or remounting the menu. Stale menu work must not reclaim focus." }
                            Button { id:"r0215-outside", variant:ButtonVariant::Secondary, "Independent destination" }
                            p { "Keyboard shortcuts in this stage: F2 switches mode, F3 removes the second child, F4 restores it and F5 remounts the menu." }
                        }
                    }
                }
                output { id:"r0215-state", "data-expanded":expanded().to_string(), "data-generation":generation(),
                    "data-second-child":second_child().to_string(), "Last action: {last_action()}" }
            }
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=super::dom_query::eval(NATIVE_MENU_CHECK);let _=query.send("menu-focus-review");
                    match query.join::<serde_json::Value>().await {
                        Ok(value)if value["ok"]==true=>{println!("Dioxus Menu mounted focus native: {value}");window.close();}
                        other=>{eprintln!("Dioxus Menu mounted focus native failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus Menu mounted focus native timed out");std::process::exit(1);});}
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.menu-focus-review{min-height:100vh;box-sizing:border-box;padding:24px}.menu-focus-review>h1{font-size:28px;margin:0 0 16px}.menu-focus-review p{max-width:65ch;line-height:1.5}.menu-focus-review section{max-width:980px;margin:28px 0;padding-bottom:24px;border-bottom:1px solid var(--ix-color-border)}.menu-focus-review h2{font-size:20px}.menu-focus-controls{display:flex;flex-wrap:wrap;align-items:center;gap:12px;margin:16px 0}.menu-focus-review output{display:block;margin-top:12px}.r0215-app{height:480px;min-height:480px;border:1px solid var(--ix-color-border)}.r0215-app .ix-application-main{padding:20px}.r0215-app h3{margin:0 0 12px;font-size:18px}.r0215-app .ix-application-main p{max-width:50ch}@media(max-width:600px){.menu-focus-review{padding:16px}.r0215-app .ix-application-main{padding:12px}.menu-focus-review .r0215-app[data-menu-expanded=true]{height:740px;min-height:740px}.menu-focus-review .r0215-app[data-menu-expanded=true] .ix-application-body{flex-direction:column}.menu-focus-review.ix-theme .r0215-app[data-menu-pinned=true][data-menu-expanded=true] .ix-application-menu-slot{width:100%;height:330px;position:relative}.menu-focus-review .r0215-app[data-menu-expanded=true] .ix-application-menu-slot .ix-menu{width:100%;position:relative;height:100%}}";

// Excluded synthetic Linux workflow. Native key and pointer events exercise the
// existing observations; only Dioxus production code moves focus or writes cues.
// Physical browser keyboard traversal is verified separately by the lead.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_MENU_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),active=()=>document.activeElement;
const wait=async(label,predicate)=>{for(let n=0;n<220;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' focus='+active()?.id);};
const key=(node,value)=>{const event=new KeyboardEvent('keydown',{key:value,code:value,bubbles:true,cancelable:true});node.dispatchEvent(event);return event;};
const root=()=>byId('r0215-stage').querySelector('[data-menu-root]'),bar=()=>root().querySelector('[role="menubar"]'),category=()=>byId('r0215-category'),parent=()=>category().querySelector('[data-ix-item-part="category-parent"]');
const popup=()=>category().getAttribute('data-menu-category-popup-open')==='true',panel=()=>category().getAttribute('data-menu-category-panel-open')==='true';
const focused=id=>active()===byId(id),ready=()=>root()?.getAttribute('data-menu-native')==='true'&&category()?.getAttribute('data-category-native')==='true';
const parentCue=()=>parent().getAttribute('data-active')==='true'&&parent().classList.contains('ix-focused');
await wait('native Menu observations ready',ready);
bar().scrollIntoView({block:'center'});bar().focus();key(bar(),'Home');await wait('root Home mounted focus',()=>focused('r0215-first'));
key(active(),'ArrowDown');await wait('root disabled and hidden skip',()=>focused('r0215-history'));
key(active(),'ArrowUp');await wait('root Up mounted focus',()=>focused('r0215-first'));
key(active(),'End');await wait('root End mounted focus',()=>focused('r0215-last'));
key(active(),'ArrowUp');await wait('root Category parent focus',()=>active()===parent());
key(parent(),'Enter');await wait('collapsed Category entry',()=>popup()&&focused('r0215-child-one'));
key(active(),'ArrowDown');await wait('collapsed child disabled skip',()=>focused('r0215-child-two'));
key(active(),'Home');await wait('collapsed child Home',()=>focused('r0215-child-one'));
key(active(),'End');await wait('collapsed child End',()=>focused('r0215-child-two'));
key(active(),'Enter');await wait('child Enter returns mounted parent',()=>!popup()&&active()===parent());
key(parent(),' ');await wait('parent Space opens collapsed Category',()=>popup()&&focused('r0215-child-one'));
key(active(),' ');await wait('child Space returns mounted parent',()=>!popup()&&active()===parent());
key(parent(),'Enter');await wait('Escape return setup',()=>popup()&&focused('r0215-child-one'));
key(active(),'Escape');await wait('collapsed Escape returns parent',()=>!popup()&&active()===parent());
await wait('Category keyboard cue',parentCue);
category().dispatchEvent(new PointerEvent('pointerenter',{pointerType:'mouse',relatedTarget:byId('r0215-outside')}));
await wait('collapsed hover opens flyout',popup);
key(parent(),'ArrowDown');await wait('hover return eligible child focus',()=>focused('r0215-child-one'));
category().dispatchEvent(new PointerEvent('pointerleave',{pointerType:'mouse',relatedTarget:byId('r0215-outside')}));
await wait('hover dismissal returns eligible mounted parent',()=>!popup()&&active()===parent());
key(parent(),'F2');await wait('expanded mode',()=>root().getAttribute('data-expanded')==='true'&&!popup());
parent().click();await wait('expanded Category panel open',panel);
parent().focus();key(parent(),'ArrowDown');await wait('expanded panel entry',()=>panel()&&focused('r0215-child-one'));
key(active(),'ArrowUp');await wait('expanded child wrap Up',()=>focused('r0215-child-two'));
key(active(),'ArrowDown');await wait('expanded child wrap Down',()=>focused('r0215-child-one'));
key(active(),'Escape');await wait('expanded Escape closes panel and returns parent',()=>!panel()&&active()===parent());
parent().click();await wait('removal setup panel',panel);parent().focus();key(parent(),'ArrowDown');await wait('removal setup child',()=>focused('r0215-child-one'));
key(active(),'ArrowUp');await wait('second child removal setup',()=>focused('r0215-child-two'));
key(active(),'F3');await wait('focused child removal',()=>!byId('r0215-child-two'));
const outside=byId('r0215-outside');outside.focus();key(outside,'F4');await wait('child remount has no stale focus',()=>byId('r0215-child-two')&&active()===outside);
const oldRoot=root(),oldParent=parent(),generation=+byId('r0215-state').dataset.generation;
byId('r0215-remount').click();outside.focus();
await wait('fresh mounted Menu and Category',()=>root()!==oldRoot&&parent()!==oldParent&&+byId('r0215-state').dataset.generation===generation+1&&ready());
if(active()!==outside)throw new Error('remount stole independent focus');
bar().focus();key(bar(),'Home');await wait('fresh root navigation uses current mount',()=>focused('r0215-first'));
return {ok:true,root:{home:true,end:true,upDown:true,disabledHiddenSkip:true},collapsed:{entry:true,childNavigation:true,enterReturn:true,spaceReturn:true,escapeReturn:true},expanded:{entry:true,wrap:true,escapeReturn:true},hoverEligibleReturn:true,categoryCue:true,childRemovalRemount:true,ownerRemount:true,cueWrites:0,rootOutline:getComputedStyle(active()).outlineWidth};
"#;
