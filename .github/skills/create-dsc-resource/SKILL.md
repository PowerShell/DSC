---
name: create-dsc-resource
description: |
    Create a complete, accurate DSC resource in this repository following the provided guidelines and design patterns.
---

# Create high‑quality DSC resource

You are a code agent. Your task is to create a complete, accurate DSC resource in this repository.

## What is a DSC Resource?

A DSC (Desired State Configuration) resource is an executable that performs configuration tasks for a specific domain.
Example domains include Windows features, package manager, services, files, registry, and more. Each resource has a specific schema and set of properties that define its behavior.
Management tasks or operations are specific to the resource type, but may include: get, set, test, and export.

## Key Principles

- **Programming language**: Resources in this repository may be implemented in Rust or as scripts (for example, PowerShell). The guidance in this skill primarily applies to Rust-based resources.
- **Resource manifest**: A JSON file that defines the resource type name, supported operations (including executable and arguments), and JSON schema for input parameters
- **Dependency management**: All crates must be listed in Cargo.toml specifying to use workspace dependencies. The root level Cargo.toml should be updated to include the new crates or associated to the DSC resource project.
- **Project files**: A `.project.data.json` file in the root of the project folder defines properties of the project and non-code files to include during build
- **Localization**: For Rust-based resources, all user-facing strings must use `rust-i18n` for internationalization. For script-based resources (such as PowerShell), follow the existing localization and string-handling patterns used by those scripts or any repository-specific localization guidance.
- **Copyright headers**: Every source file must start with the copyright header:
  ```
  // Copyright (c) Microsoft Corporation.
  // Licensed under the MIT License.

  ```
  For PowerShell test files, use `#` comment syntax instead of `//`.

## File Structure and Content Guidelines

### 1. Required Setup

- Use `cargo new --bin <resource_name>` to create a new Rust binary project for the resource under the `resources` directory
- Create a resource manifest JSON file named `<resource_name>.dsc.resource.json` in the same directory using `./resources/windows_service/windows_service.dsc.resource.json` as an example
- Create a `.project.data.json` file in the root of the resource project directory
- Create a `locales/en-us.toml` file for localized strings

### 2. .project.data.json

This file defines the project metadata used by the build system. Structure:

```json
{
    "Name": "<resource_name>",
    "Kind": "Resource",
    "IsRust": true,
    "SupportedPlatformOS": "Windows",
    "Binaries": [
        "<resource_name>"
    ],
    "CopyFiles": {
        "Windows": [
            "<resource_name>.dsc.resource.json"
        ]
    }
}
```

- `SupportedPlatformOS`: Use `"Windows"` for Windows-only resources. Omit this field for cross-platform resources.
- `CopyFiles`: Use `"Windows"` for Windows-only files, `"All"` for cross-platform files.

### 3. Cargo.toml

```toml
[package]
name = "<resource_name>"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "<resource_name>"
path = "src/main.rs"

[package.metadata.i18n]
available-locales = ["en-us"]
default-locale = "en-us"
load-path = "locales"

[dependencies]
rust-i18n = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }

[target.'cfg(windows)'.dependencies]
windows = { workspace = true }
```

- All dependencies should reference `{ workspace = true }` to use workspace-level version pinning.
- Use `[target.'cfg(windows)'.dependencies]` for Windows-only crates like `windows`.
- The `[package.metadata.i18n]` section configures the `rust-i18n` crate.

### 4. Resource Manifest (`<resource_name>.dsc.resource.json`)

Key fields:

```json
{
    "$schema": "https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json",
    "type": "Microsoft.<Namespace>/<Resource>",
    "description": "Short description of the resource",
    "tags": ["Windows"],
    "version": "0.1.0",
    "get": {
        "executable": "<resource_name>",
        "args": ["get", { "jsonInputArg": "--input", "mandatory": true }]
    },
    "set": {
        "executable": "<resource_name>",
        "args": ["set", { "jsonInputArg": "--input", "mandatory": true }],
        "implementsPretest": false,
        "return": "state",
        "requireSecurityContext": "elevated"
    },
    "export": {
        "executable": "<resource_name>",
        "args": ["export", { "jsonInputArg": "--input", "mandatory": false }]
    },
    "exitCodes": {
        "0": "Success",
        "1": "Invalid arguments",
        "2": "Invalid input",
        "3": "Resource-specific error description"
    },
    "schema": {
        "embedded": { ... }
    }
}
```

- `implementsPretest`: Set to `false` if `set` does not internally perform a `test` first. DSC will call `test` before `set` automatically.
- `return`: Use `"state"` if the set operation returns the final state of the resource.
- `requireSecurityContext`: Use `"elevated"` for operations that require administrator privileges.
- `schema.embedded`: Define the JSON schema inline with `"additionalProperties": false`. Schema property names use camelCase. Use `"readOnly": true` for properties that are output-only (e.g., `_exist`).

### 5. Essential Design Patterns

#### main.rs file

- Initialize `rust-i18n` with `rust_i18n::i18n!("locales", fallback = "en-us");`
- Define named exit code constants at the module level:
  ```rust
  const EXIT_SUCCESS: i32 = 0;
  const EXIT_INVALID_ARGS: i32 = 1;
  const EXIT_INVALID_INPUT: i32 = 2;
  const EXIT_RESOURCE_ERROR: i32 = 3;
  ```
- Implement common helper functions:
  - `write_error(message)` — writes `{"error": "<message>"}` to stderr
  - `require_input(input_json)` — deserializes the JSON input or exits with an error
  - `print_json(value)` — serializes and prints to stdout, or exits with an error
  - `parse_input_arg(args)` — parses `--input <json>` from command-line arguments
- Use `t!("key")` macro from `rust-i18n` for all user-facing strings (error messages, etc.)
- For platform-specific resources, use conditional compilation:
  ```rust
  #[cfg(not(windows))]
  fn main() {
      write_error(&t!("main.windowsOnly"));
      exit(EXIT_RESOURCE_ERROR);
  }

  #[cfg(windows)]
  fn main() {
      // ... actual implementation
  }
  ```
- Parse operations as the first positional argument (e.g., `get`, `set`, `export`), with `--input <json>` as the input argument

#### types.rs file

- Define Rust structs that represent the input parameters for each operation, matching the JSON schema defined in the resource manifest
- Implement serialization and deserialization for these types using Serde
- Use Serde attributes to map between Rust snake_case and JSON camelCase:
  ```rust
  #[derive(Debug, Default, Serialize, Deserialize, Clone)]
  #[serde(rename_all = "camelCase")]
  pub struct MyResource {
      #[serde(skip_serializing_if = "Option::is_none")]
      pub some_field: Option<String>,

      #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
      pub exist: Option<bool>,
  }
  ```
- Use `Option<T>` for all fields so that partial input is accepted (only specified fields are acted upon)
- Define enums for properties with constrained values, each with `Serialize`, `Deserialize`, `PartialEq`, `Clone` derives
- Implement `std::fmt::Display` for enums that need string representation
- Define a custom error type with `std::error::Error` and `std::fmt::Display` implementations, plus a `From<String>` conversion

#### Localization (`locales/en-us.toml`)

All user-facing strings must be defined in a TOML locale file and referenced via the `t!()` macro. Structure:

```toml
_version = 1

[main]
missingOperation = "Missing operation. Usage: <resource_name> get --input <json> | set --input <json>"
unknownOperation = "Unknown operation: '%{operation}'. Expected: get, set, or export"
missingInput = "Missing --input argument"
invalidJson = "Invalid JSON input: %{error}"

[get]
someError = "Failed to do something: %{error}"

[set]
someError = "Failed to do something: %{error}"
```

- Group messages by operation (main, get, set, export)
- Use `%{variable}` syntax for interpolated values
- Reference in Rust code as `t!("section.key", variable = value)`

#### Calling native APIs

- Create a separate module (e.g., `service.rs`) for interacting with native OS APIs that contains unsafe code and FFI bindings
- Gate the module with `#[cfg(windows)]` if it uses Windows-specific APIs
- Use RAII wrappers for OS handles that call cleanup functions on `Drop` (e.g., `CloseServiceHandle`, `CloseHandle`)
- Use helper functions for UTF-16 string conversion (`to_wide`, `pwstr_to_string`)
- The code should be designed to be architecture agnostic, working on both x64 and ARM64 systems

#### Error handling

- Use Rust's `Result` type for error handling in operation handlers
- Writing of errors should only be done in the main.rs file, with error messages that are clear and actionable for users of the resource
- Errors are written as a JSONLine to stderr using the format: `{"error": "Error message here"}`
- Warnings are written as a JSONLine to stderr using the format: `{"warn": "Warning message here"}`
- A non-zero exit code should be returned for any operation that encounters an error, while warnings should not affect the exit code
- Use different exit codes for different types of errors. Define named constants (e.g., `EXIT_INVALID_ARGS`, `EXIT_INVALID_INPUT`) and document each in the resource manifest `exitCodes` section.

#### Testing Instructions

- Create a `tests` folder in the resource project directory
- For each operation supported by the resource (get, set, test, export as applicable), create a separate test file (e.g., `<resource_name>_get.tests.ps1`, `<resource_name>_set.tests.ps1`, etc.)
- The resource should be tested using `dsc resource` command instead of calling the executable directly
- Pipe JSON input via stdin using the `-f -` flag: `$json | dsc resource get -r $resourceType -f -`
- Redirect stderr to a file for debugging: `2>$testdrive/error.log`
- Use `-Because` with `$LASTEXITCODE` assertions to include stderr output for diagnostics:
  ```powershell
  $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
  $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
  ```
- Parse the output and access the actual state via `$output.actualState`:
  ```powershell
  $output = $out | ConvertFrom-Json
  $result = $output.actualState
  ```
- For platform-specific resources, use `-Skip` on the top-level `Describe` block:
  ```powershell
  Describe 'Resource tests' -Skip:(!$IsWindows) { ... }
  ```
- For tests requiring elevated privileges, add an admin check to the skip condition
- Use `-ForEach` or `-TestCases` for data-driven tests (e.g., validating enum property values)
- Follow the PowerShell Pester v5 testing instructions for structuring the tests and making assertions

#### Code Style Guidelines

- Use Rust pedantic clippy linting rules for all code in the resource project

#### Build and Deployment

- The resource should be built using `build.ps1 -project <resource_name>` from the root of the repository, which will handle building the Rust code and ensure it is found in PATH for testing

## What-If support

Follow this pattern exactly when adding what-if (a.k.a. `dsc config set --what-if`) support to any resource so the implementation path, manifest changes, tests, and naming stay consistent across the repository.

What-if must:

1. Project the **final state** the resource would produce, without mutating the system.
2. Echo back the relevant input fields (`keyPath`, `valueName`, `valueData`, etc.) so the engine can diff before/after.
3. Attach human-readable "would do …" messages under `_metadata.whatIf` (an array of strings).
4. Exit `0` on success — what-if is not an error path.

### 1. Resource manifest changes

In the resource's `*.dsc.resource.json`, add `whatIfArg` to the `set` (and `delete`, if it supports what-if) args array, and declare `whatIfReturns: "state"` on `set`:

```json
"set": {
    "executable": "<resource_name>",
    "args": [
        "config", "set",
        { "jsonInputArg": "--input", "mandatory": true },
        { "whatIfArg": "-w" }
    ],
    "whatIfReturns": "state"
},
"delete": {
    "executable": "<resource_name>",
    "args": [
        "config", "delete",
        { "jsonInputArg": "--input", "mandatory": true },
        { "whatIfArg": "-w" }
    ]
}
```

- `whatIfArg` is the literal CLI flag DSC will append when the user runs `dsc config set --what-if`. Always use `"--what-if"` (long form) for consistency across resources.
- `whatIfReturns: "state"` tells DSC the executable prints the projected post-state JSON on stdout (same shape as `get`/`set` returns).
- The `--list` (bulk) variant of a resource uses the same two manifest additions; do not invent new flag names.

### 2. CLI args (`args.rs`) changes

Add a `-w` / `--what-if` boolean to every `ConfigSubCommand` variant that can support what-if:

```rust
#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum ConfigSubCommand {
    #[clap(name = "set", about = t!("args.configSetAbout").to_string())]
    Set {
        #[clap(short, long, required = true, help = t!("args.configArgsInputHelp").to_string())]
        input: String,
        #[clap(short = 'w', long, help = t!("args.configArgsWhatIfHelp").to_string())]
        what_if: bool,
    },
    #[clap(name = "delete", about = t!("args.configDeleteAbout").to_string())]
    Delete {
        #[clap(short, long, required = true, help = t!("args.configArgsInputHelp").to_string())]
        input: String,
        #[clap(short = 'w', long, help = t!("args.configArgsWhatIfHelp").to_string())]
        what_if: bool,
    },
}
```

Naming is fixed: clap field is `what_if`, short flag is `-w`, long flag is `--what-if`, help key is `args.configArgsWhatIfHelp`.

### 3. `main.rs` dispatch

In each `Set` / `Delete` arm, destructure `what_if`, call `helper.enable_what_if()` when true, and print the returned projected state on stdout. **Never** mutate state when `what_if` is true.

```rust
args::ConfigSubCommand::Set { input, what_if } => {
    trace!("Set input: {input}, what_if: {what_if}");
    let mut helper = match Helper::new_from_json(&input) {
        Ok(h) => h,
        Err(err) => { error!("{err}"); exit(EXIT_INVALID_INPUT); }
    };
    if what_if { helper.enable_what_if(); }

    match helper.set() {
        Ok(Some(state)) => {
            // Set returns Some(state) when what_if is true (projected state)
            // OR when whatIfReturns == "state" and the resource emits final state.
            let json = serde_json::to_string(&state).unwrap();
            println!("{json}");
        }
        Ok(None) => {}
        Err(err) => { error!("{err}"); exit(EXIT_RESOURCE_ERROR); }
    }
    exit(EXIT_SUCCESS);
}
```

For the `--list` bulk variant, accumulate projected states in a `Vec`, then print the whole list once at the end.

### 4. Library / helper changes

In the resource's `dsc-lib-*` crate:

- Add a `what_if: bool` field on the helper struct, defaulting to `false` in every constructor.
- Expose `pub fn enable_what_if(&mut self) { self.what_if = true; }`.
- Change `set()` (and `remove()`) to return `Result<Option<T>, Error>` where `Some(T)` is the projected state when `what_if` is true.
- Inside `set` / `remove`, build a `Vec<String> what_if_metadata`, push localized "Would …" strings at each side-effecting branch, and **short-circuit** before the real OS call when `self.what_if`:

  ```rust
  if self.what_if {
      what_if_metadata.push(t!("<resource>_helper.whatIfCreate<Thing>", name = name).to_string());
  } else {
      // perform the real OS mutation
  }
  ```

- Return the projected state with the metadata attached:

  ```rust
  return Ok(Some(<ResourceState> {
      // identity + projected fields the engine needs to diff
      metadata: if what_if_metadata.is_empty() {
          None
      } else {
          Some(Metadata { what_if: Some(what_if_metadata) })
      },
      ..Default::default()
  }));
  ```

- Add a `handle_error_or_what_if(error)` helper that, in what-if mode, turns an error into a projected state whose `_metadata.whatIf` contains the error message, instead of failing the run:

  ```rust
  fn handle_error_or_what_if(&self, error: Error) -> Result<Option<T>, Error> {
      if self.what_if {
          return Ok(Some(T {
              // identity fields from self.config
              metadata: Some(Metadata { what_if: Some(vec![error.to_string()]) }),
              ..Default::default()
          }));
      }
      Err(error)
  }
  ```

- Handle the `_exist: false` delete case inside `set()` by routing through `remove()` (with `what_if` honored), so users get a single what-if message describing the deletion.

### 5. Types (`config.rs` / `types.rs`) changes

Add a `_metadata` field of type `Option<Metadata>` to every public state struct, and define `Metadata` exactly once per crate:

```rust
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "<ResourceName>", deny_unknown_fields)]
pub struct <ResourceName> {
    // ... resource properties ...

    #[serde(rename = "_metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,

    #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
    pub exist: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    #[serde(rename = "whatIf", skip_serializing_if = "Option::is_none")]
    pub what_if: Option<Vec<String>>,
}
```

Naming is fixed: JSON field is `_metadata`, nested array is `whatIf`, Rust field is `what_if: Option<Vec<String>>`.

### 6. Localization strings

Add localized what-if messages to `locales/en-us.toml` under the helper's section, all starting with the verb **"Would"**:

```toml
[<resource>_helper]
whatIfCreate<Thing>  = "Would create %{name}"
whatIfUpdate<Thing>  = "Would update %{name} to '%{value}'"
whatIfDelete<Thing>  = "Would delete %{name} '%{value}'"
```

Examples:

```toml
whatIfCreate<Thing>  = "<Thing> '%{name}' not found, would create it"
whatIfDelete<Thing>  = "Would delete <thing> '%{name}'"
```

Also add `args.configArgsWhatIfHelp = "Run the operation in what-if mode"` (or equivalent) for the clap flag.

### 7. What-if Pester tests

Create one dedicated test file per resource variant. Names are fixed:

- `<resource>.config.whatif.tests.ps1` — single-instance what-if
- `<resource>list_whatif.tests.ps1` — `--list` bulk what-if (only if the resource has a list variant)

Structure:

```powershell
# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe '<resource> config whatif tests' {
    BeforeAll {
        # Ensure a clean starting state
    }

    AfterEach {
        # Roll back anything a test may have created
    }

    It 'Can whatif a new <thing>' -Skip:(!$IsWindows) {
        $json = @'
        { "<key>": "<value>" }
'@
        # 1. Capture pre-state
        $get_before = <resource> config get --input $json 2>$null

        # 2. Run what-if
        $result = <resource> config set -w --input $json 2>$null | ConvertFrom-Json

        # 3. Assert success + projected state + whatIf metadata
        $LASTEXITCODE       | Should -Be 0
        $result.<key>       | Should -Be '<value>'
        $result._metadata.whatIf[0] | Should -Match '.*<expected fragment>.*'

        # 4. Assert NO mutation happened
        $get_after = <resource> config get --input $json 2>$null
        $get_before | Should -EQ $get_after
    }

    It 'Can whatif delete an existing <thing> using _exist is false' -Skip:(!$IsWindows) {
        # ... arrange real state via plain `config set` ...
        $whatif_delete = @'
        { "<key>": "<value>", "_exist": false }
'@
        $result = <resource> config set -w --input $whatif_delete 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result._metadata.whatIf | Should -Match "Would delete .*"
    }

    It 'Can whatif delete an existing <thing>' -Skip:(!$IsWindows) {
        # Same as above, but via the `delete` subcommand:
        $result = <resource> config delete -w --input $whatif_delete 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result._metadata.whatIf | Should -Match "Would delete .*"
        # For delete what-if, payload should only include identity fields (and _metadata)
        ($result.psobject.properties | Where-Object { $_.Name -ne '_metadata' } | Measure-Object).Count |
            Should -Be 1
    }
}
```

Rules for what-if tests:

- Always call the executable directly (`<resource> config set -w --input ...`), **not** via `dsc resource`, so the test pins the CLI contract used by the manifest.
- Use `-w` (not `--what-if`) in tests to lock in the short flag.
- Redirect stderr with `2>$null` to keep test output clean; for failing-test debugging, prefer `2>$testdrive/error.log` + `-Because`.
- Pipe through `ConvertFrom-Json` and assert on `_metadata.whatIf` entries with `Should -Match`.
- Always include at least one assertion that the system state did **not** change (compare `config get` before/after).
- Always include both `set -w` (with and without `_exist: false`) and `delete -w` coverage if the manifest exposes `delete` what-if.
- Top-level `Describe` block uses `-Skip:(!$IsWindows)` for Windows-only resources (or the appropriate platform guard).

### 8. Naming convention summary (do not deviate)

| Concern | Name |
|---|---|
| CLI short flag | `-w` |
| CLI long flag | `--what-if` |
| Clap field | `what_if: bool` |
| Helper field | `what_if: bool` |
| Helper enable method | `enable_what_if()` |
| Manifest arg entry | `{ "whatIfArg": "-w" }` |
| Manifest declaration | `"whatIfReturns": "state"` (on `set`) |
| State field | `_metadata` (`metadata: Option<Metadata>` in Rust) |
| Metadata field | `whatIf` (`what_if: Option<Vec<String>>` in Rust) |
| Locale section | `[<resource>_helper]` |
| Locale key prefix | `whatIfCreate*`, `whatIfUpdate*`, `whatIfDelete*` |
| Locale message style | starts with `"Would "` |
| Error-to-whatif helper | `handle_error_or_what_if(error)` |
| Test file (single) | `<resource>.config.whatif.tests.ps1` |
| Test file (list) | `<resource>list_whatif.tests.ps1` |
| Describe block title | `'<resource> config whatif tests'` |

### 9. Implementation checklist

When asked to add what-if to a new resource, perform these steps in order:

1. Update the resource manifest: add `whatIfArg` to `set` (and `delete`) args, add `whatIfReturns: "state"` to `set`.
2. Add `what_if: bool` to the relevant clap `ConfigSubCommand` variants in `args.rs`.
3. Destructure `what_if` in `main.rs`, call `helper.enable_what_if()`, print projected state JSON.
4. Add `what_if` field, `enable_what_if()` method, and `handle_error_or_what_if()` helper to the resource library struct.
5. Change `set()` / `remove()` to short-circuit OS mutations when `what_if`, accumulate `Vec<String>` of "Would …" messages, return `Some(state)` with `_metadata.whatIf` attached.
6. Add `_metadata: Option<Metadata>` to public state structs and define `Metadata { what_if: Option<Vec<String>> }` with the JSON renames shown above.
7. Add `whatIf*` localized strings under `[<resource>_helper]` and `args.configArgsWhatIfHelp` in `locales/en-us.toml`.
8. Create `<resource>.config.whatif.tests.ps1` (and the list variant if applicable) following the test template; cover create, update, delete-via-`_exist`, and `delete -w`.
9. Build with `./build.ps1 -project <resource_name>` and run the new Pester file.

