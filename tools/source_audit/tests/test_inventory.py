import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

TOOLS = Path(__file__).resolve().parents[1]
ROOT = TOOLS.parents[1]
sys.dont_write_bytecode = True
sys.path.insert(0, str(TOOLS))
from rust_components import inventory_source


class RustInventoryTests(unittest.TestCase):
    def parse(self, source):
        return inventory_source(source.encode("utf-8"), "fixture.rs")

    def test_component_attributes_exclude_comments_strings_and_helpers(self):
        source = '''/// Docs mention fn Button(label: String) here.
#[component]
pub fn Button(
    label: String, // the text
    disabled: bool,
) -> Element {
    let s = "fn Fake(x: u8)";
    rsx! { button { "{label}" } }
}
#[component]
pub fn Badge(count: u32) -> Element { todo!() }
fn helper(x: u8) -> u8 { x }
'''
        components, issues = self.parse(source)
        self.assertEqual(issues, [])
        self.assertEqual([c["name"] for c in components], ["Button", "Badge"])
        self.assertEqual([p["declaration"] for p in components[0]["parameters"]],
                         ["label: String", "disabled: bool"])
        self.assertEqual(components[0]["name_range"]["start_line"], 3)
        self.assertEqual(components[0]["documentation"], [source.splitlines()[0]])

    def test_generic_types_defaults_and_intervening_attributes(self):
        components, issues = self.parse('''#[dioxus::component]
// Attributes need not be immediately adjacent.
#[allow(non_snake_case)]
pub(crate) fn Panel(
    #[props(default = String::from("a,b)"))]
    label: String,
    #[props(default)] /* between */
    callback: Option<fn(u8, (u8, u8)) -> Result<Vec<String>, Error>>,
    mut r#type: bool,
) -> Element { todo!() }
''')
        self.assertEqual(issues, [])
        component = components[0]
        self.assertEqual(component["visibility"], "pub(crate)")
        self.assertEqual(component["attributes"],
                         ["#[dioxus::component]", "#[allow(non_snake_case)]"])
        self.assertEqual(component["parameters"][0]["props_attributes"],
                         ['#[props(default = String::from("a,b)"))]'])
        self.assertEqual(component["parameters"][1]["type"],
                         "Option<fn(u8, (u8, u8)) -> Result<Vec<String>, Error>>")
        self.assertEqual(component["parameters"][1]["attributes"], ["#[props(default)]"])
        self.assertEqual(component["parameters"][2]["pattern"], "r#type")

    def test_unicode_byte_ranges_and_crlf(self):
        source = '// blåbær 🦀\r\n#[component]\r\npub fn Café(label: String) -> Element { todo!() }\r\n'
        components, issues = self.parse(source)
        self.assertEqual(issues, [])
        component = components[0]
        raw = source.encode()
        name = component["name_range"]
        self.assertEqual(raw[name["start_byte"]:name["end_byte"]].decode(), "Café")
        args = component["parameters_range"]
        self.assertEqual(raw[args["start_byte"]:args["end_byte"]], b"(label: String)")
        self.assertEqual(component["range"]["start_line"], 3)

    def test_nested_scopes_and_cfg_are_recorded_without_evaluation(self):
        components, issues = self.parse('''#![cfg(feature = "web")]
#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    #[component]
    fn Sample() -> Element { todo!() }
}
''')
        self.assertEqual(issues, [])
        self.assertEqual(components[0]["scope"], ["tests"])
        self.assertEqual(components[0]["enclosing_attributes"], [
            {"scope": [], "attributes": ['#![cfg(feature = "web")]']},
            {"scope": ["tests"], "attributes": ["#[cfg(test)]"]},
            {"scope": ["tests"], "attributes": ["#![allow(dead_code)]"]},
        ])

    def test_macro_bodies_are_not_treated_as_expanded_components(self):
        components, issues = self.parse('''macro_rules! generate {
            () => { #[component] fn Hidden() -> Element { todo!() } };
        }
        generate!();
        #[component]
        fn Visible() -> Element { rsx! { "#[component] fn Fake()" } }
        ''')
        self.assertEqual(issues, [])
        self.assertEqual([c["name"] for c in components], ["Visible"])

    def test_explicit_props_and_conditional_components_are_unresolved(self):
        components, issues = self.parse('''#[derive(Clone, Props)]
        struct ManualProps { value: u8 }
        #[cfg_attr(feature = "web", component)]
        fn Conditional(value: u8) -> Element { todo!() }
        ''')
        self.assertEqual(components, [])
        self.assertEqual([i["code"] for i in issues],
                         ["explicit_props_struct", "conditional_component"])

    def test_broken_rust_has_explicit_errors(self):
        _, issues = self.parse("#[component]\nfn Broken(value: Vec<) -> Element {")
        self.assertTrue(any(i["severity"] == "error" for i in issues))

    def test_accessor_regression_on_many_components(self):
        # A real-source run exposed a native Point.row crash after repeated calls.
        source = "\n".join(f"#[component] fn C{i}(x: u8) -> Element {{todo!()}}" for i in range(200))
        components, issues = self.parse(source)
        self.assertEqual(issues, [])
        self.assertEqual(len(components), 200)
        self.assertEqual(components[-1]["name_range"]["start_line"], 200)


class CommandTests(unittest.TestCase):
    def run_cli(self, *args):
        return subprocess.run([sys.executable, "-B", str(TOOLS / "cli.py"), *map(str, args)],
                              capture_output=True, text=True, cwd=ROOT, timeout=30)

    def test_real_repository_strict_inventory_and_exact_ranges(self):
        result = self.run_cli("--json", "--strict")
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(result.stdout)
        self.assertGreater(report["component_count"], 100)
        self.assertEqual(report["diagnostics"], [])
        button = next(c for c in report["components"]
                      if c["path"] == "src/actions.rs" and c["name"] == "Button")
        self.assertIn("class", [p["pattern"] for p in button["parameters"]])
        self.assertTrue(any(c["enclosing_attributes"] for c in report["components"]))
        for component in report["components"]:
            source = (ROOT / component["path"]).read_bytes()
            name = component["name_range"]
            self.assertEqual(source[name["start_byte"]:name["end_byte"]].decode(), component["name"])

    def test_deterministic_read_only_scan_deduplicates_inputs_and_skips_build_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "sample.rs"
            source.write_text("#[component] fn Sample() -> Element {todo!()}")
            (root / "target").mkdir()
            (root / "target" / "bad.rs").write_text("this should not be parsed")
            def snapshot():
                return {str(p): (hashlib.sha256(p.read_bytes()).hexdigest(), p.stat().st_mtime_ns)
                        for p in root.rglob("*") if p.is_file()}
            before = snapshot()
            first = self.run_cli(root, source, "--json", "--strict")
            second = self.run_cli(root, source, "--json", "--strict")
            self.assertEqual(first.returncode, 0, first.stderr)
            self.assertEqual(first.stdout, second.stdout)
            self.assertEqual(json.loads(first.stdout)["component_count"], 1)
            self.assertEqual(before, snapshot())
            self.assertFalse(list(root.rglob("__pycache__")))

    def test_errors_have_nonzero_status_and_no_success_report(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "sample.rs"
            missing = self.run_cli(path, "--json")
            self.assertEqual(missing.returncode, 1)
            self.assertEqual(missing.stdout, "")
            path.write_bytes(b"\xff")
            invalid = self.run_cli(path, "--json")
            self.assertEqual(invalid.returncode, 1)
            path.write_text("#[component] fn Broken(")
            syntax = self.run_cli(path, "--json")
            self.assertEqual(syntax.returncode, 2)
            self.assertTrue(json.loads(syntax.stdout)["diagnostics"])

    def test_strict_mode_rejects_unsupported_declarations(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "sample.rs"
            path.write_text("#[derive(Props)] struct Manual { value: u8 }")
            self.assertEqual(self.run_cli(path, "--json").returncode, 0)
            self.assertEqual(self.run_cli(path, "--json", "--strict").returncode, 2)

    def test_bootstrap_refuses_to_overwrite_unmanaged_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            wrapper = root / "apy"
            wrapper.write_text("existing user command")
            result = subprocess.run(
                [sys.executable, "-B", str(TOOLS / "bootstrap.py"),
                 "--env", str(root / "environment"), "--wrapper", str(wrapper)],
                capture_output=True, text=True, timeout=30,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(wrapper.read_text(), "existing user command")
            self.assertFalse((root / "environment").exists())


if __name__ == "__main__":
    unittest.main()
