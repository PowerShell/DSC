import sys
import json
import argparse
from typing import Optional
# TODO: Currently using absolute imports. Switch to relative imports if this is later used as a module in the adapter manifest; otherwise, keep as-is for direct script execution.
from adapter import ResourceAdapter

# --------------------
# CLI / entrypoint API
# --------------------
def _build_parser() -> argparse.ArgumentParser:
    """Construct the argument parser for the DSC adapter CLI."""
    parser = argparse.ArgumentParser(
        prog="pyDscAdapter",
        description="DSC v3 Python adapter."
    )
    sub = parser.add_subparsers(dest="command", required=True)

    adapter = sub.add_parser("adapter", help="Adapter operations")
    adapter.add_argument("--operation", required=True, choices=["list", "get", "set", "test", "export", "validate"],
                         help="Adapter operation to execute.")
    adapter.add_argument("--input", default="{}", help="JSON string with resource configuration (single input).")
    adapter.add_argument("--resource", dest="ResourceType", default="", help="Resource type selector (e.g., Microsoft.Linux.Apt/Package).")
    adapter.add_argument("--resource-path", dest="ResourcePath", default="", help="Optional resource module file path.")
    return parser


def main(argv: Optional[list] = None) -> int:
    """Main entry point for the DSC adapter CLI."""
    parser = _build_parser()
    args = parser.parse_args(argv)

    if args.command != "adapter":
        print(json.dumps({"error": "Unsupported command"}))
        return 2

    adapter = ResourceAdapter()


    # 1. Start with --input as the authoritative source
    input_str = args.input

    # 2. If stdin has data, it overrides --input (DSC convention)
    stdin_data = sys.stdin.read().strip() if not sys.stdin.isatty() else ""
    if stdin_data:
        input_str = stdin_data

    # 3. Call operation handler
    exit_code, result = adapter.run_operation(
        args.operation,
        input_str,
        args.ResourceType,
        getattr(args, "ResourcePath", "")
    )
    
    # If set branch (or similar) already wrote to stdout, skip emitting a wrapper
    if isinstance(result, dict) and result.get("_stdout_emitted"):
        return exit_code

    # 4. Capture EXACT output passed to DSC
    out_json = json.dumps(result, ensure_ascii=False)

    print(out_json)
    return exit_code
