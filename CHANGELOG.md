---
title: "Desired State Configuration changelog"
description: >-
  A log of the changes for releases of DSCv3.
ms.date: 09/06/2023
---

# Changelog

All notable changes to DSCv3 are documented in this file. The format is based on
[Keep a Changelog][m1], and DSCv3 adheres to [Semantic Versioning][m2].

<!-- Meta links -->
[m1]: https://keepachangelog.com/en/1.1.0/
[m2]: https://semver.org/spec/v2.0.0.html

## Unreleased

This section includes a summary of user-facing changes since the last release. For the full list of
changes since the last release, see the [diff on GitHub][unreleased].

<!-- Unreleased comparison link -->
[unreleased]: https://github.com/PowerShell/DSC/compare/v3.0.0-alpha.2...main

<!-- Add entries between releases under the appropriate section heading here  -->

## [v3.0.0-alpha.2][release-v3.0.0-alpha.2] - 2023-09-05

This section includes a summary of changes for the `alpha.2` release. For the full list of changes
in this release, see the [diff on GitHub][compare-v3.0.0-alpha.2].

<!-- Release links -->
[release-v3.0.0-alpha.2]: https://github.com/PowerShell/DSC/releases/tag/v3.0.0-alpha.2 "Link to the DSC v3.0.0-alpha.2 release on GitHub"
[compare-v3.0.0-alpha.2]: https://github.com/PowerShell/DSC/compare/v3.0.0-alpha.1...v3.0.0-alpha.2

### Added

- Implemented functionality for the [dependsOn property of resource instances][01] in configuration
  documents, enabling resource instances to depend on the successful processing of one or more
  other instances in the document.

  <details><summary>Related work items</summary>

  - Issues: [#45][#45]
  - PRs: [#175][#175]

  </details>

- Added the [export][02] property to the resource manifest schema, indicating that the resource is
  exportable and defining how DSC can retrieve the current state for every instance of the
  resource.

  <details><summary>Related work items</summary>

  - Issues: [#73][#73]
  - PRs: [#171][#171]

  </details>

- Added the [dsc config export][03] command to convert an input configuration document defining a
  list of resource types into a usable configuration document that defines the current state for
  every instance of those resources.

  <details><summary>Related work items</summary>

  - Issues: [#73][#73]
  - PRs: [#171][#171]

  </details>

- Added the [dsc resource export][04] command to generate a usable configuration document that
  defines the current state for every instance of a specified resource.

  <details><summary>Related work items</summary>

  - Issues: [#73][#73]
  - PRs: [#171][#171]

  </details>

- Added the [--all][05] option for the [dsc resource get][06] command, enabling users to retrieve
  the current state for every instance of an exportable resource with a single command.

  <details><summary>Related work items</summary>

  - Issues:
    - [#73][#73]
    - [#174][#174]
  - PRs: [#171][#171]

  </details>

- Added handling for the <kbd>Ctrl</kbd>+<kbd>C</kbd> key combination to cancel a DSC operation.
  When `dsc` cancels an operation due to this key-press, it indicates that the operation was
  cancelled with [exit code 6][07].

  <details><summary>Related work items</summary>

  - PRs: [#177][#177]
  - Issues: [#150][#150]

  </details>

- Added support for using the [DSC_RESOURCE_PATH environment variable][08] to define a list of
  folders to search for command-based DSC Resource manifests. When `DSC_RESOURCE_PATH` is defined,
  DSC searches those folders for resources and ignores the `PATH` variable for resource discovery.

  <details><summary>Related work items</summary>

  - PRs: [#176][#176]
  - Issues: [#133][#133]

  </details>

- The `DSC/AssertionGroup`, `DSC/Group`, and `DSC/ParallelGroup` resources now define semantic exit
  codes in their manifests. These resources now indicate that they use the same
  [exit codes as the dsc command][08].

  <details><summary>Related work items</summary>

  - PRs: [#182][#182]
  - Issues: [#181][#181]

  </details>

- Added type validation in the schema for the [defaultValue][09] and [allowedValues][10] properties
  of [configuration document parameters][11] to improve the authoring experience. Now, when a
  parameter defines values for these properties that are incompatible with the defined data type,
  validation raises an error indicating that the values are invalid and why.

  <details><summary>Related work items</summary>

  - PRs: [#172][#172]

  </details>

- Enhanced VS Code-specific schemas for configuration documents and resource manifests to improve
  the authoring experience. The enhanced schemas use keywords only supported by VS Code to:

  - Render Markdown help information for properties and enums.
  - Provide contextual error messages when a value fails pattern validation.
  - Define default snippets to autocomplete values.

  These schemas are non-canonical and should only be used for authoring. For more information, see
  [Authoring with enhanced schemas][12].

  <details><summary>Related work items</summary>

  - PRs: [#172][#172]

  </details>

- Documentation to the [Microsoft/OSInfo][13] resource instance schema and command-line tool to
  provide contextual help about the properties the resource can validate.

  <details><summary>Related work items</summary>

  - PRs: [#168][#168]

  </details>

### Changed

- The [$schema][14] value for configuration documents now points to the canonical published schema
  URI,
  `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json`.

  <details><summary>Related work items</summary>

  - PRs: [#156][#156]

  </details>

### Fixed

- The data-type conditionals for the [configuration parameters][11] schema so that the `min*` and
  `max*` keywords apply to the correct data types. Previously, the logic prevented them from ever
  applying.

  <details><summary>Related work items</summary>

  - PRs: [#172][#172]

  </details>

- Using the `registry find` command no longer raises a panic error due to conflicting option
  definitions on the command.

  <details><summary>Related work items</summary>

  - PRs: [#163][#163]

  </details>

## [v3.0.0-alpha.1][release-v3.0.0-alpha.1] - 2023-08-04

This is the first public release of DSC v3. Consider this release alpha quality. Use it only for
development evaluation, as it has known issues and isn't feature complete.

For the full list of changes in this release, see the [diff on GitHub][compare-v3.0.0-alpha.1].

<!-- Release comparison link -->
[release-v3.0.0-alpha.1]: https://github.com/PowerShell/DSC/releases/tag/v3.0.0-alpha.1 "Link to the DSC v3.0.0-alpha.1 release on GitHub"
[compare-v3.0.0-alpha.1]: https://github.com/PowerShell/DSC/compare/6090b1464bbf81fded5453351708482a4db35258...v3.0.0-alpha.1

<!-- alpha.2 links -->
[01]: docs/reference/schemas/config/resource.md#dependson
[02]: docs/reference/schemas/resource/manifest/export.md
[03]: docs/reference/cli/config/export.md
[04]: docs/reference/cli/resource/export.md
[05]: docs/reference/cli/resource/get.md##a---all
[06]: docs/reference/cli/resource/get.md
[07]: docs/reference/cli/dsc.md#exit-codes
[08]: docs/reference/cli/dsc.md#environment-variables
[09]: docs/reference/schemas/config/parameter.md#defaultvalue
[10]: docs/reference/schemas/config/parameter.md#allowedvalues
[11]: docs/reference/schemas/config/parameter.md
[12]: https://learn.microsoft.com/powershell/dsc/concepts/enhanced-authoring?view=dsc-3.0&preserve-view=true
[13]: https://learn.microsoft.com/powershell/dsc/reference/microsoft/osinfo/resource?view=dsc-3.0&preserve-view=true
[14]: docs/reference/schemas/config/document.md#schema

<!-- Issue and PR links -->
[#163]: https://github.com/PowerShell/DSC/163
[#156]: https://github.com/PowerShell/DSC/156
[#168]: https://github.com/PowerShell/DSC/168
[#172]: https://github.com/PowerShell/DSC/172
[#182]: https://github.com/PowerShell/DSC/182
[#181]: https://github.com/PowerShell/DSC/181
[#177]: https://github.com/PowerShell/DSC/177
[#150]: https://github.com/PowerShell/DSC/150
[#176]: https://github.com/PowerShell/DSC/176
[#133]: https://github.com/PowerShell/DSC/133
[#73]:  https://github.com/PowerShell/DSC/73
[#174]: https://github.com/PowerShell/DSC/174
[#171]: https://github.com/PowerShell/DSC/171
[#45]:  https://github.com/PowerShell/DSC/45
[#175]: https://github.com/PowerShell/DSC/175
