#!/usr/bin/env python3
"""Codex Stop hook: don't let the agent finish while the Rust code fails to compile.

- Skips the check when no Rust files or Cargo manifests have uncommitted changes.
- On compile errors, exits 2 with the errors on stderr, so Codex continues
  with them as a new prompt.
- Gives up after MAX_RETRIES attempts in a row and hands control back to you.
"""
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

MAX_RETRIES = 3
MAX_LINES = 40


def git(args, cwd):
    try:
        out = subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True)
    except OSError:
        return ""
    return out.stdout.strip() if out.returncode == 0 else ""


def main():
    try:
        event = json.load(sys.stdin)
    except Exception:
        event = {}

    cwd = event.get("cwd") or os.getcwd()
    root = git(["rev-parse", "--show-toplevel"], cwd) or cwd

    if not git(["status", "--porcelain", "--", "*.rs", "Cargo.toml", "Cargo.lock"], root):
        return 0

    session = re.sub(r"[^A-Za-z0-9_.-]", "_", event.get("session_id") or "default")
    state_dir = os.path.join(tempfile.gettempdir(), "codex-rust-check")
    os.makedirs(state_dir, exist_ok=True)
    state_file = os.path.join(state_dir, session)

    if shutil.which("just"):
        cmd = ["just", "check", "--message-format=short"]
    else:
        cmd = ["cargo", "check", "--features", "web", "--all-targets", "--message-format=short"]

    proc = subprocess.run(cmd, cwd=root, capture_output=True, text=True)

    if proc.returncode == 0:
        if os.path.exists(state_file):
            os.remove(state_file)
        return 0

    try:
        with open(state_file) as f:
            attempts = int(f.read().strip() or 0) + 1
    except (OSError, ValueError):
        attempts = 1

    if attempts > MAX_RETRIES:
        os.remove(state_file)
        print(json.dumps({
            "systemMessage": f"Rust check still failing after {MAX_RETRIES} fix attempts; stopping."
        }))
        return 0

    with open(state_file, "w") as f:
        f.write(str(attempts))

    output = (proc.stdout + proc.stderr).splitlines()
    errors = [l for l in output if re.search(r"(^|: )error(\[E\d+\])?:", l)]
    errors = [l for l in dict.fromkeys(errors)
              if not any(s in l for s in ("could not compile", "aborting due to", "recipe `"))]
    body = "\n".join((errors or output)[:MAX_LINES])

    hint = ""
    if has_rust_analyzer_mcp(root):
        position = first_mcp_position(errors)
        exact = (f" First compiler location for MCP: file={position[0]!r}, "
                 f"line={position[1]}, character={position[2]}." if position else "")
        hint = ("Rust-analyzer: use this zero-based location; if empty, call symbols first."
                f"{exact} Don't reset the workspace or run MCP diagnostics; this hook "
                "re-checks on finish.\n")

    sys.stderr.write(
        f"`{' '.join(cmd)}` failed (attempt {attempts}/{MAX_RETRIES}). "
        f"Fix these errors before finishing:\n{body}\n{hint}"
    )
    return 2


def has_rust_analyzer_mcp(root):
    try:
        with open(os.path.join(root, ".codex", "config.toml")) as f:
            return "[mcp_servers.rust-analyzer]" in f.read()
    except OSError:
        return False


def first_mcp_position(lines):
    """Convert Cargo's first one-based short diagnostic location to MCP coordinates."""
    for line in lines:
        match = re.match(r"^(.*?):(\d+):(\d+):\s+error(?:\[E\d+\])?:", line)
        if match:
            return match.group(1), int(match.group(2)) - 1, int(match.group(3)) - 1
    return None


if __name__ == "__main__":
    sys.exit(main())
