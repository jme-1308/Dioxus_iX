//! Native regression fixtures for text-control lifecycle and source event behavior.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::{cell::RefCell, rc::Rc};

#[path = "../gallery/date_dropdown_review.rs"]
mod date_dropdown_review;
#[path = "../gallery/dropdown_review.rs"]
mod dropdown_review;

#[component]
pub fn Fixtures() -> Element {
    let input = use_input();
    let inspect = input.clone();
    let stale_controller = input.clone();
    let retained = use_hook(|| Rc::new(RefCell::new(None::<NativeTextControl>)));
    let save = retained.clone();
    let stale = retained.clone();
    let mut mounted = use_signal(|| true);
    let mut email = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut events = use_signal(|| 0usize);
    let mut note_events = use_signal(|| 0usize);
    let mut inspection = use_signal(String::new);
    let mut rejection = use_signal(|| false);
    let mut custom = use_signal(|| FieldState::Default);
    let mut raw = use_signal(|| "0".to_string());
    let mut number = use_signal(|| "none".to_string());
    let mut commits = use_signal(|| 0usize);
    let mut submits = use_signal(|| 0usize);
    let mut empty = use_signal(|| "4".to_string());
    let mut zero = use_signal(|| "4".to_string());
    let mut empty_event = use_signal(|| "untouched".to_string());
    let mut zero_event = use_signal(|| "untouched".to_string());
    let number_control = use_number_input();
    let inspect_number = number_control.clone();
    let mut number_presence = use_signal(|| false);
    rsx! {
        section { id: "v14-native-fields", style: "width:400px",
            form { id: "v14-input-form", name: "v14-form", method: "post", onsubmit: move |event| event.prevent_default(),
                if mounted() {
                    Input { id: "v14-email", controller: input, label: "Native required email", name: "email", input_type: "email", required: true,
                        value: email(), onvaluechange: move |value| email.set(value), helper_text: "Email helper", invalid_text: "Invalid native email",
                        onvaliditystatechange: move |_| events += 1 }
                }
                Textarea { id: "v14-notes", label: "Native required notes", required: true, value: notes(),
                    onvaluechange: move |value| notes.set(value), invalid_text: "Invalid native notes", onvaliditystatechange: move |_| note_events += 1 }
            }
            form { novalidate: true,
                Input { id: "v14-unvalidated", label: "Native suppressed required field", required: true, invalid_text: "Suppressed error" }
            }
            button { id: "v14-inspect-input", r#type: "button", onclick: move |_| {
                let controller = inspect.clone(); let save = save.clone();
                spawn(async move {
                    *save.borrow_mut() = controller.get_native_input_element().ok();
                    let focused = controller.focus_input().await.is_ok();
                    let missing = controller.get_validity_state().await.is_ok_and(|state| state.value_missing && !state.valid);
                    let form = controller.get_associated_form_element().await.ok().flatten();
                    let form = form.map(|form| format!("{}|{}|{}|{}", form.id, form.name, form.method, form.no_validate)).unwrap_or_default();
                    inspection.set(format!("focused={focused};missing={missing};form={form};touched={};nonempty={}", controller.is_touched().unwrap_or(true), controller.has_valid_value().unwrap_or(true)));
                });
            }, "Inspect native input" }
            button { id: "v14-unmount-input", r#type: "button", onclick: move |_| mounted.set(false), "Unmount native input" }
            button { id: "v14-stale-input", r#type: "button", onclick: move |_| {
                let descriptor = stale.borrow().clone(); let controller = stale_controller.clone();
                spawn(async move {
                    let rejected = if let Some(descriptor) = descriptor { matches!(descriptor.focus().await, Err(TextControlError::NotMounted)) } else { false };
                    rejection.set(rejected && matches!(controller.get_native_input_element(), Err(TextControlError::NotMounted)));
                });
            }, "Check stale native input" }
            output { id: "v14-input-events", "{events}" }
            output { id: "v14-note-events", "{note_events}" }
            output { id: "v14-input-inspection", "{inspection}" }
            output { id: "v14-input-stale", "{rejection}" }
            button { id: "v14-custom-invalid", r#type: "button", onclick: move |_| custom.set(FieldState::Invalid), "Invalidate custom control" }
            button { id: "v14-custom-valid", r#type: "button", onclick: move |_| custom.set(FieldState::Valid), "Validate custom control" }
            div { id: "v14-custom-field",
                CustomField { label: "Native custom control", helper_text: "Custom helper", invalid_text: "Custom invalid", valid_text: "Custom valid",
                    div { class: if custom() == FieldState::Invalid { "ix-invalid" } else if custom() == FieldState::Valid { "ix-valid" } else { "" },
                        button { r#type: "button", "Custom native action" }
                    }
                }
            }
            form { id: "v14-number-form", onsubmit: move |event| { event.prevent_default(); submits += 1; },
                NumberInput { id: "v14-number", label: "Native numeric commit", value: raw(), onvaluechange: move |value| raw.set(value),
                    suppress_submit_on_enter: true, onnumericvaluechange: move |value| number.set(number_text(value)), oncommit: move |_| commits += 1 }
                NumberInput { id: "v14-empty-number", controller: number_control, label: "Native empty numeric event", value: empty(), onvaluechange: move |value| empty.set(value),
                    allow_empty_value_change: true, onnumericvaluechange: move |value| empty_event.set(number_text(value)) }
                NumberInput { id: "v14-zero-number", label: "Native zero numeric event", value: zero(), onvaluechange: move |value| zero.set(value),
                    onnumericvaluechange: move |value| zero_event.set(number_text(value)) }
            }
            output { id: "v14-number-events", "{number}" }
            output { id: "v14-number-commits", "{commits}" }
            output { id: "v14-number-submits", "{submits}" }
            output { id: "v14-number-empty", "{empty_event}" }
            output { id: "v14-number-zero", "{zero_event}" }
            button { id: "v14-number-inspect", r#type: "button", onclick: move |_| number_presence.set(inspect_number.has_valid_value() == Ok(true)), "Inspect optional numeric value" }
            output { id: "v14-number-presence", "{number_presence}" }
            dropdown_review::DropdownOrderReview {}
            date_dropdown_review::DateDropdownReview {}
        }
    }
}

fn number_text(value: Option<f64>) -> String {
    value.map_or_else(|| "none".into(), |value| value.to_string())
}

async fn run(script: &str) -> Result<(), String> {
    super::native_eval::<()>(script).await
}

async fn focus(id: &str) -> Result<(), String> {
    let id = serde_json::to_string(id).map_err(|error| error.to_string())?;
    run(&format!(
        "window.__ixProbeFocus(document.getElementById({id}));return null;"
    ))
    .await?;
    super::wait_native_observation("Native field actual focus", &format!(r#"
        const input=document.getElementById({id}), active=document.activeElement;
        return {{ok:!!input&&active===input,id:{id},active:active?.id,disabled:input?.matches(':disabled'),modals:[...document.querySelectorAll(':modal')].map(node=>node.id)}};
    "#)).await?;
    Ok(())
}

async fn blur(id: &str) -> Result<(), String> {
    let id = serde_json::to_string(id).map_err(|error| error.to_string())?;
    run(&format!(
        "window.__ixProbeFocus(document.getElementById({id}),false);return null;"
    ))
    .await
}

async fn edit(id: &str, text: &str) -> Result<(), String> {
    let args = serde_json::to_string(&(id, text)).map_err(|error| error.to_string())?;
    run(&format!(r#"const [id,text]={args},input=document.getElementById(id);input.value=text;input.dispatchEvent(new InputEvent('input',{{bubbles:true,inputType:'insertText',data:text}}));return null;"#)).await
}

pub async fn verify() -> Result<(), String> {
    super::wait_native_observation("Native required field starts untouched", r#"
        const input=document.getElementById('v14-email');
        return {ok:input?.getAttribute('aria-invalid')==='false'&&input.dataset.ixTouched==='false'&&document.getElementById('v14-input-events').textContent==='0'};
    "#).await?;
    run("document.getElementById('v14-inspect-input').click();return null;").await?;
    super::wait_native_observation("Native input methods and associated form", r#"
        const text=document.getElementById('v14-input-inspection').textContent;
        return {ok:text==='focused=true;missing=true;form=v14-input-form|v14-form|post|false;touched=false;nonempty=false',text};
    "#).await?;
    focus("v14-email").await?;
    blur("v14-email").await?;
    super::wait_native_observation("Native Input required blur publishes one invalid transition", r#"
        const input=document.getElementById('v14-email'),events=document.getElementById('v14-input-events').textContent;
        return {ok:input.getAttribute('aria-invalid')==='true'&&input.dataset.ixTouched==='true'&&events==='1',events,invalid:input.getAttribute('aria-invalid')};
    "#).await?;
    focus("v14-email").await?;
    blur("v14-email").await?;
    focus("v14-email").await?;
    // The next bounded observation occurs after a real second blur, without
    // replaying mutations or accepting extra validity notifications.
    super::wait_native_observation(
        "Native Input unchanged validity is deduplicated",
        r#"
        const events=document.getElementById('v14-input-events').textContent;
        return {ok:events==='1',events};
    "#,
    )
    .await?;
    edit("v14-email", "operator@example.com").await?;
    super::wait_native_observation("Native Input correction publishes valid transition", r#"
        const input=document.getElementById('v14-email'),events=document.getElementById('v14-input-events').textContent;
        return {ok:input.value==='operator@example.com'&&input.getAttribute('aria-invalid')==='false'&&events==='2',events,invalid:input.getAttribute('aria-invalid')};
    "#).await?;
    blur("v14-email").await?;
    focus("v14-notes").await?;
    blur("v14-notes").await?;
    super::wait_native_observation("Native Textarea required blur", r#"
        const input=document.getElementById('v14-notes');return {ok:input.getAttribute('aria-invalid')==='true'&&document.getElementById('v14-note-events').textContent==='1'};
    "#).await?;
    focus("v14-notes").await?;
    edit("v14-notes", "Service complete").await?;
    super::wait_native_observation("Native Textarea correction", r#"
        const input=document.getElementById('v14-notes');return {ok:input.getAttribute('aria-invalid')==='false'&&document.getElementById('v14-note-events').textContent==='2'};
    "#).await?;
    blur("v14-notes").await?;
    focus("v14-unvalidated").await?;
    blur("v14-unvalidated").await?;
    super::wait_native_observation("Native noValidate retains touched state and suppresses automatic feedback", r#"
        const input=document.getElementById('v14-unvalidated');return {ok:input.dataset.ixTouched==='true'&&input.getAttribute('aria-invalid')==='false'&&input.validity.valueMissing&&input.form.noValidate};
    "#).await?;
    for (button, expected) in [
        ("v14-custom-invalid", "Custom invalid"),
        ("v14-custom-valid", "Custom valid"),
    ] {
        run(&format!(
            "document.getElementById('{button}').click();return null;"
        ))
        .await?;
        super::wait_native_observation("CustomField follows descendant state", &format!(r#"
            const helper=document.querySelector('#v14-custom-field .ix-helper');return {{ok:helper?.textContent==='{expected}',text:helper?.textContent}};
        "#)).await?;
    }
    run("document.getElementById('v14-unmount-input').click();return null;").await?;
    super::wait_native_observation(
        "Native Input unmount",
        "return {ok:!document.getElementById('v14-email')};",
    )
    .await?;
    run("document.getElementById('v14-stale-input').click();return null;").await?;
    super::wait_native_observation(
        "Retained text descriptor and controller reject unmount",
        "return {ok:document.getElementById('v14-input-stale').textContent==='true'};",
    )
    .await?;
    verify_numbers().await?;
    verify_dropdown_order().await?;
    verify_date_notifications().await?;
    println!(
        "Native Input/Textarea touched constraints, transition deduplication, noValidate, controller/form methods, stale descriptors, CustomField, NumberInput, keyed Dropdown and DateDropdown source events verified."
    );
    Ok(())
}

async fn verify_numbers() -> Result<(), String> {
    focus("v14-number").await?;
    edit("v14-number", "1E3").await?;
    super::wait_native_observation(
        "Native NumberInput scientific input",
        "return {ok:document.getElementById('v14-number-events').textContent==='1000'};",
    )
    .await?;
    let prevented: bool = super::native_eval(r#"
        const input=document.getElementById('v14-number');const event=new KeyboardEvent('keydown',{key:'Enter',code:'Enter',bubbles:true,cancelable:true});
        input.dispatchEvent(event);return event.defaultPrevented;
    "#).await?;
    if !prevented {
        return Err("Native NumberInput suppress-submit did not cancel Enter.".into());
    }
    super::wait_native_observation(
        "Native NumberInput Enter commits once",
        "return {ok:document.getElementById('v14-number-commits').textContent==='1'};",
    )
    .await?;
    blur("v14-number").await?;
    super::wait_native_observation("Native NumberInput blur normalizes without duplicate commit", r#"
        const input=document.getElementById('v14-number'),commits=document.getElementById('v14-number-commits').textContent,submits=document.getElementById('v14-number-submits').textContent;
        return {ok:input.value==='1000'&&commits==='1'&&submits==='0',value:input.value,commits,submits};
    "#).await?;
    for id in ["v14-empty-number", "v14-zero-number"] {
        focus(id).await?;
        edit(id, "").await?;
        blur(id).await?;
    }
    super::wait_native_observation("Native NumberInput distinguishes allowed empty events from default zero", r#"
        const empty=document.getElementById('v14-number-empty').textContent,zero=document.getElementById('v14-number-zero').textContent;
        return {ok:empty==='none'&&zero==='0',empty,zero};
    "#).await?;
    run("document.getElementById('v14-number-inspect').click();return null;").await?;
    super::wait_native_observation("Native optional empty NumberInput satisfies numeric controller presence semantics", r#"
        const value=document.getElementById('v14-empty-number').value,presence=document.getElementById('v14-number-presence').textContent;
        return {ok:value===''&&presence==='true',value,presence};
    "#).await?;
    Ok(())
}

async fn verify_dropdown_order() -> Result<(), String> {
    run("document.querySelector('#dropdown-order-review [role=button][data-ix-overlay-trigger]').click();return null;").await?;
    super::wait_native_observation("Native reorderable Dropdown opens and focuses its first enabled item", r#"
        const menu=document.querySelector('#dropdown-order-review .ix-dropdown:not(.ix-dropdown-submenu-surface) > .ix-dropdown-root-menu'),active=document.activeElement;
        return {ok:!!menu&&active?.closest('[role=menu]')===menu&&active.textContent.trim()==='A',active:active?.tagName==='BODY'?'BODY':active?.textContent?.slice(0,120)};
    "#).await?;
    let focused: bool = super::native_eval(r#"
        const menu=document.querySelector('#dropdown-order-review .ix-dropdown:not(.ix-dropdown-submenu-surface) > .ix-dropdown-root-menu');
        const c=[...menu.querySelectorAll('[data-ix-dropdown-item]')].find(node=>node.closest('[role=menu]')===menu&&node.textContent.trim()==='C');window.__ixProbeFocus(c);const focused=document.activeElement===c;window.__ixProbeDropdownC=c;c.click();return focused;
    "#).await?;
    if !focused {
        return Err("Native Dropdown C did not receive actual focus before selection.".into());
    }
    super::wait_native_observation("Native Dropdown keyed C remains focused after reorder", r#"
        const menu=document.querySelector('#dropdown-order-review .ix-dropdown:not(.ix-dropdown-submenu-surface) > .ix-dropdown-root-menu');
        const items=[...menu?.querySelectorAll('[data-ix-dropdown-item]')??[]].filter(node=>node.closest('[role=menu]')===menu&&node.getAttribute('aria-disabled')!=='true');
        return {ok:items.map(node=>node.textContent.trim()).join(',')==='C,A,B,Tools'&&document.activeElement===items[0]&&items[0]===window.__ixProbeDropdownC,labels:items.map(node=>node.textContent.trim()),sameNode:items[0]===window.__ixProbeDropdownC,active:document.activeElement?.tagName==='BODY'?'BODY':document.activeElement?.textContent?.slice(0,120)};
    "#).await?;
    // Queue parent navigation, then open another level in the same browser
    // turn. The parent measurement must not steal the newer submenu focus.
    run(r#"
        const menu=document.querySelector('#dropdown-order-review .ix-dropdown:not(.ix-dropdown-submenu-surface) > .ix-dropdown-root-menu');
        const c=window.__ixProbeDropdownC,tools=[...menu.querySelectorAll('[data-ix-dropdown-item]')].find(node=>node.closest('[role=menu]')===menu&&node.textContent.trim()==='Tools');
        if(document.activeElement!==c)throw new Error('Rapid-navigation fixture lost C focus');
        c.dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowDown',code:'ArrowDown',bubbles:true,cancelable:true}));
        window.__ixProbeFocus(tools);
        tools.dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowRight',code:'ArrowRight',bubbles:true,cancelable:true}));
        return null;
    "#).await?;
    const RAPID_SUBMENU: &str = r#"
        const menu=document.querySelector('#dropdown-order-review .ix-dropdown-submenu-surface > .ix-dropdown-root-menu'),active=document.activeElement;
        const tools=document.querySelector('#dropdown-order-review .ix-dropdown-submenu > [data-ix-dropdown-item]');
        return {ok:!!menu&&tools?.getAttribute('aria-expanded')==='true'&&active?.closest('[role=menu]')===menu&&active.textContent.trim()==='A',
          open:tools?.getAttribute('aria-expanded'),menu:!!menu,activeTag:active?.tagName,active:active?.tagName==='BODY'?'BODY':active?.textContent?.slice(0,120)};
    "#;
    super::wait_native_observation(
        "Rapid Dropdown child opening supersedes pending parent navigation",
        RAPID_SUBMENU,
    )
    .await?;
    let after_paint: serde_json::Value = super::native_eval(&format!(
        "await new Promise(requestAnimationFrame);await new Promise(resolve=>setTimeout(resolve,0));{RAPID_SUBMENU}"
    ))
    .await?;
    if after_paint["ok"] != true {
        return Err(format!(
            "Rapid Dropdown submenu lost newer focus after another paint/turn: {after_paint}"
        ));
    }
    run("document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowLeft',code:'ArrowLeft',bubbles:true,cancelable:true}));return null;").await?;
    super::wait_native_observation("Explicit-roving Dropdown Left closes child and retains root", r#"
        const tools=document.querySelector('#dropdown-order-review .ix-dropdown-submenu > [data-ix-dropdown-item]');
        return {ok:tools?.getAttribute('aria-expanded')==='false'&&document.activeElement===tools&&!document.querySelector('#dropdown-order-review .ix-dropdown:not(.ix-dropdown-submenu-surface) > .ix-dropdown-root-menu').hidden};
    "#).await?;
    for (key, expected) in [("Home", "C"), ("ArrowDown", "A"), ("End", "Tools")] {
        run(&format!("document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{{key:'{key}',code:'{key}',bubbles:true,cancelable:true}}));return null;")).await?;
        super::wait_native_observation("Native Dropdown navigation follows current keyed DOM order", &format!("return {{ok:document.activeElement?.textContent.trim()==='{expected}',active:document.activeElement?.textContent}};")).await?;
    }
    run("document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowRight',code:'ArrowRight',bubbles:true,cancelable:true}));return null;").await?;
    super::wait_native_observation("Native Dropdown submenu owns focus independently", r#"
        const menu=document.querySelector('#dropdown-order-review .ix-dropdown-submenu-surface > .ix-dropdown-root-menu');return {ok:!!menu&&document.activeElement.closest('[role=menu]')===menu&&document.activeElement.textContent.trim()==='A'};
    "#).await?;
    run(r#"
        const menu=document.querySelector('#dropdown-order-review .ix-dropdown-submenu-surface > .ix-dropdown-root-menu');
        const c=[...menu.querySelectorAll('[data-ix-dropdown-item]')].find(node=>node.textContent.trim()==='C');window.__ixProbeFocus(c);c.click();return null;
    "#).await?;
    super::wait_native_observation("Native submenu keyed reorder retains focus", r#"
        const menu=document.querySelector('#dropdown-order-review .ix-dropdown-submenu-surface > .ix-dropdown-root-menu');
        return {ok:[...menu.querySelectorAll('[data-ix-dropdown-item]')].filter(node=>node.getAttribute('aria-disabled')!=='true').map(node=>node.textContent.trim()).join(',')==='C,A,B'&&document.activeElement.textContent.trim()==='C'};
    "#).await?;
    for (key, expected) in [("Home", "C"), ("ArrowDown", "A"), ("End", "B")] {
        run(&format!("document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{{key:'{key}',code:'{key}',bubbles:true,cancelable:true}}));return null;")).await?;
        super::wait_native_observation("Native submenu traversal excludes parent items", &format!("return {{ok:document.activeElement?.textContent.trim()==='{expected}'&&document.activeElement.closest('[role=menu]').parentElement.classList.contains('ix-dropdown-submenu-surface'),active:document.activeElement?.textContent}};")).await?;
    }
    run("document.getElementById('dropdown-order-outside').click();return null;").await?;
    super::wait_native_observation(
        "Native keyed Dropdown closes",
        "const menus=[...document.querySelectorAll('#dropdown-order-review [role=menu]')];return {ok:menus.length===2&&menus.every(menu=>menu.hidden)};",
    )
    .await?;
    Ok(())
}

async fn verify_date_notifications() -> Result<(), String> {
    run("document.querySelector('#date-dropdown-event-review .ix-date-dropdown>button').click();return null;").await?;
    super::wait_native_observation(
        "Native DateDropdown calendar opens",
        "return {ok:!!document.querySelector('#date-dropdown-event-review [role=dialog]')};",
    )
    .await?;
    run("document.querySelectorAll('#date-dropdown-event-review .ix-date-preset')[1].click();return null;").await?;
    super::wait_native_observation("Native DateDropdown preset immediately publishes while remaining open", r#"
        const text=document.getElementById('date-dropdown-event-status').textContent;
        return {ok:text==='Range events: 1. week: 2026-10-10 to 2026-10-16'&&!!document.querySelector('#date-dropdown-event-review [role=dialog]'),text};
    "#).await?;
    run("document.querySelector('#date-dropdown-event-review [role=dialog]').dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',code:'Escape',bubbles:true,cancelable:true}));return null;").await?;
    super::wait_native_observation("Native DateDropdown Escape republishes current preset", r#"
        const text=document.getElementById('date-dropdown-event-status').textContent;return {ok:text==='Range events: 2. week: 2026-10-10 to 2026-10-16'&&!document.querySelector('#date-dropdown-event-review [role=dialog]'),text};
    "#).await?;
    run("document.querySelector('#date-dropdown-event-review .ix-date-dropdown>button').click();return null;").await?;
    super::wait_native_observation(
        "Native DateDropdown reopens",
        "return {ok:!!document.querySelector('#date-dropdown-event-review [role=dialog]')};",
    )
    .await?;
    run("[...document.querySelectorAll('#date-dropdown-event-review [role=dialog] button')].find(node=>node.textContent.trim()==='Done').click();return null;").await?;
    super::wait_native_observation("Native DateDropdown Done publishes selection and dismissal", r#"
        const text=document.getElementById('date-dropdown-event-status').textContent;return {ok:text==='Range events: 4. week: 2026-10-10 to 2026-10-16'&&!document.querySelector('#date-dropdown-event-review [role=dialog]'),text};
    "#).await?;
    Ok(())
}
