"""Inventory #[component] Rust functions. Reads source only; no network or writes.

Exit 0: parsed successfully; 1: input/runtime failure; 2: syntax errors or
unsupported declarations with --strict. This is a syntax inventory, not parity.
"""

import argparse
from importlib.metadata import PackageNotFoundError, version
import json
import os
from pathlib import Path
import sys

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[2]
EXCLUDED = {".git", "target", ".venv", "node_modules", "__pycache__"}
LIMITATIONS = [
    "Raw declarations only: cfg is recorded, not evaluated; test/private components are included.",
    "Rust macros (including rsx!) are not expanded; generated components and prop uses are not inventoried.",
    "Explicit Props structs and cfg_attr components are reported as unresolved diagnostics.",
    "Types, defaults and attributes are source text, not resolved Rust semantics.",
    "This inventory does not establish API completeness or behavior/visual/platform parity.",
]


def runtime():
    packages = {}
    for line in Path(__file__).with_name("requirements.lock").read_text().splitlines():
        if not line or line.startswith("#"):
            continue
        name, expected = line.split("==")
        actual = version(name)
        if actual != expected:
            raise ValueError(f"{name}: expected {expected}, found {actual}; run just source-audit-setup")
        packages[name] = actual
    return packages


def source_files(paths):
    files = set()
    for path in paths:
        if path.is_file():
            if path.suffix != ".rs":
                raise ValueError(f"expected a Rust .rs file: {path}")
            files.add(path.resolve())
        elif path.is_dir():
            for directory, children, names in os.walk(path, followlinks=False):
                children[:] = sorted(c for c in children if c not in EXCLUDED)
                for name in sorted(names):
                    if name.endswith(".rs"):
                        files.add((Path(directory) / name).resolve())
        else:
            raise ValueError(f"input does not exist: {path}")
    if not files:
        raise ValueError("no Rust source files found")
    return sorted(files)


def display_path(path):
    try:
        return path.relative_to(ROOT).as_posix()
    except ValueError:
        return str(path)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="*", type=Path, default=[ROOT / "src"])
    parser.add_argument("--json", action="store_true", help="emit a deterministic JSON report")
    parser.add_argument("--strict", action="store_true", help="also fail on unsupported declarations")
    args = parser.parse_args(argv)
    try:
        packages = runtime()
        from rust_components import inventory_source
        files = source_files(args.paths)
        components, diagnostics = [], []
        for path in files:
            found, issues = inventory_source(path.read_bytes(), display_path(path))
            components.extend(found)
            diagnostics.extend(issues)
    except (OSError, UnicodeError, ValueError, ImportError, PackageNotFoundError) as error:
        print(f"source-audit: {error}", file=sys.stderr)
        print("Install dependencies with: just source-audit-setup", file=sys.stderr)
        return 1
    report = {
        "schema_version": 1,
        "kind": "rust_component_syntax_inventory",
        "packages": packages,
        "files": [display_path(path) for path in files],
        "component_count": len(components),
        "components": components,
        "diagnostics": diagnostics,
        "limitations": LIMITATIONS,
    }
    if args.json:
        print(json.dumps(report, indent=2, ensure_ascii=False))
    else:
        for component in components:
            qualified = "::".join(component["scope"] + [component["name"]])
            parameters = ", ".join(f'{p["pattern"]}: {p["type"]}' for p in component["parameters"])
            print(f'{component["path"]}:{component["name_range"]["start_line"]} '
                  f'{component["visibility"]} {qualified}({parameters})')
        print(f"\n{len(components)} component declarations in {len(files)} Rust files.")
        for issue in diagnostics:
            print(f'{issue["path"]}:{issue["start_line"]}: {issue["severity"]}: '
                  f'{issue["code"]}: {issue["message"]}', file=sys.stderr)
        for limitation in LIMITATIONS:
            print(f"Note: {limitation}", file=sys.stderr)
    return 2 if any(d["severity"] == "error" for d in diagnostics) or (args.strict and diagnostics) else 0


if __name__ == "__main__":
    sys.exit(main())
