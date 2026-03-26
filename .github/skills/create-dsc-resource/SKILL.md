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
