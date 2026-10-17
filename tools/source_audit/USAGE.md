# Source audit tooling

This development-only tool inventories raw Rust `#[component]` functions. It
does not edit source, expand macros, resolve Rust types, evaluate `cfg`, or prove
parity with Siemens iX. The existing `examples/contract_audit.rs` and
`IMPLEMENTATION.md` retain their current roles.

## Setup

Run `just source-audit-setup` once with network access and Python 3.10 or newer.
It installs the exact versions in `requirements.lock` into
`~/.local/share/codex-py`, verifies Rust/TypeScript/TSX parsing, and creates
`~/.local/bin/apy`. It refuses to overwrite an unmanaged wrapper or a non-venv
directory. Existing unrelated toolbox packages are preserved. The bootstrap
currently targets POSIX environments (including this Linux workstation).

`apy` runs toolbox Python with bytecode writes disabled and unbuffered output.
The wrapper's directory must be on your PATH to invoke it by name. The just
recipes use the interpreter's path directly, so they work without PATH changes.
No global Codex settings, toolchain settings or project runtime dependencies
are modified.

To select another environment, run
`python3 -B tools/source_audit/bootstrap.py --env /absolute/path/to/venv --wrapper /absolute/path/to/apy`
and set `SOURCE_AUDIT_PYTHON` to that environment's Python when using just.

## Commands

```sh
just inventory-rust
just inventory-rust src/forms.rs --json
just inventory-rust --json --strict
just test-source-audit
```

No runtime command installs packages or makes network requests. Interpreter
package versions must match the lock file. The default input is the project's
`src/` directory. Explicit paths can be Rust files or directories; overlapping
inputs are deduplicated. Directory walks skip `.git`, `target`, `.venv`,
`node_modules`, and `__pycache__` and do not follow directory symlinks.

JSON schema version 1 records functions, names, parameter lists and parameter
types with half-open **UTF-8 byte offsets** and one-based source lines. It
preserves documentation, visibility, associated attributes, enclosing scopes
and attributes, parameter patterns/types, and raw `#[props(...)]` expressions
(including defaults). It includes private and test components. These are raw
source declarations, not a list of exported or enabled APIs.

Diagnostics identify syntax errors, explicit `Props` structs that are not yet
linked to functions, conditional `cfg_attr(..., component)` declarations, and
unsupported parameter/component forms. Exit 0 means parsing succeeded; exit 1
means input/runtime failure; exit 2 means syntax errors, or any diagnostics in
`--strict` mode. Always inspect diagnostics and limitations before consuming a
report. An empty inventory is valid for a Rust file with no components.

## Boundary

The first milestone is Rust component inventory. TypeScript and TSX grammars
are installed and smoke-tested, but upstream extraction/comparison and
codemods are not implemented. `rsx!` bodies remain unexpanded token trees.
Component aliases, macro-generated declarations, manually implemented Props
and exports require further analysis. Keep behavioral evidence in the parity
ledger; matching names and signatures cannot establish equivalence.
