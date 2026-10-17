// SPDX-FileCopyrightText: 2026 Siemens AG
// SPDX-License-Identifier: MIT

//! Popover focus policy; native code supplies eligible candidates in document
//! order and executes only the selected target. See pinned focus-trap.ts79–94,
//! 430–469 and popover.tsx473–518. Candidate eligibility and native focus success
//! are separate facts: this module neither discovers DOM nor invents a fallback.

#[cfg(any(
    test,
    all(feature = "web", target_arch = "wasm32"),
    all(feature = "desktop", not(target_arch = "wasm32"))
))]
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub(crate) struct FocusCandidate {
    pub(crate) target: String,
    pub(crate) tab_index: i32,
    pub(crate) in_header: bool,
}

/// Click opening prefers content over header, with tab order within each role.
/// Hover opening never requests this explicit first-focus operation.
#[cfg(any(
    test,
    all(feature = "web", target_arch = "wasm32"),
    all(feature = "desktop", not(target_arch = "wasm32"))
))]
pub(crate) fn opening_target(candidates: &[FocusCandidate], hover_mode: bool) -> Option<String> {
    if hover_mode {
        return None;
    }
    candidates
        .iter()
        .enumerate()
        .min_by_key(|(document_order, candidate)| {
            let positive = candidate.tab_index > 0;
            (
                candidate.in_header,
                !positive,
                if positive { candidate.tab_index } else { 0 },
                *document_order,
            )
        })
        .map(|(_, candidate)| candidate.target.clone())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PostCloseFocus {
    None,
    /// Restore the configured trigger in a frame after native close.
    Restore,
    /// Release contained/trigger focus in a frame, then blur trigger next frame.
    Release,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RustCloseFocusPlan {
    /// Release contained and trigger focus immediately before native hide.
    pub(crate) pre_release: bool,
    /// Blur the configured trigger immediately after native hide.
    pub(crate) blur_trigger: bool,
    pub(crate) post_close: PostCloseFocus,
}

/// `cached_focusable` is captured at opening, not recomputed at dismissal.
/// A release still blurs its configured trigger after hide when content is
/// static. Only the further explicit corrections require focusable content.
pub(crate) fn close_focus_plan(
    cached_focusable: bool,
    configured_trigger: bool,
    hover_mode: bool,
    restore_focus: bool,
) -> RustCloseFocusPlan {
    let eligible = cached_focusable && configured_trigger;
    let hover_release = hover_mode && !restore_focus;
    RustCloseFocusPlan {
        pre_release: eligible && hover_release,
        blur_trigger: configured_trigger && !restore_focus,
        post_close: if !eligible || hover_release {
            PostCloseFocus::None
        } else if restore_focus {
            PostCloseFocus::Restore
        } else {
            PostCloseFocus::Release
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(target: &str, tab_index: i32, in_header: bool) -> FocusCandidate {
        FocusCandidate {
            target: target.into(),
            tab_index,
            in_header,
        }
    }

    #[test]
    fn opening_prefers_content_even_over_an_earlier_positive_tabindex_header() {
        let candidates = [
            candidate("header-close", 1, true),
            candidate("body-first", 0, false),
            candidate("footer-second", 0, false),
        ];
        assert_eq!(
            opening_target(&candidates, false).as_deref(),
            Some("body-first")
        );
    }

    #[test]
    fn positive_content_tabindices_precede_zero_and_ties_keep_document_order() {
        let candidates = [
            candidate("body-zero", 0, false),
            candidate("body-nine", 9, false),
            candidate("body-two-first", 2, false),
            candidate("body-two-second", 2, false),
        ];
        assert_eq!(
            opening_target(&candidates, false).as_deref(),
            Some("body-two-first")
        );
        assert_eq!(
            opening_target(&candidates[..2], false).as_deref(),
            Some("body-nine")
        );
    }

    #[test]
    fn header_only_falls_back_in_tab_order_and_static_or_hover_has_no_target() {
        let headers = [
            candidate("close-zero", 0, true),
            candidate("header-action", 3, true),
        ];
        assert_eq!(
            opening_target(&headers, false).as_deref(),
            Some("header-action")
        );
        assert_eq!(opening_target(&[], false), None);
        assert_eq!(opening_target(&headers, true), None);
        assert_eq!(opening_target(&[candidate("body", 1, false)], true), None);
    }

    #[test]
    fn nonpositive_bucket_preserves_supplied_document_order_without_new_filtering() {
        let candidates = [
            candidate("provided-negative", -1, false),
            candidate("zero", 0, false),
        ];
        assert_eq!(
            opening_target(&candidates, false).as_deref(),
            Some("provided-negative")
        );
        assert_eq!(
            opening_target(
                &[
                    candidate("max", i32::MAX, false),
                    candidate("zero", 0, false)
                ],
                false
            )
            .as_deref(),
            Some("max")
        );
    }

    #[test]
    fn close_focus_matrix_preserves_static_release_blur_and_hover_frame_distinction() {
        use PostCloseFocus::{None, Release, Restore};
        // cached content, configured trigger, hover, restore, pre, blur, post
        let cases = [
            (false, false, false, false, false, false, None),
            (false, false, false, true, false, false, None),
            (false, false, true, false, false, false, None),
            (false, false, true, true, false, false, None),
            (false, true, false, false, false, true, None),
            (false, true, false, true, false, false, None),
            (false, true, true, false, false, true, None),
            (false, true, true, true, false, false, None),
            (true, false, false, false, false, false, None),
            (true, false, false, true, false, false, None),
            (true, false, true, false, false, false, None),
            (true, false, true, true, false, false, None),
            (true, true, false, false, false, true, Release),
            (true, true, false, true, false, false, Restore),
            (true, true, true, false, true, true, None),
            (true, true, true, true, false, false, Restore),
        ];
        for (content, trigger, hover, restore, pre, blur, post) in cases {
            assert_eq!(
                close_focus_plan(content, trigger, hover, restore),
                RustCloseFocusPlan {
                    pre_release: pre,
                    blur_trigger: blur,
                    post_close: post
                },
                "content={content} trigger={trigger} hover={hover} restore={restore}"
            );
        }
    }
}
