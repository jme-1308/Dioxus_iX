use super::*;

#[derive(Default)]
struct PaneFormsCapture {
    runtime: Option<crate::overlay_runtime::RuntimeHandle>,
    visible: Option<Signal<bool>>,
    epoch: Option<Signal<u64>>,
}
#[component]
fn PaneStandardForms(prefix: String, epoch: u64) -> Element {
    rsx! {
        Input {id:format!("{prefix}-input-{epoch}"),autofocus:true,value:"Pump"}
        Textarea {id:format!("{prefix}-textarea-{epoch}"),value:"Inspection"}
        Select {id:format!("{prefix}-select-{epoch}"),options:vec![SelectOption::new("a","Alpha")]}
        Checkbox {id:format!("{prefix}-checkbox-{epoch}"),name:"pane-check-mirror",label:"Approved",checked:true}
        Radio {id:format!("{prefix}-radio-{epoch}"),name:"pane-radio",value:"primary",label:"Primary",checked:true}
        Toggle {id:format!("{prefix}-toggle-{epoch}"),name:"pane-toggle-mirror",label:"Remote",checked:true}
    }
}
fn pane_forms_fixture(capture: Rc<std::cell::RefCell<PaneFormsCapture>>) -> Element {
    let runtime = crate::overlay_runtime::use_overlay_runtime();
    let visible = use_signal(|| true);
    let epoch = use_signal(|| 0_u64);
    {
        let mut capture = capture.borrow_mut();
        capture.runtime = Some(runtime);
        capture.visible = Some(visible);
        capture.epoch = Some(epoch);
    }
    rsx! {
        PaneStandardForms {prefix:"outside",epoch:0}
        if visible() {
            crate::Pane {key:"pane-{epoch()}",heading:"Registered forms",expanded:true,
                PaneStandardForms {prefix:"pane",epoch:epoch()}
            }
        }
    }
}
#[derive(Clone)]
struct PaneFormsMount;
impl dioxus::html::RenderedElementBacking for PaneFormsMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct PaneFormsConverter;
macro_rules! pane_forms_converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for PaneFormsConverter {
                $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;pane_forms_converter!(@data $group,event)})*
            }
        };
        (@data Mouse,$event:ident) => {dioxus::html::MouseData::new(GroupFeedbackMouse)};
        (@data Keyboard,$event:ident) => {dioxus::html::KeyboardData::new(GroupFeedbackKey($event.downcast::<Key>().unwrap().clone()))};
        (@data Form,$event:ident) => {dioxus::html::FormData::new(GroupFeedbackForm($event.downcast::<String>().unwrap().clone()))};
        (@data Mounted,$event:ident) => {MountedData::new(PaneFormsMount)};
        (@data $other:ident,$event:ident) => {panic!("unexpected Pane form test event")};
    }
dioxus::html::with_html_event_groups!(pane_forms_converter);
fn mount_pane_forms(dom: &mut VirtualDom, mut mutations: dioxus::core::Mutations) {
    for _ in 0..8 {
        let mut current = dioxus::core::ElementId::ROOT;
        for edit in mutations.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = id
                }
                dioxus::core::Mutation::NewEventListener { name } if name == "mounted" => {
                    let data: Rc<dyn std::any::Any> =
                        Rc::new(dioxus::html::PlatformEventData::new(Box::new(())));
                    dom.runtime()
                        .handle_event("mounted", Event::new(data, false), current);
                }
                _ => {}
            }
        }
        mutations = dom.render_immediate_to_vec();
    }
}
#[test]
fn pane_standard_forms_enroll_native_controls_and_expire_with_keyed_scope() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(PaneFormsConverter));
    let capture = Rc::new(std::cell::RefCell::new(PaneFormsCapture::default()));
    let mut dom = VirtualDom::new_with_props(pane_forms_fixture, capture.clone());
    let mutations = dom.rebuild_to_vec();
    mount_pane_forms(&mut dom, mutations);
    let runtime = capture.borrow().runtime.clone().unwrap();
    let ids = ["input", "textarea", "select", "checkbox", "radio", "toggle"];
    let registered: Vec<_> = ids
        .iter()
        .map(|kind| {
            assert!(
                runtime
                    .capture_registered_focus(&format!("outside-{kind}-0"))
                    .is_none(),
                "ordinary outside form remains outside the surface registry"
            );
            runtime
                .capture_registered_focus(&format!("pane-{kind}-0"))
                .expect("standard control enrolls in Pane scope")
        })
        .collect();
    let retained: Vec<_> = registered
        .iter()
        .map(|target| target.live_node().unwrap())
        .collect();
    assert!(
        runtime
            .capture_registered_focus("pane-select-0-control")
            .is_none(),
        "Select presentation wrapper is not the focus target"
    );
    let html = dioxus_ssr::render(&dom);
    let input = html.find("id=\"pane-input-0\"").unwrap();
    let end = input + html[input..].find('>').unwrap();
    assert!(
        html[input..end].contains("autofocus"),
        "Input authors native autofocus"
    );
    dom.in_scope(ScopeId::ROOT, || capture.borrow().epoch.unwrap().set(1));
    let mutations = dom.render_immediate_to_vec();
    mount_pane_forms(&mut dom, mutations);
    for (kind, target) in ids.iter().zip(&registered) {
        assert!(
            runtime
                .capture_registered_focus(&format!("pane-{kind}-0"))
                .is_none()
        );
        assert!(target.live_node().is_none());
        assert!(
            runtime
                .capture_registered_focus(&format!("pane-{kind}-1"))
                .is_some()
        );
    }
    dom.in_scope(ScopeId::ROOT, || {
        capture.borrow().visible.unwrap().set(false)
    });
    let mutations = dom.render_immediate_to_vec();
    mount_pane_forms(&mut dom, mutations);
    for kind in ids {
        assert!(
            runtime
                .capture_registered_focus(&format!("pane-{kind}-1"))
                .is_none()
        );
    }
    assert!(
        registered
            .iter()
            .zip(&retained)
            .all(|(target, node)| target.matches_node(node)),
        "retaining backing handles cannot retain a retired registration"
    );
}

#[test]
fn text_commit_snapshots_focus_and_deduplicates_enter_then_blur() {
    let mut tracker = TextCommitState {
        current: "initial".into(),
        baseline: None,
    };
    tracker.focus();
    assert_eq!(tracker.commit(), None, "unchanged blur is silent");
    tracker.current = "changed".into();
    assert_eq!(
        tracker.commit().as_deref(),
        Some("changed"),
        "Enter commits"
    );
    assert_eq!(tracker.commit(), None, "repeated Enter and blur are silent");
    tracker.current.clear();
    assert_eq!(
        tracker.commit().as_deref(),
        Some(""),
        "clearing is a change"
    );
    tracker.current = "programmatic before focus".into();
    tracker.focus();
    assert_eq!(tracker.commit(), None, "refocus snapshots latest prop");
    tracker.current = "programmatic while focused".into();
    assert_eq!(
        tracker.commit().as_deref(),
        Some("programmatic while focused")
    );
}

#[derive(Default)]
struct TextCommitTestConfig {
    value: String,
    callback_revision: usize,
    callbacks: Option<(EventHandler<()>, EventHandler<()>)>,
    events: Vec<String>,
}

fn text_commit_fixture(config: Rc<std::cell::RefCell<TextCommitTestConfig>>) -> Element {
    let value = config.borrow().value.clone();
    let revision = config.borrow().callback_revision;
    let output = config.clone();
    let onchange = use_callback(move |value| {
        output
            .borrow_mut()
            .events
            .push(format!("{revision}:{value}"))
    });
    let callbacks = use_text_commit(&value, onchange);
    config.borrow_mut().callbacks = Some(callbacks);
    VNode::empty()
}

#[test]
fn text_commit_uses_latest_controlled_props_and_callbacks_without_emitting_on_render() {
    let config = Rc::new(std::cell::RefCell::new(TextCommitTestConfig {
        value: "initial".into(),
        ..Default::default()
    }));
    let mut dom = VirtualDom::new_with_props(text_commit_fixture, config.clone());
    dom.rebuild_in_place();
    let (focus, commit) = config.borrow().callbacks.unwrap();
    dom.in_runtime(|| focus.call(()));
    // A parent-rejected native edit leaves the controlled prop unchanged.
    dom.in_runtime(|| commit.call(()));
    assert!(config.borrow().events.is_empty());
    // Parent transforms an edit before acknowledging it through props.
    config.borrow_mut().value = "ACCEPTED".into();
    config.borrow_mut().callback_revision = 1;
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec();
    assert!(
        config.borrow().events.is_empty(),
        "prop changes alone emit nothing"
    );
    dom.in_runtime(|| commit.call(()));
    dom.in_runtime(|| commit.call(()));
    assert_eq!(config.borrow().events, ["1:ACCEPTED"]);
    config.borrow_mut().value = "programmatic".into();
    config.borrow_mut().callback_revision = 2;
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec();
    assert_eq!(config.borrow().events.len(), 1);
    dom.in_runtime(|| commit.call(()));
    assert_eq!(config.borrow().events, ["1:ACCEPTED", "2:programmatic"]);
}

#[test]
fn field_feedback_ignores_blank_and_inactive_messages_and_keeps_helper_fallback_neutral() {
    for state in [
        FieldState::Invalid,
        FieldState::Warning,
        FieldState::Info,
        FieldState::Valid,
    ] {
        let feedback = field_feedback(
            state,
            Some("Helper".into()),
            Some(" ".into()),
            Some("\t".into()),
            None,
            Some("\n".into()),
        )
        .unwrap();
        assert_eq!(feedback.text, "Helper");
        assert_eq!(feedback.state, FieldState::Default);
    }
    assert!(
        field_feedback(
            FieldState::Default,
            Some(" \t".into()),
            Some("Ignored".into()),
            None,
            None,
            None
        )
        .is_none()
    );
    let feedback = field_feedback(
        FieldState::Invalid,
        Some("Helper".into()),
        Some("Required".into()),
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(feedback.text, "Required");
    assert_eq!(feedback.state, FieldState::Invalid);
}

fn text_feedback_fixture() -> Element {
    rsx! {
        Input { id: "input-feedback", value: "a😀", max_length: 4,
            show_text_as_tooltip: true, state: FieldState::Invalid,
            helper_text: "Inactive input helper", invalid_text: "Input error" }
        Textarea { id: "textarea-feedback", value: "a😀", max_length: 4,
            show_text_as_tooltip: true, state: FieldState::Warning,
            helper_text: "Inactive textarea helper", warning_text: "Textarea warning" }
    }
}

#[test]
fn text_feedback_tooltips_describe_controls_keep_counters_and_render_state_icons_once() {
    let mut dom = VirtualDom::new(text_feedback_fixture);
    dom.rebuild_in_place();
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
    let html = dioxus_ssr::render(&dom);
    for id in ["input-feedback", "textarea-feedback"] {
        assert!(
            html.contains(&format!("aria-describedby=\"{id}-help\"")),
            "{html}"
        );
        assert!(html.contains(&format!("id=\"{id}-help\"")), "{html}");
    }
    assert_eq!(html.matches("role=\"tooltip\"").count(), 2);
    assert_eq!(html.matches("data-placement=\"bottom\"").count(), 2);
    assert_eq!(html.matches("Input error").count(), 1);
    assert_eq!(html.matches("Textarea warning").count(), 1);
    assert_eq!(
        html.matches("3/4").count(),
        2,
        "counters remain outside tooltips"
    );
    assert_eq!(html.matches("ix-helper-icon").count(), 2);
    assert!(!html.contains("Inactive input helper"));
    assert!(!html.contains("Inactive textarea helper"));
}

fn blank_text_feedback_fixture() -> Element {
    rsx! {
        Input { id: "blank-input", show_text_as_tooltip: true, helper_text: " \t", invalid_text: "Ignored inactive error" }
        Textarea { id: "blank-textarea", show_text_as_tooltip: true, state: FieldState::Invalid, invalid_text: " ", helper_text: "\n" }
        Input { id: "inline-input", state: FieldState::Invalid, invalid_text: " ", helper_text: "Inline helper fallback" }
        Textarea { id: "inline-textarea", state: FieldState::Valid, valid_text: "Accepted textarea" }
    }
}

#[test]
fn blank_tooltips_are_absent_and_inline_feedback_has_matching_accessible_ids() {
    let mut dom = VirtualDom::new(blank_text_feedback_fixture);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains("role=\"tooltip\""));
    assert!(!html.contains("blank-input-help"));
    assert!(!html.contains("blank-textarea-help"));
    assert!(!html.contains("Ignored inactive error"));
    assert!(html.contains("id=\"inline-input-help\""));
    assert!(html.contains("id=\"inline-textarea-help\""));
    assert_eq!(html.matches("Inline helper fallback").count(), 1);
    assert_eq!(html.matches("Accepted textarea").count(), 1);
    assert_eq!(
        html.matches("ix-helper-icon").count(),
        1,
        "helper fallback has no state icon"
    );
}

fn changing_feedback_mode(config: Rc<std::cell::Cell<(bool, bool)>>) -> Element {
    let (tooltip, feedback) = config.get();
    rsx! {
        Input {id:"mode-input",show_text_as_tooltip:tooltip,helper_text:feedback.then(||"Mode input helper".to_string())}
        Textarea {id:"mode-textarea",show_text_as_tooltip:tooltip,helper_text:feedback.then(||"Mode textarea helper".to_string())}
        Select {id:"mode-select",options:vec![SelectOption::new("a","A")],show_text_as_tooltip:tooltip,helper_text:feedback.then(||"Mode select helper".to_string())}
        Slider {show_tooltip:false,show_text_as_tooltip:tooltip,helper_text:feedback.then(||"Mode slider helper".to_string())}
    }
}

#[test]
fn feedback_mode_and_message_changes_update_explicit_tooltip_descriptions() {
    let config = Rc::new(std::cell::Cell::new((true, true)));
    let mut dom = VirtualDom::new_with_props(changing_feedback_mode, config.clone());
    dom.rebuild_in_place();
    for (tooltip, feedback) in [(true, true), (false, true), (true, false), (true, true)] {
        config.set((tooltip, feedback));
        dom.mark_dirty(ScopeId::APP);
        // Each panel attachment dirties its owning control. Drain the
        // four independent post-render effects and their follow-up renders.
        for _ in 0..6 {
            dom.render_immediate_to_vec();
        }
        let html = dioxus_ssr::render(&dom);
        assert_eq!(
            html.matches("role=\"tooltip\"").count(),
            if tooltip && feedback { 4 } else { 0 },
            "{html}"
        );
        for name in ["input", "textarea", "select", "slider"] {
            assert_eq!(
                html.matches(&format!("Mode {name} helper")).count(),
                usize::from(feedback),
                "{html}"
            );
        }
        let controls = html
            .split('<')
            .filter_map(|part| part.split_once('>').map(|(tag, _)| tag))
            .filter(|tag| {
                tag.starts_with("textarea ")
                    || (tag.starts_with("input ")
                        && (tag.contains("id=\"mode-input\"")
                            || tag.contains("id=\"mode-select\"")
                            || tag.contains("type=\"range\"")))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            controls.len(),
            4,
            "all four actual controls must be checked: {html}"
        );
        for tag in controls {
            let description = tag
                .split_once("aria-describedby=\"")
                .map(|(_, rest)| rest.split('"').next().unwrap());
            assert_eq!(
                description.is_some(),
                feedback,
                "tooltip={tooltip} feedback={feedback}: {tag}; {html}"
            );
            if let Some(id) = description {
                assert!(
                    html.contains(&format!("id=\"{id}\"")),
                    "description must reference current feedback: {html}"
                );
            }
        }
    }
}
#[test]
fn upload_policy_rejects_multiple_plain_files_but_allows_folder_contents() {
    assert_eq!(
        upload_count_allowed(2, false, false),
        Err(UploadError::MultipleFilesNotAllowed)
    );
    assert_eq!(upload_count_allowed(2, false, true), Ok(()));
    assert_eq!(upload_count_allowed(2, true, false), Ok(()));
    assert_eq!(upload_count_allowed(0, false, false), Ok(()));
}

#[test]
fn directory_picker_attributes_are_absent_in_file_mode_and_preserve_native_form_submission() {
    fn normal() -> Element {
        rsx! { Upload { name: "attachment" } }
    }
    fn folder() -> Element {
        rsx! { Upload { directory_upload: true, name: "folder", multiline: true } }
    }
    for (app, directory) in [
        (normal as fn() -> Element, false),
        (folder as fn() -> Element, true),
    ] {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert_eq!(html.contains("webkitdirectory"), directory, "{html}");
        assert_eq!(html.contains(" multiple"), directory, "{html}");
        assert!(html.contains(if directory {
            "name=\"folder\""
        } else {
            "name=\"attachment\""
        }));
        assert!(html.contains(if directory {
            "Upload folder…"
        } else {
            "Upload file…"
        }));
    }
}

#[test]
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn native_folder_selection_keeps_nested_relative_names_and_readable_file_handles() {
    let directory = std::env::temp_dir().join(format!(
        "dioxus-ix-upload-{}-{}",
        std::process::id(),
        FIELD_ID.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&directory).unwrap();
    std::fs::create_dir(directory.join("nested")).unwrap();
    std::fs::write(directory.join("same.rs"), "root").unwrap();
    std::fs::write(directory.join("nested/same.rs"), "nested").unwrap();
    let mut entries = Vec::new();
    collect_upload_directory(&directory, std::path::Path::new("Folder"), &mut entries).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(
        entries[0].relative_path,
        std::path::PathBuf::from("Folder/nested/same.rs")
    );
    assert_eq!(
        entries[1].relative_path,
        std::path::PathBuf::from("Folder/same.rs")
    );
    assert_eq!(entries[0].file.name(), entries[1].file.name());
    assert_ne!(entries[0].file.path(), entries[1].file.path());
    assert_eq!(
        std::fs::read_to_string(entries[0].file.path()).unwrap(),
        "nested"
    );
    assert_eq!(entries[1].file.size(), 4);
    std::fs::remove_file(directory.join("nested/same.rs")).unwrap();
    std::fs::remove_file(directory.join("same.rs")).unwrap();
    std::fs::remove_dir(directory.join("nested")).unwrap();
    std::fs::remove_dir(directory).unwrap();
}

#[test]
fn measured_chip_partition_reacts_to_resize_removal_and_badge_digits() {
    let widths = [80.0, 110.0, 50.0, 70.0];
    let badges = [25.0, 25.0, 25.0, 25.0];
    assert_eq!(fitting_chips(&widths, &badges, 322.0, 4.0), 4);
    assert_eq!(fitting_chips(&widths, &badges, 280.0, 4.0), 3);
    assert_eq!(fitting_chips(&widths, &badges, 210.0, 4.0), 1);
    assert_eq!(fitting_chips(&widths, &badges, 223.0, 4.0), 2);
    assert_eq!(fitting_chips(&widths, &badges, 20.0, 4.0), 1);
    assert_eq!(fitting_chips(&widths[..1], &badges, 20.0, 4.0), 1);
    assert_eq!(fitting_chips(&[], &[], 0.0, 4.0), 0);
    // Remove a selected chip: the remaining values can all reappear.
    assert_eq!(fitting_chips(&[80.0, 50.0], &badges, 134.0, 4.0), 2);
    // A wider two-digit +N indicator changes the number of visible chips.
    let many = [40.0; 12];
    let mut badges = [25.0; 12];
    badges[9..].fill(34.0);
    assert_eq!(fitting_chips(&many, &badges, 117.0, 4.0), 1);
    assert_eq!(fitting_chips(&many, &[25.0; 12], 117.0, 4.0), 2);
}
#[test]
fn select_filter_matches_upstream_substrings_case_and_trimmed_whitespace() {
    assert!(select_matches("Framework Option 10", " OPTION 1 "));
    assert!(select_matches("Überwachung", "ÜBER"));
    assert!(select_matches("Option 1", "  "));
    assert!(!select_matches("Option 1", "Option 10"));
    let options = [
        SelectOption::new("a", "Alpha").disabled(),
        SelectOption::new("b", "Beta"),
    ];
    let enabled: Vec<_> = options
        .iter()
        .filter(|option| select_matches(&option.label, "a") && !option.disabled)
        .map(|option| option.value.as_str())
        .collect();
    assert_eq!(enabled, ["b"]);
}
#[test]
fn select_inline_navigation_starts_from_input_skips_disabled_and_includes_add_row() {
    // Visible source indices 0 and 2 are disabled; index 4 is Add item.
    let enabled = [1, 3, 4];
    assert_eq!(select_next_active(&enabled, usize::MAX, false), Some(1));
    assert_eq!(select_next_active(&enabled, usize::MAX, true), Some(4));
    assert_eq!(select_next_active(&enabled, 1, false), Some(3));
    assert_eq!(select_next_active(&enabled, 3, false), Some(4));
    assert_eq!(select_next_active(&enabled, 4, false), Some(1));
    assert_eq!(select_next_active(&enabled, 1, true), Some(4));
    assert_eq!(select_next_active(&[], usize::MAX, false), None);
    // Removing/filtering the formerly active option restarts at the first.
    assert_eq!(select_next_active(&[3], 1, false), Some(3));
}
#[test]
fn select_add_row_uses_trimmed_exact_label_equality_like_source() {
    let options = [SelectOption::new("existing-id", "Existing")];
    assert!(!select_can_add(&options, "  ", true));
    assert!(!select_can_add(&options, "New", false));
    assert!(!select_can_add(&options, " Existing ", true));
    assert!(select_can_add(&options, "existing", true));
    assert!(select_can_add(&options, "New", true));
}
#[test]
fn select_chip_accessible_labels_accept_translated_prefixes_and_count_templates() {
    assert_eq!(
        select_remove_label("Entfernen", "Überwachung"),
        "Entfernen Überwachung"
    );
    assert_eq!(select_remove_label("إزالة", "مضخة"), "إزالة مضخة");
    assert_eq!(select_remove_label("Remove", ""), "Remove");
    assert_eq!(
        select_more_label("Weitere {count} Elemente", 12),
        "Weitere 12 Elemente"
    );
    assert_eq!(select_more_label("{count} إضافية", 3), "3 إضافية");
    assert_eq!(
        select_more_label("Hidden selections", 3),
        "Hidden selections"
    );
}
#[test]
fn select_required_feedback_waits_for_touch_and_clears_when_selected_or_optional() {
    assert_eq!(
        select_field_state(FieldState::Default, true, false, false),
        FieldState::Default
    );
    assert_eq!(
        select_field_state(FieldState::Default, true, true, false),
        FieldState::Invalid
    );
    assert_eq!(
        select_field_state(FieldState::Default, true, true, true),
        FieldState::Default
    );
    assert_eq!(
        select_field_state(FieldState::Default, false, true, false),
        FieldState::Default
    );
    assert_eq!(
        select_field_state(FieldState::Warning, true, true, false),
        FieldState::Warning
    );
}
#[derive(Clone)]
struct SelectMethodsTestConfig {
    controller: SelectController,
    value: String,
    required: bool,
    visible: bool,
}
fn select_methods_fixture(config: Rc<std::cell::RefCell<SelectMethodsTestConfig>>) -> Element {
    let config = config.borrow().clone();
    rsx! {
        if config.visible {
            Select { id: "method-fixture", controller: config.controller, value: config.value,
                required: config.required, options: vec![SelectOption::new("a", "Alpha")],
                helper_text: "Choose an item", show_text_as_tooltip: true }
        }
    }
}
#[test]
fn select_methods_track_controlled_values_and_reject_stale_or_ambiguous_mounts() {
    let controller = SelectController::default();
    assert_eq!(
        controller.has_valid_value(),
        Err(SelectMethodError::NotMounted)
    );
    let config = Rc::new(std::cell::RefCell::new(SelectMethodsTestConfig {
        controller: controller.clone(),
        value: "a".into(),
        required: true,
        visible: true,
    }));
    let mut dom = VirtualDom::new_with_props(select_methods_fixture, config.clone());
    dom.rebuild_in_place();
    assert_eq!(controller.has_valid_value(), Ok(true));
    assert_eq!(controller.is_touched(), Ok(false));
    assert!(matches!(
        controller.get_native_input_element(),
        Err(SelectMethodError::NotMounted)
    ));
    config.borrow_mut().required = false;
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec();
    assert_eq!(
        controller.has_valid_value(),
        Ok(false),
        "upstream requires both required and a value"
    );
    config.borrow_mut().required = true;
    config.borrow_mut().value.clear();
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec();
    assert_eq!(controller.has_valid_value(), Ok(false));
    let mut second = VirtualDom::new_with_props(select_methods_fixture, config.clone());
    second.rebuild_in_place();
    assert_eq!(
        controller.is_touched(),
        Err(SelectMethodError::MultipleMounted)
    );
    drop(second);
    assert_eq!(controller.is_touched(), Ok(false));
    config.borrow_mut().visible = false;
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec();
    assert_eq!(controller.is_touched(), Err(SelectMethodError::NotMounted));
}

struct SelectFocusMount(Rc<std::cell::Cell<usize>>);
impl dioxus::html::RenderedElementBacking for SelectFocusMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        let count = self.0.clone();
        Box::pin(async move {
            assert!(focus);
            count.set(count.get() + 1);
            Ok(())
        })
    }
}
fn ready_select_method<T>(future: impl std::future::Future<Output = T>) -> T {
    use std::{
        pin::pin,
        task::{Context, Poll, Waker},
    };
    match pin!(future).poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("mock native method did not settle"),
    }
}
struct SelectScrollMount(Rc<std::cell::Cell<usize>>);
impl dioxus::html::RenderedElementBacking for SelectScrollMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn scroll_to(
        &self,
        options: dioxus::html::ScrollToOptions,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        let count = self.0.clone();
        Box::pin(async move {
            assert_eq!(options.behavior, dioxus::html::ScrollBehavior::Instant);
            assert!(matches!(
                options.vertical,
                dioxus::html::ScrollLogicalPosition::Nearest
            ));
            assert!(matches!(
                options.horizontal,
                dioxus::html::ScrollLogicalPosition::Nearest
            ));
            count.set(count.get() + 1);
            Ok(())
        })
    }
}
#[component]
fn SelectScrollTestOption(active: bool, calls: Rc<std::cell::Cell<usize>>) -> Element {
    let mut mounted = use_select_option_scroll(active);
    use_hook(move || mounted.set(Some(Rc::new(MountedData::new(SelectScrollMount(calls))))));
    rsx! { button {} }
}
type SelectScrollConfig = Rc<std::cell::RefCell<Vec<(String, bool, Rc<std::cell::Cell<usize>>)>>>;
fn select_scroll_fixture(config: SelectScrollConfig) -> Element {
    let options = config.borrow().clone();
    rsx! { for (key, active, calls) in options {
        SelectScrollTestOption { key: "{key}", active, calls }
    } }
}
fn settle_select_scroll(dom: &mut VirtualDom) {
    use std::{
        future::Future,
        pin::pin,
        task::{Context, Poll, Waker},
    };
    let mut context = Context::from_waker(Waker::noop());
    for _ in 0..8 {
        let work = { pin!(dom.wait_for_work()).poll(&mut context) };
        match work {
            Poll::Ready(()) => {
                dom.render_immediate_to_vec();
            }
            Poll::Pending => return,
        }
    }
    panic!("Select option effects did not settle");
}
#[test]
fn select_option_scroll_uses_keyed_mounts_across_filtering_and_reordering() {
    let alpha = Rc::new(std::cell::Cell::new(0));
    let beta = Rc::new(std::cell::Cell::new(0));
    let config = Rc::new(std::cell::RefCell::new(vec![
        ("a".into(), true, alpha.clone()),
        ("b".into(), false, beta.clone()),
    ]));
    let mut dom = VirtualDom::new_with_props(select_scroll_fixture, config.clone());
    dom.rebuild_in_place();
    settle_select_scroll(&mut dom);
    assert_eq!((alpha.get(), beta.get()), (1, 0));
    config.borrow_mut().reverse();
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec();
    settle_select_scroll(&mut dom);
    assert_eq!(
        (alpha.get(), beta.get()),
        (1, 0),
        "reordering retains the keyed mount"
    );
    *config.borrow_mut() = vec![("b".into(), true, beta.clone())];
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec();
    settle_select_scroll(&mut dom);
    assert_eq!(
        (alpha.get(), beta.get()),
        (1, 1),
        "filtered-out mounts receive no scroll"
    );
    config.borrow_mut().clear();
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec();
    settle_select_scroll(&mut dom);
    assert_eq!((alpha.get(), beta.get()), (1, 1));
}

#[test]
fn select_native_input_descriptor_focuses_its_mount_and_stops_after_unmount() {
    let mut dom = VirtualDom::new(|| rsx! { div {} });
    dom.rebuild_in_place();
    let focused = dom.in_scope(ScopeId::ROOT, || Signal::new(false));
    let count = Rc::new(std::cell::Cell::new(0));
    let alive = Rc::new(std::cell::Cell::new(true));
    let input = SelectNativeInput {
        id: "choice".into(),
        alive: alive.clone(),
        focused,
        node: Rc::new(MountedData::new(SelectFocusMount(count.clone()))),
    };
    assert_eq!(input.id(), "choice");
    assert_eq!(ready_select_method(input.focus()), Ok(()));
    assert_eq!(count.get(), 1);
    alive.set(false);
    assert_eq!(
        ready_select_method(input.focus()),
        Err(SelectMethodError::NotMounted)
    );
    assert_eq!(
        count.get(),
        1,
        "stale descriptor must not focus a new mount"
    );
    {
        let unsupported = SelectNativeInput {
            id: "unsupported".into(),
            alive: Rc::new(std::cell::Cell::new(true)),
            focused,
            node: Rc::new(MountedData::new(())),
        };
        assert!(matches!(
            ready_select_method(unsupported.focus()),
            Err(SelectMethodError::Dom(_))
        ));
    }
}
struct SelectFocusResponseMount {
    attempts: Rc<std::cell::Cell<usize>>,
    focused: Signal<bool>,
    confirms: Rc<std::cell::Cell<bool>>,
    release: Rc<std::cell::Cell<bool>>,
}
impl dioxus::html::RenderedElementBacking for SelectFocusResponseMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        let attempts = self.attempts.clone();
        let confirms = self.confirms.clone();
        let release = self.release.clone();
        let mut focused = self.focused;
        Box::pin(async move {
            attempts.set(attempts.get() + 1);
            std::future::poll_fn(|_| {
                if release.get() {
                    std::task::Poll::Ready(())
                } else {
                    std::task::Poll::Pending
                }
            })
            .await;
            // Simulate the existing typed onfocus confirmation before a
            // renderer command reports its failed response decoding.
            if confirms.get() {
                focused.set(true);
            }
            Err(dioxus::html::MountedError::OperationFailed(Box::new(
                std::io::Error::other("focus response unavailable"),
            )))
        })
    }
}
#[test]
fn select_focus_normalizes_only_current_confirmation_and_still_attempts_each_call() {
    let mut dom = VirtualDom::new(|| rsx! { div {} });
    dom.rebuild_in_place();
    let mut focused = dom.in_scope(ScopeId::ROOT, || Signal::new(false));
    let attempts = Rc::new(std::cell::Cell::new(0));
    let confirms = Rc::new(std::cell::Cell::new(true));
    let input = SelectNativeInput {
        id: "confirmed-choice".into(),
        alive: Rc::new(std::cell::Cell::new(true)),
        focused,
        node: Rc::new(MountedData::new(SelectFocusResponseMount {
            attempts: attempts.clone(),
            focused,
            confirms: confirms.clone(),
            release: Rc::new(std::cell::Cell::new(true)),
        })),
    };
    for expected in [1, 2] {
        assert_eq!(
            dom.in_scope(ScopeId::ROOT, || ready_select_method(input.focus())),
            Ok(())
        );
        assert_eq!(
            attempts.get(),
            expected,
            "even an already focused input makes exactly one framework attempt"
        );
    }
    dom.in_scope(ScopeId::ROOT, || focused.set(false));
    confirms.set(false);
    assert!(
        matches!(
            dom.in_scope(ScopeId::ROOT, || ready_select_method(input.focus())),
            Err(SelectMethodError::Dom(_))
        ),
        "blurred or unconfirmed focus must preserve the renderer error"
    );
    assert_eq!(attempts.get(), 3);
}
#[test]
fn select_focus_rejects_unmount_while_the_single_mounted_attempt_is_pending() {
    let mut dom = VirtualDom::new(|| rsx! { div {} });
    dom.rebuild_in_place();
    let focused = dom.in_scope(ScopeId::ROOT, || Signal::new(false));
    let alive = Rc::new(std::cell::Cell::new(true));
    let attempts = Rc::new(std::cell::Cell::new(0));
    let release = Rc::new(std::cell::Cell::new(false));
    let input = SelectNativeInput {
        id: "pending-choice".into(),
        alive: alive.clone(),
        focused,
        node: Rc::new(MountedData::new(SelectFocusResponseMount {
            attempts: attempts.clone(),
            focused,
            confirms: Rc::new(std::cell::Cell::new(true)),
            release: release.clone(),
        })),
    };
    let mut operation = Box::pin(input.focus());
    let mut context = std::task::Context::from_waker(std::task::Waker::noop());
    assert!(
        dom.in_scope(ScopeId::ROOT, || operation.as_mut().poll(&mut context))
            .is_pending()
    );
    alive.set(false);
    release.set(true);
    assert_eq!(
        dom.in_scope(ScopeId::ROOT, || operation.as_mut().poll(&mut context)),
        std::task::Poll::Ready(Err(SelectMethodError::NotMounted))
    );
    assert_eq!(attempts.get(), 1);
}
#[test]
fn select_tooltip_exposes_active_feedback_as_the_combobox_description() {
    let controller = SelectController::default();
    let config = Rc::new(std::cell::RefCell::new(SelectMethodsTestConfig {
        controller,
        value: String::new(),
        required: true,
        visible: true,
    }));
    let mut dom = VirtualDom::new_with_props(select_methods_fixture, config);
    dom.rebuild_in_place();
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("aria-describedby=\"method-fixture-help\""));
    assert!(html.contains("id=\"method-fixture-help\""));
    let tooltip = html
        .split('<')
        .find(|fragment| {
            fragment
                .split('>')
                .next()
                .is_some_and(|tag| tag.contains("id=\"method-fixture-help\""))
        })
        .expect("described tooltip element");
    let (attributes, _) = tooltip.split_once('>').expect("tooltip opening tag");
    assert!(attributes.contains("role=\"tooltip\""));
    assert_eq!(html.matches("Choose an item").count(), 1);
    assert!(
        !html.contains("class=\"ix-helper"),
        "tooltip mode must not duplicate feedback below the control"
    );
    assert_eq!(
        select_feedback_text(
            FieldState::Invalid,
            Some("Help".into()),
            Some("Choose an item first".into()),
            None,
            None,
            None
        )
        .as_deref(),
        Some("Choose an item first")
    );
    assert_eq!(
        select_feedback_text(
            FieldState::Info,
            Some("Help".into()),
            None,
            None,
            None,
            None
        )
        .as_deref(),
        Some("Help")
    );
}

#[test]
fn select_controller_reassignment_releases_the_old_handle_without_remounting_the_input() {
    let first = SelectController::default();
    let second = SelectController::default();
    let config = Rc::new(std::cell::RefCell::new(SelectMethodsTestConfig {
        controller: first.clone(),
        value: "a".into(),
        required: true,
        visible: true,
    }));
    let mut dom = VirtualDom::new_with_props(select_methods_fixture, config.clone());
    dom.rebuild_in_place();
    assert_eq!(first.has_valid_value(), Ok(true));
    assert_eq!(second.has_valid_value(), Err(SelectMethodError::NotMounted));
    let first_binding = first.binding().unwrap();
    config.borrow_mut().controller = second.clone();
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec();
    assert_eq!(first.has_valid_value(), Err(SelectMethodError::NotMounted));
    assert_eq!(second.has_valid_value(), Ok(true));
    let second_binding = second.binding().unwrap();
    assert!(
        Rc::ptr_eq(&first_binding.alive, &second_binding.alive),
        "controller replacement must preserve the actual input mount"
    );
    assert!(Rc::ptr_eq(&first_binding.input, &second_binding.input));
    assert_eq!(second_binding.id, "method-fixture");
    drop(dom);
    assert!(!first_binding.alive.get());
    assert!(!second_binding.alive.get());
    assert_eq!(second.is_touched(), Err(SelectMethodError::NotMounted));
}
#[test]
fn select_add_cancellation_is_shared_and_scoped_to_each_request() {
    let rejected = SelectAddEvent::new("Keep typed text".into());
    let listener = rejected.clone();
    assert!(!rejected.default_prevented());
    listener.prevent_default();
    assert!(rejected.default_prevented());
    assert_eq!(rejected.value, "Keep typed text");
    assert!(!SelectAddEvent::new("Another option".into()).default_prevented());
}
fn select_add_guard_fixture() -> Element {
    let cancel = EventHandler::new(|event: SelectAddEvent| event.prevent_default());
    let allow = EventHandler::new(|_: SelectAddEvent| {});
    let rejected = select_add_allowed("Rejected".into(), cancel);
    let accepted = select_add_allowed("Accepted".into(), allow);
    rsx! { output { "{rejected}/{accepted}" } }
}
#[test]
fn select_add_callback_can_veto_the_default_before_insertion() {
    let mut dom = VirtualDom::new(select_add_guard_fixture);
    dom.rebuild_in_place();
    assert!(dioxus_ssr::render(&dom).contains("false/true"));
}
fn select_surface_contract_fixture() -> Element {
    rsx! {
        SelectOptionsSurface { rtl: false, input_id: "choice",
            anchor: MountedAnchor::default(), content_revision: 0, enable_top_layer: true,
            dropdown_width: Some("320px".into()), dropdown_max_width: None,
            onnativeerror: move |_| {}, onclose: move |_| {}, "Native list" }
        SelectOptionsSurface { rtl: true, input_id: "inline-choice",
            anchor: MountedAnchor::default(), content_revision: 0, enable_top_layer: false,
            dropdown_width: Some("280px".into()), dropdown_max_width: Some("90%".into()),
            onnativeerror: move |_| {}, onclose: move |_| {}, "Inline list" }
    }
}
#[test]
fn select_top_layer_is_opt_in_and_inline_fallback_preserves_custom_sizes() {
    let mut dom = VirtualDom::new(select_surface_contract_fixture);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert_eq!(html.matches("popover=\"auto\"").count(), 1);
    assert_eq!(html.matches("data-top-layer=\"true\"").count(), 1);
    assert_eq!(html.matches("data-top-layer=\"false\"").count(), 1);
    assert_eq!(
        html.matches("aria-label=\"Close selection list\"").count(),
        1
    );
    assert!(html.contains("width:280px;max-width:90%"));
}
type SelectSurfaceTestConfig = Rc<std::cell::RefCell<(Option<String>, Option<String>)>>;
fn select_reactive_surface_fixture(config: SelectSurfaceTestConfig) -> Element {
    let (width, max_width) = config.borrow().clone();
    rsx! {
        SelectOptionsSurface { rtl: false, input_id: "reactive-choice",
            anchor: MountedAnchor::default(), content_revision: 0, enable_top_layer: true,
            dropdown_width: width, dropdown_max_width: max_width,
            onnativeerror: move |_| {}, onclose: move |_| {}, "Stable open list" }
    }
}
#[test]
fn select_open_surface_receives_changed_and_removed_width_constraints_without_remounting() {
    let config = Rc::new(std::cell::RefCell::new((Some("480px".into()), None)));
    let mut dom = VirtualDom::new_with_props(select_reactive_surface_fixture, config.clone());
    dom.rebuild_in_place();
    let initial = dioxus_ssr::render(&dom);
    let id = initial
        .split("id=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap()
        .to_string();
    assert!(initial.contains("data-requested-width=\"480px\""));
    assert!(initial.contains("width:480px;max-width:100%"));
    *config.borrow_mut() = (Some("480px".into()), Some("360px".into()));
    // Dioxus 0.8 inserts root error/suspense boundaries. The fixture lives
    // at APP; rerendering ROOT only revisits their memoized child props.
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec();
    let limited = dioxus_ssr::render(&dom);
    assert!(limited.contains(&format!("id=\"{id}\"")));
    assert!(limited.contains("data-requested-max-width=\"360px\""));
    assert!(limited.contains("width:480px;max-width:360px"));
    *config.borrow_mut() = (None, None);
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec();
    let reset = dioxus_ssr::render(&dom);
    assert!(reset.contains(&format!("id=\"{id}\"")));
    assert!(!reset.contains("data-requested-width="));
    assert!(!reset.contains("data-requested-max-width="));
    assert!(reset.contains("width:100%;max-width:100%"));
}
#[test]
fn select_direction_probe_ignores_hidden_and_unresolved_resize_facts() {
    assert_eq!(select_rtl_from_probe_width(8.0), Some(false));
    assert_eq!(select_rtl_from_probe_width(4.0), Some(true));
    assert_eq!(select_rtl_from_probe_width(7.9), Some(false));
    assert_eq!(select_rtl_from_probe_width(4.1), Some(true));
    for width in [0.0, -4.0, 1.0, 6.0, 12.0, f64::NAN, f64::INFINITY] {
        assert_eq!(select_rtl_from_probe_width(width), None);
    }
}
#[test]
fn select_popup_placement_flips_and_aligns_rtl_and_width_constraints_in_rust() {
    let base = SelectPopupMetrics {
        anchor_x: 360.0,
        anchor_y: 80.0,
        anchor_width: 240.0,
        anchor_height: 40.0,
        popup_width: 240.0,
        popup_height: 180.0,
        content_height: 180.0,
        viewport_left: 0.0,
        viewport_top: 0.0,
        viewport_width: 1000.0,
        viewport_height: 600.0,
        rtl: true,
    };
    let wide_rtl = place_select_popup(
        Some(SelectPopupMetrics {
            popup_width: 480.0,
            ..base
        }),
        Some("480px"),
        Some("600px"),
    );
    assert_eq!(wide_rtl.placement, "bottom-start");
    assert_eq!(wide_rtl.direction, "rtl");
    assert!(wide_rtl.style.contains("left:120px;top:120px"));
    let capped_rtl = place_select_popup(
        Some(SelectPopupMetrics {
            popup_width: 360.0,
            ..base
        }),
        Some("480px"),
        Some("360px"),
    );
    assert!(capped_rtl.style.contains("left:240px;top:120px"));
    let percent_rtl = place_select_popup(
        Some(SelectPopupMetrics {
            popup_width: 500.0,
            ..base
        }),
        Some("50%"),
        None,
    );
    assert!(percent_rtl.style.contains("left:100px;top:120px"));
    let clipped = SelectPopupMetrics {
        anchor_y: 400.0,
        popup_height: 100.0,
        content_height: 700.0,
        ..base
    };
    assert_eq!(
        place_select_popup(Some(clipped), None, None).placement,
        "top-start"
    );
    let near_bottom = SelectPopupMetrics {
        anchor_y: 500.0,
        rtl: false,
        ..base
    };
    let flipped = place_select_popup(Some(near_bottom), None, None);
    assert_eq!(flipped.placement, "top-start");
    assert_eq!(flipped.direction, "ltr");
    assert!(flipped.style.contains("left:360px;top:320px"));
}
#[test]
fn select_popup_stays_within_offset_visual_viewports_after_resize() {
    let metrics = SelectPopupMetrics {
        anchor_x: 480.0,
        anchor_y: 450.0,
        anchor_width: 180.0,
        anchor_height: 32.0,
        popup_width: 250.0,
        popup_height: 160.0,
        content_height: 200.0,
        viewport_left: 100.0,
        viewport_top: 200.0,
        viewport_width: 500.0,
        viewport_height: 400.0,
        rtl: true,
    };
    let initial = place_select_popup(Some(metrics), Some("250px"), None);
    assert_eq!(initial.placement, "top-start");
    assert_eq!(initial.direction, "rtl");
    assert!(initial.style.contains("left:350px;top:290px"));
    let resized = place_select_popup(
        Some(SelectPopupMetrics {
            viewport_left: 250.0,
            viewport_top: 300.0,
            viewport_width: 280.0,
            viewport_height: 240.0,
            ..metrics
        }),
        Some("250px"),
        None,
    );
    assert_eq!(resized.placement, "top-start");
    assert!(resized.style.contains("left:280px;top:300px"));
    assert!(resized.style.contains("max-height:min(150px"));
}
fn inline_select_contract_fixture() -> Element {
    rsx! {
        Select { id: "inline-choice", name: "choice", label: "Choice", value: "b",
            options: vec![SelectOption::new("a", "Alpha"), SelectOption::new("b", "Beta")], allow_clear: true }
        Select { id: "editable-choice", editable: true, editable_placeholder: "Create an option", options: vec![] }
        Select { id: "readonly-choice", readonly: true, value: "a", options: vec![SelectOption::new("a", "Alpha")] }
    }
}
#[test]
fn select_visible_combobox_is_inline_native_text_and_submission_keeps_option_value() {
    let mut dom = VirtualDom::new(inline_select_contract_fixture);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    let input = html
        .split("<input")
        .find(|tag| {
            tag.split('>')
                .next()
                .is_some_and(|tag| tag.contains("id=\"inline-choice\""))
        })
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    assert!(input.contains("type=\"text\""));
    assert!(html.contains("role=\"combobox\""));
    assert!(html.contains("aria-autocomplete=\"list\""));
    assert!(html.contains("value=\"Beta\""));
    assert!(html.contains("value=\"b\" selected"));
    assert!(html.contains("name=\"choice\""));
    assert!(html.contains("placeholder=\"Create an option\""));
    assert!(html.contains("readonly"));
    assert!(!html.contains("Filter options"));
}
fn native_contract_fixture() -> Element {
    rsx! {
        Input { input_type: "password", value: "a😀", max_length: 4 }
        Select { name: "choices", multiple: true, values: vec!["a".into(), "b".into()],
            options: vec![SelectOption::new("a", "Alpha"), SelectOption::new("b", "Beta")] }
    }
}
#[test]
fn password_and_multiple_selection_keep_native_and_accessible_contracts() {
    let mut dom = VirtualDom::new(native_contract_fixture);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("type=\"password\""));
    assert!(html.contains("aria-label=\"Show password\""));
    assert!(
        html.contains("3/4"),
        "Length counters must match HTML UTF-16 length"
    );
    assert!(html.contains("aria-label=\"Remove Alpha\""));
    assert!(html.contains("aria-label=\"Remove Beta\""));
    assert!(html.contains("name=\"choices\""));
    assert!(html.contains("class=\"ix-select-native\""));
    assert!(html.contains("aria-hidden=\"true\""));
    let chip_start = html.find("ix-select-chips").unwrap();
    let trigger_start = html
        .find("ix-select-trigger ix-select-trigger-multiple")
        .unwrap();
    assert!(
        chip_start < trigger_start,
        "Chip buttons must precede the inline combobox input"
    );
}
#[test]
fn character_filter_checks_every_inserted_unicode_character() {
    let letters = CharacterPattern::new(Some("[a-d]".into()));
    assert!(letters.allows("abcd"));
    assert!(letters.allows(""));
    assert!(!letters.allows("abed"));
    assert!(!letters.allows("a\n"));
    let unicode = CharacterPattern::new(Some("\\p{L}".into()));
    assert!(unicode.allows("å漢é"));
    assert!(!unicode.allows("å1"));
    assert!(CharacterPattern::new(None).allows("anything 123"));
    assert!(CharacterPattern::new(Some("[".into())).allows("anything"));
}
#[test]
fn number_steps_preserve_decimal_precision_and_clamp_bounds() {
    assert_eq!(stepped_number("0.2", 0.1, None, None, true), "0.3");
    assert_eq!(stepped_number("0.3", 0.1, None, None, false), "0.2");
    assert_eq!(stepped_number("0.15", 0.1, None, None, true), "0.25");
    assert_eq!(stepped_number("9", 2.0, None, Some(10.0), true), "10");
    assert_eq!(stepped_number("1", 2.0, Some(0.0), None, false), "0");
    assert_eq!(stepped_number("", 1.0, None, None, true), "1");
    assert_eq!(stepped_number("NaN", f64::NAN, None, None, false), "-1");
    assert_eq!(stepped_number("0.1", 0.1, None, None, false), "0");
    assert_eq!(
        stepped_number("0.0000001", 0.0000001, None, None, true),
        "0.0000002"
    );
}
#[test]
fn slider_values_are_bounded_and_nonfinite_values_do_not_escape() {
    assert_eq!(bounded_slider_value(-4.0, 0.0, 100.0), 0.0);
    assert_eq!(bounded_slider_value(250.0, 0.0, 100.0), 100.0);
    assert_eq!(bounded_slider_value(25.0, 10.0, 5.0), 10.0);
    assert_eq!(bounded_slider_value(f64::NAN, 10.0, 20.0), 10.0);
    assert_eq!(bounded_slider_value(50.0, f64::NAN, f64::NAN), 50.0);
    assert_eq!(bounded_slider_value(f64::INFINITY, 0.0, 100.0), 0.0);
}
#[test]
fn single_selection_replaces_and_multiple_selection_toggles() {
    assert_eq!(
        update_selection(vec!["a".into()], "b".into(), false),
        vec!["b"]
    );
    assert_eq!(
        update_selection(vec!["a".into()], "b".into(), true),
        vec!["a", "b"]
    );
    assert_eq!(
        update_selection(vec!["a".into(), "b".into()], "a".into(), true),
        vec!["b"]
    );
}
#[test]
fn number_input_typed_events_keep_empty_and_zero_distinct() {
    assert_eq!(number_input_event_value("", false), Some(0.0));
    assert_eq!(number_input_event_value(" ", true), None);
    assert_eq!(number_input_event_value("0", true), Some(0.0));
    assert_eq!(number_input_event_value("1E6", true), Some(1_000_000.0));
    assert_eq!(number_input_event_value("1e-6", true), Some(0.000001));
    assert_eq!(number_input_event_value("-2.5E2", false), Some(-250.0));
    assert_eq!(number_input_value("NaN"), None);
    assert_eq!(number_input_value("Infinity"), None);
    assert_eq!(number_input_value("1e999"), None);
}

#[test]
fn number_input_commit_tracks_focus_and_deduplicates_enter_then_blur() {
    let mut commit = NumberCommit::new("123");
    commit.focus("123");
    assert!(!commit.commit(Some(123.0)));
    assert!(
        commit.commit(Some(1_000_000.0)),
        "Enter commits the numeric change"
    );
    assert!(
        !commit.commit(Some(1_000_000.0)),
        "Blur must not repeat Enter"
    );
    assert!(
        commit.commit(None),
        "Clearing commits empty, including with zero-valued input events"
    );
    assert!(!commit.commit(None));
    commit.focus("1e2");
    assert!(
        !commit.commit(Some(100.0)),
        "Equivalent notation is not a numeric change"
    );
    assert!(commit.commit(Some(101.0)));
}

#[test]
fn number_input_blur_format_normalizes_scientific_notation_and_negative_zero() {
    assert_eq!(number_input_format_value(1e6), "1000000");
    assert_eq!(number_input_format_value(1e-6), "0.000001");
    assert_eq!(number_input_format_value(1e-7), "1e-7");
    assert_eq!(number_input_format_value(1e21), "1e+21");
    assert_eq!(number_input_format_value(-0.0), "0");
}

#[test]
fn number_input_step_accepts_current_zero_and_negative_step_contracts() {
    assert_eq!(stepped_number("4", 0.0, None, None, true), "4");
    assert_eq!(stepped_number("4", -0.5, None, None, true), "3.5");
    assert_eq!(stepped_number("4", -0.5, None, None, false), "4.5");
    assert_eq!(stepped_number("1e2", 0.1, None, None, true), "100.1");
    assert_eq!(
        stepped_number("1e-17", 1e-17, None, None, true),
        "0.00000000000000002"
    );
}

fn number_input_contract_fixture() -> Element {
    rsx! {
        NumberInput { id: "number-default", label: "Default numeric value" }
        NumberInput { id: "number-empty", value: "", text_alignment_end: false,
            placeholder: "Enter an amount", required: true, suppress_submit_on_enter: true,
            show_stepper_buttons: true, name: "amount" }
        NumberInput { id: "number-readonly", value: "12", readonly: true, show_stepper_buttons: true }
        NumberInput { id: "number-disabled", value: "12", disabled: true, show_stepper_buttons: true }
    }
}

#[test]
fn number_input_native_contract_defaults_to_zero_and_hides_unusable_steppers() {
    let mut dom = VirtualDom::new(number_input_contract_fixture);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("id=\"number-default\""));
    assert!(html.contains("value=\"0\""));
    assert!(html.contains("text-align:end"));
    assert!(html.contains("text-align:start"));
    assert!(html.contains("placeholder=\"Enter an amount\""));
    assert!(html.contains("name=\"amount\""));
    assert_eq!(html.matches("aria-label=\"increment number\"").count(), 4);
    assert_eq!(html.matches("aria-label=\"decrement number\"").count(), 4);
    assert_eq!(html.matches("data-hidden=\"true\"").count(), 3);
    assert_eq!(html.matches("data-hidden=\"false\"").count(), 1);
}
include!("../group_feedback_tests.rs");
include!("../external_form_tests.rs");

mod slider_content_tests {
    use super::*;
    include!("slider_content_tests.rs");
}

mod slider_value_tooltip_tests {
    use super::*;
    include!("slider_value_tooltip_tests.rs");
}

mod text_mount_tests {
    use super::*;
    include!("text_mount_tests.rs");
}

mod select_mount_tests {
    use super::*;
    include!("select_mount_tests.rs");
}

mod select_press_tests {
    use super::*;
    include!("select_press_tests.rs");
}
