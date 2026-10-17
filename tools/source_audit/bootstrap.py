"""Install the source-audit runtime outside the checkout (explicit network step)."""

import argparse
import os
from pathlib import Path
import shlex
import subprocess
import sys
import venv


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--env", type=Path, default=Path.home() / ".local/share/codex-py")
    parser.add_argument("--wrapper", type=Path, default=Path.home() / ".local/bin/apy")
    args = parser.parse_args()
    environment = args.env.expanduser().absolute()
    wrapper = args.wrapper.expanduser().absolute()
    marker = "# Dioxus_iX source-audit toolbox wrapper"
    if wrapper.exists() and marker not in wrapper.read_text():
        parser.error(f"refusing to replace an unmanaged wrapper: {wrapper}")
    if environment.exists() and not (environment / "pyvenv.cfg").is_file():
        parser.error(f"refusing to use a non-venv directory: {environment}")
    if not environment.exists():
        venv.EnvBuilder(with_pip=True).create(environment)
    python = environment / "bin/python"
    subprocess.run(
        [str(python), "-B", "-m", "pip", "install", "--disable-pip-version-check",
         "--only-binary=:all:", "-r", str(Path(__file__).with_name("requirements.lock"))],
        check=True,
        env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1", "PIP_NO_INPUT": "1"},
    )
    # Exercise all three binary wheels before publishing the convenience command.
    subprocess.run(
        [str(python), "-B", "-c",
         "from tree_sitter import Language, Parser; "
         "import tree_sitter_rust as r, tree_sitter_typescript as t; "
         "assert not Parser(Language(r.language())).parse(b'fn main() {}').root_node.has_error; "
         "assert not Parser(Language(t.language_typescript())).parse(b'const n: number = 1;').root_node.has_error; "
         "assert not Parser(Language(t.language_tsx())).parse(b'const n = <div/>;').root_node.has_error"],
        check=True,
    )
    wrapper.parent.mkdir(parents=True, exist_ok=True)
    wrapper.write_text(
        f"#!/bin/sh\n{marker}\nexport PYTHONDONTWRITEBYTECODE=1\n"
        f"export PYTHONUNBUFFERED=1\nexec {shlex.quote(str(python))} -B -u \"$@\"\n"
    )
    wrapper.chmod(0o755)
    print(f"Toolbox: {python}\nWrapper: {wrapper}")
    print("Run just inventory-rust; runtime commands do not install or download packages.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
