//! Excluded representative Modal autofocus, dismissal and mounted-return workflows.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn ModalFocusPreview() -> Element {
    let modal = use_modal();
    let non_blocking = use_modal();
    let mut name = use_signal(|| "Pump P-101".to_string());
    let mut note = use_signal(|| "Inspection ready".to_string());
    let mut guard = use_signal(|| false);
    let mut pending = use_signal(|| None::<ModalDismissRequest>);
    let mut requests = use_signal(|| 0_usize);
    let mut results = use_signal(Vec::<String>::new);
    let mut error = use_signal(String::new);
    let field_mount = use_focus_mount("r0212-name");
    let note_mount = use_focus_mount("r0212-note");
    let custom_mount = use_focus_mount("r0212-custom-open");
    let input_mount = use_focus_mount("r0212-input-open");
    let non_blocking_custom_mount = use_focus_mount("r0260-nonblocking-custom-open");
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class:"modal-focus-review",
            h1 { "Dioxus-owned Modal return focus" }
            p { "Edit the equipment form, then confirm or dismiss it. Closing returns to the control that opened the dialog." }
            section {
                h2 { "Equipment form" }
                div { class:"modal-focus-controls",
                    div { id:"r0212-open-a", ModalTrigger { modal, "Edit equipment" } }
                    div { id:"r0212-open-b", ModalTrigger { modal, variant:ButtonVariant::Secondary, "Edit from overview" } }
                    button { id:"r0212-custom-open", class:"ix-button ix-button-secondary", r#type:"button", onmounted:custom_mount,
                        onclick:move |_|{spawn(async move {if let Err(reason)=modal.open().await {error.set(reason.to_string());}});}, "Open with custom control" }
                }
                label { r#for:"r0212-input-open", "Registered opener note (F2 opens the form)" }
                input { id:"r0212-input-open", class:"modal-focus-field", value:"Equipment overview", onmounted:input_mount,
                    onkeydown:move |event:KeyboardEvent|if event.key()==Key::F2 {
                        event.prevent_default();spawn(async move {if let Err(reason)=modal.open().await {error.set(reason.to_string());}});
                    } }
                div { class:"modal-focus-controls",
                    Button { id:"r0212-guard-toggle", variant:ButtonVariant::Secondary, onclick:move |_|guard.toggle(),
                        if guard(){"Disable close guard"}else{"Enable close guard"} }
                    button { class:"ix-button ix-button-subtle-secondary", r#type:"button", "data-r0212-unregistered":"true",
                        onclick:move |_|{spawn(async move {if let Err(reason)=modal.open().await {error.set(reason.to_string());}});}, "Open from unregistered control" }
                }
                p { "The close guard keeps the form open until you allow or deny a pending request. Confirmation always completes the form." }
                Modal { modal, label:"Equipment modal form", size:ModalSize::Sm, centered:true, close_on_backdrop_click:true,
                    onresult:move |result|results.write().push(format!("{result:?}")),
                    onopenchange:move |open: bool|if !open {pending.set(None);},
                    ondismissrequest:move |request:ModalDismissRequest| {
                        requests.set(requests()+1);
                        if guard(){pending.set(Some(request));}else{request.allow();}
                    },
                    ModalHeader { close_label:"Close equipment modal", "Equipment details" }
                    ModalContent {
                        label { r#for:"r0212-name", "Equipment name" }
                        input { id:"r0212-name", class:"modal-focus-field", value:name(), autofocus:true, onmounted:field_mount,
                            oninput:move |event|name.set(event.value()) }
                        p { "Use the header close button, Cancel, Escape or the backdrop to request dismissal." }
                        if modal.is_dismiss_pending() {
                            div { class:"modal-focus-decision", role:"status",
                                p { "Close request pending. Keep editing, or choose whether to dismiss." }
                                div { class:"modal-focus-controls",
                                    Button { id:"r0212-deny", variant:ButtonVariant::Secondary,
                                        onclick:move |_|{let request=pending.write().take();if let Some(request)=request{request.deny();}}, "Keep editing" }
                                    Button { id:"r0212-allow", variant:ButtonVariant::Primary,
                                        onclick:move |_|{let request=pending.write().take();if let Some(request)=request{request.allow();}}, "Allow dismissal" }
                                }
                            }
                        }
                    }
                    ModalFooter {
                        span { id:"r0212-cancel", ModalClose { dismiss:true, "Cancel" } }
                        span { id:"r0212-confirm", ModalClose { variant:ButtonVariant::Primary, value:format!("equipment={}",name()), "Save equipment" } }
                    }
                }
                output { id:"r0212-state", "data-open":modal.is_open().to_string(), "data-pending":modal.is_dismiss_pending().to_string(),
                    "data-guard":guard().to_string(), "data-requests":requests(), "data-result":format!("{:?}",modal.result()),
                    "Name: {name()}; last result: {modal.result():?}" }
                output { id:"r0212-results", "Results: {results():?}" }
                if !error().is_empty() { p { id:"r0212-error", role:"alert", "Opening failed: {error()}" } }
            }
            section {
                h2 { "Nonblocking dialog" }
                p { "This panel leaves the rest of the page available. Closing still returns to its registered opener." }
                div { class:"modal-focus-controls",
                    div { id:"r0212-nonblocking-open", ModalTrigger { modal:non_blocking, variant:ButtonVariant::Secondary, "Open inspection note" } }
                    button { id:"r0260-nonblocking-custom-open", class:"ix-button ix-button-secondary", r#type:"button", onmounted:non_blocking_custom_mount,
                        onclick:move |_|{spawn(async move {if let Err(reason)=non_blocking.open().await {error.set(reason.to_string());}});}, "Open note with custom control" }
                }
                Modal { modal:non_blocking, label:"Nonblocking inspection note", is_non_blocking:true, size:ModalSize::Xs,
                    ModalHeader { close_label:"Close inspection note", "Inspection note" }
                    ModalContent {
                        label { r#for:"r0212-note", "Note" }
                        input { id:"r0212-note", class:"modal-focus-field", value:note(), autofocus:true, onmounted:note_mount,
                            oninput:move |event|note.set(event.value()) }
                    }
                    ModalFooter { span { id:"r0212-nonblocking-close", ModalClose { value:"note saved", "Save note" } } }
                }
                output { id:"r0212-nonblocking-state", "data-open":non_blocking.is_open().to_string(),
                    "data-result":format!("{:?}",non_blocking.result()), "Note: {note()}" }
            }
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=super::dom_query::eval(NATIVE_MODAL_CHECK);let _=query.send("modal-focus-review");
                    match query.join::<serde_json::Value>().await {
                        Ok(value)if value["ok"]==true=>{println!("Dioxus Modal focus native: {value}");window.close();}
                        other=>{eprintln!("Dioxus Modal focus native failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus Modal focus native timed out");std::process::exit(1);});}
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.modal-focus-review{min-height:100vh;box-sizing:border-box;padding:24px}.modal-focus-review>h1{font-size:28px;margin:0 0 16px}.modal-focus-review p{max-width:65ch;line-height:1.5}.modal-focus-review section{max-width:900px;margin:28px 0;padding-bottom:24px;border-bottom:1px solid var(--ix-color-border)}.modal-focus-review h2{font-size:20px}.modal-focus-controls{display:flex;flex-wrap:wrap;align-items:center;gap:12px;margin:16px 0}.modal-focus-review label{display:block;margin-bottom:8px}.modal-focus-field{box-sizing:border-box;min-height:32px;max-width:100%;width:260px;padding:4px 8px;background:var(--ix-color-component-1);color:inherit;border:1px solid var(--ix-color-border);font:inherit}.modal-focus-field:focus-visible{outline:2px solid var(--ix-color-primary);outline-offset:2px}.modal-focus-review output{display:block;margin-top:12px;overflow-wrap:anywhere}.modal-focus-decision{margin-top:20px;padding-top:12px;border-top:1px solid var(--ix-color-border)}@media(max-width:600px){.modal-focus-review{padding:16px}}";

// Excluded synthetic Linux checks. Dispatching cancel exercises the Rust Escape
// request; physical Escape and native Tab containment are verified in Chromium.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_MODAL_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),active=()=>document.activeElement;
const wait=async(label,predicate)=>{for(let n=0;n<200;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' focus='+active()?.id+' error='+byId('r0212-error')?.textContent);};
const button=id=>byId(id).querySelector('button'),openerA=()=>button('r0212-open-a'),openerB=()=>button('r0212-open-b');
const panel=()=>byId(openerA().getAttribute('aria-controls')),isOpen=()=>panel().open&&byId('r0212-state').dataset.open==='true';
const closed=()=>!panel().open&&byId('r0212-state').dataset.open==='false';
const open=async(node,label)=>{node.scrollIntoView({block:'center'});node.focus();node.click();await wait(label,()=>isOpen()&&active()===byId('r0212-name'));};
const returned=async(node,label)=>wait(label,()=>closed()&&active()===node);
await wait('Modal triggers mounted',()=>openerA()&&openerB());
await open(openerA(),'first typed opener autofocus');
byId('r0212-name').value='Pump revised';byId('r0212-name').dispatchEvent(new Event('input',{bubbles:true}));
await wait('equipment editing',()=>byId('r0212-state').textContent.includes('Pump revised'));
button('r0212-confirm').click();await returned(openerA(),'confirmation returns first typed opener');
await wait('confirmation payload',()=>byId('r0212-state').dataset.result.includes('equipment=Pump revised'));
await open(openerB(),'second typed opener autofocus');
button('r0212-cancel').click();await returned(openerB(),'Cancel returns actual second opener');
await open(openerA(),'header close setup');
panel().querySelector('[aria-label="Close equipment modal"]').click();await returned(openerA(),'header close return');
await open(openerB(),'Escape request setup');
panel().dispatchEvent(new Event('cancel',{bubbles:false,cancelable:true}));await returned(openerB(),'Escape request return');
const custom=byId('r0212-custom-open');await open(custom,'registered imperative opener autofocus');
button('r0212-confirm').click();await returned(custom,'registered imperative opener return');
const input=byId('r0212-input-open');input.focus();input.dispatchEvent(new KeyboardEvent('keydown',{key:'F2',code:'F2',bubbles:true,cancelable:true}));
await wait('registered input imperative autofocus',()=>isOpen()&&active()===byId('r0212-name'));
button('r0212-cancel').click();await returned(input,'registered input imperative return');
byId('r0212-guard-toggle').click();await wait('guard enabled',()=>byId('r0212-state').dataset.guard==='true');
await open(openerA(),'guarded form autofocus');button('r0212-cancel').click();
await wait('dismissal pending',()=>isOpen()&&byId('r0212-state').dataset.pending==='true'&&byId('r0212-deny'));
byId('r0212-deny').click();await wait('denied dismissal retains form',()=>isOpen()&&byId('r0212-state').dataset.pending==='false');
button('r0212-cancel').click();await wait('second dismissal pending',()=>byId('r0212-state').dataset.pending==='true'&&byId('r0212-allow'));
byId('r0212-allow').click();await returned(openerA(),'allowed dismissal return');
byId('r0212-guard-toggle').click();await wait('guard disabled',()=>byId('r0212-state').dataset.guard==='false');
const nonblocking=button('r0212-nonblocking-open'),notePanel=()=>byId(nonblocking.getAttribute('aria-controls'));
nonblocking.focus();nonblocking.click();await wait('nonblocking autofocus',()=>notePanel().matches(':popover-open')&&active()===byId('r0212-note'));
byId('r0212-note').value='Inspection revised';byId('r0212-note').dispatchEvent(new Event('input',{bubbles:true}));
await wait('nonblocking editing',()=>byId('r0212-nonblocking-state').textContent.includes('Inspection revised'));
button('r0212-nonblocking-close').click();await wait('nonblocking return',()=>!notePanel().matches(':popover-open')&&active()===nonblocking);
await wait('native nonblocking result',()=>byId('r0212-nonblocking-state').dataset.result.includes('Closed("note saved")'));
const nonblockingCustom=byId('r0260-nonblocking-custom-open');
const openCustomNote=async label=>{nonblockingCustom.scrollIntoView({block:'center'});nonblockingCustom.focus();nonblockingCustom.click();await wait(label,()=>notePanel().matches(':popover-open')&&byId('r0212-nonblocking-state').dataset.open==='true'&&active()===byId('r0212-note'));};
const customNoteReturned=async label=>wait(label,()=>!notePanel().matches(':popover-open')&&byId('r0212-nonblocking-state').dataset.open==='false'&&active()===nonblockingCustom);
await openCustomNote('imperative nonblocking autofocus');
byId('r0212-note').value='Inspection from custom opener';byId('r0212-note').dispatchEvent(new Event('input',{bubbles:true}));
await wait('imperative nonblocking editing',()=>byId('r0212-nonblocking-state').textContent.includes('Inspection from custom opener'));
button('r0212-nonblocking-close').click();await customNoteReturned('imperative nonblocking confirmation return');
await wait('imperative nonblocking result',()=>byId('r0212-nonblocking-state').dataset.result.includes('Closed("note saved")'));
await openCustomNote('imperative nonblocking Escape setup');
byId('r0212-note').dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',code:'Escape',bubbles:true,cancelable:true}));
await customNoteReturned('imperative nonblocking Escape return');
await wait('imperative nonblocking Escape result',()=>byId('r0212-nonblocking-state').dataset.result.includes('Dismissed(Escape)'));
const raw=document.querySelector('[data-r0212-unregistered]');if(raw.id)throw new Error('unregistered fixture must not have an ID');
await open(raw,'unregistered opener still opens');button('r0212-confirm').click();await wait('unregistered close',closed);
if(raw.id||raw.hasAttribute('data-ix-popover-focus-token'))throw new Error('unregistered opener received a focus identity');
return {ok:true,autofocus:true,editing:true,typedOpeners:2,confirmationPayload:true,registeredImperativeButton:true,registeredImperativeInput:true,header:true,cancel:true,escapeRequest:true,guard:{denied:true,allowed:true},nonblocking:true,imperativeNonblocking:{autofocus:true,editing:true,result:true,escapeRequest:true,registeredReturn:true},unregisteredIdMint:false,nativeTab:'physical browser check'};
"#;
