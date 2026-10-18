// Rendering and retained controlled-prop regressions. These do not establish
// browser geometry, trusted input, native picker behavior, or computed parity.
use super::*;
use std::cell::RefCell;

fn attribute<'a>(opening: &'a str, name: &str) -> Option<&'a str> {
    let value = opening.split_once(&format!(" {name}="))?.1;
    if let Some(value) = value.strip_prefix('"') {
        Some(value.split('"').next().unwrap())
    } else if let Some(value) = value.strip_prefix('\'') {
        Some(value.split('\'').next().unwrap())
    } else {
        Some(
            value
                .split(|c: char| c.is_whitespace() || c == '>')
                .next()
                .unwrap(),
        )
    }
}
fn classes(opening: &str) -> Vec<&str> {
    attribute(opening, "class")
        .unwrap_or_default()
        .split_whitespace()
        .collect()
}
fn has_attribute(opening: &str, name: &str) -> bool {
    opening
        .match_indices(&format!(" {name}"))
        .any(|(index, _)| {
            opening[index + name.len() + 1..]
                .chars()
                .next()
                .is_none_or(|c| c.is_whitespace() || c == '=' || c == '>')
        })
}
fn openings(html: &str) -> impl Iterator<Item = &str> {
    html.match_indices('<').filter_map(move |(index, _)| {
        let rest = &html[index..];
        Some(&rest[..=rest.find('>')?])
    })
}
fn part<'a>(html: &'a str, marker: &str) -> (&'a str, &'a str) {
    let (start, opening) = html
        .match_indices('<')
        .find_map(|(index, _)| {
            let rest = &html[index..];
            let opening = &rest[..=rest.find('>')?];
            classes(opening)
                .contains(&marker)
                .then_some((index, opening))
        })
        .unwrap_or_else(|| panic!("missing {marker}: {html}"));
    let tag = opening[1..]
        .split(|c: char| c.is_whitespace() || c == '>')
        .next()
        .unwrap();
    let closing = format!("</{tag}>");
    let rest = &html[start..];
    let mut depth = 0usize;
    for (index, _) in rest.match_indices('<') {
        let current = &rest[index..];
        if current.starts_with(&format!("<{tag} ")) || current.starts_with(&format!("<{tag}>")) {
            depth += 1;
        } else if current.starts_with(&closing) {
            depth -= 1;
            if depth == 0 {
                return (opening, &rest[..index + closing.len()]);
            }
        }
    }
    panic!("unclosed {marker}: {rest}");
}
fn text_content(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for character in html.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }
    text
}
fn native_input(html: &str) -> &str {
    openings(html)
        .find(|opening| {
            opening.starts_with("<input ") && classes(opening).contains(&"ix-upload-browser")
        })
        .expect("native file input")
}
fn render_fixture(component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn upload_layers_contain_a_real_secondary_button_and_named_native_file_input() {
    let html = render_fixture(|| {
        rsx! {
            div { class: "files", Upload { name: "attachments", accept: ".txt" } }
            div { class: "folder", Upload { name: "directory", directory_upload: true, multiline: true } }
            div { class: "custom", Upload { class: "application-upload", multiple: true, select_file_text: "Choose a document", button_text: "Browse documents", aria_label: "Document files" } }
        }
    });
    for (case, name, text, button_label, accessible, directory, multiple) in [
        (
            "files",
            Some("attachments"),
            "+ Drag files here or…",
            "Upload file…",
            "Upload files",
            false,
            false,
        ),
        (
            "folder",
            Some("directory"),
            "+ Drag folder here or…",
            "Upload folder…",
            "Upload folder",
            true,
            true,
        ),
        (
            "custom",
            None,
            "Choose a document",
            "Browse documents",
            "Document files",
            false,
            true,
        ),
    ] {
        let (_, body) = part(&html, case);
        let (root, _) = part(body, "ix-upload");
        assert!(root.starts_with("<div "));
        assert!(classes(root).contains(&"ix-upload-visual"));
        assert_eq!(
            classes(root).contains(&"application-upload"),
            case == "custom"
        );
        assert_eq!(attribute(root, "data-state"), Some("select"));
        assert_eq!(attribute(root, "aria-disabled"), Some("false"));
        assert_eq!(
            attribute(root, "data-multiline"),
            Some(if directory { "true" } else { "false" })
        );
        let (_, area) = part(body, "ix-upload-area");
        let (state, _) = part(area, "ix-upload-state");
        assert_eq!(attribute(state, "role"), Some("status"));
        assert_eq!(text_content(part(area, "ix-upload-text").1), text);
        let (actions, actions_body) = part(area, "ix-upload-actions");
        assert!(actions.starts_with("<div "));
        let input = native_input(actions_body);
        assert_eq!(attribute(input, "type"), Some("file"));
        assert_eq!(attribute(input, "name"), name);
        assert_eq!(attribute(input, "tabindex"), Some("-1"));
        assert_eq!(attribute(input, "aria-label"), Some(accessible));
        assert_eq!(has_attribute(input, "multiple"), multiple);
        assert!(!has_attribute(input, "disabled"));
        assert_eq!(has_attribute(input, "webkitdirectory"), directory);
        assert_eq!(has_attribute(input, "directory"), directory);
        if case == "files" {
            assert_eq!(attribute(input, "accept"), Some(".txt"));
        }
        let (host, host_body) = part(actions_body, "ix-upload-button-host");
        assert!(host.starts_with("<span "));
        let (button, contents) = part(host_body, "ix-upload-button");
        assert!(button.starts_with("<button "));
        for class in [
            "ix-button",
            "ix-button-tokenized",
            "ix-button-standard",
            "ix-button-secondary",
        ] {
            assert!(classes(button).contains(&class));
        }
        assert_eq!(attribute(button, "type"), Some("button"));
        assert!(!has_attribute(button, "disabled"));
        assert_eq!(
            text_content(part(contents, "ix-button-content").1),
            button_label
        );
        assert!(
            !area.contains("ix-upload-status-icon") && !area.contains("ix-upload-spinner-host")
        );
    }
}

#[test]
fn upload_disabled_text_takes_precedence_and_keeps_the_blocked_button_mounted() {
    let html = render_fixture(|| {
        rsx! {
            for (name, state) in [("select", UploadState::SelectFile), ("loading", UploadState::Loading), ("failed", UploadState::Failed), ("success", UploadState::Success)] {
                div { class: "disabled-{name}", Upload { state, disabled: true, disabled_text: "Selection unavailable" } }
            }
        }
    });
    for name in ["select", "loading", "failed", "success"] {
        let (_, body) = part(&html, &format!("disabled-{name}"));
        let (root, _) = part(body, "ix-upload");
        assert_eq!(attribute(root, "data-state"), Some(name));
        assert_eq!(attribute(root, "data-disabled"), Some("true"));
        assert_eq!(attribute(root, "aria-disabled"), Some("true"));
        let (area, _) = part(body, "ix-upload-area");
        assert_eq!(attribute(area, "data-disabled"), Some("true"));
        assert_eq!(
            attribute(area, "data-checking"),
            Some(if name == "loading" { "true" } else { "false" })
        );
        assert_eq!(
            text_content(part(body, "ix-upload-text").1),
            "Selection unavailable"
        );
        assert!(
            !body.contains("ix-upload-status-icon") && !body.contains("ix-upload-spinner-host")
        );
        assert!(has_attribute(native_input(body), "disabled"));
        let (button, button_body) = part(body, "ix-upload-button");
        assert!(has_attribute(button, "disabled"));
        assert_eq!(text_content(button_body), "Upload file…");
    }
}

#[test]
fn upload_source_state_texts_pair_with_the_actual_md_spinner_and_error_success_glyphs() {
    let html = render_fixture(|| {
        rsx! {
            div { class: "loading", Upload { state: UploadState::Loading } }
            div { class: "loading-folder", Upload { state: UploadState::Loading, directory_upload: true } }
            div { class: "failed", Upload { state: UploadState::Failed } }
            div { class: "success", Upload { state: UploadState::Success } }
        }
    });
    for (case, text, glyph) in [
        ("loading", "Checking files…", None),
        ("loading-folder", "Checking folder…", None),
        ("failed", "Upload failed. Please try again.", Some("error")),
        ("success", "Upload successful", Some("success")),
    ] {
        let (_, body) = part(&html, case);
        assert_eq!(text_content(part(body, "ix-upload-text").1), text);
        let (button, _) = part(body, "ix-upload-button");
        assert_eq!(has_attribute(button, "disabled"), glyph.is_none());
        assert_eq!(
            has_attribute(native_input(body), "disabled"),
            glyph.is_none()
        );
        if let Some(glyph) = glyph {
            let (icon, contents) = part(body, "ix-upload-status-icon");
            assert!(icon.starts_with("<svg "));
            assert_eq!(attribute(icon, "width"), Some("24"));
            assert_eq!(attribute(icon, "height"), Some("24"));
            assert_eq!(attribute(icon, "aria-hidden"), Some("true"));
            assert!(contents.contains(crate::icons::icon_svg(glyph).unwrap()));
            assert!(!body.contains("ix-upload-spinner-host"));
        } else {
            let (_, host) = part(body, "ix-upload-spinner-host");
            let (spinner, contents) = part(host, "ix-spinner");
            assert!(classes(spinner).contains(&"ix-spinner-md"));
            assert!(classes(spinner).contains(&"ix-spinner-primary"));
            assert!(contents.contains("ix-spinner-track") && contents.contains("ix-spinner-arc"));
            assert!(!body.contains("ix-upload-status-icon"));
        }
    }
}

#[derive(Clone)]
struct LiveUpload {
    stage: Rc<RefCell<Option<Signal<u8>>>>,
    requests: Rc<RefCell<Vec<&'static str>>>,
}
fn live_upload(state: LiveUpload) -> Element {
    let stage = use_signal(|| 0u8);
    *state.stage.borrow_mut() = Some(stage);
    let current = stage();
    let files = state.requests.clone();
    let entries = state.requests.clone();
    let errors = state.requests.clone();
    rsx! {
        Upload { name: if current == 0 { "original" } else { "updated" },
            accept: if current == 0 { ".txt" } else { ".csv" },
            directory_upload: current == 1, multiple: current == 4, multiline: current >= 3,
            disabled: current == 2,
            state: match current { 1 | 2 => UploadState::Loading, 3 => UploadState::Failed, 4 => UploadState::Success, _ => UploadState::SelectFile },
            select_file_text: "Pick documents", loading_text: "Checking documents", disabled_text: "Selection paused",
            failed_text: "Retry documents", success_text: "Documents accepted",
            button_text: if current == 0 { "Browse" } else { "Browse again" },
            aria_label: if current == 0 { "Original picker" } else { "Updated picker" },
            onfileschanged: move |_| files.borrow_mut().push("files"),
            onentrieschanged: move |_| entries.borrow_mut().push("entries"),
            onerror: move |_| errors.borrow_mut().push("error"),
        }
    }
}

#[test]
fn upload_retained_external_props_update_native_constraints_and_presentation_without_requests() {
    let state = LiveUpload {
        stage: Rc::new(RefCell::new(None)),
        requests: Rc::new(RefCell::new(Vec::new())),
    };
    let mut dom = VirtualDom::new_with_props(live_upload, state.clone());
    dom.rebuild_in_place();
    let mut stage = state.stage.borrow().unwrap();
    let first = dioxus_ssr::render(&dom);
    let input_id = attribute(native_input(&first), "id").unwrap().to_string();
    for current in [0, 1, 2, 3, 4, 0] {
        dom.in_scope(ScopeId::ROOT, || stage.set(current));
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = dioxus_ssr::render(&dom);
        let input = native_input(&html);
        assert_eq!(
            attribute(input, "id"),
            Some(input_id.as_str()),
            "the mounted picker keeps its identity across prop updates"
        );
        assert_eq!(
            attribute(input, "name"),
            Some(if current == 0 { "original" } else { "updated" })
        );
        assert_eq!(
            attribute(input, "accept"),
            Some(if current == 0 { ".txt" } else { ".csv" })
        );
        assert_eq!(
            attribute(input, "aria-label"),
            Some(if current == 0 {
                "Original picker"
            } else {
                "Updated picker"
            })
        );
        assert_eq!(
            has_attribute(input, "multiple"),
            current == 1 || current == 4
        );
        assert_eq!(has_attribute(input, "webkitdirectory"), current == 1);
        assert_eq!(has_attribute(input, "directory"), current == 1);
        assert_eq!(
            has_attribute(input, "disabled"),
            current == 1 || current == 2
        );
        let (button, contents) = part(&html, "ix-upload-button");
        assert_eq!(
            has_attribute(button, "disabled"),
            current == 1 || current == 2
        );
        assert_eq!(
            text_content(contents),
            if current == 0 {
                "Browse"
            } else {
                "Browse again"
            }
        );
        assert_eq!(
            text_content(part(&html, "ix-upload-text").1),
            match current {
                1 => "Checking documents",
                2 => "Selection paused",
                3 => "Retry documents",
                4 => "Documents accepted",
                _ => "Pick documents",
            }
        );
        let (root, _) = part(&html, "ix-upload");
        assert_eq!(
            attribute(root, "data-multiline"),
            Some(if current >= 3 { "true" } else { "false" })
        );
        assert_eq!(html.contains("ix-upload-spinner-host"), current == 1);
        assert_eq!(
            html.contains("ix-upload-status-icon"),
            current == 3 || current == 4
        );
        assert!(
            state.requests.borrow().is_empty(),
            "external controlled prop changes must not emit selection or error callbacks"
        );
    }
}
