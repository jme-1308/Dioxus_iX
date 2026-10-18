//! Excluded representative Popover and responsive Header focus workflows.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn HeaderField(
    id: String,
    value: Signal<String>,
    small: Signal<bool>,
    veto: Signal<bool>,
) -> Element {
    let mounted = use_focus_mount(id.clone());
    let mut value = value;
    let mut small = small;
    let mut veto = veto;
    rsx! { input { id, class:"generic-focus-field", "aria-label":"Equipment header note", value:value(),
        onmounted:mounted, oninput:move |event|value.set(event.value()),
        onkeydown:move |event:KeyboardEvent|match event.key() {
            Key::F2=>{event.prevent_default();small.set(true);},
            Key::F3=>{event.prevent_default();small.set(false);},
            Key::F4=>{event.prevent_default();veto.toggle();},
            _=>{},
        }
    } }
}

#[component]
pub fn GenericFocusPreview() -> Element {
    let popover = use_popover();
    let hover = use_popover();
    let field_mount = use_focus_mount("r0211-popover-field");
    let mut note = use_signal(|| "Pump P-101".to_string());
    let header_note = use_signal(|| "Station A".to_string());
    let mut veto = use_signal(|| false);
    let mut header_veto = use_signal(|| false);
    let mut small = use_signal(|| false);
    let mut top_layer = use_signal(|| true);
    let mut actions = use_signal(|| 0_usize);
    let mut close_requests = use_signal(|| 0_usize);
    let mut changes = use_signal(Vec::<bool>::new);
    let mut header_close_requests = use_signal(|| 0_usize);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class:"generic-focus-review",
            h1 { "Dioxus-owned overlay focus" }
            p { "Popover content and Header actions use registered mounted controls for opening, returning and keyboard boundaries." }
            section {
                h2 { "Equipment panel" }
                div { class:"generic-focus-controls",
                    div { id:"r0211-popover-trigger", PopoverTrigger { popover, "Open equipment panel" } }
                    Button { id:"r0211-outside", variant:ButtonVariant::Secondary, "Independent destination" }
                }
                Popover { popover, label:"Equipment focus panel", close_on_click_outside:true,
                    onshowchanged:move |open|changes.write().push(open),
                    onbeforechange:move |event:PopoverChangeEvent|if !event.open {
                        close_requests.set(close_requests()+1);
                        if veto(){event.prevent_default();}
                    },
                    PopoverHeader { close_label:"Close equipment panel",
                        additional_items:rsx! { Button { id:"r0211-header-action", variant:ButtonVariant::SubtleSecondary,
                            onclick:move |_|actions.set(actions()+1), "Panel information" } }, "Equipment details"
                    }
                    PopoverContent {
                        label { r#for:"r0211-popover-field", "Equipment name" }
                        input { id:"r0211-popover-field", class:"generic-focus-field", value:note(), onmounted:field_mount,
                            oninput:move |event|note.set(event.value()) }
                        div { class:"generic-focus-controls",
                            Button { id:"r0211-content-action", onclick:move |_|actions.set(actions()+1), "Apply equipment name" }
                            Button { id:"r0211-veto", variant:ButtonVariant::Secondary, onclick:move |_|veto.toggle(),
                                if veto(){"Allow panel close"}else{"Veto panel close"} }
                        }
                    }
                    PopoverFooter {
                        Button { id:"r0211-popover-close", variant:ButtonVariant::Secondary,
                            onclick:move |_|popover.hide(), "Close and return" }
                    }
                }
                output { id:"r0211-state", "data-veto":veto().to_string(), "data-close-requests":close_requests(), "data-changes":format!("{:?}",changes()), "Name: {note()}; actions: {actions()}" }
            }
            section {
                h2 { "Hover panel" }
                p { "Hover opens without moving focus. Explicit close returns to its trigger without reopening it." }
                div { id:"r0211-hover-trigger", PopoverTrigger { popover:hover, trigger_mode:PopoverTriggerMode::Hover,
                    variant:ButtonVariant::Secondary, "Hover equipment help" } }
                Popover { popover:hover, label:"Equipment hover help",
                    PopoverContent { p { "Inspect the current equipment before applying changes." }
                        Button { id:"r0211-hover-close", variant:ButtonVariant::Secondary, onclick:move |_|hover.hide(), "Close equipment help" }
                    }
                }
            }
            section {
                h2 { "Responsive Header controls" }
                p { "In either note field: F2 uses the small layout, F3 the large layout, and F4 toggles the close veto." }
                div { class:"generic-focus-controls",
                    Button { id:"r0211-layout", variant:ButtonVariant::Secondary, onclick:move |_|small.toggle(), "Switch Header layout" }
                    Button { id:"r0211-top-layer", variant:ButtonVariant::Secondary, onclick:move |_|top_layer.toggle(), "Switch top layer" }
                    Button { id:"r0211-header-veto", variant:ButtonVariant::Secondary, onclick:move |_|header_veto.toggle(), "Toggle Header close veto" }
                }
                Application { class:"r0211-app", force_breakpoint:if small(){ApplicationBreakpoint::Small}else{ApplicationBreakpoint::Large},
                    header:rsx! { ApplicationHeader { name:"Equipment", show_menu:false, enable_top_layer:top_layer(),
                        aria_label_more_menu_icon_button:"Equipment Header actions",
                        onbeforeoverflowchange:move |event:HeaderOverflowEvent|if !event.open {
                            header_close_requests.set(header_close_requests()+1);
                            if header_veto(){event.prevent_default();}
                        },
                        actions:rsx! {
                            HeaderField { id:"r0211-header-note", value:header_note, small, veto:header_veto }
                            Button { id:"r0211-header-action-button", variant:ButtonVariant::SubtleSecondary,
                                onclick:move |_|actions.set(actions()+1), "Inspect station" }
                        },
                        overflow:rsx! {
                            HeaderField { id:"r0211-overflow-note", value:header_note, small, veto:header_veto }
                            Button { id:"r0211-overflow-last", variant:ButtonVariant::SubtleSecondary,
                                onclick:move |_|actions.set(actions()+1), "Export station" }
                        }
                    } },
                    p { "The note and its mounted element stay in the same Dioxus content tree across layouts." }
                }
                output { id:"r0211-header-state", "data-small":small().to_string(), "data-veto":header_veto().to_string(),
                    "data-close-requests":header_close_requests(),
                    "data-top-layer":top_layer().to_string(), "Header note: {header_note()}" }
            }
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=super::dom_query::eval(NATIVE_FOCUS_CHECK);let _=query.send("generic-focus-review");
                    match query.join::<serde_json::Value>().await {
                        Ok(value)if value["ok"]==true=>{println!("Dioxus generic focus native: {value}");window.close();}
                        other=>{eprintln!("Dioxus generic focus native failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus generic focus native timed out");std::process::exit(1);});}
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.generic-focus-review{min-height:100vh;box-sizing:border-box;padding:24px}.generic-focus-review>h1{font-size:28px;margin:0 0 16px}.generic-focus-review p{max-width:65ch;line-height:1.5}.generic-focus-review section{max-width:900px;margin:28px 0;padding-bottom:24px;border-bottom:1px solid var(--ix-color-border)}.generic-focus-review h2{font-size:20px}.generic-focus-controls{display:flex;flex-wrap:wrap;align-items:center;gap:12px;margin:16px 0}.generic-focus-field{box-sizing:border-box;min-height:32px;max-width:100%;width:190px;padding:4px 8px;background:var(--ix-color-component-1);color:inherit;border:1px solid var(--ix-color-border);font:inherit}.generic-focus-field:focus-visible{outline:2px solid var(--ix-color-primary);outline-offset:2px}.generic-focus-review output{display:block;margin-top:12px}.generic-focus-review .r0211-app{height:220px;min-height:220px}.generic-focus-review .r0211-app .ix-application-main{padding:16px}@media(max-width:600px){.generic-focus-review{padding:16px}}";

// Excluded synthetic Linux workflow driver. Native interior Tab is asserted
// unprevented; physical browser traversal is verified independently by the lead.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_FOCUS_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),active=()=>document.activeElement;
const wait=async(label,predicate)=>{for(let n=0;n<200;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' focus='+active()?.id);};
const key=(node,value,shiftKey=false)=>{const event=new KeyboardEvent('keydown',{key:value,code:value,shiftKey,bubbles:true,cancelable:true});node.dispatchEvent(event);return event;};
const trigger=()=>byId('r0211-popover-trigger').querySelector('button');
const panel=()=>byId(trigger().getAttribute('aria-controls'));
const isOpen=node=>node?.matches(':popover-open')&&node.getAttribute('data-ix-overlay-open')==='true';
const closePanel=()=>!panel().matches(':popover-open')&&panel().getAttribute('data-ix-overlay-open')==='false';
await wait('Popover trigger mounted',()=>trigger());
trigger().scrollIntoView({block:'center'});trigger().click();
await wait('content preferred over header',()=>isOpen(panel())&&active()===byId('r0211-popover-field'));
await wait('opening completion',()=>byId('r0211-state').dataset.changes==='[true]');
const field=byId('r0211-popover-field');field.value='Pump revised';field.dispatchEvent(new Event('input',{bubbles:true}));
await wait('Popover typing',()=>byId('r0211-state').textContent.includes('Pump revised'));
if(key(field,'Tab').defaultPrevented)throw new Error('Interior Popover Tab was intercepted');
byId('r0211-popover-close').focus();if(!key(active(),'Tab').defaultPrevented)throw new Error('Popover forward boundary not trapped');
await wait('Popover forward wrap',()=>active()===byId('r0211-header-action'));
if(!key(active(),'Tab',true).defaultPrevented)throw new Error('Popover reverse boundary not trapped');
await wait('Popover reverse wrap',()=>active()===byId('r0211-popover-close'));
key(active(),'Escape');await wait('Popover Escape return',()=>closePanel()&&active()===trigger());
await wait('close completion',()=>byId('r0211-state').dataset.changes==='[true, false]');
trigger().click();await wait('Popover explicit reopen',()=>isOpen(panel())&&active()===field);
byId('r0211-popover-close').click();await wait('Popover explicit return',()=>closePanel()&&active()===trigger());
trigger().click();await wait('Popover veto setup',()=>isOpen(panel())&&active()===field);
byId('r0211-veto').click();await wait('Popover veto enabled',()=>byId('r0211-state').dataset.veto==='true');
const panelRequests=+byId('r0211-state').dataset.closeRequests;
byId('r0211-popover-close').click();await wait('Popover veto retains visible panel',()=>+byId('r0211-state').dataset.closeRequests>panelRequests&&isOpen(panel()));
byId('r0211-veto').click();await wait('Popover veto disabled',()=>byId('r0211-state').dataset.veto==='false');
const outside=byId('r0211-outside');outside.focus();outside.click();
await wait('outside close preserves destination',()=>closePanel()&&active()===outside);
const hoverTrigger=byId('r0211-hover-trigger').querySelector('button'),hoverPanel=()=>byId(hoverTrigger.getAttribute('aria-controls'));
hoverTrigger.dispatchEvent(new MouseEvent('mouseenter',{bubbles:false}));
await wait('hover non-stealing opening',()=>isOpen(hoverPanel())&&active()===outside);
byId('r0211-hover-close').click();
await wait('hover explicit return without focus reopening',()=>!hoverPanel().matches(':popover-open')&&active()===hoverTrigger);
const header=()=>document.querySelector('.r0211-app .ix-header-layout'),more=()=>header().querySelector('.ix-header-more-button');
await wait('Header native ready',()=>header()?.dataset.ixOverlayReady==='true'&&more()&&!more().hidden);
more().focus();key(more(),'Home');await wait('Header visible handoff setup',()=>header().dataset.open==='true'&&active()===byId('r0211-overflow-note'));
const headerField=byId('r0211-overflow-note');headerField.scrollIntoView({block:'center'});headerField.focus();
headerField.value='Station retained';headerField.dispatchEvent(new Event('input',{bubbles:true}));
await wait('Header edit',()=>byId('r0211-header-state').textContent.includes('Station retained'));
key(headerField,'F2');await wait('small Header preserves field',()=>header().dataset.small==='true'&&header().dataset.open==='true'&&active()===headerField&&headerField.value==='Station retained');
key(headerField,'F3');await wait('large Header preserves field',()=>header().dataset.small==='false'&&active()===headerField&&headerField.value==='Station retained');
if(header().dataset.open==='true'){key(more(),'Escape');await wait('Header close before opening cases',()=>header().dataset.open==='false');}
more().focus();key(more(),'Home');await wait('Header Home entry',()=>header().dataset.open==='true'&&active()===byId('r0211-overflow-note'));
if(key(active(),'Tab').defaultPrevented)throw new Error('Interior Header Tab was intercepted');
byId('r0211-overflow-last').focus();if(!key(active(),'Tab').defaultPrevented)throw new Error('Header forward boundary not trapped');
await wait('Header forward wrap',()=>active()===byId('r0211-overflow-note'));
if(!key(active(),'Tab',true).defaultPrevented)throw new Error('Header reverse boundary not trapped');
await wait('Header reverse wrap',()=>active()===byId('r0211-overflow-last'));
key(active(),'Escape');await wait('Header Escape return',()=>header().dataset.open==='false'&&active()===more());
byId('r0211-header-veto').click();await wait('Header veto enabled',()=>byId('r0211-header-state').dataset.veto==='true');
more().focus();key(more(),'End');await wait('Header End entry',()=>header().dataset.open==='true'&&active()===byId('r0211-overflow-last'));
const headerRequests=+byId('r0211-header-state').dataset.closeRequests;
key(active(),'Escape');await wait('Header close veto',()=>+byId('r0211-header-state').dataset.closeRequests>headerRequests&&header().dataset.open==='true');
byId('r0211-overflow-note').focus();key(active(),'F4');await wait('Header veto released',()=>byId('r0211-header-state').dataset.veto==='false');
key(active(),'Escape');await wait('Header released close',()=>header().dataset.open==='false'&&active()===more());
byId('r0211-top-layer').click();await wait('inline Header selected',()=>header().dataset.topLayer==='false');
more().focus();key(more(),'Home');await wait('inline Header mounted focus',()=>header().dataset.open==='true'&&active()===byId('r0211-overflow-note'));
key(active(),'Escape');await wait('inline Header return',()=>header().dataset.open==='false'&&active()===more());
return {ok:true,popover:{contentFirst:true,typing:true,boundaries:2,interiorDefault:true,escape:true,explicit:true,veto:true,outsidePreserved:true},hover:{nonStealing:true,returned:true},header:{home:true,end:true,boundaries:2,interiorDefault:true,escape:true,veto:true,responsiveValue:headerField.value,inline:true}};
"#;
