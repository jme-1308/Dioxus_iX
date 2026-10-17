#!/usr/bin/env python3
"""Codex PostToolUse hook: record which Argon jobs belong to THIS Codex session.

The argon MCP server cannot identify its caller — Codex passes it no session id, and
every server in the app is a child of the same `codex` process, so neither env nor the
process tree distinguishes one thread from another. The Stop guard therefore has no way
to tell "my unread results" from "another thread's", and would block a turn over work it
has nothing to do with.

Hooks, unlike the server, do get `session_id`. This runs right after argon_spawn, reads
the job ids out of the tool's own output, and files them under this session. The Stop
guard then only enforces the ids it finds there.

Install:
    PostToolUse matcher "^mcp__argon__argon_spawn$"
"""
import json
import os
import re
import sys
import time
import traceback
from pathlib import Path

ERROR_LOG = Path.home() / ".cache" / "codex-hooks" / "errors.log"
STATE_DIR = Path(".codex") / "argon-runs"
JOB_ID = re.compile(r"\b(\d{8}-\d{6}-[0-9a-f]{6})\b")
KEEP_FOR = 24 * 3600  # a session's claims are irrelevant long after it ends


def log_error():
    try:
        ERROR_LOG.parent.mkdir(parents=True, exist_ok=True)
        with ERROR_LOG.open("a") as f:
            f.write(f"--- {time.strftime('%Y-%m-%d %H:%M:%S')} argon_claim\n{traceback.format_exc()}\n")
    except OSError:
        pass


def flatten(value, out):
    """Tool responses arrive as a string, or nested content blocks."""
    if isinstance(value, str):
        out.append(value)
    elif isinstance(value, dict):
        for v in value.values():
            flatten(v, out)
    elif isinstance(value, list):
        for v in value:
            flatten(v, out)


def main():
    try:
        event = json.load(sys.stdin)
    except ValueError:
        return 0
    if event.get("hook_event_name") != "PostToolUse":
        return 0

    session = re.sub(r"[^A-Za-z0-9_.-]", "_", event.get("session_id") or "default")
    chunks = []
    flatten(event.get("tool_response"), chunks)
    ids = sorted({m for c in chunks for m in JOB_ID.findall(c)})
    if not ids:
        return 0

    root = Path(event.get("cwd") or os.getcwd())
    path = root / STATE_DIR / f".session-{session}.json"
    try:
        old = json.loads(path.read_text())
        known = set(old.get("jobs") or [])
    except (OSError, ValueError):
        known = set()
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps({"jobs": sorted(known | set(ids)),
                                    "updated": time.time()}, indent=2))
    except OSError:
        pass

    # Opportunistically drop claim files from sessions that ended long ago.
    try:
        for f in path.parent.glob(".session-*.json"):
            try:
                if time.time() - json.loads(f.read_text()).get("updated", 0) > KEEP_FOR:
                    f.unlink(missing_ok=True)
            except (OSError, ValueError):
                pass
    except OSError:
        pass
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception:
        log_error()
        sys.exit(0)  # fail open
