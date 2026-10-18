// SPDX-FileCopyrightText: 2026 Siemens AG
// SPDX-License-Identifier: MIT

//! Popover focus policy; native code supplies eligible candidates in document
//! order; registered Dioxus mounted handles execute the selected target. See pinned focus-trap.ts79–94,
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

/// Native document-order facts for a Tab trap. `active` includes the source's
/// resolved native/custom-control descendant equivalence; priority selection is
/// performed after excluded hosts are removed, without opening's header rule.
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub(crate) struct TabCandidate {
    pub(crate) target: String,
    pub(crate) tab_index: i32,
    pub(crate) hosts: Vec<String>,
    pub(crate) active: bool,
}

/// Source document queries use the same positive-index ordering as traps.
/// Eligibility and source DOM projection have already been applied by Rust.
pub(crate) fn tab_order_targets(candidates: &[TabCandidate]) -> Vec<String> {
    let mut ordered = candidates.iter().enumerate().collect::<Vec<_>>();
    ordered.sort_by_key(|(index, candidate)| {
        let positive = candidate.tab_index > 0;
        (
            !positive,
            if positive { candidate.tab_index } else { 0 },
            *index,
        )
    });
    ordered
        .into_iter()
        .map(|(_, candidate)| candidate.target.clone())
        .collect()
}

/// Only authorize a boundary wrap. Interior and unknown focus keep the original
/// native Tab default, and an empty trap never invents a panel-focus fallback.
pub(crate) fn tab_boundary_target(
    candidates: &[TabCandidate],
    excluded_hosts: &[String],
    backwards: bool,
) -> Option<String> {
    let mut ordered = candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| {
            !candidate
                .hosts
                .iter()
                .any(|host| excluded_hosts.contains(host))
        })
        .collect::<Vec<_>>();
    ordered.sort_by_key(|(document_order, candidate)| {
        let positive = candidate.tab_index > 0;
        (
            !positive,
            if positive { candidate.tab_index } else { 0 },
            *document_order,
        )
    });
    let active = ordered.iter().position(|(_, candidate)| candidate.active)?;
    let target = if backwards && active == 0 {
        ordered.last()
    } else if !backwards && active + 1 == ordered.len() {
        ordered.first()
    } else {
        None
    }?;
    Some(target.1.target.clone())
}

/// Dropdown exit asks the immediate Popover parent for its trigger's adjacent
/// candidate, wrapping within that parent and omitting the child overlay path.
pub(crate) fn tab_adjacent_target(
    candidates: &[TabCandidate],
    excluded_hosts: &[String],
    current: &str,
    backwards: bool,
) -> Option<String> {
    let mut ordered = candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| {
            !candidate
                .hosts
                .iter()
                .any(|host| excluded_hosts.contains(host))
        })
        .collect::<Vec<_>>();
    ordered.sort_by_key(|(document_order, candidate)| {
        let positive = candidate.tab_index > 0;
        (
            !positive,
            if positive { candidate.tab_index } else { 0 },
            *document_order,
        )
    });
    let index = ordered
        .iter()
        .position(|(_, candidate)| candidate.target == current)?;
    let next = if backwards {
        (index + ordered.len() - 1) % ordered.len()
    } else {
        (index + 1) % ordered.len()
    };
    Some(ordered[next].1.target.clone())
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

    fn tab_candidate(target: &str, tab_index: i32, hosts: &[&str], active: bool) -> TabCandidate {
        TabCandidate {
            target: target.into(),
            tab_index,
            hosts: hosts.iter().map(|host| (*host).into()).collect(),
            active,
        }
    }

    #[test]
    fn tab_header_uses_ordinary_order_and_interior_keeps_native_default() {
        let mut candidates = [
            tab_candidate("header-close", 0, &["parent"], true),
            tab_candidate("body-first", 0, &["parent"], false),
            tab_candidate("body-last", 0, &["parent"], false),
        ];
        assert_eq!(
            tab_boundary_target(&candidates, &[], true).as_deref(),
            Some("body-last")
        );
        assert_eq!(tab_boundary_target(&candidates, &[], false), None);
        candidates[0].active = false;
        candidates[1].active = true;
        assert_eq!(tab_boundary_target(&candidates, &[], false), None);
        assert_eq!(tab_boundary_target(&candidates, &[], true), None);
        candidates[1].active = false;
        candidates[2].active = true;
        assert_eq!(
            tab_boundary_target(&candidates, &[], false).as_deref(),
            Some("header-close")
        );
    }

    #[test]
    fn tab_positive_order_and_equal_ties_preserve_document_order() {
        let mut candidates = [
            tab_candidate("zero", 0, &[], false),
            tab_candidate("positive-first", 2, &[], true),
            tab_candidate("positive-second", 2, &[], false),
            tab_candidate("positive-later", 9, &[], false),
        ];
        assert_eq!(
            tab_boundary_target(&candidates, &[], true).as_deref(),
            Some("zero")
        );
        candidates[1].active = false;
        candidates[0].active = true;
        assert_eq!(
            tab_boundary_target(&candidates, &[], false).as_deref(),
            Some("positive-first")
        );
        candidates[0].active = false;
        candidates[2].active = true;
        assert_eq!(tab_boundary_target(&candidates, &[], false), None);
        assert_eq!(tab_boundary_target(&candidates, &[], true), None);
    }

    #[test]
    fn tab_excludes_child_hosts_before_resolving_boundary_and_active_fact() {
        let mut candidates = [
            tab_candidate("parent-first", 0, &["parent"], true),
            tab_candidate("child-first", 0, &["parent", "child"], false),
            tab_candidate("child-last", 0, &["parent", "child"], false),
            tab_candidate("parent-last", 0, &["parent"], false),
        ];
        let excluded = ["child".to_string()];
        assert_eq!(
            tab_boundary_target(&candidates, &excluded, true).as_deref(),
            Some("parent-last")
        );
        candidates[0].active = false;
        candidates[2].active = true;
        assert_eq!(tab_boundary_target(&candidates, &excluded, true), None);
        assert_eq!(tab_boundary_target(&candidates, &excluded, false), None);
    }

    #[test]
    fn tab_single_active_candidate_wraps_self_and_empty_or_unknown_does_nothing() {
        let mut candidates = [tab_candidate(
            "custom-host-with-focused-descendant",
            0,
            &[],
            true,
        )];
        for backwards in [false, true] {
            assert_eq!(
                tab_boundary_target(&candidates, &[], backwards).as_deref(),
                Some("custom-host-with-focused-descendant")
            );
            assert_eq!(tab_boundary_target(&[], &[], backwards), None);
        }
        candidates[0].active = false;
        assert_eq!(tab_boundary_target(&candidates, &[], false), None);
        assert_eq!(tab_boundary_target(&candidates, &[], true), None);
    }
}
