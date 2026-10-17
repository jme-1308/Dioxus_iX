//! Native checks reuse the actual gallery's menu and tab review components.
use dioxus::prelude::*;

#[path = "../gallery/menu_overlay_review.rs"]
mod menu_overlay_review;
#[path = "../gallery/tabs_review.rs"]
mod tabs_review;

#[component]
pub fn Fixtures() -> Element {
    rsx! {
        section { id: "native-navigation-review", style: "width:700px;max-width:100%",
            menu_overlay_review::MenuOverlayReview {}
            tabs_review::TabsReviewExamples {}
        }
    }
}

async fn run(script: &str) -> Result<(), String> {
    super::native_eval::<()>(script).await
}

async fn click(selector: &str) -> Result<(), String> {
    let selector = serde_json::to_string(selector).map_err(|error| error.to_string())?;
    run(&format!(
        r#"
        const target=document.querySelector({selector});
        if(!target)throw new Error('Native navigation click target missing: '+{selector});
        target.click();return null;
    "#
    ))
    .await
}

async fn named_button(scope: &str, label: &str) -> Result<(), String> {
    let args = serde_json::to_string(&(scope, label)).map_err(|error| error.to_string())?;
    run(&format!(r#"
        const [scope,label]={args};
        const target=[...document.querySelectorAll(scope+' button')].find(node=>node.textContent.trim()===label);
        if(!target)throw new Error('Native navigation named button missing: '+label);
        target.click();return null;
    "#)).await
}

async fn focus(selector: &str) -> Result<(), String> {
    let encoded = serde_json::to_string(selector).map_err(|error| error.to_string())?;
    super::wait_native_observation(
        "Native navigation focus target mounts",
        &format!(
            "const target=document.querySelector({encoded});return {{ok:!!target&&target.getClientRects().length>0,selector:{encoded}}};"
        ),
    )
    .await?;
    run(&format!(
        "window.__ixProbeFocus(document.querySelector({encoded}));return null;"
    ))
    .await?;
    super::wait_native_observation("Native navigation actual focus", &format!(r#"
        const target=document.querySelector({encoded}),active=document.activeElement;
        return {{ok:!!target&&active===target,selector:{encoded},activeTag:active?.tagName,activeLabel:active?.getAttribute('aria-label')}};
    "#)).await?;
    Ok(())
}

async fn key(key: &str) -> Result<(), String> {
    let key = serde_json::to_string(key).map_err(|error| error.to_string())?;
    run(&format!("document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{{key:{key},code:{key},bubbles:true,cancelable:true}}));return null;")).await
}

async fn menu_status(expected: &str, extra: &str) -> Result<(), String> {
    let expected = serde_json::to_string(expected).map_err(|error| error.to_string())?;
    super::wait_native_observation("Native Menu cancelable event counts and overlay state", &format!(r#"
        const root=document.getElementById('menu-overlay-review'),text=root.querySelector('#menu-overlay-review-status').textContent;
        return {{ok:text==={expected}&&({extra}),text,overlay:root.querySelector('.ix-menu-overlay-panel')?.getAttribute('aria-label'),active:document.activeElement?.getAttribute('aria-label')}};
    "#)).await?;
    Ok(())
}

pub async fn verify() -> Result<(), String> {
    verify_menu().await?;
    verify_tabs().await?;
    println!(
        "Native Menu Settings/About vetoes, accepted callbacks, close/Escape return focus and external-close focus preservation; Tabs veto, disabled skip, persistent draft, controlled hold, explicit empty selection, manual activation and indicator placement verified."
    );
    Ok(())
}

async fn verify_menu() -> Result<(), String> {
    const SETTINGS: &str = "#menu-overlay-review .ix-menu [aria-label='Settings']";
    const ABOUT: &str = "#menu-overlay-review .ix-menu [aria-label='About & legal information']";
    const CLOSE: &str = "#menu-overlay-review [aria-label='Close Settings']";
    focus(SETTINGS).await?;
    click(SETTINGS).await?;
    menu_status("Settings requests: 1; accepted: 0. About requests: 0; accepted: 0. Explicit closes: 0. Active: None.", "!root.querySelector('.ix-menu-overlay-panel')").await?;
    named_button("#menu-overlay-review", "Veto Settings action").await?;
    focus(SETTINGS).await?;
    click(SETTINGS).await?;
    menu_status("Settings requests: 2; accepted: 1. About requests: 0; accepted: 0. Explicit closes: 0. Active: Some(Settings).", "root.querySelector('.ix-menu-overlay-panel')?.getAttribute('aria-label')==='Settings'").await?;
    super::wait_native_observation("Native Menu overlay close button receives native mounted focus", r#"
        const close=document.querySelector('#menu-overlay-review [aria-label="Close Settings"]');return {ok:!!close&&document.activeElement===close,active:document.activeElement?.getAttribute('aria-label')};
    "#).await?;
    // Supply focus-family delivery only if the hidden WebView withheld it;
    // actual mounted focus was checked above before invoking this helper.
    focus(CLOSE).await?;
    click(CLOSE).await?;
    menu_status("Settings requests: 2; accepted: 1. About requests: 0; accepted: 0. Explicit closes: 1. Active: None.", "!root.querySelector('.ix-menu-overlay-panel')&&document.activeElement===root.querySelector('.ix-menu [aria-label=\"Settings\"]')").await?;
    focus(SETTINGS).await?;
    click(SETTINGS).await?;
    focus(CLOSE).await?;
    key("Escape").await?;
    menu_status("Settings requests: 3; accepted: 2. About requests: 0; accepted: 0. Explicit closes: 2. Active: None.", "!root.querySelector('.ix-menu-overlay-panel')&&document.activeElement===root.querySelector('.ix-menu [aria-label=\"Settings\"]')").await?;
    focus(SETTINGS).await?;
    click(SETTINGS).await?;
    super::wait_native_observation("Native Menu reopens before external close", "return {ok:!!document.querySelector('#menu-overlay-review [aria-label=\"Close Settings\"]')};").await?;
    focus("#menu-overlay-review .menu-review-external-close").await?;
    click("#menu-overlay-review .menu-review-external-close").await?;
    menu_status("Settings requests: 4; accepted: 3. About requests: 0; accepted: 0. Explicit closes: 2. Active: None.", "!root.querySelector('.ix-menu-overlay-panel')&&document.activeElement===root.querySelector('.menu-review-external-close')").await?;
    named_button("#menu-overlay-review", "Veto About action").await?;
    focus(ABOUT).await?;
    click(ABOUT).await?;
    menu_status("Settings requests: 4; accepted: 3. About requests: 1; accepted: 0. Explicit closes: 2. Active: None.", "!root.querySelector('.ix-menu-overlay-panel')").await?;
    named_button("#menu-overlay-review", "Veto About action").await?;
    focus(ABOUT).await?;
    click(ABOUT).await?;
    menu_status("Settings requests: 4; accepted: 3. About requests: 2; accepted: 1. Explicit closes: 2. Active: Some(About).", "root.querySelector('.ix-menu-overlay-panel')?.getAttribute('aria-label')==='About & legal information'").await?;
    focus("#menu-overlay-review [aria-label='Close About']").await?;
    key("Escape").await?;
    menu_status("Settings requests: 4; accepted: 3. About requests: 2; accepted: 1. Explicit closes: 3. Active: None.", "!root.querySelector('.ix-menu-overlay-panel')&&document.activeElement===root.querySelector('.ix-menu [aria-label=\"About & legal information\"]')").await?;
    Ok(())
}

async fn verify_tabs() -> Result<(), String> {
    const OVERVIEW: &str = "#uncontrolled-tab-review [data-tab-key='overview'] [role=tab]";
    const DIAGNOSTICS: &str = "#uncontrolled-tab-review [data-tab-key='diagnostics'] [role=tab]";
    const CONTROLLED_OVERVIEW: &str = "#controlled-tab-review [data-tab-key='overview'] [role=tab]";
    const CONTROLLED_DIAGNOSTICS: &str =
        "#controlled-tab-review [data-tab-key='diagnostics'] [role=tab]";
    super::wait_native_observation("Native TabSet initially mounts every persistent panel", r#"
        const root=document.getElementById('uncontrolled-tab-review'),panels=[...root.querySelectorAll('[role=tabpanel]')];
        return {ok:panels.length===3&&panels.filter(panel=>!panel.hidden).length===1&&root.querySelector('[data-tab-key="overview"] [role=tab]').getAttribute('aria-selected')==='true'};
    "#).await?;
    run(r#"
        const input=document.querySelector('#uncontrolled-tab-review input[aria-label="Persistent inspection note"]');
        window.__ixProbePersistentDraft=input;input.value='Native persistent draft';input.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertText',data:'Native persistent draft'}));return null;
    "#).await?;
    click(DIAGNOSTICS).await?;
    super::wait_native_observation("Native TabSet veto precedes state and accepted callback", r#"
        const root=document.getElementById('uncontrolled-tab-review'),status=document.getElementById('uncontrolled-tab-review-status').textContent;
        return {ok:status==='Requests: 1; accepted: 0; selected: overview'&&root.querySelector('[data-tab-key="overview"] [role=tab]').getAttribute('aria-selected')==='true'&&root.querySelector('[data-tab-key="diagnostics"] [role=tab]').getAttribute('aria-selected')==='false',status};
    "#).await?;
    named_button("#tabs-selection-contract", "Block diagnostics tab").await?;
    focus(OVERVIEW).await?;
    key("ArrowRight").await?;
    super::wait_native_observation("Native automatic tab navigation skips disabled and accepts selection", r#"
        const root=document.getElementById('uncontrolled-tab-review'),tab=root.querySelector('[data-tab-key="diagnostics"] [role=tab]'),panels=[...root.querySelectorAll('[role=tabpanel]')];
        const input=root.querySelector('input[aria-label="Persistent inspection note"]'),status=document.getElementById('uncontrolled-tab-review-status').textContent;
        return {ok:status==='Requests: 2; accepted: 1; selected: diagnostics'&&tab.getAttribute('aria-selected')==='true'&&document.activeElement===tab&&panels.length===3&&input===window.__ixProbePersistentDraft&&input.value==='Native persistent draft'&&input.closest('[role=tabpanel]').hidden,status,active:document.activeElement?.textContent?.slice(0,80)};
    "#).await?;
    click(OVERVIEW).await?;
    super::wait_native_observation("Native persistent tab draft survives selection round trip", r#"
        const root=document.getElementById('uncontrolled-tab-review'),input=root.querySelector('input[aria-label="Persistent inspection note"]'),status=document.getElementById('uncontrolled-tab-review-status').textContent;
        return {ok:status==='Requests: 3; accepted: 2; selected: overview'&&input===window.__ixProbePersistentDraft&&input.value==='Native persistent draft'&&!input.closest('[role=tabpanel]').hidden,status};
    "#).await?;
    click("#uncontrolled-tab-review [data-tab-key='unavailable'] [role=tab]").await?;
    super::wait_native_observation("Native disabled tab does not request selection", "return {ok:document.getElementById('uncontrolled-tab-review-status').textContent==='Requests: 3; accepted: 2; selected: overview'};").await?;
    named_button("#tabs-empty-selection", "Hold controlled selection").await?;
    click(CONTROLLED_DIAGNOSTICS).await?;
    super::wait_native_observation("Native controlled tab ignores requests while host holds selection", r#"
        const root=document.getElementById('controlled-tab-review'),status=document.getElementById('controlled-tab-review-status').textContent;
        return {ok:status==='Requests: 1; selected: overview'&&root.querySelector('[data-tab-key="overview"] [role=tab]').getAttribute('aria-selected')==='true'&&root.querySelector('[data-tab-key="diagnostics"] [role=tab]').getAttribute('aria-selected')==='false',status};
    "#).await?;
    named_button("#tabs-empty-selection", "Clear controlled selection").await?;
    super::wait_native_observation("Native explicit empty selection hides every mounted panel and keeps keyboard entry", r#"
        const root=document.getElementById('controlled-tab-review'),panels=[...root.querySelectorAll('[role=tabpanel]')],tabs=[...root.querySelectorAll('[role=tab]')],status=document.getElementById('controlled-tab-review-status').textContent;
        return {ok:status==='Requests: 1; selected: none'&&panels.length===3&&panels.every(panel=>panel.hidden)&&tabs.every(tab=>tab.getAttribute('aria-selected')==='false')&&tabs.filter(tab=>tab.tabIndex===0).length===1&&tabs[0].tabIndex===0,status};
    "#).await?;
    focus(CONTROLLED_OVERVIEW).await?;
    key("ArrowRight").await?;
    super::wait_native_observation("Native empty controlled selection accepts keyboard requests without forcing state", r#"
        const root=document.getElementById('controlled-tab-review'),tab=root.querySelector('[data-tab-key="diagnostics"] [role=tab]'),status=document.getElementById('controlled-tab-review-status').textContent;
        return {ok:status==='Requests: 2; selected: none'&&document.activeElement===tab&&[...root.querySelectorAll('[role=tabpanel]')].every(panel=>panel.hidden),status};
    "#).await?;
    named_button("#tabs-empty-selection", "Hold controlled selection").await?;
    named_button("#tabs-empty-selection", "Manual tab activation").await?;
    named_button("#tabs-empty-selection", "Restore overview selection").await?;
    focus(CONTROLLED_OVERVIEW).await?;
    key("ArrowRight").await?;
    super::wait_native_observation("Native manual arrows move focus without requesting selection", r#"
        const root=document.getElementById('controlled-tab-review'),tab=root.querySelector('[data-tab-key="diagnostics"] [role=tab]'),status=document.getElementById('controlled-tab-review-status').textContent;
        return {ok:status==='Requests: 2; selected: overview'&&document.activeElement===tab&&root.querySelector('[data-tab-key="overview"] [role=tab]').getAttribute('aria-selected')==='true',status};
    "#).await?;
    // Synthetic keydown does not perform a browser's trusted Enter default
    // action. Exercise the native button activation separately and label it.
    click(CONTROLLED_DIAGNOSTICS).await?;
    super::wait_native_observation("Native manual activation publishes controlled selection", r#"
        const root=document.getElementById('controlled-tab-review'),status=document.getElementById('controlled-tab-review-status').textContent;
        return {ok:status==='Requests: 3; selected: diagnostics'&&root.querySelector('[data-tab-key="diagnostics"] [role=tab]').getAttribute('aria-selected')==='true',status};
    "#).await?;
    super::wait_native_observation("Native top tab indicator occupies top border", r#"
        const root=document.getElementById('controlled-tab-review'),strip=root.querySelector('[role=tablist]'),selected=root.querySelector('[data-selected=true]'),style=getComputedStyle(selected);
        return {ok:strip.dataset.placement==='top'&&style.borderTopWidth==='2px'&&style.borderBottomWidth==='0px',placement:strip.dataset.placement,top:style.borderTopWidth,bottom:style.borderBottomWidth};
    "#).await?;
    named_button("#tabs-empty-selection", "Top indicator").await?;
    super::wait_native_observation("Native bottom tab indicator occupies bottom border", r#"
        const root=document.getElementById('controlled-tab-review'),strip=root.querySelector('[role=tablist]'),selected=root.querySelector('[data-selected=true]'),style=getComputedStyle(selected);
        return {ok:strip.dataset.placement==='bottom'&&style.borderTopWidth==='0px'&&style.borderBottomWidth==='2px',placement:strip.dataset.placement,top:style.borderTopWidth,bottom:style.borderBottomWidth};
    "#).await?;
    Ok(())
}
