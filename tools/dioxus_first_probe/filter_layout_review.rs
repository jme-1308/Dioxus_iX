//! Excluded CategoryFilter mounted popup layout, viewport flip/clamp and token-scroll workflows.
use dioxus::prelude::*;
use dioxus_ix::*;

fn sample_categories() -> Vec<FilterCategory> {
    vec![
        FilterCategory::new(
            "pump",
            "Pump",
            vec![
                "P-101 Primary".into(),
                "P-102 Booster".into(),
                "P-204 Standby".into(),
            ],
        ),
        FilterCategory::new(
            "status",
            "Status",
            vec!["Running".into(), "Maintenance".into(), "Standby".into()],
        ),
    ]
}

fn sample_suggestions() -> Vec<String> {
    vec![
        "High vibration".into(),
        "Seal leakage".into(),
        "Cavitation alert".into(),
        "Bearing temperature".into(),
    ]
}

fn initial_filter_state() -> FilterState {
    FilterState {
        tokens: vec!["Line-A".into()],
        categories: vec![CategoryToken {
            id: "pump".into(),
            value: "P-101 Primary".into(),
            operator: LogicalFilterOperator::Equal,
        }],
    }
}

fn overflow_filter_state() -> FilterState {
    let tokens = (1..=8)
        .map(|index| format!("Manifold-Telemetry-Channel-{index:02}-High-Pressure"))
        .collect();
    let categories = (1..=8)
        .map(|index| CategoryToken {
            id: if index % 2 == 1 {
                "pump".into()
            } else {
                "status".into()
            },
            value: format!("Loop-{index:02}-Auxiliary-Coolant- Circuit"),
            operator: if index % 2 == 1 {
                LogicalFilterOperator::Equal
            } else {
                LogicalFilterOperator::NotEqual
            },
        })
        .collect();
    FilterState { tokens, categories }
}

fn controller_error_label(result: Result<(), CategoryFilterError>) -> String {
    match result {
        Ok(()) => "Ok".into(),
        Err(CategoryFilterError::NotMounted) => "NotMounted".into(),
        Err(CategoryFilterError::MultipleMounted) => "MultipleMounted".into(),
    }
}

#[component]
pub fn FilterLayoutReview() -> Element {
    let controller = use_category_filter();
    let populate_controller = controller.clone();
    let clear_controller = controller.clone();
    let mut mounted = use_signal(|| true);
    let mut epoch = use_signal(|| 0_u64);
    let mut plain_mounted = use_signal(|| true);
    let mut plain_epoch = use_signal(|| 0_u64);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let mut window_wide = use_signal(|| false);
    let mut narrow = use_signal(|| false);
    let mut near_bottom = use_signal(|| false);
    let mut scrolled = use_signal(|| false);
    let mut scroll_mount = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let mut defer_ack = use_signal(|| false);
    let mut state = use_signal(initial_filter_state);
    let mut pending_state = use_signal(|| None::<FilterState>);
    let mut plain_state = use_signal(FilterState::default);
    let mut draft_query = use_signal(String::new);
    let mut active_category = use_signal(|| None::<String>);
    let mut changes = use_signal(|| 0_usize);
    let mut clears = use_signal(|| 0_usize);
    let mut acks = use_signal(|| 0_usize);
    let mut plain_changes = use_signal(|| 0_usize);
    let mut controller_status = use_signal(|| "Ready".to_string());
    let mut native_errors = use_signal(Vec::<String>::new);

    let toggle_scroll = use_callback(move |()| {
        scrolled.toggle();
        if let Some(node) = scroll_mount() {
            let offset = if scrolled() { 28.0 } else { 0.0 };
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

    let committed = state();
    let token_count = committed.tokens.len();
    let category_count = committed.categories.len();
    let chip_count = token_count + category_count;
    let pending_count = pending_state()
        .as_ref()
        .map(|next| next.tokens.len() + next.categories.len())
        .unwrap_or(0);
    let plain_count = plain_state().tokens.len() + plain_state().categories.len();
    let category_text = active_category().unwrap_or_else(|| "none".into());
    let last_error = native_errors()
        .last()
        .cloned()
        .unwrap_or_else(|| "none".into());
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();

    rsx! {
        style { {STYLES} }
        IxTheme { class: "r0235-filter-review",
            h1 { "CategoryFilter mounted layout and token scrolling" }
            p {
                "Inspect native top-layer popup anchoring inside a clipped scroll container, narrow anchor resizing (F2), clipped host scrolling (F3), near-bottom flip placement (F4), and controller token-list scrolling."
            }
            div {
                onkeydown: move |event: KeyboardEvent| match event.key() {
                    Key::F2 => {
                        event.prevent_default();
                        narrow.toggle();
                    }
                    Key::F3 => {
                        event.prevent_default();
                        toggle_scroll.call(());
                    }
                    Key::F4 => {
                        event.prevent_default();
                        near_bottom.toggle();
                    }
                    Key::F5 => {
                        event.prevent_default();
                        plain_mounted.toggle();
                    }
                    Key::F6 => {
                        event.prevent_default();
                        plain_epoch += 1;
                        plain_mounted.set(true);
                    }
                    Key::F9 => {
                        #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                        {
                            event.prevent_default();
                            window_wide.toggle();
                            dioxus::desktop::window().set_inner_size(dioxus::desktop::LogicalSize::new(
                                if window_wide() { 1120.0 } else { 1000.0 }, 850.0,
                            ));
                        }
                    }
                    Key::F7 => {
                        event.prevent_default();
                        mounted.set(false);
                    }
                    Key::F8 => {
                        event.prevent_default();
                        mounted.set(true);
                    }
                    _ => {}
                },
                div { class: "r0235-controls",
                    Button {
                        id: "r0235-populate-16",
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| {
                            let status = controller_error_label(
                                populate_controller.set_filter_state(overflow_filter_state()),
                            );
                            controller_status.set(status);
                        },
                        "Populate 16 long chips"
                    }
                    Button {
                        id: "r0235-remove-last",
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| {
                            let mut next = state();
                            if next.remove_last() {
                                state.set(next);
                                changes += 1;
                            }
                        },
                        "Remove last chip"
                    }
                    Button {
                        id: "r0235-clear",
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| {
                            let status = controller_error_label(
                                clear_controller.set_filter_state(FilterState::default()),
                            );
                            controller_status.set(status);
                        },
                        "Clear via controller"
                    }
                    Button {
                        id: "r0235-toggle-defer",
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| defer_ack.toggle(),
                        if defer_ack() { "Immediate onchange" } else { "Defer onchange ack" }
                    }
                    Button {
                        id: "r0235-ack-pending",
                        variant: ButtonVariant::Secondary,
                        disabled: pending_state().is_none(),
                        onclick: move |_| {
                            if let Some(next) = pending_state.take() {
                                state.set(next);
                                acks += 1;
                            }
                        },
                        "Acknowledge pending ({pending_count})"
                    }
                    Button {
                        id: "r0235-toggle-narrow",
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| narrow.toggle(),
                        if narrow() { "Wide anchor (F2)" } else { "Narrow anchor (F2)" }
                    }
                    Button {
                        id: "r0235-scroll-anchor",
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| toggle_scroll.call(()),
                        if scrolled() { "Reset host scroll (F3)" } else { "Scroll clipped host (F3)" }
                    }
                    Button {
                        id: "r0235-toggle-bottom",
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| near_bottom.toggle(),
                        if near_bottom() { "Restore top fixture (F4)" } else { "Move near bottom (F4)" }
                    }
                    Button {
                        id: "r0235-toggle-mount",
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| mounted.toggle(),
                        if mounted() { "Hide filter (F7)" } else { "Show filter (F8)" }
                    }
                    Button {
                        id: "r0235-remount",
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| epoch.set(epoch() + 1),
                        "Remount filter"
                    }
                    Button {
                        id: "r0252-plain-visible",
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| plain_mounted.toggle(),
                        if plain_mounted() { "Hide plain filter (F5)" } else { "Restore plain filter (F5)" }
                    }
                    Button {
                        id: "r0252-plain-remount",
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| { plain_epoch += 1; plain_mounted.set(true); },
                        "Remount plain filter (F6)"
                    }
                    Button {
                        id: "r0235-outside",
                        variant: ButtonVariant::Secondary,
                        "Outside focus target"
                    }
                }
                section {
                    id: "r0235-stage",
                    class: if near_bottom() { "r0235-stage r0235-near-bottom" } else { "r0235-stage" },
                    "data-near-bottom": near_bottom().to_string(),
                    "data-narrow": narrow().to_string(),
                    "data-scrolled": scrolled().to_string(),
                    h2 { "Categorized equipment filter (clipped scroll host)" }
                    div {
                        id: "r0235-scroll-box",
                        class: "r0235-scroll-box",
                        tabindex: 0,
                        onmounted: move |event| scroll_mount.set(Some(event.data())),
                        div {
                            id: "r0235-scroll-inner",
                            class: "r0235-scroll-inner",
                            div {
                                id: "r0235-anchor-frame",
                                class: if narrow() { "r0235-anchor-frame r0235-narrow" } else { "r0235-anchor-frame" },
                                if mounted() {
                                    div {
                                        id: "r0235-categorized-host",
                                        key: "categorized-{epoch()}",
                                        CategoryFilter {
                                            categories: Some(sample_categories()),
                                            state: Some(state()),
                                            controller: Some(controller),
                                            enable_top_layer: true,
                                            placeholder: "Filter pumps and status".to_string(),
                                            label: "Categorized equipment filter".to_string(),
                                            onchange: move |next: FilterState| {
                                                changes += 1;
                                                if defer_ack() {
                                                    pending_state.set(Some(next));
                                                } else {
                                                    pending_state.set(None);
                                                    state.set(next);
                                                }
                                            },
                                            oninput: move |input: FilterInputState| {
                                                draft_query.set(input.token);
                                                active_category.set(input.category);
                                            },
                                            oncategory: move |category: Option<String>| {
                                                active_category.set(category);
                                            },
                                            onclear: move |()| {
                                                clears += 1;
                                            },
                                            onnativeerror: move |error: NativeSurfaceError| {
                                                native_errors.write().push(error.to_string());
                                            },
                                        }
                                    }
                                } else {
                                    p { id: "r0235-unmounted", "Categorized filter hidden." }
                                }
                            }
                            p { class: "r0235-clip-note",
                                "Clipped overflow container: scrolling moves the filter anchor while the top-layer popup escapes clipping."
                            }
                            div { class: "r0235-scroll-rail", "aria-hidden": "true" }
                        }
                    }
                }
                section { id: "r0235-plain-section", class: "r0235-stage",
                    h2 { "Plain suggestion filter" }
                    p { "Open both filters with ArrowDown to keep both popups available. F5 hides or restores this filter; F6 remounts it while the categorized filter remains open. On desktop, F9 resizes the window." }
                    if mounted() && plain_mounted() {
                        div {
                            id: "r0235-plain-host",
                            key: "plain-{epoch()}-{plain_epoch()}",
                            CategoryFilter {
                                suggestions: sample_suggestions(),
                                state: Some(plain_state()),
                                enable_top_layer: true,
                                placeholder: "Filter by plain suggestion".to_string(),
                                label: "Plain suggestion filter".to_string(),
                                onchange: move |next: FilterState| {
                                    plain_changes += 1;
                                    plain_state.set(next);
                                },
                                onnativeerror: move |error: NativeSurfaceError| {
                                    native_errors.write().push(error.to_string());
                                },
                            }
                        }
                    }
                }
                output {
                    id: "r0235-status",
                    role: "status",
                    "aria-live": "polite",
                    "data-mounted": mounted().to_string(),
                    "data-epoch": epoch().to_string(),
                    "data-plain-mounted": plain_mounted().to_string(),
                    "data-plain-epoch": plain_epoch().to_string(),
                    "data-narrow": narrow().to_string(),
                    "data-near-bottom": near_bottom().to_string(),
                    "data-scrolled": scrolled().to_string(),
                    "data-defer-ack": defer_ack().to_string(),
                    "data-pending": pending_state().is_some().to_string(),
                    "data-pending-count": pending_count.to_string(),
                    "data-token-count": token_count.to_string(),
                    "data-category-count": category_count.to_string(),
                    "data-chip-count": chip_count.to_string(),
                    "data-changes": changes().to_string(),
                    "data-clears": clears().to_string(),
                    "data-acks": acks().to_string(),
                    "data-active-category": category_text.clone(),
                    "data-draft": draft_query(),
                    "data-controller-status": controller_status(),
                    "data-plain-count": plain_count.to_string(),
                    "data-plain-changes": plain_changes().to_string(),
                    "data-native-errors": native_errors().len().to_string(),
                    "data-last-native-error": last_error.clone(),
                    span {
                        "Chips: {chip_count} (tokens: {token_count}, categories: {category_count}); changes: {changes()}; clears: {clears()}; acks: {acks()}"
                    }
                    span {
                        "Category: {category_text}; draft: {draft_query()}; controller: {controller_status()}; pending: {pending_count}"
                    }
                    span {
                        "Plain chips: {plain_count} (changes: {plain_changes()}); native errors: {native_errors().len()} ({last_error})"
                    }
                }
            }
            div {
                onmounted: move |_| {
                    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                    {
                        let window = window.clone();
                        spawn(async move {
                            let query = super::dom_query::eval(NATIVE_FILTER_LAYOUT_CHECK);
                            let _ = query.send("filter-layout-review");
                            match query.join::<serde_json::Value>().await {
                                Ok(value) if value["ok"] == true => {
                                    println!("Dioxus category filter layout native: {value}");
                                    window.close();
                                }
                                other => {
                                    eprintln!("Dioxus category filter layout native failed: {other:?}");
                                    std::process::exit(1);
                                }
                            }
                        });
                        spawn(async move {
                            futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                            eprintln!("Dioxus category filter layout native timed out");
                            std::process::exit(1);
                        });
                    }
                },
            }
        }
    }
}

#[allow(dead_code)]
#[component]
pub fn FilterLayoutPreview() -> Element {
    rsx! { FilterLayoutReview {} }
}

const STYLES: &str = "body{margin:0}.r0235-filter-review{min-height:100vh;box-sizing:border-box;padding:24px}.r0235-filter-review>h1{font-size:28px;margin:0 0 12px}.r0235-filter-review p{max-width:70ch;line-height:1.5;margin:0 0 16px}.r0235-controls{display:flex;flex-wrap:wrap;align-items:center;gap:10px;max-width:940px;margin:16px 0}.r0235-stage{max-width:940px;margin:20px 0;padding:16px;border:1px solid var(--ix-color-border);background:var(--ix-color-surface-1)}.r0235-stage>h2{font-size:20px;margin:0 0 12px}.r0235-scroll-box{height:156px;overflow:auto;padding:12px;border:1px solid var(--ix-color-border);background:var(--ix-color-surface)}.r0235-scroll-inner{width:100%;min-height:340px;padding:36px 8px 180px;box-sizing:border-box}.r0235-anchor-frame{width:100%;max-width:680px}.r0235-anchor-frame.r0235-narrow{width:240px;max-width:100%}.r0235-clip-note{margin-top:96px;color:var(--ix-color-text-secondary)}.r0235-scroll-rail{width:calc(100% + 180px);height:1px}.r0235-stage.r0235-near-bottom{position:fixed;left:16px;right:16px;bottom:8px;z-index:10;margin:0;padding:12px;box-shadow:0 -4px 16px #0003}.r0235-stage.r0235-near-bottom .r0235-scroll-box{height:112px;padding:8px}.r0235-stage.r0235-near-bottom .r0235-scroll-inner{padding:8px 4px 120px}.r0235-stage.r0235-near-bottom .r0235-anchor-frame:not(.r0235-narrow){max-width:620px}.r0235-filter-review output{display:block;max-width:75ch;margin-top:16px;line-height:1.6;font-variant-numeric:tabular-nums;overflow-wrap:anywhere}.r0235-filter-review output span{display:block}@media(max-width:600px){.r0235-filter-review{padding:16px}.r0235-stage{padding:12px}}";

// Synthetic Linux WebView driver: dispatches synthetic click/scroll/keyboard
// events while Dioxus owns mounted popup geometry, top-layer lifecycle and
// token-list scrolling. Actual pointer and typing checks run in Chromium.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_FILTER_LAYOUT_CHECK: &str = r#"
await dioxus.recv();
const byId = id => document.getElementById(id);
const status = () => byId('r0235-status');
const catHost = () => byId('r0235-categorized-host');
const catRoot = () => catHost()?.querySelector('.ix-category-filter');
const catAnchor = () => catHost()?.querySelector('.ix-category-filter-input');
const catTokens = () => catHost()?.querySelector('.ix-category-filter-tokens');
const catInput = () => catHost()?.querySelector('.ix-category-filter-text');
const catPanel = () => catHost()?.querySelector('.ix-category-filter-options');
const catReset = () => catHost()?.querySelector('.ix-category-filter-reset');
const plainHost = () => byId('r0235-plain-host');
const plainInput = () => plainHost()?.querySelector('.ix-category-filter-text');
const plainPanel = () => plainHost()?.querySelector('.ix-category-filter-options');
const key = (node, value, shiftKey = false) => {
    const event = new KeyboardEvent('keydown', { key: value, code: value, shiftKey, bubbles: true, cancelable: true });
    node.dispatchEvent(event);
    return event;
};
const wait = async (label, predicate) => {
    for (let n = 0; n < 220; n++) {
        if (predicate()) return;
        await new Promise(resolve => setTimeout(resolve, 20));
    }
    const pRect = catPanel()?.getBoundingClientRect();
    const aRect = catAnchor()?.getBoundingClientRect();
    throw new Error(
        label +
        ' chips=' + status()?.dataset.chipCount +
        ' placement=' + catPanel()?.dataset.placement +
        ' pRect=' + JSON.stringify(pRect) +
        ' aRect=' + JSON.stringify(aRect) +
        ' scrollTop=' + catTokens()?.scrollTop + '/' + catTokens()?.scrollHeight +
        ' errors=' + status()?.dataset.nativeErrors
    );
};

await wait('initial filters mounted', () => catInput() && plainInput() && status()?.dataset.chipCount === '2');

// 1. Open native top-layer surface inside clipped scroll container and verify bottom placement & clipping escape.
const scrollBox = byId('r0235-scroll-box');
scrollBox.scrollTop = 0;
scrollBox.scrollLeft = 0;
catInput().click();
await wait('top-layer popup open and placed below anchor', () => {
    const panel = catPanel(), anchor = catAnchor();
    if (!panel || !anchor || panel.dataset.topLayer !== 'true' || !panel.matches(':popover-open') || getComputedStyle(panel).visibility !== 'visible') return false;
    const p = panel.getBoundingClientRect(), a = anchor.getBoundingClientRect(), s = scrollBox.getBoundingClientRect();
    return p.width > 280 && Math.abs(p.width - a.width) <= 2 && Math.abs(p.top - (a.bottom + 2)) <= 6 && p.bottom > s.bottom && (!panel.dataset.placement || panel.dataset.placement.startsWith('bottom'));
});
// Actual mounted option focus and both popup boundary Tab directions.
catInput().focus();key(catInput(), 'ArrowDown');
const options = () => [...catPanel().querySelectorAll('.ix-category-filter-option')];
await wait('mounted first option focus', () => document.activeElement === options()[0]);
key(options()[0], 'Tab', true);
await wait('mounted reverse Tab wraps to last option', () => document.activeElement === options().at(-1));
key(options().at(-1), 'Tab');
await wait('mounted forward Tab wraps to first option', () => document.activeElement === options()[0]);
catInput().focus();
const initialAnchorTop = catAnchor().getBoundingClientRect().top;
const initialPanelTop = catPanel().getBoundingClientRect().top;
const wideWidth = catPanel().getBoundingClientRect().width;

// 2. Parent vertical scroll updates anchored popup position, and horizontal scroll clamps popup to viewport edge.
key(catInput(), 'F3');
await wait('parent vertical scroll updates popup top', () => {
    const p = catPanel()?.getBoundingClientRect(), a = catAnchor()?.getBoundingClientRect();
    return status()?.dataset.scrolled === 'true' && p && a && a.top < initialAnchorTop - 16 && p.top < initialPanelTop - 16 && Math.abs(p.top - (a.bottom + 2)) <= 6;
});
scrollBox.scrollLeft = 96;
scrollBox.dispatchEvent(new Event('scroll', { bubbles: true }));
await wait('horizontal scroll clamps popup left inside viewport', () => {
    const vw = window.visualViewport?.width ?? document.documentElement.clientWidth;
    const p = catPanel()?.getBoundingClientRect(), a = catAnchor()?.getBoundingClientRect();
    return p && a && a.left < 4 && p.left >= 7 && p.left > a.left && p.right <= vw - 7;
});
scrollBox.scrollLeft = 0;
key(catInput(), 'F3');
await wait('parent scroll restored', () => {
    const p = catPanel()?.getBoundingClientRect(), a = catAnchor()?.getBoundingClientRect();
    return status()?.dataset.scrolled === 'false' && p && a && Math.abs(p.top - (a.bottom + 2)) <= 6 && Math.abs(p.left - a.left) <= 2;
});

// 3. Narrow anchor resize updates popup width while open.
key(catInput(), 'F2');
await wait('narrow anchor resize shrinks popup width', () => {
    const p = catPanel()?.getBoundingClientRect(), a = catAnchor()?.getBoundingClientRect();
    return status()?.dataset.narrow === 'true' && p && a && a.width <= 244 && Math.abs(p.width - a.width) <= 2 && p.width < wideWidth - 100;
});
const narrowWidth = catPanel().getBoundingClientRect().width;
key(catInput(), 'F2');
await wait('wide anchor restored', () => {
    const p = catPanel()?.getBoundingClientRect(), a = catAnchor()?.getBoundingClientRect();
    return status()?.dataset.narrow === 'false' && p && a && p.width > narrowWidth + 100 && Math.abs(p.width - a.width) <= 2;
});

// 4. Near-bottom fixture flips popup to top placement within viewport bounds.
key(catInput(), 'F4');
scrollBox.dispatchEvent(new Event('scroll', { bubbles: true }));
await wait('near-bottom fixture flips popup above anchor', () => {
    const panel = catPanel(), anchor = catAnchor();
    if (!panel || !anchor || status()?.dataset.nearBottom !== 'true') return false;
    const vw = window.visualViewport?.width ?? document.documentElement.clientWidth;
    const p = panel.getBoundingClientRect(), a = anchor.getBoundingClientRect();
    return p.top >= 7 && p.left >= 7 && p.right <= vw - 7 && p.bottom <= a.top && Math.abs(a.top - p.bottom - 2) <= 6 && (!panel.dataset.placement || panel.dataset.placement.startsWith('top'));
});
key(catInput(), 'F4');
scrollBox.dispatchEvent(new Event('scroll', { bubbles: true }));
await wait('top fixture restored to bottom placement', () => {
    const panel = catPanel(), anchor = catAnchor();
    if (!panel || !anchor || status()?.dataset.nearBottom !== 'false') return false;
    const p = panel.getBoundingClientRect(), a = anchor.getBoundingClientRect();
    return p.top >= a.bottom && Math.abs(p.top - (a.bottom + 2)) <= 6 && (!panel.dataset.placement || panel.dataset.placement.startsWith('bottom'));
});

// 5. Category and value selection + plain suggestion selection.
const statusCategory = () => catPanel()?.querySelector('.ix-category-filter-option[data-id="status"]');
await wait('status category option available', () => Boolean(statusCategory()));
statusCategory().click();
await wait('status category preview selected', () => status()?.dataset.activeCategory === 'status' && Boolean(catHost()?.querySelector('.ix-category-filter-preview')));
const runningValue = () => catPanel()?.querySelector('.ix-category-filter-option[data-id="Running"]');
await wait('Running value option available', () => Boolean(runningValue()));
runningValue().click();
await wait('Running category token committed', () => status()?.dataset.chipCount === '3' && status()?.dataset.categoryCount === '2');

plainInput().click();
await wait('plain suggestion popup open', () => plainPanel()?.dataset.topLayer === 'true' && plainPanel()?.matches(':popover-open'));
const sealOption = () => plainPanel()?.querySelector('.ix-category-filter-option[data-id="Seal leakage"]');
await wait('plain suggestion option present', () => Boolean(sealOption()));
sealOption().click();
await wait('plain suggestion committed', () => status()?.dataset.plainCount === '1' && status()?.dataset.plainChanges === '1');
byId('r0235-outside').click();
await wait('outside click closes popups', () => !catPanel() && !plainPanel());

// Shared viewport lifetime: two live consumers, then one closes/retires while the other measures.
const popupReady = panel => panel?.matches(':popover-open') && getComputedStyle(panel).visibility === 'visible';
const categoryAligned = () => {
    if (!popupReady(catPanel())) return false;
    const p = catPanel().getBoundingClientRect(), a = catAnchor().getBoundingClientRect();
    return Math.abs(p.width - a.width) <= 2 && Math.abs(p.left - a.left) <= 2 && Math.abs(p.top - a.bottom - 2) <= 6;
};
key(catInput(), 'ArrowDown');
await wait('categorized popup opens for shared viewport', categoryAligned);
key(plainInput(), 'ArrowDown');
await wait('two native geometry consumers remain live', () => categoryAligned() && popupReady(plainPanel()));
key(plainInput(), 'Escape');
await wait('closing one consumer leaves the other open', () => !plainPanel() && categoryAligned());
key(plainInput(), 'ArrowDown');
await wait('peer popup reopens', () => popupReady(plainPanel()) && categoryAligned());
catInput().focus({preventScroll:true});
key(catInput(), 'F5');
await wait('independent peer removal preserves categorized popup', () => status()?.dataset.plainMounted === 'false' && !plainHost() && categoryAligned());
const sharedAnchorTop = catAnchor().getBoundingClientRect().top;
key(catInput(), 'F3');
await wait('remaining consumer follows real ancestor scroll', () => categoryAligned() && catAnchor().getBoundingClientRect().top < sharedAnchorTop - 16);
key(catInput(), 'F3');
await wait('remaining consumer restores ancestor alignment', () => status()?.dataset.scrolled === 'false' && categoryAligned());
key(catInput(), 'F9');
await wait('remaining consumer follows Dioxus window resize', () => Math.abs(innerWidth - 1120) <= 2 && categoryAligned());
key(catInput(), 'F5');
await wait('peer restored alongside live consumer', () => status()?.dataset.plainMounted === 'true' && plainInput() && categoryAligned());
key(plainInput(), 'ArrowDown');
await wait('restored peer receives current shared viewport', () => popupReady(plainPanel()) && categoryAligned());
const retiredPeer = plainHost(), retiredPanel = plainPanel();
key(catInput(), 'F6');
await wait('keyed peer retirement keeps other consumer live', () => status()?.dataset.plainEpoch === '1' && plainHost() !== retiredPeer && !retiredPeer.isConnected && !retiredPanel.isConnected && categoryAligned());
key(plainInput(), 'ArrowDown');
await wait('remounted peer opens with current shared facts', () => popupReady(plainPanel()) && categoryAligned());
key(catInput(), 'F9');
await wait('shared consumers follow restored window width', () => Math.abs(innerWidth - 1000) <= 2 && categoryAligned() && popupReady(plainPanel()));
byId('r0235-outside').click();
await wait('shared workflow outside close', () => !catPanel() && !plainPanel());

// 6. Deferred acknowledgement and source-like controller command scrolling token list to end.
byId('r0235-toggle-defer').click();
await wait('defer mode enabled', () => status()?.dataset.deferAck === 'true');
byId('r0235-populate-16').click();
await wait('controller proposal held pending while deferred', () => status()?.dataset.controllerStatus === 'Ok' && status()?.dataset.pending === 'true' && status()?.dataset.pendingCount === '16' && status()?.dataset.chipCount === '3');
byId('r0235-ack-pending').click();
await wait('acknowledged 16 chips overflow and scroll token list to end', () => {
    const tokens = catTokens();
    return status()?.dataset.chipCount === '16' && status()?.dataset.acks === '1' && tokens && tokens.querySelectorAll('.ix-category-filter-chip').length === 16 && tokens.scrollHeight > tokens.clientHeight + 40 && tokens.scrollTop > 0 && tokens.scrollTop + tokens.clientHeight >= tokens.scrollHeight - 8;
});
const scrolledTop = catTokens().scrollTop;
byId('r0235-toggle-defer').click();
await wait('defer mode disabled', () => status()?.dataset.deferAck === 'false');

// 7. Chip removal and reset/clear remain usable after overflow scroll.
const removeButtons = () => catTokens()?.querySelectorAll('.ix-category-filter-chip > button[aria-label^="Remove "]') ?? [];
removeButtons()[removeButtons().length - 1].click();
await wait('chip remove button removes one token', () => status()?.dataset.chipCount === '15');
byId('r0235-remove-last').click();
await wait('remove-last control removes another token', () => status()?.dataset.chipCount === '14');
catReset().click();
await wait('reset button clears all tokens', () => status()?.dataset.chipCount === '0' && status()?.dataset.clears === '1');
byId('r0235-populate-16').click();
await wait('direct controller populate scrolls token list to end', () => {
    const tokens = catTokens();
    return status()?.dataset.chipCount === '16' && tokens && tokens.scrollHeight > tokens.clientHeight + 40 && tokens.scrollTop > 0;
});
byId('r0235-clear').click();
await wait('controller clear empties tokens', () => status()?.dataset.chipCount === '0');

// 8. Teardown and remount leave no stray popup or native surface errors.
catInput().click();
await wait('popup reopened before unmount', () => catPanel()?.matches(':popover-open'));
const oldRoot = catRoot();
byId('r0235-toggle-mount').click();
await wait('unmount removes filter and open popup cleanly', () => status()?.dataset.mounted === 'false' && !catRoot() && !oldRoot.isConnected && document.querySelectorAll('.ix-category-filter-options').length === 0 && status()?.dataset.nativeErrors === '0');
byId('r0235-populate-16').click();
await wait('unmounted controller returns NotMounted', () => status()?.dataset.controllerStatus === 'NotMounted');
byId('r0235-toggle-mount').click();
await wait('filter restored', () => status()?.dataset.mounted === 'true' && catInput());
byId('r0235-remount').click();
await wait('keyed remount increments epoch', () => status()?.dataset.epoch === '1' && catInput() && catRoot() !== oldRoot);
catInput().click();
await wait('remounted filter opens top-layer popup without errors', () => catPanel()?.dataset.topLayer === 'true' && catPanel()?.matches(':popover-open') && status()?.dataset.nativeErrors === '0');
byId('r0235-outside').click();
await wait('final outside close', () => !catPanel() && status()?.dataset.nativeErrors === '0');

return {
    ok: true,
    syntheticEvents: true,
    mountedOptionFocusAndTab: true,
    topLayerEscapeClipping: true,
    scrollReposition: true,
    narrowResizeWidth: Math.round(narrowWidth),
    wideWidth: Math.round(wideWidth),
    viewportClampAndFlipTop: true,
    categoryAndPlainSelection: true,
    simultaneousSharedViewportConsumers: true,
    closedRemovedRemountedPeerIndependent: true,
    remainingConsumerAncestorScrollAndWindowResize: true,
    deferredAckAndControllerScrollTop: Math.round(scrolledTop),
    removeAndClearUsable: true,
    teardownAndRemountClean: true,
    nativeErrors: Number(status().dataset.nativeErrors)
};
"#;
