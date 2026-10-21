//! Rust implementations of Industrial Experience components for Dioxus.
//!
//! Components and interactions are authored in Rust without the upstream
//! JavaScript or Web Component package. Dioxus generates its platform runtime.
//! Render [`IxTheme`] once around your application to install the embedded styles.
//! This project is independent of Siemens and does not ship Siemens logos.
//!
//! ```rust,no_run
//! use dioxus::prelude::*;
//! use dioxus_ix::*;
//!
//! fn app() -> Element {
//!     rsx! {
//!         IxTheme {
//!             Button { "Start" }
//!         }
//!     }
//! }
//! ```

#![forbid(unsafe_code)]

// Dioxus's HTML event converter is process-global. Actual-listener regressions
// hold this guard once per test, before their DOMs, so parallel test modules
// cannot replace the converter while another test dispatches an event.
#[cfg(test)]
static TEST_EVENT_CONVERTER_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
pub(crate) fn test_event_converter_guard() -> std::sync::MutexGuard<'static, ()> {
    TEST_EVENT_CONVERTER_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub mod actions;
pub mod application;
mod application_viewport;
mod avatar;
mod breadcrumb;
mod button_styles;
mod card_list;
mod card_list_model;
pub mod catalog;
pub mod chart_advanced;
pub mod chart_export;
pub mod chart_toolbox;
pub mod charts;
pub mod chat;
mod checkbox_styles;
pub mod composition;
pub mod data_grid;
mod data_grid_columns;
mod data_grid_custom_editor;
mod data_grid_editor;
mod data_grid_render;
mod data_grid_value;
mod date_dropdown_controller;
pub mod datetime;
pub mod dialogs;
mod disclosure;
mod document_viewport;
mod dom_query;
mod dropdown;
mod dropdown_inventory;
mod event_list;
mod expanding_search_styles;
mod flip_tile;
mod focus_containment;
pub mod form_control;
pub mod forms;
mod group_styles;
mod groups;
mod header_control_scope;
mod header_layout;
mod icon_button_styles;
pub mod icons;
mod information_states;
mod input_geometry;
mod input_selection;
mod input_styles;
mod key_value;
pub mod layout;
mod localized_date;
mod menu_avatar;
pub mod message_dialog;
mod native_surface;
pub mod notifications;
mod number_input_styles;
mod outside_click;
mod overlay_completion;
mod overlay_domain;
mod overlay_registry;
mod overlay_registry_dom;
mod overlay_runtime;
pub mod overlays;
mod page_layout;
mod pagination;
pub mod patterns;
mod popover_focus;
mod progress_styles;
mod radio_styles;
mod select_item;
mod select_visual_styles;
mod slider_styles;
mod spinner_styles;
mod split_button;
mod task_cards;
pub mod theme;
mod toggle_button_styles;
mod toggle_styles;
pub mod tooltip;
pub mod tree;
mod upload_styles;

pub use actions::*;
pub use application::*;
pub use catalog::{PORT_CATALOG, PortEntry, PortMappingKind, UPSTREAM_EXAMPLE_NAMES};
pub use chart_advanced::*;
pub use chart_export::*;
pub use chart_toolbox::*;
pub use charts::*;
pub use chat::*;
pub use composition::*;
pub use data_grid::*;
pub use data_grid_custom_editor::*;
pub use data_grid_editor::*;
pub use data_grid_render::*;
pub use data_grid_value::*;
pub use datetime::*;
pub use dialogs::*;
pub use form_control::*;
pub use forms::*;
pub use icons::{
    Icon, IconDefinition, IconProps, IconRegistry, IconRegistryProvider, IconRegistryProviderProps,
    icon_names, use_icon_registry,
};
pub use layout::*;
pub use message_dialog::*;
pub use notifications::*;
pub use overlays::*;
pub use patterns::*;
pub use select_item::*;
pub use theme::{ColorScheme, Density, IxTheme, IxThemeProps};
pub use tooltip::*;
pub use tree::*;

/// Upstream source snapshot used when the port was started.
pub const UPSTREAM_REVISION: &str = "8bf599bc68365d7752e12832e71b625883b93d7a";
