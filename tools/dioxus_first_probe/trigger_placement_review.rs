//! Excluded Dioxus trigger-placement preview and synthetic Linux workflow driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn EquipmentTrigger(dropdown: DropdownHandle) -> Element {
    let binding = use_dropdown_trigger_binding(dropdown);
    rsx! {button {
        id:"r0247-custom-trigger",r#type:"button",class:"ix-button ix-button-secondary",
        "aria-haspopup":"menu","aria-expanded":dropdown.is_open().to_string(),
        onmounted:binding.onmounted,onvisible:binding.onvisible,onkeydown:binding.onkeydown,onclick:binding.onclick,
        ..binding.attributes(),"Inspection tools"
    }}
}

#[component]
pub fn TriggerPlacementReview() -> Element {
    let linked = use_dropdown();
    let custom = use_dropdown();
    let mut suppressed = use_signal(|| false);
    let mut nested = use_signal(|| true);
    let mut revision = use_signal(|| 0_usize);
    let mut actions = use_signal(|| 0_usize);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style {{STYLES}}
        IxTheme {class:"trigger-placement-review",
            h1 {"Equipment action placement"}
            p {"Standalone menus follow their configured placement. Linked rows and triggers composed inside an action row open their menus beside that row, using Dioxus-owned relationships."}
            div {class:"r0247-controls",
                Button {id:"r0247-suppression",variant:ButtonVariant::Secondary,onclick:move |_|suppressed.toggle(),if suppressed(){"Restore visibility checking"}else{"Suppress visibility checking"}}
                Button {id:"r0247-location",variant:ButtonVariant::Secondary,onclick:move |_|{nested.toggle();revision+=1;},if nested(){"Move inspection trigger outside"}else{"Return inspection trigger to row"}}
                Button {id:"r0247-remount",variant:ButtonVariant::Secondary,onclick:move |_|revision+=1,"Replace inspection trigger"}
            }
            section {
                h2 {"Standalone equipment actions"}
                p {"This menu requests bottom-end placement. The inspection trigger can also be moved here to check that its former row relationship is released."}
                div {class:"r0247-actions",
                    Dropdown {label:"Equipment overview",class:"r0247-standalone",placement:DropdownPlacement::BottomEnd,enable_top_layer:true,
                        navigation_mode:DropdownNavigationMode::RovingTabindex,suppress_trigger_visibility_check:suppressed(),
                        DropdownItem {id:"r0247-overview-action",label:"Read equipment status",onitemclick:move |_|actions+=1}
                        DropdownItem {id:"r0247-overview-disabled",label:"Unavailable export",disabled:true}
                    }
                    if !nested() {EquipmentTrigger {key:"outside-{revision()}",dropdown:custom}}
                }
            }
            section {
                h2 {"Maintenance workspace"}
                p {"Open Maintenance actions, then choose the linked row, inspection button or built-in sensor trigger. Escape closes an action menu; the linked row also supports ArrowRight and ArrowLeft."}
                Dropdown {label:"Maintenance actions",class:"r0247-parent",enable_top_layer:true,navigation_mode:DropdownNavigationMode::RovingTabindex,
                    DropdownItem {id:"r0247-linked-trigger",label:"Service actions",dropdown:linked}
                    DropdownItem {id:"r0247-custom-row",label:"Inspection",keep_open:true,
                        if nested() {EquipmentTrigger {key:"inside-{revision()}",dropdown:custom}}
                    }
                    DropdownItem {id:"r0247-internal-row",label:"Sensors",keep_open:true,
                        Dropdown {label:"Sensor actions",class:"r0247-internal",placement:DropdownPlacement::BottomEnd,enable_top_layer:true,
                            navigation_mode:DropdownNavigationMode::RovingTabindex,suppress_trigger_visibility_check:suppressed(),
                            DropdownItem {id:"r0247-sensor-action",label:"Read sensor values",onitemclick:move |_|actions+=1}
                        }
                    }
                }
                Dropdown {controller:linked,label:"Service actions",class:"r0247-linked",placement:DropdownPlacement::BottomEnd,enable_top_layer:true,
                    navigation_mode:DropdownNavigationMode::RovingTabindex,suppress_trigger_visibility_check:suppressed(),
                    DropdownItem {id:"r0247-service-action",label:"Review service history",onitemclick:move |_|actions+=1}
                }
                Dropdown {controller:custom,trigger:Some("r0247-custom-trigger".into()),label:"Inspection tools",class:"r0247-custom",placement:DropdownPlacement::BottomEnd,enable_top_layer:true,
                    navigation_mode:DropdownNavigationMode::RovingTabindex,suppress_trigger_visibility_check:suppressed(),
                    DropdownItem {id:"r0247-inspection-action",label:"Inspect equipment",onitemclick:move |_|actions+=1}
                }
            }
            output {id:"r0247-state",role:"status","aria-live":"polite","data-suppressed":suppressed().to_string(),"data-nested":nested().to_string(),"data-revision":revision().to_string(),"data-actions":actions().to_string(),
                "Completed actions: {actions()}; inspection in row: {nested()}; visibility checking suppressed: {suppressed()}"
            }
            p {class:"r0247-note","The Linux driver uses synthetic clicks and boundary keys. Trusted browser input and a separate visual check establish real keyboard use and popup fit."}
            div {onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=super::dom_query::eval(NATIVE_TRIGGER_PLACEMENT_CHECK);let _=query.send("trigger-placement-review");
                    match query.join::<serde_json::Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dioxus trigger placement native: {value}");window.close();}
                        other=>{eprintln!("Dioxus trigger placement native failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(35)).await;eprintln!("Dioxus trigger placement native timed out");std::process::exit(1);});}
            }}
        }
    }
}

const STYLES: &str = "body{margin:0}.trigger-placement-review{box-sizing:border-box;min-height:100vh;padding:24px}.trigger-placement-review>h1{font-size:28px;margin:0 0 16px}.trigger-placement-review h2{font-size:20px;margin:0 0 12px}.trigger-placement-review p{max-width:70ch;line-height:1.5}.trigger-placement-review section{margin:28px 0;max-width:900px}.r0247-controls,.r0247-actions{display:flex;flex-wrap:wrap;align-items:center;gap:12px;margin:20px 0}.trigger-placement-review .ix-dropdown-item{gap:12px}.trigger-placement-review output{display:block;line-height:1.6;overflow-wrap:anywhere}.r0247-note{font-size:14px;color:var(--ix-color-soft-text)}@media(max-width:600px){.trigger-placement-review{padding:16px}}";

// Excluded synthetic workflow evidence, not trusted browser input/defaults.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_TRIGGER_PLACEMENT_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),root=name=>document.querySelector('.r0247-'+name),panel=name=>root(name)?.querySelector(':scope > [role="menu"]'),opened=name=>root(name)?.dataset.ixOverlayOpen==='true',state=()=>byId('r0247-state');
// Separate synthetic actions from asynchronous typed observation delivery.
const settleInput=()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));
const wait=async(label,predicate)=>{for(let n=0;n<220;n++){if(predicate()){await settleInput();return;}await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' active='+document.activeElement?.id+' state='+state()?.textContent);};
const key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,bubbles:true,cancelable:true}));
const placement=async(name,value)=>{await wait(name+' placement '+value,()=>panel(name)?.dataset.placement===value);if(panel(name).dataset.configuredPlacement!=='bottom-end')throw new Error(name+' configured placement changed');};
const presented=name=>panel(name)?.matches(':popover-open')&&getComputedStyle(panel(name)).visibility==='visible';
const parent=async()=>{if(!opened('parent'))root('parent').click();await wait('parent open',()=>opened('parent')&&presented('parent'));};
const escape=async(name)=>{if(opened(name)){panel(name).focus();key(panel(name),'Escape');await wait(name+' closed',()=>!opened(name));}};
await wait('trigger and menu mounts',()=>['standalone','parent','linked','custom','internal'].every(name=>panel(name)?.isConnected)&&byId('r0247-custom-trigger')?.isConnected);
root('standalone').click();await wait('standalone open',()=>opened('standalone')&&presented('standalone'));await placement('standalone','bottom-end');
if(byId('r0247-overview-disabled').getAttribute('aria-disabled')!=='true')throw new Error('disabled action lost');
const before=+state().dataset.actions;byId('r0247-overview-action').click();await wait('standalone action once and closes',()=>+state().dataset.actions===before+1&&!opened('standalone'));
await parent();panel('parent').focus();key(panel('parent'),'Home');await wait('linked row keyboard focus',()=>document.activeElement===byId('r0247-linked-trigger'));key(document.activeElement,'ArrowRight');await wait('linked child keyboard opens',()=>opened('linked')&&document.activeElement===byId('r0247-service-action'));await placement('linked','right-start');key(document.activeElement,'ArrowLeft');await wait('linked return',()=>!opened('linked')&&document.activeElement===byId('r0247-linked-trigger'));
byId('r0247-custom-trigger').click();await wait('custom nested opens',()=>opened('custom'));await placement('custom','right-start');await escape('custom');
await parent();root('internal').click();await wait('internal nested opens',()=>opened('internal'));await placement('internal','right-start');await escape('internal');await escape('parent');
byId('r0247-suppression').click();await wait('suppression enabled',()=>state().dataset.suppressed==='true');await parent();byId('r0247-linked-trigger').click();await wait('linked suppressed opens',()=>opened('linked'));await placement('linked','right-start');await escape('linked');await parent();byId('r0247-custom-trigger').click();await wait('custom suppressed opens',()=>opened('custom'));await placement('custom','right-start');await escape('custom');await parent();root('internal').click();await wait('internal suppressed opens',()=>opened('internal'));await placement('internal','right-start');await escape('internal');await escape('parent');
const retired=byId('r0247-custom-trigger');byId('r0247-location').click();await wait('custom replaced outside item',()=>state().dataset.nested==='false'&&byId('r0247-custom-trigger')!==retired&&!retired.isConnected);byId('r0247-custom-trigger').click();await wait('custom standalone opens',()=>opened('custom'));await placement('custom','bottom-end');await escape('custom');
const outside=byId('r0247-custom-trigger');byId('r0247-remount').click();await wait('standalone keyed remount',()=>byId('r0247-custom-trigger')!==outside&&!outside.isConnected);byId('r0247-custom-trigger').click();await wait('remounted standalone opens',()=>opened('custom'));await placement('custom','bottom-end');await escape('custom');
byId('r0247-location').click();await wait('custom restored to item',()=>state().dataset.nested==='true');await parent();byId('r0247-custom-trigger').click();await wait('restored nested opens',()=>opened('custom'));await placement('custom','right-start');const beforeCustom=+state().dataset.actions;byId('r0247-inspection-action').click();await wait('custom action once and closes',()=>+state().dataset.actions===beforeCustom+1&&!opened('custom'));await escape('parent');
return {ok:true,standaloneConfiguredPlacement:true,linkedRightStart:true,customContentRightStart:true,internalContentRightStart:true,suppressionKeepsRelationship:true,movingOutsideClearsRelationship:true,keyedReplacement:true,restoredRelationship:true,linkedKeyboardReturn:true,disabled:true,actionCallbacksOnce:true,syntheticInput:true,trustedBrowserInput:false};
"#;
