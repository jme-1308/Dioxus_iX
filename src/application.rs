//! Application frame, navigation, side panes and responsive layout primitives.
use dioxus::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static NAV_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Copy)]
struct ApplicationContext {
    expanded: Signal<bool>,
    pinned: Memo<bool>,
    app_switch: crate::ModalHandle,
    has_app_switch: Memo<bool>,
    breakpoint: Memo<Option<ApplicationBreakpoint>>,
}

#[derive(Clone)]
struct NavigationEntry {
    id: usize,
    node: Rc<MountedData>,
    enabled: Rc<Cell<bool>>,
    order: Vec<usize>,
}

#[derive(Clone)]
struct NavigationContext {
    entries: Rc<RefCell<Vec<NavigationEntry>>>,
    focused: Rc<Cell<Option<usize>>>,
    expanded: Signal<bool>,
    category: Signal<Option<usize>>,
    prefix: Vec<usize>,
}

/// Which optional application-menu overlay is visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuOverlayKind {
    Settings,
    About,
}

/// Cancelable request from the menu's Settings or About action. As in iX,
/// the action emits this request when toggling its current overlay closed too.
/// Explicit close buttons, Escape and writes to `active_overlay` bypass it.
#[derive(Clone)]
pub struct MenuOpenEvent {
    pub overlay: MenuOverlayKind,
    pub opening: bool,
    prevented: Rc<Cell<bool>>,
}

impl MenuOpenEvent {
    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }

    pub fn default_prevented(&self) -> bool {
        self.prevented.get()
    }
}

fn request_menu_overlay(
    current: Option<MenuOverlayKind>,
    requested: MenuOverlayKind,
    before: EventHandler<MenuOpenEvent>,
) -> Option<Option<MenuOverlayKind>> {
    let opening = current != Some(requested);
    let event = MenuOpenEvent {
        overlay: requested,
        opening,
        prevented: Rc::new(Cell::new(false)),
    };
    before.call(event.clone());
    (!event.default_prevented()).then_some(opening.then_some(requested))
}

/// A related application offered in the application switch dialog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppSwitchEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon_src: String,
    pub url: String,
    pub target: String,
}

/// Configuration for the header's native application-switch dialog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppSwitchConfig {
    pub current_app_id: String,
    pub apps: Vec<AppSwitchEntry>,
    pub label: String,
    pub loading_label: String,
}

impl Default for AppSwitchConfig {
    fn default() -> Self {
        Self {
            current_app_id: String::new(),
            apps: vec![],
            label: "Switch to application".into(),
            loading_label: "Loading available applications...".into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct MenuOverlayContext {
    active: Signal<Option<MenuOverlayKind>>,
}

fn navigation() -> NavigationContext {
    use_hook(|| NavigationContext {
        entries: Rc::new(RefCell::new(vec![])),
        focused: Rc::new(Cell::new(None)),
        expanded: Signal::new(false),
        category: Signal::new(None),
        prefix: vec![],
    })
}

fn navigation_key(event: KeyboardEvent, context: &NavigationContext) {
    let mut entries: Vec<_> = context
        .entries
        .borrow()
        .iter()
        .filter(|entry| entry.enabled.get())
        .cloned()
        .collect();
    entries.sort_by(|left, right| left.order.cmp(&right.order));
    if entries.is_empty() {
        return;
    }
    let current = entries
        .iter()
        .position(|entry| Some(entry.id) == context.focused.get())
        .unwrap_or(0);
    let index = match event.key() {
        Key::ArrowDown => (current + 1) % entries.len(),
        Key::ArrowUp => (current + entries.len() - 1) % entries.len(),
        Key::Home => 0,
        Key::End => entries.len() - 1,
        _ => return,
    };
    event.prevent_default();
    event.stop_propagation();
    let mounted = entries[index].node.clone();
    context.focused.set(Some(entries[index].id));
    spawn(async move {
        let _ = mounted.set_focus(true).await;
    });
}

fn navigation_item() -> (usize, Option<NavigationContext>) {
    let id = use_hook(|| NAV_ID.fetch_add(1, Ordering::Relaxed));
    let context = try_consume_context::<NavigationContext>();
    let cleanup = context.clone();
    use_drop(move || {
        if let Some(context) = cleanup {
            context.entries.borrow_mut().retain(|entry| entry.id != id);
        }
    });
    (id, context)
}

/// Forces the upstream responsive frame mode, useful for embedded previews and testing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplicationBreakpoint {
    Small,
    Medium,
    Large,
}

impl ApplicationBreakpoint {
    fn class(self) -> &'static str {
        match self {
            Self::Small => "ix-app-small",
            Self::Medium => "ix-app-medium",
            Self::Large => "ix-app-large",
        }
    }
}

/// Application frame. Header/menu/sidebar/footer are explicit Rust element slots.
/// Supply `menu_expanded` to share the menu state with external controls.
#[component]
pub fn Application(
    children: Element,
    #[props(default)] header: Option<Element>,
    #[props(default)] menu: Option<Element>,
    #[props(default)] sidebar: Option<Element>,
    #[props(default)] footer: Option<Element>,
    #[props(default)] menu_expanded: Option<Signal<bool>>,
    #[props(default)] start_expanded: bool,
    #[props(default)] pinned_menu: bool,
    #[props(default)] force_breakpoint: Option<ApplicationBreakpoint>,
    #[props(default)] class: String,
    #[props(default)] app_switch_config: Option<AppSwitchConfig>,
) -> Element {
    let local = use_signal(|| start_expanded);
    let mut expanded = menu_expanded.unwrap_or(local);
    let pinned = use_memo(use_reactive((&pinned_menu,), |(pinned,)| pinned));
    let app_switch = crate::use_modal();
    let has_app_switch = use_memo(use_reactive!(
        |app_switch_config| app_switch_config.is_some()
    ));
    let breakpoint = use_memo(use_reactive!(|force_breakpoint| force_breakpoint));
    use_context_provider(|| ApplicationContext {
        expanded,
        pinned,
        app_switch,
        has_app_switch,
        breakpoint,
    });
    let breakpoint = force_breakpoint
        .map(ApplicationBreakpoint::class)
        .unwrap_or("ix-app-auto");
    rsx! {
        div { class: "ix-application {breakpoint} {class}",
            "data-menu-expanded": expanded().to_string(), "data-menu-pinned": pinned_menu.to_string(),
            {header}
            div { class: "ix-application-body",
                if menu.is_some() { aside { class: "ix-application-menu-slot", {menu} } }
                if sidebar.is_some() { aside { class: "ix-application-sidebar", {sidebar} } }
                div { class: "ix-application-content-area",
                    main { class: "ix-application-main", onclick: move |_| if !pinned_menu { expanded.set(false); }, {children} }
                    if footer.is_some() { footer { class: "ix-application-footer", {footer} } }
                }
            }
            if let Some(config) = app_switch_config { ApplicationSwitch { modal: app_switch, config } }
        }
    }
}

#[component]
fn ApplicationSwitch(modal: crate::ModalHandle, config: AppSwitchConfig) -> Element {
    let current = config.current_app_id;
    rsx! {
        crate::Modal { modal, label: config.label.clone(), size: crate::ModalSize::Xl, close_on_backdrop_click: true,
            crate::ModalHeader { icon: "apps", "{config.label}" }
            crate::ModalContent {
                div { class: "ix-application-switch-apps",
                    if config.apps.is_empty() { div { class: "ix-application-switch-loading", crate::Spinner {} "{config.loading_label}" } }
                    for app in config.apps {
                        a { class: "ix-application-switch-entry", href: app.url, target: app.target.clone(),
                            rel: if app.target == "_blank" { Some("noopener noreferrer") } else { None },
                            "data-selected": (app.id == current).to_string(),
                            "aria-current": if app.id == current { Some("page") } else { None },
                            onclick: move |_| modal.close(String::new()),
                            img { src: app.icon_src, alt: "" }
                            div { h3 { "{app.name}" if !matches!(app.target.as_str(), "_self" | "_parent" | "_top") { crate::Icon { name: "open-external", size: 12 } } } p { "{app.description}" } }
                        }
                    }
                }
            }
        }
    }
}

/// Application title bar with neutral application pictogram, menu toggle and action slots.
/// The port deliberately has no company logo default or bundled trademark artwork.
#[component]
pub fn ApplicationHeader(
    #[props(default = "Dioxus iX".to_string())] name: String,
    #[props(default)] name_suffix: Option<String>,
    #[props(default)] app_icon: Option<String>,
    #[props(default)] actions: Option<Element>,
    #[props(default)] children: Option<Element>,
    #[props(default)] hide_bottom_border: bool,
    #[props(default)] show_menu: bool,
    #[props(default)] show_app_switch: bool,
    #[props(default)] onmenutoggle: EventHandler<bool>,
    #[props(default)] onopenappswitch: EventHandler<MouseEvent>,
) -> Element {
    let context = try_consume_context::<ApplicationContext>();
    let header_breakpoint = context
        .and_then(|context| (context.breakpoint)())
        .map(ApplicationBreakpoint::class)
        .unwrap_or("ix-app-auto");
    let overflow = crate::use_popover();
    let mut standalone = use_signal(|| false);
    let mut expanded = context.map(|c| c.expanded).unwrap_or(standalone);
    rsx! {
        header { class: "ix-application-header {header_breakpoint}", "data-borderless": hide_bottom_border.to_string(),
            div { class: "ix-application-header-leading",
                if show_menu || context.is_some() {
                    button { class: "ix-app-icon-button ix-header-menu-toggle", r#type: "button", "data-explicit": show_menu.to_string(), "aria-label": if expanded() { "Collapse navigation" } else { "Expand navigation" },
                        "aria-expanded": expanded().to_string(), onclick: move |_| { let next = !expanded(); expanded.set(next); standalone.set(next); onmenutoggle.call(next); },
                        crate::Icon { name: "app-menu".to_string(), size: 24 }
                    }
                }
                if let Some(icon) = app_icon { span { class: "ix-application-app-icon", crate::Icon { name: icon, size: 32 } } }
                if let Some(context) = context.filter(|context| (context.has_app_switch)()) {
                    span { class: "ix-header-app-switch", onclick: move |event| onopenappswitch.call(event),
                        crate::ModalTrigger { modal: context.app_switch, variant: crate::ButtonVariant::SubtleTertiary,
                            span { class: "ix-app-switch-icon", crate::Icon { name: "apps", size: 24 } }
                            span { class: "ix-sr-only", "Switch application" }
                        }
                    }
                } else if show_app_switch {
                    button { class: "ix-app-icon-button", r#type: "button", "aria-label": "Switch application", onclick: move |event| onopenappswitch.call(event),
                        crate::Icon { name: "apps".to_string(), size: 24 }
                    }
                }
                div { class: "ix-application-name",
                    span { "{name}" }
                    if let Some(suffix) = name_suffix { span { class: "ix-application-name-suffix", "{suffix}" } }
                }
                {children}
            }
            div { class: "ix-application-header-actions", {actions.clone()} }
            if actions.is_some() { div { class: "ix-header-overflow",
                crate::PopoverTrigger { popover: overflow, variant: crate::ButtonVariant::SubtleTertiary,
                    crate::Icon { name: "more-menu", size: 24 }
                    span { class: "ix-sr-only", "More header actions" }
                }
                crate::Popover { popover: overflow, label: "Header actions", close_on_click_outside: true,
                    div { class: "ix-header-overflow-actions", {actions} }
                }
            } }
        }
    }
}

/// Application navigation rail. Categories, avatar and overlays are Rust element slots.
/// Up/Down/Home/End move among enabled mounted navigation controls.
#[component]
pub fn Menu(
    children: Element,
    #[props(default)] bottom: Option<Element>,
    #[props(default)] avatar: Option<Element>,
    #[props(default)] settings: Option<Element>,
    #[props(default)] about: Option<Element>,
    #[props(default)] start_expanded: bool,
    #[props(default)] pinned: bool,
    #[props(default)] application_name: Option<String>,
    #[props(default)] enable_toggle_theme: bool,
    #[props(default)] onexpandchange: EventHandler<bool>,
    #[props(default)] onopensettings: EventHandler<()>,
    #[props(default)] onopenabout: EventHandler<()>,
    /// Veto the Settings action before changing the active overlay.
    #[props(default)]
    onbeforeopensettings: EventHandler<MenuOpenEvent>,
    /// Veto the About action before changing the active overlay.
    #[props(default)]
    onbeforeopenabout: EventHandler<MenuOpenEvent>,
    #[props(default)] ontoggletheme: EventHandler<()>,
    /// Share this signal with an about/news action to open a specific overlay.
    #[props(default)]
    active_overlay: Option<Signal<Option<MenuOverlayKind>>>,
    #[props(default)] initial_overlay: Option<MenuOverlayKind>,
) -> Element {
    let inherited = try_consume_context::<ApplicationContext>();
    let standalone = use_signal(|| start_expanded);
    let mut expanded = inherited.map(|c| c.expanded).unwrap_or(standalone);
    let pinned = inherited.map(|c| (c.pinned)()).unwrap_or(pinned);
    let mut nav = navigation();
    nav.expanded = expanded;
    let context = nav.clone();
    use_context_provider(|| nav.clone());
    let local_overlay = use_signal(|| initial_overlay);
    let mut overlay = active_overlay.unwrap_or(local_overlay);
    use_context_provider(|| MenuOverlayContext { active: overlay });
    let menu_id = use_hook(|| format!("ix-menu-{}", NAV_ID.fetch_add(1, Ordering::Relaxed)));
    let owner_id = menu_id.clone();
    let about_owner_id = menu_id.clone();
    let bridge = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    let cleanup_bridge = bridge.clone();
    use_drop(move || {
        if let Some(bridge) = cleanup_bridge.borrow_mut().take() {
            let _ = bridge.send("finish");
        }
    });
    let mount_id = menu_id.clone();
    let mut previous_expanded = use_signal(move || *expanded.peek());
    use_effect(move || {
        let next = expanded();
        if *previous_expanded.peek() != next {
            onexpandchange.call(next);
            previous_expanded.set(next);
        }
    });
    rsx! {
        nav { id: menu_id, class: "ix-menu", "aria-label": "Application navigation", "data-expanded": expanded().to_string(),
            onmounted: move |_| { *bridge.borrow_mut() = Some(dioxus::document::eval(&menu_focus_bridge(&mount_id))); },
            onkeydown: move |event| {
                if event.key() == Key::Escape { expanded.set(false); overlay.set(None); }
                else { navigation_key(event, &context); }
            },
            if let Some(name) = application_name { div { class: "ix-menu-app-name", "{name}" } }
            div { class: "ix-menu-primary", {children} }
            div { class: "ix-menu-bottom",
                {bottom}
                if settings.is_some() {
                    MenuItem { label: "Settings", icon: "cogwheel", close_on_select: false, active: overlay() == Some(MenuOverlayKind::Settings), onclick: move |_| {
                        if let Some(next) = request_menu_overlay(overlay(), MenuOverlayKind::Settings, onbeforeopensettings) {
                            overlay.set(next); onopensettings.call(());
                        }
                    } }
                }
                if enable_toggle_theme { MenuItem { label: "Toggle theme", icon: "light-dark", close_on_select: false, onclick: move |_| ontoggletheme.call(()) } }
                if about.is_some() { MenuItem { label: "About & legal information", icon: "info", close_on_select: false, active: overlay() == Some(MenuOverlayKind::About), onclick: move |_| {
                    if let Some(next) = request_menu_overlay(overlay(), MenuOverlayKind::About, onbeforeopenabout) {
                        overlay.set(next); onopenabout.call(());
                    }
                } } }
                {avatar}
                button { class: "ix-menu-toggle ix-menu-item", r#type: "button", "aria-label": if expanded() { "Collapse" } else { "Expand" }, "aria-expanded": expanded().to_string(),
                    onclick: move |_| { let next = !expanded(); expanded.set(next); },
                    span { class: "ix-menu-item-icon", crate::Icon { name: (if expanded() { "double-chevron-left" } else { "double-chevron-right" }).to_string(), size: 24 } }
                    span { class: "ix-menu-label", if expanded() { "Collapse" } else { "Expand" } }
                }
            }
        }
        if expanded() && !pinned { button { class: "ix-menu-scrim", r#type: "button", tabindex: "-1", "aria-label": "Collapse navigation", onclick: move |_| { expanded.set(false); } } }
        if overlay() == Some(MenuOverlayKind::Settings) { div { class: "ix-menu-overlay", "data-menu-owner": owner_id, {settings} } }
        if overlay() == Some(MenuOverlayKind::About) { div { class: "ix-menu-overlay", "data-menu-owner": about_owner_id, {about} } }
    }
}

// The bridge only remembers native focus and observes mounted/unmounted overlay
// nodes. Overlay selection, vetoes and notifications remain Rust state changes.
fn menu_focus_bridge(menu_id: &str) -> String {
    let id = serde_json::to_string(menu_id).expect("menu id serialization");
    format!(
        r#"
        const nav = document.getElementById({id});
        if (!nav) return;
        const owner = {id};
        const panelForMenu = () => Array.from(document.querySelectorAll('.ix-menu-overlay[data-menu-owner]')).find(node => node.dataset.menuOwner === owner) || null;
        let panel = null;
        let invoker = null;
        let outsideFocus = document.activeElement;
        let focusOwned = false;
        const focusIn = event => {{
            focusOwned = !!event.target.closest?.('.ix-menu-overlay[data-menu-owner="' + owner + '"]');
            if (!focusOwned) outsideFocus = event.target;
        }};
        document.addEventListener('focusin', focusIn, true);
        const sync = () => {{
            const next = panelForMenu();
            if (next === panel) return;
            if (next) {{
                // A second action may replace Settings with About directly.
                // Preserve the prior invoker while focus remains in the old panel.
                if (!panel || !panel.contains(outsideFocus)) invoker = outsideFocus;
                panel = next;
                focusOwned = focusOwned || panel.contains(document.activeElement);
                return;
            }}
            const active = document.activeElement;
            const owned = panel && (panel.contains(active) || (active === document.body && focusOwned));
            panel = null;
            focusOwned = false;
            if (owned && invoker instanceof HTMLElement && invoker.isConnected && !invoker.matches(':disabled,[aria-disabled="true"]') && !invoker.closest('[hidden],[inert]') && invoker.getClientRects().length) {{
                invoker.focus({{preventScroll: true}});
            }}
            invoker = null;
        }};
        let finish;
        const removed = new Promise(resolve => {{ finish = resolve; }});
        const observer = new MutationObserver(() => {{ if (!nav.isConnected) finish(); else sync(); }});
        observer.observe(document.documentElement, {{childList:true,subtree:true}});
        sync();
        try {{ await Promise.race([removed, dioxus.recv()]); }}
        finally {{ observer.disconnect(); document.removeEventListener('focusin', focusIn, true); }}
        "#
    )
}

/// Navigation action or link with active, disabled and notification states.
#[component]
pub fn MenuItem(
    label: String,
    #[props(default)] icon: Option<String>,
    #[props(default)] active: bool,
    #[props(default)] disabled: bool,
    #[props(default)] home: bool,
    #[props(default)] notifications: Option<u32>,
    #[props(default)] href: Option<String>,
    #[props(default = "_self".to_string())] target: String,
    #[props(default)] tooltip_text: Option<String>,
    #[props(default = true)] close_on_select: bool,
    #[props(default)] onclick: EventHandler<MouseEvent>,
) -> Element {
    let (id, context) = navigation_item();
    let application = try_consume_context::<ApplicationContext>();
    let overlay = try_consume_context::<MenuOverlayContext>();
    let enabled = use_hook(|| Rc::new(Cell::new(!disabled)));
    enabled.set(!disabled);
    let mount_enabled = enabled.clone();
    let mount_context = context.clone();
    let focus_context = context.clone();
    let click_context = context.clone();
    let title = tooltip_text.unwrap_or(label.clone());
    let content = rsx! {
        span { class: "ix-menu-item-icon", if let Some(icon) = icon { crate::Icon { name: icon, size: 24 } } }
        span { class: "ix-menu-label", "{label}" }
        if let Some(count) = notifications.filter(|count| *count > 0) { span { class: "ix-menu-notifications", "aria-label": format!("{count} notifications"), "{count.min(99)}" } }
    };
    let class = if active {
        "ix-menu-item ix-menu-item-active"
    } else {
        "ix-menu-item"
    };
    let onmount = move |event: MountedEvent| {
        if let Some(context) = &mount_context {
            let mut order = context.prefix.clone();
            order.push(id);
            context.entries.borrow_mut().push(NavigationEntry {
                id,
                node: event.data(),
                enabled: mount_enabled.clone(),
                order,
            });
        }
    };
    let onfocus = move |_| {
        if let Some(context) = &focus_context {
            context.focused.set(Some(id));
        }
    };
    let onactivate = move |event: MouseEvent| {
        if disabled {
            event.prevent_default();
            return;
        }
        if let Some(context) = &click_context {
            let mut category = context.category;
            category.set(None);
        }
        if close_on_select {
            if let Some(mut application) = application {
                if !(application.pinned)() {
                    application.expanded.set(false);
                }
            }
            if let Some(mut overlay) = overlay {
                overlay.active.set(None);
            }
        }
        onclick.call(event);
    };
    rsx! {
        if let Some(href) = href {
            a { class, href: if disabled { None } else { Some(href) }, target: target.clone(),
                rel: if target == "_blank" { Some("noopener noreferrer") } else { None },
                title, "aria-label": label, "aria-current": if active { Some("page") } else { None },
                "aria-disabled": disabled.to_string(), tabindex: if disabled { "-1" } else { "0" }, "data-home": home.to_string(),
                onmounted: onmount, onfocus, onclick: onactivate, {content}
            }
        } else {
            button { class, r#type: "button", disabled, title, "aria-label": label,
                "aria-current": if active { Some("page") } else { None }, "data-home": home.to_string(),
                onmounted: onmount, onfocus, onclick: onactivate, {content}
            }
        }
    }
}

/// Collapsible menu category; a collapsed rail shows its children in an adjacent flyout.
#[component]
pub fn MenuCategory(
    children: Element,
    label: String,
    #[props(default)] icon: Option<String>,
    #[props(default)] notifications: Option<u32>,
    #[props(default)] active: bool,
    #[props(default)] disable_tooltip: bool,
) -> Element {
    let (id, context) = navigation_item();
    let mut standalone = use_signal(|| None::<usize>);
    let mut category = context.as_ref().map(|c| c.category).unwrap_or(standalone);
    let expanded = context.as_ref().map(|c| (c.expanded)()).unwrap_or(true);
    let is_open = category() == Some(id);
    let mount_context = context.clone();
    let focus_context = context.clone();
    let content_id = format!("ix-category-{id}");
    let mut trigger = use_signal(|| None::<Rc<MountedData>>);
    rsx! {
        div { class: "ix-menu-category", "data-open": is_open.to_string(),
            onkeydown: move |event| match event.key() {
                Key::ArrowRight => { event.prevent_default(); event.stop_propagation(); category.set(Some(id)); }
                Key::ArrowLeft | Key::Escape => { event.prevent_default(); event.stop_propagation(); category.set(None);
                    let node = trigger.peek().clone(); spawn(async move { if let Some(node) = node { let _ = node.set_focus(true).await; } });
                }
                _ => {}
            },
            button { class: if active { "ix-menu-item ix-menu-item-active" } else { "ix-menu-item" }, r#type: "button", title: (!disable_tooltip).then(|| label.clone()),
                "aria-label": label.clone(), "aria-expanded": is_open.to_string(), "aria-controls": content_id.clone(),
                onmounted: move |event| { trigger.set(Some(event.data())); if let Some(context) = &mount_context { let mut order = context.prefix.clone(); order.push(id); context.entries.borrow_mut().push(NavigationEntry { id, node: event.data(), enabled: Rc::new(Cell::new(true)), order }); } },
                onfocus: move |_| if let Some(context) = &focus_context { context.focused.set(Some(id)); },
                onclick: move |_| { let next = if category() == Some(id) { None } else { Some(id) }; category.set(next); standalone.set(next); },
                span { class: "ix-menu-item-icon", if let Some(icon) = icon { crate::Icon { name: icon, size: 24 } } }
                span { class: "ix-menu-label", "{label}" }
                if let Some(count) = notifications { span { class: "ix-menu-notifications", "{count}" } }
                span { class: "ix-menu-category-chevron", crate::Icon { name: (if is_open { "chevron-up-small" } else { "chevron-down-small" }).to_string(), size: 16 } }
            }
            if is_open {
                if !expanded { button { class: "ix-category-backdrop", r#type: "button", tabindex: "-1", "aria-label": "Close category", onclick: move |_| category.set(None) } }
                div { id: content_id.clone(), class: if expanded { "ix-menu-category-items" } else { "ix-menu-category-flyout" },
                    if !expanded { div { class: "ix-menu-category-heading", "{label}" } }
                    MenuCategoryContent { parent: id, {children} }
                }
            }
        }
    }
}

#[component]
fn MenuCategoryContent(children: Element, parent: usize) -> Element {
    let fallback = navigation();
    let mut context = try_consume_context::<NavigationContext>().unwrap_or(fallback);
    context.prefix.push(parent);
    use_context_provider(|| context);
    rsx! { {children} }
}

#[derive(Clone, Copy)]
struct AvatarMenuContext {
    open: Signal<bool>,
}

/// Avatar dropdown for user-specific actions and logout.
#[component]
pub fn MenuAvatar(
    #[props(default = "User".to_string())] top: String,
    #[props(default)] bottom: Option<String>,
    #[props(default)] initials: Option<String>,
    #[props(default)] image: Option<String>,
    #[props(default)] children: Option<Element>,
    #[props(default)] hide_logout_button: bool,
    #[props(default)] onlogout: EventHandler<MouseEvent>,
) -> Element {
    let mut open = use_signal(|| false);
    let mut trigger = use_signal(|| None::<Rc<MountedData>>);
    let mut was_open = use_signal(|| false);
    let mut restore_focus = use_signal(|| true);
    use_effect(move || {
        let next = open();
        if *was_open.peek() && !next && *restore_focus.peek() {
            let node = trigger.peek().clone();
            spawn(async move {
                if let Some(node) = node {
                    let _ = node.set_focus(true).await;
                }
            });
        }
        if !*was_open.peek() && next {
            restore_focus.set(true);
        }
        was_open.set(next);
    });
    use_context_provider(|| AvatarMenuContext { open });
    let nav = navigation();
    let nav_keys = nav.clone();
    use_context_provider(|| nav);
    let initials = initials.unwrap_or_else(|| {
        top.split_whitespace()
            .take(2)
            .filter_map(|word| word.chars().next())
            .collect()
    });
    rsx! {
        div { class: "ix-menu-avatar", onkeydown: move |event| {
            if event.key() == Key::Escape { event.prevent_default(); event.stop_propagation(); open.set(false); }
            else if event.key() == Key::Tab { restore_focus.set(false); open.set(false); }
            else if open() { navigation_key(event, &nav_keys); }
        },
            button { class: "ix-menu-avatar-trigger", r#type: "button", title: top.clone(), "aria-label": top.clone(), "aria-haspopup": "menu", "aria-expanded": open().to_string(), onclick: move |_| open.set(!open()), onmounted: move |event| trigger.set(Some(event.data())),
                span { class: "ix-menu-avatar-circle", if let Some(image) = image { img { src: image, alt: "" } } else { "{initials}" } }
                span { class: "ix-menu-avatar-text", span { "{top}" } if let Some(bottom) = bottom { small { "{bottom}" } } }
            }
            if open() {
                button { class: "ix-avatar-backdrop", r#type: "button", tabindex: "-1", "aria-label": "Close user menu", onclick: move |_| open.set(false) }
                div { class: "ix-menu-avatar-popup", role: "menu", "aria-label": "User menu",
                    {children}
                    if !hide_logout_button { MenuAvatarItem { label: "Logout", icon: "log-out", onclick: onlogout } }
                }
            }
        }
    }
}

/// User-menu action; closes its owning avatar dropdown when activated.
#[component]
pub fn MenuAvatarItem(
    label: String,
    #[props(default)] icon: Option<String>,
    #[props(default)] disabled: bool,
    #[props(default)] onclick: EventHandler<MouseEvent>,
) -> Element {
    let avatar = try_consume_context::<AvatarMenuContext>();
    let (id, context) = navigation_item();
    let enabled = use_hook(|| Rc::new(Cell::new(!disabled)));
    enabled.set(!disabled);
    let focus_context = context.clone();
    rsx! {
        button { class: "ix-menu-avatar-item", r#type: "button", role: "menuitem", disabled,
            onmounted: move |event| if let Some(context) = &context {
                let should_focus = enabled.get() && !context.entries.borrow().iter().any(|entry| entry.enabled.get());
                let mut order = context.prefix.clone(); order.push(id); context.entries.borrow_mut().push(NavigationEntry { id, node: event.data(), enabled: enabled.clone(), order });
                if should_focus && avatar.is_some() { spawn(async move { let _ = event.set_focus(true).await; }); }
            },
            onfocus: move |_| if let Some(context) = &focus_context { context.focused.set(Some(id)); },
            onclick: move |event| { if let Some(mut avatar) = avatar { avatar.open.set(false); } onclick.call(event); },
            if let Some(icon) = icon { crate::Icon { name: icon, size: 24 } } "{label}"
        }
    }
}

/// Settings content opened by the owning menu, or standalone with `show: true`.
#[component]
pub fn MenuSettings(
    children: Element,
    #[props(default = "Settings".to_string())] label: String,
    #[props(default = true)] show: bool,
    #[props(default)] onclose: EventHandler<()>,
    #[props(default = "Close Settings".to_string())] aria_label_close_button: String,
) -> Element {
    let context = try_consume_context::<MenuOverlayContext>();
    rsx! { MenuOverlay { label: label.clone(), show, context, onclose, close_label: aria_label_close_button,
        {children}
    } }
}

/// About and legal content. No vendor logo or vendor-specific legal claims are injected.
#[component]
pub fn MenuAbout(
    children: Element,
    #[props(default = "About & legal information".to_string())] label: String,
    #[props(default = true)] show: bool,
    #[props(default)] onclose: EventHandler<()>,
    #[props(default = "Close About".to_string())] aria_label_close_button: String,
) -> Element {
    let context = try_consume_context::<MenuOverlayContext>();
    rsx! { MenuOverlay { label: label.clone(), show, context, onclose, close_label: aria_label_close_button,
        {children}
    } }
}

#[component]
fn MenuOverlay(
    children: Element,
    label: String,
    show: bool,
    context: Option<MenuOverlayContext>,
    onclose: EventHandler<()>,
    close_label: String,
) -> Element {
    rsx! {
        if show {
            section { class: "ix-menu-overlay-panel", role: "dialog", "aria-label": label.clone(),
                onkeydown: move |event| if event.key() == Key::Escape { event.prevent_default(); event.stop_propagation(); if let Some(mut context) = context { context.active.set(None); } onclose.call(()); },
                div { class: "ix-menu-overlay-header", h2 { "{label}" }
                    button { class: "ix-app-icon-button", r#type: "button", "aria-label": close_label,
                        onmounted: move |event| async move { let _ = event.set_focus(true).await; },
                        onclick: move |_| { if let Some(mut context) = context { context.active.set(None); } onclose.call(()); },
                        crate::Icon { name: "close".to_string(), size: 24 }
                    }
                }
                div { class: "ix-menu-overlay-content", {children} }
            }
        }
    }
}

/// Side pane presentation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaneVariant {
    #[default]
    Inline,
    Floating,
}

/// Side of the content occupied by a pane.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaneComposition {
    Left,
    Right,
    #[default]
    Top,
    Bottom,
}

impl PaneComposition {
    fn class(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }
}

/// Determines which pair of panes spans the full available cross axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaneLayoutOrder {
    #[default]
    FullVertical,
    FullHorizontal,
}

#[derive(Clone, Copy, PartialEq)]
struct PaneSettings {
    composition: PaneComposition,
    variant: PaneVariant,
    borderless: bool,
}

#[derive(Clone, Copy)]
struct PaneContext {
    settings: Memo<PaneSettings>,
}

#[derive(Clone, Copy)]
struct PaneLayoutContext {
    close_requested: Signal<u64>,
}

/// Cancelable expansion request. Vetoing it preserves both the current pane
/// state and the post-change notification. `slot` is empty outside a layout.
#[derive(Clone)]
pub struct PaneExpandedChangeEvent {
    pub slot: String,
    pub expanded: bool,
    prevented: Rc<Cell<bool>>,
}

impl PaneExpandedChangeEvent {
    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }
    pub fn default_prevented(&self) -> bool {
        self.prevented.get()
    }
}

fn request_pane_expansion(
    current: bool,
    next: bool,
    slot: String,
    before: EventHandler<PaneExpandedChangeEvent>,
) -> bool {
    if current == next {
        return false;
    }
    let event = PaneExpandedChangeEvent {
        slot,
        expanded: next,
        prevented: Rc::new(Cell::new(false)),
    };
    before.call(event.clone());
    !event.default_prevented()
}

#[component]
fn PaneSlot(
    children: Element,
    composition: PaneComposition,
    variant: PaneVariant,
    borderless: bool,
) -> Element {
    let settings = use_memo(use_reactive(
        (&composition, &variant, &borderless),
        |(composition, variant, borderless)| PaneSettings {
            composition,
            variant,
            borderless,
        },
    ));
    use_context_provider(|| PaneContext { settings });
    rsx! { div { class: "ix-pane-slot ix-pane-slot-{composition.class()}", {children} } }
}

/// Layout with independently collapsible left, right, top and bottom pane slots.
#[component]
pub fn PaneLayout(
    children: Element,
    #[props(default)] left: Option<Element>,
    #[props(default)] right: Option<Element>,
    #[props(default)] top: Option<Element>,
    #[props(default)] bottom: Option<Element>,
    #[props(default)] layout: PaneLayoutOrder,
    #[props(default)] variant: PaneVariant,
    #[props(default)] borderless: bool,
) -> Element {
    let mut close_requested = use_signal(|| 0_u64);
    use_context_provider(|| PaneLayoutContext { close_requested });
    rsx! {
        div { class: "ix-pane-layout", "data-layout": if layout == PaneLayoutOrder::FullVertical { "full-vertical" } else { "full-horizontal" },
            if let Some(left) = left { PaneSlot { composition: PaneComposition::Left, variant, borderless, {left} } }
            if let Some(right) = right { PaneSlot { composition: PaneComposition::Right, variant, borderless, {right} } }
            if let Some(top) = top { PaneSlot { composition: PaneComposition::Top, variant, borderless, {top} } }
            if let Some(bottom) = bottom { PaneSlot { composition: PaneComposition::Bottom, variant, borderless, {bottom} } }
            div { class: "ix-pane-layout-content", onclick: move |_| { let next = close_requested().wrapping_add(1); close_requested.set(next); }, {children} }
        }
    }
}

/// Collapsible pane with mutable expansion and optional external state updates.
/// Changing `expanded` synchronizes the local state; user requests can be vetoed
/// through `onbeforeexpandedchange`. The owning layout supplies composition and
/// default styling. Desktop size is capped at half of the parent dimension;
/// the upstream small breakpoint (48em) uses full-height mobile overlays.
#[component]
pub fn Pane(
    children: Element,
    heading: String,
    #[props(default)] expanded: bool,
    #[props(default)] composition: PaneComposition,
    #[props(default)] variant: PaneVariant,
    #[props(default = "240px".to_string())] size: String,
    #[props(default)] icon: Option<String>,
    #[props(default)] header: Option<Element>,
    #[props(default)] hide_on_collapse: bool,
    #[props(default)] borderless: bool,
    #[props(default)] no_padding: bool,
    #[props(default)] close_on_click_outside: bool,
    #[props(default)] ignore_layout_settings: bool,
    #[props(default)] aria_label_icon: Option<String>,
    #[props(default)] aria_label_collapse_close_button: Option<String>,
    #[props(default)] onbeforeexpandedchange: EventHandler<PaneExpandedChangeEvent>,
    #[props(default)] onexpandedchange: EventHandler<bool>,
) -> Element {
    let context = try_consume_context::<PaneContext>().map(|c| (c.settings)());
    let composition = context.map(|c| c.composition).unwrap_or(composition);
    let styling = context.filter(|_| !ignore_layout_settings);
    let variant = styling.map(|c| c.variant).unwrap_or(variant);
    let borderless = styling.map(|c| c.borderless).unwrap_or(borderless);
    let layout = try_consume_context::<PaneLayoutContext>();
    let mut state = use_signal(|| expanded);
    use_effect(use_reactive((&expanded,), move |(expanded,)| {
        state.set(expanded)
    }));
    let slot = context
        .map(|_| composition.class().to_string())
        .unwrap_or_default();
    let request = use_callback(move |next: bool| {
        if request_pane_expansion(*state.peek(), next, slot.clone(), onbeforeexpandedchange) {
            state.set(next);
            onexpandedchange.call(next);
        }
    });
    let mut observed_close = use_signal(|| {
        layout
            .map(|layout| *layout.close_requested.peek())
            .unwrap_or(0)
    });
    use_effect(use_reactive((&variant,), move |(variant,)| {
        if let Some(layout) = layout {
            let tick = (layout.close_requested)();
            if tick != *observed_close.peek() {
                observed_close.set(tick);
                if variant == PaneVariant::Floating {
                    request.call(false);
                }
            }
        }
    }));
    let mut dimensions = use_signal(|| (0.0, 0.0, false));
    let (width, height, mobile) = dimensions();
    let vertical = matches!(composition, PaneComposition::Left | PaneComposition::Right);
    let size = pane_expanded_size(&size, if vertical { width } else { height });
    let expanded = state();
    let id = use_hook(|| format!("ix-pane-{}", NAV_ID.fetch_add(1, Ordering::Relaxed)));
    let root_id = format!("{id}-root");
    let focus_id = root_id.clone();
    let cleanup_id = root_id.clone();
    use_drop(move || {
        let id = serde_json::to_string(&cleanup_id).expect("pane id serialization");
        let _ = dioxus::document::eval(&format!(
            "document.getElementById({id})?.__ixPaneDispose?.();"
        ));
    });
    let label = if expanded {
        if hide_on_collapse || mobile {
            "Close pane"
        } else {
            "Collapse pane"
        }
    } else {
        "Expand pane"
    };
    let label = aria_label_collapse_close_button.unwrap_or_else(|| label.to_string());
    let toggle_icon = if (hide_on_collapse || mobile) && expanded {
        "close"
    } else {
        let position = if mobile {
            match composition {
                PaneComposition::Left => PaneComposition::Top,
                PaneComposition::Right => PaneComposition::Bottom,
                other => other,
            }
        } else {
            composition
        };
        match (position, expanded) {
            (PaneComposition::Left, true) | (PaneComposition::Right, false) => {
                "double-chevron-left"
            }
            (PaneComposition::Right, true) | (PaneComposition::Left, false) => {
                "double-chevron-right"
            }
            (PaneComposition::Top, true) | (PaneComposition::Bottom, false) => "double-chevron-up",
            _ => "double-chevron-down",
        }
    };
    rsx! {
            aside { id: root_id.clone(), class: "ix-pane ix-pane-{composition.class()}", "aria-label": heading.clone(), hidden: !expanded && hide_on_collapse,
                "data-expanded": expanded.to_string(), "data-floating": (variant == PaneVariant::Floating).to_string(),
                "data-mobile": mobile.to_string(), "data-hide-on-collapse": hide_on_collapse.to_string(),
                "data-close-on-outside": close_on_click_outside.to_string(),
                "data-borderless": borderless.to_string(), style: "--ix-pane-size:{size}px",
                onmounted: move |_| {
                    let id = root_id.clone();
                    spawn(async move {
                        let mut bridge = dioxus::document::eval(PANE_DOM_BRIDGE);
                        if bridge.send(id).is_err() { return; }
                        while let Ok((kind, width, height, mobile)) = bridge.recv::<(String, f64, f64, bool)>().await {
                            match kind.as_str() {
                                "measure" if width.is_finite() && height.is_finite() => dimensions.set((width.max(0.0), height.max(0.0), mobile)),
                                "outside" => request.call(false),
                                "disposed" => break,
                                _ => {}
                            }
                        }
                    });
                },
                onkeydown: move |event| match event.key() {
                    Key::Escape if expanded && variant == PaneVariant::Floating => { event.prevent_default(); event.stop_propagation(); request.call(false); }
                    Key::Tab if expanded && variant == PaneVariant::Floating => {
                        event.prevent_default(); event.stop_propagation();
                        let id = focus_id.clone();
                        spawn(async move { pane_advance_focus(id, event.modifiers().shift()).await; });
                    }
                    _ => {}
                },
                div { class: "ix-pane-header",
                    button { class: "ix-app-icon-button", r#type: "button", "aria-label": label, "aria-expanded": expanded.to_string(), "aria-controls": id.clone(),
                        onclick: move |_| request.call(!expanded), crate::Icon { name: toggle_icon.to_string(), size: 24 }
                    }
                    div { class: "ix-pane-heading", if let Some(icon) = icon { span { role: if aria_label_icon.is_some() { Some("img") } else { None }, "aria-label": aria_label_icon, crate::Icon { name: icon, size: 24 } } } span { "{heading}" } }
                    if expanded { {header} }
                }
                div { id, class: "ix-pane-content", hidden: !expanded, "data-no-padding": no_padding.to_string(), {children} }
            }
    }
}

fn pane_expanded_size(size: &str, parent: f64) -> f64 {
    let size = pane_size(size);
    if size.ends_with('%') {
        return if size == "50%" {
            (parent / 2.0).round()
        } else {
            (parent / 3.0).round()
        };
    }
    let pixels = size.trim_end_matches("px").parse::<f64>().unwrap_or(320.0);
    let half_parent = (parent / 2.0).round();
    if half_parent > 0.0 {
        pixels.min(half_parent)
    } else {
        pixels
    }
}

fn pane_focus_index(current: Option<usize>, count: usize, backwards: bool) -> Option<usize> {
    if count == 0 {
        return None;
    }
    Some(match current.filter(|current| *current < count) {
        Some(current) if backwards => (current + count - 1) % count,
        Some(current) => (current + 1) % count,
        None if backwards => count - 1,
        None => 0,
    })
}

async fn pane_advance_focus(id: String, backwards: bool) {
    let query = crate::dom_query::eval(
        r#"
        const id = await dioxus.recv(), pane = document.getElementById(id);
        if (!pane || pane.dataset.expanded !== 'true') return null;
        const items = [...pane.querySelectorAll('button,a[href],input,select,textarea,[tabindex]')].filter(el => el.tabIndex >= 0 && !el.matches(':disabled') && !el.closest('[aria-hidden="true"],[hidden],[inert]') && el.getClientRects().length);
        return [items.indexOf(document.activeElement), items.length];
    "#,
    );
    if query.send(id.clone()).is_err() {
        return;
    }
    let Ok(value) = query.await else {
        return;
    };
    let Ok((current, count)) = serde_json::from_value::<(i64, usize)>(value) else {
        return;
    };
    let Some(index) = pane_focus_index(usize::try_from(current).ok(), count, backwards) else {
        return;
    };
    let focus = crate::dom_query::eval(
        r#"
        const [id, index] = await dioxus.recv(), pane = document.getElementById(id);
        if (!pane || pane.dataset.expanded !== 'true') return null;
        const items = [...pane.querySelectorAll('button,a[href],input,select,textarea,[tabindex]')].filter(el => el.tabIndex >= 0 && !el.matches(':disabled') && !el.closest('[aria-hidden="true"],[hidden],[inert]') && el.getClientRects().length);
        items[index]?.focus(); return null;
    "#,
    );
    let _ = focus.send((id, index));
    let _ = focus.await;
}

// Native metrics, focus activation/restoration and outside-click delivery only.
// Expansion decisions, cancellation, size limits and Tab traversal stay in Rust.
const PANE_DOM_BRIDGE: &str = r#"
const id = await dioxus.recv(), pane = document.getElementById(id);
if (!pane) return;
pane.__ixPaneDispose?.();
const parent = pane.closest('.ix-pane-layout') ?? pane.parentElement;
const stackKey = Symbol.for('dioxus-ix.pane-focus-stack');
const stack = document[stackKey] ??= [];
let disposed = false, trapped = false, previousFocus = null, frame = 0;
const focusable = () => [...pane.querySelectorAll('button,a[href],input,select,textarea,[tabindex]')].filter(el => el.tabIndex >= 0 && !el.matches(':disabled') && !el.closest('[aria-hidden="true"],[hidden],[inert]') && el.getClientRects().length);
const top = () => stack.at(-1) === pane;
const releaseFocus = () => {
    const at = stack.indexOf(pane); if (at >= 0) stack.splice(at, 1);
    const active = document.activeElement, target = previousFocus;
    previousFocus = null; trapped = false;
    if (target?.isConnected && (pane.contains(active) || active === document.body || active === document.documentElement)) target.focus({preventScroll:true});
};
const syncFocus = () => {
    const active = pane.dataset.expanded === 'true' && pane.dataset.floating === 'true';
    if (active && !trapped) {
        previousFocus = document.activeElement; trapped = true; stack.push(pane);
        cancelAnimationFrame(frame); frame = requestAnimationFrame(() => {
            if (!disposed && top() && pane.dataset.expanded === 'true') (pane.querySelector('[autofocus]') ?? focusable()[0])?.focus({preventScroll:true});
        });
    } else if (!active && trapped) releaseFocus();
};
const measure = () => {
    if (disposed) return;
    const rect = parent?.getBoundingClientRect();
    dioxus.send(['measure', rect?.width ?? 0, rect?.height ?? 0, matchMedia('(max-width:48em)').matches]);
};
const outsideEvents = new WeakSet();
const captureOutside = event => {
    if (pane.dataset.expanded !== 'true' || pane.dataset.closeOnOutside !== 'true' || event.composedPath().includes(pane)) return;
    if (pane.dataset.floating === 'true' && !top()) return;
    outsideEvents.add(event);
};
const outside = event => {
    // Capture eligibility before the target handler; deliver during bubbling
    // only if its action did not already close the pane. Opening clicks were
    // never eligible, independent of synchronous web or desktop IPC timing.
    if (!outsideEvents.has(event)) return;
    outsideEvents.delete(event);
    if (!disposed && pane.dataset.expanded === 'true') dioxus.send(['outside', 0, 0, false]);
};
const focusIn = event => {
    if (!trapped || !top() || pane.contains(event.target)) return;
    // Native top-layer content belongs to the nested control, not the pane's
    // regular DOM subtree in every renderer. Allow its own focus controller.
    if (event.target.closest?.('dialog[open],[popover]:popover-open')) return;
    (pane.querySelector('[autofocus]') ?? focusable()[0])?.focus({preventScroll:true});
};
const resize = new ResizeObserver(measure); if (parent) resize.observe(parent);
const attributes = new MutationObserver(syncFocus); attributes.observe(pane, {attributes:true, attributeFilter:['data-expanded','data-floating']});
const detached = new MutationObserver(() => { if (!pane.isConnected) dispose(); });
detached.observe(document.documentElement, {childList:true, subtree:true});
const dispose = () => {
    if (disposed) return; disposed = true; cancelAnimationFrame(frame);
    resize.disconnect(); attributes.disconnect(); detached.disconnect();
    window.removeEventListener('resize', measure);
    window.removeEventListener('click', captureOutside, true); window.removeEventListener('click', outside);
    document.removeEventListener('focusin', focusIn); if (trapped) releaseFocus();
    delete pane.__ixPaneDispose; dioxus.send(['disposed', 0, 0, false]);
};
pane.__ixPaneDispose = dispose;
window.addEventListener('resize', measure);
window.addEventListener('click', captureOutside, true); window.addEventListener('click', outside);
document.addEventListener('focusin', focusIn);
measure(); syncFocus();
"#;

fn pane_size(size: &str) -> &str {
    match size {
        "240px" | "320px" | "360px" | "480px" | "600px" | "33%" | "50%" => size,
        _ => "320px",
    }
}

/// Responsive column grid. Column widths are fractions of `columns`, clamped to 2–12.
#[component]
pub fn LayoutGrid(
    children: Element,
    #[props(default = 12)] columns: u8,
    #[props(default = 24)] gap: u8,
    #[props(default)] no_margin: bool,
) -> Element {
    let columns = columns.clamp(2, 12);
    let gap = if [8, 12, 16, 24].contains(&gap) {
        gap
    } else {
        24
    };
    rsx! { div { class: "ix-layout-grid", "data-no-margin": no_margin.to_string(), style: "--ix-grid-columns:{columns};--ix-grid-gap:{gap}px", {children} } }
}

/// Wrapping row inside [`LayoutGrid`].
#[component]
pub fn Row(children: Element) -> Element {
    rsx! { div { class: "ix-layout-row", {children} } }
}

/// Responsive column. Omitted size shares available space; explicit sizes use grid fractions.
#[component]
pub fn Col(
    children: Element,
    #[props(default)] size: Option<u8>,
    #[props(default)] size_sm: Option<u8>,
    #[props(default)] size_md: Option<u8>,
    #[props(default)] size_lg: Option<u8>,
) -> Element {
    fn basis(value: Option<u8>) -> String {
        value
            .map(|n| {
                format!(
                    "calc({} / var(--ix-grid-columns,12) * 100%)",
                    n.clamp(1, 12)
                )
            })
            .unwrap_or_else(|| "0%".to_string())
    }
    let base = basis(size);
    let sm_size = size_sm.or(size_md).or(size_lg).or(size);
    let md_size = size_md.or(size_lg).or(size);
    let lg_size = size_lg.or(size);
    let sm = basis(sm_size);
    let md = basis(md_size);
    let lg = basis(lg_size);
    let grow = u8::from(size.is_none());
    let sm_grow = u8::from(sm_size.is_none());
    let md_grow = u8::from(md_size.is_none());
    let lg_grow = u8::from(lg_size.is_none());
    rsx! { div { class: "ix-layout-col", style: "--ix-col-basis:{base};--ix-col-sm:{sm};--ix-col-md:{md};--ix-col-lg:{lg};--ix-col-grow:{grow};--ix-col-grow-sm:{sm_grow};--ix-col-grow-md:{md_grow};--ix-col-grow-lg:{lg_grow}", {children} } }
}

pub(crate) const STYLES: &str = r#"
.ix-theme .ix-application-header:not(.ix-app-small) .ix-header-menu-toggle[data-explicit=false]{display:none}@container ix-app (max-width:576px){.ix-theme .ix-application-header.ix-app-auto .ix-header-menu-toggle[data-explicit=false]{display:inline-flex}}
.ix-theme .ix-header-overflow{display:none;margin-inline-start:auto}.ix-theme .ix-header-overflow>.ix-button{width:2.5rem;height:2.5rem;padding:.5rem}.ix-theme .ix-header-overflow-actions{display:flex;flex-direction:column;gap:.5rem;align-items:flex-start}.ix-theme .ix-application-header.ix-app-small>.ix-application-header-actions{display:none}.ix-theme .ix-application-header.ix-app-small>.ix-header-overflow{display:block}.ix-theme .ix-application-header.ix-app-small .ix-header-app-switch{display:none}@container ix-app (max-width:576px){.ix-theme .ix-application-header.ix-app-auto>.ix-application-header-actions{display:none}.ix-theme .ix-application-header.ix-app-auto>.ix-header-overflow{display:block}.ix-theme .ix-application-header.ix-app-auto .ix-header-app-switch{display:none}}@media(max-width:576px){.ix-theme .ix-application-header.ix-app-auto>.ix-application-header-actions{display:none}.ix-theme .ix-application-header.ix-app-auto>.ix-header-overflow{display:block}.ix-theme .ix-application-header.ix-app-auto .ix-header-app-switch{display:none}}
.ix-theme .ix-header-app-switch>.ix-button{padding:.5rem;width:2.5rem;height:2.5rem}.ix-theme .ix-app-switch-icon{display:flex}.ix-theme .ix-application-switch-apps{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:1rem}.ix-theme .ix-application-switch-entry{display:flex;gap:1rem;padding:1rem;text-decoration:none;color:var(--ix-color-text);border:1px solid transparent;border-radius:var(--ix-radius);min-width:0}.ix-theme .ix-application-switch-entry[data-selected=true]{border-color:var(--ix-color-primary)}.ix-theme .ix-application-switch-entry:hover{background:var(--ix-color-surface-2)}.ix-theme .ix-application-switch-entry:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}.ix-theme .ix-application-switch-entry>img{width:3rem;height:3rem;object-fit:contain}.ix-theme .ix-application-switch-entry h3{display:flex;gap:.25rem;align-items:center;margin:0 0 .5rem;font-size:1rem}.ix-theme .ix-application-switch-entry p{margin:0;color:var(--ix-color-text-secondary);font-size:.875rem;overflow-wrap:anywhere}.ix-theme .ix-application-switch-loading{display:flex;align-items:center;gap:1rem;grid-column:1/-1}@media(max-width:576px){.ix-theme .ix-application-switch-apps{grid-template-columns:1fr}}
.ix-theme .ix-application{display:flex;flex-direction:column;position:relative;width:100%;height:100%;min-height:20rem;background:var(--ix-color-surface);overflow:hidden;container:ix-app / inline-size;color:var(--ix-color-text)}
.ix-theme .ix-application-body{display:flex;flex:1;min-height:0;position:relative;isolation:isolate}.ix-theme .ix-application-menu-slot{width:3.5rem;flex:none;z-index:30}.ix-theme .ix-application[data-menu-pinned=true][data-menu-expanded=true] .ix-application-menu-slot{width:16rem}
.ix-theme .ix-application-sidebar{flex:none;width:16rem;border-inline-end:1px solid var(--ix-color-border);overflow:auto}.ix-theme .ix-application-content-area{display:flex;flex-direction:column;flex:1;min-width:0;min-height:0}.ix-theme .ix-application-main{overflow:auto;flex:1;min-height:0;position:relative}.ix-theme .ix-application-footer{flex:none}
.ix-theme .ix-application-header{display:flex;align-items:center;justify-content:space-between;flex:none;min-height:3.5rem;padding:0 .75rem;background:var(--ix-color-surface-1);border-bottom:1px solid var(--ix-color-border);gap:1rem;position:relative;z-index:35;box-sizing:border-box}
.ix-theme .ix-application-header[data-borderless=true]{border-bottom-color:transparent}.ix-theme .ix-application-header-leading,.ix-theme .ix-application-header-actions{display:flex;align-items:center;min-width:0;gap:.5rem}.ix-theme .ix-application-header-actions{margin-inline-start:auto}.ix-theme .ix-application-app-icon{display:flex;align-items:center;justify-content:center;margin-inline:.25rem}
.ix-theme .ix-application-name{display:flex;align-items:baseline;gap:1rem;min-width:0;margin-inline:.5rem;font-size:1rem;font-weight:600;white-space:nowrap}.ix-theme .ix-application-name>span{overflow:hidden;text-overflow:ellipsis}.ix-theme .ix-application-name-suffix{font-weight:400;color:var(--ix-color-text-secondary);font-size:.875rem}
.ix-theme .ix-app-icon-button{display:inline-flex;align-items:center;justify-content:center;border:0;background:transparent;color:var(--ix-color-primary);width:2.5rem;height:2.5rem;padding:.5rem;border-radius:var(--ix-radius);cursor:pointer;flex:none}.ix-theme .ix-app-icon-button:hover{background:var(--ix-color-surface-2)}.ix-theme .ix-app-icon-button:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}
.ix-theme .ix-menu{display:flex;flex-direction:column;position:absolute;inset-block:0;inset-inline-start:0;width:3.5rem;max-width:100%;background:var(--ix-color-surface-1);border-inline-end:1px solid var(--ix-color-border);padding:.5rem 0;box-sizing:border-box;z-index:32;transition:width 120ms ease-out;min-height:0}.ix-theme .ix-menu[data-expanded=true]{width:16rem;box-shadow:4px 4px 16px #0003}.ix-theme .ix-application[data-menu-pinned=true] .ix-menu{box-shadow:none}
.ix-theme .ix-menu-primary{flex:1;min-height:0;overflow-y:auto;overflow-x:clip;padding-block:.25rem}.ix-theme .ix-menu-bottom{flex:none;padding-block-start:.5rem}.ix-theme .ix-menu-app-name{padding:.75rem;font-size:1rem;overflow:hidden;white-space:nowrap;font-weight:600}.ix-theme .ix-menu:not([data-expanded=true]) .ix-menu-app-name{display:none}
.ix-theme .ix-menu-item{display:flex;align-items:center;position:relative;min-height:3rem;gap:.75rem;width:100%;box-sizing:border-box;border:0;background:transparent;color:var(--ix-color-text);font:inherit;font-size:.875rem;text-align:start;text-decoration:none;padding:.375rem .75rem;cursor:pointer;white-space:nowrap}.ix-theme .ix-menu-item-icon{display:flex;align-items:center;justify-content:center;flex:none;width:2rem;min-height:2rem}.ix-theme .ix-menu-label{min-width:0;overflow:hidden;text-overflow:ellipsis}
.ix-theme .ix-menu-item:hover{background:var(--ix-color-surface-2)}.ix-theme .ix-menu-item-active{background:var(--ix-color-surface-2);color:var(--ix-color-primary)}.ix-theme .ix-menu-item-active:before{content:'';position:absolute;inset-inline-start:0;inset-block:.5rem;width:2px;background:var(--ix-color-primary)}.ix-theme .ix-menu-item:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-3px}.ix-theme .ix-menu-item:disabled,.ix-theme .ix-menu-item[aria-disabled=true]{opacity:.45;cursor:not-allowed}
.ix-theme .ix-menu:not([data-expanded=true])>.ix-menu-primary>.ix-menu-item .ix-menu-label,.ix-theme .ix-menu:not([data-expanded=true])>.ix-menu-bottom>.ix-menu-item .ix-menu-label,.ix-theme .ix-menu:not([data-expanded=true]) .ix-menu-category>button>.ix-menu-label,.ix-theme .ix-menu:not([data-expanded=true]) .ix-menu-category-chevron{display:none}
.ix-theme .ix-menu-notifications{display:inline-flex;align-items:center;justify-content:center;min-width:1rem;height:1rem;padding:0 .125rem;background:var(--ix-color-danger);color:var(--ix-color-on-primary);border-radius:.625rem;font-size:.625rem;margin-inline-start:auto;line-height:1;flex:none}.ix-theme .ix-menu:not([data-expanded=true]) .ix-menu-notifications{position:absolute;inset-inline-end:.25rem;top:.25rem}
.ix-theme .ix-category-backdrop{position:fixed;inset:0;border:0;background:transparent;z-index:39}.ix-theme .ix-menu-category{position:relative}.ix-theme .ix-menu-category-chevron{margin-inline-start:auto}.ix-theme .ix-menu-category-items .ix-menu-item{padding-inline-start:2rem}.ix-theme .ix-menu-category-flyout{position:fixed;margin-inline-start:3.5rem;transform:translateY(-3rem);min-width:14rem;max-width:18rem;background:var(--ix-color-surface-1);border:1px solid var(--ix-color-border);box-shadow:4px 8px 24px #0004;z-index:40;padding:.25rem 0}.ix-theme .ix-menu-category-flyout .ix-menu-label{display:inline!important}.ix-theme .ix-menu-category-flyout .ix-menu-item{padding-inline:.75rem}.ix-theme .ix-menu-category-heading{padding:.75rem;font-size:.875rem;font-weight:600;border-bottom:1px solid var(--ix-color-border)}
.ix-theme .ix-menu-scrim{position:absolute;inset:0;border:0;background:#0002;z-index:31;cursor:default}.ix-theme .ix-menu-overlay{position:absolute;inset-block:0;inset-inline-start:3.5rem;inset-inline-end:0;background:var(--ix-color-surface);z-index:34;overflow:auto}.ix-theme .ix-menu-overlay-panel{padding:2rem;min-height:100%;box-sizing:border-box}.ix-theme .ix-menu-overlay-header{display:flex;align-items:center;justify-content:space-between;gap:1rem;margin-block-end:2rem}.ix-theme .ix-menu-overlay-header h2{margin:0;font-size:1.75rem;line-height:1.25}.ix-theme .ix-menu-overlay-content{max-width:80rem}
.ix-theme .ix-menu-avatar{position:relative}.ix-theme .ix-menu-avatar-trigger{display:flex;align-items:center;gap:.75rem;min-height:3.5rem;width:100%;border:0;background:transparent;color:var(--ix-color-text);font:inherit;text-align:start;padding:.5rem .75rem;cursor:pointer}.ix-theme .ix-menu-avatar-trigger:hover{background:var(--ix-color-surface-2)}.ix-theme .ix-menu-avatar-trigger:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-3px}.ix-theme .ix-menu-avatar-circle{display:flex;align-items:center;justify-content:center;overflow:hidden;width:2rem;height:2rem;border-radius:50%;flex:none;background:var(--ix-color-surface-2);font-size:.75rem;color:var(--ix-color-text)}.ix-theme .ix-menu-avatar-circle img{width:100%;height:100%;object-fit:cover}.ix-theme .ix-menu-avatar-text{display:flex;flex-direction:column;gap:.25rem;font-size:.875rem;overflow:hidden}.ix-theme .ix-menu-avatar-text>span,.ix-theme .ix-menu-avatar-text>small{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.ix-theme .ix-menu-avatar-text>small{font-size:.75rem;color:var(--ix-color-text-secondary)}.ix-theme .ix-menu:not([data-expanded=true]) .ix-menu-avatar-text{display:none}
.ix-theme .ix-avatar-backdrop{position:fixed;inset:0;border:0;background:transparent;z-index:70}.ix-theme .ix-menu-avatar-popup{position:absolute;bottom:100%;inset-inline-start:0;width:16rem;border:1px solid var(--ix-color-border);background:var(--ix-color-surface-1);box-shadow:0 8px 24px #0004;z-index:71;padding:.25rem 0}.ix-theme .ix-menu-avatar-item{display:flex;align-items:center;gap:.75rem;width:100%;border:0;background:transparent;color:var(--ix-color-text);font:inherit;font-size:.875rem;padding:.625rem 1rem;text-align:start;cursor:pointer;min-height:2.5rem}.ix-theme .ix-menu-avatar-item:hover{background:var(--ix-color-surface-2)}.ix-theme .ix-menu-avatar-item:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-2px}.ix-theme .ix-menu-avatar-item:disabled{opacity:.45;cursor:not-allowed}
.ix-theme .ix-pane-layout{position:relative;display:grid;width:100%;height:100%;min-height:16rem;grid-template-columns:auto minmax(0,1fr) auto;grid-template-rows:auto minmax(0,1fr) auto;grid-template-areas:'left top right' 'left content right' 'left bottom right';container:ix-panes / size;isolation:isolate}.ix-theme .ix-pane-layout[data-layout=full-horizontal]{grid-template-areas:'top top top' 'left content right' 'bottom bottom bottom'}.ix-theme .ix-pane-slot{position:relative;min-width:0;min-height:0}.ix-theme .ix-pane-slot-left{grid-area:left}.ix-theme .ix-pane-slot-right{grid-area:right}.ix-theme .ix-pane-slot-top{grid-area:top}.ix-theme .ix-pane-slot-bottom{grid-area:bottom}.ix-theme .ix-pane-layout-content{grid-area:content;min-width:0;min-height:0;overflow:auto}
.ix-theme .ix-pane{display:flex;flex-direction:column;position:relative;background:var(--ix-color-surface-1);min-width:0;box-sizing:border-box;z-index:12;overflow:hidden}.ix-theme .ix-pane[hidden]{display:none}.ix-theme .ix-pane:not([data-borderless=true]){border:1px solid var(--ix-color-border)}.ix-theme .ix-pane-left,.ix-theme .ix-pane-right{width:40px;height:100%}.ix-theme .ix-pane-top,.ix-theme .ix-pane-bottom{height:40px;width:100%}.ix-theme .ix-pane-left[data-expanded=true],.ix-theme .ix-pane-right[data-expanded=true]{width:var(--ix-pane-size,240px)}.ix-theme .ix-pane-top[data-expanded=true],.ix-theme .ix-pane-bottom[data-expanded=true]{height:var(--ix-pane-size,240px)}
.ix-theme .ix-pane-header{display:flex;align-items:center;gap:.25rem;padding:0;flex:none;min-height:40px;box-sizing:border-box}.ix-theme .ix-pane-heading{display:flex;align-items:center;gap:.5rem;overflow:hidden;white-space:nowrap;font-size:.875rem;font-weight:600;min-width:0}.ix-theme .ix-pane-heading>span{overflow:hidden;text-overflow:ellipsis}.ix-theme .ix-pane-left:not([data-expanded=true]) .ix-pane-header,.ix-theme .ix-pane-right:not([data-expanded=true]) .ix-pane-header{flex-direction:column;min-height:100%;align-items:center}.ix-theme .ix-pane-left:not([data-expanded=true]) .ix-pane-heading,.ix-theme .ix-pane-right:not([data-expanded=true]) .ix-pane-heading{writing-mode:vertical-rl;overflow:visible;padding-block:.5rem}.ix-theme .ix-pane-content{padding:1rem;overflow:auto;min-height:0;flex:1}.ix-theme .ix-pane-content[hidden]{display:none}.ix-theme .ix-pane-content[data-no-padding=true]{padding-inline:0;padding-block-end:0}
.ix-theme .ix-pane-slot:has(>.ix-pane[data-floating=true]):is(.ix-pane-slot-left,.ix-pane-slot-right){width:40px}.ix-theme .ix-pane-slot:has(>.ix-pane[data-floating=true]):is(.ix-pane-slot-top,.ix-pane-slot-bottom){height:40px}.ix-theme .ix-pane-slot:has(>.ix-pane[data-hide-on-collapse=true]:is([hidden],[data-floating=true])):is(.ix-pane-slot-left,.ix-pane-slot-right){width:0}.ix-theme .ix-pane-slot:has(>.ix-pane[data-hide-on-collapse=true]:is([hidden],[data-floating=true])):is(.ix-pane-slot-top,.ix-pane-slot-bottom){height:0}.ix-theme .ix-pane[data-floating=true][data-expanded=true]{position:absolute;z-index:52;box-shadow:0 8px 24px #0004}.ix-theme .ix-pane-left[data-floating=true]{inset-inline-start:0;top:0}.ix-theme .ix-pane-right[data-floating=true]{inset-inline-end:0;top:0}.ix-theme .ix-pane-top[data-floating=true]{top:0;inset-inline:0}.ix-theme .ix-pane-bottom[data-floating=true]{bottom:0;inset-inline:0}
.ix-theme .ix-pane[data-mobile=true]{width:100%;height:48px;max-width:none}.ix-theme .ix-pane[data-mobile=true][data-expanded=true]{position:absolute;inset-inline:0;height:100%;z-index:54}.ix-theme .ix-pane-left[data-mobile=true][data-expanded=true],.ix-theme .ix-pane-top[data-mobile=true][data-expanded=true]{top:0;bottom:auto}.ix-theme .ix-pane-right[data-mobile=true][data-expanded=true],.ix-theme .ix-pane-bottom[data-mobile=true][data-expanded=true]{bottom:0;top:auto}.ix-theme .ix-pane[data-mobile=true] .ix-pane-header{flex-direction:row-reverse;justify-content:space-between;min-height:48px;padding-inline:.25rem}.ix-theme .ix-pane[data-mobile=true]:not([data-expanded=true]) .ix-pane-header{min-height:48px}.ix-theme .ix-pane[data-mobile=true] .ix-pane-heading{writing-mode:horizontal-tb;padding-block:0;overflow:hidden}
.ix-theme .ix-layout-grid{width:100%;box-sizing:border-box;padding-inline:calc(var(--ix-grid-gap,24px)*.5);container:ix-grid / inline-size}.ix-theme .ix-layout-grid[data-no-margin=true]{padding-inline:0}.ix-theme .ix-layout-row{display:flex;flex-wrap:wrap;margin:calc(var(--ix-grid-gap,24px)*-.5)}.ix-theme .ix-layout-col{box-sizing:border-box;flex:var(--ix-col-grow,1) 0 var(--ix-col-basis,0%);min-width:auto;padding:calc(var(--ix-grid-gap,24px)*.5);max-width:100%}
@container ix-grid (max-width:992px){.ix-theme .ix-layout-col{flex-grow:var(--ix-col-grow-lg,1);flex-basis:var(--ix-col-lg,var(--ix-col-basis,0%))}}@container ix-grid (max-width:768px){.ix-theme .ix-layout-col{flex-grow:var(--ix-col-grow-md,1);flex-basis:var(--ix-col-md,var(--ix-col-basis,0%))}}@container ix-grid (max-width:576px){.ix-theme .ix-layout-col{flex-grow:var(--ix-col-grow-sm,1);flex-basis:var(--ix-col-sm,var(--ix-col-basis,0%))}}
.ix-theme .ix-app-small .ix-application-menu-slot{width:0}.ix-theme .ix-app-small .ix-menu:not([data-expanded=true]){display:none}.ix-theme .ix-app-small .ix-application-sidebar{display:none}.ix-theme .ix-app-small .ix-application-app-icon,.ix-theme .ix-app-small .ix-application-name-suffix{display:none}.ix-theme .ix-app-small .ix-menu-overlay{inset-inline-start:0}
@container ix-app (max-width:576px){.ix-theme .ix-app-auto .ix-application-menu-slot{width:0}.ix-theme .ix-app-auto .ix-menu:not([data-expanded=true]){display:none}.ix-theme .ix-app-auto .ix-application-sidebar{display:none}.ix-theme .ix-app-auto .ix-application-app-icon,.ix-theme .ix-app-auto .ix-application-name-suffix{display:none}.ix-theme .ix-app-auto .ix-menu-overlay{inset-inline-start:0}.ix-theme .ix-application-header{gap:.25rem}.ix-theme .ix-menu-overlay-panel{padding:1rem}}
@media(max-width:48em){.ix-theme .ix-pane-layout,.ix-theme .ix-pane-layout[data-layout=full-horizontal]{grid-template-columns:minmax(0,1fr);grid-template-rows:auto auto minmax(0,1fr) auto auto;grid-template-areas:'top' 'left' 'content' 'right' 'bottom'}.ix-theme .ix-pane-slot{position:static}.ix-theme .ix-pane-slot:is(.ix-pane-slot-left,.ix-pane-slot-right,.ix-pane-slot-top,.ix-pane-slot-bottom){width:auto;height:auto;min-height:48px}.ix-theme .ix-pane-slot:has(>.ix-pane[data-hide-on-collapse=true]){min-height:0}}
@media(prefers-reduced-motion:reduce){.ix-theme .ix-menu{transition:none}}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn menu_open_veto_preserves_current_overlay_for_open_switch_and_toggle_close() {
        fn probe() -> Element {
            for (current, requested, opening) in [
                (None, MenuOverlayKind::Settings, true),
                (
                    Some(MenuOverlayKind::Settings),
                    MenuOverlayKind::About,
                    true,
                ),
                (Some(MenuOverlayKind::About), MenuOverlayKind::About, false),
            ] {
                let veto = EventHandler::new(move |event: MenuOpenEvent| {
                    assert_eq!(event.overlay, requested);
                    assert_eq!(event.opening, opening);
                    assert!(!event.default_prevented());
                    let retained = event.clone();
                    event.prevent_default();
                    assert!(retained.default_prevented());
                });
                assert_eq!(request_menu_overlay(current, requested, veto), None);
            }
            rsx! { div { "Menu veto verified" } }
        }
        let mut dom = VirtualDom::new(probe);
        dom.rebuild_in_place();
        assert!(dioxus_ssr::render(&dom).contains("Menu veto verified"));
    }

    #[test]
    fn menu_accepted_requests_distinguish_opening_switching_and_closing() {
        fn probe() -> Element {
            let accept = EventHandler::new(|event: MenuOpenEvent| {
                assert!(!event.default_prevented());
            });
            assert_eq!(
                request_menu_overlay(None, MenuOverlayKind::Settings, accept),
                Some(Some(MenuOverlayKind::Settings))
            );
            assert_eq!(
                request_menu_overlay(
                    Some(MenuOverlayKind::Settings),
                    MenuOverlayKind::About,
                    accept
                ),
                Some(Some(MenuOverlayKind::About))
            );
            assert_eq!(
                request_menu_overlay(
                    Some(MenuOverlayKind::Settings),
                    MenuOverlayKind::Settings,
                    accept
                ),
                Some(None)
            );
            rsx! { div { "Menu transitions verified" } }
        }
        let mut dom = VirtualDom::new(probe);
        dom.rebuild_in_place();
        assert!(dioxus_ssr::render(&dom).contains("Menu transitions verified"));
    }

    #[test]
    fn pane_size_uses_the_upstream_size_set() {
        assert_eq!(pane_size("480px"), "480px");
        assert_eq!(pane_size("33%"), "33%");
        assert_eq!(pane_size("calc(100% - 1px)"), "320px");
        assert_eq!(pane_size("66%"), "320px");
    }

    #[test]
    fn pane_sizes_track_parent_dimensions_without_consuming_more_than_half() {
        assert_eq!(pane_expanded_size("600px", 720.0), 360.0);
        assert_eq!(pane_expanded_size("240px", 1400.0), 240.0);
        assert_eq!(pane_expanded_size("50%", 719.0), 360.0);
        assert_eq!(pane_expanded_size("33%", 719.0), 240.0);
        assert_eq!(pane_expanded_size("480px", 0.0), 480.0);
        assert_eq!(pane_expanded_size("invalid", 400.0), 200.0);
    }

    #[test]
    fn pane_focus_wraps_both_directions_and_recovers_removed_controls() {
        assert_eq!(pane_focus_index(Some(2), 3, false), Some(0));
        assert_eq!(pane_focus_index(Some(0), 3, true), Some(2));
        assert_eq!(pane_focus_index(None, 3, false), Some(0));
        assert_eq!(pane_focus_index(None, 3, true), Some(2));
        assert_eq!(pane_focus_index(Some(8), 2, false), Some(0));
        assert_eq!(pane_focus_index(Some(0), 0, false), None);
    }

    #[test]
    fn pane_request_veto_preserves_state_and_reports_layout_slot() {
        fn probe() -> Element {
            let rejected = EventHandler::new(|event: PaneExpandedChangeEvent| {
                assert_eq!(event.slot, "left");
                assert!(!event.expanded);
                let retained = event.clone();
                event.prevent_default();
                assert!(retained.default_prevented());
            });
            assert!(!request_pane_expansion(
                true,
                false,
                "left".into(),
                rejected
            ));
            let accepted = EventHandler::new(|event: PaneExpandedChangeEvent| {
                assert!(event.slot.is_empty());
                assert!(event.expanded);
                assert!(!event.default_prevented());
            });
            assert!(request_pane_expansion(false, true, String::new(), accepted));
            let must_not_run = EventHandler::new(|_: PaneExpandedChangeEvent| -> () {
                panic!("unchanged state is not an expansion request")
            });
            assert!(!request_pane_expansion(
                true,
                true,
                String::new(),
                must_not_run
            ));
            rsx! { div { "Pane requests verified" } }
        }
        let mut dom = VirtualDom::new(probe);
        dom.rebuild_in_place();
        assert!(dioxus_ssr::render(&dom).contains("Pane requests verified"));
    }

    #[test]
    fn initial_menu_overlay_and_controlled_key_select_the_expected_content() {
        fn probe() -> Element {
            rsx! { Application {
                menu: rsx! { Menu { initial_overlay: MenuOverlayKind::About,
                    about: rsx! { MenuAbout { crate::TabSet { items: vec![crate::TabItem::new("one","One"), crate::TabItem::new("two","Two")], active_key: "two",
                        crate::TabPanel { tab_key: "one", "First panel" }
                        crate::TabPanel { tab_key: "two", "Second panel" }
                    } } }
                } }
            } }
        }
        let mut dom = VirtualDom::new(probe);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("role=\"dialog\""));
        assert!(html.contains("legal information"));
        assert!(html.contains("Second panel"));
        assert!(html.contains("aria-selected=\"true\"") || html.contains("aria-selected=true"));
        assert!(
            html.contains("hidden"),
            "inactive panel remains mounted but hidden"
        );
    }

    #[test]
    fn application_switch_uses_native_dialog_and_real_navigation_links() {
        fn probe() -> Element {
            rsx! { Application {
                app_switch_config: AppSwitchConfig { current_app_id: "chosen".into(), apps: vec![AppSwitchEntry { id: "chosen".into(), name: "Example app".into(), description: "Example description".into(), icon_src: "data:image/svg+xml,%3Csvg/%3E".into(), url: "https://example.com/".into(), target: "_blank".into() }], ..Default::default() },
                header: rsx! { ApplicationHeader {} }
            } }
        }
        let mut dom = VirtualDom::new(probe);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("command=\"show-modal\""));
        assert!(html.contains("<dialog"));
        assert!(html.contains("href=\"https://example.com/\""));
        assert!(html.contains("target=\"_blank\""));
        assert!(html.contains("rel=\"noopener noreferrer\""));
        assert!(html.contains("data-selected=\"true\""));
        assert!(html.contains("aria-current=\"page\""));
    }
}
