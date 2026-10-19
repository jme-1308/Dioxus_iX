//! Excluded mixed-item workflows for Rust-rendered focus cues and ARIA.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn CustomCueTrigger(dropdown: DropdownHandle) -> Element {
    let binding = use_dropdown_trigger_binding(dropdown);
    rsx! { button { id:"r0213-custom-trigger", class:"ix-button ix-button-secondary", r#type:"button", onmounted:binding.onmounted,onvisible:binding.onvisible,onkeydown:binding.onkeydown,onclick:binding.onclick,
        "aria-haspopup":"menu", "aria-expanded":dropdown.is_open().to_string(), aria_activedescendant:dropdown.active_descendant(),
        ..binding.attributes(), "Custom equipment actions"
    } }
}

#[component]
pub fn CuePreview() -> Element {
    let mixed = use_dropdown();
    let child = use_dropdown();
    let custom = use_dropdown();
    let mut roving = use_signal(|| false);
    let mut revision = use_signal(|| 1_usize);
    let mut select_visible = use_signal(|| true);
    let mut actions = use_signal(|| 0_usize);
    let mut outside_note = use_signal(|| "Independent equipment note".to_string());
    let outside_mount = use_focus_mount("r0214-outside-field");
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class:"cue-review",
            h1 { "Dioxus-rendered menu focus" }
            p { "Mixed controls share one Rust focus state. The focused row, visible cue and active-descendant relationship stay aligned while the component updates." }
            section {
                h2 { "Mixed equipment actions" }
                p { "Use arrows, Home and End to inspect each control. F2 refreshes the labels, F3 removes the Select row and F4 restores it. F5 switches navigation mode." }
                div { id:"r0213-mixed-frame",
                    onkeydown:move |event:KeyboardEvent|match event.key() {
                        Key::F2=>{event.prevent_default();revision.set(revision()+1);},
                        Key::F3=>{event.prevent_default();select_visible.set(false);},
                        Key::F4=>{event.prevent_default();select_visible.set(true);},
                        Key::F5=>{event.prevent_default();roving.toggle();},
                        _=>{},
                    },
                    div { class:"cue-controls", id:"r0213-trigger", DropdownTrigger { dropdown:mixed, "Review equipment actions" } }
                    Dropdown { controller:mixed, label:"Mixed equipment actions", class:"r0213-mixed", close_behavior:DropdownCloseBehavior::None,
                        navigation_mode:if roving(){DropdownNavigationMode::RovingTabindex}else{DropdownNavigationMode::ActiveDescendant},
                        DropdownItem { id:"r0213-first", order:Some(0), label:format!("Inspect equipment {}",revision()), tabindex:0, keep_open:true,
                            onclick:move |_|actions.set(actions()+1) }
                        DropdownItem { id:"r0213-disabled", order:Some(1), label:"Unavailable action", disabled:true, tabindex:0 }
                        if select_visible() { SelectItem { id:"r0213-select", order:Some(2), value:"equipment", label:format!("Choose equipment {}",revision()), tabindex:0 } }
                        MenuItem { id:"r0213-menu", order:Some(3), label:format!("Equipment navigation {}",revision()), close_on_select:false }
                        MenuCategory { id:"r0213-category", order:Some(4), label:format!("Grouped equipment {}",revision()),
                            MenuItem { label:"Grouped detail" }
                        }
                        DropdownItem { id:"r0213-linked", order:Some(5), label:"Open linked equipment actions", dropdown:child, tabindex:0 }
                        DropdownItem { id:"r0213-hidden", order:Some(6), label:"Hidden action", hidden:true, tabindex:0 }
                    }
                    Dropdown { controller:child, label:"Linked equipment actions", class:"r0213-child",
                        navigation_mode:if roving(){DropdownNavigationMode::RovingTabindex}else{DropdownNavigationMode::ActiveDescendant},
                        MenuItem { id:"r0213-child-menu", label:"Inspect linked equipment", close_on_select:false }
                        DropdownItem { id:"r0213-child-last", label:"Export linked equipment", tabindex:0 }
                    }
                }
                div { class:"cue-controls",
                    Button { id:"r0213-mode", variant:ButtonVariant::Secondary, onclick:move |_|roving.toggle(),
                        if roving(){"Use active descendant"}else{"Use roving tabindex"} }
                    Button { id:"r0213-refresh", variant:ButtonVariant::Secondary, onclick:move |_|revision.set(revision()+1), "Refresh row labels" }
                    Button { id:"r0213-toggle-select", variant:ButtonVariant::Secondary, onclick:move |_|select_visible.toggle(),
                        if select_visible(){"Remove Select row"}else{"Restore Select row"} }
                }
                label { r#for:"r0214-outside-field", "Independent note" }
                input { id:"r0214-outside-field", class:"cue-review-field", value:outside_note(), onmounted:outside_mount,
                    oninput:move |event|outside_note.set(event.value()) }
                p { "Move focus between an open menu and this note. Keyboard focus shows a row cue; pointer focus keeps the normal pointer appearance." }
                output { id:"r0214-outside-state", "Note: {outside_note()}" }
                output { id:"r0213-state", "data-mode":if roving(){"roving"}else{"ad"}, "data-revision":revision(),
                    "data-select-visible":select_visible().to_string(),
                    "Mode: {roving():?}; revision: {revision()}; inspections: {actions()}" }
            }
            section {
                h2 { "Explicit custom trigger relationship" }
                p { "This mounted custom button reads its active-descendant relationship directly from the Dropdown handle." }
                CustomCueTrigger { dropdown:custom }
                Dropdown { controller:custom, trigger:Some("r0213-custom-trigger".into()), label:"Custom equipment actions", class:"r0213-custom",
                    DropdownItem { id:"r0213-custom-first", label:"Inspect custom equipment", tabindex:0 }
                    DropdownItem { id:"r0213-custom-last", label:"Export custom equipment", tabindex:0 }
                }
            }
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=super::dom_query::eval(NATIVE_CUE_CHECK);let _=query.send("cue-review");
                    match query.join::<serde_json::Value>().await {
                        Ok(value)if value["ok"]==true=>{println!("Dioxus rendered cues native: {value}");window.close();}
                        other=>{eprintln!("Dioxus rendered cues native failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus rendered cues native timed out");std::process::exit(1);});}
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.cue-review{min-height:100vh;box-sizing:border-box;padding:24px}.cue-review>h1{font-size:28px;margin:0 0 16px}.cue-review p{max-width:65ch;line-height:1.5}.cue-review section{max-width:900px;margin:28px 0;padding-bottom:24px;border-bottom:1px solid var(--ix-color-border)}.cue-review h2{font-size:20px}.cue-controls{display:flex;flex-wrap:wrap;align-items:center;gap:12px;margin:16px 0}.cue-review output{display:block;margin-top:12px}.cue-review label{display:block;margin:20px 0 8px}.cue-review-field{box-sizing:border-box;min-height:32px;width:280px;max-width:100%;padding:4px 8px;background:var(--ix-color-component-1);color:inherit;border:1px solid var(--ix-color-border);font:inherit}.cue-review-field:focus-visible{outline:2px solid var(--ix-color-primary);outline-offset:2px}.r0213-mixed [role=menu],.r0213-child [role=menu]{min-width:270px;max-width:calc(100vw - 40px)}@media(max-width:600px){.cue-review{padding:16px}}";

// Synthetic interactions only: cue classes, data-active and ARIA are never
// written by this driver. Direct focus/blur exercise real Dioxus focus events;
// synthetic Tab only checks prevention, not native traversal. The lead checks
// physical browser keyboard and pointer input.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_CUE_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),active=()=>document.activeElement;
const wait=async(label,predicate)=>{for(let n=0;n<200;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' focus='+active()?.id);};
const key=(node,value)=>{const event=new KeyboardEvent('keydown',{key:value,code:value,bubbles:true,cancelable:true});node.dispatchEvent(event);return event;};
const cue=node=>node?.getAttribute('data-active')==='true'&&node.classList.contains('ix-focused');
const inactive=node=>node?.getAttribute('data-active')!=='true'&&!node?.classList.contains('ix-focused');
const root=()=>document.querySelector('.r0213-mixed'),child=()=>document.querySelector('.r0213-child'),trigger=()=>byId('r0213-trigger').querySelector('button');
const agreement=(target,roving)=>[root(),trigger()].every(node=>node.getAttribute('aria-activedescendant')===(roving?null:target));
await wait('mixed cue trigger mounted',()=>trigger()&&root()?.getAttribute('data-ix-overlay-root'));
for(const roving of [false,true]){
 trigger().scrollIntoView({block:'center'});trigger().focus();key(trigger(),'Home');
 await wait('first rendered cue '+roving,()=>root().getAttribute('data-ix-overlay-open')==='true'&&active()===byId('r0213-first')&&cue(byId('r0213-first'))&&agreement('r0213-first',roving));
 const outside=byId('r0214-outside-field'),select=()=>byId('r0213-select'),option=()=>select().querySelector('[data-ix-item-part="select-option"]'),menu=()=>byId('r0213-menu-control'),parentControl=()=>byId('r0213-category').querySelector('[data-ix-item-part="category-parent"]');
 byId('r0213-first').blur();await wait('direct blur clears current item',()=>inactive(byId('r0213-first'))&&agreement(null,roving));
 key(outside,'Shift');select().focus();
 await wait('direct Select focus reaches Dioxus',()=>active()===select()&&cue(select())&&cue(option())&&agreement('r0213-select',roving));
 menu().focus();await wait('newer Menu focus survives previous Select blur',()=>active()===menu()&&cue(byId('r0213-menu'))&&inactive(select())&&inactive(option())&&agreement('r0213-menu',roving));
 parentControl().focus();await wait('direct physical Category focus',()=>active()===parentControl()&&cue(parentControl())&&inactive(byId('r0213-category'))&&inactive(byId('r0213-menu'))&&agreement(null,roving));
 outside.focus();await wait('outside focus clears only the departed cue',()=>active()===outside&&inactive(parentControl())&&inactive(byId('r0213-menu'))&&root().getAttribute('data-ix-overlay-open')==='true'&&agreement(null,roving));
 outside.value='Note edited '+(roving?'roving':'AD');outside.dispatchEvent(new Event('input',{bubbles:true}));
 await wait('outside typing does not restore an item cue',()=>byId('r0214-outside-state').textContent.includes(outside.value)&&inactive(parentControl())&&agreement(null,roving));
 outside.dispatchEvent(new MouseEvent('mousedown',{bubbles:true,cancelable:true}));select().focus();
 await wait('pointer-mode direct focus has no keyboard cue',()=>active()===select()&&inactive(select())&&inactive(option())&&agreement(null,roving));
 key(outside,'Shift');byId('r0213-first').focus();
 await wait('keyboard-mode direct Dropdown focus restores cue',()=>active()===byId('r0213-first')&&cue(byId('r0213-first'))&&inactive(select())&&agreement('r0213-first',roving));
 if(!roving&&key(active(),'Tab').defaultPrevented)throw new Error('ordinary interior AD Tab default was prevented');
 key(active(),'ArrowDown');await wait('Select host and option cue',()=>active()===byId('r0213-select')&&cue(byId('r0213-select'))&&cue(byId('r0213-select').querySelector('[data-ix-item-part="select-option"]'))&&agreement('r0213-select',roving));
 const revision=+byId('r0213-state').dataset.revision;key(active(),'F2');
 await wait('normal prop rerender preserves cue',()=>+byId('r0213-state').dataset.revision===revision+1&&cue(byId('r0213-select'))&&cue(byId('r0213-select').querySelector('[data-ix-item-part="select-option"]'))&&agreement('r0213-select',roving));
 key(active(),'F3');await wait('active Select unmount clears Rust cue',()=>!byId('r0213-select')&&agreement(null,roving));
 trigger().focus();key(trigger(),'F4');await wait('Select remount does not inherit cue',()=>byId('r0213-select')&&inactive(byId('r0213-select'))&&inactive(byId('r0213-select').querySelector('[data-ix-item-part="select-option"]')));
 key(trigger(),'Home');await wait('restart current navigation',()=>active()===byId('r0213-first')&&cue(byId('r0213-first')));
 key(active(),'ArrowDown');await wait('Select after remount',()=>active()===byId('r0213-select')&&cue(byId('r0213-select')));
 key(active(),'ArrowDown');await wait('Menu host and physical control',()=>active()===byId('r0213-menu-control')&&cue(byId('r0213-menu'))&&agreement('r0213-menu',roving));
 key(active(),'ArrowDown');const category=()=>byId('r0213-category').querySelector('[data-ix-item-part="category-parent"]');
 await wait('Category parent cue without outer ARIA',()=>active()===category()&&cue(category())&&inactive(byId('r0213-category'))&&agreement(null,roving));
 key(active(),'F2');await wait('Category prop rerender preserves parent cue',()=>cue(category())&&inactive(byId('r0213-category')));
 key(active(),'End');await wait('linked parent focus cue',()=>active()===byId('r0213-linked')&&cue(byId('r0213-linked'))&&agreement('r0213-linked',roving));
 key(active(),'ArrowRight');await wait('linked submenu rendered open cue',()=>child().getAttribute('data-ix-overlay-open')==='true'&&byId('r0213-linked').classList.contains('ix-dropdown-submenu-trigger-active')&&active()===byId('r0213-child-menu-control')&&cue(byId('r0213-child-menu')));
 key(active(),'ArrowLeft');await wait('submenu close returns coherent parent cue',()=>child().getAttribute('data-ix-overlay-open')==='false'&&!byId('r0213-linked').classList.contains('ix-dropdown-submenu-trigger-active')&&active()===byId('r0213-linked')&&cue(byId('r0213-linked'))&&agreement('r0213-linked',roving));
 key(active(),'Escape');await wait('closed menu clears ARIA',()=>root().getAttribute('data-ix-overlay-open')==='false'&&active()===trigger()&&agreement(null,roving));
 if(!roving){key(trigger(),'F5');await wait('roving mode selected',()=>byId('r0213-state').dataset.mode==='roving');}
}
const custom=byId('r0213-custom-trigger'),customRoot=()=>document.querySelector('.r0213-custom');
custom.scrollIntoView({block:'center'});custom.focus();key(custom,'Home');
await wait('custom Rust ARIA binding',()=>customRoot().getAttribute('data-ix-overlay-open')==='true'&&cue(byId('r0213-custom-first'))&&custom.getAttribute('aria-activedescendant')==='r0213-custom-first'&&customRoot().getAttribute('aria-activedescendant')==='r0213-custom-first');
key(active(),'End');await wait('custom Rust ARIA moves',()=>cue(byId('r0213-custom-last'))&&custom.getAttribute('aria-activedescendant')==='r0213-custom-last');
key(active(),'Escape');await wait('custom closed ARIA',()=>customRoot().getAttribute('data-ix-overlay-open')==='false'&&!custom.hasAttribute('aria-activedescendant'));
return {ok:true,modes:2,typedCue:true,selectPart:true,menuPhysicalFocus:true,categoryParent:true,rootTriggerAria:true,customRustAria:true,directFocusBlur:true,newerCuePreserved:true,outsideFocus:true,pointerCueClear:true,outsideEditing:true,interiorTabDefault:true,nativeTabTraversal:'Chromium only',rerender:true,unmountRemount:true,linkedOpenReturn:true,disabledHiddenSkip:true,cueWrites:0};
"#;
