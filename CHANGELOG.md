---
title: "Desired State Configuration changelog"
description: >-
  A log of the changes for releases of DSCv3.
ms.date: 11/15/2023
---

# Changelog

<!-- markdownlint-disable-file MD033 -->

All notable changes to DSCv3 are documented in this file. The format is based on
[Keep a Changelog][m1], and DSCv3 adheres to [Semantic Versioning][m2].

<!-- Meta links -->
[m1]: https://keepachangelog.com/en/1.1.0/
[m2]: https://semver.org/spec/v2.0.0.html

## Unreleased

This section includes a summary of user-facing changes since the last release. For the full list of
changes since the last release, see the [diff on GitHub][unreleased].

<!-- Unreleased comparison link -->
[unreleased]: https://github.com/PowerShell/DSC/compare/v3.0.0-alpha.3...main

<!-- Add entries between releases under the appropriate section heading here  -->

### Changed

- Updated the canonical version of the schema URIs from `2023/08` to `2023/10`, as this release
  includes breaking changes for the schemas.

  As part of this change, the `$schema` keyword for both [configuration documents][21] and
  [resource manifests][22] accepts any valid URI for the schemas, instead of only one. Now, you
  can set the value for the keyword to the unbundled schema, the bundled schema, or the enhanced
  authoring schema for any supported version.

- Replaced the `_ensure` well-known property with the boolean [_exist][23] property. This improves
  the semantics for users and simplifies implementation for resources, replacing the string enum
  values `Present` and `Absent` with `true` and `false` respectively.

  <details><summary>Related work items</summary>

  - Issues: [#202][#202]
  - PRs: [#206][#206]

  </details>

- Updated the `Microsoft.Windows/Registry` resource to use the `_exist` property instead of
  `_ensure` and updated the output to be idiomatic for a DSC Resource.

  <details><summary>Related work items</summary>

  - Issues: [#162][#162]
  - PRs: [#206][#206]

  </details>

- When a user presses the <kbd>Ctrl</kbd>+<kbd>C</kbd> key combination, DSC now recursively
  terminates all child processes before exiting. This helps prevent dangling processes that were
  previously unhandled by the interrupt event.

  <details><summary>Related work items</summary>

  - PRs: [#213][#213]

  </details>

### Added

- Added the [--input][24] and [--input-file][25] global options to the root `dsc` command. Now, you
  can pass input to DSC from a variable or file instead of piping from stdin.

  <details><summary>Related work items</summary>

  - Issues: [#130][#130]
  - PRs: [#217][#217]

  </details>

- Added the `arg` value as an option for defining how a command-based DSC Resource expects to
  receive input. This enables resource authors to define resources that handle DSC passing the
  instance JSON as an argument.

  <details><summary>Related work items</summary>

  - PRs: [#213][#213]

  </details>

- Added the new [completer][26] command enables users to add shell completions for DSC to their
  shell. The command supports completions for Bash, Elvish, fish, PowerShell, and ZSH.

  <details><summary>Related work items</summary>

  - Issues: [#186][#186]
  - PRs: [#216][#216]

  </details>

- DSC now emits log messages to the stderr stream. This can make it easier to understand what DSC
  is doing. This doesn't affect the data output. By default, DSC emits errors, warnings, and
  informational messages, but not debug or trace messaging. You can control the level of the
  logging with the new [--logging-level][27] option on the root `dsc` command.

  <details><summary>Related work items</summary>

  - Issues:
    - [#107][#107]
    - [#158][#158]
  - PRs:
    - [#211][#211]
    - [#248][#248]

  </details>

- Added optimizations for the resource discovery process that runs before most `dsc` commands.
  These optimizations significantly reduce the command execution duration, especially for the
  `dsc resource *` commands, which rarely need to run a full discovery for resources.

  <details><summary>Related work items</summary>

  - Issues: [#173][#173]
  - PRs: [#240][#240]

  </details>

- Added initial [configuration document functions][28] to DSC. You can now use the [base64()][29],
  [concat()][30], and [resourceId()][31] functions in the configuration document.

  > [!NOTE]
  > The `resourceId` function has been reimplemented as a document function instead of a special
  > case, but it has the same functionality and parameters.

  <details><summary>Related work items</summary>

  - Issues: [#57][#57]
  - PRs:
    - [#241][#241]
    - [#252][#252]

  </details>

### Fixed

- The `--format` option now works as users expect when the output is redirected or saved to a
  variable. Before this fix, DSC always returned JSON output, even when the user wanted to save
  the output as YAML. With this fix, the specified format is respected.

  <details><summary>Related work items</summary>

  - PRs: [#215][#215]

  </details>

- The `DSC/PowerShellGroup` resource now correctly returns the _labels_ for enumerations instead of
  their integer value, making it easier to understand and compare results.

  <details><summary>Related work items</summary>

  - Issues: [#159][#159]
  - PRs: [#208][#208]

  </details>

- DSC no longer terminates during discovery when a resource errors unless the erroring resource is
  being used for the command. DSC still terminates on a resource error during discovery under the
  following conditions:

  - When the erroring resource type is the same as the value of the `--resource` option for a
    `dsc resource *` command.
  - When an instance in the configuration document uses the erroring resource type for a
    `dsc config *` command.

  DSC emits the resource errors during discovery as warning messages for the `dsc resource list`
  command. In all other cases, DSC emits the errors as debug messages.

  <details><summary>Related work items</summary>

  - Issues: [#121][#121]
  - PRs: [#240][#240]

  </details>

## [v3.0.0-alpha.3][release-v3.0.0-alpha.3] - 2023-09-26

This section includes a summary of changes for the `alpha.3` release. For the full list of changes
in this release, see the [diff on GitHub][compare-v3.0.0-alpha.3].

<!-- Release links -->
[release-v3.0.0-alpha.3]: https://github.com/PowerShell/DSC/releases/tag/v3.0.0-alpha.3 "Link to the DSC v3.0.0-alpha.3 release on GitHub"
[compare-v3.0.0-alpha.3]: https://github.com/PowerShell/DSC/compare/v3.0.0-alpha.2...v3.0.0-alpha.3

### Changed

- Replaced the `manifestVersion` property for resource manifests with [$schema][15]. Instead of
  specifying a semantic version, resources need to indicate which canonical schema DSC should use
  to validate and process the manifest.

  <details><summary>Related work items</summary>

  - Issues: [#127][#127]
  - PRs: [#199][#199]

  </details>

- Updated the `preTest` property for the `set` command in resource manifests to
  [implementsPretest][16] to more make the manifest easier to read and understand.

  <details><summary>Related work items</summary>

  - PRs: [#197][#197]

  </details>

- The [dsc resource set][17] command no longer tests the resource instance before invoking the
  `set` operation. This simplifies the behavior for the command and adheres more accurately to the
  implied contract for directly invoking a resource with DSC.

  <details><summary>Related work items</summary>

  - Issues: [#98][#98]
  - PRs: [#197][#197]

  </details>

- Replaced the `args` option with `env` for defining how a command-based resource expects to
  receive input for the [get][18], [set][19], and [test][20] commands in the resource manifest.

  The `args` option was never implemented. Instead, resource authors can set the `input` property
  to `env` to indicate that the resource expects input as environmental variables.

  <details><summary>Related work items</summary>

  - PRs: [#198][#198]

  </details>

- The `input` property for the [get][18] command in a resource manifest no longer has a default
  value. Instead, when a resource doesn't define `input` for the `get` command, DSC doesn't send
  any input to the resource for that command.

  <details><summary>Related work items</summary>

  - PRs: [#198][#198]

  </details>

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

<!-- alpha.3 links -->
[15]: docs/reference/schemas/resource/manifest/root.md#schema
[16]: docs/reference/schemas/resource/manifest/set.md#implementspretest
[17]: docs/reference/cli/resource/set.md
[18]: docs/reference/schemas/resource/manifest/get.md#input
[19]: docs/reference/schemas/resource/manifest/set.md#input
[20]: docs/reference/schemas/resource/manifest/test.md#input

<!-- alpha.4 links -->
[21]: docs/reference/schemas/config/document.md#schema
[22]: docs/reference/schemas/resource/manifest/root.md#schema
[23]: docs/reference/schemas/resource/properties/exist.md
[24]: docs/reference/cli/dsc.md#-i---input
[25]: docs/reference/cli/dsc.md#-p---input-file
[26]: docs/reference/cli/completer/command.md
[27]: docs/reference/cli/dsc.md#-l---logging-level
[28]: docs/reference/schemas/config/functions/overview.md
[29]: docs/reference/schemas/config/functions/base64.md
[30]: docs/reference/schemas/config/functions/concat.md
[31]: docs/reference/schemas/config/functions/resourceId.md

<!-- Issue and PR links -->
[#107]: https://github.com/PowerShell/DSC/issues/107
[#121]: https://github.com/PowerShell/DSC/issues/121
[#127]: https://github.com/PowerShell/DSC/issues/127
[#130]: https://github.com/PowerShell/DSC/issues/130
[#133]: https://github.com/PowerShell/DSC/issues/133
[#150]: https://github.com/PowerShell/DSC/issues/150
[#156]: https://github.com/PowerShell/DSC/issues/156
[#158]: https://github.com/PowerShell/DSC/issues/158
[#159]: https://github.com/PowerShell/DSC/issues/159
[#162]: https://github.com/PowerShell/DSC/issues/162
[#163]: https://github.com/PowerShell/DSC/issues/163
[#168]: https://github.com/PowerShell/DSC/issues/168
[#171]: https://github.com/PowerShell/DSC/issues/171
[#172]: https://github.com/PowerShell/DSC/issues/172
[#173]: https://github.com/PowerShell/DSC/issues/173
[#174]: https://github.com/PowerShell/DSC/issues/174
[#175]: https://github.com/PowerShell/DSC/issues/175
[#176]: https://github.com/PowerShell/DSC/issues/176
[#177]: https://github.com/PowerShell/DSC/issues/177
[#181]: https://github.com/PowerShell/DSC/issues/181
[#182]: https://github.com/PowerShell/DSC/issues/182
[#186]: https://github.com/PowerShell/DSC/issues/186
[#197]: https://github.com/PowerShell/DSC/issues/197
[#198]: https://github.com/PowerShell/DSC/issues/198
[#199]: https://github.com/PowerShell/DSC/issues/199
[#202]: https://github.com/PowerShell/DSC/issues/202
[#206]: https://github.com/PowerShell/DSC/issues/206
[#208]: https://github.com/PowerShell/DSC/issues/208
[#211]: https://github.com/PowerShell/DSC/issues/211
[#213]: https://github.com/PowerShell/DSC/issues/213
[#215]: https://github.com/PowerShell/DSC/issues/215
[#216]: https://github.com/PowerShell/DSC/issues/216
[#217]: https://github.com/PowerShell/DSC/issues/217
[#240]: https://github.com/PowerShell/DSC/issues/240
[#241]: https://github.com/PowerShell/DSC/issues/241
[#248]: https://github.com/PowerShell/DSC/issues/248
[#252]: https://github.com/PowerShell/DSC/issues/252
[#45]:  https://github.com/PowerShell/DSC/issues/45
[#57]:  https://github.com/PowerShell/DSC/issues/57
[#73]:  https://github.com/PowerShell/DSC/issues/73
[#98]:  https://github.com/PowerShell/DSC/issues/98
