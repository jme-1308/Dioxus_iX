//! Hidden, bounded native DOM/controller and actual PNG-raster integration probe.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::time::{Duration, Instant};

// Exercise the same private transport as library DOM operations, including
// cancellation, without adding an implementation detail to the crate's API.
#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "native_bridge_probe/forms.rs"]
mod native_form_probe;
#[path = "native_bridge_probe/navigation.rs"]
mod native_navigation_probe;

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("Dioxus iX native bridge probe")
                .with_visible(false)
                .with_always_on_top(false)
                .with_inner_size(dioxus::desktop::LogicalSize::new(800.0, 900.0)),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}

async fn native_matches(id: String, selector: &str) -> Result<bool, String> {
    let task = native_query(
        "const [id, selector] = await dioxus.recv(); return document.getElementById(id)?.matches(selector) === true;",
    );
    task.send((id, selector))
        .map_err(|error| error.to_string())?;
    task.join().await.map_err(|error| error.to_string())
}

// The pinned desktop renderer can collect a completed eval before Rust polls
// its return value. Keep each query alive until its actual result is received
// and acknowledged. Scripts, including input dispatch, execute exactly once.
struct NativeQuery {
    task: dom_query::OneShot,
}

fn native_query(script: &str) -> NativeQuery {
    NativeQuery {
        task: dom_query::eval(script),
    }
}

impl NativeQuery {
    fn send(&self, value: impl serde::Serialize) -> Result<(), dioxus::document::EvalError> {
        self.task.send(value)
    }

    async fn join<T: serde::de::DeserializeOwned>(self) -> Result<T, String> {
        self.task.join().await.map_err(|error| format!("{error:?}"))
    }
}

impl std::future::IntoFuture for NativeQuery {
    type Output = Result<serde_json::Value, String>;
    type IntoFuture = std::pin::Pin<Box<dyn std::future::Future<Output = Self::Output>>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.join())
    }
}

async fn native_eval<T: serde::de::DeserializeOwned>(script: &str) -> Result<T, String> {
    native_query(script).join().await
}

/// Observe actual asynchronous DOM state with a finite deadline. Keep the last
/// measured fields in failures instead of guessing that one delay was enough.
async fn wait_native_observation(label: &str, script: &str) -> Result<serde_json::Value, String> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let observation: serde_json::Value = native_eval(script)
            .await
            .map_err(|error| format!("{label} observation failed: {error}"))?;
        if observation["ok"] == true {
            return Ok(observation);
        }
        if Instant::now() >= deadline {
            return Err(format!("{label}: {observation}"));
        }
        futures_timer::Delay::new(Duration::from_millis(25)).await;
    }
}

async fn exercise(
    modal: ModalHandle,
    popover: PopoverHandle,
    select: SelectController,
) -> Result<(), String> {
    futures_timer::Delay::new(Duration::from_millis(500)).await;
    initialize_hidden_focus_delivery().await?;
    // Run the navigation boundary independently when an unrelated broad probe
    // fails before reaching it. The default remains the complete regression.
    if std::env::var("DIOXUS_IX_PROBE_SCOPE").as_deref() == Ok("navigation") {
        native_navigation_probe::verify().await?;
        println!("Scoped native navigation regression passed (menu overlays and tabs).");
        return Ok(());
    }
    verify_native_query_lifecycle().await?;
    verify_theme_tokens().await?;
    verify_native_surface().await?;
    verify_native_surface_motion().await?;
    verify_native_zoom().await?;
    let png = export_chart_png("bridge-probe-chart", 1.0)
        .await
        .map_err(|e| e.to_string())?;
    let bytes = png.bytes().map_err(|e| e.to_string())?;
    if png.width < 300 || png.height != 600 {
        return Err(format!("Unexpected PNG size: {}×{}", png.width, png.height));
    }
    let directory = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join("target")
        .join("review");
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let path = directory.join("desktop-chart-export.png");
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    println!(
        "Native PNG export: {}×{} pixels at {}",
        png.width,
        png.height,
        path.display()
    );
    modal.open().await.map_err(|e| e.to_string())?;
    if !modal.is_open() || !native_matches(modal.id(), ":modal").await? {
        return Err("Programmatic dialog did not enter the native modal top layer.".into());
    }
    modal.close("native probe");
    futures_timer::Delay::new(Duration::from_millis(100)).await;
    if native_matches(modal.id(), ":modal").await? {
        return Err("Modal remained in the top layer after close.".into());
    }
    popover.show().await.map_err(|e| e.to_string())?;
    if !popover.is_open() || !native_matches(popover.id(), ":popover-open").await? {
        return Err("Programmatic popover did not enter the native top layer.".into());
    }
    popover.hide();
    futures_timer::Delay::new(Duration::from_millis(100)).await;
    if native_matches(popover.id(), ":popover-open").await? {
        return Err("Popover remained in the top layer after hide.".into());
    }
    println!("Native modal/popover open, top-layer membership and close verified.");
    verify_native_filter().await?;
    verify_native_select().await?;
    verify_native_select_sizing().await?;
    verify_native_select_validation().await?;
    verify_native_select_methods(select).await?;
    verify_native_tooltip().await?;
    verify_native_datetime_boundaries().await?;
    verify_native_grid_editing().await?;
    verify_native_dropdown_form().await?;
    verify_native_pane().await?;
    verify_native_tree().await?;
    verify_native_tree_model().await?;
    verify_native_grid().await?;
    native_form_probe::verify().await?;
    native_navigation_probe::verify().await?;
    let synthetic: usize = native_eval("return window.__ixProbeFocusState.synthetic;").await?;
    println!(
        "Hidden WebView focus delivery: {synthetic} synthetic focus-family events supplied only where native delivery was absent; actual activeElement assertions retained."
    );
    Ok(())
}

// Hidden WebViews can update activeElement while withholding focus events.
// Supply only missing events for explicitly requested probe focus operations;
// native focus methods and actual active-element checks still run unchanged.
async fn initialize_hidden_focus_delivery() -> Result<(), String> {
    native_eval::<()>(r#"
        const state=window.__ixProbeFocusState={focus:null,focusin:null,synthetic:0};
        for(const type of ['focus','focusin'])document.addEventListener(type,event=>state[type]=event.target,true);
        for(const [type,key] of [['blur','focus'],['focusout','focusin']])document.addEventListener(type,event=>{if(state[key]===event.target)state[key]=null;},true);
        const deliver=(node,type,relatedTarget)=>{
            state.synthetic++;
            node.dispatchEvent(new FocusEvent(type,{bubbles:type==='focusin'||type==='focusout',relatedTarget}));
        };
        window.__ixProbeFocus=(node,focus=true)=>{
            if(!(node instanceof Element))throw new Error('Probe focus target is missing');
            const previous=document.activeElement;
            if(focus)node.focus();else node.blur();
            const active=document.activeElement;
            if(active!==previous){
                if(state.focus===previous)deliver(previous,'blur',active);
                if(state.focusin===previous)deliver(previous,'focusout',active);
            }
            if(active===node&&focus){
                if(state.focus!==node)deliver(node,'focus',previous===node?null:previous);
                if(state.focusin!==node)deliver(node,'focusin',previous===node?null:previous);
            }
            return active===node;
        };
        return null;
    "#).await
}

async fn verify_native_query_lifecycle() -> Result<(), String> {
    let baseline: usize = native_eval(
        "window.__ix_query_probe = {never:0, once:0}; return window.__msg_queues.filter(Boolean).length;",
    ).await?;

    // Cancellation before arguments arrive must not leave a waiting channel.
    let waiting =
        native_query("await dioxus.recv(); window.__ix_query_probe.never++; return true;");
    drop(waiting);

    // Cancellation while a result is pending must release the acknowledgement
    // wait, while preserving exactly-once execution of already-started work.
    let mut pending = Box::pin(native_query(
        "window.__ix_query_probe.once++; await new Promise(resolve => setTimeout(resolve, 30));",
    ).join::<serde_json::Value>());
    std::future::poll_fn(|cx| {
        assert!(std::future::Future::poll(pending.as_mut(), cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    drop(pending);

    for _ in 0..32 {
        let task = native_query(
            "const value = await dioxus.recv(); return {value, bytes:'x'.repeat(8192)};",
        );
        task.send(42).map_err(|error| error.to_string())?;
        let reply: serde_json::Value = task.join().await?;
        if reply["value"] != 42 || reply["bytes"].as_str().map(str::len) != Some(8192) {
            return Err("Acknowledged DOM query lost its actual result.".into());
        }
    }
    let undefined: serde_json::Value = native_eval("return undefined;").await?;
    if !undefined.is_null() {
        return Err("Undefined DOM query result was not normalized to null.".into());
    }
    let error = native_query("throw new Error('expected native query failure');")
        .join::<bool>()
        .await;
    if !error.is_err_and(|error| error.contains("expected native query failure")) {
        return Err("Thrown DOM error was not returned to Rust.".into());
    }
    let task = native_query(
        r#"
        const baseline = await dioxus.recv();
        await new Promise(resolve => setTimeout(resolve, 60));
        const state = window.__ix_query_probe;
        const channels = window.__msg_queues.filter(Boolean).length;
        return {ok:state.never === 0 && state.once === 1 && channels <= baseline, channels, baseline, ...state};
    "#,
    );
    task.send(baseline).map_err(|error| error.to_string())?;
    let observed: serde_json::Value = task.join().await?;
    if observed["ok"] != true {
        return Err(format!(
            "DOM query cancellation or channel leak: {observed}"
        ));
    }
    println!(
        "Native DOM transport: 32 buffered results, pending/input cancellation, undefined and thrown-error cleanup verified: {observed}"
    );
    Ok(())
}

async fn verify_native_select_methods(controller: SelectController) -> Result<(), String> {
    if controller
        .has_valid_value()
        .map_err(|error| error.to_string())?
        || controller.is_touched().map_err(|error| error.to_string())?
    {
        return Err("Native Select method initial state is incorrect.".into());
    }
    let input = controller
        .get_native_input_element()
        .map_err(|error| error.to_string())?;
    if input.id() != "native-method-choice" {
        return Err("Native Select input descriptor has the wrong ID.".into());
    }
    let form = controller
        .get_associated_form_element()
        .await
        .map_err(|error| error.to_string())?
        .ok_or("Native Select did not report its associated form.")?;
    if form.id != "native-method-form" || form.name != "method-probe" || !form.no_validate {
        return Err(format!("Unexpected Select associated form: {form:?}"));
    }
    controller
        .focus_input()
        .await
        .map_err(|error| error.to_string())?;
    native_query(
        "window.__ixProbeFocus(document.getElementById('native-method-choice'));return null;",
    )
    .await?;
    wait_native_observation("Select method focus and tooltip", r#"
        const input=document.getElementById('native-method-choice');
        const tooltip=document.getElementById(input?.getAttribute('aria-describedby'));
        return {ok:document.activeElement===input && tooltip?.getAttribute('role')==='tooltip' && getComputedStyle(tooltip).visibility==='visible', active:document.activeElement?.id, tooltip:tooltip?.textContent};
    "#).await?;
    native_query(r#"
        document.getElementById('native-method-choice').dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true,cancelable:true}));
        return null;
    "#).await?;
    wait_native_observation("Select tooltip Escape", r#"
        const input=document.getElementById('native-method-choice');
        const tooltip=document.getElementById(input.getAttribute('aria-describedby'));
        return {ok:document.activeElement===input && getComputedStyle(tooltip).visibility==='hidden'};
    "#).await?;
    native_query("window.__ixProbeFocus(document.getElementById('native-method-set')); document.getElementById('native-method-set').click(); return null;").await?;
    wait_native_observation(
        "Select controlled method value",
        r#"
        const input=document.getElementById('native-method-choice');
        return {ok:input.value==='Second option'};
    "#,
    )
    .await?;
    if !controller
        .has_valid_value()
        .map_err(|error| error.to_string())?
        || !controller.is_touched().map_err(|error| error.to_string())?
    {
        return Err("Native Select methods did not observe controlled value and blur.".into());
    }
    native_query("document.getElementById('native-method-unmount').click(); return null;").await?;
    wait_native_observation(
        "Select method unmount",
        "return {ok:!document.getElementById('native-method-choice')};",
    )
    .await?;
    if !matches!(
        controller.has_valid_value(),
        Err(SelectMethodError::NotMounted)
    ) || !matches!(input.focus().await, Err(SelectMethodError::NotMounted))
    {
        return Err(
            "Native Select retained controller/input did not reject unmounted access.".into(),
        );
    }
    println!(
        "Native Select input/form methods, controlled state, tooltip focus/Escape, and stale handles verified."
    );
    Ok(())
}

async fn verify_native_tooltip() -> Result<(), String> {
    native_query(
        "window.__ixProbeFocus(document.getElementById('native-tooltip-first')); return null;",
    )
    .await?;
    let first = wait_native_observation("Tooltip top layer, clipping escape and bottom flip", r#"
        const tip=document.getElementById('native-tooltip-layer');
        const anchor=document.getElementById('native-tooltip-first');
        const host=document.getElementById('native-tooltip-clip');
        const t=tip.getBoundingClientRect(),a=anchor.getBoundingClientRect(),h=host.getBoundingClientRect();
        return {ok:tip.matches(':popover-open') && tip.dataset.positioned==='true' && tip.dataset.placement==='top' && t.bottom<=a.top && t.top<h.top && t.left>=0 && t.right<=innerWidth && anchor.getAttribute('aria-describedby').includes(tip.id),x:t.x,y:t.y,width:t.width,anchorX:a.x};
    "#).await?;
    native_query(
        "window.__ixProbeFocus(document.getElementById('native-tooltip-second')); return null;",
    )
    .await?;
    let second = wait_native_observation("Shared tooltip follows another anchor", r#"
        const tip=document.getElementById('native-tooltip-layer');
        const a=document.getElementById('native-tooltip-second').getBoundingClientRect(),t=tip.getBoundingClientRect();
        return {ok:tip.matches(':popover-open') && tip.dataset.positioned==='true' && Math.abs((t.left+t.width/2)-(a.left+a.width/2))<32,x:t.x};
    "#).await?;
    if first["x"] == second["x"] {
        return Err("Shared native tooltip did not move to the second anchor.".into());
    }
    native_query("window.__ixProbeFocus(document.getElementById('native-tooltip-action')); document.getElementById('native-tooltip-action').click(); return null;").await?;
    wait_native_observation("Interactive tooltip delivers Rust action", r#"
        const tip=document.getElementById('native-tooltip-layer');
        const active=document.activeElement?.id,count=document.getElementById('native-tooltip-actions').textContent;
        return {ok:tip.matches(':popover-open') && active==='native-tooltip-action' && count==='1',open:tip.matches(':popover-open'),active,count,inert:tip.getAttribute('inert')};
    "#).await?;
    // Wait beyond the source's default interactive hide delay; focus inside
    // must have canceled the anchor's pending close.
    futures_timer::Delay::new(Duration::from_millis(180)).await;
    if !native_matches("native-tooltip-layer".into(), ":popover-open").await? {
        return Err("Interactive tooltip closed while its Rust action retained focus.".into());
    }
    native_query(r#"
        document.getElementById('native-tooltip-action').dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true,cancelable:true})); return null;
    "#).await?;
    wait_native_observation(
        "Tooltip Escape",
        "return {ok:!document.getElementById('native-tooltip-layer').matches(':popover-open')};",
    )
    .await?;
    native_query(
        "window.__ixProbeFocus(document.getElementById('native-tooltip-first')); return null;",
    )
    .await?;
    wait_native_observation(
        "Tooltip reentry",
        "return {ok:document.getElementById('native-tooltip-layer').matches(':popover-open')};",
    )
    .await?;
    native_query("document.getElementById('native-tooltip-unmount').click(); return null;").await?;
    wait_native_observation(
        "Tooltip unmount",
        "return {ok:!document.getElementById('native-tooltip-layer')};",
    )
    .await?;
    println!(
        "Native tooltip escaped clipping, flipped at viewport edge, followed shared anchors, retained interactive focus/Rust clicks, dismissed and unmounted."
    );
    Ok(())
}

async fn verify_native_datetime_boundaries() -> Result<(), String> {
    for (button, disabled_twelve, disabled_eighteen) in [
        ("native-boundary-first", true, false),
        ("native-boundary-middle", false, false),
        ("native-boundary-last", false, true),
        ("native-boundary-range", false, false),
    ] {
        let click =
            native_query("document.getElementById(await dioxus.recv()).click(); return null;");
        click.send(button).map_err(|error| error.to_string())?;
        click.await?;
        let script = format!(
            r#"
            const hours=[...document.querySelectorAll('#native-boundary-picker [role="listbox"][aria-label="Hours"] [role="option"]')];
            const twelve=hours.find(node=>node.textContent.trim()==='12');
            const eighteen=hours.find(node=>node.textContent.trim()==='18');
            return {{ok:!!twelve && !!eighteen && twelve.disabled==={disabled_twelve} && eighteen.disabled==={disabled_eighteen},twelve:twelve?.disabled,eighteen:eighteen?.disabled}};
        "#
        );
        wait_native_observation(button, &script).await?;
    }
    for (value, invalid) in [
        ("2026/10/11 12:00:00", true),
        ("2026/10/16 12:00:00", false),
        ("2026/10/21 18:00:00", true),
    ] {
        let input = native_query(
            r#"
            const value=await dioxus.recv(),input=document.getElementById('native-boundary-input');
            input.value=value;input.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertText',data:value}));return null;
        "#,
        );
        input.send(value).map_err(|error| error.to_string())?;
        input.await?;
        let script = format!(
            r#"
            const input=document.getElementById('native-boundary-input');
            return {{ok:(input.getAttribute('aria-invalid')==='true')==={invalid},invalid:input.getAttribute('aria-invalid'),value:input.value}};
        "#
        );
        wait_native_observation("Typed date/time boundary validation", &script).await?;
    }
    println!(
        "Native date/time constraints restricted boundary days, allowed interior/range clocks, and matched typed-input validation."
    );
    Ok(())
}

async fn verify_native_grid_editing() -> Result<(), String> {
    native_query(r#"
        const cell=document.querySelector('#native-editable-grid [data-row-id="edit-0"] [data-column-id="asset"]');
        window.__ixProbeFocus(cell);cell.dispatchEvent(new KeyboardEvent('keydown',{key:'F2',code:'F2',bubbles:true,cancelable:true}));return null;
    "#).await?;
    wait_native_observation("Grid F2 editor focus", r#"
        const input=document.querySelector('#native-editable-grid input.ix-data-grid-editor');
        return {ok:input?.getAttribute('aria-label')==='Edit Asset for edit-0' && document.activeElement===input};
    "#).await?;
    native_query(r#"
        const input=document.querySelector('#native-editable-grid input.ix-data-grid-editor');input.value='';
        input.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'deleteContentBackward'}));
        input.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',code:'Enter',bubbles:true,cancelable:true}));return null;
    "#).await?;
    wait_native_observation("Grid invalid commit retains editor", r#"
        const input=document.querySelector('#native-editable-grid input.ix-data-grid-editor');
        return {ok:input?.getAttribute('aria-invalid')==='true' && !!document.querySelector('#native-editable-grid [role="alert"]') && document.getElementById('native-edit-count').textContent==='0'};
    "#).await?;
    native_query(r#"
        const input=document.querySelector('#native-editable-grid input.ix-data-grid-editor');input.value='Updated asset';
        input.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertText',data:'Updated asset'}));
        input.dispatchEvent(new KeyboardEvent('keydown',{key:'Tab',code:'Tab',bubbles:true,cancelable:true}));return null;
    "#).await?;
    wait_native_observation("Grid Tab commits and continues", r#"
        const input=document.querySelector('#native-editable-grid input.ix-data-grid-editor');
        return {ok:input?.getAttribute('aria-label')==='Edit Note for edit-0' && document.activeElement===input && document.getElementById('native-edit-count').textContent==='1' && document.querySelector('#native-editable-grid [data-row-id="edit-0"] [data-column-id="asset"]').textContent==='Updated asset'};
    "#).await?;
    native_query(r#"
        const input=document.querySelector('#native-editable-grid input.ix-data-grid-editor');
        input.dispatchEvent(new KeyboardEvent('keydown',{key:'Tab',code:'Tab',bubbles:true,cancelable:true}));return null;
    "#).await?;
    wait_native_observation("Grid skips read-only cells", r#"
        const input=document.querySelector('#native-editable-grid input.ix-data-grid-editor');
        return {ok:input?.getAttribute('aria-label')==='Edit Note for edit-1' && document.activeElement===input && document.getElementById('native-edit-count').textContent==='1'};
    "#).await?;
    native_query(r#"
        const input=document.querySelector('#native-editable-grid input.ix-data-grid-editor');input.value='Discard';
        input.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertText',data:'Discard'}));
        input.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',code:'Escape',bubbles:true,cancelable:true}));return null;
    "#).await?;
    wait_native_observation("Grid Escape discards", r#"
        const cell=document.querySelector('#native-editable-grid [data-row-id="edit-1"] [data-column-id="note"]');
        return {ok:!document.querySelector('#native-editable-grid input.ix-data-grid-editor') && cell.textContent==='Normal' && document.activeElement===cell && document.getElementById('native-edit-count').textContent==='1'};
    "#).await?;
    native_query(r#"
        const root=document.getElementById('native-editable-grid'),rows=[...root.querySelectorAll('[data-row-id]')];
        const row=rows.at(-1);window.__nativeEditNext=Number(row.dataset.rowId.split('-')[1])+1;
        const cell=row.querySelector('[data-column-id="note"]');window.__ixProbeFocus(cell);
        cell.dispatchEvent(new KeyboardEvent('keydown',{key:'F2',code:'F2',bubbles:true,cancelable:true}));return null;
    "#).await?;
    wait_native_observation("Grid edge editor starts", "const input=document.querySelector('#native-editable-grid input.ix-data-grid-editor');return {ok:!!input && document.activeElement===input};").await?;
    native_query(r#"
        document.querySelector('#native-editable-grid input.ix-data-grid-editor').dispatchEvent(new KeyboardEvent('keydown',{key:'Tab',code:'Tab',bubbles:true,cancelable:true}));return null;
    "#).await?;
    wait_native_observation("Grid Tab reveals previously unmounted row", r#"
        const input=document.querySelector('#native-editable-grid input.ix-data-grid-editor');
        return {ok:input?.getAttribute('aria-label')===`Edit Asset for edit-${window.__nativeEditNext}` && document.activeElement===input && document.querySelectorAll('#native-editable-grid [data-row-id]').length<40,next:window.__nativeEditNext,editor:input?.getAttribute('aria-label')};
    "#).await?;
    native_query("window.__ixProbeFocus(document.getElementById('native-edit-readonly')); document.getElementById('native-edit-readonly').click(); return null;").await?;
    wait_native_observation("Grid dynamic read-only cancels without taking focus", r#"
        return {ok:!document.querySelector('#native-editable-grid input.ix-data-grid-editor') && document.activeElement?.id==='native-edit-readonly' && document.getElementById('native-edit-count').textContent==='1'};
    "#).await?;
    println!(
        "Native grid editing verified F2, invalid blocking, controlled commit, readonly skip, Escape discard, virtualized Tab handoff and readonly focus preservation."
    );
    Ok(())
}

async fn verify_native_dropdown_form() -> Result<(), String> {
    native_query(
        "document.querySelector('#native-dropdown-form .ix-dropdown-trigger').click();return null;",
    )
    .await?;
    wait_native_observation(
        "Dropdown in a form opens",
        "return {ok:!!document.querySelector('#native-dropdown-form [role=menu]')};",
    )
    .await?;
    native_query("document.querySelector('#native-dropdown-form .ix-dropdown-backdrop').click();return null;").await?;
    wait_native_observation("Dropdown outside dismissal does not submit", r#"
        const count=document.getElementById('native-dropdown-submits').textContent;
        return {ok:!document.querySelector('#native-dropdown-form [role=menu]') && count==='0',submits:count};
    "#).await?;
    println!("Native dropdown outside dismissal preserved its containing form without submission.");
    Ok(())
}

async fn verify_native_pane() -> Result<(), String> {
    native_query("window.__ixProbeFocus(document.getElementById('native-pane-open')); document.getElementById('native-pane-open').click(); return null;").await?;
    wait_native_observation("Pane expansion and autofocus", r#"
        const pane=document.querySelector('#native-pane-probe .ix-pane');
        const bounds=pane?.getBoundingClientRect();
        return {ok:pane?.dataset.expanded==='true' && document.activeElement?.id==='native-pane-note' && Math.abs(bounds.width-320)<2, width:bounds?.width, active:document.activeElement?.id};
    "#).await?;
    native_query(
        r#"
        document.getElementById('native-pane-protect').click();
        return null;
    "#,
    )
    .await?;
    futures_timer::Delay::new(Duration::from_millis(40)).await;
    native_query(r#"
        document.getElementById('native-pane-note').dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true,cancelable:true}));
        return null;
    "#).await?;
    wait_native_observation("Pane cancelable close", r#"
        return {ok:document.querySelector('#native-pane-probe .ix-pane')?.dataset.expanded==='true' && document.getElementById('native-pane-veto').textContent==='1'};
    "#).await?;
    native_query(
        r#"
        document.getElementById('native-pane-protect').click();
        window.__ixProbeFocus(document.getElementById('native-pane-note')); return null;
    "#,
    )
    .await?;
    futures_timer::Delay::new(Duration::from_millis(40)).await;
    native_query(r#"
        document.getElementById('native-pane-note').dispatchEvent(new KeyboardEvent('keydown',{key:'Tab',bubbles:true,cancelable:true}));
        return null;
    "#).await?;
    wait_native_observation(
        "Pane Tab traversal",
        "return {ok:document.activeElement?.id==='native-pane-protect'};",
    )
    .await?;
    for selector in [
        "#native-pane-probe button[role=checkbox]",
        "#native-pane-choice",
        "#native-pane-probe .ix-pane-header button",
    ] {
        native_query(r#"
            document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'Tab',bubbles:true,cancelable:true}));return null;
        "#).await?;
        let observation = native_query(
            r#"
            const selector=await dioxus.recv();
            await new Promise(resolve=>setTimeout(resolve,40));
            return {ok:document.activeElement?.matches(selector),active:document.activeElement?.outerHTML};
        "#,
        );
        observation
            .send(selector)
            .map_err(|error| error.to_string())?;
        let measured: serde_json::Value = observation.join().await?;
        if measured["ok"] != true {
            return Err(format!(
                "Pane Tab did not skip hidden form mirrors for {selector}: {measured}"
            ));
        }
    }
    wait_native_observation("Pane Tab wrapping", r#"
        return {ok:document.activeElement===document.querySelector('#native-pane-probe .ix-pane-header button')};
    "#).await?;
    native_query(r#"
        document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'Tab',shiftKey:true,bubbles:true,cancelable:true}));return null;
    "#).await?;
    wait_native_observation(
        "Pane reverse Tab skips hidden mirrors",
        "return {ok:document.activeElement?.id==='native-pane-choice'};",
    )
    .await?;
    native_query(r#"
        document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'Tab',bubbles:true,cancelable:true}));return null;
    "#).await?;
    wait_native_observation("Pane forward Tab returns to header", "return {ok:document.activeElement===document.querySelector('#native-pane-probe .ix-pane-header button')};").await?;
    native_query(r#"
        document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true,cancelable:true}));
        return null;
    "#).await?;
    wait_native_observation("Pane Escape and focus return", r#"
        return {ok:document.querySelector('#native-pane-probe .ix-pane')?.dataset.expanded==='false' && document.activeElement?.id==='native-pane-open', active:document.activeElement?.id};
    "#).await?;
    native_query("document.getElementById('native-pane-open').click(); return null;").await?;
    wait_native_observation("Pane reopen", "return {ok:document.querySelector('#native-pane-probe .ix-pane')?.dataset.expanded==='true'};").await?;
    native_query("document.getElementById('native-pane-outside').click(); return null;").await?;
    wait_native_observation("Pane layout outside click", "return {ok:document.querySelector('#native-pane-probe .ix-pane')?.dataset.expanded==='false'};").await?;
    println!(
        "Native Pane half-parent cap, autofocus, close veto, Tab wrap, Escape focus return and content dismissal verified."
    );
    Ok(())
}

async fn verify_native_select() -> Result<(), String> {
    let inline: bool = native_query(r#"
        const root=document.getElementById('native-select-probe');
        const input=root.querySelector('input[role="combobox"]');
        if(!input || input.closest('.ix-select-popup'))return false;
        window.__ixProbeFocus(input);
        input.value='bet';
        input.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertText',data:'bet'}));
        return true;
    "#).join().await.map_err(|error| error.to_string())?;
    if !inline {
        return Err("Native Select did not mount an inline combobox input.".into());
    }
    futures_timer::Delay::new(Duration::from_millis(100)).await;
    let filtered: bool = native_query(
        r#"
        const root=document.getElementById('native-select-probe');
        const options=[...root.querySelectorAll('[role="option"]:not(.ix-select-add)')];
        return options.length===1 && options[0].textContent.trim()==='Beta';
    "#,
    )
    .join()
    .await
    .map_err(|error| error.to_string())?;
    if !filtered {
        return Err("Native inline Select did not filter to Beta.".into());
    }
    let escaped: bool = native_query(
        r#"
        const root=document.getElementById('native-select-probe');
        const panel=root.querySelector('.ix-select-popup');
        if(!panel?.matches(':popover-open'))return false;
        const a=root.getBoundingClientRect(), b=panel.getBoundingClientRect();
        return b.bottom>a.bottom || b.top<a.top;
    "#,
    )
    .join()
    .await
    .map_err(|error| error.to_string())?;
    if !escaped {
        return Err(
            "Native Select options did not escape the clipped host in the top layer.".into(),
        );
    }
    native_query(r#"
        document.querySelector('#native-select-probe input[role="combobox"]').dispatchEvent(
          new KeyboardEvent('keydown',{key:'ArrowDown',code:'ArrowDown',bubbles:true,cancelable:true}));
        return null;
    "#).await.map_err(|error| error.to_string())?;
    futures_timer::Delay::new(Duration::from_millis(100)).await;
    native_query(
        r#"
        document.querySelector('#native-select-probe input[role="combobox"]').dispatchEvent(
          new KeyboardEvent('keydown',{key:'Enter',code:'Enter',bubbles:true,cancelable:true}));
        return null;
    "#,
    )
    .await
    .map_err(|error| error.to_string())?;
    futures_timer::Delay::new(Duration::from_millis(100)).await;
    let committed: bool = native_query(
        r#"
        const root=document.getElementById('native-select-probe');
        return root.querySelector('select[name="native-selection"]').value==='beta'
          && root.querySelector('[role="status"]').textContent==='beta';
    "#,
    )
    .join()
    .await
    .map_err(|error| error.to_string())?;
    if !committed {
        return Err("Native inline Select did not commit Rust state and native form value.".into());
    }
    native_query(r#"
        const input=document.querySelector('#native-select-probe input[role="combobox"]');
        input.value='Blocked';
        input.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertText',data:'Blocked'}));
        return null;
    "#).await.map_err(|error| error.to_string())?;
    futures_timer::Delay::new(Duration::from_millis(100)).await;
    for key in ["ArrowDown", "Enter"] {
        let task = native_query(
            r#"
            const key=await dioxus.recv();
            document.querySelector('#native-select-probe input[role="combobox"]').dispatchEvent(
              new KeyboardEvent('keydown',{key,code:key,bubbles:true,cancelable:true}));
            return null;
        "#,
        );
        task.send(key).map_err(|error| error.to_string())?;
        task.await.map_err(|error| error.to_string())?;
        futures_timer::Delay::new(Duration::from_millis(100)).await;
    }
    let vetoed: bool = native_query(
        r#"
        const root=document.getElementById('native-select-probe');
        return root.querySelector('input[role="combobox"]').value==='Blocked'
          && root.querySelector('select[name="native-selection"]').value==='beta'
          && ![...root.querySelector('select').options].some(option=>option.value==='Blocked')
          && document.getElementById('native-add-count').textContent==='0';
    "#,
    )
    .join()
    .await
    .map_err(|error| error.to_string())?;
    if !vetoed {
        return Err(
            "Native Select cancellation changed input, options, selection or add notification."
                .into(),
        );
    }
    println!(
        "Native inline Select escaped a clipped host, filtered and committed Rust state and native form value."
    );
    println!(
        "Native Select veto preserved query, options, selected value and add-notification count."
    );
    Ok(())
}

async fn verify_native_grid() -> Result<(), String> {
    let bounded: bool = native_query(r#"
        const grid=document.querySelector('[role="grid"][aria-label="Native large grid"]');
        const cell=grid?.querySelector('.ix-data-grid-cell[role="gridcell"]');
        if(!cell || grid.querySelectorAll('[data-row-id]').length>=40)return false;
        window.__ixProbeFocus(cell);
        cell.dispatchEvent(new KeyboardEvent('keydown',{key:'End',code:'End',ctrlKey:true,bubbles:true,cancelable:true}));
        return true;
    "#).join().await.map_err(|error| error.to_string())?;
    if !bounded {
        return Err("Native data grid did not mount a bounded, navigable row set.".into());
    }
    wait_native_observation("Native data grid did not reveal/focus the last record and column",
        r#"
        const grid=document.querySelector('[role="grid"][aria-label="Native large grid"]');
        const cell=document.activeElement;
        const count=grid.querySelectorAll('[data-row-id]').length;
        const row=cell?.closest('[data-row-id]')?.dataset.rowId ?? null;
        const column=cell?.getAttribute('data-column-id') ?? null;
        const ids=[...grid.querySelectorAll('[data-row-id]')].map(row=>row.dataset.rowId);
        return {ok:grid.scrollTop>0 && count<40 && row==='native-grid-999' && column==='status',
          count,row,column,top:grid.scrollTop,height:grid.clientHeight,first:ids[0],last:ids.at(-1),
          activeTag:cell?.tagName ?? null,activeId:cell?.id ?? null,
          targetPresent:!!grid.querySelector('[data-row-id="native-grid-999"] [data-column-id="status"]'),
          documentFocus:document.hasFocus(),active:cell?.outerHTML?.slice(0,1000),
          target: grid.querySelector('[data-row-id="native-grid-999"] [data-column-id="status"]')?.outerHTML?.slice(0,1000)};
    "#,
    )
    .await?;
    println!(
        "Native data grid kept mounted rows bounded and navigated to its offscreen last cell."
    );
    Ok(())
}

async fn verify_native_surface() -> Result<(), String> {
    let dispatched: bool = native_query(r#"
        const svg=document.querySelector('#native-surface-probe svg');
        if(!svg)return false;
        const rect=svg.getBoundingClientRect();
        svg.dispatchEvent(new PointerEvent('pointermove',{pointerId:99,pointerType:'mouse',bubbles:true,
          clientX:rect.left+413.299*rect.width/640,clientY:rect.top+72.85*rect.height/400}));
        return true;
    "#).join().await.map_err(|error| error.to_string())?;
    if !dispatched {
        return Err("Native inspected surface did not mount.".into());
    }
    wait_native_observation("Native surface did not inspect the visible source peak",r#"
        const root=document.getElementById('native-surface-probe'), pointer=root.querySelector('.ix-surface-axis-pointer'), svg=root.querySelector('svg');
        const index=pointer?.getAttribute('data-source-index') ?? null;
        const lines=pointer?.querySelectorAll(':scope > line').length ?? 0;
        const labels=pointer?.querySelectorAll('.ix-surface-pointer-label').length ?? 0;
        const text=root.querySelector('output')?.textContent ?? null;
        const ok=index==='1260'
          && pointer.querySelectorAll(':scope > line').length===9
          && pointer.querySelectorAll('.ix-surface-pointer-label').length===3
          && text==='x: 0.50 · y: 0.50 · z: 1.000';
        return {ok,index,lines,labels,text,viewBox:svg.getAttribute('viewBox'),rect:svg.getBoundingClientRect().toJSON()};
    "#).await?;
    println!(
        "Native surface picked source vertex 1260 and rendered nine pointer segments and three labels."
    );
    Ok(())
}

async fn verify_native_zoom() -> Result<(), String> {
    native_query(r#"
        const svg=document.querySelector('#native-zoom-probe svg[id^="ix-zoom-svg-"]'), rect=svg.getBoundingClientRect();
        svg.dispatchEvent(new WheelEvent('wheel',{bubbles:true,cancelable:true,deltaY:-300,
          clientX:rect.left+rect.width*.5,clientY:rect.top+rect.height*.5}));
        return null;
    "#).await.map_err(|error| error.to_string())?;
    wait_native_observation("Native ZoomChart did not measure its SVG and zoom around the wheel position",r#"
        const root=document.getElementById('native-zoom-probe'), handles=root.querySelectorAll('[role="slider"]');
        const start=+handles[0].getAttribute('aria-valuenow'), end=+handles[1].getAttribute('aria-valuenow');
        const viewBox=root.querySelector('svg[id^="ix-zoom-svg-"]').getAttribute('viewBox');
        return {ok:viewBox==='0 0 640 600' && start>0 && start<end && end<10,viewBox,start,end};
    "#).await?;
    println!("Native ZoomChart measured its SVG and updated the viewing window from wheel input.");
    Ok(())
}

async fn verify_native_surface_motion() -> Result<(), String> {
    native_eval::<()>(
        r#"
        const svg=document.querySelector('#native-surface-probe svg');
        svg.dispatchEvent(new WheelEvent('wheel',{bubbles:true,cancelable:true,deltaY:-120}));
        return null;
    "#,
    )
    .await?;
    let first = wait_native_observation(
        "Native surface wheel did not start zoom motion",
        r#"
        const svg=document.querySelector('#native-surface-probe svg');
        const zoom=+svg.getAttribute('data-zoom');
        return {ok:zoom>1,zoom};
    "#,
    )
    .await?;
    futures_timer::Delay::new(Duration::from_millis(80)).await;
    let after: f64 = native_eval(
        "return +document.querySelector('#native-surface-probe svg').getAttribute('data-zoom');",
    )
    .await?;
    if after <= first["zoom"].as_f64().unwrap_or(f64::INFINITY) {
        return Err(format!(
            "Native surface did not continue zoom momentum: first={first}, after={after}"
        ));
    }
    native_eval::<()>(
        r#"
        document.querySelector('#native-surface-probe svg').dispatchEvent(
          new KeyboardEvent('keydown',{key:'Home',code:'Home',bubbles:true,cancelable:true}));
        return null;
    "#,
    )
    .await?;
    let reset_script = r#"
        const svg=document.querySelector('#native-surface-probe svg');
        const yaw=+svg.getAttribute('data-yaw'),pitch=+svg.getAttribute('data-pitch'),zoom=+svg.getAttribute('data-zoom');
        return {ok:yaw===40&&pitch===20&&zoom===1,yaw,pitch,zoom};
    "#;
    wait_native_observation("Native surface Home did not reset motion", reset_script).await?;
    futures_timer::Delay::new(Duration::from_millis(100)).await;
    let stable: serde_json::Value = native_eval(reset_script).await?;
    if stable["ok"] != true {
        return Err(format!("Native surface momentum survived Home: {stable}"));
    }
    println!(
        "Native surface wheel momentum continued between frames and Home canceled pending motion."
    );
    Ok(())
}

async fn verify_native_select_sizing() -> Result<(), String> {
    native_eval::<()>(
        r#"
        // One event turn exposes queued focus-return work after the user has
        // already moved on. Never replay either input operation.
        document.querySelector('#native-select-probe input[role=combobox]').dispatchEvent(
          new KeyboardEvent('keydown',{key:'Escape',code:'Escape',bubbles:true,cancelable:true}));
        const input=document.querySelector('#native-select-sizing input[role=combobox]');
        window.__ixProbeFocus(input); input.value='';
        input.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertText',data:''}));
        return null;
    "#,
    )
    .await?;
    let initial=wait_native_observation("Native RTL Select did not align to its control",r#"
        const root=document.getElementById('native-select-sizing'),panel=root.querySelector('.ix-select-popup'),control=root.querySelector('.ix-select-control');
        const p=panel?.getBoundingClientRect(),c=control?.getBoundingClientRect();
        return {ok:panel?.matches(':popover-open')&&panel.dataset.direction==='rtl'&&Math.abs(p.width-320)<1&&Math.abs(p.right-c.right)<1&&document.activeElement.id==='native-sizing-select',
          id:panel?.id,direction:panel?.dataset.direction,width:p?.width,right:p?.right,controlRight:c?.right,
          activeId:document.activeElement?.id,activeTag:document.activeElement?.tagName,
          expanded:root.querySelector('[role=combobox]')?.getAttribute('aria-expanded'),html:root.outerHTML.slice(0,1400)};
    "#).await?;
    for (query, width) in [("wide", 480.0), ("wide limit", 360.0)] {
        let task = native_query(
            r#"
            const query=await dioxus.recv(),input=document.querySelector('#native-select-sizing input[role=combobox]');
            input.value=query;
            input.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertText',data:query}));
            return null;
        "#,
        );
        task.send(query).map_err(|error| error.to_string())?;
        task.await.map_err(|error| error.to_string())?;
        let script = format!(
            r#"
            const root=document.getElementById('native-select-sizing'),panel=root.querySelector('.ix-select-popup'),control=root.querySelector('.ix-select-control');
            const p=panel.getBoundingClientRect(),c=control.getBoundingClientRect();
            return {{ok:panel.matches(':popover-open')&&Math.abs(p.width-{width})<1&&Math.abs(p.right-c.right)<1,
              id:panel.id,width:p.width,right:p.right,controlRight:c.right}};
        "#
        );
        let changed =
            wait_native_observation("Native RTL Select did not apply live sizing", &script).await?;
        if changed["id"] != initial["id"] {
            return Err("Native Select sizing remounted its open surface.".into());
        }
    }
    native_query(
        r#"
        document.querySelector('#native-select-sizing input[role=combobox]').dispatchEvent(
          new KeyboardEvent('keydown',{key:'Escape',code:'Escape',bubbles:true,cancelable:true}));
        return null;
    "#,
    )
    .await
    .map_err(|error| error.to_string())?;
    println!(
        "Native RTL Select retained right alignment and the same open popup through 320→480→360px live sizing."
    );
    Ok(())
}

async fn verify_native_select_validation() -> Result<(), String> {
    for id in ["native-required-select", "native-unvalidated-select"] {
        let focus = native_query(
            "const id=await dioxus.recv();window.__ixProbeFocus(document.getElementById(id));return null;",
        );
        focus.send(id).map_err(|error| error.to_string())?;
        focus.await?;
        let encoded = serde_json::to_string(id).map_err(|error| error.to_string())?;
        wait_native_observation(
            "Native validation Select did not receive focus before blur",
            &format!(
                r#"
                const input=document.getElementById({encoded}),active=document.activeElement;
                return {{ok:!!input&&active===input,id:{encoded},
                  active:{{tag:active?.tagName,id:active?.id,label:active?.getAttribute('aria-label')}},
                  disabled:input?.matches(':disabled'),inert:!!input?.closest('[inert]'),
                  modals:[...document.querySelectorAll(':modal')].map(node=>node.id),
                  popovers:[...document.querySelectorAll(':popover-open')].map(node=>node.id)}};
            "#
            ),
        )
        .await?;
        let blur = native_query(
            "const id=await dioxus.recv();window.__ixProbeFocus(document.getElementById(id),false);return null;",
        );
        blur.send(id).map_err(|error| error.to_string())?;
        blur.await?;
    }
    wait_native_observation("Native Select required blur did not respect form.noValidate",r#"
        const required=document.getElementById('native-required-select'),skip=document.getElementById('native-unvalidated-select');
        const validatedBlurs=+document.getElementById('native-validated-blurs').textContent,skippedBlurs=+document.getElementById('native-skipped-blurs').textContent;
        const invalid=required.getAttribute('aria-invalid'),suppressed=skip.getAttribute('aria-invalid');
        return {ok:validatedBlurs===1&&skippedBlurs===1&&invalid==='true'&&suppressed==='false'&&skip.closest('form').noValidate,
          validatedBlurs,skippedBlurs,invalid,suppressed,active:document.activeElement?.id,
          modals:[...document.querySelectorAll(':modal')].map(node=>node.id)};
    "#).await?;
    println!(
        "Native required Select marked empty selection invalid on blur and respected form.noValidate."
    );
    Ok(())
}

async fn verify_native_tree() -> Result<(), String> {
    let bounded: bool = native_query(r#"
        const tree=document.querySelector('[role="tree"][aria-label="Native large tree"]');
        const rows=tree?.querySelectorAll('[role="treeitem"]');
        if(!rows?.length || rows.length>=30)return false;
        window.__ixProbeFocus(rows[0]);
        rows[0].dispatchEvent(new KeyboardEvent('keydown',{key:'End',code:'End',bubbles:true,cancelable:true}));
        return true;
    "#).join().await.map_err(|error| error.to_string())?;
    if !bounded {
        return Err("Native tree mounted an unbounded row set or no rows.".into());
    }
    wait_native_observation("Native tree End did not reveal/focus the offscreen last row",
        r#"
        const tree=document.querySelector('[role="tree"][aria-label="Native large tree"]');
        const count=tree.querySelectorAll('[role="treeitem"]').length;
        const focus=document.activeElement?.getAttribute('data-tree-node-id');
        const ids=[...tree.querySelectorAll('[data-tree-node-id]')].map(row=>row.dataset.treeNodeId);
        return {ok:tree.scrollTop>0 && count<30 && focus==='native-device-999',
          count,focus,top:tree.scrollTop,height:tree.clientHeight,first:ids[0],last:ids.at(-1)};
    "#,
    )
    .await?;
    println!("Native tree kept mounted rows bounded and revealed/focused its offscreen last row.");
    Ok(())
}

async fn verify_native_tree_model() -> Result<(), String> {
    let initial: String = native_query(
        "return document.querySelector('[role=tree][aria-label=\"Native model tree\"]').id;",
    )
    .join()
    .await
    .map_err(|error| error.to_string())?;
    for (button, label, script) in [
        (
            "native-model-add",
            "Native model append did not render its new stable item",
            r#"
          const tree=document.querySelector('[role=tree][aria-label="Native model tree"]');
          const row=tree.querySelector('[data-tree-node-id="added"]');
          return {ok:row?.textContent.includes('Added device'),text:tree.textContent};
        "#,
        ),
        (
            "native-model-rename",
            "Native model typed payload update did not rerender",
            r#"
          const row=document.querySelector('[role=tree][aria-label="Native model tree"] [data-tree-node-id="added"]');
          return {ok:row?.textContent.includes('Renamed device'),text:row?.textContent};
        "#,
        ),
        (
            "native-model-remove",
            "Native model removal did not reconcile rendered items and callback",
            r#"
          const tree=document.querySelector('[role=tree][aria-label="Native model tree"]');
          const removed=document.getElementById('native-model-removed').textContent;
          return {ok:!tree.querySelector('[data-tree-node-id="added"]')&&tree.querySelectorAll('[role=treeitem]').length===1&&removed==='Renamed device',removed};
        "#,
        ),
        (
            "native-model-refresh",
            "Native model forced refresh did not notify",
            r#"
          const force=document.getElementById('native-model-force').textContent;
          const count=+document.getElementById('native-model-events').textContent;
          return {ok:force==='true'&&count>=4,force,count};
        "#,
        ),
    ] {
        let task = native_query(
            "const id=await dioxus.recv(); document.getElementById(id).click(); return null;",
        );
        task.send(button).map_err(|error| error.to_string())?;
        task.await.map_err(|error| error.to_string())?;
        wait_native_observation(label, script).await?;
    }
    let stable: String = native_query(
        "return document.querySelector('[role=tree][aria-label=\"Native model tree\"]').id;",
    )
    .join()
    .await
    .map_err(|error| error.to_string())?;
    if stable != initial {
        return Err("Native forced model refresh remounted the tree root.".into());
    }
    println!(
        "Native flat TreeModel append, typed rename, removal callback and forced refresh retained the tree root."
    );
    Ok(())
}

async fn verify_native_filter() -> Result<(), String> {
    // Prior native modal/popover restoration can already leave this input
    // focused with its suggestions closed. Establish a real focus transition
    // instead of assuming focus() on the current active element emits an event.
    native_query(
        r#"
        const input=document.querySelector('input[aria-label="Native top layer filter"]');
        input.__probeFocusEvents=[];
        for(const type of ['focus','blur','focusin','focusout'])input.addEventListener(type,event=>input.__probeFocusEvents.push({type:event.type,trusted:event.isTrusted,active:document.activeElement===input}));
        window.__ixProbeFocus(document.querySelector('#bridge-probe-chart [role=listbox]'));return null;
    "#,
    )
    .await?;
    wait_native_observation("Native filter outside-focus precondition", r#"
        const outside=document.querySelector('#bridge-probe-chart [role=listbox]');
        return {ok:document.activeElement===outside,active:document.activeElement?.getAttribute('aria-label')};
    "#).await?;
    let found: bool = native_query("const input=document.querySelector('input[aria-label=\"Native top layer filter\"]'); if(input)window.__ixProbeFocus(input); return !!input;").join().await.map_err(|error| error.to_string())?;
    if !found {
        return Err("Native filter input was not mounted.".into());
    }
    wait_native_observation(
        "Native category options did not escape the clipped host in the top layer",
        r#"
        const host=document.getElementById('native-filter-clip'), panel=host?.querySelector('.ix-category-filter-options');
        const input=host?.querySelector('input'), active=document.activeElement;
        const a=host?.getBoundingClientRect(), b=panel?.getBoundingClientRect();
        const opened=panel?.matches(':popover-open')===true;
        const rect=node=>node?{top:node.top,bottom:node.bottom,left:node.left,right:node.right,width:node.width,height:node.height}:null;
        return {ok:opened && !!a && !!b && (b.bottom>a.bottom || b.top<a.top),opened,
          active:{tag:active?.tagName,id:active?.id,label:active?.getAttribute('aria-label')},
          focused:active===input,expanded:input?.getAttribute('aria-expanded'),
          host:rect(a),panel:rect(b),topLayer:panel?.dataset.topLayer,placement:panel?.dataset.placement,
          focusEvents:input?.__probeFocusEvents,documentFocused:document.hasFocus(),dioxusId:input?.getAttribute('data-dioxus-id')};
    "#,
    ).await?;
    native_query(
        "window.__ixProbeFocus(document.querySelector('#bridge-probe-chart [role=listbox]')); return null;",
    )
    .await
    .map_err(|error| error.to_string())?;
    wait_native_observation(
        "Native category filter did not close after focus left its surface",
        r#"
        const panel=document.querySelector('#native-filter-clip .ix-category-filter-options'), active=document.activeElement;
        return {ok:!panel?.matches(':popover-open'),opened:panel?.matches(':popover-open'),
          active:{tag:active?.tagName,id:active?.id,label:active?.getAttribute('aria-label')},
          expanded:document.querySelector('#native-filter-clip input')?.getAttribute('aria-expanded')};
    "#,
    ).await?;
    println!("Native category filter escaped a clipped host and closed after focus left.");
    Ok(())
}

async fn verify_theme_tokens() -> Result<(), String> {
    #[derive(serde::Deserialize, Debug)]
    struct Resolved {
        background: String,
        color: String,
        radius: String,
        height: String,
        family: String,
        size: String,
    }
    let resolved: Vec<Resolved> = native_query(r#"
        return ['light','dark','compact','override','fallback'].map(id => {
          const root=document.querySelector('.theme-probe-'+id), button=root.querySelector('button');
          const style=getComputedStyle(button), parent=getComputedStyle(root), control=getComputedStyle(root.querySelector('.ix-temporal-control'));
          return {background:style.backgroundColor,color:style.color,radius:style.borderRadius,
                  height:control.minHeight,family:style.fontFamily,size:parent.fontSize};
        });
    "#).join().await.map_err(|error| error.to_string())?;
    for (index, value) in resolved.iter().enumerate() {
        let valid = match index {
            0..=2 => {
                value.background == "rgb(18, 52, 86)"
                    && value.color == "rgb(255, 255, 255)"
                    && value.radius == "7px"
                    && value.height == "44px"
                    && value.family == "monospace"
                    && value.size == "18px"
            }
            3 => {
                value.background == "rgb(101, 67, 33)"
                    && value.height == "46px"
                    && value.family == "serif"
            }
            4 => value.height == "32px",
            _ => false,
        };
        if !valid {
            return Err(format!(
                "Theme token probe {index} did not resolve supplied/fallback values: {value:?}"
            ));
        }
    }
    if resolved.len() != 5 {
        return Err("Theme token probe did not return every scope.".into());
    }
    println!(
        "Supplied theme tokens, light/dark/compact scopes, direct overrides and compact fallback verified."
    );
    Ok(())
}

#[component]
fn ThemeTokenProbes() -> Element {
    // Synthetic values exercise the adapter, not licensed corporate token data.
    let tokens = "--si-sys-color-background-accent:#123456;--si-sys-color-text-on-accent:#fff;--si-ref-typography-font-family-sans:monospace;--si-sys-sizing-size-90:44px;--si-sys-sizing-border-radius-sm:7px;--si-sys-typography-body:normal 18px/1.5 monospace";
    rsx! {
        IxTheme { class: "theme-probe-light", scheme: ColorScheme::Light, style: tokens, TokenControls {} }
        IxTheme { class: "theme-probe-dark", scheme: ColorScheme::Dark, style: tokens, TokenControls {} }
        IxTheme { class: "theme-probe-compact", scheme: ColorScheme::Dark, density: Density::Compact, style: tokens, TokenControls {} }
        IxTheme { class: "theme-probe-override", scheme: ColorScheme::Dark, style: format!("{tokens};--ix-color-primary:#654321;--ix-control-height:46px;--ix-font:serif"), TokenControls {} }
        IxTheme { class: "theme-probe-fallback", scheme: ColorScheme::Dark, density: Density::Compact, TokenControls {} }
    }
}

#[component]
fn TokenControls() -> Element {
    rsx! {
        Button { "Token probe" }
        DateInput { today: IxDate::new(2026, 10, 16).unwrap(), label: "Token date", onvaluechange: move |_| {} }
    }
}

#[component]
fn NativeSelectProbe() -> Element {
    let mut value = use_signal(String::new);
    let mut added = use_signal(|| 0usize);
    rsx! {
        div { id: "native-select-probe", style: "width:320px;height:92px;overflow:hidden",
            Select {
                id: "native-inline-select", label: "Native inline select", name: "native-selection",
                filterable: true, enable_top_layer: true, value: value(), onvaluechange: move |next| value.set(next),
                editable: true, onbeforeadd: move |event: SelectAddEvent| { if event.value == "Blocked" { event.prevent_default(); } },
                onadd: move |_| added += 1,
                options: vec![SelectOption::new("alpha", "Alpha"), SelectOption::new("beta", "Beta"), SelectOption::new("gamma", "Gamma")],
            }
            output { role: "status", "{value}" }
            output { id: "native-add-count", "{added}" }
        }
    }
}

#[component]
fn NativeDropdownFormProbe() -> Element {
    let mut submits = use_signal(|| 0usize);
    rsx! {
        form {id:"native-dropdown-form",onsubmit:move |event|{event.prevent_default();submits+=1;},
            Dropdown {label:"Native form dropdown",DropdownItem {label:"Native form action"}}
        }
        output {id:"native-dropdown-submits","{submits}"}
    }
}

#[component]
fn NativeGridEditingProbe() -> Element {
    let mut state = use_signal(DataGridState::default);
    let mut rows = use_signal(|| {
        (0..120)
            .map(|index| {
                let mut row = DataGridRow::new(format!("edit-{index}"))
                    .cell("asset", format!("Asset {index:03}"))
                    .cell("note", "Normal");
                if index == 1 {
                    row.read_only_cells.insert("asset".into());
                }
                row
            })
            .collect::<Vec<_>>()
    });
    let mut changes = use_signal(|| 0usize);
    let mut read_only = use_signal(|| false);
    let columns = use_hook(|| {
        let mut asset = DataGridColumn::new("asset", "Asset");
        asset.editable = true;
        asset.editor = DataGridTextEditor {
            required: true,
            max_length: Some(20),
        };
        let mut note = DataGridColumn::new("note", "Note");
        note.editable = true;
        note.editor.max_length = Some(8);
        vec![asset, note]
    });
    rsx! {
        div {id:"native-editable-grid",style:"width:640px",
            DataGrid {columns,rows:rows(),state:state(),onstatechange:move |next|state.set(next),height:224,read_only:read_only(),label:"Native editable grid",
                oncellvaluechanged:move |event:DataGridCellValueChanged| {if event.apply_to(&mut rows.write()) {changes+=1;}}
            }
        }
        button {id:"native-edit-readonly",r#type:"button",onclick:move |_|read_only.set(true),"Lock native grid"}
        output {id:"native-edit-count","{changes}"}
    }
}

#[component]
fn NativeDateTimeBoundaryProbe() -> Element {
    let first = IxDate::new(2026, 10, 11).unwrap();
    let middle = IxDate::new(2026, 10, 16).unwrap();
    let last = IxDate::new(2026, 10, 21).unwrap();
    let mut from = use_signal(|| middle);
    let mut range = use_signal(|| false);
    let mut value = use_signal(|| "2026/10/16 12:00:00".to_string());
    rsx! {
        div {id:"native-boundary-picker",
            button {id:"native-boundary-first",r#type:"button",onclick:move |_|from.set(first),"Native first date"}
            button {id:"native-boundary-middle",r#type:"button",onclick:move |_|from.set(middle),"Native middle date"}
            button {id:"native-boundary-last",r#type:"button",onclick:move |_|from.set(last),"Native last date"}
            button {id:"native-boundary-range",r#type:"button",onclick:move |_|{from.set(first);range.set(true);},"Native date range"}
            DateTimePicker {today:middle,from:Some(from()),to:range().then_some(last),single_selection:!range(),
                min_date:Some(first),max_date:Some(last),min_time:IxTime::new(13,10,0,0),max_time:IxTime::new(17,30,0,0),
                time:IxTime::new(12,0,0,0).unwrap(),time_format:TimeFormat::Hm,ondatechange:move |next:DateSelection|{if let Some(next)=next.from{from.set(next);}},
                ontimechange:|_|{},onselect:|_|{}
            }
        }
        DateTimeInput {id:"native-boundary-input",today:middle,label:"Native constrained date and time",value:value(),
            min_date:Some(first),max_date:Some(last),min_time:IxTime::new(13,10,0,0),max_time:IxTime::new(17,30,0,0),
            onvaluechange:move |next|value.set(next)
        }
    }
}

#[component]
fn NativeTooltipProbe() -> Element {
    let mut mounted = use_signal(|| true);
    let mut actions = use_signal(|| 0usize);
    rsx! {
        if mounted() {
            div {id:"native-tooltip-clip",style:"position:fixed;right:8px;bottom:8px;width:360px;height:40px;overflow:hidden;display:flex;gap:12px;align-items:center",
                button {id:"native-tooltip-first",class:"native-tooltip-trigger",r#type:"button","Native tooltip first"}
                button {id:"native-tooltip-second",class:"native-tooltip-trigger",r#type:"button","Native tooltip second"}
                TooltipFor {id:"native-tooltip-layer",anchor_selectors:vec![".native-tooltip-trigger".into()],placement:TooltipPlacement::Bottom,interactive:true,
                    title:"Native interactive help",p{"This panel must escape its clipped host and remain inside the viewport."}
                    button {id:"native-tooltip-action",r#type:"button",onclick:move |_|actions+=1,"Native tooltip action"}
                }
            }
        }
        button {id:"native-tooltip-unmount",r#type:"button",onclick:move |_|mounted.set(false),"Unmount native tooltip"}
        output {id:"native-tooltip-actions","{actions}"}
    }
}

#[component]
fn NativeSelectMethodsProbe(controller: SelectController) -> Element {
    let mut mounted = use_signal(|| true);
    let mut value = use_signal(String::new);
    rsx! {
        form {id:"native-method-form",name:"method-probe",novalidate:true,style:"width:320px",
            if mounted() {
                Select {id:"native-method-choice",controller,label:"Native method choice",required:true,
                    value:value(),onvaluechange:move |next|value.set(next),show_text_as_tooltip:true,
                    helper_text:"Native method helper",options:vec![SelectOption::new("one","First option"),SelectOption::new("two","Second option")]}
            }
            button {id:"native-method-set",r#type:"button",onclick:move |_|value.set("two".into()),"Set method value"}
            button {id:"native-method-unmount",r#type:"button",onclick:move |_|mounted.set(false),"Unmount method choice"}
        }
    }
}

#[component]
fn NativePaneProbe() -> Element {
    let mut expanded = use_signal(|| false);
    let mut protect = use_signal(|| false);
    let mut vetoes = use_signal(|| 0usize);
    let mut checked = use_signal(|| false);
    rsx! {
        button {id:"native-pane-open",r#type:"button",onclick:move |_|expanded.set(true),"Open native pane"}
        div {id:"native-pane-probe",style:"width:640px;height:320px",
            PaneLayout {variant:PaneVariant::Floating,
                left:rsx!{Pane {heading:"Native pane",expanded:expanded(),size:"600px",close_on_click_outside:true,
                    onbeforeexpandedchange:move |event:PaneExpandedChangeEvent| {if !event.expanded && protect(){event.prevent_default();vetoes+=1;}},
                    onexpandedchange:move |next|expanded.set(next),
                    input {id:"native-pane-note",autofocus:true,"aria-label":"Native pane note"}
                    button {id:"native-pane-protect",r#type:"button",onclick:move |_|{let next=!protect();protect.set(next);},"Protect native note"}
                    Checkbox {label:"Native pane option",checked:checked(),oncheckedchange:move |value|checked.set(value)}
                    Select {id:"native-pane-choice",label:"Native pane choice",options:vec![SelectOption::new("one","First choice"),SelectOption::new("two","Second choice")]}
                }},
                button {id:"native-pane-outside",r#type:"button","Native pane content"}
            }
        }
        output {id:"native-pane-veto","{vetoes}"}
    }
}

#[component]
fn NativeSelectSizingProbe() -> Element {
    let mut query = use_signal(String::new);
    let width = if query().contains("wide") {
        "480px"
    } else {
        "320px"
    };
    let max_width = if query().contains("limit") {
        "360px"
    } else {
        "600px"
    };
    rsx! {
        div { id:"native-select-sizing",dir:"rtl",style:"margin-left:360px;width:240px;height:92px;overflow:hidden",
            Select {id:"native-sizing-select",label:"Native RTL sizing",enable_top_layer:true,
                dropdown_width:Some(width.to_string()),dropdown_max_width:Some(max_width.to_string()),
                oninputchange:move |value|query.set(value),
                options:vec![SelectOption::new("one","wide alpha"),SelectOption::new("two","wide limit beta")]
            }
        }
    }
}

#[component]
fn NativeSelectValidationProbe() -> Element {
    let mut validated_blurs = use_signal(|| 0usize);
    let mut skipped_blurs = use_signal(|| 0usize);
    rsx! {
        div {style:"width:320px",
            form {
                Select {id:"native-required-select",label:"Native required Select",required:true,
                    options:vec![SelectOption::new("one","First option")],onixblur:move |_|validated_blurs+=1}
            }
            form {novalidate:true,
                Select {id:"native-unvalidated-select",label:"Native noValidate Select",required:true,
                    options:vec![SelectOption::new("one","First option")],onixblur:move |_|skipped_blurs+=1}
            }
            output {id:"native-validated-blurs","{validated_blurs}"}
            output {id:"native-skipped-blurs","{skipped_blurs}"}
        }
    }
}

#[component]
fn NativeGridProbe() -> Element {
    let mut state = use_signal(DataGridState::default);
    let rows = use_hook(|| {
        (0..1000)
            .map(|index| {
                DataGridRow::new(format!("native-grid-{index}"))
                    .cell("device", format!("Device {index:04}"))
                    .cell("status", "Normal")
            })
            .collect::<Vec<_>>()
    });
    rsx! {
        div { style: "width:640px",
            DataGrid { columns: vec![DataGridColumn::new("device", "Device"),DataGridColumn::new("status", "Status")],
                rows, state: state(), onstatechange: move |next| state.set(next), height: 224, label: "Native large grid" }
        }
    }
}

#[component]
fn NativeSurfaceProbe() -> Element {
    let surface = use_hook(|| {
        ChartSurface::from_equation(41, |x, y| {
            if x.abs() < 0.1 && y.abs() < 0.1 {
                None
            } else {
                Some((x * std::f64::consts::PI).sin() * (y * std::f64::consts::PI).sin())
            }
        })
    });
    rsx! {
        div { id: "native-surface-probe", style: "width:640px",
            SurfaceChart { surface, height: 400.0, show_instructions: false, label: "Native inspected surface" }
        }
    }
}

#[component]
fn NativeZoomProbe() -> Element {
    rsx! {
        div { id: "native-zoom-probe", style: "width:640px",
            ZoomChart { values: (0..128).map(|index| index as f64).collect::<Vec<_>>(),
                labels: (0..128).map(|index| format!("Sample {index}")).collect::<Vec<_>>(), label: "Native zoom probe" }
        }
    }
}

#[component]
fn NativeTreeModelProbe() -> Element {
    let controller = use_tree_model(|| {
        TreeModel::new(
            "root",
            [
                TreeModelItem::branch("root", String::from("Hidden root"), vec!["device".into()]),
                TreeModelItem::new("device", String::from("Original device")),
            ],
        )
        .unwrap()
    });
    let mut events = use_signal(|| 0usize);
    let mut force = use_signal(|| false);
    let mut removed = use_signal(String::new);
    rsx! {
        div {id:"native-model-probe",style:"width:320px",
            button {id:"native-model-add",r#type:"button",onclick:move |_|{controller.append("root",TreeModelItem::new("added",String::from("Added device"))).unwrap();},"Add model item"}
            button {id:"native-model-rename",r#type:"button",onclick:move |_|{controller.update_data("added",|data|*data="Renamed device".into()).unwrap();},"Rename model item"}
            button {id:"native-model-remove",r#type:"button",onclick:move |_|{controller.remove("added").unwrap();},"Remove model item"}
            button {id:"native-model-refresh",r#type:"button",onclick:move |_|controller.refresh(true),"Refresh model"}
            ModelTree {controller,item_label:move |data:String|data,label:"Native model tree",
                onrefresh:move |event:TreeRefreshEvent<String>|{events+=1;force.set(event.force);},
                onremove:move |items:Vec<TreeModelItem<String>>|removed.set(items.into_iter().map(|item|item.data).collect::<Vec<_>>().join(", "))
            }
            output {id:"native-model-events","{events}"}
            output {id:"native-model-force","{force}"}
            output {id:"native-model-removed","{removed}"}
        }
    }
}

#[component]
fn NativeTreeProbe() -> Element {
    let nodes = use_hook(|| {
        (0..1000)
            .map(|index| {
                TreeNode::new(
                    format!("native-device-{index}"),
                    format!("Device {index:04}"),
                )
            })
            .collect::<Vec<_>>()
    });
    rsx! {
        div { style: "width:320px",
            Tree { nodes, label: "Native large tree", max_height: 224 }
        }
    }
}

#[component]
fn Probe() -> Element {
    let modal = use_modal();
    let popover = use_popover();
    let select = use_select();
    let select_probe = select.clone();
    let desktop = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(25)).await;
            eprintln!("Native bridge probe timed out.");
            std::process::exit(1);
        });
    });
    rsx! { IxTheme { scheme: ColorScheme::Dark,
        div { id: "bridge-probe-chart", style: "width:640px", onmounted: move |_| {
            let desktop = desktop.clone(); let select = select.clone(); spawn(async move { match exercise(modal, popover, select).await {
                Ok(()) => desktop.close(),
                Err(error) => { eprintln!("Native bridge probe failed: {error}"); std::process::exit(1); }
            } });
        }, CartesianChart { categories: ["Mon","Tue","Wed","Thu","Fri","Sat","Sun"].map(str::to_string).to_vec(),
            series: vec![ChartSeries { step_end:true, ..ChartSeries::new("",vec![150.0,230.0,224.0,218.0,135.0,147.0,260.0],ChartKind::Line) }],
            legend:false, label:"Native export probe" }
        }
        Modal { modal, label: "Native controller probe", ModalHeader { "Native controller probe" } ModalContent { "Native modal" } }
        Popover { popover, label: "Native controller popover", "Native popover" }
        ThemeTokenProbes {}
        div { id: "native-filter-clip", style: "height:60px;width:320px;overflow:hidden",
            CategoryFilter { enable_top_layer: true, label: "Native top layer filter",
                categories: vec![FilterCategory::new("vendor", "Vendor", vec!["A".into()]), FilterCategory::new("product", "Product", vec!["B".into()]), FilterCategory::new("status", "Status", vec!["C".into()])] }
        }
        NativeSelectProbe {}
        NativeSelectSizingProbe {}
        NativeSelectValidationProbe {}
        NativeSelectMethodsProbe {controller:select_probe}
        NativeTooltipProbe {}
        NativeDateTimeBoundaryProbe {}
        NativeGridEditingProbe {}
        NativeDropdownFormProbe {}
        NativePaneProbe {}
        NativeTreeProbe {}
        NativeTreeModelProbe {}
        NativeGridProbe {}
        NativeSurfaceProbe {}
        NativeZoomProbe {}
        native_form_probe::Fixtures {}
        native_navigation_probe::Fixtures {}
    } }
}
