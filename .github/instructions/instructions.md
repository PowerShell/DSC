# DSC Build and Test Instructions

This document provides comprehensive instructions for building and testing the DSC (Desired State Configuration) v3 project. These instructions are intended for contributors and maintainers to validate changes before submitting pull requests.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Building the Project](#building-the-project)
- [Running Tests](#running-tests)
- [Linting](#linting)
- [Creating Packages](#creating-packages)
- [CI/CD Workflows](#cicd-workflows)
- [Troubleshooting](#troubleshooting)

## Prerequisites

The DSC project is built using Rust and PowerShell. The build system automatically installs most dependencies, but you need the following installed on your system:

### Required

1. **PowerShell**: Version 7.2 or later (cross-platform)
   - Download from: https://github.com/PowerShell/PowerShell
   - The build script is invoked with `pwsh`

2. **Rust**: Latest stable version
   - The build script can install/update Rust automatically
   - Manual installation: https://rustup.rs/

### Automatically Installed by Build Script

The `build.ps1` script automatically installs or verifies the following dependencies:

- **Clippy**: Rust linting tool (when using `-Clippy` flag)
- **Node.js**: Required for tree-sitter grammar generation
- **tree-sitter**: For parsing DSC expression grammars
- **Windows C++ Build Tools**: For Windows builds (when needed)

### Platform-Specific Requirements

#### Windows
- Visual Studio 2019 or later with C++ build tools (automatically checked)
- MakeAppx.exe (for creating MSIX packages)

#### Linux
- GCC or Clang
- Standard development tools (make, pkg-config)

#### macOS
- Xcode Command Line Tools

## Quick Start

For a quick build and test on your current platform:

```powershell
# Build the project in debug mode
./build.ps1

# Build with linting (recommended)
./build.ps1 -Clippy

# Build and run all tests
./build.ps1 -Clippy -Test

# Build in release mode (optimized)
./build.ps1 -Release
```

## Building the Project

The main build script is `build.ps1`, which orchestrates the entire build process.

### Basic Build

```powershell
# Debug build (default)
./build.ps1

# Release build (optimized, slower compile time)
./build.ps1 -Release
```

### Build with Linting

It's recommended to lint before building to catch issues early:

```powershell
# Build with Clippy linting
./build.ps1 -Clippy

# Build release version with linting
./build.ps1 -Clippy -Release
```

### Cross-Platform Builds

Build for a specific architecture:

```powershell
# Windows on ARM
./build.ps1 -Architecture aarch64-pc-windows-msvc

# Windows on x64
./build.ps1 -Architecture x86_64-pc-windows-msvc

# macOS on ARM (Apple Silicon)
./build.ps1 -Architecture aarch64-apple-darwin

# macOS on x64 (Intel)
./build.ps1 -Architecture x86_64-apple-darwin

# Linux on ARM with glibc
./build.ps1 -Architecture aarch64-unknown-linux-gnu

# Linux on x64 with glibc
./build.ps1 -Architecture x86_64-unknown-linux-gnu

# Linux on x64 with musl (static linking)
./build.ps1 -Architecture x86_64-unknown-linux-musl
```

### Build Specific Projects

```powershell
# Build only specific projects
./build.ps1 -Project dsc,y2j
```

### Clean Build

```powershell
# Clean and rebuild
./build.ps1 -Clean
```

### Skip Build (Useful for Testing Only)

```powershell
# Skip building, only run tests
./build.ps1 -SkipBuild -Test
```

### Build Artifacts

After a successful build, artifacts are located in:
- **bin/**: Compiled binaries and executables
- **target/**: Rust build artifacts (debug or release)

## Running Tests

The DSC project includes two types of tests:

1. **Rust Tests**: Unit and integration tests written in Rust
2. **Pester Tests**: End-to-end tests written in PowerShell using Pester framework

### Run All Tests

```powershell
# Build and run all tests
./build.ps1 -Test

# Run all tests with linting
./build.ps1 -Clippy -Test
```

### Run Only Rust Tests

```powershell
# Build and run Rust tests only
./build.ps1 -Test -ExcludePesterTests

# Skip build, run Rust tests only
./build.ps1 -SkipBuild -Test -ExcludePesterTests
```

### Run Only Pester Tests

```powershell
# Build and run Pester tests only
./build.ps1 -Test -ExcludeRustTests

# Skip build, run Pester tests only
./build.ps1 -SkipBuild -Test -ExcludeRustTests
```

### Run Specific Pester Test Groups

Pester tests are organized into groups:

```powershell
# Run tests for DSC CLI
./build.ps1 -SkipBuild -Test -ExcludeRustTests -PesterTestGroup dsc

# Run tests for adapters
./build.ps1 -SkipBuild -Test -ExcludeRustTests -PesterTestGroup adapters

# Run tests for extensions
./build.ps1 -SkipBuild -Test -ExcludeRustTests -PesterTestGroup extensions

# Run tests for resources
./build.ps1 -SkipBuild -Test -ExcludeRustTests -PesterTestGroup resources

# Run tests for grammars
./build.ps1 -SkipBuild -Test -ExcludeRustTests -PesterTestGroup grammars

# Run multiple test groups
./build.ps1 -SkipBuild -Test -ExcludeRustTests -PesterTestGroup dsc,resources
```

### Test Documentation

```powershell
# Generate and test Rust documentation
./build.ps1 -RustDocs
./build.ps1 -SkipBuild -RustDocs -Test -ExcludeRustTests -ExcludePesterTests
```

## Linting

### Clippy (Rust Linter)

Clippy is the recommended linter for Rust code:

```powershell
# Lint and build
./build.ps1 -Clippy

# Lint without building (faster)
./build.ps1 -SkipBuild -Clippy
```

Clippy checks are enforced in CI and must pass before merging pull requests.

### Security Audit

Run cargo audit to check for security vulnerabilities:

```powershell
./build.ps1 -Audit
```

## Creating Packages

The build script can create distribution packages for different platforms.

### Prerequisites for Packaging

You must specify a specific architecture (not `current`) when packaging.

### Package Types

#### Windows Packages

```powershell
# Create MSIX package (Windows only)
./build.ps1 -PackageType msix -Architecture x86_64-pc-windows-msvc -Release

# Create MSIX private package
./build.ps1 -PackageType msix-private -Architecture x86_64-pc-windows-msvc -Release

# Create MSIX bundle (builds both x64 and ARM64)
./build.ps1 -PackageType msixbundle -Release

# Create ZIP package
./build.ps1 -PackageType zip -Architecture x86_64-pc-windows-msvc -Release
```

#### Linux/macOS Packages

```powershell
# Create tar.gz package for Linux
./build.ps1 -PackageType tgz -Architecture x86_64-unknown-linux-gnu -Release

# Create tar.gz package for macOS
./build.ps1 -PackageType tgz -Architecture aarch64-apple-darwin -Release
```

### Get Package Version

```powershell
# Get the current version from Cargo.toml
./build.ps1 -GetPackageVersion
```

## CI/CD Workflows

The project uses GitHub Actions for continuous integration. Workflows are defined in `.github/workflows/`.

### Main Workflow: rust.yml

The `rust.yml` workflow runs on every push and pull request to `main` and `release/*` branches.

#### Workflow Jobs

1. **docs**: Generates and tests Rust documentation on all platforms (Ubuntu, macOS, Windows)
   - Installs prerequisites with Clippy
   - Generates Rust documentation
   - Tests documentation examples

2. **linux-build**: Builds on Ubuntu
   - Installs prerequisites
   - Builds with Clippy linting
   - Runs Rust tests
   - Uploads build artifacts

3. **linux-pester**: Runs Pester tests on Ubuntu (depends on linux-build)
   - Matrix strategy for test groups: dsc, adapters, extensions, resources
   - Downloads build artifacts
   - Runs specific test groups

4. **macos-build**: Builds on macOS
   - Same steps as linux-build

5. **macos-pester**: Runs Pester tests on macOS (depends on macos-build)
   - Same matrix strategy as linux-pester

6. **windows-build**: Builds on Windows
   - Same steps as linux-build

7. **windows-pester**: Runs Pester tests on Windows (depends on windows-build)
   - Same matrix strategy as linux-pester

#### Simulating CI Locally

To simulate the CI workflow locally:

```powershell
# Install prerequisites and build with Clippy (matches CI)
./build.ps1 -SkipBuild -Clippy -Verbose
./build.ps1 -Clippy -Verbose

# Run Rust tests (matches CI)
./build.ps1 -SkipBuild -Test -ExcludePesterTests -Verbose

# Run Pester tests for a specific group (matches CI)
./build.ps1 -SkipBuild -Test -ExcludeRustTests -PesterTestGroup dsc -Verbose

# Test documentation (matches CI docs job)
./build.ps1 -RustDocs -Verbose
./build.ps1 -SkipBuild -RustDocs -Test -ExcludeRustTests -ExcludePesterTests -Verbose
```

### Winget Workflow: winget.yml

The `winget.yml` workflow publishes releases to Windows Package Manager (WinGet):
- Triggered on release publication or manual workflow dispatch
- Only processes stable releases (not pre-releases)
- Creates WinGet package submission

## Troubleshooting

### Common Issues

#### Rust Not Found

If Rust is not installed:
```powershell
# The build script will attempt to install Rust automatically
# Or manually install from: https://rustup.rs/
```

#### Build Fails on Windows

Ensure Visual Studio C++ build tools are installed:
```powershell
# The build script checks and provides guidance
./build.ps1 -Verbose
```

#### Tests Fail

Check that the build completed successfully:
```powershell
# Ensure clean build before testing
./build.ps1 -Clean
./build.ps1 -Clippy -Test
```

#### Node.js or tree-sitter Missing

The build script automatically installs these. If issues persist:
```powershell
# Re-run with verbose output
./build.ps1 -Verbose
```

### Build Script Parameters Reference

| Parameter | Type | Description |
|-----------|------|-------------|
| `-Release` | Switch | Build with release optimizations (slower compile, faster runtime) |
| `-Architecture` | String | Target architecture (current, aarch64-pc-windows-msvc, x86_64-pc-windows-msvc, aarch64-apple-darwin, x86_64-apple-darwin, aarch64-unknown-linux-gnu, aarch64-unknown-linux-musl, x86_64-unknown-linux-gnu, x86_64-unknown-linux-musl) |
| `-Clippy` | Switch | Lint Rust code with Clippy before building |
| `-SkipBuild` | Switch | Skip building (useful for testing only) |
| `-Test` | Switch | Run tests after building |
| `-GetPackageVersion` | Switch | Output current package version |
| `-PackageType` | String | Create package (msix, msix-private, msixbundle, tgz, zip) |
| `-Project` | String[] | Build specific projects only |
| `-ExcludeRustTests` | Switch | Skip Rust tests (use with -Test) |
| `-ExcludePesterTests` | Switch | Skip Pester tests (use with -Test) |
| `-PesterTestGroup` | String[] | Run specific Pester test groups (dsc, adapters, extensions, grammars, resources) |
| `-SkipLinkCheck` | Switch | Skip Windows C++ link tools check |
| `-UseX64MakeAppx` | Switch | Use x64 version of MakeAppx.exe |
| `-UseCFS` | Switch | Use Component Framework Service |
| `-UpdateLockFile` | Switch | Update Cargo.lock file |
| `-Audit` | Switch | Run cargo audit for security vulnerabilities |
| `-UseCFSAuth` | Switch | Use CFS authentication |
| `-Clean` | Switch | Clean build artifacts before building |
| `-CacheRustBuild` | Switch | Cache Rust build artifacts |
| `-RustDocs` | Switch | Generate Rust documentation |
| `-Quiet` | Switch | Suppress verbose output |

### Verbose Output

For detailed build information:
```powershell
./build.ps1 -Verbose
```

### Getting Help

- Review error messages from the build script
- Check GitHub Issues: https://github.com/PowerShell/DSC/issues
- See CONTRIBUTING.md for contribution guidelines

## Validating Changes

Before submitting a pull request, validate your changes with the following commands:

```powershell
# 1. Clean build with linting
./build.ps1 -Clean -Clippy -Release

# 2. Run all tests
./build.ps1 -SkipBuild -Test

# 3. Check for security vulnerabilities
./build.ps1 -Audit

# 4. Generate and test documentation (optional)
./build.ps1 -RustDocs
./build.ps1 -SkipBuild -RustDocs -Test -ExcludeRustTests -ExcludePesterTests
```

This ensures your changes:
- Build successfully with optimizations
- Pass all linting checks
- Pass all tests
- Have no known security vulnerabilities
- Have valid documentation

## Additional Resources

- **Main README**: [README.md](README.md)
- **Contributing Guide**: [CONTRIBUTING.md](CONTRIBUTING.md)
- **Documentation**: [docs/](docs/)
- **Build Helper Module**: [build.helpers.psm1](build.helpers.psm1)
- **Legacy Build Script**: [packaging.ps1](packaging.ps1) (deprecated, use `build.ps1`)
