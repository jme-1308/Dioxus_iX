//! Review-only menu overlay controls; upstream source examples remain unchanged.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn MenuOverlayReview() -> Element {
    let mut active = use_signal(|| None::<MenuOverlayKind>);
    let mut veto_settings = use_signal(|| true);
    let mut veto_about = use_signal(|| false);
    let mut settings_attempts = use_signal(|| 0_u32);
    let mut about_attempts = use_signal(|| 0_u32);
    let mut settings_accepted = use_signal(|| 0_u32);
    let mut about_accepted = use_signal(|| 0_u32);
    let mut closes = use_signal(|| 0_u32);
    let mut about_tab = use_signal(|| "overview".to_string());
    rsx! {
        style { ".menu-overlay-review-frame{{position:relative;height:24rem;min-width:0;border:1px solid var(--ix-color-border)}}.menu-overlay-review-frame>.ix-application{{height:100%}}.menu-overlay-review-frame .ix-menu-overlay-content{{max-width:38rem}}" }
        section { class: "demo-section", id: "menu-overlay-review",
            h2 { "Cancelable menu overlays" }
            p { "Veto Settings or About before the menu changes. Close with its button or Escape to return focus to the action that opened it. External controls share the active overlay signal." }
            div { class: "demo-row",
                Checkbox { label: "Veto Settings action", checked: veto_settings(), oncheckedchange: move |value| veto_settings.set(value) }
                Checkbox { label: "Veto About action", checked: veto_about(), oncheckedchange: move |value| veto_about.set(value) }
            }
            div { class: "demo-row",
                Button { class: "menu-review-programmatic-open", onclick: move |_| active.set(Some(MenuOverlayKind::Settings)), "Open Settings externally" }
                Button { class: "menu-review-external-close", variant: ButtonVariant::Secondary, onclick: move |_| active.set(None), "Close from outside" }
            }
            p { role: "status", id: "menu-overlay-review-status",
                "Settings requests: {settings_attempts}; accepted: {settings_accepted}. About requests: {about_attempts}; accepted: {about_accepted}. Explicit closes: {closes}. Active: {active():?}."
            }
            div { class: "menu-overlay-review-frame",
                Application { force_breakpoint: ApplicationBreakpoint::Large, pinned_menu: true,
                    header: rsx! { ApplicationHeader { name: "Menu behavior" } },
                    menu: rsx! { Menu { active_overlay: active,
                        onbeforeopensettings: move |event: MenuOpenEvent| {
                            settings_attempts += 1;
                            if veto_settings() { event.prevent_default(); }
                        },
                        onbeforeopenabout: move |event: MenuOpenEvent| {
                            about_attempts += 1;
                            if veto_about() { event.prevent_default(); }
                        },
                        onopensettings: move |_| settings_accepted += 1,
                        onopenabout: move |_| about_accepted += 1,
                        settings: rsx! { MenuSettings { onclose: move |_| closes += 1,
                            p { "The Settings action was accepted. Close this panel to return focus to its invoker." }
                            Button { "Settings content action" }
                        } },
                        about: rsx! { MenuAbout { onclose: move |_| closes += 1,
                            TabSet { items: vec![TabItem::new("overview", "Overview"), TabItem::new("news", "News")], active_key: about_tab(), onselectedchange: move |value| about_tab.set(value),
                                TabPanel { tab_key: "overview", p { "Current tab components share the About overlay." } }
                                TabPanel { tab_key: "news", p { "Updates remain available through the controlled About tab." } }
                            }
                        } },
                        MenuItem { label: "Review home", icon: "home", home: true }
                    } },
                    p { "Use Settings and About in the navigation rail to test accepted and vetoed requests." }
                }
            }
        }
    }
}
