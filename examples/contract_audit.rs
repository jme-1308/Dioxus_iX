//! Inspect the pinned upstream API without treating a mapping as verified parity.
//!
//! Run `cargo run --example contract_audit -- ../ix [--component ix-input] [--json]`.
//! Output goes only to stdout. Exit 2 means selected source components are missing
//! generated metadata; exit 1 means invalid inputs or a stale upstream checkout.

use dioxus_ix::{PORT_CATALOG, UPSTREAM_REVISION};
use regex::Regex;
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

type AuditResult<T> = Result<T, String>;

#[derive(Default)]
struct Options {
    root: PathBuf,
    component: Option<String>,
    json: bool,
}

#[derive(Clone, Debug, Serialize)]
struct Declaration {
    path: String,
    deprecated: bool,
    internal: bool,
}

#[derive(Debug, Serialize)]
struct Member {
    kind: &'static str,
    name: String,
    review_status: &'static str,
    /// Original generated metadata, including type/default/event flags.
    source_contract: Value,
}

#[derive(Debug, Serialize)]
struct Component {
    tag: String,
    declaration: Declaration,
    rust_api: Option<&'static str>,
    rust_module: Option<&'static str>,
    mapping_kind: &'static str,
    metadata_status: &'static str,
    review_status: &'static str,
    members: Vec<Member>,
    excluded_deprecated_members: Vec<Member>,
}

#[derive(Default, Debug, Serialize)]
struct Counts {
    properties: usize,
    methods: usize,
    events: usize,
    slots: usize,
    css_properties: usize,
    css_parts: usize,
    css_states: usize,
}

impl Counts {
    fn add(&mut self, member: &Member) {
        match member.kind {
            "property" => self.properties += 1,
            "method" => self.methods += 1,
            "event" => self.events += 1,
            "slot" => self.slots += 1,
            "css_property" => self.css_properties += 1,
            "css_part" => self.css_parts += 1,
            "css_state" => self.css_states += 1,
            _ => unreachable!("validated member kind"),
        }
    }
}

#[derive(Debug, Serialize)]
struct Report {
    upstream_revision: &'static str,
    upstream_inputs_clean: bool,
    metadata_source: &'static str,
    source_component_count: usize,
    supported_component_count: usize,
    generated_metadata_component_count: usize,
    excluded_deprecated_components: Vec<String>,
    excluded_deprecated_component_members: Counts,
    excluded_deprecated_members: Counts,
    current_members: Counts,
    selected_component_count: usize,
    complete_metadata_for_selected_components: bool,
    complete_metadata_for_all_supported_components: bool,
    missing_generated_metadata: Vec<String>,
    limitations: Vec<&'static str>,
    components: Vec<Component>,
}

fn options(args: impl IntoIterator<Item = String>) -> AuditResult<Options> {
    let mut options = Options {
        root: PathBuf::from("../ix"),
        ..Options::default()
    };
    let mut root_seen = false;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" if !options.json => options.json = true,
            "--component" if options.component.is_none() => {
                let tag = args.next().ok_or("--component requires an ix-* tag")?;
                if !tag.starts_with("ix-") || tag.starts_with("--") {
                    return Err("--component requires an ix-* tag".into());
                }
                options.component = Some(tag);
            }
            value if !value.starts_with('-') && !root_seen => {
                options.root = PathBuf::from(value);
                root_seen = true;
            }
            _ => return Err(format!("unknown or duplicate argument: {arg}")),
        }
    }
    Ok(options)
}

fn read(path: &Path) -> AuditResult<String> {
    fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn git(root: &Path, args: &[&str]) -> AuditResult<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|error| format!("cannot inspect upstream Git checkout: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "upstream Git check failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout).map_err(|error| format!("Git output is not UTF-8: {error}"))
}

fn verify_checkout(root: &Path) -> AuditResult<()> {
    let revision = git(root, &["rev-parse", "HEAD"])?;
    if revision.trim() != UPSTREAM_REVISION {
        return Err(format!(
            "expected upstream revision {UPSTREAM_REVISION}, found {}",
            revision.trim()
        ));
    }
    // These are read-only Git commands. Reject edited, staged and untracked
    // inputs: an unchanged HEAD alone does not prove metadata freshness.
    let paths = [
        "packages/core/component-api.json",
        "packages/core/src/components",
        "packages/core/src/components.d.ts",
    ];
    let mut status_args = vec!["status", "--porcelain", "--untracked-files=all", "--"];
    status_args.extend(paths);
    let status = git(root, &status_args)?;
    if !status.trim().is_empty() {
        return Err(format!(
            "upstream audit inputs have local changes:\n{status}"
        ));
    }
    // Require metadata to belong to this revision rather than an ignored build
    // artifact that happens to be present next to the source.
    git(
        root,
        &[
            "ls-files",
            "--error-unmatch",
            "packages/core/component-api.json",
        ],
    )?;
    Ok(())
}

fn source_files(path: &Path, files: &mut Vec<PathBuf>) -> AuditResult<()> {
    for entry in fs::read_dir(path).map_err(|error| format!("{}: {error}", path.display()))? {
        let entry = entry.map_err(|error| error.to_string())?;
        let kind = entry.file_type().map_err(|error| error.to_string())?;
        if kind.is_symlink() {
            return Err(format!(
                "unexpected symlink in upstream sources: {}",
                entry.path().display()
            ));
        }
        if kind.is_dir() {
            source_files(&entry.path(), files)?;
        } else if entry.path().extension().is_some_and(|value| value == "tsx") {
            files.push(entry.path());
        }
    }
    Ok(())
}

fn parse_declarations(source: &str, path: &str) -> AuditResult<Vec<(String, Declaration)>> {
    let component = Regex::new(r"(?s)@Component\s*\(\s*\{(.*?)\}\s*\)").unwrap();
    let tag = Regex::new(r#"\btag\s*:\s*['"](ix-[a-z0-9-]+)['"]"#).unwrap();
    let marker = Regex::new(r"@Component\s*\(").unwrap();
    let mut declarations = Vec::new();
    for capture in component.captures_iter(source) {
        let full = capture.get(0).unwrap();
        let config = capture.get(1).unwrap().as_str();
        let tags: Vec<_> = tag.captures_iter(config).collect();
        if tags.len() != 1 {
            return Err(format!(
                "{path}: component must have exactly one literal ix-* tag"
            ));
        }
        let prefix = &source[..full.start()];
        let doc = prefix
            .rfind("/**")
            .and_then(|start| {
                let candidate = &prefix[start..];
                let end = candidate.find("*/")?;
                candidate[end + 2..]
                    .trim()
                    .is_empty()
                    .then_some(&candidate[..end])
            })
            .unwrap_or("");
        declarations.push((
            tags[0][1].to_owned(),
            Declaration {
                path: path.to_owned(),
                deprecated: doc.contains("@deprecated"),
                internal: doc.contains("@internal"),
            },
        ));
    }
    if declarations.len() != marker.find_iter(source).count() {
        return Err(format!("{path}: unsupported @Component declaration syntax"));
    }
    Ok(declarations)
}

fn declarations(root: &Path) -> AuditResult<BTreeMap<String, Declaration>> {
    let mut files = Vec::new();
    source_files(&root.join("packages/core/src/components"), &mut files)?;
    files.sort();
    let mut result = BTreeMap::new();
    for file in files {
        let relative = file
            .strip_prefix(root)
            .map_err(|error| error.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        for (tag, declaration) in parse_declarations(&read(&file)?, &relative)? {
            if result.insert(tag.clone(), declaration).is_some() {
                return Err(format!("duplicate source component tag: {tag}"));
            }
        }
    }
    if result.is_empty() {
        return Err("no upstream @Component declarations found".into());
    }
    Ok(result)
}

fn object<'a>(value: &'a Value, context: &str) -> AuditResult<&'a Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| format!("{context}: expected object"))
}

fn string<'a>(value: &'a Value, context: &str) -> AuditResult<&'a str> {
    value
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("{context}: expected nonempty string"))
}

fn boolean(object: &Map<String, Value>, key: &str, context: &str) -> AuditResult<bool> {
    object
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("{context}.{key}: expected boolean"))
}

fn exact_keys(object: &Map<String, Value>, allowed: &[&str], context: &str) -> AuditResult<()> {
    if let Some(key) = object.keys().find(|key| !allowed.contains(&key.as_str())) {
        return Err(format!(
            "{context}: unknown metadata field {key}; update audit schema first"
        ));
    }
    Ok(())
}

fn array<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    context: &str,
) -> AuditResult<&'a Vec<Value>> {
    object
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{context}.{key}: expected array (including when empty)"))
}

const MEMBER_ARRAYS: [(&str, &str); 7] = [
    ("properties", "property"),
    ("methods", "method"),
    ("events", "event"),
    ("slots", "slot"),
    ("cssProperties", "css_property"),
    ("cssParts", "css_part"),
    ("cssStates", "css_state"),
];

struct ParsedComponent {
    deprecated: bool,
    members: Vec<Member>,
    excluded: Vec<Member>,
}

fn parse_metadata(json: &str) -> AuditResult<BTreeMap<String, ParsedComponent>> {
    let value: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let root = object(&value, "root")?;
    exact_keys(root, &["components"], "root")?;
    let mut result = BTreeMap::new();
    let tag_pattern = Regex::new(r"^ix-[a-z0-9-]+$").unwrap();
    for component in array(root, "components", "root")? {
        let object = object(component, "component")?;
        let tag = string(
            object.get("tag").ok_or("component.tag is missing")?,
            "component.tag",
        )?;
        if !tag_pattern.is_match(tag) {
            return Err(format!("invalid component tag: {tag}"));
        }
        exact_keys(
            object,
            &[
                "tag",
                "deprecated",
                "properties",
                "methods",
                "events",
                "slots",
                "cssProperties",
                "cssParts",
                "cssStates",
            ],
            tag,
        )?;
        let deprecated = boolean(object, "deprecated", tag)?;
        let mut members = Vec::new();
        let mut excluded = Vec::new();
        for (key, kind) in MEMBER_ARRAYS {
            let mut seen = BTreeSet::new();
            for value in array(object, key, tag)? {
                let context = format!("{tag}.{key}");
                let (name, is_deprecated) = if matches!(kind, "property" | "method" | "event") {
                    let member = self::object(value, &context)?;
                    let name = string(
                        member
                            .get("name")
                            .ok_or_else(|| format!("{context}: missing name"))?,
                        &context,
                    )?;
                    let is_deprecated = boolean(member, "deprecated", &context)?;
                    match kind {
                        "property" => {
                            exact_keys(
                                member,
                                &[
                                    "name",
                                    "attribute",
                                    "type",
                                    "mutable",
                                    "reflectToAttribute",
                                    "default",
                                    "optional",
                                    "required",
                                    "deprecated",
                                ],
                                &context,
                            )?;
                            string(
                                member
                                    .get("type")
                                    .ok_or_else(|| format!("{context}: missing type"))?,
                                &context,
                            )?;
                            for flag in ["mutable", "reflectToAttribute", "optional", "required"] {
                                boolean(member, flag, &context)?;
                            }
                            for field in ["attribute", "default"] {
                                if let Some(value) = member.get(field) {
                                    string(value, &format!("{context}.{field}"))?;
                                }
                            }
                        }
                        "method" => {
                            exact_keys(member, &["name", "signature", "deprecated"], &context)?;
                            string(
                                member
                                    .get("signature")
                                    .ok_or_else(|| format!("{context}: missing signature"))?,
                                &context,
                            )?;
                        }
                        "event" => {
                            exact_keys(
                                member,
                                &[
                                    "name",
                                    "detail",
                                    "bubbles",
                                    "cancelable",
                                    "composed",
                                    "deprecated",
                                ],
                                &context,
                            )?;
                            string(
                                member
                                    .get("detail")
                                    .ok_or_else(|| format!("{context}: missing detail"))?,
                                &context,
                            )?;
                            for flag in ["bubbles", "cancelable", "composed"] {
                                boolean(member, flag, &context)?;
                            }
                        }
                        _ => unreachable!(),
                    }
                    (name.to_owned(), is_deprecated)
                } else {
                    (string(value, &context)?.to_owned(), false)
                };
                if !seen.insert(name.clone()) {
                    return Err(format!("{context}: duplicate member {name}"));
                }
                let member = Member {
                    kind,
                    name,
                    review_status: "unreviewed",
                    source_contract: value.clone(),
                };
                if is_deprecated {
                    excluded.push(member);
                } else {
                    members.push(member);
                }
            }
        }
        if result
            .insert(
                tag.to_owned(),
                ParsedComponent {
                    deprecated,
                    members,
                    excluded,
                },
            )
            .is_some()
        {
            return Err(format!("duplicate metadata component: {tag}"));
        }
    }
    if result.is_empty() {
        return Err("metadata contains no components".into());
    }
    Ok(result)
}

fn build_report(
    declarations: BTreeMap<String, Declaration>,
    mut metadata: BTreeMap<String, ParsedComponent>,
    filter: Option<&str>,
) -> AuditResult<Report> {
    let mut catalog = BTreeMap::new();
    for entry in PORT_CATALOG {
        let tag = entry
            .upstream_tag()
            .ok_or("catalog contains a nontag entry")?;
        if catalog.insert(tag.clone(), entry).is_some() {
            return Err(format!("duplicate catalog tag: {tag}"));
        }
    }
    let source_supported: BTreeSet<_> = declarations
        .iter()
        .filter(|(_, value)| !value.deprecated)
        .map(|(tag, _)| tag.clone())
        .collect();
    let catalog_tags = catalog.keys().cloned().collect::<BTreeSet<_>>();
    if source_supported != catalog_tags {
        return Err(format!(
            "catalog/source mismatch; missing mappings: {:?}; unexpected or deprecated mappings: {:?}",
            source_supported
                .difference(&catalog_tags)
                .collect::<Vec<_>>(),
            catalog_tags
                .difference(&source_supported)
                .collect::<Vec<_>>()
        ));
    }
    for (tag, component) in &metadata {
        let declaration = declarations
            .get(tag)
            .ok_or_else(|| format!("metadata tag has no source declaration: {tag}"))?;
        if declaration.deprecated != component.deprecated {
            return Err(format!(
                "source/metadata component deprecation mismatch: {tag}"
            ));
        }
    }
    if let Some(tag) = filter {
        if !source_supported.contains(tag) {
            return Err(format!("unknown or excluded deprecated component: {tag}"));
        }
    }
    let mut report = Report {
        upstream_revision: UPSTREAM_REVISION,
        upstream_inputs_clean: true,
        metadata_source: "packages/core/component-api.json",
        source_component_count: declarations.len(),
        supported_component_count: source_supported.len(),
        generated_metadata_component_count: metadata.len(),
        excluded_deprecated_components: Vec::new(),
        excluded_deprecated_component_members: Counts::default(),
        excluded_deprecated_members: Counts::default(),
        current_members: Counts::default(),
        selected_component_count: 0,
        complete_metadata_for_selected_components: true,
        complete_metadata_for_all_supported_components: true,
        missing_generated_metadata: Vec::new(),
        limitations: vec![
            "Every member starts unreviewed: component mappings and passing tests do not prove member parity.",
            "This inventories generated API declarations, not all behavior, visual, accessibility or platform requirements.",
            "Source declarations missing generated metadata are explicit gaps; their member lists are unknown, never assumed empty.",
            "Types and signatures are upstream TypeScript contracts; Rust mapping is component-level, not a per-member equivalence claim.",
            "Slots and CSS surfaces are limited to entries declared in generated metadata; an empty array does not prove no runtime extension points.",
        ],
        components: Vec::new(),
    };
    for (tag, declaration) in declarations {
        let component = metadata.remove(&tag);
        if declaration.deprecated {
            report.excluded_deprecated_components.push(tag);
            if let Some(component) = component {
                for member in component.members.iter().chain(&component.excluded) {
                    report.excluded_deprecated_component_members.add(member);
                }
            }
            continue;
        }
        let available = component.is_some();
        if !available {
            report.missing_generated_metadata.push(tag.clone());
            report.complete_metadata_for_all_supported_components = false;
        }
        let (members, excluded) = component
            .map(|component| (component.members, component.excluded))
            .unwrap_or_default();
        for member in &members {
            report.current_members.add(member);
        }
        for member in &excluded {
            report.excluded_deprecated_members.add(member);
        }
        if filter.is_some_and(|filter| filter != tag) {
            continue;
        }
        if !available {
            report.complete_metadata_for_selected_components = false;
        }
        let mapping = catalog
            .get(&tag)
            .expect("catalog source equality was checked");
        report.components.push(Component {
            tag,
            declaration,
            rust_api: mapping.rust_api,
            rust_module: mapping.module,
            mapping_kind: mapping.mapping_kind().label(),
            metadata_status: if available { "available" } else { "missing" },
            review_status: "unreviewed",
            members,
            excluded_deprecated_members: excluded,
        });
    }
    report.selected_component_count = report.components.len();
    Ok(report)
}

fn text_report(report: &Report) -> String {
    use std::fmt::Write;
    let mut text = String::new();
    writeln!(
        text,
        "Pinned upstream: {} (audit inputs clean)",
        report.upstream_revision
    )
    .unwrap();
    writeln!(
        text,
        "Source tags: {}; current: {}; excluded deprecated: {}; generated metadata tags: {}",
        report.source_component_count,
        report.supported_component_count,
        report.excluded_deprecated_components.len(),
        report.generated_metadata_component_count
    )
    .unwrap();
    writeln!(text, "Known current contracts across all supported tags: {} properties, {} methods, {} events, {} slots, {} CSS properties, {} CSS parts, {} CSS states",
        report.current_members.properties, report.current_members.methods, report.current_members.events,
        report.current_members.slots, report.current_members.css_properties,
        report.current_members.css_parts, report.current_members.css_states).unwrap();
    writeln!(
        text,
        "Excluded current-component members: {} properties, {} methods, {} events",
        report.excluded_deprecated_members.properties,
        report.excluded_deprecated_members.methods,
        report.excluded_deprecated_members.events
    )
    .unwrap();
    writeln!(
        text,
        "Excluded deprecated tags: {}",
        report.excluded_deprecated_components.join(", ")
    )
    .unwrap();
    writeln!(
        text,
        "Complete selected metadata: {}; complete all-current metadata: {}",
        report.complete_metadata_for_selected_components,
        report.complete_metadata_for_all_supported_components
    )
    .unwrap();
    if !report.missing_generated_metadata.is_empty() {
        writeln!(
            text,
            "MISSING GENERATED METADATA: {}",
            report.missing_generated_metadata.join(", ")
        )
        .unwrap();
    }
    for component in &report.components {
        writeln!(
            text,
            "\n{} -> {} [{}; metadata {}; unreviewed] ({})",
            component.tag,
            component.rust_api.unwrap_or("NO RUST MAPPING"),
            component.mapping_kind,
            component.metadata_status,
            component.declaration.path
        )
        .unwrap();
        if component.metadata_status == "missing" {
            writeln!(
                text,
                "  UNKNOWN member list: source component is absent from generated metadata."
            )
            .unwrap();
        }
        for member in &component.members {
            writeln!(
                text,
                "  {} {} [unreviewed]: {}",
                member.kind, member.name, member.source_contract
            )
            .unwrap();
        }
        for member in &component.excluded_deprecated_members {
            writeln!(
                text,
                "  EXCLUDED deprecated {} {}: {}",
                member.kind, member.name, member.source_contract
            )
            .unwrap();
        }
    }
    writeln!(text, "\nLimits:").unwrap();
    for limit in &report.limitations {
        writeln!(text, "- {limit}").unwrap();
    }
    text
}

fn run(options: Options) -> AuditResult<(Report, bool)> {
    verify_checkout(&options.root)?;
    let source = declarations(&options.root)?;
    let metadata = parse_metadata(&read(
        &options.root.join("packages/core/component-api.json"),
    )?)?;
    let report = build_report(source, metadata, options.component.as_deref())?;
    Ok((report, options.json))
}

fn main() -> ExitCode {
    match options(env::args().skip(1)).and_then(run) {
        Ok((report, json)) => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report)
                        .expect("report contains serializable values")
                );
            } else {
                print!("{}", text_report(&report));
            }
            if report.complete_metadata_for_selected_components {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(2)
            }
        }
        Err(error) => {
            eprintln!("contract audit failed: {error}");
            eprintln!(
                "Usage: cargo run --example contract_audit -- [upstream-root] [--component ix-input] [--json]"
            );
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> Value {
        json!({"components": [{"tag": "ix-example", "deprecated": false,
            "properties": [{"name": "value", "type": "string", "default": "''", "mutable": true,
                "reflectToAttribute": false, "optional": false, "required": false, "deprecated": false},
                {"name": "old", "type": "number", "mutable": false, "reflectToAttribute": false,
                    "optional": true, "required": false, "deprecated": true}],
            "methods": [{"name": "focus", "signature": "focus() => Promise<void>", "deprecated": false}],
            "events": [{"name": "change", "detail": "string", "bubbles": true,
                "cancelable": true, "composed": false, "deprecated": false}],
            "slots": ["default", "end"], "cssProperties": [], "cssParts": ["content"], "cssStates": []}]})
    }

    #[test]
    fn preserves_contracts_and_separates_deprecated_members_without_granting_parity() {
        let mut parsed = parse_metadata(&fixture().to_string()).unwrap();
        let component = parsed.remove("ix-example").unwrap();
        assert_eq!(component.members.len(), 6);
        assert_eq!(component.excluded.len(), 1);
        assert_eq!(component.excluded[0].name, "old");
        assert!(
            component
                .members
                .iter()
                .all(|member| member.review_status == "unreviewed")
        );
        assert_eq!(component.members[0].source_contract["default"], "''");
        assert_eq!(component.members[2].source_contract["cancelable"], true);
    }

    #[test]
    fn rejects_missing_arrays_flags_signatures_types_duplicates_and_unknown_schema() {
        let mutations: Vec<fn(&mut Value)> = vec![
            |v| {
                v["components"][0].as_object_mut().unwrap().remove("slots");
            },
            |v| {
                v["components"][0]["events"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("cancelable");
            },
            |v| {
                v["components"][0]["methods"][0]["signature"] = Value::Null;
            },
            |v| {
                v["components"][0]["properties"][0]["type"] = json!(42);
            },
            |v| {
                v["components"][0]["properties"][0]["deprecated"] = json!("false");
            },
            |v| {
                v["components"][0]["slots"] = json!(["end", "end"]);
            },
            |v| {
                v["components"][0]["newContractKind"] = json!([]);
            },
            |v| {
                v["components"] = json!([v["components"][0].clone(), v["components"][0].clone()]);
            },
        ];
        for mutate in mutations {
            let mut value = fixture();
            mutate(&mut value);
            assert!(
                parse_metadata(&value.to_string()).is_err(),
                "accepted malformed fixture: {value}"
            );
        }
    }

    #[test]
    fn component_deprecation_comes_from_its_own_doc_not_a_deprecated_property() {
        let source = r#"/** @internal */
            @Component({ tag: 'ix-current', shadow: true })
            export class Current { /** @deprecated */ @Prop() old: string; }
            /** @deprecated use ix-current */
            @Component({ tag: "ix-old" }) export class Old {}"#;
        let declarations = parse_declarations(source, "fixture.tsx").unwrap();
        assert_eq!(declarations.len(), 2);
        assert!(declarations[0].1.internal);
        assert!(!declarations[0].1.deprecated);
        assert!(declarations[1].1.deprecated);
        assert!(parse_declarations("@Component(makeOptions())", "fixture.tsx").is_err());
    }

    #[test]
    fn missing_metadata_is_a_visible_failure_and_deprecated_source_tags_are_excluded() {
        let mut source = BTreeMap::new();
        for entry in PORT_CATALOG {
            source.insert(
                entry.upstream_tag().unwrap(),
                Declaration {
                    path: "fixture.tsx".into(),
                    deprecated: false,
                    internal: false,
                },
            );
        }
        source.insert(
            "ix-retired".into(),
            Declaration {
                path: "retired.tsx".into(),
                deprecated: true,
                internal: false,
            },
        );
        let report = build_report(source, BTreeMap::new(), Some("ix-input")).unwrap();
        assert_eq!(report.excluded_deprecated_components, ["ix-retired"]);
        assert!(!report.complete_metadata_for_selected_components);
        assert_eq!(report.components[0].metadata_status, "missing");
        assert!(report.components[0].members.is_empty());
        assert!(text_report(&report).contains("UNKNOWN member list"));
    }

    #[test]
    fn rejects_unsupported_filters_and_ambiguous_cli_arguments() {
        assert!(options(["--component".into()]).is_err());
        assert!(options(["--json".into(), "--json".into()]).is_err());
        assert!(options(["one".into(), "two".into()]).is_err());
        assert!(options(["--unknown".into()]).is_err());
        assert_eq!(
            options(["--component".into(), "ix-input".into()])
                .unwrap()
                .root,
            PathBuf::from("../ix")
        );
    }
}
