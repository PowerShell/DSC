# Repository instructions

## Building

- Always use `./build.ps1` from the repository root to build or compile the project.
- Do not invoke Cargo directly for build validation, including `cargo build` or `cargo check`.
- For targeted builds, use `./build.ps1 -Project <project-name>` instead of a package-scoped Cargo command.
- Use the relevant `build.ps1` switches for linting, testing, release builds, documentation, auditing, and other supported validation.
