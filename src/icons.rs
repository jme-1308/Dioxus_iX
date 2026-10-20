//! Industrial pictograms from @siemens/ix-icons 3.7.0, embedded as Rust data.
//!
//! Copyright (c) 2022 Siemens AG. SPDX-License-Identifier: MIT.
//! The upstream MIT notice is reproduced in THIRD_PARTY_NOTICES.md. Logo/brand pictograms
//! are deliberately excluded. No external asset loading or JS runtime is used.

use dioxus::prelude::*;
use std::collections::BTreeMap;

/// Application-authored SVG geometry and its coordinate system.
///
/// `geometry` is trusted SVG markup inserted inside the icon's outer `<svg>`.
/// It can contain paths, groups, gradients, or a complete nested SVG. Supplying
/// the original `viewBox` preserves icons whose dimensions differ from 24×24.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IconDefinition {
    pub view_box: String,
    pub geometry: String,
}

impl IconDefinition {
    pub fn new(view_box: impl Into<String>, geometry: impl Into<String>) -> Self {
        Self {
            view_box: view_box.into(),
            geometry: geometry.into(),
        }
    }

    /// Copy a bundled icon into a definition suitable for scoped registration.
    pub fn bundled(name: &str) -> Option<Self> {
        icon_svg(name).map(|geometry| Self::new("0 0 24 24", geometry))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RegisteredIcons {
    definitions: BTreeMap<String, IconDefinition>,
    aliases: BTreeMap<String, String>,
}

impl RegisteredIcons {
    fn get(&self, name: &str) -> Option<&IconDefinition> {
        self.aliases
            .get(name)
            .and_then(|key| self.definitions.get(key))
    }

    fn register(&mut self, name: &str, definition: IconDefinition) -> Option<IconDefinition> {
        let kebab = registration_alias(name);
        let key = self
            .aliases
            .get(name)
            .or_else(|| self.aliases.get(&kebab))
            .cloned()
            .unwrap_or_else(|| name.to_string());
        let previous = self.definitions.insert(key.clone(), definition);
        self.aliases.insert(name.to_string(), key.clone());
        self.aliases.insert(kebab, key);
        // Alias collisions can transfer the final name of an older entry.
        // Retain only definitions reachable through at least one name.
        self.definitions
            .retain(|key, _| self.aliases.values().any(|value| value == key));
        previous
    }

    fn remove(&mut self, name: &str) -> Option<IconDefinition> {
        let key = self.aliases.get(name)?.clone();
        self.aliases.retain(|_, target| target != &key);
        self.definitions.remove(&key)
    }
}

// Matches the aliases produced by ix-icons addIconToCache, including digit runs
// and export names containing initialisms. The original spelling is also kept.
fn registration_alias(name: &str) -> String {
    let characters: Vec<_> = name.chars().collect();
    let mut result = String::with_capacity(name.len());
    let mut index = 0;
    while let Some(&character) = characters.get(index) {
        if (character.is_ascii_lowercase() || character.is_ascii_digit())
            && characters
                .get(index + 1)
                .is_some_and(|next| next.is_ascii_uppercase() || next.is_ascii_digit())
        {
            result.push(character);
            result.push('-');
            result.push(characters[index + 1].to_ascii_lowercase());
            index += 2;
        } else {
            if character.is_ascii_uppercase() {
                result.push('-');
            }
            result.push(character.to_ascii_lowercase());
            index += 1;
        }
    }
    result
}

fn registration_name(name: &str) -> String {
    match name.strip_prefix("icon") {
        Some(name) => {
            let mut characters = name.chars();
            characters
                .next()
                .map(|first| first.to_ascii_lowercase().to_string() + characters.as_str())
                .unwrap_or_default()
        }
        None => name.to_string(),
    }
}

/// Reactive custom icon collection owned by a Dioxus component scope.
///
/// Install it with [`IconRegistryProvider`]. Descendant icons use this registry
/// before the bundled icons. Nested providers isolate their custom names;
/// registration does not mutate process globals or leak SVG strings.
/// The handle remains valid while the component that created it is mounted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IconRegistry {
    entries: Signal<RegisteredIcons>,
}

/// Create an independent registry. Share it through [`IconRegistryProvider`].
///
/// ```rust,no_run
/// use dioxus::prelude::*;
/// use dioxus_ix::icons::{Icon, IconDefinition, IconRegistryProvider, use_icon_registry};
///
/// fn app() -> Element {
///     let registry = use_icon_registry();
///     use_hook(move || registry.add_icons([
///         ("iconCustomStatus", IconDefinition::new("0 0 32 32", "<circle cx='16' cy='16' r='12'/>")),
///     ]));
///     rsx! { IconRegistryProvider { registry,
///         Icon { name: "custom-status", label: "Machine status" }
///     } }
/// }
/// ```
pub fn use_icon_registry() -> IconRegistry {
    IconRegistry {
        entries: use_signal(RegisteredIcons::default),
    }
}

impl IconRegistry {
    fn update<R>(mut self, update: impl FnOnce(&mut RegisteredIcons) -> R) -> R {
        let mut next = self.entries.peek().clone();
        let result = update(&mut next);
        if *self.entries.peek() != next {
            self.entries.set(next);
        }
        result
    }

    /// Register a name and its camelCase/kebab-case alias. Return its previous
    /// definition, if any. Repeating an identical registration is a no-op.
    pub fn register(self, name: &str, definition: IconDefinition) -> Option<IconDefinition> {
        self.update(|entries| entries.register(name, definition))
    }

    /// Register exported icons, following upstream `addIcons` name semantics.
    /// For example, `iconStarFilled` registers `starFilled` and `star-filled`.
    pub fn add_icons<N: AsRef<str>>(self, icons: impl IntoIterator<Item = (N, IconDefinition)>) {
        self.update(|entries| {
            for (name, definition) in icons {
                entries.register(&registration_name(name.as_ref()), definition);
            }
        });
    }

    /// Remove a custom definition and all its aliases. A matching bundled icon
    /// becomes visible again; custom-only names become empty.
    pub fn remove(self, name: &str) -> Option<IconDefinition> {
        self.update(|entries| entries.remove(name))
    }

    pub fn clear(self) {
        self.update(|entries| *entries = RegisteredIcons::default());
    }

    /// Read a custom definition reactively. Bundled fallback is not included.
    pub fn get(self, name: &str) -> Option<IconDefinition> {
        self.entries.read().get(name).cloned()
    }

    /// Number of custom definitions, excluding aliases and bundled icons.
    pub fn len(self) -> usize {
        self.entries.read().definitions.len()
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
}

#[derive(Clone, Copy)]
struct IconRegistryContext(Signal<IconRegistry>);

/// Make a custom registry available to descendant [`Icon`] components.
/// Changing the supplied registry switches descendants to the new collection.
#[component]
pub fn IconRegistryProvider(registry: IconRegistry, children: Element) -> Element {
    let mut current = use_signal(|| registry);
    if *current.peek() != registry {
        current.set(registry);
    }
    use_context_provider(|| IconRegistryContext(current));
    children
}

/// Names of all bundled, non-logo upstream icons, in sorted order.
pub fn icon_names() -> impl ExactSizeIterator<Item = &'static str> {
    ICONS.iter().map(|(name, _)| *name)
}

/// Look up static SVG geometry by an upstream kebab-case or camelCase name.
///
/// This only reads the bundled catalog. For reactive custom registration use
/// [`IconRegistry`]. `starFilled` and `star-filled` resolve to the same geometry.
pub fn icon_svg(name: &str) -> Option<&'static str> {
    let normalized;
    let name = if name.bytes().any(|character| character.is_ascii_uppercase()) {
        normalized = normalize_icon_name(name);
        normalized.as_str()
    } else {
        name
    };
    ICONS
        .binary_search_by_key(&name, |(key, _)| *key)
        .ok()
        .map(|i| ICONS[i].1)
}

fn normalize_icon_name(name: &str) -> String {
    let mut normalized = String::with_capacity(name.len());
    for character in name.chars() {
        if character.is_ascii_uppercase() {
            if !normalized.is_empty() && !normalized.ends_with('-') {
                normalized.push('-');
            }
            normalized.push(character.to_ascii_lowercase());
        } else {
            normalized.push(character);
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn bundled_catalog_is_sorted_and_excludes_company_marks() {
        let names: Vec<_> = icon_names().collect();
        assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
        for excluded in ["logoSiemens", "youtube", "youtube-filled", "youtubeFilled"] {
            assert!(icon_svg(excluded).is_none(), "bundled mark: {excluded}");
            assert!(IconDefinition::bundled(excluded).is_none());
        }
    }

    #[test]
    fn camel_case_aliases_resolve_geometry_and_unknown_names_remain_absent() {
        assert!(icon_svg("star").is_some());
        assert!(icon_svg("starFilled").is_some());
        assert_eq!(icon_svg("starFilled"), icon_svg("star-filled"));
        assert_ne!(icon_svg("star"), icon_svg("starFilled"));
        assert!(icon_svg("missingCustomIcon").is_none());
        assert!(icon_svg("logoSiemens").is_none());
    }

    fn definition(phase: &str) -> IconDefinition {
        IconDefinition::new(
            "0 0 64 32",
            format!("<path data-phase='{phase}' d='M0 0h64v32H0z'/>"),
        )
    }

    #[test]
    fn registration_preserves_export_aliases_and_replaces_or_removes_one_definition() {
        let mut icons = RegisteredIcons::default();
        let exported = registration_name("iconDevice32Bit");
        assert_eq!(exported, "device32Bit");
        icons.register(&exported, definition("original"));
        assert_eq!(icons.get("device32Bit"), icons.get("device-32-bit"));
        assert!(icons.get("device32Bit").is_some());
        icons.register("device-32-bit", definition("replacement"));
        assert_eq!(icons.definitions.len(), 1);
        assert_eq!(icons.get("device32Bit"), Some(&definition("replacement")));
        icons.register(&registration_name("iconXML"), definition("initialism"));
        assert_eq!(icons.get("xML"), icons.get("x-m-l"));
        assert_eq!(icons.definitions.len(), 2);
        assert_eq!(icons.remove("device32Bit"), Some(definition("replacement")));
        assert!(icons.get("device-32-bit").is_none());
        assert!(icons.remove("device-32-bit").is_none());
    }

    fn runtime_fixture() -> Element {
        let registry = use_icon_registry();
        let capture = consume_context::<Rc<RefCell<Option<IconRegistry>>>>();
        *capture.borrow_mut() = Some(registry);
        rsx! { IconRegistryProvider { registry,
            Icon { name: "customFlag", label: "Custom flag" }
            Icon { name: "star", label: "Bundled star" }
        } }
    }

    #[test]
    fn mounted_unknown_icons_appear_replace_and_remove_without_remounting() {
        let capture = Rc::new(RefCell::new(None::<IconRegistry>));
        let mut dom = VirtualDom::new(runtime_fixture).with_root_context(capture.clone());
        dom.rebuild_in_place();
        let registry = capture.borrow().unwrap();
        let original = dioxus_ssr::render(&dom);
        assert!(!original.contains("Custom flag"));
        assert!(original.contains("Bundled star"));
        dom.in_scope(ScopeId::ROOT, || {
            registry.add_icons([("iconCustomFlag", definition("registered"))])
        });
        dom.render_immediate_to_vec();
        let registered = dioxus_ssr::render(&dom);
        assert!(registered.contains("Custom flag"));
        assert!(registered.contains("viewBox=\"0 0 64 32\""));
        assert!(registered.contains("data-phase='registered'"));
        dom.in_scope(ScopeId::ROOT, || {
            assert_eq!(
                registry.register("custom-flag", definition("updated")),
                Some(definition("registered"))
            );
            registry.register("star", definition("overridden"));
        });
        dom.render_immediate_to_vec();
        let updated = dioxus_ssr::render(&dom);
        assert!(updated.contains("data-phase='updated'"));
        assert!(updated.contains("data-phase='overridden'"));
        assert!(!updated.contains("data-phase='registered'"));
        dom.in_scope(ScopeId::ROOT, || {
            registry.remove("customFlag");
            registry.remove("star");
        });
        dom.render_immediate_to_vec();
        let removed = dioxus_ssr::render(&dom);
        assert_eq!(removed, original);
    }

    #[derive(Clone, Copy)]
    struct ScopeCapture {
        first: IconRegistry,
        second: IconRegistry,
        alternate: Signal<bool>,
    }

    fn scoped_fixture() -> Element {
        let first = use_icon_registry();
        let second = use_icon_registry();
        let alternate = use_signal(|| false);
        use_hook(move || {
            first.register("customScoped", definition("first"));
            second.register("customScoped", definition("second"));
        });
        let capture = consume_context::<Rc<RefCell<Option<ScopeCapture>>>>();
        *capture.borrow_mut() = Some(ScopeCapture {
            first,
            second,
            alternate,
        });
        rsx! {
            IconRegistryProvider { registry: if alternate() { second } else { first },
                Icon { name: "custom-scoped", label: "Outer scope" }
                IconRegistryProvider { registry: second,
                    Icon { name: "customScoped", label: "Inner scope" }
                }
            }
            Icon { name: "customScoped", label: "Outside scope" }
        }
    }

    #[test]
    fn nested_registries_are_isolated_and_provider_switches_update_existing_icons() {
        let capture = Rc::new(RefCell::new(None::<ScopeCapture>));
        let mut dom = VirtualDom::new(scoped_fixture).with_root_context(capture.clone());
        dom.rebuild_in_place();
        let mut controllers = capture.borrow().unwrap();
        let initial = dioxus_ssr::render(&dom);
        assert_eq!(initial.matches("data-phase='first'").count(), 1);
        assert_eq!(initial.matches("data-phase='second'").count(), 1);
        assert!(!initial.contains("Outside scope"));
        dom.in_scope(ScopeId::ROOT, || controllers.first.clear());
        dom.render_immediate_to_vec();
        let cleared = dioxus_ssr::render(&dom);
        assert!(!cleared.contains("Outer scope"));
        assert!(cleared.contains("Inner scope"));
        assert_eq!(cleared.matches("data-phase='second'").count(), 1);
        dom.in_scope(ScopeId::ROOT, || controllers.alternate.set(true));
        dom.render_immediate_to_vec();
        let switched = dioxus_ssr::render(&dom);
        assert!(switched.contains("Outer scope"));
        assert_eq!(switched.matches("data-phase='second'").count(), 2);
        dom.in_scope(ScopeId::ROOT, || {
            controllers
                .second
                .register("customScoped", definition("both updated"));
        });
        dom.render_immediate_to_vec();
        assert_eq!(
            dioxus_ssr::render(&dom)
                .matches("data-phase='both updated'")
                .count(),
            2
        );
    }
}

/// An upstream iX pictogram. Unknown names render no icon.
///
/// Provide `label` for a meaningful standalone icon. Leave it unset when
/// adjacent text or the containing button already supplies an accessible name.
#[component]
pub fn Icon(
    name: String,
    #[props(default = 24)] size: u32,
    #[props(default)] label: Option<String>,
    #[props(default)] class: String,
) -> Element {
    let registry = try_use_context::<IconRegistryContext>();
    let custom = registry.and_then(|context| context.0.read().get(&name));
    let geometry = custom
        .as_ref()
        .map(|definition| definition.geometry.as_str())
        .or_else(|| icon_svg(&name));
    let Some(geometry) = geometry else {
        return VNode::empty();
    };
    let view_box = custom
        .as_ref()
        .map_or("0 0 24 24", |definition| definition.view_box.as_str());
    let decorative = label.is_none();
    rsx! {
        svg {
            class: "ix-icon {class}",
            width: size,
            height: size,
            view_box,
            fill: "currentColor",
            role: if decorative { "presentation" } else { "img" },
            "aria-hidden": if decorative { "true" } else { "false" },
            "aria-label": label,
            "focusable": "false",
            dangerous_inner_html: geometry,
        }
    }
}

const ICONS: &[(&str, &str)] = &[
    (
        "aas-project-ax",
        include_str!("icons/bundled/aas-project-ax.svg"),
    ),
    (
        "aas-project-ax-filled",
        include_str!("icons/bundled/aas-project-ax-filled.svg"),
    ),
    (
        "aas-project-c",
        include_str!("icons/bundled/aas-project-c.svg"),
    ),
    (
        "aas-project-c-filled",
        include_str!("icons/bundled/aas-project-c-filled.svg"),
    ),
    (
        "aas-project-empty",
        include_str!("icons/bundled/aas-project-empty.svg"),
    ),
    (
        "aas-project-generic",
        include_str!("icons/bundled/aas-project-generic.svg"),
    ),
    (
        "aas-project-generic-filled",
        include_str!("icons/bundled/aas-project-generic-filled.svg"),
    ),
    (
        "aas-project-tc",
        include_str!("icons/bundled/aas-project-tc.svg"),
    ),
    (
        "aas-project-tc-filled",
        include_str!("icons/bundled/aas-project-tc-filled.svg"),
    ),
    (
        "aas-project-tcx",
        include_str!("icons/bundled/aas-project-tcx.svg"),
    ),
    (
        "aas-project-tcx-filled",
        include_str!("icons/bundled/aas-project-tcx-filled.svg"),
    ),
    (
        "aas-project-tia",
        include_str!("icons/bundled/aas-project-tia.svg"),
    ),
    (
        "aas-project-tia-filled",
        include_str!("icons/bundled/aas-project-tia-filled.svg"),
    ),
    ("aas-shell", include_str!("icons/bundled/aas-shell.svg")),
    (
        "aas-shell-check-error",
        include_str!("icons/bundled/aas-shell-check-error.svg"),
    ),
    (
        "aas-shell-check-error-filled",
        include_str!("icons/bundled/aas-shell-check-error-filled.svg"),
    ),
    (
        "aas-shell-check-ok",
        include_str!("icons/bundled/aas-shell-check-ok.svg"),
    ),
    (
        "aas-shell-check-ok-filled",
        include_str!("icons/bundled/aas-shell-check-ok-filled.svg"),
    ),
    (
        "aas-shell-check-warning",
        include_str!("icons/bundled/aas-shell-check-warning.svg"),
    ),
    (
        "aas-shell-check-warning-filled",
        include_str!("icons/bundled/aas-shell-check-warning-filled.svg"),
    ),
    (
        "aas-shell-export",
        include_str!("icons/bundled/aas-shell-export.svg"),
    ),
    (
        "aas-shell-export-filled",
        include_str!("icons/bundled/aas-shell-export-filled.svg"),
    ),
    (
        "aas-shell-filled",
        include_str!("icons/bundled/aas-shell-filled.svg"),
    ),
    (
        "aas-shell-import",
        include_str!("icons/bundled/aas-shell-import.svg"),
    ),
    (
        "aas-shell-import-filled",
        include_str!("icons/bundled/aas-shell-import-filled.svg"),
    ),
    (
        "aas-shell-search",
        include_str!("icons/bundled/aas-shell-search.svg"),
    ),
    (
        "aas-shell-search-filled",
        include_str!("icons/bundled/aas-shell-search-filled.svg"),
    ),
    ("about", include_str!("icons/bundled/about.svg")),
    (
        "about-filled",
        include_str!("icons/bundled/about-filled.svg"),
    ),
    ("add", include_str!("icons/bundled/add.svg")),
    (
        "add-application",
        include_str!("icons/bundled/add-application.svg"),
    ),
    ("add-circle", include_str!("icons/bundled/add-circle.svg")),
    (
        "add-circle-filled",
        include_str!("icons/bundled/add-circle-filled.svg"),
    ),
    (
        "add-circle-small",
        include_str!("icons/bundled/add-circle-small.svg"),
    ),
    (
        "add-circle-small-filled",
        include_str!("icons/bundled/add-circle-small-filled.svg"),
    ),
    (
        "add-document-note",
        include_str!("icons/bundled/add-document-note.svg"),
    ),
    ("add-eye", include_str!("icons/bundled/add-eye.svg")),
    (
        "add-eye-filled",
        include_str!("icons/bundled/add-eye-filled.svg"),
    ),
    ("add-filter", include_str!("icons/bundled/add-filter.svg")),
    (
        "add-filter-filled",
        include_str!("icons/bundled/add-filter-filled.svg"),
    ),
    (
        "add-selection",
        include_str!("icons/bundled/add-selection.svg"),
    ),
    (
        "add-shield-half",
        include_str!("icons/bundled/add-shield-half.svg"),
    ),
    ("add-task", include_str!("icons/bundled/add-task.svg")),
    (
        "add-task-list",
        include_str!("icons/bundled/add-task-list.svg"),
    ),
    ("add-user", include_str!("icons/bundled/add-user.svg")),
    (
        "add-user-filled",
        include_str!("icons/bundled/add-user-filled.svg"),
    ),
    ("agent", include_str!("icons/bundled/agent.svg")),
    (
        "agent-filled",
        include_str!("icons/bundled/agent-filled.svg"),
    ),
    ("ai", include_str!("icons/bundled/ai.svg")),
    ("ai-agent", include_str!("icons/bundled/ai-agent.svg")),
    ("ai-skill", include_str!("icons/bundled/ai-skill.svg")),
    (
        "ai-sub-agent",
        include_str!("icons/bundled/ai-sub-agent.svg"),
    ),
    ("alarm", include_str!("icons/bundled/alarm.svg")),
    ("alarm-bell", include_str!("icons/bundled/alarm-bell.svg")),
    (
        "alarm-bell-cancelled",
        include_str!("icons/bundled/alarm-bell-cancelled.svg"),
    ),
    (
        "alarm-bell-cancelled-filled",
        include_str!("icons/bundled/alarm-bell-cancelled-filled.svg"),
    ),
    (
        "alarm-bell-filled",
        include_str!("icons/bundled/alarm-bell-filled.svg"),
    ),
    ("alarm-clock", include_str!("icons/bundled/alarm-clock.svg")),
    (
        "alarm-clock-cancelled",
        include_str!("icons/bundled/alarm-clock-cancelled.svg"),
    ),
    (
        "alarm-clock-filled",
        include_str!("icons/bundled/alarm-clock-filled.svg"),
    ),
    (
        "alarm-clock-success",
        include_str!("icons/bundled/alarm-clock-success.svg"),
    ),
    (
        "alarm-filled",
        include_str!("icons/bundled/alarm-filled.svg"),
    ),
    (
        "alarm-multiple",
        include_str!("icons/bundled/alarm-multiple.svg"),
    ),
    (
        "alarm-multiple-filled",
        include_str!("icons/bundled/alarm-multiple-filled.svg"),
    ),
    (
        "align-center-horizontally",
        include_str!("icons/bundled/align-center-horizontally.svg"),
    ),
    (
        "align-center-vertically",
        include_str!("icons/bundled/align-center-vertically.svg"),
    ),
    (
        "align-object-dimensions",
        include_str!("icons/bundled/align-object-dimensions.svg"),
    ),
    (
        "align-object-height",
        include_str!("icons/bundled/align-object-height.svg"),
    ),
    (
        "align-object-width",
        include_str!("icons/bundled/align-object-width.svg"),
    ),
    (
        "align-objects",
        include_str!("icons/bundled/align-objects.svg"),
    ),
    (
        "align-objects-bottom",
        include_str!("icons/bundled/align-objects-bottom.svg"),
    ),
    (
        "align-objects-centered",
        include_str!("icons/bundled/align-objects-centered.svg"),
    ),
    (
        "align-objects-horizontally",
        include_str!("icons/bundled/align-objects-horizontally.svg"),
    ),
    (
        "align-objects-left",
        include_str!("icons/bundled/align-objects-left.svg"),
    ),
    (
        "align-objects-right",
        include_str!("icons/bundled/align-objects-right.svg"),
    ),
    (
        "align-objects-top",
        include_str!("icons/bundled/align-objects-top.svg"),
    ),
    (
        "align-objects-vertically",
        include_str!("icons/bundled/align-objects-vertically.svg"),
    ),
    ("analysis", include_str!("icons/bundled/analysis.svg")),
    (
        "analysis-filled",
        include_str!("icons/bundled/analysis-filled.svg"),
    ),
    ("analyze", include_str!("icons/bundled/analyze.svg")),
    ("anomaly", include_str!("icons/bundled/anomaly.svg")),
    (
        "anomaly-found",
        include_str!("icons/bundled/anomaly-found.svg"),
    ),
    (
        "anomaly-none",
        include_str!("icons/bundled/anomaly-none.svg"),
    ),
    (
        "app-document",
        include_str!("icons/bundled/app-document.svg"),
    ),
    (
        "app-document-filled",
        include_str!("icons/bundled/app-document-filled.svg"),
    ),
    ("app-menu", include_str!("icons/bundled/app-menu.svg")),
    ("app-update", include_str!("icons/bundled/app-update.svg")),
    (
        "application-screen",
        include_str!("icons/bundled/application-screen.svg"),
    ),
    (
        "application-screen-alarm-classes",
        include_str!("icons/bundled/application-screen-alarm-classes.svg"),
    ),
    (
        "application-screen-globe",
        include_str!("icons/bundled/application-screen-globe.svg"),
    ),
    (
        "application-screen-play",
        include_str!("icons/bundled/application-screen-play.svg"),
    ),
    (
        "application-screens",
        include_str!("icons/bundled/application-screens.svg"),
    ),
    (
        "applications",
        include_str!("icons/bundled/applications.svg"),
    ),
    ("apps", include_str!("icons/bundled/apps.svg")),
    (
        "archive-document",
        include_str!("icons/bundled/archive-document.svg"),
    ),
    (
        "archive-document-filled",
        include_str!("icons/bundled/archive-document-filled.svg"),
    ),
    ("areachart", include_str!("icons/bundled/areachart.svg")),
    (
        "arrow-diagonal-bottom-left",
        include_str!("icons/bundled/arrow-diagonal-bottom-left.svg"),
    ),
    (
        "arrow-diagonal-bottom-right",
        include_str!("icons/bundled/arrow-diagonal-bottom-right.svg"),
    ),
    (
        "arrow-diagonal-top-left",
        include_str!("icons/bundled/arrow-diagonal-top-left.svg"),
    ),
    (
        "arrow-diagonal-top-right",
        include_str!("icons/bundled/arrow-diagonal-top-right.svg"),
    ),
    ("arrow-down", include_str!("icons/bundled/arrow-down.svg")),
    (
        "arrow-down-right",
        include_str!("icons/bundled/arrow-down-right.svg"),
    ),
    ("arrow-left", include_str!("icons/bundled/arrow-left.svg")),
    ("arrow-right", include_str!("icons/bundled/arrow-right.svg")),
    (
        "arrow-right-down",
        include_str!("icons/bundled/arrow-right-down.svg"),
    ),
    ("arrow-up", include_str!("icons/bundled/arrow-up.svg")),
    (
        "arrow-up-left",
        include_str!("icons/bundled/arrow-up-left.svg"),
    ),
    ("aspects", include_str!("icons/bundled/aspects.svg")),
    (
        "aspects-filled",
        include_str!("icons/bundled/aspects-filled.svg"),
    ),
    (
        "asset-indoor",
        include_str!("icons/bundled/asset-indoor.svg"),
    ),
    (
        "asset-network",
        include_str!("icons/bundled/asset-network.svg"),
    ),
    (
        "asset-network-filled",
        include_str!("icons/bundled/asset-network-filled.svg"),
    ),
    (
        "asset-outdoor",
        include_str!("icons/bundled/asset-outdoor.svg"),
    ),
    ("asterisk", include_str!("icons/bundled/asterisk.svg")),
    ("attach", include_str!("icons/bundled/attach.svg")),
    (
        "attachment-upload",
        include_str!("icons/bundled/attachment-upload.svg"),
    ),
    (
        "audio-description-2-ai",
        include_str!("icons/bundled/audio-description-2-ai.svg"),
    ),
    (
        "audio-description1",
        include_str!("icons/bundled/audio-description1.svg"),
    ),
    (
        "audio-description2",
        include_str!("icons/bundled/audio-description2.svg"),
    ),
    (
        "audit-report",
        include_str!("icons/bundled/audit-report.svg"),
    ),
    ("average", include_str!("icons/bundled/average.svg")),
    (
        "axes-synchronous",
        include_str!("icons/bundled/axes-synchronous.svg"),
    ),
    (
        "axis-positioning",
        include_str!("icons/bundled/axis-positioning.svg"),
    ),
    (
        "axis-rotation",
        include_str!("icons/bundled/axis-rotation.svg"),
    ),
    ("backup", include_str!("icons/bundled/backup.svg")),
    (
        "backup-filled",
        include_str!("icons/bundled/backup-filled.svg"),
    ),
    ("bar-code", include_str!("icons/bundled/bar-code.svg")),
    ("barchart", include_str!("icons/bundled/barchart.svg")),
    ("barchart-ai", include_str!("icons/bundled/barchart-ai.svg")),
    (
        "barchart-horizontal",
        include_str!("icons/bundled/barchart-horizontal.svg"),
    ),
    (
        "battery-charge",
        include_str!("icons/bundled/battery-charge.svg"),
    ),
    (
        "battery-check",
        include_str!("icons/bundled/battery-check.svg"),
    ),
    (
        "battery-empty",
        include_str!("icons/bundled/battery-empty.svg"),
    ),
    (
        "battery-empty-question",
        include_str!("icons/bundled/battery-empty-question.svg"),
    ),
    (
        "battery-exclamation",
        include_str!("icons/bundled/battery-exclamation.svg"),
    ),
    (
        "battery-full",
        include_str!("icons/bundled/battery-full.svg"),
    ),
    (
        "battery-full-check",
        include_str!("icons/bundled/battery-full-check.svg"),
    ),
    (
        "battery-half",
        include_str!("icons/bundled/battery-half.svg"),
    ),
    ("battery-low", include_str!("icons/bundled/battery-low.svg")),
    (
        "battery-quarter",
        include_str!("icons/bundled/battery-quarter.svg"),
    ),
    (
        "battery-slash",
        include_str!("icons/bundled/battery-slash.svg"),
    ),
    (
        "battery-three-quarter",
        include_str!("icons/bundled/battery-three-quarter.svg"),
    ),
    (
        "battery-upright-charge",
        include_str!("icons/bundled/battery-upright-charge.svg"),
    ),
    (
        "battery-upright-check",
        include_str!("icons/bundled/battery-upright-check.svg"),
    ),
    (
        "battery-upright-empty",
        include_str!("icons/bundled/battery-upright-empty.svg"),
    ),
    (
        "battery-upright-exclamation",
        include_str!("icons/bundled/battery-upright-exclamation.svg"),
    ),
    (
        "battery-upright-full",
        include_str!("icons/bundled/battery-upright-full.svg"),
    ),
    (
        "battery-upright-full-check",
        include_str!("icons/bundled/battery-upright-full-check.svg"),
    ),
    (
        "battery-upright-half",
        include_str!("icons/bundled/battery-upright-half.svg"),
    ),
    (
        "battery-upright-low",
        include_str!("icons/bundled/battery-upright-low.svg"),
    ),
    (
        "battery-upright-quarter",
        include_str!("icons/bundled/battery-upright-quarter.svg"),
    ),
    (
        "battery-upright-question",
        include_str!("icons/bundled/battery-upright-question.svg"),
    ),
    (
        "battery-upright-slash",
        include_str!("icons/bundled/battery-upright-slash.svg"),
    ),
    (
        "battery-upright-three-quarter",
        include_str!("icons/bundled/battery-upright-three-quarter.svg"),
    ),
    (
        "battery-upright-xmark",
        include_str!("icons/bundled/battery-upright-xmark.svg"),
    ),
    (
        "battery-xmark",
        include_str!("icons/bundled/battery-xmark.svg"),
    ),
    (
        "bezier-curve",
        include_str!("icons/bundled/bezier-curve.svg"),
    ),
    ("binoculars", include_str!("icons/bundled/binoculars.svg")),
    (
        "binoculars-filled",
        include_str!("icons/bundled/binoculars-filled.svg"),
    ),
    ("blazor", include_str!("icons/bundled/blazor.svg")),
    ("book", include_str!("icons/bundled/book.svg")),
    ("bookmark", include_str!("icons/bundled/bookmark.svg")),
    (
        "bookmark-filled",
        include_str!("icons/bundled/bookmark-filled.svg"),
    ),
    (
        "boundary-signals",
        include_str!("icons/bundled/boundary-signals.svg"),
    ),
    ("box-closed", include_str!("icons/bundled/box-closed.svg")),
    ("box-ipc", include_str!("icons/bundled/box-ipc.svg")),
    (
        "box-ipc-fail",
        include_str!("icons/bundled/box-ipc-fail.svg"),
    ),
    (
        "box-ipc-question",
        include_str!("icons/bundled/box-ipc-question.svg"),
    ),
    (
        "box-ipc-success",
        include_str!("icons/bundled/box-ipc-success.svg"),
    ),
    ("box-open", include_str!("icons/bundled/box-open.svg")),
    ("box-plot", include_str!("icons/bundled/box-plot.svg")),
    (
        "box-plot-filled",
        include_str!("icons/bundled/box-plot-filled.svg"),
    ),
    (
        "bring-forward",
        include_str!("icons/bundled/bring-forward.svg"),
    ),
    (
        "bring-to-front",
        include_str!("icons/bundled/bring-to-front.svg"),
    ),
    ("brush", include_str!("icons/bundled/brush.svg")),
    ("bug", include_str!("icons/bundled/bug.svg")),
    ("bug-filled", include_str!("icons/bundled/bug-filled.svg")),
    ("bug-runtime", include_str!("icons/bundled/bug-runtime.svg")),
    (
        "bug-runtime-filled",
        include_str!("icons/bundled/bug-runtime-filled.svg"),
    ),
    (
        "building-block",
        include_str!("icons/bundled/building-block.svg"),
    ),
    (
        "building-block-filled",
        include_str!("icons/bundled/building-block-filled.svg"),
    ),
    ("building1", include_str!("icons/bundled/building1.svg")),
    (
        "building1-filled",
        include_str!("icons/bundled/building1-filled.svg"),
    ),
    ("building2", include_str!("icons/bundled/building2.svg")),
    (
        "building2-filled",
        include_str!("icons/bundled/building2-filled.svg"),
    ),
    ("bulb", include_str!("icons/bundled/bulb.svg")),
    ("bulb-filled", include_str!("icons/bundled/bulb-filled.svg")),
    (
        "calculate-ai",
        include_str!("icons/bundled/calculate-ai.svg"),
    ),
    ("calculator", include_str!("icons/bundled/calculator.svg")),
    ("calendar", include_str!("icons/bundled/calendar.svg")),
    (
        "calendar-day",
        include_str!("icons/bundled/calendar-day.svg"),
    ),
    (
        "calendar-day-filled",
        include_str!("icons/bundled/calendar-day-filled.svg"),
    ),
    (
        "calendar-filled",
        include_str!("icons/bundled/calendar-filled.svg"),
    ),
    (
        "calendar-settings",
        include_str!("icons/bundled/calendar-settings.svg"),
    ),
    (
        "calendar-week",
        include_str!("icons/bundled/calendar-week.svg"),
    ),
    (
        "calendar-week-filled",
        include_str!("icons/bundled/calendar-week-filled.svg"),
    ),
    ("cam", include_str!("icons/bundled/cam.svg")),
    ("cam-disk", include_str!("icons/bundled/cam-disk.svg")),
    (
        "cam-disk-10k",
        include_str!("icons/bundled/cam-disk-10k.svg"),
    ),
    (
        "cam-disk-600-seg",
        include_str!("icons/bundled/cam-disk-600-seg.svg"),
    ),
    (
        "cam-disk-6k-seg",
        include_str!("icons/bundled/cam-disk-6k-seg.svg"),
    ),
    ("cam-track", include_str!("icons/bundled/cam-track.svg")),
    ("cancel", include_str!("icons/bundled/cancel.svg")),
    ("cancelled", include_str!("icons/bundled/cancelled.svg")),
    ("capacity", include_str!("icons/bundled/capacity.svg")),
    (
        "capacity-check",
        include_str!("icons/bundled/capacity-check.svg"),
    ),
    (
        "capacity-check-filled",
        include_str!("icons/bundled/capacity-check-filled.svg"),
    ),
    (
        "capacity-filled",
        include_str!("icons/bundled/capacity-filled.svg"),
    ),
    (
        "capacity-locked",
        include_str!("icons/bundled/capacity-locked.svg"),
    ),
    (
        "capacity-locked-filled",
        include_str!("icons/bundled/capacity-locked-filled.svg"),
    ),
    (
        "capacity-pen",
        include_str!("icons/bundled/capacity-pen.svg"),
    ),
    (
        "capacity-pen-filled",
        include_str!("icons/bundled/capacity-pen-filled.svg"),
    ),
    ("capture", include_str!("icons/bundled/capture.svg")),
    ("car", include_str!("icons/bundled/car.svg")),
    ("car-filled", include_str!("icons/bundled/car-filled.svg")),
    ("card-layout", include_str!("icons/bundled/card-layout.svg")),
    (
        "card-layout-filled",
        include_str!("icons/bundled/card-layout-filled.svg"),
    ),
    ("certificate", include_str!("icons/bundled/certificate.svg")),
    (
        "certificate-error",
        include_str!("icons/bundled/certificate-error.svg"),
    ),
    (
        "certificate-error-filled",
        include_str!("icons/bundled/certificate-error-filled.svg"),
    ),
    (
        "certificate-exclamation",
        include_str!("icons/bundled/certificate-exclamation.svg"),
    ),
    (
        "certificate-exclamation-filled",
        include_str!("icons/bundled/certificate-exclamation-filled.svg"),
    ),
    (
        "certificate-success",
        include_str!("icons/bundled/certificate-success.svg"),
    ),
    (
        "certificate-success-filled",
        include_str!("icons/bundled/certificate-success-filled.svg"),
    ),
    (
        "chain-alternative",
        include_str!("icons/bundled/chain-alternative.svg"),
    ),
    (
        "chain-parallel",
        include_str!("icons/bundled/chain-parallel.svg"),
    ),
    ("chain-step", include_str!("icons/bundled/chain-step.svg")),
    (
        "chart-cursor",
        include_str!("icons/bundled/chart-cursor.svg"),
    ),
    (
        "chart-curve-linear",
        include_str!("icons/bundled/chart-curve-linear.svg"),
    ),
    (
        "chart-curve-spline",
        include_str!("icons/bundled/chart-curve-spline.svg"),
    ),
    (
        "chart-curve-stepped",
        include_str!("icons/bundled/chart-curve-stepped.svg"),
    ),
    (
        "chart-diagram",
        include_str!("icons/bundled/chart-diagram.svg"),
    ),
    (
        "chart-diagram-add",
        include_str!("icons/bundled/chart-diagram-add.svg"),
    ),
    (
        "chart-diagrams",
        include_str!("icons/bundled/chart-diagrams.svg"),
    ),
    (
        "chart-duration-curve",
        include_str!("icons/bundled/chart-duration-curve.svg"),
    ),
    (
        "chart-error-bar",
        include_str!("icons/bundled/chart-error-bar.svg"),
    ),
    (
        "chart-labels",
        include_str!("icons/bundled/chart-labels.svg"),
    ),
    (
        "chart-labels-filled",
        include_str!("icons/bundled/chart-labels-filled.svg"),
    ),
    ("chart-types", include_str!("icons/bundled/chart-types.svg")),
    (
        "chart-types-filled",
        include_str!("icons/bundled/chart-types-filled.svg"),
    ),
    (
        "chart-value-horizontal",
        include_str!("icons/bundled/chart-value-horizontal.svg"),
    ),
    (
        "chart-value-vertical",
        include_str!("icons/bundled/chart-value-vertical.svg"),
    ),
    ("check", include_str!("icons/bundled/check.svg")),
    ("check-in", include_str!("icons/bundled/check-in.svg")),
    ("check-out", include_str!("icons/bundled/check-out.svg")),
    ("checkbox", include_str!("icons/bundled/checkbox.svg")),
    (
        "checkbox-empty",
        include_str!("icons/bundled/checkbox-empty.svg"),
    ),
    (
        "checkbox-empty-filled",
        include_str!("icons/bundled/checkbox-empty-filled.svg"),
    ),
    (
        "checkbox-filled",
        include_str!("icons/bundled/checkbox-filled.svg"),
    ),
    (
        "checkbox-mixed",
        include_str!("icons/bundled/checkbox-mixed.svg"),
    ),
    (
        "checkbox-mixed-filled",
        include_str!("icons/bundled/checkbox-mixed-filled.svg"),
    ),
    ("checkboxes", include_str!("icons/bundled/checkboxes.svg")),
    (
        "checkboxes-empty",
        include_str!("icons/bundled/checkboxes-empty.svg"),
    ),
    (
        "checkboxes-empty-filled",
        include_str!("icons/bundled/checkboxes-empty-filled.svg"),
    ),
    (
        "checkboxes-filled",
        include_str!("icons/bundled/checkboxes-filled.svg"),
    ),
    (
        "chevron-down",
        include_str!("icons/bundled/chevron-down.svg"),
    ),
    (
        "chevron-down-bar",
        include_str!("icons/bundled/chevron-down-bar.svg"),
    ),
    (
        "chevron-down-bar-small",
        include_str!("icons/bundled/chevron-down-bar-small.svg"),
    ),
    (
        "chevron-down-small",
        include_str!("icons/bundled/chevron-down-small.svg"),
    ),
    (
        "chevron-left",
        include_str!("icons/bundled/chevron-left.svg"),
    ),
    (
        "chevron-left-bar",
        include_str!("icons/bundled/chevron-left-bar.svg"),
    ),
    (
        "chevron-left-bar-small",
        include_str!("icons/bundled/chevron-left-bar-small.svg"),
    ),
    (
        "chevron-left-small",
        include_str!("icons/bundled/chevron-left-small.svg"),
    ),
    (
        "chevron-right",
        include_str!("icons/bundled/chevron-right.svg"),
    ),
    (
        "chevron-right-bar",
        include_str!("icons/bundled/chevron-right-bar.svg"),
    ),
    (
        "chevron-right-bar-small",
        include_str!("icons/bundled/chevron-right-bar-small.svg"),
    ),
    (
        "chevron-right-small",
        include_str!("icons/bundled/chevron-right-small.svg"),
    ),
    ("chevron-up", include_str!("icons/bundled/chevron-up.svg")),
    (
        "chevron-up-bar",
        include_str!("icons/bundled/chevron-up-bar.svg"),
    ),
    (
        "chevron-up-bar-small",
        include_str!("icons/bundled/chevron-up-bar-small.svg"),
    ),
    (
        "chevron-up-small",
        include_str!("icons/bundled/chevron-up-small.svg"),
    ),
    ("circle", include_str!("icons/bundled/circle.svg")),
    ("circle-dot", include_str!("icons/bundled/circle-dot.svg")),
    (
        "circle-dot-filled",
        include_str!("icons/bundled/circle-dot-filled.svg"),
    ),
    (
        "circle-filled",
        include_str!("icons/bundled/circle-filled.svg"),
    ),
    (
        "circle-pause",
        include_str!("icons/bundled/circle-pause.svg"),
    ),
    (
        "circle-pause-filled",
        include_str!("icons/bundled/circle-pause-filled.svg"),
    ),
    ("circle-play", include_str!("icons/bundled/circle-play.svg")),
    (
        "circle-play-filled",
        include_str!("icons/bundled/circle-play-filled.svg"),
    ),
    ("circle-stop", include_str!("icons/bundled/circle-stop.svg")),
    (
        "circle-stop-filled",
        include_str!("icons/bundled/circle-stop-filled.svg"),
    ),
    ("clear", include_str!("icons/bundled/clear.svg")),
    (
        "clear-filter",
        include_str!("icons/bundled/clear-filter.svg"),
    ),
    (
        "clear-filter-filled",
        include_str!("icons/bundled/clear-filter-filled.svg"),
    ),
    (
        "client-interface",
        include_str!("icons/bundled/client-interface.svg"),
    ),
    ("clipboard", include_str!("icons/bundled/clipboard.svg")),
    (
        "clipboard-filled",
        include_str!("icons/bundled/clipboard-filled.svg"),
    ),
    ("clock", include_str!("icons/bundled/clock.svg")),
    (
        "clock-filled",
        include_str!("icons/bundled/clock-filled.svg"),
    ),
    (
        "clock-person",
        include_str!("icons/bundled/clock-person.svg"),
    ),
    ("close", include_str!("icons/bundled/close.svg")),
    ("close-small", include_str!("icons/bundled/close-small.svg")),
    ("cloud", include_str!("icons/bundled/cloud.svg")),
    (
        "cloud-cancelled",
        include_str!("icons/bundled/cloud-cancelled.svg"),
    ),
    (
        "cloud-cancelled-filled",
        include_str!("icons/bundled/cloud-cancelled-filled.svg"),
    ),
    (
        "cloud-download",
        include_str!("icons/bundled/cloud-download.svg"),
    ),
    (
        "cloud-download-add",
        include_str!("icons/bundled/cloud-download-add.svg"),
    ),
    (
        "cloud-download-add-filled",
        include_str!("icons/bundled/cloud-download-add-filled.svg"),
    ),
    (
        "cloud-download-filled",
        include_str!("icons/bundled/cloud-download-filled.svg"),
    ),
    (
        "cloud-download-list",
        include_str!("icons/bundled/cloud-download-list.svg"),
    ),
    (
        "cloud-download-list-filled",
        include_str!("icons/bundled/cloud-download-list-filled.svg"),
    ),
    ("cloud-fail", include_str!("icons/bundled/cloud-fail.svg")),
    (
        "cloud-fail-filled",
        include_str!("icons/bundled/cloud-fail-filled.svg"),
    ),
    (
        "cloud-filled",
        include_str!("icons/bundled/cloud-filled.svg"),
    ),
    ("cloud-new", include_str!("icons/bundled/cloud-new.svg")),
    (
        "cloud-new-filled",
        include_str!("icons/bundled/cloud-new-filled.svg"),
    ),
    ("cloud-rain", include_str!("icons/bundled/cloud-rain.svg")),
    (
        "cloud-rain-filled",
        include_str!("icons/bundled/cloud-rain-filled.svg"),
    ),
    ("cloud-snow", include_str!("icons/bundled/cloud-snow.svg")),
    (
        "cloud-snow-filled",
        include_str!("icons/bundled/cloud-snow-filled.svg"),
    ),
    (
        "cloud-success",
        include_str!("icons/bundled/cloud-success.svg"),
    ),
    (
        "cloud-success-filled",
        include_str!("icons/bundled/cloud-success-filled.svg"),
    ),
    (
        "cloud-thunder",
        include_str!("icons/bundled/cloud-thunder.svg"),
    ),
    (
        "cloud-thunder-filled",
        include_str!("icons/bundled/cloud-thunder-filled.svg"),
    ),
    (
        "cloud-upload",
        include_str!("icons/bundled/cloud-upload.svg"),
    ),
    (
        "cloud-upload-filled",
        include_str!("icons/bundled/cloud-upload-filled.svg"),
    ),
    ("code", include_str!("icons/bundled/code.svg")),
    ("code-ai", include_str!("icons/bundled/code-ai.svg")),
    (
        "code-document-check",
        include_str!("icons/bundled/code-document-check.svg"),
    ),
    (
        "code-function",
        include_str!("icons/bundled/code-function.svg"),
    ),
    ("code-script", include_str!("icons/bundled/code-script.svg")),
    ("coffee", include_str!("icons/bundled/coffee.svg")),
    (
        "coffee-empty",
        include_str!("icons/bundled/coffee-empty.svg"),
    ),
    (
        "coffee-empty-filled",
        include_str!("icons/bundled/coffee-empty-filled.svg"),
    ),
    (
        "coffee-filled",
        include_str!("icons/bundled/coffee-filled.svg"),
    ),
    ("cogwheel", include_str!("icons/bundled/cogwheel.svg")),
    (
        "cogwheel-filled",
        include_str!("icons/bundled/cogwheel-filled.svg"),
    ),
    ("coin", include_str!("icons/bundled/coin.svg")),
    ("coin-filled", include_str!("icons/bundled/coin-filled.svg")),
    ("coin-stack", include_str!("icons/bundled/coin-stack.svg")),
    (
        "coin-stack-filled",
        include_str!("icons/bundled/coin-stack-filled.svg"),
    ),
    ("coins", include_str!("icons/bundled/coins.svg")),
    (
        "coins-filled",
        include_str!("icons/bundled/coins-filled.svg"),
    ),
    (
        "collapse-all",
        include_str!("icons/bundled/collapse-all.svg"),
    ),
    ("combine", include_str!("icons/bundled/combine.svg")),
    (
        "command-line",
        include_str!("icons/bundled/command-line.svg"),
    ),
    (
        "command-line-filled",
        include_str!("icons/bundled/command-line-filled.svg"),
    ),
    ("comment-alt", include_str!("icons/bundled/comment-alt.svg")),
    (
        "comment-alt-filled",
        include_str!("icons/bundled/comment-alt-filled.svg"),
    ),
    (
        "communication",
        include_str!("icons/bundled/communication.svg"),
    ),
    (
        "communication-filled",
        include_str!("icons/bundled/communication-filled.svg"),
    ),
    (
        "compact-disc",
        include_str!("icons/bundled/compact-disc.svg"),
    ),
    (
        "compact-disc-filled",
        include_str!("icons/bundled/compact-disc-filled.svg"),
    ),
    ("compare", include_str!("icons/bundled/compare.svg")),
    (
        "compound-block",
        include_str!("icons/bundled/compound-block.svg"),
    ),
    (
        "configuration",
        include_str!("icons/bundled/configuration.svg"),
    ),
    (
        "configuration-safety",
        include_str!("icons/bundled/configuration-safety.svg"),
    ),
    ("configure", include_str!("icons/bundled/configure.svg")),
    (
        "configure-filled",
        include_str!("icons/bundled/configure-filled.svg"),
    ),
    ("connected", include_str!("icons/bundled/connected.svg")),
    (
        "connected-circle",
        include_str!("icons/bundled/connected-circle.svg"),
    ),
    (
        "connected-circle-filled",
        include_str!("icons/bundled/connected-circle-filled.svg"),
    ),
    (
        "connection-bulb",
        include_str!("icons/bundled/connection-bulb.svg"),
    ),
    (
        "connection-fail",
        include_str!("icons/bundled/connection-fail.svg"),
    ),
    (
        "connection-local",
        include_str!("icons/bundled/connection-local.svg"),
    ),
    (
        "connection-signal",
        include_str!("icons/bundled/connection-signal.svg"),
    ),
    (
        "connection-success",
        include_str!("icons/bundled/connection-success.svg"),
    ),
    ("connections", include_str!("icons/bundled/connections.svg")),
    (
        "connections-settings",
        include_str!("icons/bundled/connections-settings.svg"),
    ),
    (
        "connectivity",
        include_str!("icons/bundled/connectivity.svg"),
    ),
    ("connector", include_str!("icons/bundled/connector.svg")),
    (
        "connector-chart",
        include_str!("icons/bundled/connector-chart.svg"),
    ),
    (
        "connector-chart-filled",
        include_str!("icons/bundled/connector-chart-filled.svg"),
    ),
    (
        "connector-filled",
        include_str!("icons/bundled/connector-filled.svg"),
    ),
    (
        "connector-hex",
        include_str!("icons/bundled/connector-hex.svg"),
    ),
    (
        "connector-hex-filled",
        include_str!("icons/bundled/connector-hex-filled.svg"),
    ),
    (
        "connector-rect",
        include_str!("icons/bundled/connector-rect.svg"),
    ),
    (
        "connector-rect-filled",
        include_str!("icons/bundled/connector-rect-filled.svg"),
    ),
    (
        "connector-rhomb",
        include_str!("icons/bundled/connector-rhomb.svg"),
    ),
    (
        "connector-rhomb-filled",
        include_str!("icons/bundled/connector-rhomb-filled.svg"),
    ),
    (
        "consistency-check",
        include_str!("icons/bundled/consistency-check.svg"),
    ),
    (
        "contact-details",
        include_str!("icons/bundled/contact-details.svg"),
    ),
    (
        "contact-details-filled",
        include_str!("icons/bundled/contact-details-filled.svg"),
    ),
    (
        "context-menu",
        include_str!("icons/bundled/context-menu.svg"),
    ),
    (
        "control-button",
        include_str!("icons/bundled/control-button.svg"),
    ),
    (
        "control-checkbox",
        include_str!("icons/bundled/control-checkbox.svg"),
    ),
    (
        "control-io-field",
        include_str!("icons/bundled/control-io-field.svg"),
    ),
    (
        "control-label",
        include_str!("icons/bundled/control-label.svg"),
    ),
    (
        "control-list-box",
        include_str!("icons/bundled/control-list-box.svg"),
    ),
    (
        "control-radiobutton",
        include_str!("icons/bundled/control-radiobutton.svg"),
    ),
    (
        "control-select",
        include_str!("icons/bundled/control-select.svg"),
    ),
    (
        "control-slider",
        include_str!("icons/bundled/control-slider.svg"),
    ),
    (
        "control-spinner",
        include_str!("icons/bundled/control-spinner.svg"),
    ),
    (
        "control-switch",
        include_str!("icons/bundled/control-switch.svg"),
    ),
    (
        "control-text-button",
        include_str!("icons/bundled/control-text-button.svg"),
    ),
    (
        "control-textbox",
        include_str!("icons/bundled/control-textbox.svg"),
    ),
    (
        "control-touch-area",
        include_str!("icons/bundled/control-touch-area.svg"),
    ),
    (
        "control-value-bar",
        include_str!("icons/bundled/control-value-bar.svg"),
    ),
    (
        "controlled-device",
        include_str!("icons/bundled/controlled-device.svg"),
    ),
    (
        "controller-device",
        include_str!("icons/bundled/controller-device.svg"),
    ),
    (
        "controller-device-safety",
        include_str!("icons/bundled/controller-device-safety.svg"),
    ),
    (
        "conversation",
        include_str!("icons/bundled/conversation.svg"),
    ),
    (
        "conversation-filled",
        include_str!("icons/bundled/conversation-filled.svg"),
    ),
    ("copy", include_str!("icons/bundled/copy.svg")),
    ("copy-filled", include_str!("icons/bundled/copy-filled.svg")),
    ("couch", include_str!("icons/bundled/couch.svg")),
    (
        "couch-filled",
        include_str!("icons/bundled/couch-filled.svg"),
    ),
    ("counter", include_str!("icons/bundled/counter.svg")),
    (
        "create-plant",
        include_str!("icons/bundled/create-plant.svg"),
    ),
    (
        "create-plant-filled",
        include_str!("icons/bundled/create-plant-filled.svg"),
    ),
    ("crosshairs", include_str!("icons/bundled/crosshairs.svg")),
    (
        "crosshairs-filled",
        include_str!("icons/bundled/crosshairs-filled.svg"),
    ),
    ("customer", include_str!("icons/bundled/customer.svg")),
    (
        "customer-filled",
        include_str!("icons/bundled/customer-filled.svg"),
    ),
    ("cut", include_str!("icons/bundled/cut.svg")),
    ("cut-filled", include_str!("icons/bundled/cut-filled.svg")),
    ("cycle", include_str!("icons/bundled/cycle.svg")),
    ("cycle-alt", include_str!("icons/bundled/cycle-alt.svg")),
    ("dashboard", include_str!("icons/bundled/dashboard.svg")),
    (
        "dashboard-ai",
        include_str!("icons/bundled/dashboard-ai.svg"),
    ),
    (
        "dashboard-ai-filled",
        include_str!("icons/bundled/dashboard-ai-filled.svg"),
    ),
    (
        "dashboard-filled",
        include_str!("icons/bundled/dashboard-filled.svg"),
    ),
    (
        "dashboard-pen",
        include_str!("icons/bundled/dashboard-pen.svg"),
    ),
    (
        "dashboard-pen-filled",
        include_str!("icons/bundled/dashboard-pen-filled.svg"),
    ),
    ("data-egress", include_str!("icons/bundled/data-egress.svg")),
    (
        "data-ingress",
        include_str!("icons/bundled/data-ingress.svg"),
    ),
    (
        "data-ingress-egress",
        include_str!("icons/bundled/data-ingress-egress.svg"),
    ),
    (
        "data-management",
        include_str!("icons/bundled/data-management.svg"),
    ),
    (
        "data-management-filled",
        include_str!("icons/bundled/data-management-filled.svg"),
    ),
    (
        "data-type-boolean",
        include_str!("icons/bundled/data-type-boolean.svg"),
    ),
    (
        "data-type-double",
        include_str!("icons/bundled/data-type-double.svg"),
    ),
    (
        "data-type-enum",
        include_str!("icons/bundled/data-type-enum.svg"),
    ),
    (
        "data-type-integer",
        include_str!("icons/bundled/data-type-integer.svg"),
    ),
    (
        "data-type-string",
        include_str!("icons/bundled/data-type-string.svg"),
    ),
    (
        "data-type-string-list",
        include_str!("icons/bundled/data-type-string-list.svg"),
    ),
    ("database", include_str!("icons/bundled/database.svg")),
    (
        "database-arrow-left",
        include_str!("icons/bundled/database-arrow-left.svg"),
    ),
    (
        "database-filled",
        include_str!("icons/bundled/database-filled.svg"),
    ),
    (
        "database-safety",
        include_str!("icons/bundled/database-safety.svg"),
    ),
    ("details", include_str!("icons/bundled/details.svg")),
    (
        "device-driver",
        include_str!("icons/bundled/device-driver.svg"),
    ),
    ("device-fan", include_str!("icons/bundled/device-fan.svg")),
    (
        "device-manager",
        include_str!("icons/bundled/device-manager.svg"),
    ),
    ("device-play", include_str!("icons/bundled/device-play.svg")),
    (
        "device-play-filled",
        include_str!("icons/bundled/device-play-filled.svg"),
    ),
    (
        "device-view-flat",
        include_str!("icons/bundled/device-view-flat.svg"),
    ),
    (
        "device-view-hierarchical",
        include_str!("icons/bundled/device-view-hierarchical.svg"),
    ),
    (
        "diagram-module",
        include_str!("icons/bundled/diagram-module.svg"),
    ),
    (
        "diagram-module-library",
        include_str!("icons/bundled/diagram-module-library.svg"),
    ),
    (
        "diagram-module-new",
        include_str!("icons/bundled/diagram-module-new.svg"),
    ),
    ("diamond", include_str!("icons/bundled/diamond.svg")),
    (
        "disconnected",
        include_str!("icons/bundled/disconnected.svg"),
    ),
    (
        "disconnected-circle",
        include_str!("icons/bundled/disconnected-circle.svg"),
    ),
    (
        "disconnected-circle-filled",
        include_str!("icons/bundled/disconnected-circle-filled.svg"),
    ),
    ("disk", include_str!("icons/bundled/disk.svg")),
    ("disk-filled", include_str!("icons/bundled/disk-filled.svg")),
    ("disk-pen", include_str!("icons/bundled/disk-pen.svg")),
    (
        "distribute-objects-horizontally",
        include_str!("icons/bundled/distribute-objects-horizontally.svg"),
    ),
    (
        "distribute-objects-vertically",
        include_str!("icons/bundled/distribute-objects-vertically.svg"),
    ),
    (
        "distribution",
        include_str!("icons/bundled/distribution.svg"),
    ),
    (
        "doc-document",
        include_str!("icons/bundled/doc-document.svg"),
    ),
    ("document", include_str!("icons/bundled/document.svg")),
    ("document-ai", include_str!("icons/bundled/document-ai.svg")),
    (
        "document-ai-filled",
        include_str!("icons/bundled/document-ai-filled.svg"),
    ),
    (
        "document-bulk",
        include_str!("icons/bundled/document-bulk.svg"),
    ),
    (
        "document-code",
        include_str!("icons/bundled/document-code.svg"),
    ),
    (
        "document-code-filled",
        include_str!("icons/bundled/document-code-filled.svg"),
    ),
    (
        "document-fail",
        include_str!("icons/bundled/document-fail.svg"),
    ),
    (
        "document-filled",
        include_str!("icons/bundled/document-filled.svg"),
    ),
    (
        "document-info",
        include_str!("icons/bundled/document-info.svg"),
    ),
    (
        "document-link",
        include_str!("icons/bundled/document-link.svg"),
    ),
    (
        "document-management",
        include_str!("icons/bundled/document-management.svg"),
    ),
    (
        "document-mapping",
        include_str!("icons/bundled/document-mapping.svg"),
    ),
    (
        "document-mapping-filled",
        include_str!("icons/bundled/document-mapping-filled.svg"),
    ),
    (
        "document-plus-minus",
        include_str!("icons/bundled/document-plus-minus.svg"),
    ),
    (
        "document-plus-minus-filled",
        include_str!("icons/bundled/document-plus-minus-filled.svg"),
    ),
    (
        "document-program",
        include_str!("icons/bundled/document-program.svg"),
    ),
    (
        "document-program-filled",
        include_str!("icons/bundled/document-program-filled.svg"),
    ),
    (
        "document-reference",
        include_str!("icons/bundled/document-reference.svg"),
    ),
    (
        "document-settings",
        include_str!("icons/bundled/document-settings.svg"),
    ),
    (
        "document-success",
        include_str!("icons/bundled/document-success.svg"),
    ),
    (
        "double-check",
        include_str!("icons/bundled/double-check.svg"),
    ),
    (
        "double-chevron-down",
        include_str!("icons/bundled/double-chevron-down.svg"),
    ),
    (
        "double-chevron-left",
        include_str!("icons/bundled/double-chevron-left.svg"),
    ),
    (
        "double-chevron-right",
        include_str!("icons/bundled/double-chevron-right.svg"),
    ),
    (
        "double-chevron-up",
        include_str!("icons/bundled/double-chevron-up.svg"),
    ),
    ("doublet", include_str!("icons/bundled/doublet.svg")),
    (
        "doublet-filled",
        include_str!("icons/bundled/doublet-filled.svg"),
    ),
    (
        "doughnutchart",
        include_str!("icons/bundled/doughnutchart.svg"),
    ),
    (
        "doughnutchart-filled",
        include_str!("icons/bundled/doughnutchart-filled.svg"),
    ),
    ("download", include_str!("icons/bundled/download.svg")),
    (
        "download-add",
        include_str!("icons/bundled/download-add.svg"),
    ),
    (
        "download-delta",
        include_str!("icons/bundled/download-delta.svg"),
    ),
    (
        "download-full",
        include_str!("icons/bundled/download-full.svg"),
    ),
    (
        "download-list",
        include_str!("icons/bundled/download-list.svg"),
    ),
    ("downtime", include_str!("icons/bundled/downtime.svg")),
    (
        "drag-and-drop",
        include_str!("icons/bundled/drag-and-drop.svg"),
    ),
    (
        "drag-gripper",
        include_str!("icons/bundled/drag-gripper.svg"),
    ),
    ("draw-circle", include_str!("icons/bundled/draw-circle.svg")),
    (
        "draw-circle-arc",
        include_str!("icons/bundled/draw-circle-arc.svg"),
    ),
    (
        "draw-circle-segment",
        include_str!("icons/bundled/draw-circle-segment.svg"),
    ),
    (
        "draw-ellipse-segment",
        include_str!("icons/bundled/draw-ellipse-segment.svg"),
    ),
    (
        "drawer-documents",
        include_str!("icons/bundled/drawer-documents.svg"),
    ),
    (
        "drawing-document",
        include_str!("icons/bundled/drawing-document.svg"),
    ),
    (
        "drawing-document-filled",
        include_str!("icons/bundled/drawing-document-filled.svg"),
    ),
    ("drive", include_str!("icons/bundled/drive.svg")),
    (
        "drive-safety",
        include_str!("icons/bundled/drive-safety.svg"),
    ),
    ("drop", include_str!("icons/bundled/drop.svg")),
    ("drop-zone", include_str!("icons/bundled/drop-zone.svg")),
    ("duplicate", include_str!("icons/bundled/duplicate.svg")),
    (
        "duplicate-document",
        include_str!("icons/bundled/duplicate-document.svg"),
    ),
    (
        "duplicate-filled",
        include_str!("icons/bundled/duplicate-filled.svg"),
    ),
    ("e-mail", include_str!("icons/bundled/e-mail.svg")),
    (
        "e-mail-filled",
        include_str!("icons/bundled/e-mail-filled.svg"),
    ),
    ("earth", include_str!("icons/bundled/earth.svg")),
    (
        "earth-filled",
        include_str!("icons/bundled/earth-filled.svg"),
    ),
    (
        "edit-document",
        include_str!("icons/bundled/edit-document.svg"),
    ),
    (
        "edit-document-filled",
        include_str!("icons/bundled/edit-document-filled.svg"),
    ),
    ("edit-plant", include_str!("icons/bundled/edit-plant.svg")),
    (
        "edit-plant-filled",
        include_str!("icons/bundled/edit-plant-filled.svg"),
    ),
    ("editor-grid", include_str!("icons/bundled/editor-grid.svg")),
    (
        "editor-grid-dots",
        include_str!("icons/bundled/editor-grid-dots.svg"),
    ),
    (
        "editor-grid-lines",
        include_str!("icons/bundled/editor-grid-lines.svg"),
    ),
    (
        "editor-grid-magnet",
        include_str!("icons/bundled/editor-grid-magnet.svg"),
    ),
    (
        "editor-grid-none",
        include_str!("icons/bundled/editor-grid-none.svg"),
    ),
    (
        "editor-guide-lines-magnet",
        include_str!("icons/bundled/editor-guide-lines-magnet.svg"),
    ),
    (
        "editor-guides",
        include_str!("icons/bundled/editor-guides.svg"),
    ),
    (
        "editor-resources",
        include_str!("icons/bundled/editor-resources.svg"),
    ),
    (
        "electrical-energy",
        include_str!("icons/bundled/electrical-energy.svg"),
    ),
    (
        "electrical-energy-filled",
        include_str!("icons/bundled/electrical-energy-filled.svg"),
    ),
    ("element", include_str!("icons/bundled/element.svg")),
    (
        "element-filled",
        include_str!("icons/bundled/element-filled.svg"),
    ),
    ("ellipse", include_str!("icons/bundled/ellipse.svg")),
    ("ellipse-arc", include_str!("icons/bundled/ellipse-arc.svg")),
    (
        "ellipse-filled",
        include_str!("icons/bundled/ellipse-filled.svg"),
    ),
    (
        "email-document",
        include_str!("icons/bundled/email-document.svg"),
    ),
    (
        "email-document-filled",
        include_str!("icons/bundled/email-document-filled.svg"),
    ),
    (
        "emergency-stop",
        include_str!("icons/bundled/emergency-stop.svg"),
    ),
    (
        "emergency-stop-filled",
        include_str!("icons/bundled/emergency-stop-filled.svg"),
    ),
    ("emote-happy", include_str!("icons/bundled/emote-happy.svg")),
    (
        "emote-happy-filled",
        include_str!("icons/bundled/emote-happy-filled.svg"),
    ),
    (
        "emote-neutral",
        include_str!("icons/bundled/emote-neutral.svg"),
    ),
    (
        "emote-neutral-filled",
        include_str!("icons/bundled/emote-neutral-filled.svg"),
    ),
    ("emote-sad", include_str!("icons/bundled/emote-sad.svg")),
    (
        "emote-sad-filled",
        include_str!("icons/bundled/emote-sad-filled.svg"),
    ),
    ("error", include_str!("icons/bundled/error.svg")),
    (
        "error-filled",
        include_str!("icons/bundled/error-filled.svg"),
    ),
    (
        "error-multiple",
        include_str!("icons/bundled/error-multiple.svg"),
    ),
    (
        "error-multiple-filled",
        include_str!("icons/bundled/error-multiple-filled.svg"),
    ),
    (
        "exclamation-mark",
        include_str!("icons/bundled/exclamation-mark.svg"),
    ),
    ("expand-all", include_str!("icons/bundled/expand-all.svg")),
    ("explore", include_str!("icons/bundled/explore.svg")),
    (
        "explore-filled",
        include_str!("icons/bundled/explore-filled.svg"),
    ),
    ("export", include_str!("icons/bundled/export.svg")),
    (
        "export-check",
        include_str!("icons/bundled/export-check.svg"),
    ),
    (
        "export-failed",
        include_str!("icons/bundled/export-failed.svg"),
    ),
    (
        "export-progress",
        include_str!("icons/bundled/export-progress.svg"),
    ),
    ("extension", include_str!("icons/bundled/extension.svg")),
    (
        "external-encoder",
        include_str!("icons/bundled/external-encoder.svg"),
    ),
    ("eye", include_str!("icons/bundled/eye.svg")),
    (
        "eye-cancelled",
        include_str!("icons/bundled/eye-cancelled.svg"),
    ),
    (
        "eye-cancelled-filled",
        include_str!("icons/bundled/eye-cancelled-filled.svg"),
    ),
    ("eye-dropper", include_str!("icons/bundled/eye-dropper.svg")),
    ("eye-filled", include_str!("icons/bundled/eye-filled.svg")),
    ("eye-focus", include_str!("icons/bundled/eye-focus.svg")),
    (
        "eye-magnifying-glass",
        include_str!("icons/bundled/eye-magnifying-glass.svg"),
    ),
    (
        "faceplate-container",
        include_str!("icons/bundled/faceplate-container.svg"),
    ),
    (
        "factory-reset",
        include_str!("icons/bundled/factory-reset.svg"),
    ),
    (
        "factory-reset-filled",
        include_str!("icons/bundled/factory-reset-filled.svg"),
    ),
    (
        "fast-forward",
        include_str!("icons/bundled/fast-forward.svg"),
    ),
    (
        "fast-forward-filled",
        include_str!("icons/bundled/fast-forward-filled.svg"),
    ),
    ("feedback", include_str!("icons/bundled/feedback.svg")),
    (
        "feedback-filled",
        include_str!("icons/bundled/feedback-filled.svg"),
    ),
    ("filter", include_str!("icons/bundled/filter.svg")),
    (
        "filter-filled",
        include_str!("icons/bundled/filter-filled.svg"),
    ),
    (
        "filter-update",
        include_str!("icons/bundled/filter-update.svg"),
    ),
    ("firmware", include_str!("icons/bundled/firmware.svg")),
    (
        "fit-to-screen",
        include_str!("icons/bundled/fit-to-screen.svg"),
    ),
    ("flag", include_str!("icons/bundled/flag.svg")),
    ("flag-alt", include_str!("icons/bundled/flag-alt.svg")),
    (
        "flag-alt-filled",
        include_str!("icons/bundled/flag-alt-filled.svg"),
    ),
    ("flag-filled", include_str!("icons/bundled/flag-filled.svg")),
    ("flare", include_str!("icons/bundled/flare.svg")),
    ("flashing", include_str!("icons/bundled/flashing.svg")),
    (
        "flow-physically",
        include_str!("icons/bundled/flow-physically.svg"),
    ),
    ("folder", include_str!("icons/bundled/folder.svg")),
    (
        "folder-application-screen",
        include_str!("icons/bundled/folder-application-screen.svg"),
    ),
    (
        "folder-collapse-all",
        include_str!("icons/bundled/folder-collapse-all.svg"),
    ),
    ("folder-down", include_str!("icons/bundled/folder-down.svg")),
    (
        "folder-down-filled",
        include_str!("icons/bundled/folder-down-filled.svg"),
    ),
    (
        "folder-expand-all",
        include_str!("icons/bundled/folder-expand-all.svg"),
    ),
    (
        "folder-filled",
        include_str!("icons/bundled/folder-filled.svg"),
    ),
    ("folder-new", include_str!("icons/bundled/folder-new.svg")),
    (
        "folder-new-filled",
        include_str!("icons/bundled/folder-new-filled.svg"),
    ),
    ("folder-open", include_str!("icons/bundled/folder-open.svg")),
    (
        "folder-open-filled",
        include_str!("icons/bundled/folder-open-filled.svg"),
    ),
    ("folder-tag", include_str!("icons/bundled/folder-tag.svg")),
    ("folder-up", include_str!("icons/bundled/folder-up.svg")),
    (
        "folder-up-filled",
        include_str!("icons/bundled/folder-up-filled.svg"),
    ),
    ("frames", include_str!("icons/bundled/frames.svg")),
    (
        "frames-filled",
        include_str!("icons/bundled/frames-filled.svg"),
    ),
    ("full-screen", include_str!("icons/bundled/full-screen.svg")),
    (
        "full-screen-exit",
        include_str!("icons/bundled/full-screen-exit.svg"),
    ),
    (
        "function-block",
        include_str!("icons/bundled/function-block.svg"),
    ),
    (
        "function-block-library",
        include_str!("icons/bundled/function-block-library.svg"),
    ),
    (
        "function-block-new",
        include_str!("icons/bundled/function-block-new.svg"),
    ),
    (
        "function-block-safety",
        include_str!("icons/bundled/function-block-safety.svg"),
    ),
    (
        "function-diagram",
        include_str!("icons/bundled/function-diagram.svg"),
    ),
    (
        "function-diagram-new",
        include_str!("icons/bundled/function-diagram-new.svg"),
    ),
    ("ganttchart", include_str!("icons/bundled/ganttchart.svg")),
    ("gauge", include_str!("icons/bundled/gauge.svg")),
    (
        "gauge-filled",
        include_str!("icons/bundled/gauge-filled.svg"),
    ),
    ("gaugechart", include_str!("icons/bundled/gaugechart.svg")),
    (
        "generic-device",
        include_str!("icons/bundled/generic-device.svg"),
    ),
    (
        "generic-device-alarm",
        include_str!("icons/bundled/generic-device-alarm.svg"),
    ),
    (
        "generic-device-brackets",
        include_str!("icons/bundled/generic-device-brackets.svg"),
    ),
    (
        "generic-device-cancelled",
        include_str!("icons/bundled/generic-device-cancelled.svg"),
    ),
    (
        "generic-device-connected",
        include_str!("icons/bundled/generic-device-connected.svg"),
    ),
    (
        "generic-device-dashed",
        include_str!("icons/bundled/generic-device-dashed.svg"),
    ),
    (
        "generic-device-disconnected",
        include_str!("icons/bundled/generic-device-disconnected.svg"),
    ),
    (
        "generic-device-error",
        include_str!("icons/bundled/generic-device-error.svg"),
    ),
    (
        "generic-device-flare",
        include_str!("icons/bundled/generic-device-flare.svg"),
    ),
    (
        "generic-device-forced-mode",
        include_str!("icons/bundled/generic-device-forced-mode.svg"),
    ),
    (
        "generic-device-incompatible",
        include_str!("icons/bundled/generic-device-incompatible.svg"),
    ),
    (
        "generic-device-io-unavailable",
        include_str!("icons/bundled/generic-device-io-unavailable.svg"),
    ),
    (
        "generic-device-lock",
        include_str!("icons/bundled/generic-device-lock.svg"),
    ),
    (
        "generic-device-maintenance",
        include_str!("icons/bundled/generic-device-maintenance.svg"),
    ),
    (
        "generic-device-pause",
        include_str!("icons/bundled/generic-device-pause.svg"),
    ),
    (
        "generic-device-play",
        include_str!("icons/bundled/generic-device-play.svg"),
    ),
    (
        "generic-device-power-saving",
        include_str!("icons/bundled/generic-device-power-saving.svg"),
    ),
    (
        "generic-device-published",
        include_str!("icons/bundled/generic-device-published.svg"),
    ),
    (
        "generic-device-question",
        include_str!("icons/bundled/generic-device-question.svg"),
    ),
    (
        "generic-device-refresh",
        include_str!("icons/bundled/generic-device-refresh.svg"),
    ),
    (
        "generic-device-reset",
        include_str!("icons/bundled/generic-device-reset.svg"),
    ),
    (
        "generic-device-restart",
        include_str!("icons/bundled/generic-device-restart.svg"),
    ),
    (
        "generic-device-rocket",
        include_str!("icons/bundled/generic-device-rocket.svg"),
    ),
    (
        "generic-device-safety",
        include_str!("icons/bundled/generic-device-safety.svg"),
    ),
    (
        "generic-device-shield",
        include_str!("icons/bundled/generic-device-shield.svg"),
    ),
    (
        "generic-device-shutdown",
        include_str!("icons/bundled/generic-device-shutdown.svg"),
    ),
    (
        "generic-device-slash",
        include_str!("icons/bundled/generic-device-slash.svg"),
    ),
    (
        "generic-device-software-alarm",
        include_str!("icons/bundled/generic-device-software-alarm.svg"),
    ),
    (
        "generic-device-standby",
        include_str!("icons/bundled/generic-device-standby.svg"),
    ),
    (
        "generic-device-stop",
        include_str!("icons/bundled/generic-device-stop.svg"),
    ),
    (
        "generic-device-stop-userprogram",
        include_str!("icons/bundled/generic-device-stop-userprogram.svg"),
    ),
    (
        "generic-device-success",
        include_str!("icons/bundled/generic-device-success.svg"),
    ),
    (
        "generic-device-synchronized",
        include_str!("icons/bundled/generic-device-synchronized.svg"),
    ),
    (
        "generic-device-traffic",
        include_str!("icons/bundled/generic-device-traffic.svg"),
    ),
    (
        "glasses-play",
        include_str!("icons/bundled/glasses-play.svg"),
    ),
    (
        "global-plant",
        include_str!("icons/bundled/global-plant.svg"),
    ),
    (
        "global-plant-filled",
        include_str!("icons/bundled/global-plant-filled.svg"),
    ),
    ("globe", include_str!("icons/bundled/globe.svg")),
    (
        "globe-filled",
        include_str!("icons/bundled/globe-filled.svg"),
    ),
    ("globe-tag", include_str!("icons/bundled/globe-tag.svg")),
    ("goto", include_str!("icons/bundled/goto.svg")),
    ("graph", include_str!("icons/bundled/graph.svg")),
    (
        "graph-filled",
        include_str!("icons/bundled/graph-filled.svg"),
    ),
    ("grid-pen", include_str!("icons/bundled/grid-pen.svg")),
    ("group", include_str!("icons/bundled/group.svg")),
    (
        "group-objects",
        include_str!("icons/bundled/group-objects.svg"),
    ),
    ("hand", include_str!("icons/bundled/hand.svg")),
    ("hand-filled", include_str!("icons/bundled/hand-filled.svg")),
    ("handshake", include_str!("icons/bundled/handshake.svg")),
    (
        "hard-disk-drive",
        include_str!("icons/bundled/hard-disk-drive.svg"),
    ),
    ("hard-reset", include_str!("icons/bundled/hard-reset.svg")),
    (
        "hardware-cabinet",
        include_str!("icons/bundled/hardware-cabinet.svg"),
    ),
    ("hat-man", include_str!("icons/bundled/hat-man.svg")),
    (
        "hat-man-filled",
        include_str!("icons/bundled/hat-man-filled.svg"),
    ),
    ("heading", include_str!("icons/bundled/heading.svg")),
    ("health", include_str!("icons/bundled/health.svg")),
    (
        "health-filled",
        include_str!("icons/bundled/health-filled.svg"),
    ),
    ("heart", include_str!("icons/bundled/heart.svg")),
    (
        "heart-filled",
        include_str!("icons/bundled/heart-filled.svg"),
    ),
    (
        "heat-map-chart",
        include_str!("icons/bundled/heat-map-chart.svg"),
    ),
    (
        "heat-map-chart-filled",
        include_str!("icons/bundled/heat-map-chart-filled.svg"),
    ),
    ("height", include_str!("icons/bundled/height.svg")),
    (
        "helmet-safety",
        include_str!("icons/bundled/helmet-safety.svg"),
    ),
    (
        "helmet-safety-filled",
        include_str!("icons/bundled/helmet-safety-filled.svg"),
    ),
    ("hexagon", include_str!("icons/bundled/hexagon.svg")),
    (
        "hexagon-filled",
        include_str!("icons/bundled/hexagon-filled.svg"),
    ),
    (
        "hexagon-vertical-bars",
        include_str!("icons/bundled/hexagon-vertical-bars.svg"),
    ),
    (
        "hexagon-vertical-bars-database",
        include_str!("icons/bundled/hexagon-vertical-bars-database.svg"),
    ),
    (
        "hexagon-vertical-bars-database-filled",
        include_str!("icons/bundled/hexagon-vertical-bars-database-filled.svg"),
    ),
    (
        "hexagon-vertical-bars-filled",
        include_str!("icons/bundled/hexagon-vertical-bars-filled.svg"),
    ),
    ("hierarchy", include_str!("icons/bundled/hierarchy.svg")),
    ("highlight", include_str!("icons/bundled/highlight.svg")),
    (
        "highlight-filled",
        include_str!("icons/bundled/highlight-filled.svg"),
    ),
    ("history", include_str!("icons/bundled/history.svg")),
    (
        "history-list",
        include_str!("icons/bundled/history-list.svg"),
    ),
    ("home", include_str!("icons/bundled/home.svg")),
    ("home-filled", include_str!("icons/bundled/home-filled.svg")),
    ("hourglass", include_str!("icons/bundled/hourglass.svg")),
    (
        "hourglass-empty",
        include_str!("icons/bundled/hourglass-empty.svg"),
    ),
    (
        "hourglass-end",
        include_str!("icons/bundled/hourglass-end.svg"),
    ),
    (
        "hourglass-filled",
        include_str!("icons/bundled/hourglass-filled.svg"),
    ),
    (
        "hourglass-start",
        include_str!("icons/bundled/hourglass-start.svg"),
    ),
    ("id", include_str!("icons/bundled/id.svg")),
    ("id-filled", include_str!("icons/bundled/id-filled.svg")),
    ("image", include_str!("icons/bundled/image.svg")),
    ("image-ai", include_str!("icons/bundled/image-ai.svg")),
    (
        "image-ai-filled",
        include_str!("icons/bundled/image-ai-filled.svg"),
    ),
    (
        "image-filled",
        include_str!("icons/bundled/image-filled.svg"),
    ),
    ("import", include_str!("icons/bundled/import.svg")),
    (
        "import-check",
        include_str!("icons/bundled/import-check.svg"),
    ),
    (
        "import-failed",
        include_str!("icons/bundled/import-failed.svg"),
    ),
    (
        "import-progress",
        include_str!("icons/bundled/import-progress.svg"),
    ),
    (
        "incompatible",
        include_str!("icons/bundled/incompatible.svg"),
    ),
    ("indicator", include_str!("icons/bundled/indicator.svg")),
    (
        "indicator-filled",
        include_str!("icons/bundled/indicator-filled.svg"),
    ),
    ("info", include_str!("icons/bundled/info.svg")),
    ("info-feed", include_str!("icons/bundled/info-feed.svg")),
    ("info-filled", include_str!("icons/bundled/info-filled.svg")),
    (
        "info-multiple",
        include_str!("icons/bundled/info-multiple.svg"),
    ),
    (
        "info-multiple-filled",
        include_str!("icons/bundled/info-multiple-filled.svg"),
    ),
    ("ingestion", include_str!("icons/bundled/ingestion.svg")),
    (
        "ingestion-report",
        include_str!("icons/bundled/ingestion-report.svg"),
    ),
    ("ink-pen", include_str!("icons/bundled/ink-pen.svg")),
    ("ink-pen-add", include_str!("icons/bundled/ink-pen-add.svg")),
    (
        "ink-pen-filled",
        include_str!("icons/bundled/ink-pen-filled.svg"),
    ),
    ("inquiry", include_str!("icons/bundled/inquiry.svg")),
    (
        "inquiry-filled",
        include_str!("icons/bundled/inquiry-filled.svg"),
    ),
    (
        "inquiry-mail",
        include_str!("icons/bundled/inquiry-mail.svg"),
    ),
    ("interpreter", include_str!("icons/bundled/interpreter.svg")),
    (
        "interpreter-filled",
        include_str!("icons/bundled/interpreter-filled.svg"),
    ),
    ("ipcs", include_str!("icons/bundled/ipcs.svg")),
    (
        "item-details",
        include_str!("icons/bundled/item-details.svg"),
    ),
    (
        "item-details-filled",
        include_str!("icons/bundled/item-details-filled.svg"),
    ),
    ("java-script", include_str!("icons/bundled/java-script.svg")),
    (
        "java-script-connection",
        include_str!("icons/bundled/java-script-connection.svg"),
    ),
    ("jigsaw", include_str!("icons/bundled/jigsaw.svg")),
    (
        "jigsaw-details",
        include_str!("icons/bundled/jigsaw-details.svg"),
    ),
    (
        "jigsaw-details-filled",
        include_str!("icons/bundled/jigsaw-details-filled.svg"),
    ),
    (
        "jigsaw-filled",
        include_str!("icons/bundled/jigsaw-filled.svg"),
    ),
    ("join", include_str!("icons/bundled/join.svg")),
    (
        "json-document",
        include_str!("icons/bundled/json-document.svg"),
    ),
    (
        "json-document-filled",
        include_str!("icons/bundled/json-document-filled.svg"),
    ),
    ("key", include_str!("icons/bundled/key.svg")),
    ("keyboard", include_str!("icons/bundled/keyboard.svg")),
    (
        "keyboard-docked-bottom",
        include_str!("icons/bundled/keyboard-docked-bottom.svg"),
    ),
    (
        "keyboard-floating",
        include_str!("icons/bundled/keyboard-floating.svg"),
    ),
    (
        "keyboard-framed",
        include_str!("icons/bundled/keyboard-framed.svg"),
    ),
    ("kinematics", include_str!("icons/bundled/kinematics.svg")),
    ("kpi", include_str!("icons/bundled/kpi.svg")),
    ("kpi-filled", include_str!("icons/bundled/kpi-filled.svg")),
    ("label", include_str!("icons/bundled/label.svg")),
    (
        "label-filled",
        include_str!("icons/bundled/label-filled.svg"),
    ),
    ("language", include_str!("icons/bundled/language.svg")),
    (
        "language-filled",
        include_str!("icons/bundled/language-filled.svg"),
    ),
    ("layers", include_str!("icons/bundled/layers.svg")),
    (
        "layers-filled",
        include_str!("icons/bundled/layers-filled.svg"),
    ),
    (
        "leading-axis-proxy",
        include_str!("icons/bundled/leading-axis-proxy.svg"),
    ),
    ("leaf", include_str!("icons/bundled/leaf.svg")),
    ("legal", include_str!("icons/bundled/legal.svg")),
    (
        "legal-circle",
        include_str!("icons/bundled/legal-circle.svg"),
    ),
    (
        "legal-circle-filled",
        include_str!("icons/bundled/legal-circle-filled.svg"),
    ),
    ("library", include_str!("icons/bundled/library.svg")),
    ("library-new", include_str!("icons/bundled/library-new.svg")),
    ("license", include_str!("icons/bundled/license.svg")),
    ("light-dark", include_str!("icons/bundled/light-dark.svg")),
    (
        "limits-cancelled",
        include_str!("icons/bundled/limits-cancelled.svg"),
    ),
    (
        "limits-check",
        include_str!("icons/bundled/limits-check.svg"),
    ),
    (
        "line-cap-flat",
        include_str!("icons/bundled/line-cap-flat.svg"),
    ),
    (
        "line-cap-round",
        include_str!("icons/bundled/line-cap-round.svg"),
    ),
    (
        "line-cap-square",
        include_str!("icons/bundled/line-cap-square.svg"),
    ),
    ("line-dash", include_str!("icons/bundled/line-dash.svg")),
    (
        "line-dash-dot",
        include_str!("icons/bundled/line-dash-dot.svg"),
    ),
    (
        "line-dash-dot-dot",
        include_str!("icons/bundled/line-dash-dot-dot.svg"),
    ),
    (
        "line-diagonal",
        include_str!("icons/bundled/line-diagonal.svg"),
    ),
    ("line-dot", include_str!("icons/bundled/line-dot.svg")),
    ("line-solid", include_str!("icons/bundled/line-solid.svg")),
    ("linechart", include_str!("icons/bundled/linechart.svg")),
    ("link", include_str!("icons/bundled/link.svg")),
    ("link-break", include_str!("icons/bundled/link-break.svg")),
    (
        "link-diagonal",
        include_str!("icons/bundled/link-diagonal.svg"),
    ),
    ("list", include_str!("icons/bundled/list.svg")),
    ("list-add", include_str!("icons/bundled/list-add.svg")),
    (
        "list-graphics",
        include_str!("icons/bundled/list-graphics.svg"),
    ),
    (
        "list-graphics-text",
        include_str!("icons/bundled/list-graphics-text.svg"),
    ),
    (
        "list-percentage",
        include_str!("icons/bundled/list-percentage.svg"),
    ),
    ("list-remove", include_str!("icons/bundled/list-remove.svg")),
    ("list-sorted", include_str!("icons/bundled/list-sorted.svg")),
    (
        "list-sorted-alt",
        include_str!("icons/bundled/list-sorted-alt.svg"),
    ),
    ("list-text", include_str!("icons/bundled/list-text.svg")),
    ("live-feed", include_str!("icons/bundled/live-feed.svg")),
    (
        "live-schedule",
        include_str!("icons/bundled/live-schedule.svg"),
    ),
    ("location", include_str!("icons/bundled/location.svg")),
    (
        "location-filled",
        include_str!("icons/bundled/location-filled.svg"),
    ),
    ("lock", include_str!("icons/bundled/lock.svg")),
    ("lock-check", include_str!("icons/bundled/lock-check.svg")),
    ("lock-filled", include_str!("icons/bundled/lock-filled.svg")),
    ("lock-key", include_str!("icons/bundled/lock-key.svg")),
    (
        "lock-key-filled",
        include_str!("icons/bundled/lock-key-filled.svg"),
    ),
    ("log", include_str!("icons/bundled/log.svg")),
    ("log-in", include_str!("icons/bundled/log-in.svg")),
    ("log-out", include_str!("icons/bundled/log-out.svg")),
    (
        "logic-diagram",
        include_str!("icons/bundled/logic-diagram.svg"),
    ),
    ("longer", include_str!("icons/bundled/longer.svg")),
    ("longer-ai", include_str!("icons/bundled/longer-ai.svg")),
    ("lower-limit", include_str!("icons/bundled/lower-limit.svg")),
    ("machine-a", include_str!("icons/bundled/machine-a.svg")),
    (
        "machine-a-filled",
        include_str!("icons/bundled/machine-a-filled.svg"),
    ),
    ("machine-b", include_str!("icons/bundled/machine-b.svg")),
    (
        "machine-b-filled",
        include_str!("icons/bundled/machine-b-filled.svg"),
    ),
    ("machine-c", include_str!("icons/bundled/machine-c.svg")),
    (
        "machine-c-filled",
        include_str!("icons/bundled/machine-c-filled.svg"),
    ),
    ("magnet", include_str!("icons/bundled/magnet.svg")),
    (
        "magnet-cancelled",
        include_str!("icons/bundled/magnet-cancelled.svg"),
    ),
    ("mail", include_str!("icons/bundled/mail.svg")),
    (
        "mail-alarm-analog",
        include_str!("icons/bundled/mail-alarm-analog.svg"),
    ),
    (
        "mail-alarm-classes",
        include_str!("icons/bundled/mail-alarm-classes.svg"),
    ),
    (
        "mail-alarm-discrete",
        include_str!("icons/bundled/mail-alarm-discrete.svg"),
    ),
    ("mail-filled", include_str!("icons/bundled/mail-filled.svg")),
    ("maintenance", include_str!("icons/bundled/maintenance.svg")),
    (
        "maintenance-documents",
        include_str!("icons/bundled/maintenance-documents.svg"),
    ),
    (
        "maintenance-filled",
        include_str!("icons/bundled/maintenance-filled.svg"),
    ),
    (
        "maintenance-info",
        include_str!("icons/bundled/maintenance-info.svg"),
    ),
    (
        "maintenance-info-filled",
        include_str!("icons/bundled/maintenance-info-filled.svg"),
    ),
    (
        "maintenance-octagon",
        include_str!("icons/bundled/maintenance-octagon.svg"),
    ),
    (
        "maintenance-octagon-filled",
        include_str!("icons/bundled/maintenance-octagon-filled.svg"),
    ),
    (
        "maintenance-rhomb",
        include_str!("icons/bundled/maintenance-rhomb.svg"),
    ),
    (
        "maintenance-rhomb-filled",
        include_str!("icons/bundled/maintenance-rhomb-filled.svg"),
    ),
    (
        "maintenance-square",
        include_str!("icons/bundled/maintenance-square.svg"),
    ),
    (
        "maintenance-square-filled",
        include_str!("icons/bundled/maintenance-square-filled.svg"),
    ),
    (
        "maintenance-triangle",
        include_str!("icons/bundled/maintenance-triangle.svg"),
    ),
    (
        "maintenance-triangle-filled",
        include_str!("icons/bundled/maintenance-triangle-filled.svg"),
    ),
    (
        "maintenance-warning",
        include_str!("icons/bundled/maintenance-warning.svg"),
    ),
    (
        "maintenance-warning-filled",
        include_str!("icons/bundled/maintenance-warning-filled.svg"),
    ),
    ("mandatory", include_str!("icons/bundled/mandatory.svg")),
    (
        "mandatory-done",
        include_str!("icons/bundled/mandatory-done.svg"),
    ),
    ("map", include_str!("icons/bundled/map.svg")),
    ("map-alt-1", include_str!("icons/bundled/map-alt-1.svg")),
    (
        "map-alt-1-filled",
        include_str!("icons/bundled/map-alt-1-filled.svg"),
    ),
    ("map-alt-2", include_str!("icons/bundled/map-alt-2.svg")),
    (
        "map-alt-2-filled",
        include_str!("icons/bundled/map-alt-2-filled.svg"),
    ),
    ("map-alt-3", include_str!("icons/bundled/map-alt-3.svg")),
    (
        "map-alt-3-filled",
        include_str!("icons/bundled/map-alt-3-filled.svg"),
    ),
    ("maximize", include_str!("icons/bundled/maximize.svg")),
    (
        "measuring-input",
        include_str!("icons/bundled/measuring-input.svg"),
    ),
    (
        "media-player",
        include_str!("icons/bundled/media-player.svg"),
    ),
    ("microphone", include_str!("icons/bundled/microphone.svg")),
    (
        "microphone-cancelled",
        include_str!("icons/bundled/microphone-cancelled.svg"),
    ),
    (
        "microphone-cancelled-filled",
        include_str!("icons/bundled/microphone-cancelled-filled.svg"),
    ),
    (
        "microphone-filled",
        include_str!("icons/bundled/microphone-filled.svg"),
    ),
    ("minimize", include_str!("icons/bundled/minimize.svg")),
    ("minus", include_str!("icons/bundled/minus.svg")),
    (
        "missing-symbol",
        include_str!("icons/bundled/missing-symbol.svg"),
    ),
    ("mix", include_str!("icons/bundled/mix.svg")),
    (
        "mobile-phone",
        include_str!("icons/bundled/mobile-phone.svg"),
    ),
    (
        "mobile-phone-filled",
        include_str!("icons/bundled/mobile-phone-filled.svg"),
    ),
    ("money", include_str!("icons/bundled/money.svg")),
    ("money-alt", include_str!("icons/bundled/money-alt.svg")),
    ("monitor", include_str!("icons/bundled/monitor.svg")),
    (
        "monitor-filled",
        include_str!("icons/bundled/monitor-filled.svg"),
    ),
    (
        "monitor-trend",
        include_str!("icons/bundled/monitor-trend.svg"),
    ),
    ("monitoring", include_str!("icons/bundled/monitoring.svg")),
    (
        "monitoring-add",
        include_str!("icons/bundled/monitoring-add.svg"),
    ),
    ("monitorings", include_str!("icons/bundled/monitorings.svg")),
    ("moon", include_str!("icons/bundled/moon.svg")),
    ("moon-filled", include_str!("icons/bundled/moon-filled.svg")),
    ("more-menu", include_str!("icons/bundled/more-menu.svg")),
    ("mouse-click", include_str!("icons/bundled/mouse-click.svg")),
    (
        "mouse-click-filled",
        include_str!("icons/bundled/mouse-click-filled.svg"),
    ),
    (
        "mouse-select",
        include_str!("icons/bundled/mouse-select.svg"),
    ),
    (
        "mouse-select-filled",
        include_str!("icons/bundled/mouse-select-filled.svg"),
    ),
    ("move", include_str!("icons/bundled/move.svg")),
    (
        "move-horizontally",
        include_str!("icons/bundled/move-horizontally.svg"),
    ),
    (
        "move-layer-down",
        include_str!("icons/bundled/move-layer-down.svg"),
    ),
    (
        "move-layer-up",
        include_str!("icons/bundled/move-layer-up.svg"),
    ),
    (
        "move-vertically",
        include_str!("icons/bundled/move-vertically.svg"),
    ),
    ("movie", include_str!("icons/bundled/movie.svg")),
    ("movie-ai", include_str!("icons/bundled/movie-ai.svg")),
    (
        "movie-ai-filled",
        include_str!("icons/bundled/movie-ai-filled.svg"),
    ),
    (
        "movie-filled",
        include_str!("icons/bundled/movie-filled.svg"),
    ),
    (
        "mp4-document",
        include_str!("icons/bundled/mp4-document.svg"),
    ),
    ("mqtt", include_str!("icons/bundled/mqtt.svg")),
    ("mqtt-filled", include_str!("icons/bundled/mqtt-filled.svg")),
    ("music-note", include_str!("icons/bundled/music-note.svg")),
    (
        "namur-check-function",
        include_str!("icons/bundled/namur-check-function.svg"),
    ),
    (
        "namur-check-function-filled",
        include_str!("icons/bundled/namur-check-function-filled.svg"),
    ),
    (
        "namur-diagnostics-passive",
        include_str!("icons/bundled/namur-diagnostics-passive.svg"),
    ),
    (
        "namur-diagnostics-passive-filled",
        include_str!("icons/bundled/namur-diagnostics-passive-filled.svg"),
    ),
    (
        "namur-failure",
        include_str!("icons/bundled/namur-failure.svg"),
    ),
    (
        "namur-failure-filled",
        include_str!("icons/bundled/namur-failure-filled.svg"),
    ),
    (
        "namur-maintenance-required",
        include_str!("icons/bundled/namur-maintenance-required.svg"),
    ),
    (
        "namur-maintenance-required-filled",
        include_str!("icons/bundled/namur-maintenance-required-filled.svg"),
    ),
    ("namur-ok", include_str!("icons/bundled/namur-ok.svg")),
    (
        "namur-ok-filled",
        include_str!("icons/bundled/namur-ok-filled.svg"),
    ),
    (
        "namur-out-of-spec",
        include_str!("icons/bundled/namur-out-of-spec.svg"),
    ),
    (
        "namur-out-of-spec-filled",
        include_str!("icons/bundled/namur-out-of-spec-filled.svg"),
    ),
    ("navigation", include_str!("icons/bundled/navigation.svg")),
    (
        "navigation-filled",
        include_str!("icons/bundled/navigation-filled.svg"),
    ),
    (
        "navigation-left",
        include_str!("icons/bundled/navigation-left.svg"),
    ),
    (
        "navigation-left-hide",
        include_str!("icons/bundled/navigation-left-hide.svg"),
    ),
    (
        "navigation-right",
        include_str!("icons/bundled/navigation-right.svg"),
    ),
    (
        "navigation-right-hide",
        include_str!("icons/bundled/navigation-right-hide.svg"),
    ),
    (
        "network-device",
        include_str!("icons/bundled/network-device.svg"),
    ),
    (
        "network-device-filled",
        include_str!("icons/bundled/network-device-filled.svg"),
    ),
    (
        "network-device-play",
        include_str!("icons/bundled/network-device-play.svg"),
    ),
    (
        "network-device-play-filled",
        include_str!("icons/bundled/network-device-play-filled.svg"),
    ),
    (
        "network-wired",
        include_str!("icons/bundled/network-wired.svg"),
    ),
    (
        "network-wired-wireless",
        include_str!("icons/bundled/network-wired-wireless.svg"),
    ),
    ("no-filter", include_str!("icons/bundled/no-filter.svg")),
    (
        "no-filter-filled",
        include_str!("icons/bundled/no-filter-filled.svg"),
    ),
    ("no-image", include_str!("icons/bundled/no-image.svg")),
    ("note", include_str!("icons/bundled/note.svg")),
    ("note-filled", include_str!("icons/bundled/note-filled.svg")),
    ("notebook", include_str!("icons/bundled/notebook.svg")),
    (
        "notebook-filled",
        include_str!("icons/bundled/notebook-filled.svg"),
    ),
    (
        "notification",
        include_str!("icons/bundled/notification.svg"),
    ),
    (
        "notification-filled",
        include_str!("icons/bundled/notification-filled.svg"),
    ),
    (
        "notifications",
        include_str!("icons/bundled/notifications.svg"),
    ),
    (
        "notifications-filled",
        include_str!("icons/bundled/notifications-filled.svg"),
    ),
    ("objects", include_str!("icons/bundled/objects.svg")),
    (
        "objects-tree",
        include_str!("icons/bundled/objects-tree.svg"),
    ),
    ("octagon", include_str!("icons/bundled/octagon.svg")),
    (
        "octagon-filled",
        include_str!("icons/bundled/octagon-filled.svg"),
    ),
    ("ontology", include_str!("icons/bundled/ontology.svg")),
    (
        "ontology-filled",
        include_str!("icons/bundled/ontology-filled.svg"),
    ),
    (
        "open-external",
        include_str!("icons/bundled/open-external.svg"),
    ),
    ("open-file", include_str!("icons/bundled/open-file.svg")),
    (
        "open-file-filled",
        include_str!("icons/bundled/open-file-filled.svg"),
    ),
    (
        "operate-plant",
        include_str!("icons/bundled/operate-plant.svg"),
    ),
    (
        "operate-plant-filled",
        include_str!("icons/bundled/operate-plant-filled.svg"),
    ),
    (
        "operating-system",
        include_str!("icons/bundled/operating-system.svg"),
    ),
    (
        "operating-system-add",
        include_str!("icons/bundled/operating-system-add.svg"),
    ),
    (
        "operating-system-settings",
        include_str!("icons/bundled/operating-system-settings.svg"),
    ),
    ("optimize", include_str!("icons/bundled/optimize.svg")),
    ("optimize-ai", include_str!("icons/bundled/optimize-ai.svg")),
    ("output-cam", include_str!("icons/bundled/output-cam.svg")),
    (
        "p-and-i-symbols",
        include_str!("icons/bundled/p-and-i-symbols.svg"),
    ),
    ("p-i-diagram", include_str!("icons/bundled/p-i-diagram.svg")),
    ("package", include_str!("icons/bundled/package.svg")),
    (
        "package-filled",
        include_str!("icons/bundled/package-filled.svg"),
    ),
    ("palette", include_str!("icons/bundled/palette.svg")),
    (
        "palette-filled",
        include_str!("icons/bundled/palette-filled.svg"),
    ),
    ("pan", include_str!("icons/bundled/pan.svg")),
    ("panel-ipc", include_str!("icons/bundled/panel-ipc.svg")),
    (
        "panel-ipc-fail",
        include_str!("icons/bundled/panel-ipc-fail.svg"),
    ),
    (
        "panel-ipc-question",
        include_str!("icons/bundled/panel-ipc-question.svg"),
    ),
    (
        "panel-ipc-success",
        include_str!("icons/bundled/panel-ipc-success.svg"),
    ),
    ("parameter", include_str!("icons/bundled/parameter.svg")),
    ("paste", include_str!("icons/bundled/paste.svg")),
    (
        "paste-filled",
        include_str!("icons/bundled/paste-filled.svg"),
    ),
    ("pause", include_str!("icons/bundled/pause.svg")),
    ("pc-tower", include_str!("icons/bundled/pc-tower.svg")),
    (
        "pc-tower-filled",
        include_str!("icons/bundled/pc-tower-filled.svg"),
    ),
    (
        "pc-tower-settings",
        include_str!("icons/bundled/pc-tower-settings.svg"),
    ),
    (
        "pc-tower-settings-filled",
        include_str!("icons/bundled/pc-tower-settings-filled.svg"),
    ),
    (
        "pdf-document",
        include_str!("icons/bundled/pdf-document.svg"),
    ),
    (
        "pdf-document-filled",
        include_str!("icons/bundled/pdf-document-filled.svg"),
    ),
    ("pen", include_str!("icons/bundled/pen.svg")),
    ("pen-ai", include_str!("icons/bundled/pen-ai.svg")),
    (
        "pen-ai-filled",
        include_str!("icons/bundled/pen-ai-filled.svg"),
    ),
    (
        "pen-cancelled",
        include_str!("icons/bundled/pen-cancelled.svg"),
    ),
    (
        "pen-cancelled-filled",
        include_str!("icons/bundled/pen-cancelled-filled.svg"),
    ),
    ("pen-filled", include_str!("icons/bundled/pen-filled.svg")),
    ("phone", include_str!("icons/bundled/phone.svg")),
    (
        "phone-filled",
        include_str!("icons/bundled/phone-filled.svg"),
    ),
    (
        "photo-camera",
        include_str!("icons/bundled/photo-camera.svg"),
    ),
    (
        "photo-camera-add",
        include_str!("icons/bundled/photo-camera-add.svg"),
    ),
    (
        "photo-camera-cancelled",
        include_str!("icons/bundled/photo-camera-cancelled.svg"),
    ),
    (
        "photo-camera-cancelled-filled",
        include_str!("icons/bundled/photo-camera-cancelled-filled.svg"),
    ),
    (
        "photo-camera-filled",
        include_str!("icons/bundled/photo-camera-filled.svg"),
    ),
    (
        "photo-cameras",
        include_str!("icons/bundled/photo-cameras.svg"),
    ),
    ("piechart", include_str!("icons/bundled/piechart.svg")),
    ("piechart-ai", include_str!("icons/bundled/piechart-ai.svg")),
    (
        "piechart-ai-filled",
        include_str!("icons/bundled/piechart-ai-filled.svg"),
    ),
    (
        "piechart-filled",
        include_str!("icons/bundled/piechart-filled.svg"),
    ),
    ("pin", include_str!("icons/bundled/pin.svg")),
    (
        "pin-cancelled",
        include_str!("icons/bundled/pin-cancelled.svg"),
    ),
    (
        "pin-cancelled-filled",
        include_str!("icons/bundled/pin-cancelled-filled.svg"),
    ),
    ("pin-filled", include_str!("icons/bundled/pin-filled.svg")),
    ("plant", include_str!("icons/bundled/plant.svg")),
    (
        "plant-details",
        include_str!("icons/bundled/plant-details.svg"),
    ),
    (
        "plant-details-filled",
        include_str!("icons/bundled/plant-details-filled.svg"),
    ),
    (
        "plant-filled",
        include_str!("icons/bundled/plant-filled.svg"),
    ),
    (
        "plant-handbook",
        include_str!("icons/bundled/plant-handbook.svg"),
    ),
    (
        "plant-handbook-filled",
        include_str!("icons/bundled/plant-handbook-filled.svg"),
    ),
    (
        "plant-search",
        include_str!("icons/bundled/plant-search.svg"),
    ),
    (
        "plant-search-filled",
        include_str!("icons/bundled/plant-search-filled.svg"),
    ),
    (
        "plant-security",
        include_str!("icons/bundled/plant-security.svg"),
    ),
    (
        "plant-security-filled",
        include_str!("icons/bundled/plant-security-filled.svg"),
    ),
    (
        "plant-settings",
        include_str!("icons/bundled/plant-settings.svg"),
    ),
    (
        "plant-settings-filled",
        include_str!("icons/bundled/plant-settings-filled.svg"),
    ),
    ("plant-user", include_str!("icons/bundled/plant-user.svg")),
    (
        "plant-user-filled",
        include_str!("icons/bundled/plant-user-filled.svg"),
    ),
    ("plants", include_str!("icons/bundled/plants.svg")),
    (
        "plants-filled",
        include_str!("icons/bundled/plants-filled.svg"),
    ),
    ("play", include_str!("icons/bundled/play.svg")),
    ("play-filled", include_str!("icons/bundled/play-filled.svg")),
    ("play-pause", include_str!("icons/bundled/play-pause.svg")),
    (
        "play-pause-filled",
        include_str!("icons/bundled/play-pause-filled.svg"),
    ),
    (
        "play-stepwise",
        include_str!("icons/bundled/play-stepwise.svg"),
    ),
    (
        "play-stepwise-filled",
        include_str!("icons/bundled/play-stepwise-filled.svg"),
    ),
    ("plc", include_str!("icons/bundled/plc.svg")),
    ("plc-device", include_str!("icons/bundled/plc-device.svg")),
    (
        "plc-device-tag",
        include_str!("icons/bundled/plc-device-tag.svg"),
    ),
    (
        "plc-device-user-data-type",
        include_str!("icons/bundled/plc-device-user-data-type.svg"),
    ),
    ("plc-tag", include_str!("icons/bundled/plc-tag.svg")),
    (
        "plc-user-data-type",
        include_str!("icons/bundled/plc-user-data-type.svg"),
    ),
    ("plus", include_str!("icons/bundled/plus.svg")),
    (
        "plus-minus-times-divide",
        include_str!("icons/bundled/plus-minus-times-divide.svg"),
    ),
    ("point-up", include_str!("icons/bundled/point-up.svg")),
    (
        "point-up-filled",
        include_str!("icons/bundled/point-up-filled.svg"),
    ),
    ("polar-plot", include_str!("icons/bundled/polar-plot.svg")),
    ("polarchart", include_str!("icons/bundled/polarchart.svg")),
    (
        "polarchart-filled",
        include_str!("icons/bundled/polarchart-filled.svg"),
    ),
    ("polygon", include_str!("icons/bundled/polygon.svg")),
    (
        "polygon-filled",
        include_str!("icons/bundled/polygon-filled.svg"),
    ),
    (
        "polygon-line",
        include_str!("icons/bundled/polygon-line.svg"),
    ),
    (
        "power-supply",
        include_str!("icons/bundled/power-supply.svg"),
    ),
    (
        "ppt-document",
        include_str!("icons/bundled/ppt-document.svg"),
    ),
    ("print", include_str!("icons/bundled/print.svg")),
    (
        "print-filled",
        include_str!("icons/bundled/print-filled.svg"),
    ),
    ("prio-high", include_str!("icons/bundled/prio-high.svg")),
    ("prio-low", include_str!("icons/bundled/prio-low.svg")),
    ("prio-middle", include_str!("icons/bundled/prio-middle.svg")),
    (
        "process-control",
        include_str!("icons/bundled/process-control.svg"),
    ),
    ("product", include_str!("icons/bundled/product.svg")),
    (
        "product-catalog",
        include_str!("icons/bundled/product-catalog.svg"),
    ),
    (
        "product-management",
        include_str!("icons/bundled/product-management.svg"),
    ),
    ("project", include_str!("icons/bundled/project.svg")),
    (
        "project-arrow-diagonal-top-right",
        include_str!("icons/bundled/project-arrow-diagonal-top-right.svg"),
    ),
    (
        "project-arrow-left",
        include_str!("icons/bundled/project-arrow-left.svg"),
    ),
    (
        "project-arrow-right",
        include_str!("icons/bundled/project-arrow-right.svg"),
    ),
    (
        "project-close",
        include_str!("icons/bundled/project-close.svg"),
    ),
    (
        "project-configuration",
        include_str!("icons/bundled/project-configuration.svg"),
    ),
    (
        "project-duplicate",
        include_str!("icons/bundled/project-duplicate.svg"),
    ),
    (
        "project-history",
        include_str!("icons/bundled/project-history.svg"),
    ),
    ("project-new", include_str!("icons/bundled/project-new.svg")),
    (
        "project-scenarios",
        include_str!("icons/bundled/project-scenarios.svg"),
    ),
    (
        "project-server",
        include_str!("icons/bundled/project-server.svg"),
    ),
    (
        "project-server-filled",
        include_str!("icons/bundled/project-server-filled.svg"),
    ),
    (
        "project-settings",
        include_str!("icons/bundled/project-settings.svg"),
    ),
    (
        "project-simulation",
        include_str!("icons/bundled/project-simulation.svg"),
    ),
    ("projects", include_str!("icons/bundled/projects.svg")),
    (
        "projects-close",
        include_str!("icons/bundled/projects-close.svg"),
    ),
    ("prompt", include_str!("icons/bundled/prompt.svg")),
    ("prompt-new", include_str!("icons/bundled/prompt-new.svg")),
    ("prompts", include_str!("icons/bundled/prompts.svg")),
    ("protocol", include_str!("icons/bundled/protocol.svg")),
    ("publish", include_str!("icons/bundled/publish.svg")),
    (
        "publish-document",
        include_str!("icons/bundled/publish-document.svg"),
    ),
    ("qr-code", include_str!("icons/bundled/qr-code.svg")),
    (
        "quality-report",
        include_str!("icons/bundled/quality-report.svg"),
    ),
    ("question", include_str!("icons/bundled/question.svg")),
    ("question-ai", include_str!("icons/bundled/question-ai.svg")),
    (
        "question-ai-filled",
        include_str!("icons/bundled/question-ai-filled.svg"),
    ),
    (
        "question-filled",
        include_str!("icons/bundled/question-filled.svg"),
    ),
    (
        "question-mark",
        include_str!("icons/bundled/question-mark.svg"),
    ),
    ("quote", include_str!("icons/bundled/quote.svg")),
    ("rack-ipc", include_str!("icons/bundled/rack-ipc.svg")),
    (
        "rack-ipc-fail",
        include_str!("icons/bundled/rack-ipc-fail.svg"),
    ),
    (
        "rack-ipc-question",
        include_str!("icons/bundled/rack-ipc-question.svg"),
    ),
    (
        "rack-ipc-success",
        include_str!("icons/bundled/rack-ipc-success.svg"),
    ),
    ("radarchart", include_str!("icons/bundled/radarchart.svg")),
    ("radio-waves", include_str!("icons/bundled/radio-waves.svg")),
    (
        "radio-waves-off",
        include_str!("icons/bundled/radio-waves-off.svg"),
    ),
    (
        "radio-waves-warning",
        include_str!("icons/bundled/radio-waves-warning.svg"),
    ),
    ("random", include_str!("icons/bundled/random.svg")),
    (
        "random-filled",
        include_str!("icons/bundled/random-filled.svg"),
    ),
    ("reboot", include_str!("icons/bundled/reboot.svg")),
    ("record", include_str!("icons/bundled/record.svg")),
    (
        "record-filled",
        include_str!("icons/bundled/record-filled.svg"),
    ),
    ("rectangle", include_str!("icons/bundled/rectangle.svg")),
    (
        "rectangle-filled",
        include_str!("icons/bundled/rectangle-filled.svg"),
    ),
    ("redo", include_str!("icons/bundled/redo.svg")),
    ("reference", include_str!("icons/bundled/reference.svg")),
    (
        "reference-point-bottom-left",
        include_str!("icons/bundled/reference-point-bottom-left.svg"),
    ),
    (
        "reference-point-bottom-right",
        include_str!("icons/bundled/reference-point-bottom-right.svg"),
    ),
    (
        "reference-point-centered",
        include_str!("icons/bundled/reference-point-centered.svg"),
    ),
    (
        "reference-point-top-left",
        include_str!("icons/bundled/reference-point-top-left.svg"),
    ),
    (
        "reference-point-top-right",
        include_str!("icons/bundled/reference-point-top-right.svg"),
    ),
    ("refresh", include_str!("icons/bundled/refresh.svg")),
    (
        "refresh-arrow-down",
        include_str!("icons/bundled/refresh-arrow-down.svg"),
    ),
    (
        "refresh-cancelled",
        include_str!("icons/bundled/refresh-cancelled.svg"),
    ),
    (
        "refresh-exclamation",
        include_str!("icons/bundled/refresh-exclamation.svg"),
    ),
    (
        "refresh-settings",
        include_str!("icons/bundled/refresh-settings.svg"),
    ),
    ("reload", include_str!("icons/bundled/reload.svg")),
    (
        "remote-access",
        include_str!("icons/bundled/remote-access.svg"),
    ),
    (
        "remove-application",
        include_str!("icons/bundled/remove-application.svg"),
    ),
    (
        "remove-circle",
        include_str!("icons/bundled/remove-circle.svg"),
    ),
    (
        "remove-circle-filled",
        include_str!("icons/bundled/remove-circle-filled.svg"),
    ),
    ("remove-eye", include_str!("icons/bundled/remove-eye.svg")),
    (
        "remove-eye-filled",
        include_str!("icons/bundled/remove-eye-filled.svg"),
    ),
    ("rename", include_str!("icons/bundled/rename.svg")),
    ("reorder", include_str!("icons/bundled/reorder.svg")),
    ("replace", include_str!("icons/bundled/replace.svg")),
    (
        "report-barchart",
        include_str!("icons/bundled/report-barchart.svg"),
    ),
    (
        "report-general",
        include_str!("icons/bundled/report-general.svg"),
    ),
    (
        "report-linechart",
        include_str!("icons/bundled/report-linechart.svg"),
    ),
    ("report-text", include_str!("icons/bundled/report-text.svg")),
    ("reset", include_str!("icons/bundled/reset.svg")),
    ("restart", include_str!("icons/bundled/restart.svg")),
    ("restore", include_str!("icons/bundled/restore.svg")),
    (
        "restore-backup",
        include_str!("icons/bundled/restore-backup.svg"),
    ),
    (
        "restore-backup-filled",
        include_str!("icons/bundled/restore-backup-filled.svg"),
    ),
    (
        "restore-backup-pc",
        include_str!("icons/bundled/restore-backup-pc.svg"),
    ),
    ("rewind", include_str!("icons/bundled/rewind.svg")),
    (
        "rewind-filled",
        include_str!("icons/bundled/rewind-filled.svg"),
    ),
    ("rhomb", include_str!("icons/bundled/rhomb.svg")),
    (
        "rhomb-filled",
        include_str!("icons/bundled/rhomb-filled.svg"),
    ),
    ("road", include_str!("icons/bundled/road.svg")),
    ("road-filled", include_str!("icons/bundled/road-filled.svg")),
    ("robotic-arm", include_str!("icons/bundled/robotic-arm.svg")),
    (
        "robotic-gripper",
        include_str!("icons/bundled/robotic-gripper.svg"),
    ),
    ("rocket", include_str!("icons/bundled/rocket.svg")),
    (
        "rocket-circle",
        include_str!("icons/bundled/rocket-circle.svg"),
    ),
    (
        "rocket-circle-filled",
        include_str!("icons/bundled/rocket-circle-filled.svg"),
    ),
    (
        "rocket-filled",
        include_str!("icons/bundled/rocket-filled.svg"),
    ),
    ("roles", include_str!("icons/bundled/roles.svg")),
    (
        "roles-filled",
        include_str!("icons/bundled/roles-filled.svg"),
    ),
    ("rotate", include_str!("icons/bundled/rotate.svg")),
    ("rotate-180", include_str!("icons/bundled/rotate-180.svg")),
    (
        "rotate-90-left",
        include_str!("icons/bundled/rotate-90-left.svg"),
    ),
    (
        "rotate-90-right",
        include_str!("icons/bundled/rotate-90-right.svg"),
    ),
    ("route", include_str!("icons/bundled/route.svg")),
    (
        "route-target",
        include_str!("icons/bundled/route-target.svg"),
    ),
    (
        "ruler-diagonal",
        include_str!("icons/bundled/ruler-diagonal.svg"),
    ),
    (
        "ruler-horizontal",
        include_str!("icons/bundled/ruler-horizontal.svg"),
    ),
    (
        "ruler-vertical",
        include_str!("icons/bundled/ruler-vertical.svg"),
    ),
    ("rules", include_str!("icons/bundled/rules.svg")),
    (
        "rules-filled",
        include_str!("icons/bundled/rules-filled.svg"),
    ),
    (
        "runtime-play",
        include_str!("icons/bundled/runtime-play.svg"),
    ),
    (
        "runtime-settings",
        include_str!("icons/bundled/runtime-settings.svg"),
    ),
    (
        "runtime-stop",
        include_str!("icons/bundled/runtime-stop.svg"),
    ),
    (
        "safety-settings",
        include_str!("icons/bundled/safety-settings.svg"),
    ),
    ("sankeychart", include_str!("icons/bundled/sankeychart.svg")),
    ("save-all", include_str!("icons/bundled/save-all.svg")),
    ("savings", include_str!("icons/bundled/savings.svg")),
    ("scale", include_str!("icons/bundled/scale.svg")),
    ("scatterplot", include_str!("icons/bundled/scatterplot.svg")),
    ("scheduler", include_str!("icons/bundled/scheduler.svg")),
    (
        "scheduler-filled",
        include_str!("icons/bundled/scheduler-filled.svg"),
    ),
    ("screen", include_str!("icons/bundled/screen.svg")),
    (
        "screen-duplicate",
        include_str!("icons/bundled/screen-duplicate.svg"),
    ),
    (
        "screen-duplicate-filled",
        include_str!("icons/bundled/screen-duplicate-filled.svg"),
    ),
    (
        "screen-filled",
        include_str!("icons/bundled/screen-filled.svg"),
    ),
    (
        "screen-pc-tower",
        include_str!("icons/bundled/screen-pc-tower.svg"),
    ),
    (
        "screen-pc-tower-filled",
        include_str!("icons/bundled/screen-pc-tower-filled.svg"),
    ),
    (
        "screen-pc-tower-settings",
        include_str!("icons/bundled/screen-pc-tower-settings.svg"),
    ),
    (
        "screen-pc-tower-settings-filled",
        include_str!("icons/bundled/screen-pc-tower-settings-filled.svg"),
    ),
    (
        "screen-settings",
        include_str!("icons/bundled/screen-settings.svg"),
    ),
    (
        "screen-settings-filled",
        include_str!("icons/bundled/screen-settings-filled.svg"),
    ),
    ("screens", include_str!("icons/bundled/screens.svg")),
    (
        "screens-filled",
        include_str!("icons/bundled/screens-filled.svg"),
    ),
    ("screenshot", include_str!("icons/bundled/screenshot.svg")),
    (
        "screenshot-filled",
        include_str!("icons/bundled/screenshot-filled.svg"),
    ),
    ("script", include_str!("icons/bundled/script.svg")),
    ("script-add", include_str!("icons/bundled/script-add.svg")),
    ("scripts", include_str!("icons/bundled/scripts.svg")),
    ("sd-card", include_str!("icons/bundled/sd-card.svg")),
    (
        "sd-card-filled",
        include_str!("icons/bundled/sd-card-filled.svg"),
    ),
    (
        "sd-card-micro",
        include_str!("icons/bundled/sd-card-micro.svg"),
    ),
    (
        "sd-card-micro-filled",
        include_str!("icons/bundled/sd-card-micro-filled.svg"),
    ),
    ("search", include_str!("icons/bundled/search.svg")),
    ("select-alt", include_str!("icons/bundled/select-alt.svg")),
    (
        "select-alt-filled",
        include_str!("icons/bundled/select-alt-filled.svg"),
    ),
    (
        "send-backward",
        include_str!("icons/bundled/send-backward.svg"),
    ),
    ("send-right", include_str!("icons/bundled/send-right.svg")),
    (
        "send-right-filled",
        include_str!("icons/bundled/send-right-filled.svg"),
    ),
    (
        "send-to-back",
        include_str!("icons/bundled/send-to-back.svg"),
    ),
    (
        "send-top-right",
        include_str!("icons/bundled/send-top-right.svg"),
    ),
    (
        "send-top-right-filled",
        include_str!("icons/bundled/send-top-right-filled.svg"),
    ),
    ("sensor", include_str!("icons/bundled/sensor.svg")),
    (
        "separator-line",
        include_str!("icons/bundled/separator-line.svg"),
    ),
    (
        "server-interface",
        include_str!("icons/bundled/server-interface.svg"),
    ),
    ("server-rack", include_str!("icons/bundled/server-rack.svg")),
    (
        "server-rack-filled",
        include_str!("icons/bundled/server-rack-filled.svg"),
    ),
    ("shapes", include_str!("icons/bundled/shapes.svg")),
    (
        "shapes-filled",
        include_str!("icons/bundled/shapes-filled.svg"),
    ),
    ("share", include_str!("icons/bundled/share.svg")),
    ("share-alt", include_str!("icons/bundled/share-alt.svg")),
    (
        "share-alt-filled",
        include_str!("icons/bundled/share-alt-filled.svg"),
    ),
    (
        "share-filled",
        include_str!("icons/bundled/share-filled.svg"),
    ),
    ("shield", include_str!("icons/bundled/shield.svg")),
    ("shield-ai", include_str!("icons/bundled/shield-ai.svg")),
    (
        "shield-ai-filled",
        include_str!("icons/bundled/shield-ai-filled.svg"),
    ),
    (
        "shield-broken",
        include_str!("icons/bundled/shield-broken.svg"),
    ),
    (
        "shield-broken-filled",
        include_str!("icons/bundled/shield-broken-filled.svg"),
    ),
    (
        "shield-check",
        include_str!("icons/bundled/shield-check.svg"),
    ),
    (
        "shield-check-filled",
        include_str!("icons/bundled/shield-check-filled.svg"),
    ),
    (
        "shield-filled",
        include_str!("icons/bundled/shield-filled.svg"),
    ),
    ("shield-half", include_str!("icons/bundled/shield-half.svg")),
    ("shift", include_str!("icons/bundled/shift.svg")),
    (
        "shift-filled",
        include_str!("icons/bundled/shift-filled.svg"),
    ),
    (
        "shopping-cart",
        include_str!("icons/bundled/shopping-cart.svg"),
    ),
    (
        "shopping-cart-filled",
        include_str!("icons/bundled/shopping-cart-filled.svg"),
    ),
    ("shorter", include_str!("icons/bundled/shorter.svg")),
    ("shorter-ai", include_str!("icons/bundled/shorter-ai.svg")),
    ("shout", include_str!("icons/bundled/shout.svg")),
    (
        "shout-filled",
        include_str!("icons/bundled/shout-filled.svg"),
    ),
    ("shutdown", include_str!("icons/bundled/shutdown.svg")),
    (
        "sign-language",
        include_str!("icons/bundled/sign-language.svg"),
    ),
    (
        "signal-strength-0",
        include_str!("icons/bundled/signal-strength-0.svg"),
    ),
    (
        "signal-strength-1",
        include_str!("icons/bundled/signal-strength-1.svg"),
    ),
    (
        "signal-strength-2",
        include_str!("icons/bundled/signal-strength-2.svg"),
    ),
    (
        "signal-strength-3",
        include_str!("icons/bundled/signal-strength-3.svg"),
    ),
    (
        "signal-strength-4",
        include_str!("icons/bundled/signal-strength-4.svg"),
    ),
    (
        "signal-strength-5",
        include_str!("icons/bundled/signal-strength-5.svg"),
    ),
    (
        "signal-strength-6",
        include_str!("icons/bundled/signal-strength-6.svg"),
    ),
    (
        "signal-strength-7",
        include_str!("icons/bundled/signal-strength-7.svg"),
    ),
    (
        "signal-strength-8",
        include_str!("icons/bundled/signal-strength-8.svg"),
    ),
    (
        "simit-component",
        include_str!("icons/bundled/simit-component.svg"),
    ),
    ("simit-macro", include_str!("icons/bundled/simit-macro.svg")),
    (
        "simit-macro-component-editor",
        include_str!("icons/bundled/simit-macro-component-editor.svg"),
    ),
    (
        "simulation-table",
        include_str!("icons/bundled/simulation-table.svg"),
    ),
    (
        "single-check",
        include_str!("icons/bundled/single-check.svg"),
    ),
    ("skip", include_str!("icons/bundled/skip.svg")),
    ("skip-back", include_str!("icons/bundled/skip-back.svg")),
    (
        "skip-back-filled",
        include_str!("icons/bundled/skip-back-filled.svg"),
    ),
    ("skip-filled", include_str!("icons/bundled/skip-filled.svg")),
    ("sms", include_str!("icons/bundled/sms.svg")),
    ("snowflake", include_str!("icons/bundled/snowflake.svg")),
    (
        "solid-state-drive",
        include_str!("icons/bundled/solid-state-drive.svg"),
    ),
    ("sort", include_str!("icons/bundled/sort.svg")),
    ("sort-alt", include_str!("icons/bundled/sort-alt.svg")),
    (
        "sort-ascending",
        include_str!("icons/bundled/sort-ascending.svg"),
    ),
    (
        "sort-descending",
        include_str!("icons/bundled/sort-descending.svg"),
    ),
    ("sound-loud", include_str!("icons/bundled/sound-loud.svg")),
    (
        "sound-loud-filled",
        include_str!("icons/bundled/sound-loud-filled.svg"),
    ),
    ("sound-mute", include_str!("icons/bundled/sound-mute.svg")),
    (
        "sound-mute-filled",
        include_str!("icons/bundled/sound-mute-filled.svg"),
    ),
    ("sound-off", include_str!("icons/bundled/sound-off.svg")),
    (
        "sound-off-filled",
        include_str!("icons/bundled/sound-off-filled.svg"),
    ),
    ("sound-quiet", include_str!("icons/bundled/sound-quiet.svg")),
    (
        "sound-quiet-filled",
        include_str!("icons/bundled/sound-quiet-filled.svg"),
    ),
    ("spatial", include_str!("icons/bundled/spatial.svg")),
    ("spiderchart", include_str!("icons/bundled/spiderchart.svg")),
    (
        "spiderchart-filled",
        include_str!("icons/bundled/spiderchart-filled.svg"),
    ),
    ("split", include_str!("icons/bundled/split.svg")),
    (
        "split-horizontally",
        include_str!("icons/bundled/split-horizontally.svg"),
    ),
    (
        "split-vertically",
        include_str!("icons/bundled/split-vertically.svg"),
    ),
    ("square", include_str!("icons/bundled/square.svg")),
    (
        "square-filled",
        include_str!("icons/bundled/square-filled.svg"),
    ),
    (
        "stacked-barchart",
        include_str!("icons/bundled/stacked-barchart.svg"),
    ),
    ("stamp", include_str!("icons/bundled/stamp.svg")),
    (
        "stamp-filled",
        include_str!("icons/bundled/stamp-filled.svg"),
    ),
    ("standby", include_str!("icons/bundled/standby.svg")),
    ("star", include_str!("icons/bundled/star.svg")),
    ("star-add", include_str!("icons/bundled/star-add.svg")),
    (
        "star-add-filled",
        include_str!("icons/bundled/star-add-filled.svg"),
    ),
    (
        "star-cancelled",
        include_str!("icons/bundled/star-cancelled.svg"),
    ),
    (
        "star-cancelled-filled",
        include_str!("icons/bundled/star-cancelled-filled.svg"),
    ),
    ("star-filled", include_str!("icons/bundled/star-filled.svg")),
    (
        "star-half-filled",
        include_str!("icons/bundled/star-half-filled.svg"),
    ),
    ("star-list", include_str!("icons/bundled/star-list.svg")),
    (
        "star-list-filled",
        include_str!("icons/bundled/star-list-filled.svg"),
    ),
    (
        "start-data-analysis",
        include_str!("icons/bundled/start-data-analysis.svg"),
    ),
    ("steering", include_str!("icons/bundled/steering.svg")),
    (
        "steering-user",
        include_str!("icons/bundled/steering-user.svg"),
    ),
    (
        "steering-user-filled",
        include_str!("icons/bundled/steering-user-filled.svg"),
    ),
    ("stethoscope", include_str!("icons/bundled/stethoscope.svg")),
    ("stop", include_str!("icons/bundled/stop.svg")),
    ("stop-filled", include_str!("icons/bundled/stop-filled.svg")),
    ("stopwatch", include_str!("icons/bundled/stopwatch.svg")),
    (
        "stopwatch-filled",
        include_str!("icons/bundled/stopwatch-filled.svg"),
    ),
    ("storage", include_str!("icons/bundled/storage.svg")),
    (
        "storage-filled",
        include_str!("icons/bundled/storage-filled.svg"),
    ),
    ("subtitle", include_str!("icons/bundled/subtitle.svg")),
    (
        "subtitle-filled",
        include_str!("icons/bundled/subtitle-filled.svg"),
    ),
    ("success", include_str!("icons/bundled/success.svg")),
    (
        "success-filled",
        include_str!("icons/bundled/success-filled.svg"),
    ),
    (
        "success-multiple",
        include_str!("icons/bundled/success-multiple.svg"),
    ),
    (
        "success-multiple-filled",
        include_str!("icons/bundled/success-multiple-filled.svg"),
    ),
    ("sun", include_str!("icons/bundled/sun.svg")),
    ("sun-cloud", include_str!("icons/bundled/sun-cloud.svg")),
    (
        "sun-cloud-filled",
        include_str!("icons/bundled/sun-cloud-filled.svg"),
    ),
    ("sun-filled", include_str!("icons/bundled/sun-filled.svg")),
    ("sunrise", include_str!("icons/bundled/sunrise.svg")),
    (
        "sunrise-filled",
        include_str!("icons/bundled/sunrise-filled.svg"),
    ),
    ("sunset", include_str!("icons/bundled/sunset.svg")),
    (
        "sunset-filled",
        include_str!("icons/bundled/sunset-filled.svg"),
    ),
    ("support", include_str!("icons/bundled/support.svg")),
    ("support-ai", include_str!("icons/bundled/support-ai.svg")),
    (
        "surveillance",
        include_str!("icons/bundled/surveillance.svg"),
    ),
    (
        "surveillance-cancelled",
        include_str!("icons/bundled/surveillance-cancelled.svg"),
    ),
    (
        "surveillance-cancelled-filled",
        include_str!("icons/bundled/surveillance-cancelled-filled.svg"),
    ),
    (
        "surveillance-filled",
        include_str!("icons/bundled/surveillance-filled.svg"),
    ),
    (
        "svg-document",
        include_str!("icons/bundled/svg-document.svg"),
    ),
    (
        "swap-left-right",
        include_str!("icons/bundled/swap-left-right.svg"),
    ),
    (
        "switch-slider",
        include_str!("icons/bundled/switch-slider.svg"),
    ),
    ("sword-swing", include_str!("icons/bundled/sword-swing.svg")),
    ("table", include_str!("icons/bundled/table.svg")),
    (
        "table-add-column-right",
        include_str!("icons/bundled/table-add-column-right.svg"),
    ),
    (
        "table-add-row-below",
        include_str!("icons/bundled/table-add-row-below.svg"),
    ),
    ("table-ai", include_str!("icons/bundled/table-ai.svg")),
    (
        "table-columns",
        include_str!("icons/bundled/table-columns.svg"),
    ),
    (
        "table-insert-column-left",
        include_str!("icons/bundled/table-insert-column-left.svg"),
    ),
    (
        "table-insert-column-right",
        include_str!("icons/bundled/table-insert-column-right.svg"),
    ),
    (
        "table-insert-row-above",
        include_str!("icons/bundled/table-insert-row-above.svg"),
    ),
    (
        "table-insert-row-below",
        include_str!("icons/bundled/table-insert-row-below.svg"),
    ),
    ("table-rows", include_str!("icons/bundled/table-rows.svg")),
    (
        "table-settings",
        include_str!("icons/bundled/table-settings.svg"),
    ),
    ("table-tag", include_str!("icons/bundled/table-tag.svg")),
    ("tag", include_str!("icons/bundled/tag.svg")),
    (
        "tag-arrow-left",
        include_str!("icons/bundled/tag-arrow-left.svg"),
    ),
    (
        "tag-arrow-left-filled",
        include_str!("icons/bundled/tag-arrow-left-filled.svg"),
    ),
    (
        "tag-arrow-right",
        include_str!("icons/bundled/tag-arrow-right.svg"),
    ),
    (
        "tag-arrow-right-filled",
        include_str!("icons/bundled/tag-arrow-right-filled.svg"),
    ),
    (
        "tag-circle-arrow-down",
        include_str!("icons/bundled/tag-circle-arrow-down.svg"),
    ),
    (
        "tag-circle-arrow-down-filled",
        include_str!("icons/bundled/tag-circle-arrow-down-filled.svg"),
    ),
    (
        "tag-connection",
        include_str!("icons/bundled/tag-connection.svg"),
    ),
    (
        "tag-connection-filled",
        include_str!("icons/bundled/tag-connection-filled.svg"),
    ),
    (
        "tag-connection-view",
        include_str!("icons/bundled/tag-connection-view.svg"),
    ),
    ("tag-eye", include_str!("icons/bundled/tag-eye.svg")),
    (
        "tag-eye-filled",
        include_str!("icons/bundled/tag-eye-filled.svg"),
    ),
    ("tag-filled", include_str!("icons/bundled/tag-filled.svg")),
    ("tag-logging", include_str!("icons/bundled/tag-logging.svg")),
    (
        "tag-logging-filled",
        include_str!("icons/bundled/tag-logging-filled.svg"),
    ),
    ("tag-plus", include_str!("icons/bundled/tag-plus.svg")),
    (
        "tag-plus-filled",
        include_str!("icons/bundled/tag-plus-filled.svg"),
    ),
    ("tag-safety", include_str!("icons/bundled/tag-safety.svg")),
    (
        "tag-safety-filled",
        include_str!("icons/bundled/tag-safety-filled.svg"),
    ),
    ("tasks-all", include_str!("icons/bundled/tasks-all.svg")),
    ("tasks-done", include_str!("icons/bundled/tasks-done.svg")),
    ("tasks-open", include_str!("icons/bundled/tasks-open.svg")),
    ("text", include_str!("icons/bundled/text.svg")),
    (
        "text-alginment-center",
        include_str!("icons/bundled/text-alginment-center.svg"),
    ),
    (
        "text-alginment-justified",
        include_str!("icons/bundled/text-alginment-justified.svg"),
    ),
    (
        "text-alginment-left",
        include_str!("icons/bundled/text-alginment-left.svg"),
    ),
    (
        "text-alginment-right",
        include_str!("icons/bundled/text-alginment-right.svg"),
    ),
    ("text-bold", include_str!("icons/bundled/text-bold.svg")),
    (
        "text-circle-rectangle",
        include_str!("icons/bundled/text-circle-rectangle.svg"),
    ),
    (
        "text-circle-rectangle-filled",
        include_str!("icons/bundled/text-circle-rectangle-filled.svg"),
    ),
    (
        "text-document",
        include_str!("icons/bundled/text-document.svg"),
    ),
    ("text-italic", include_str!("icons/bundled/text-italic.svg")),
    (
        "text-strike-through",
        include_str!("icons/bundled/text-strike-through.svg"),
    ),
    (
        "text-underline",
        include_str!("icons/bundled/text-underline.svg"),
    ),
    ("theme", include_str!("icons/bundled/theme.svg")),
    (
        "theme-filled",
        include_str!("icons/bundled/theme-filled.svg"),
    ),
    ("thermometer", include_str!("icons/bundled/thermometer.svg")),
    (
        "thermometer-filled",
        include_str!("icons/bundled/thermometer-filled.svg"),
    ),
    (
        "threshold-cancelled",
        include_str!("icons/bundled/threshold-cancelled.svg"),
    ),
    (
        "threshold-off",
        include_str!("icons/bundled/threshold-off.svg"),
    ),
    (
        "threshold-on",
        include_str!("icons/bundled/threshold-on.svg"),
    ),
    ("thumb-down", include_str!("icons/bundled/thumb-down.svg")),
    (
        "thumb-down-filled",
        include_str!("icons/bundled/thumb-down-filled.svg"),
    ),
    ("thumb-up", include_str!("icons/bundled/thumb-up.svg")),
    (
        "thumb-up-filled",
        include_str!("icons/bundled/thumb-up-filled.svg"),
    ),
    ("ticket", include_str!("icons/bundled/ticket.svg")),
    (
        "ticket-filled",
        include_str!("icons/bundled/ticket-filled.svg"),
    ),
    ("tiles", include_str!("icons/bundled/tiles.svg")),
    (
        "tiles-filled",
        include_str!("icons/bundled/tiles-filled.svg"),
    ),
    ("time-zone", include_str!("icons/bundled/time-zone.svg")),
    (
        "time-zone-filled",
        include_str!("icons/bundled/time-zone-filled.svg"),
    ),
    (
        "to-be-published",
        include_str!("icons/bundled/to-be-published.svg"),
    ),
    ("to-search", include_str!("icons/bundled/to-search.svg")),
    ("topic", include_str!("icons/bundled/topic.svg")),
    (
        "topic-filled",
        include_str!("icons/bundled/topic-filled.svg"),
    ),
    ("touch", include_str!("icons/bundled/touch.svg")),
    (
        "touch-filled",
        include_str!("icons/bundled/touch-filled.svg"),
    ),
    ("trace-eye", include_str!("icons/bundled/trace-eye.svg")),
    (
        "traffic-left-right",
        include_str!("icons/bundled/traffic-left-right.svg"),
    ),
    ("train", include_str!("icons/bundled/train.svg")),
    (
        "train-filled",
        include_str!("icons/bundled/train-filled.svg"),
    ),
    ("trashcan", include_str!("icons/bundled/trashcan.svg")),
    (
        "trashcan-filled",
        include_str!("icons/bundled/trashcan-filled.svg"),
    ),
    ("tree", include_str!("icons/bundled/tree.svg")),
    (
        "tree-two-level",
        include_str!("icons/bundled/tree-two-level.svg"),
    ),
    ("trend", include_str!("icons/bundled/trend.svg")),
    (
        "trend-companion",
        include_str!("icons/bundled/trend-companion.svg"),
    ),
    (
        "trend-downward",
        include_str!("icons/bundled/trend-downward.svg"),
    ),
    (
        "trend-downward-circle",
        include_str!("icons/bundled/trend-downward-circle.svg"),
    ),
    (
        "trend-downward-filled",
        include_str!("icons/bundled/trend-downward-filled.svg"),
    ),
    (
        "trend-flat-curve",
        include_str!("icons/bundled/trend-flat-curve.svg"),
    ),
    (
        "trend-sideways",
        include_str!("icons/bundled/trend-sideways.svg"),
    ),
    (
        "trend-sideways-circle",
        include_str!("icons/bundled/trend-sideways-circle.svg"),
    ),
    (
        "trend-sideways-filled",
        include_str!("icons/bundled/trend-sideways-filled.svg"),
    ),
    (
        "trend-upward",
        include_str!("icons/bundled/trend-upward.svg"),
    ),
    (
        "trend-upward-circle",
        include_str!("icons/bundled/trend-upward-circle.svg"),
    ),
    (
        "trend-upward-filled",
        include_str!("icons/bundled/trend-upward-filled.svg"),
    ),
    ("triangle", include_str!("icons/bundled/triangle.svg")),
    (
        "triangle-filled",
        include_str!("icons/bundled/triangle-filled.svg"),
    ),
    ("trophy", include_str!("icons/bundled/trophy.svg")),
    (
        "trophy-filled",
        include_str!("icons/bundled/trophy-filled.svg"),
    ),
    ("truck", include_str!("icons/bundled/truck.svg")),
    (
        "truck-filled",
        include_str!("icons/bundled/truck-filled.svg"),
    ),
    ("tulip", include_str!("icons/bundled/tulip.svg")),
    (
        "tulip-filled",
        include_str!("icons/bundled/tulip-filled.svg"),
    ),
    (
        "txt-document",
        include_str!("icons/bundled/txt-document.svg"),
    ),
    (
        "txt-document-filled",
        include_str!("icons/bundled/txt-document-filled.svg"),
    ),
    ("undo", include_str!("icons/bundled/undo.svg")),
    ("ungroup", include_str!("icons/bundled/ungroup.svg")),
    (
        "ungroup-objects",
        include_str!("icons/bundled/ungroup-objects.svg"),
    ),
    ("unlock", include_str!("icons/bundled/unlock.svg")),
    (
        "unlock-filled",
        include_str!("icons/bundled/unlock-filled.svg"),
    ),
    (
        "unlock-plant",
        include_str!("icons/bundled/unlock-plant.svg"),
    ),
    (
        "unlock-plant-filled",
        include_str!("icons/bundled/unlock-plant-filled.svg"),
    ),
    (
        "update-application",
        include_str!("icons/bundled/update-application.svg"),
    ),
    ("upload", include_str!("icons/bundled/upload.svg")),
    (
        "upload-document-note",
        include_str!("icons/bundled/upload-document-note.svg"),
    ),
    ("upload-fail", include_str!("icons/bundled/upload-fail.svg")),
    (
        "upload-success",
        include_str!("icons/bundled/upload-success.svg"),
    ),
    ("upper-limit", include_str!("icons/bundled/upper-limit.svg")),
    ("usb-drive", include_str!("icons/bundled/usb-drive.svg")),
    ("user", include_str!("icons/bundled/user.svg")),
    ("user-check", include_str!("icons/bundled/user-check.svg")),
    (
        "user-check-filled",
        include_str!("icons/bundled/user-check-filled.svg"),
    ),
    (
        "user-data-types",
        include_str!("icons/bundled/user-data-types.svg"),
    ),
    ("user-fail", include_str!("icons/bundled/user-fail.svg")),
    (
        "user-fail-filled",
        include_str!("icons/bundled/user-fail-filled.svg"),
    ),
    ("user-filled", include_str!("icons/bundled/user-filled.svg")),
    ("user-group", include_str!("icons/bundled/user-group.svg")),
    ("user-key", include_str!("icons/bundled/user-key.svg")),
    ("user-lock", include_str!("icons/bundled/user-lock.svg")),
    (
        "user-lock-filled",
        include_str!("icons/bundled/user-lock-filled.svg"),
    ),
    (
        "user-management",
        include_str!("icons/bundled/user-management.svg"),
    ),
    (
        "user-management-filled",
        include_str!("icons/bundled/user-management-filled.svg"),
    ),
    (
        "user-management-settings",
        include_str!("icons/bundled/user-management-settings.svg"),
    ),
    (
        "user-management-settings-filled",
        include_str!("icons/bundled/user-management-settings-filled.svg"),
    ),
    ("user-manual", include_str!("icons/bundled/user-manual.svg")),
    (
        "user-manual-filled",
        include_str!("icons/bundled/user-manual-filled.svg"),
    ),
    ("user-pen", include_str!("icons/bundled/user-pen.svg")),
    (
        "user-profile",
        include_str!("icons/bundled/user-profile.svg"),
    ),
    (
        "user-profile-filled",
        include_str!("icons/bundled/user-profile-filled.svg"),
    ),
    (
        "user-reading",
        include_str!("icons/bundled/user-reading.svg"),
    ),
    (
        "user-reading-filled",
        include_str!("icons/bundled/user-reading-filled.svg"),
    ),
    (
        "user-settings",
        include_str!("icons/bundled/user-settings.svg"),
    ),
    (
        "user-settings-filled",
        include_str!("icons/bundled/user-settings-filled.svg"),
    ),
    (
        "user-success",
        include_str!("icons/bundled/user-success.svg"),
    ),
    (
        "user-success-filled",
        include_str!("icons/bundled/user-success-filled.svg"),
    ),
    ("variable", include_str!("icons/bundled/variable.svg")),
    ("vdi-folder", include_str!("icons/bundled/vdi-folder.svg")),
    (
        "version-history",
        include_str!("icons/bundled/version-history.svg"),
    ),
    (
        "video-camera",
        include_str!("icons/bundled/video-camera.svg"),
    ),
    (
        "video-camera-filled",
        include_str!("icons/bundled/video-camera-filled.svg"),
    ),
    (
        "video-camera-record",
        include_str!("icons/bundled/video-camera-record.svg"),
    ),
    (
        "video-camera-record-filled",
        include_str!("icons/bundled/video-camera-record-filled.svg"),
    ),
    ("video-file", include_str!("icons/bundled/video-file.svg")),
    (
        "video-file-filled",
        include_str!("icons/bundled/video-file-filled.svg"),
    ),
    ("voltage", include_str!("icons/bundled/voltage.svg")),
    (
        "voltage-filled",
        include_str!("icons/bundled/voltage-filled.svg"),
    ),
    ("warning", include_str!("icons/bundled/warning.svg")),
    (
        "warning-filled",
        include_str!("icons/bundled/warning-filled.svg"),
    ),
    (
        "warning-hexagon",
        include_str!("icons/bundled/warning-hexagon.svg"),
    ),
    (
        "warning-hexagon-filled",
        include_str!("icons/bundled/warning-hexagon-filled.svg"),
    ),
    (
        "warning-multiple",
        include_str!("icons/bundled/warning-multiple.svg"),
    ),
    (
        "warning-multiple-filled",
        include_str!("icons/bundled/warning-multiple-filled.svg"),
    ),
    (
        "warning-octagon",
        include_str!("icons/bundled/warning-octagon.svg"),
    ),
    (
        "warning-octagon-filled",
        include_str!("icons/bundled/warning-octagon-filled.svg"),
    ),
    (
        "warning-octagon-multiple",
        include_str!("icons/bundled/warning-octagon-multiple.svg"),
    ),
    (
        "warning-octagon-multiple-filled",
        include_str!("icons/bundled/warning-octagon-multiple-filled.svg"),
    ),
    (
        "warning-rhomb",
        include_str!("icons/bundled/warning-rhomb.svg"),
    ),
    (
        "warning-rhomb-filled",
        include_str!("icons/bundled/warning-rhomb-filled.svg"),
    ),
    (
        "warning-rhomb-multiple",
        include_str!("icons/bundled/warning-rhomb-multiple.svg"),
    ),
    (
        "warning-rhomb-multiple-filled",
        include_str!("icons/bundled/warning-rhomb-multiple-filled.svg"),
    ),
    (
        "warning-square",
        include_str!("icons/bundled/warning-square.svg"),
    ),
    (
        "warning-square-filled",
        include_str!("icons/bundled/warning-square-filled.svg"),
    ),
    ("watch-table", include_str!("icons/bundled/watch-table.svg")),
    (
        "water-bathing",
        include_str!("icons/bundled/water-bathing.svg"),
    ),
    ("water-fish", include_str!("icons/bundled/water-fish.svg")),
    ("water-plant", include_str!("icons/bundled/water-plant.svg")),
    (
        "water-sunbathing",
        include_str!("icons/bundled/water-sunbathing.svg"),
    ),
    ("waveform", include_str!("icons/bundled/waveform.svg")),
    (
        "web-browser-screen",
        include_str!("icons/bundled/web-browser-screen.svg"),
    ),
    ("webcam", include_str!("icons/bundled/webcam.svg")),
    (
        "webcam-cancelled",
        include_str!("icons/bundled/webcam-cancelled.svg"),
    ),
    (
        "webcam-cancelled-filled",
        include_str!("icons/bundled/webcam-cancelled-filled.svg"),
    ),
    (
        "webcam-filled",
        include_str!("icons/bundled/webcam-filled.svg"),
    ),
    ("width", include_str!("icons/bundled/width.svg")),
    ("wlan-off", include_str!("icons/bundled/wlan-off.svg")),
    (
        "wlan-strength-0",
        include_str!("icons/bundled/wlan-strength-0.svg"),
    ),
    (
        "wlan-strength-1",
        include_str!("icons/bundled/wlan-strength-1.svg"),
    ),
    (
        "wlan-strength-1-lock",
        include_str!("icons/bundled/wlan-strength-1-lock.svg"),
    ),
    (
        "wlan-strength-2",
        include_str!("icons/bundled/wlan-strength-2.svg"),
    ),
    (
        "wlan-strength-2-lock",
        include_str!("icons/bundled/wlan-strength-2-lock.svg"),
    ),
    (
        "wlan-strength-3",
        include_str!("icons/bundled/wlan-strength-3.svg"),
    ),
    (
        "wlan-strength-3-lock",
        include_str!("icons/bundled/wlan-strength-3-lock.svg"),
    ),
    (
        "wlan-warning",
        include_str!("icons/bundled/wlan-warning.svg"),
    ),
    ("work-case", include_str!("icons/bundled/work-case.svg")),
    (
        "work-case-filled",
        include_str!("icons/bundled/work-case-filled.svg"),
    ),
    ("workspace", include_str!("icons/bundled/workspace.svg")),
    ("workspaces", include_str!("icons/bundled/workspaces.svg")),
    (
        "x-axis-settings",
        include_str!("icons/bundled/x-axis-settings.svg"),
    ),
    (
        "xls-document",
        include_str!("icons/bundled/xls-document.svg"),
    ),
    (
        "xls-document-filled",
        include_str!("icons/bundled/xls-document-filled.svg"),
    ),
    (
        "xml-document",
        include_str!("icons/bundled/xml-document.svg"),
    ),
    (
        "y-axis-settings",
        include_str!("icons/bundled/y-axis-settings.svg"),
    ),
    ("zone", include_str!("icons/bundled/zone.svg")),
    ("zoom-in", include_str!("icons/bundled/zoom-in.svg")),
    ("zoom-out", include_str!("icons/bundled/zoom-out.svg")),
    (
        "zoom-selection",
        include_str!("icons/bundled/zoom-selection.svg"),
    ),
];
