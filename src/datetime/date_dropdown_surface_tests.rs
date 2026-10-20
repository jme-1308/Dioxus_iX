//! Shared surface semantics and controlled rendering; native focus/dismissal is
//! verified by the paired inspection-report workflow and retained event tests.
use super::*;
fn date(year: u16, month: u8, day: u8) -> IxDate {
    IxDate::new(year, month, day).unwrap()
}
fn settle(dom: &mut VirtualDom) {
    for _ in 0..8 {
        dom.render_immediate_to_vec();
        dom.process_events();
    }
}
fn render(component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    settle(&mut dom);
    dioxus_ssr::render(&dom)
}
fn opening<'a>(html: &'a str, tag: &str, marker: &str) -> &'a str {
    html.split('<')
        .filter(|part| part.starts_with(&format!("{tag} ")))
        .map(|part| part.split('>').next().unwrap())
        .find(|part| part.contains(marker))
        .unwrap_or_else(|| panic!("missing {tag} {marker}: {html}"))
}
fn attr<'a>(opening: &'a str, name: &str) -> Option<&'a str> {
    let value = opening.split_once(&format!(" {name}="))?.1;
    if let Some(value) = value.strip_prefix('"') {
        Some(value.split('"').next().unwrap())
    } else {
        Some(
            value
                .split(|character: char| character.is_whitespace() || character == '>')
                .next()
                .unwrap(),
        )
    }
}
#[test]
fn default_menu_roles_remain_while_dialog_role_reaches_built_in_and_external_triggers() {
    fn fixture() -> Element {
        let menu = crate::use_dropdown();
        let dialog = crate::use_dropdown();
        rsx! {
            crate::Dropdown {label:"Legacy menu",crate::DropdownItem {label:"Action"}}
            crate::Dropdown {label:"Built-in dialog",surface_role:crate::DropdownSurfaceRole::Dialog,disable_focus_handling:true,input {"aria-label":"Dialog entry"}}
            crate::DropdownTrigger {dropdown:menu,aria_label:"External menu trigger","Menu"}
            crate::Dropdown {label:"External menu",controller:menu,"Menu content"}
            crate::DropdownTrigger {dropdown:dialog,aria_label:"External dialog trigger","Dialog"}
            crate::Dropdown {label:"External dialog",controller:dialog,surface_role:crate::DropdownSurfaceRole::Dialog,disable_focus_handling:true,"Dialog content"}
        }
    }
    let html = render(fixture);
    for (name, role) in [("Legacy menu", "menu"), ("Built-in dialog", "dialog")] {
        assert_eq!(
            attr(
                opening(&html, "div", &format!("aria-label=\"{name}\"")),
                "aria-haspopup"
            ),
            Some(role)
        );
        assert_eq!(
            attr(
                opening(&html, "dialog", &format!("aria-label=\"{name}\"")),
                "role"
            ),
            Some(role)
        );
    }
    for (name, role) in [
        ("External menu trigger", "menu"),
        ("External dialog trigger", "dialog"),
    ] {
        let trigger = opening(&html, "button", &format!("aria-label=\"{name}\""));
        assert_eq!(attr(trigger, "aria-haspopup"), Some(role), "{html}");
        let host = attr(trigger, "aria-controls").unwrap();
        assert_eq!(
            attr(opening(&html, "dialog", &format!("id=\"{host}\"")), "role"),
            Some(role)
        );
    }
}
#[test]
fn date_dropdown_uses_a_persistent_shared_dialog_without_its_private_backdrop() {
    let html = render(|| {
        rsx! {
            DateDropdown {id:"inline-period",class:"caller-date-class",today:date(2026,10,20),onrangechange:|_|{}}
            DateDropdown {id:"top-period",today:date(2026,10,20),enable_top_layer:true,onrangechange:|_|{}}
        }
    });
    assert!(opening(&html, "div", "id=\"inline-period\"").contains("caller-date-class"));
    assert_eq!(
        html.matches("class=\"ix-date-dropdown-content\"").count(),
        2
    );
    assert_eq!(html.matches("role=\"dialog\"").count(), 2);
    assert_eq!(html.matches("popover=\"manual\"").count(), 1);
    assert!(!html.contains("ix-temporal-backdrop"));
    assert!(!html.contains("ix-temporal-popover"));
    let trigger = opening(&html, "button", "aria-haspopup=\"dialog\"");
    assert_eq!(attr(trigger, "aria-expanded"), Some("false"));
    let host = attr(trigger, "aria-controls").unwrap();
    let surface = opening(&html, "dialog", &format!("id=\"{host}\""));
    assert_eq!(attr(surface, "role"), Some("dialog"));
    // The pinned SSR renderer omits empty-string inert attributes. The native
    // hidden attribute is the closed surface's layout and focus boundary.
    assert!(surface.contains("hidden"));
    assert!(surface.contains("ix-dropdown-root-menu"));
    assert!(html.contains("No range set"));
}
#[derive(Clone)]
struct Settings {
    from: IxDate,
    to: IxDate,
    disabled: bool,
    loading: bool,
    single: bool,
}
fn controlled(settings: Rc<RefCell<Settings>>) -> Element {
    let current = settings.borrow().clone();
    rsx! {DateDropdown{id:"controlled-period",today:date(2026,10,20),from:current.from,to:current.to,
        disabled:current.disabled,loading:current.loading,single_selection:current.single,onrangechange:|_|{}
    }}
}
#[test]
fn controlled_range_updates_and_disabled_loading_guards_keep_one_native_trigger_and_surface() {
    let settings = Rc::new(RefCell::new(Settings {
        from: date(2026, 10, 10),
        to: date(2026, 10, 20),
        disabled: false,
        loading: false,
        single: false,
    }));
    let mut dom = VirtualDom::new_with_props(controlled, settings.clone());
    dom.rebuild_in_place();
    settle(&mut dom);
    let initial = dioxus_ssr::render(&dom);
    assert!(initial.contains("2026/10/10 - 2026/10/20"));
    let first = opening(&initial, "button", "aria-haspopup=\"dialog\"");
    let trigger_id = attr(first, "id").unwrap().to_string();
    let surface_id = attr(first, "aria-controls").unwrap().to_string();
    for loading in [false, true] {
        {
            let mut current = settings.borrow_mut();
            current.from = date(2026, 10, 15);
            current.to = date(2026, 10, 15);
            current.single = true;
            current.disabled = !loading;
            current.loading = loading;
        }
        dom.mark_dirty(ScopeId::APP);
        settle(&mut dom);
        let html = dioxus_ssr::render(&dom);
        let trigger = opening(&html, "button", "aria-haspopup=\"dialog\"");
        assert_eq!(attr(trigger, "id"), Some(trigger_id.as_str()));
        assert_eq!(attr(trigger, "aria-controls"), Some(surface_id.as_str()));
        assert!(trigger.contains("disabled"));
        assert_eq!(
            attr(trigger, "aria-busy"),
            Some(if loading { "true" } else { "false" })
        );
        assert!(html.contains("2026/10/15"));
        assert!(!html.contains("2026/10/10 - 2026/10/20"));
        assert_eq!(html.matches(&format!("id=\"{surface_id}\"")).count(), 1);
        assert_eq!(html.matches("role=\"dialog\"").count(), 1);
    }
}
