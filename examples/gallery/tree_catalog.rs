//! Local catalog parsing belongs to the application; Tree owns no loader API.
use dioxus_ix::{TreeModel, TreeModelItem};
use serde::Deserialize;

pub const DEFAULT_REMOTE_CATALOG: &str = r#"[
  {"id":"model-remote-1","label":"Remote drive 001"},
  {"id":"model-remote-2","label":"Remote drive 002"}
]"#;

#[derive(Deserialize)]
struct CatalogNode {
    id: String,
    label: String,
    #[serde(default)]
    disabled: bool,
    #[serde(default)]
    children: Vec<CatalogNode>,
}

/// Stage a complete replacement of one branch's descendants. Nothing in the
/// current model is mutated on success or failure; callers publish the returned
/// model through their existing Tree controller transaction. The count includes
/// all imported descendants. Empty catalogs make the target a known leaf.
pub fn replace_remote_catalog(
    current: &TreeModel<String>,
    parent: &str,
    json: &str,
) -> Result<(TreeModel<String>, usize), String> {
    let nodes: Vec<CatalogNode> =
        serde_json::from_str(json).map_err(|error| format!("Catalog JSON is invalid: {error}"))?;
    current.validate().map_err(|error| error.to_string())?;
    let old_children = current
        .items
        .get(parent)
        .ok_or_else(|| format!("Catalog parent {parent} is missing"))?
        .children
        .clone();
    let mut staged = current.clone();
    for child in old_children {
        staged.remove(&child).map_err(|error| error.to_string())?;
    }
    let count = append_nodes(&mut staged, parent, nodes)?;
    let target = staged
        .items
        .get_mut(parent)
        .expect("validated parent retained by subtree removal");
    target.loading = false;
    target.has_children = !target.children.is_empty();
    staged.validate().map_err(|error| error.to_string())?;
    Ok((staged, count))
}

fn append_nodes(
    model: &mut TreeModel<String>,
    parent: &str,
    nodes: Vec<CatalogNode>,
) -> Result<usize, String> {
    let mut count = 0;
    for node in nodes {
        if node.id.trim().is_empty() || node.label.trim().is_empty() {
            return Err("Catalog entries need a nonblank id and label".into());
        }
        let mut item = TreeModelItem::new(node.id.clone(), node.label);
        item.disabled = node.disabled;
        model
            .append(parent, item)
            .map_err(|error| error.to_string())?;
        count += 1 + append_nodes(model, &node.id, node.children)?;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn current() -> TreeModel<String> {
        let mut remote = TreeModelItem::branch(
            "model-remote",
            "Renamed remote branch".into(),
            vec!["old".into()],
        );
        remote.disabled = true;
        remote.loading = true;
        TreeModel::new(
            "root",
            [
                TreeModelItem::branch(
                    "root",
                    "Inventory".into(),
                    vec!["model-remote".into(), "local".into()],
                ),
                remote,
                TreeModelItem::branch("old", "Old branch".into(), vec!["old-child".into()]),
                TreeModelItem::new("old-child", "Old drive".into()),
                TreeModelItem::new("local", "Locally edited motor".into()),
                TreeModelItem::new("staged", "Detached pending asset".into()),
            ],
        )
        .unwrap()
    }
    #[test]
    fn nested_catalog_replaces_only_former_descendants_and_retains_host_edits() {
        let original = current();
        let before = original.clone();
        let (next, count) = replace_remote_catalog(
            &original,
            "model-remote",
            r#"[
            {"id":"new-branch","label":"Remote group","disabled":true,"children":[
                {"id":"new-drive","label":"Drive α"}
            ]},
            {"id":"old-child","label":"Reused identifier","disabled":true}
        ]"#,
        )
        .unwrap();
        assert_eq!(count, 3);
        assert_eq!(original, before);
        assert!(!next.items.contains_key("old"));
        assert_eq!(
            next.items["model-remote"].children,
            ["new-branch", "old-child"]
        );
        assert_eq!(next.items["new-branch"].children, ["new-drive"]);
        assert!(next.items["new-branch"].disabled);
        assert!(next.items["old-child"].disabled);
        assert!(!next.items["new-drive"].disabled);
        assert_eq!(next.items["new-drive"].data, "Drive α");
        assert_eq!(
            next.items["model-remote"].data,
            before.items["model-remote"].data
        );
        assert!(next.items["model-remote"].disabled);
        assert!(!next.items["model-remote"].loading);
        assert!(next.items["model-remote"].has_children);
        for id in ["root", "local", "staged"] {
            assert_eq!(next.items[id], before.items[id]);
        }
        assert!(next.validate().is_ok());
    }
    #[test]
    fn malformed_duplicate_and_outside_collision_failures_leave_the_original_untouched() {
        let original = current();
        let before = original.clone();
        for json in [
            "not JSON",
            "{}",
            r#"[{"id":"x"}]"#,
            r#"[{"id":" ","label":"Valid"}]"#,
            r#"[{"id":"x","label":"  "}]"#,
            r#"[{"id":"x","label":"One","children":[{"id":"x","label":"Duplicate"}]}]"#,
            r#"[{"id":"x","label":"First"},{"id":"x","label":"Second"}]"#,
            r#"[{"id":"new","label":"Staged first"},{"id":"local","label":"Outside collision"}]"#,
            r#"[{"id":"staged","label":"Detached collision"}]"#,
            r#"[{"id":"model-remote","label":"Parent collision"}]"#,
        ] {
            assert!(
                replace_remote_catalog(&original, "model-remote", json).is_err(),
                "{json}"
            );
            assert_eq!(original, before);
        }
        assert!(replace_remote_catalog(&original, "missing", DEFAULT_REMOTE_CATALOG).is_err());
        assert_eq!(original, before);
    }
    #[test]
    fn empty_catalog_is_a_known_leaf_and_a_later_reload_reuses_default_identifiers() {
        let original = current();
        let (empty, count) = replace_remote_catalog(&original, "model-remote", "[]").unwrap();
        assert_eq!(count, 0);
        assert!(empty.items["model-remote"].children.is_empty());
        assert!(!empty.items["model-remote"].has_children);
        assert!(!empty.items["model-remote"].loading);
        assert!(!empty.items.contains_key("old-child"));
        let (loaded, count) =
            replace_remote_catalog(&empty, "model-remote", DEFAULT_REMOTE_CATALOG).unwrap();
        assert_eq!(count, 2);
        assert!(loaded.items["model-remote"].has_children);
        assert_eq!(
            loaded.items["model-remote"].children,
            ["model-remote-1", "model-remote-2"]
        );
        assert_eq!(loaded.items["model-remote-1"].data, "Remote drive 001");
        assert_eq!(loaded.items["model-remote-2"].data, "Remote drive 002");
        let (reloaded, count) =
            replace_remote_catalog(&loaded, "model-remote", DEFAULT_REMOTE_CATALOG).unwrap();
        assert_eq!(count, 2);
        assert_eq!(loaded, reloaded);
    }
}
