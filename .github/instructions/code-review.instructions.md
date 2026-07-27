---
applyTo: '**/*.rs,**/*.ps1,**/*.psm1,**/*.tests.ps1,**/*.json'
description: 'Repository-specific code review guidance for DSC (Rust, PowerShell/Pester, JSON)'
---

# Code Review Instructions for DSC Repository

These instructions guide Copilot when performing code reviews on pull requests in this repository.
Focus on high-confidence, actionable findings. Do not comment on style, formatting, or trivial issues.

Detailed review guidance is split by contribution area in `.github/instructions/code-review/`.
Use the path-based mapping below to determine which area instructions apply to the files under review.

## Area Routing

| Area | Paths | Instruction File |
|------|-------|------------------|
| Engine | `lib/dsc-lib/src/configure/`, `lib/dsc-lib/src/discovery/`, `lib/dsc-lib/src/dscresources/`, `lib/dsc-lib/src/functions/`, `lib/dsc-lib/src/settings/` | `code-review/engine.instructions.md` |
| Resources | `resources/` | `code-review/resource.instructions.md` |
| Adapters | `adapters/` | `code-review/adapter.instructions.md` |
| Extensions | `extensions/`, `lib/dsc-lib/src/extensions/` | `code-review/extension.instructions.md` |
| CLI | `dsc/src/` | `code-review/cli.instructions.md` |
| Tests | `**/*.tests.ps1`, `**/tests/`, `**/test/`, `**/Tests/` | `code-review/tests.instructions.md` |
| Libraries | `lib/` | `code-review/library.instructions.md` |
| Security | `lib/dsc-lib/src/util.rs`, `lib/dsc-lib-security_context/`, `lib/dsc-lib-registry/`, `resources/registry/`, `resources/windows_firewall/`, `resources/windows_service/`, `resources/dism_dsc/` | `code-review/security.instructions.md` |
| Performance | Any `*.rs` file in hot paths | `code-review/performance.instructions.md` |
| Windows | `resources/windows_*`, `resources/dism_dsc/`, `resources/registry/`, `lib/dsc-lib-registry/`, `lib/dsc-lib-pal/` | `code-review/windows.instructions.md` |
| Linux/macOS | `resources/apt/`, `resources/sshdconfig/`, `resources/brew/`, `lib/dsc-lib-pal/` | `code-review/linux.instructions.md` |

Multiple areas may apply to a single file. For example, `resources/windows_firewall/` triggers
both the Resource, Windows, and Security instructions.

## General Principles (False-Positive Avoidance)

These apply to ALL areas:

- **Do not claim code will not compile unless you are certain**: Multiple reviews were rejected because Copilot incorrectly claimed Rust ownership/borrowing errors. If the code compiles and tests pass in CI, do not assert otherwise.
- **Test resources are not production code**: Code in `tools/dsctest/` is for testing only and is never user-facing. Do not require production-grade error handling in test harnesses unless panics would hide regressions.
- **Automatically-generated files**: Files like `lib/dsc-lib-jsonschema/.versions.json` are updated by build automation. Do not flag version bumps as unintentional.
- **Do not demand large abstractions in small PRs**: Suggest extracting a helper function. Do not block a focused PR by requesting trait/framework redesigns -- that is follow-up work.
- **Table output is not a stable API**: JSON/YAML are canonical machine-readable outputs. Table layout changes are not breaking.
- **Separate input/output structs can be intentional**: Resources often distinguish desired state from observed state. Do not assume duplicate structs are accidental.
- **Intentional design decisions**: When maintainers explicitly label behavior as "intentional", do not re-flag.

## Documentation and Logging (All Areas)

- **Accurate comments**: If code behavior changes, update comments to match.
- **Log level appropriateness**: Full `PATH` contents at `trace!`, not `debug!`. Never log secrets.
- **Doc comments matching implementation**: Update when described behavior doesn't match reality.
- **Remove debug print statements**: Do not use `println!` for debugging -- use `debug!`/`trace!` macros. Note that `println!` is acceptable for intentional CLI user-facing output.
- **Locale/i18n string accuracy**: Verify key names match the keyword/function they describe.
- **Dead locale strings**: Do not add i18n keys never referenced in code.
- **Prefer scan-friendly wording**: Log/error strings should be concise for readability in diagnostics.

## CI/CD (All Areas)

- **Fork permission limitations**: Steps posting PR comments should gate on same-repo PRs or use `continue-on-error: true`.
- **Conditional tool installation**: Install expensive tools only after determining the PR needs them.
- **Check control flow, not just syntax**: Verify `if: always()`, `continue-on-error`, and `needs`/stage dependencies match stated failure behavior.
- **Preserve downstream artifact contracts**: Flag changes to artifact names/paths when downstream jobs expect the old layout.
- **PowerShell conventions**: Singular function names (`Test-RustProject`). Build failures should throw.
