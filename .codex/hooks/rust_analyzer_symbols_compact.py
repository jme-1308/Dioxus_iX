#!/usr/bin/env python3
"""Replace verbose rust-analyzer symbol metadata with a navigation index."""

import json
import sys
from collections import Counter


SYMBOLS_TOOL = "mcp__rust_analyzer__rust_analyzer_symbols"


def symbols_from_response(response):
    """Extract the symbol array from the MCP content envelope."""
    if isinstance(response, list):
        if all(
            isinstance(item, dict) and "name" in item and "location" in item
            for item in response
        ):
            return response
        for item in response:
            symbols = symbols_from_response(item)
            if symbols is not None:
                return symbols
        return None
    if isinstance(response, str):
        try:
            return symbols_from_response(json.loads(response))
        except (json.JSONDecodeError, TypeError):
            return None
    if not isinstance(response, dict):
        return None
    if response.get("type") == "text":
        return symbols_from_response(response.get("text"))
    for key in ("content", "structuredContent"):
        if key in response:
            symbols = symbols_from_response(response[key])
            if symbols is not None:
                return symbols
    return None


def compact_symbols(symbols):
    """Keep only the fields needed to target a positional lookup."""
    names = Counter(symbol.get("name", "?") for symbol in symbols)
    lines = []
    for symbol in symbols:
        name = symbol.get("name", "?")
        start = symbol.get("location", {}).get("range", {}).get("start", {})
        line = start.get("line", "?")
        character = start.get("character", "?")
        container = symbol.get("containerName")
        suffix = f" [{container}]" if names[name] > 1 and container else ""
        lines.append(f"{name}@{line}:{character}{suffix}")
    return (
        f"rust-analyzer symbols ({len(symbols)}; zero-based name@line:character; "
        "container shown for duplicate names):\n" + "\n".join(lines)
    )


def main():
    try:
        event = json.load(sys.stdin)
    except Exception:
        return 0

    if event.get("tool_name") != SYMBOLS_TOOL:
        return 0
    symbols = symbols_from_response(event.get("tool_response"))
    if symbols is None:
        return 0

    print(json.dumps({
        "decision": "block",
        "reason": (
            "The symbols call succeeded. Use this compact index; do not retry symbols.\n"
            + compact_symbols(symbols)
        ),
    }))
    return 0


if __name__ == "__main__":
    sys.exit(main())
