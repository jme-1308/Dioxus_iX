//! Excluded representative Popover and responsive Header focus workflows.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::rc::Rc;

#[component]
fn HeaderField(
    id: String,
    value: Signal<String>,
    small: Signal<bool>,
    veto: Signal<bool>,
    details: Signal<bool>,
    onpreviewscroll: EventHandler<()>,
    onpreviewresize: EventHandler<()>,
) -> Element {
    let mounted = use_focus_mount(id.clone());
    let mut value = value;
    let mut small = small;
    let mut veto = veto;
    let mut details = details;
    rsx! { input { id, class:"generic-focus-field", "aria-label":"Equipment header note", value:value(),
        onmounted:mounted, oninput:move |event|value.set(event.value()),
        onkeydown:move |event:KeyboardEvent|match event.key() {
            Key::F2=>{event.prevent_default();small.set(true);},
            Key::F3=>{event.prevent_default();small.set(false);},
            Key::F4=>{event.prevent_default();veto.toggle();},
            Key::F5=>{event.prevent_default();onpreviewscroll.call(());},
            Key::F6=>{event.prevent_default();details.toggle();},
            Key::F8=>{event.prevent_default();onpreviewresize.call(());},
            _=>{},
        }
    } }
}

#[component]
pub fn GenericFocusPreview() -> Element {
    let popover = use_popover();
    let hover = use_popover();
    let custom = use_popover();
    let centered = use_popover();
    let custom_trigger = use_popover_trigger_binding(custom, "r0228-custom-popover-trigger");
    let action_mount = use_focus_mount("r0211-content-action");
    let mut note = use_signal(|| "Pump P-101".to_string());
    let mut priority = use_signal(|| false);
    let header_note = use_signal(|| "Station A".to_string());
    let mut veto = use_signal(|| false);
    let mut header_veto = use_signal(|| false);
    let mut small = use_signal(|| false);
    let mut top_layer = use_signal(|| true);
    let mut actions = use_signal(|| 0_usize);
    let mut close_requests = use_signal(|| 0_usize);
    let mut changes = use_signal(Vec::<bool>::new);
    let mut header_close_requests = use_signal(|| 0_usize);
    let mut details = use_signal(|| false);
    let mut compact = use_signal(|| false);
    let mut scroll_offset = use_signal(|| false);
    let mut scroll_mount = use_signal(|| None::<Rc<MountedData>>);
    let scroll_header = EventHandler::new(move |_| {
        scroll_offset.toggle();
        if let Some(node) = scroll_mount() {
            let offset = if scroll_offset() { 72.0 } else { 0.0 };
            spawn(async move {
                let _ = node
                    .scroll(
                        dioxus::html::geometry::PixelsVector2D::new(0.0, offset),
                        ScrollBehavior::Instant,
                    )
                    .await;
            });
        }
    });
    let resize_header = EventHandler::new(move |_| {
        compact.toggle();
        #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
        dioxus::desktop::window().set_inner_size(dioxus::desktop::LogicalSize::new(
            if compact() { 720.0 } else { 1000.0 },
            if compact() { 760.0 } else { 850.0 },
        ));
    });
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
                    button { r#type:"button", class:"ix-button ix-button-secondary",
                        onmounted:custom_trigger.onmounted, onvisible:custom_trigger.onvisible,
                        onclick:move |_|{spawn(async move {let _=custom.show().await;});}, ..custom_trigger.attributes(), "Open custom equipment help" }
                    Button { id:"r0228-center-open", variant:ButtonVariant::Secondary, onclick:move |_|{spawn(async move {let _=centered.show().await;});}, "Open centered notice" }
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
                        Input { id:"r0211-popover-field", aria_label:"Equipment name", value:note(),
                            onvaluechange:move |value|note.set(value),
                            onkeydown:move |event:KeyboardEvent| {
                                // Consumer cancellation must reach the shared
                                // document Escape handler on both renderers.
                                if event.key()==Key::Escape && event.modifiers().contains(Modifiers::SHIFT) {
                                    event.prevent_default();
                                }
                                if event.key()==Key::F7 {
                                    event.prevent_default();
                                    event.stop_propagation();
                                    priority.toggle();
                                }
                            }
                        }
                        div { class:"generic-focus-controls",
                            button { id:"r0211-content-action", r#type:"button",
                                class:"ix-button ix-button-standard ix-button-tokenized ix-button-primary ix-align-center",
                                tabindex:if priority(){1}else{0}, onmounted:action_mount,
                                onclick:move |_|actions.set(actions()+1),
                                span { class:"ix-button-content", "Apply equipment name" }
                            }
                            Button { id:"r0211-veto", variant:ButtonVariant::Secondary, onclick:move |_|veto.toggle(),
                                if veto(){"Allow panel close"}else{"Veto panel close"} }
                        }
                    }
                    PopoverFooter {
                        Button { id:"r0211-popover-close", variant:ButtonVariant::Secondary,
                            onclick:move |_|popover.hide(), "Close and return" }
                    }
                }
                Popover { popover:custom, label:"Custom equipment help",
                    PopoverContent { p { "This native button supplies an explicit Dioxus anchor binding." }
                        Button { id:"r0228-custom-popover-close", variant:ButtonVariant::Secondary, onclick:move |_|custom.hide(), "Close custom equipment help" } }
                }
                Popover { popover:centered, label:"Centered equipment notice",
                    PopoverContent { p { "An imperative panel with no registered trigger centers in the viewport." }
                        Button { id:"r0228-center-close", variant:ButtonVariant::Secondary, onclick:move |_|centered.hide(), "Close centered notice" } }
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
                p { "In either note field: F2 uses the small layout, F3 the large layout, F4 toggles the close veto, F5 scrolls this region, F6 changes action details, and F8 resizes the preview area." }
                div { class:"generic-focus-controls",
                    Button { id:"r0211-layout", variant:ButtonVariant::Secondary, onclick:move |_|small.toggle(), "Switch Header layout" }
                    Button { id:"r0211-top-layer", variant:ButtonVariant::Secondary, onclick:move |_|top_layer.toggle(), "Switch top layer" }
                    Button { id:"r0211-header-veto", variant:ButtonVariant::Secondary, onclick:move |_|header_veto.toggle(), "Toggle Header close veto" }
                    Button { id:"r0228-scroll-header", variant:ButtonVariant::Secondary, onclick:move |_|scroll_header.call(()), "Scroll Header region" }
                    Button { id:"r0228-resize-header", variant:ButtonVariant::Secondary, onclick:move |_|resize_header.call(()), "Resize Header area" }
                    Button { id:"r0228-resize-content", variant:ButtonVariant::Secondary, onclick:move |_|details.toggle(), "Change action details" }
                }
                div { id:"r0228-header-scroll", class:"r0228-header-scroll", "data-compact":compact().to_string(),
                    onmounted:move |event|scroll_mount.set(Some(event.data())),
                    div { class:"r0228-scroll-spacer", "aria-hidden":"true" }
                Application { class:"r0211-app", force_breakpoint:if small(){ApplicationBreakpoint::Small}else{ApplicationBreakpoint::Large},
                    header:rsx! { ApplicationHeader { name:"Equipment", show_menu:false, enable_top_layer:top_layer(),
                        aria_label_more_menu_icon_button:"Equipment Header actions",
                        onbeforeoverflowchange:move |event:HeaderOverflowEvent|if !event.open {
                            header_close_requests.set(header_close_requests()+1);
                            if header_veto(){event.prevent_default();}
                        },
                        actions:rsx! {
                            HeaderField { id:"r0211-header-note", value:header_note, small, veto:header_veto, details, onpreviewscroll:scroll_header, onpreviewresize:resize_header }
                            Button { id:"r0211-header-action-button", variant:ButtonVariant::SubtleSecondary,
                                onclick:move |_|actions.set(actions()+1), "Inspect station" }
                        },
                        overflow:rsx! {
                            HeaderField { id:"r0211-overflow-note", value:header_note, small, veto:header_veto, details, onpreviewscroll:scroll_header, onpreviewresize:resize_header }
                            div { id:"r0228-action-details", class:"r0228-action-details", "data-expanded":details().to_string(),
                                if details() { p { "Station export includes equipment labels, operating notes and the current inspection summary." }
                                    p { "Review these details before exporting the station." } }
                            }
                            Button { id:"r0211-overflow-last", variant:ButtonVariant::SubtleSecondary,
                                onclick:move |_|actions.set(actions()+1), "Export station" }
                        }
                    } },
                    p { "The note and its mounted element stay in the same Dioxus content tree across layouts." }
                }
                    div { class:"r0228-scroll-spacer", "aria-hidden":"true" }
                }
                output { id:"r0211-header-state", "data-small":small().to_string(), "data-veto":header_veto().to_string(),
                    "data-close-requests":header_close_requests(),
                    "data-top-layer":top_layer().to_string(), "data-compact":compact().to_string(), "data-details":details().to_string(), "Header note: {header_note()}" }
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

const STYLES: &str = "body{margin:0}.generic-focus-review{min-height:100vh;box-sizing:border-box;padding:24px}.generic-focus-review>h1{font-size:28px;margin:0 0 16px}.generic-focus-review p{max-width:65ch;line-height:1.5}.generic-focus-review section{max-width:900px;margin:28px 0;padding-bottom:24px;border-bottom:1px solid var(--ix-color-border)}.generic-focus-review h2{font-size:20px}.generic-focus-controls{display:flex;flex-wrap:wrap;align-items:center;gap:12px;margin:16px 0}.generic-focus-field{box-sizing:border-box;min-height:32px;max-width:100%;width:190px;padding:4px 8px;background:var(--ix-color-component-1);color:inherit;border:1px solid var(--ix-color-border);font:inherit}.generic-focus-field:focus-visible{outline:2px solid var(--ix-color-primary);outline-offset:2px}.generic-focus-review output{display:block;margin-top:12px}.generic-focus-review .r0211-app{height:220px;min-height:220px}.generic-focus-review .r0211-app .ix-application-main{padding:16px}.r0228-header-scroll{height:340px;overflow:auto;max-width:100%;box-sizing:border-box;border:1px solid var(--ix-color-border);scrollbar-gutter:stable}.r0228-header-scroll[data-compact=true]{width:min(100%,520px)}.r0228-scroll-spacer{height:96px}.r0228-action-details{max-width:32rem}.r0228-action-details:empty{display:none}.r0228-action-details p{margin:8px 0}@media(max-width:600px){.generic-focus-review{padding:16px}}";

// Excluded synthetic Linux workflow driver. Native interior Tab is asserted
// unprevented; physical browser traversal is verified independently by the lead.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_FOCUS_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),active=()=>document.activeElement;
// Synthetic actions wait for the visible state and its asynchronous native
// observation before dispatching the next key; elapsed time is not asserted.
const settleInput=()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));
const wait=async(label,predicate)=>{for(let n=0;n<200;n++){if(predicate()){await settleInput();return;}await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' focus='+active()?.id+' geometry='+JSON.stringify({viewport:{width:document.documentElement.clientWidth,height:document.documentElement.clientHeight,visual:window.visualViewport?{width:visualViewport.width,height:visualViewport.height}:null},header:document.querySelector('.ix-header-layout')?.dataset,more:document.querySelector('.ix-header-more-button')?.getBoundingClientRect().toJSON(),popup:document.querySelector('[data-header-popup-host][data-active="true"]')?.getBoundingClientRect().toJSON(),style:document.querySelector('[data-header-popup-host][data-active="true"]')?.getAttribute('style')}));};
// Real focus and key input arrive on separate browser turns. Yield after the
// driver's imperative focus so typed native facts can reach Dioxus before Tab.
const focusControl=async node=>{node.focus();await settleInput();};
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
const canceledRequests=+byId('r0211-state').dataset.closeRequests;
if(!key(field,'Escape',true).defaultPrevented)throw new Error('Consumer Escape cancellation lost');
await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));
if(!isOpen(panel())||+byId('r0211-state').dataset.closeRequests!==canceledRequests)throw new Error('Canceled Escape reached global dismissal');
await focusControl(byId('r0211-popover-close'));if(!key(active(),'Tab').defaultPrevented)throw new Error('Popover forward boundary not trapped');
await wait('Popover forward wrap',()=>active()===byId('r0211-header-action'));
if(!key(active(),'Tab',true).defaultPrevented)throw new Error('Popover reverse boundary not trapped');
await wait('Popover reverse wrap',()=>active()===byId('r0211-popover-close'));
field.focus();key(field,'F7');await wait('positive Tab priority enabled',()=>byId('r0211-content-action').tabIndex===1);
await focusControl(byId('r0211-popover-close'));if(!key(active(),'Tab').defaultPrevented)throw new Error('Positive Tab forward boundary not trapped');
await wait('positive Tab forward wrap',()=>active()===byId('r0211-content-action'));
if(!key(active(),'Tab',true).defaultPrevented)throw new Error('Positive Tab reverse boundary not trapped');
await wait('positive Tab reverse wrap',()=>active()===byId('r0211-popover-close'));
field.focus();key(field,'F7');await wait('positive Tab priority reset',()=>byId('r0211-content-action').tabIndex===0);
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
// Focus-triggered Hover mode uses the real panel containment observation.
// Synthetic focus is explicit; elapsed hide delay is not an acceptance metric.
await focusControl(outside);
await focusControl(hoverTrigger);
await wait('focus trigger opens hover panel without stealing trigger focus',()=>isOpen(hoverPanel())&&active()===hoverTrigger);
await focusControl(byId('r0211-hover-close'));
await wait('focus within hover content retains visible panel',()=>isOpen(hoverPanel())&&active()===byId('r0211-hover-close'));
// Let the ordinary delayed hide settle; no elapsed-time threshold is asserted.
await new Promise(resolve=>setTimeout(resolve,250));
if(!isOpen(hoverPanel())||active()!==byId('r0211-hover-close'))throw new Error('settled internal hover focus was dismissed');
await focusControl(outside);
await wait('outside focus dismisses hover panel and retains outside destination',()=>!hoverPanel().matches(':popover-open')&&active()===outside);
await focusControl(hoverTrigger);
await wait('focus hover panel reopens with current native ownership',()=>isOpen(hoverPanel())&&active()===hoverTrigger);
byId('r0211-hover-close').click();
await wait('focus hover explicit close returns without reopening',()=>!hoverPanel().matches(':popover-open')&&active()===hoverTrigger);

const customTrigger=byId('r0228-custom-popover-trigger');
const customPanel=()=>Array.from(document.querySelectorAll('[data-ix-overlay-native-surface="popover"]')).find(node=>node.getAttribute('aria-label')==='Custom equipment help');
customTrigger.scrollIntoView({block:'center'});customTrigger.click();
await wait('explicit custom anchor',()=>isOpen(customPanel())&&customPanel().getAttribute('data-unanchored')!=='true'&&getComputedStyle(customPanel()).getPropertyValue('position-anchor').trim()===customTrigger.style.getPropertyValue('anchor-name').trim());
byId('r0228-custom-popover-close').click();await wait('custom trigger return',()=>!customPanel().matches(':popover-open')&&active()===customTrigger);
const centeredPanel=()=>Array.from(document.querySelectorAll('[data-ix-overlay-native-surface="popover"]')).find(node=>node.getAttribute('aria-label')==='Centered equipment notice');
byId('r0228-center-open').click();
await wait('imperative unanchored center',()=>{
  const node=centeredPanel();if(!isOpen(node)||node.getAttribute('data-unanchored')!=='true')return false;
  const rect=node.getBoundingClientRect();return Math.abs(rect.left+rect.width/2-document.documentElement.clientWidth/2)<3&&Math.abs(rect.top+rect.height/2-window.innerHeight/2)<3;
});
byId('r0228-center-close').click();await wait('centered notice closed',()=>!centeredPanel().matches(':popover-open'));
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
await focusControl(byId('r0211-overflow-last'));if(!key(active(),'Tab').defaultPrevented)throw new Error('Header forward boundary not trapped');
await wait('Header forward wrap',()=>active()===byId('r0211-overflow-note'));
if(!key(active(),'Tab',true).defaultPrevented)throw new Error('Header reverse boundary not trapped');
await wait('Header reverse wrap',()=>active()===byId('r0211-overflow-last'));
key(active(),'Escape');await wait('Header Escape return',()=>header().dataset.open==='false'&&active()===more());
byId('r0211-header-veto').click();await wait('Header veto enabled',()=>byId('r0211-header-state').dataset.veto==='true');
more().focus();key(more(),'End');await wait('Header End entry',()=>header().dataset.open==='true'&&active()===byId('r0211-overflow-last'));
const headerRequests=+byId('r0211-header-state').dataset.closeRequests;
key(active(),'Escape');await wait('Header close veto',()=>+byId('r0211-header-state').dataset.closeRequests>headerRequests&&header().dataset.open==='true');
if(+byId('r0211-header-state').dataset.closeRequests!==headerRequests+1)throw new Error('Header Escape requested twice');
byId('r0211-overflow-note').focus();key(active(),'F4');await wait('Header veto released',()=>byId('r0211-header-state').dataset.veto==='false');
key(active(),'Escape');await wait('Header released close',()=>header().dataset.open==='false'&&active()===more());
byId('r0211-top-layer').click();await wait('inline Header selected',()=>header().dataset.topLayer==='false');
more().focus();key(more(),'Home');await wait('inline Header mounted focus',()=>header().dataset.open==='true'&&active()===byId('r0211-overflow-note'));
key(active(),'Escape');await wait('inline Header return',()=>header().dataset.open==='false'&&active()===more());
// Bounded geometry workflows use ordinary Dioxus controls/mounted scrolling.
// DOM access below only observes the resulting native layout.
const geometry=[];
const popup=()=>header().querySelector('[data-header-popup-host][data-active="true"]');
const popupVisible=()=>{
  const node=popup();if(!node||header().dataset.open!=='true')return false;
  const rect=node.getBoundingClientRect();
  return rect.width>0&&rect.height>0&&getComputedStyle(node).display!=='none'&&(header().dataset.topLayer!=='true'||node.matches(':popover-open'));
};
const positioned=()=>{
  if(!popupVisible())return false;
  const anchor=more().getBoundingClientRect(),rect=popup().getBoundingClientRect(),width=document.documentElement.clientWidth;
  const expectedLeft=Math.max(8,Math.min(anchor.left,width-rect.width-8));
  return Math.abs(rect.top-anchor.bottom)<3&&Math.abs(rect.left-expectedLeft)<3&&rect.left>=7&&rect.right<=width-7;
};
const checkGeometry=async label=>{
  await wait(label,positioned);
  const anchor=more().getBoundingClientRect(),rect=popup().getBoundingClientRect();
  geometry.push({label,layer:header().dataset.topLayer,small:header().dataset.small,left:Math.round(rect.left),top:Math.round(rect.top),anchorBottom:Math.round(anchor.bottom),width:Math.round(rect.width),viewport:document.documentElement.clientWidth});
};
const scrollRegion=byId('r0228-header-scroll');
for(const topLayer of [false,true]){
  if((header().dataset.topLayer==='true')!==topLayer){byId('r0211-top-layer').click();await wait('geometry layer selected',()=>header().dataset.topLayer===String(topLayer));}
  scrollRegion.scrollIntoView({block:'center'});
  more().focus();key(more(),'Home');
  await wait('geometry Header opened',()=>popupVisible()&&active()===headerField);
  await checkGeometry('opening');
  key(headerField,'F5');await wait('mounted ancestor scroll down',()=>Math.abs(scrollRegion.scrollTop-72)<2);
  await checkGeometry('ancestor scroll down');
  key(headerField,'F5');await wait('mounted ancestor scroll reset',()=>scrollRegion.scrollTop<2);
  await checkGeometry('ancestor scroll reset');
  const beforeWindow=more().getBoundingClientRect().top;
  window.scrollBy(0,window.scrollY>40?-36:36);
  await wait('window scroll moved Header',()=>Math.abs(more().getBoundingClientRect().top-beforeWindow)>12);
  await checkGeometry('window scroll');
  const beforeHeight=popup().getBoundingClientRect().height;
  key(headerField,'F6');await wait('content expanded',()=>byId('r0211-header-state').dataset.details==='true'&&popup().getBoundingClientRect().height>beforeHeight+10);
  await checkGeometry('content resize');
  const beforeViewport=document.documentElement.clientWidth;
  key(headerField,'F8');await wait('viewport and area resize',()=>byId('r0211-header-state').dataset.compact==='true'&&document.documentElement.clientWidth<beforeViewport-50);
  await checkGeometry('viewport resize');
  key(headerField,'F2');await wait('small responsive geometry',()=>header().dataset.small==='true'&&popupVisible());
  await checkGeometry('small layout');
  key(headerField,'F3');await wait('large responsive geometry',()=>header().dataset.small==='false'&&popupVisible());
  await checkGeometry('large layout');
  key(headerField,'F6');await wait('content reset',()=>byId('r0211-header-state').dataset.details==='false');
  key(headerField,'F8');await wait('viewport reset',()=>byId('r0211-header-state').dataset.compact==='false'&&document.documentElement.clientWidth>beforeViewport-20);
  await checkGeometry('restored layout');
  key(headerField,'Escape');await wait('geometry Header close return',()=>header().dataset.open==='false'&&active()===more());
}
return {ok:true,hoverFocusContainment:{triggerOpening:true,internalFocusRetained:true,outsideFocusDismissed:true,reopenAndExplicitClose:true,input:"Excluded synthetic native focus/click; real focus ownership and popover state observed."},popover:{contentFirst:true,typing:true,boundaries:2,positiveTabOrder:true,interiorDefault:true,escape:true,explicit:true,veto:true,outsidePreserved:true},hover:{nonStealing:true,returned:true},customAnchor:true,unanchoredCentered:true,header:{home:true,end:true,boundaries:2,interiorDefault:true,escape:true,veto:true,responsiveValue:headerField.value,inline:true,geometry}};
"#;
