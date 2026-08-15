# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""CLI entry point for the Python DSC adapter."""
from __future__ import annotations

import argparse
import json
import sys


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="pyadapter",
        description="Microsoft.Adapter/Python — DSC Python adapter.",
    )
    parser.add_argument(
        "operation",
        choices=["discover", "list", "get", "set", "test", "delete", "export", "validate", "clear-cache"],
        help="Operation to perform.",
    )
    parser.add_argument(
        "--resource",
        dest="resource_type",
        default="",
        metavar="TYPE",
        help="Fully-qualified resource type (e.g. MyPackage/MyResource).",
    )
    parser.add_argument(
        "--content",
        dest="adapted_content",
        default=None,
        nargs="?",
        const=None,
        metavar="JSON",
        help="JSON object from the adapted resource manifest's 'content' field (injected by DSC via adaptedContentArg).",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    from ms_dsc.logging import configure_logging

    configure_logging()

    parser = _build_parser()
    args = parser.parse_args(argv)

    if args.operation == "discover":
        from pyadapter.discovery import cmd_discover
        return cmd_discover()

    if args.operation == "list":
        from pyadapter.discovery import cmd_list
        return cmd_list()

    if args.operation == "clear-cache":
        from pyadapter.discovery import cmd_clear_cache
        return cmd_clear_cache()

    if args.operation == "validate":
        print(json.dumps({"valid": True}))
        return 0

    # Operations that require --resource and read JSON from stdin.
    if not args.resource_type:
        print(json.dumps({"error": "--resource is required for this operation"}), file=sys.stderr)
        return 2

    # Parse the optional adapted content JSON injected by DSC via adaptedContentArg.
    adapted_content: dict | None = None
    if args.adapted_content:
        try:
            adapted_content = json.loads(args.adapted_content)
        except json.JSONDecodeError as exc:
            print(json.dumps({"error": f"Invalid --content JSON: {exc}"}), file=sys.stderr)
            return 2

    stdin_json = sys.stdin.read()

    from pyadapter.router import dispatch
    return dispatch(args.operation, args.resource_type, stdin_json, adapted_content)


if __name__ == "__main__":
    sys.exit(main())
