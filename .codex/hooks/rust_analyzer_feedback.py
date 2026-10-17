#!/usr/bin/env python3
"""Inject concise recovery guidance after a failed rust-analyzer MCP lookup."""

import json
import sys


POSITIONAL_TOOLS = {
    "mcp__rust_analyzer__rust_analyzer_hover",
    "mcp__rust_analyzer__rust_analyzer_definition",
    "mcp__rust_analyzer__rust_analyzer_references",
    "mcp__rust_analyzer__rust_analyzer_completion",
    "mcp__rust_analyzer__rust_analyzer_code_actions",
    "mcp__rust_analyzer__rust_analyzer_rename",
}


def unusable_response(value):
    """Recognize empty results and coordinate errors that need symbol recovery."""
    if value is None:
        return True
    if isinstance(value, str):
        normalized = value.strip().lower()
        return normalized in {"", "null", "[]", "{}"} or any(
            marker in normalized
            for marker in ("tool call error", "mcp error", "invalid offset")
        )
    if isinstance(value, list):
        return not value or all(unusable_response(item) for item in value)
    if isinstance(value, dict):
        if value.get("type") == "text" and "text" in value:
            return unusable_response(value["text"])
        if "content" in value:
            return unusable_response(value["content"])
        if "structuredContent" in value:
            return unusable_response(value["structuredContent"])
        if "error" in value:
            return True
    return False


def main():
    try:
        event = json.load(sys.stdin)
    except Exception:
        return 0

    if event.get("tool_name") not in POSITIONAL_TOOLS:
        return 0
    if not unusable_response(event.get("tool_response")):
        return 0

    tool_input = event.get("tool_input") or {}
    file_path = tool_input.get("file_path")
    target = f" for {file_path!r}" if file_path else ""
    context = (
        f"Rust-analyzer lookup failed or returned no result{target}. Call symbols, then retry on "
        "the identifier in its zero-based range (which may begin at docs or attributes). For empty "
        "references, use a resolved call site. Do not guess."
    )
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PostToolUse",
            "additionalContext": context,
        }
    }))
    return 0


if __name__ == "__main__":
    sys.exit(main())
