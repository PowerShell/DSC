---
applyTo: 'dsc/src/**'
description: 'Code review guidance for DSC CLI (command-line interface, subcommands, server mode)'
---

# CLI Code Review

The `dsc` crate is the main CLI binary. It handles argument parsing, subcommand dispatch,
output formatting (JSON/YAML/table), and the MCP server mode.

## Output Formatting

- **Deterministic output**: JSON format (compact vs pretty) must be consistent regardless of internal state.
- **Table output is not a stable API**: JSON/YAML are canonical. Table changes are not breaking.
- **Extract shared formatting logic**: When table/list formatting appears in multiple subcommands, extract a helper rather than duplicating.

## Error Handling

- **Prefer `Result` over panics**: Avoid `unwrap()`/`expect()` on user input or parsed data. Return clear error messages and exit codes.
- **Names must match semantics**: Flag singular/plural mismatches, stale help text, and deprecated options still visibly advertised.

## Server Mode (JSONRPC)

- **CLI additions must be reflected in server mode**: Any new CLI subcommand or capability must
  also be exposed as a corresponding JSONRPC API in server mode. Flag PRs that add CLI
  functionality without a matching server-mode implementation or a tracking issue for follow-up.
- **Schema/typing for tool parameters**: Prefer typed parameters over generic `serde_json::Value` with runtime validation. This gives clients correct schemas.
- **Tool name accuracy**: Ensure locale strings and schema descriptions reference the correct tool name (e.g., `list_dsc_functions` not `list_dsc_function`).

## Backward Compatibility

- **CLI flag changes are compatibility events**: Renaming, removing, or changing the semantics of CLI flags requires consideration of existing users and scripts.
- **Breaking test expectations**: If existing tests must change due to CLI behavior changes, that signals a potential breaking change needing discussion.
