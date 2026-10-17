#!/usr/bin/env python3
"""Codex Stop hook: don't let a turn end with Argon work unread.

argon_spawn returns immediately, so nothing in the protocol forces Codex to ever call
argon_collect. Without this, a turn can end with finished reviews sitting on disk and
the quota already spent.

Two cases, deliberately treated differently:

  finished but uncollected -> always block. The answers exist and reading them costs
                              nothing, so there is no excuse for skipping them.
  still running            -> block at most MAX_BOUNCES times. Subagents may run
                              without limit (AGY_TIMEOUT_MIN=0), and a 40-minute run
                              must not be able to deadlock the session. After that the
                              turn ends with the pending ids named.

Blocking is `exit 2` with the reason on stderr, the same mechanism rust_check.py uses.
Any other outcome is exit 0, so a broken hook can never wedge a session.
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
MAX_BOUNCES = 3          # ~3 x argon_collect(240s) of real waiting before giving up
BOUNCE_TTL = 3600        # forget the counter after an hour; a new turn starts fresh
STALE_AFTER = 1800       # state written longer ago than this is from a dead server


def log_error():
    try:
        ERROR_LOG.parent.mkdir(parents=True, exist_ok=True)
        with ERROR_LOG.open("a") as f:
            f.write(f"--- {time.strftime('%Y-%m-%d %H:%M:%S')} argon_guard\n{traceback.format_exc()}\n")
    except OSError:
        pass


def bounces(path, reset=False):
    """Count how many times we have already blocked, so we cannot loop forever."""
    now = time.time()
    try:
        d = json.loads(path.read_text())
        if now - d.get("at", 0) > BOUNCE_TTL:
            d = {"n": 0}
    except (OSError, ValueError):
        d = {"n": 0}
    if reset:
        try:
            path.unlink(missing_ok=True)
        except OSError:
            pass
        return 0
    d = {"n": d.get("n", 0) + 1, "at": now}
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(d))
    except OSError:
        pass
    return d["n"]


def merged_state(state_dir):
    """Combine the state of every argon server still alive.

    Codex runs one MCP server per thread and each writes its own state.<pid>.json, so
    the fleet is the union of them. Files from dead servers are ignored: their jobs are
    either finished and on disk, or will be adopted by the next server.
    """
    merged = {"running": [], "running_detail": [], "queued": 0,
              "uncollected": [], "servers": 0}
    found = False
    for f in sorted(state_dir.glob("state.*.json")):
        try:
            pid = int(f.name.split(".")[1])
        except (IndexError, ValueError):
            continue
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            continue
        except PermissionError:
            pass
        try:
            s = json.loads(f.read_text())
        except OSError:
            continue
        except ValueError:
            log_error()  # blind is worse than noisy; this masked a corruption bug once
            continue
        if time.time() - s.get("updated", 0) > STALE_AFTER:
            continue
        found = True
        merged["servers"] += 1
        merged["running"] += s.get("running") or []
        merged["running_detail"] += s.get("running_detail") or []
        merged["queued"] += s.get("queued", 0)
        merged["uncollected"] += s.get("uncollected") or []
    return merged if found else None


def session_jobs(state_dir, session_id):
    """Job ids this Codex session actually spawned, per the PostToolUse claim file.

    Returns None when there is no claim file — meaning this session has not spawned
    anything, so it must not be held responsible for another thread's work.
    """
    safe = re.sub(r"[^A-Za-z0-9_.-]", "_", session_id or "default")
    try:
        return set(json.loads((state_dir / f".session-{safe}.json").read_text()).get("jobs") or [])
    except (OSError, ValueError):
        return None


def main():
    try:
        event = json.load(sys.stdin)
    except ValueError:
        return 0
    root = Path(event.get("cwd") or os.getcwd())
    bounce_path = root / STATE_DIR / ".stop-bounces.json"

    state = merged_state(root / STATE_DIR)
    if state is None:
        return 0  # argon never ran here, or every server that did is gone

    # Enforce only what THIS session spawned. Several Codex threads share one checkout
    # and each has its own argon server; without this a turn gets blocked by a sibling
    # thread's unread results, which it cannot collect and did not ask for.
    mine = session_jobs(root / STATE_DIR, event.get("session_id"))
    if mine is None:
        return 0  # this session never spawned anything
    state["uncollected"] = [u for u in state["uncollected"] if u.get("id") in mine]
    running_detail = [r for r in state["running_detail"] if r.get("id") in mine]
    state["running"] = [r.get("name", "?") for r in running_detail]
    state["queued"] = 0  # queued jobs are counted inside running_detail

    uncollected = state.get("uncollected") or []
    running = state.get("running") or []
    queued = state.get("queued", 0)

    if not uncollected and not running and not queued:
        bounces(bounce_path, reset=True)
        return 0

    if uncollected:
        ids = ", ".join(repr(u["id"]) for u in uncollected[:8])
        names = ", ".join(f"{u['name']} ({u['status']})" for u in uncollected[:8])
        sys.stderr.write(
            f"{len(uncollected)} Argon subagent(s) finished and you have not read the "
            f"results: {names}.\n"
            f"Call argon_collect([{ids}]) and use what it returns before you finish. "
            f"These runs already cost quota; ending the turn now throws them away.\n")
        return 2

    n = bounces(bounce_path)
    pending = ", ".join(running[:6]) or f"{queued} queued"
    if n <= MAX_BOUNCES:
        sys.stderr.write(
            f"{len(running) + queued} Argon subagent(s) still running ({pending}).\n"
            f"Call argon_collect() to wait for them, then use the results. "
            f"(reminder {n}/{MAX_BOUNCES} — after that the turn may end without them)\n")
        return 2

    bounces(bounce_path, reset=True)
    sys.stderr.write(
        f"Note: {len(running) + queued} Argon subagent(s) are still running ({pending}) "
        f"after {MAX_BOUNCES} waits. Finishing without them. Tell the user the Argon "
        f"results are still pending and can be read next turn with argon_collect().\n")
    return 0  # released: a long subagent must not be able to deadlock the session


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception:
        log_error()
        sys.exit(0)  # fail open
