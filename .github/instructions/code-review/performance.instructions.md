---
applyTo: '**/*.rs'
description: 'Code review guidance for performance (Rust allocations, caching, serialization)'
---

# Performance Code Review

These patterns apply to all Rust code but are most critical in engine hot paths: resource
discovery (many manifests), schema caching, and configuration processing.

## Caching

- **Cache key correctness**: Keys must include all dimensions that affect the value. Adapter schemas vary by `target_resource`.
- **Centralize cache writes**: Duplicate cache mutations lead to inconsistent keying and redundant work.
- **Avoid serialize/parse roundtrips**: Don't stringify a `Value` just to re-parse it.
- **Deterministic output**: Cache hits and misses must produce identical format (compact vs pretty).

## Allocations

- **Reduce allocations only when semantics stay intact**: Flag repeated `to_lowercase()` or cloning owned data that can be borrowed. Do NOT flag clone removal that changes ownership or breaks compilation.
- **Short-circuit redundant work**: Use entry APIs instead of `contains_key` + `insert`. Exit loops early when all matches are found.
- **Avoid cloning before parsing**: Parse first, then return the original owned value.

## Native Interop

- **Resource cleanup as performance issue**: Missing `Drop`/RAII cleanup for handles, DLLs, or `VARIANT`s is both correctness and long-run efficiency problem. Especially in loops (firewall enumeration, registry key iteration).

## False Positives to Avoid

- **Do not flag performance when call order makes it moot**: If `get` always runs before `set`/`test`, redundant validation in later operations doesn't affect real-world performance.
- **Compact vs pretty JSON**: Maintainers have noted "doesn't matter here" for format differences in non-user-facing internal paths. Only flag when the difference is externally observable.
