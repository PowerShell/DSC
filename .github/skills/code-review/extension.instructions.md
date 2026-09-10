---
applyTo: 'extensions/**,lib/dsc-lib/src/extensions/**'
description: 'Code review guidance for DSC extensions (discovery, lifecycle)'
---

# Extension Code Review

Extensions extend DSC's discovery and resource capabilities. They have their own discovery
protocol and manifest format.

## Discovery Protocol

- **Manifest content handling**: When deserializing discovered manifests, ensure the expected shape (`ImportedManifest` enum with `Resource`/`Extension` variants) matches what extensions actually emit.
- **Unused variables after refactoring**: When switching from reused helpers (e.g., `process_get_args`) to dedicated ones (e.g., `process_discover_args`), remove intermediate variables and imports that are no longer needed.
- **Doc comments must match parameter semantics**: If a doc comment says "file path" but the parameter actually receives an argument name or extension list, update the comment.

## Schema Updates

- **Update JSON schemas when adding protocol fields**: New fields in extension stdout or args (e.g., `manifestContent`, `extensionsArg`) must be reflected in the checked-in JSON schemas under `schemas/`.

## PowerShell Extensions

- **`$using:` in parallel blocks**: Variables from parent scope are not available inside `ForEach-Object -Parallel`. Use `$using:variableName`.
- **Error isolation**: One extension failing discovery should not abort the entire extension enumeration.
