//! Excluded workflows for Dioxus-owned Dropdown anchor visibility.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn CustomAnchor(dropdown: DropdownHandle) -> Element {
    let binding = use_dropdown_trigger_binding(dropdown);
    let mut visible = use_signal(|| false);
    rsx! { button {
        id:"r0219-custom-trigger",r#type:"button",class:"ix-button ix-button-secondary",
        onmounted:binding.onmounted,onkeydown:binding.onkeydown,onclick:binding.onclick,"data-visible":visible().to_string(),onvisible:move |event:VisibleEvent|{if let Ok(value)=event.is_intersecting(){visible.set(value);}binding.onvisible.call(event);},
        "aria-haspopup":"menu","aria-expanded":dropdown.is_open().to_string(),
        ..binding.attributes(), "Custom Dioxus anchor"
    } }
}

#[component]
pub fn VisibilityPreview() -> Element {
    let mut veto = use_signal(|| false);
    let mut suppress = use_signal(|| false);
    let mut generation = use_signal(|| 1usize);
    let custom = use_dropdown();
    let mut built_changes = use_signal(Vec::<bool>::new);
    let mut custom_changes = use_signal(Vec::<bool>::new);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! { style {{STYLES}}
        IxTheme {class:"visibility-review",
            h1 {"Dioxus-owned anchor visibility"}
            p {"Open an action menu, then scroll its anchor out of the panel. The menu closes while its controls keep their normal Dioxus ownership."}
            div {class:"visibility-controls",
                Button {id:"r0219-veto",variant:ButtonVariant::Secondary,onclick:move |_|veto.toggle(),
                    if veto(){"Allow dismissal"}else{"Veto dismissal"}}
                Button {id:"r0219-suppress",variant:ButtonVariant::Secondary,onclick:move |_|suppress.toggle(),
                    if suppress(){"Enable visibility dismissal"}else{"Suppress visibility dismissal"}}
                Button {id:"r0219-remount",variant:ButtonVariant::Secondary,onclick:move |_|generation.set(generation()+1),"Remount anchor"}
            }
            section {h2 {"Built-in action anchor"}
                p {"The menu uses a native top layer and automatic viewport placement."}
                div {id:"r0219-built-scroll",class:"visibility-scroll",tabindex:0,
                    div {class:"visibility-tall",
                        for current in [generation()] {
                            div {key:"{current}",class:"visibility-anchor-row",
                                Dropdown {label:"Built-in actions",class:"r0219-built",enable_top_layer:true,
                                    navigation_mode:DropdownNavigationMode::RovingTabindex,
                                    suppress_trigger_visibility_check:suppress(),
                                    onbeforechange:move |event:DropdownChangeEvent|{if !event.open&&veto(){event.prevent_default();}},
                                    onchange:move |open|built_changes.write().push(open),
                                    DropdownItem {id:"r0219-built-first",label:"Inspect equipment"}
                                    DropdownItem {id:"r0219-built-last",label:"Review service history"}
                                }
                            }
                        }
                        p {class:"visibility-scroll-note","Scroll here to move the anchor out of view."}
                    }
                }
            }
            section {h2 {"Custom Dioxus anchor"}
                p {"A custom control attaches the explicit mounted and visibility handlers."}
                div {id:"r0219-custom-scroll",class:"visibility-scroll",tabindex:0,
                    div {class:"visibility-tall",
                        div {class:"visibility-anchor-row",CustomAnchor {dropdown:custom}}
                        p {class:"visibility-scroll-note","The same dismissal policy applies to this registered custom control."}
                    }
                }
                Dropdown {label:"Custom action menu",controller:custom,trigger:Some("r0219-custom-trigger".into()),
                    class:"r0219-custom",enable_top_layer:true,navigation_mode:DropdownNavigationMode::RovingTabindex,
                    onchange:move |open|custom_changes.write().push(open),
                    DropdownItem {id:"r0219-custom-first",label:"Inspect custom equipment"}
                    DropdownItem {id:"r0219-custom-last",label:"Review custom history"}
                }
            }
            Button {id:"r0219-outside",variant:ButtonVariant::Secondary,"Independent destination"}
            output {id:"r0219-state","data-veto":veto().to_string(),"data-suppress":suppress().to_string(),
                "data-generation":generation(),"Built-in changes: {built_changes():?}; custom changes: {custom_changes():?}"}
            div {onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=super::dom_query::eval(NATIVE_VISIBILITY_CHECK);let _=query.send("visibility-review");
                    match query.join::<serde_json::Value>().await {
                        Ok(value)if value["ok"]==true=>{println!("Dioxus anchor visibility native: {value}");window.close();}
                        other=>{eprintln!("Dioxus anchor visibility native failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus anchor visibility native timed out");std::process::exit(1);});}
            }}
        }
    }
}

const STYLES: &str = "body{margin:0}.visibility-review{min-height:100vh;padding:24px;box-sizing:border-box}.visibility-review>h1{font-size:28px;margin:0 0 16px}.visibility-review h2{font-size:20px}.visibility-review p{max-width:65ch;line-height:1.5}.visibility-review section{max-width:800px;margin:28px 0}.visibility-controls{display:flex;flex-wrap:wrap;gap:12px}.visibility-scroll{height:180px;overflow:auto;border:1px solid var(--ix-color-border);background:var(--ix-color-component-1);max-width:100%}.visibility-tall{height:600px;padding:24px;box-sizing:border-box}.visibility-anchor-row{display:flex;justify-content:flex-end;max-width:100%}.visibility-anchor-row .ix-dropdown{max-width:100%}.visibility-scroll-note{margin-top:300px}.visibility-review output{display:block;margin-top:16px;max-width:65ch;overflow-wrap:anywhere}@media(max-width:600px){.visibility-review{padding:16px}.visibility-tall{padding:16px}.visibility-review h1{font-size:28px}.visibility-anchor-row{justify-content:flex-start}.visibility-controls{align-items:stretch;flex-direction:column}}";

// Synthetic Linux workflow drives normal clicks and scrolling. Dioxus's own
// onvisible observation supplies all production visibility facts; the driver
// never dispatches a made-up visible event or writes overlay state/cues.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_VISIBILITY_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),built=()=>document.querySelector('.r0219-built'),custom=()=>document.querySelector('.r0219-custom');
// Keep synthetic clicks on separate browser turns so typed observations and
// Dioxus prop updates settle before the next user action. No duration is tested.
const settleInput=()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));
const wait=async(label,predicate)=>{for(let n=0;n<220;n++){if(predicate()){await settleInput();return;}await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' '+byId('r0219-state')?.textContent);};
const opened=root=>root?.getAttribute('data-ix-overlay-open')==='true',shown=root=>!!root?.querySelector('dialog:popover-open');
await wait('library and custom anchors mounted',()=>built()?.getBoundingClientRect().width>0&&custom()?.isConnected&&byId('r0219-custom-trigger')?.getBoundingClientRect().width>0);
const first=byId('r0219-built-scroll'),second=byId('r0219-custom-scroll');
first.scrollIntoView({block:'center'});built().click();await wait('built-in open',()=>opened(built())&&shown(built()));
first.scrollTop=420;await wait('Dioxus visibility dismisses built-in',()=>!opened(built()));
first.scrollTop=0;await wait('built-in anchor visible',()=>built().getBoundingClientRect().top>=first.getBoundingClientRect().top);
byId('r0219-veto').click();await wait('veto enabled',()=>byId('r0219-state').dataset.veto==='true');built().click();await wait('veto setup open',()=>opened(built()));first.scrollTop=420;
await new Promise(resolve=>setTimeout(resolve,400));if(!opened(built()))throw new Error('visibility ignored dismissal veto');
first.scrollTop=0;byId('r0219-veto').click();byId('r0219-outside').click();await wait('veto released close',()=>!opened(built()));
byId('r0219-suppress').click();await wait('suppression enabled',()=>byId('r0219-state').dataset.suppress==='true');built().click();await wait('suppression setup open',()=>opened(built()));first.scrollTop=420;
await new Promise(resolve=>setTimeout(resolve,400));if(!opened(built()))throw new Error('visibility ignored suppression');
byId('r0219-outside').click();await wait('suppressed ordinary outside closes',()=>!opened(built()));first.scrollTop=0;byId('r0219-suppress').click();
second.scrollIntoView({block:'center'});await wait('custom framework visibility ready',()=>byId('r0219-custom-trigger').dataset.visible==='true');byId('r0219-custom-trigger').click();await wait('custom paired anchor open',()=>opened(custom())&&shown(custom()));
second.scrollTop=420;await wait('Dioxus visibility dismisses custom',()=>!opened(custom()));
second.scrollTop=0;await wait('custom visible again',()=>byId('r0219-custom-trigger').dataset.visible==='true');byId('r0219-custom-trigger').click();await wait('custom reopen',()=>opened(custom()));byId('r0219-outside').click();await wait('custom outside close',()=>!opened(custom()));
const old=built();byId('r0219-remount').click();await wait('new anchor mounted',()=>built()&&built()!==old);first.scrollTop=0;first.scrollIntoView({block:'center'});built().click();await wait('new anchor works',()=>opened(built()));first.scrollTop=420;await wait('new anchor visibility close',()=>!opened(built()));
const popup=custom().querySelector('dialog');
return {ok:true,builtInScrollDismiss:true,customScrollDismiss:true,veto:true,suppression:true,outsideDismiss:true,reopen:true,remount:true,frameworkVisibleEvents:true,customPlacement:popup?.dataset.placement,cueWrites:0};
"#;
