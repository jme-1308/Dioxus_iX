#!/usr/bin/env python3
"""Codex PreToolUse hook: tell Codex the argon fleet's live state before it spawns.

The argon tool descriptions already explain how to drive the fleet, and they sit in
context permanently — repeating them here would just cost tokens twice. What they cannot
express is anything that changes: how many slots are free right now, what is already
running, and whether the last few subagents died on something a retry will not fix.

The argon MCP server writes that to .codex/argon-runs/state.json on every job
transition; this reads it and injects a few lines.

Fails open: any problem means no output, so a broken hook never blocks a tool call.

Install (matcher must cover the spawning tools only):
    PreToolUse matcher "^mcp__argon__(argon_spawn|ask_argon)$"
"""
import json
import os
import sys
import time
import traceback
from pathlib import Path

ERROR_LOG = Path.home() / ".cache" / "codex-hooks" / "errors.log"
MAX_CHARS = 700
STALE_AFTER = 1800  # state older than this is from a dead server; ignore it


def log_error():
    try:
        ERROR_LOG.parent.mkdir(parents=True, exist_ok=True)
        with ERROR_LOG.open("a") as f:
            f.write(f"--- {time.strftime('%Y-%m-%d %H:%M:%S')} argon_brief\n{traceback.format_exc()}\n")
    except OSError:
        pass


def merged_state(state_dir):
    """Union the state of every argon server still alive (one per Codex thread)."""
    merged = {"running": [], "queued": 0, "max_parallel": 0, "free_slots": 0,
              "recent_failures": [], "skip_perms": False}
    found = False
    for f in sorted(state_dir.glob("state.*.json")):
        try:
            pid = int(f.name.split(".")[1])
            os.kill(pid, 0)
            s = json.loads(f.read_text())
        except (IndexError, ValueError, ProcessLookupError, OSError):
            continue
        if time.time() - s.get("updated", 0) > STALE_AFTER:
            continue
        found = True
        merged["running"] += s.get("running") or []
        merged["queued"] += s.get("queued", 0)
        merged["max_parallel"] = max(merged["max_parallel"], s.get("max_parallel", 0))
        merged["free_slots"] = max(merged["free_slots"], s.get("free_slots", 0))
        merged["recent_failures"] += s.get("recent_failures") or []
        merged["skip_perms"] = merged["skip_perms"] or s.get("skip_perms", False)
    if not found:
        return None
    # Slots are per server, so the honest figure is the cap minus everything in flight.
    merged["free_slots"] = max(0, merged["max_parallel"] - len(merged["running"]) - merged["queued"])
    return merged


def config_defaults(root):
    """Read the argon server's own env block out of .codex/config.toml.

    Hooks run as separate processes and do not inherit mcp_servers.argon.env, so this is
    the only honest source for the limits before the first state.json exists.
    """
    env = {}
    try:
        import tomllib
        with open(root / ".codex" / "config.toml", "rb") as f:
            env = (tomllib.load(f).get("mcp_servers", {})
                   .get("argon", {}).get("env", {})) or {}
    except Exception:
        pass
    return {
        "max_parallel": int(env.get("ARGON_MAX_PARALLEL", 4)),
        "skip_perms": env.get("ARGON_DANGEROUSLY_SKIP_PERMISSIONS") == "1",
    }


def build(state, requested):
    cap = state.get("max_parallel", 4)
    free = state.get("free_slots", cap)
    running = state.get("running") or []
    queued = state.get("queued", 0)
    lines = []

    cur = f"Argon fleet: {cap} subagents run at once, {free} slot(s) free"
    if running:
        cur += f"; running now: {', '.join(running[:6])}"
    if queued:
        cur += f"; {queued} already queued"
    lines.append(cur + ".")

    if requested and requested > free:
        lines.append(
            f"You asked for {requested}; {max(0, requested - free)} will queue behind the "
            f"running ones rather than start immediately. That is fine — spawning is "
            f"non-blocking — but expect the later ones to finish later.")
    elif requested and requested == 1 and free > 1:
        lines.append(
            f"You are spawning 1 subagent with {free} slots free. If this task splits into "
            f"independent parts, spawn them together: parallel subagents cost the "
            f"wall-clock of the slowest, not the sum.")

    # State the contract up front. A Stop hook that only appears at the end ambushes
    # Codex after it has already written its summary; said here, it can plan for it.
    lines.append(
        "Spawning creates an obligation: call argon_collect for these jobs before you "
        "finish this turn. A Stop hook blocks the turn while results are unread.")

    fails = state.get("recent_failures") or []
    hard = [f for f in fails if not f.get("retryable")]
    if hard:
        f = hard[-1]
        lines.append(
            f"WARNING: a recent subagent ({f.get('name')}) failed in a way a retry will "
            f"not fix — {f.get('why', '')[:160]} Fix that before spawning more.")

    if not state.get("skip_perms", False):
        lines.append(
            "Shell is denied for subagents here, so name the exact files each one should "
            "read; it cannot discover them itself.")
    return "\n".join(lines)


def main():
    try:
        event = json.load(sys.stdin)
    except ValueError:
        return
    if event.get("hook_event_name") != "PreToolUse":
        return

    root = Path(event.get("cwd") or os.getcwd())
    state = merged_state(root / ".codex" / "argon-runs")
    if state is None:
        # No state yet: first spawn of the session. Read the limits from the server's
        # own config — a hook process does not inherit mcp_servers.argon.env, so
        # os.environ would report a default that has nothing to do with reality.
        state = dict(config_defaults(root))
        state["free_slots"] = state["max_parallel"]
    tool_input = event.get("tool_input") or {}
    tasks = tool_input.get("tasks")
    requested = len(tasks) if isinstance(tasks, list) else (1 if tool_input.get("task") else 0)

    text = build(state, requested)
    if text:
        print(json.dumps({"hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "additionalContext": text[:MAX_CHARS],
        }}))


if __name__ == "__main__":
    try:
        main()
    except Exception:
        log_error()
    sys.exit(0)  # fail open: never block an argon call
