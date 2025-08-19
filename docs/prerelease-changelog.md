---
title: "Desired State Configuration changelog"
description: >-
  A log of the changes for releases of DSCv3.
ms.topic: whats-new
ms.date: 06/24/2024
---

# Changelog

<!-- markdownlint-disable-file MD033 -->

<!--
    Helpful docs snippets

    You can use the following snippets in this file to make authoring new change entries and
    releases easier.

    docs-changelog-entry-single-issue-pr: Adds a new changelog list entry with one related issue
    and PR. Use this when a change entry only has one related issue and/or pull request. Always
    use this snippet under a change kind heading, like 'Added' or 'Changed.'

    docs-changelog-entry-multi-issue-pr: Adds a new changelog list entry with sub-lists for issues
    and PRs. Use this when a change entry has more than one related issue and/or pull request.
    Always use this snippet under a change kind heading, like 'Added' or 'Changed.'

    docs-changelog-release-heading: Adds a new changelog release heading, following our existing
    format. Use this when a new release is created to ensure that the new release heading has the
    required links and synopsis.

    docs-gh-link: Adds a new link to an issue or pull request on GitHub. Use this when adding a new
    work item link reference and it will automatically construct the URL and reference link ID for
    you from the work item ID.
-->

All notable changes to DSCv3 are documented in this file. The format is based on
[Keep a Changelog][m1], and DSCv3 adheres to [Semantic Versioning][m2].

<!-- Meta links -->
[m1]: https://keepachangelog.com/en/1.1.0/
[m2]: https://semver.org/spec/v2.0.0.html

## Unreleased

This section includes a summary of user-facing changes since the last release. For the full list of
changes since the last release, see the [diff on GitHub][unreleased].

<!-- Unreleased comparison link - always update version to match last release tag-->
[unreleased]: https://github.com/PowerShell/DSC/compare/v3.0.0-preview.11...main

<!--
    Unreleased change entry instructions:

    Add entries between releases under the appropriate section heading here. When you need to add
    a change, make sure it's under one of the H3s, not this H2. Use one of the following snippets
    to add the change entry:

    - docs-changelog-entry-single-issue-pr: Adds a new changelog list entry with one related issue
      and PR. Use this when a change entry only has one related issue and/or pull request.

    - docs-changelog-entry-multi-issue-pr: Adds a new changelog list entry with sub-lists for
      issues and PRs. Use this when a change entry has more than one related issue and/or pull
      request.

    When you're ready to update the unreleased changelog entries for a new release, use the
    docs-changelog-release-heading snippet to create the new release heading after this comment and
    before the first H3 for the changes.

    After doing so, rename the unreleased reference links from `ur-##` to `<prefix>-##`, where
    <prefix> is a two-character prefix for the release. For alpha releases, we use `a#`, like `a5`
    for the `v3.0.0.0-alpha.5` release. Leave the release links under the release section.
-->

<!-- Unreleased change links -->

## [v3.0.0-preview.11][release-v3.0.0-preview.11] - 2024-10-24

This section includes a summary of changes for the `preview.11` release. For the full list of changes
in this release, see the [diff on GitHub][compare-v3.0.0-preview.11].

<!-- Release links -->
[release-v3.0.0-preview.11]: https://github.com/PowerShell/DSC/releases/tag/v3.0.0-preview.11 "Link to the DSC v3.0.0-preview.11 release on GitHub"
[compare-v3.0.0-preview.11]: https://github.com/PowerShell/DSC/compare/v3.0.0-preview.10...v3.0.0-preview.11

### Changed

- Renamed the resource `kind` value `Import` to `Importer`. If your resource manifest sets the
  `kind` property to `Import`, update it to `Importer`.

  <details><summary>Related work items</summary>

  - Issues:[#436][#436].
  - PRs: [#552][#552]

  </details>

- Improved performance for the adapter lookup table used to resolve adapted resources, reducing
  lookup overhead during operations. See also [dsc resource list][p10-aa].

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#568][#568]

  </details>

- `dsc` now returns a non-zero exit code when a requested resource isn't found, making failures
  easier to detect in automation.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#561][#561]

  </details>

- Changed the Echo test resource to `Microsoft.DSC.Debug/Echo`.

  <details><summary>Related work items</summary>

  - Issues: [#537][#537].
  - PRs: [#553][#553]

  </details>

### Added

- Added example configurations for common Windows baselines to help users get started.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#572][#572]

  </details>

### Fixed

- Class-based PowerShell DSC Resources no longer include hidden properties in their output.

  <details><summary>Related work items</summary>

  - Issues: [#157][#157].
  - PRs: [#556][#556]

  </details>

- Improved trace messaging for the `Microsoft.Windows/Registry` resource, emitting messages at the
  proper levels.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#555][#555]

  </details>

- Fixed input schema validation for the `Microsoft.Windows/RebootPending` resource.

  <details><summary>Related work items</summary>

  - Issues: [#485][#485].
  - PRs: [#488][#488]

  </details>

- Fixed a regression in the PowerShell adapter `Test` operation.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#565][#565]

  </details>

## [v3.0.0-preview.10][release-v3.0.0-preview.10] - 2024-09-17

This section includes a summary of changes for the `preview.10` release. For the full list of changes
in this release, see the [diff on GitHub][compare-v3.0.0-preview.10].

<!-- Release links -->
[release-v3.0.0-preview.10]: https://github.com/PowerShell/DSC/releases/tag/v3.0.0-preview.10 "Link to the DSC v3.0.0-preview.10 release on GitHub"
[compare-v3.0.0-preview.10]: https://github.com/PowerShell/DSC/compare/v3.0.0-preview.9...v3.0.0-preview.10

### Changed

- The WMI adapter now treats instance properties as query properties. Prior to this change, adapted
  instances would return every property. Starting with this release, only properties defined in the
  instance declaration are returned. If an instance property is defined with a value, the adapter
  uses that property and value to filter the instance.

  <details><summary>Related work items</summary>

  - Issues: [#475][#475].
  - PRs: [#548][#548]

  </details>

### Added

- Added capability for users to specify expressions when indexing into arrays for configuration
  functions.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#527][#527]

  </details>

- Added a lookup table to improve performance when invoking adapted resources. DSC uses this table
  to avoid needing to enumerate all adapted resources for non-list operations where possible. For
  more information, see [dsc resource list][p10-aa].

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#530][#530]

  </details>

### Fixed

- Fixed a bug in the tree-sitter grammar preventing use of multiline strings and escaped single
  quotes in configuration functions.

  <details><summary>Related work items</summary>

  - Issues: [#518][#518]
  - PRs: [#524][#524]

  </details>

- Fixed trace messaging for the `Microsoft.DSC/PowerShell` and `Microsoft.Windows/PowerShell`
  adapters to correctly emit warning and error messages instead of emitting all messages as debug.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#528][#528]

  </details>

- Fixed error messages for the `Microsoft.DSC/PowerShell` and `Microsoft.Windows/PowerShell`
  adapters to clarify the actual error instead of returning a generic message.

  <details><summary>Related work items</summary>

  - Issues: [#516][#516]
  - PRs: [#525][#525]

  </details>

- Fixed the check for caching in the `Microsoft.DSC/PowerShell` and `Microsoft.Windows/PowerShell`
  adapters to check on whole seconds instead of fractional seconds, reducing the frequency of
  unneccessary cache invalidation.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#533][#533]

  </details>

- Fixed behavior for built-in resources to correctly handle trace messaging for nested calls to
  `dsc`.

  <details><summary>Related work items</summary>

  - Issues: [#512][#512]
  - PRs: [#541][#541]

  </details>

[p10-aa]: docs/reference/cli/resource/list.md#adapted-resource-cache

## [v3.0.0-preview.9][release-v3.0.0-preview.9] - 2024-08-15

This section includes a summary of changes for the `preview.9` release. For the full list of changes
in this release, see the [diff on GitHub][compare-v3.0.0-preview.9].

<!-- Release links -->
[release-v3.0.0-preview.9]: https://github.com/PowerShell/DSC/releases/tag/v3.0.0-preview.9 "Link to the DSC v3.0.0-preview.9 release on GitHub"
[compare-v3.0.0-preview.9]: https://github.com/PowerShell/DSC/compare/v3.0.0-alpha.4...v3.0.0-preview.9

### Removed

- Removed the `url` sub-property from the `schema` property in resource manifests. Starting with
  this release, resources must either embed their instance property JSON schema in the manifest or
  define the command that returns the JSON schema for validation.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#457][#457]

  </details>

### Changed

- Changed the invocation for resources from synchronous to asynchronous. Starting with this
  release, resource invocations are handled asynchronously. This reduced errors related to
  processing and laid the groundwork for real-time progress reporting.

  <details><summary>Related work items</summary>

  - Issues: [#491][#491]
  - PRs: [#493][#493]

  </details>

- Changed the `import` resource type to function as a group resource. This resource instances
  resolved from import to be used correctly for all operations.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#500][#500]

  </details>

- Changed the inserted property indicating the fully qualified type for an adapted resource from
  `type` to `adapted_dsc_type`. Prior to this release, DSC forwarded the information about adapted
  resource instances to the adapters by inserting the `type` property into the property bag for the
  instance, which had the potential to cause conflicts with actual resource properties named
  `type`. This change reduces the probability of conflicts by renaming the inserted property to the
  more explicit `adapted_dsc_type`.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#482][#482]

  </details>

### Added

- Added support for using variables in a configuration document. Prior to this release, variables
  could be defined in the document but not referenced from resource instances with a configuration
  function. This release includes the new `variables()` configuration function. For more
  information, see the [reference documentation][p9-01].

  <details><summary>Related work items</summary>

  - Issues: [#57][#57]
  - PRs: [#511][#511]

  </details>

- Added support for indexing into arrays when using configuration functions. This enables users to
  access specific items in an array of values returned by a configuration function, such as whe
  referencing the output of a resource. For more information about configuration functions, see
  [DSC Configuration document functions reference][p9-02]. For a detailed example showing how to
  access items in an array, see [Example 4][p9-03].

  <details><summary>Related work items</summary>

  - Issues: [#509][#509]
  - PRs: [#514][#514]

  </details>

- Added handling to ensure that the folder containing `dsc` is always searched for resources. This
  enables users to find and use built-in resources without manually updating their `PATH`
  environment variable. This change has no effect when the `DSC_RESOURCE_PATH` environment variable
  is defined.

  <details><summary>Related work items</summary>

  - Issues: [#494][#494]
  - PRs: [#499][#499]

  </details>

- Added support for PSDSC resources defined as derived classes. Prior to this release, the adapter
  didn't support invoking derived classes as resources.

  <details><summary>Related work items</summary>

  - Issues: [#462][#462]
  - PRs: [#469][#469]

  </details>

- Added the option to clear the PowerShell adapter caches with the `ClearCache` operation. Prior to
  this release, the caches needed to be cleared manually.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#468][#468]

  </details>

- Improved reliability of the PowerShell adapter caches. Starting with this release, the adapter
  caches include a property defining the version of the caching logic they use. If the adapter
  caching version doesn't match the property of the cache, the adapter rebuilds it with the new
  version. This enables updating the caching logic in new releases.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#468][#468]

  </details>

- Added support for the [WhatIf capability][p9-04] to the `Microsoft.Windows/Registry` resource,
  improving the user experience when calling `dsc config set` with the [--what-if][p9-05] option.

  <details><summary>Related work items</summary>

  - Issues: [#452][#452]
  - PRs: [#465][#465]

  </details>

- Added handling for when `dsc` is launched from Explorer or the Microsoft Store. Starting with
  this release, when `dsc` is launched from the Microsoft Store application or Explorer, it shows a
  message linking users to the documentation and waits for a key press before exiting.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#481][#481]

  </details>

- Improved performance for the PowerShell adapter caching by immediately invalidating the cache
  when the cache timestampe entries are stale or missing instead of checking each module in the
  cache.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#497][#497]

  </details>

### Fixed

- Fixed a bug in the `Microsoft.Windows/PowerShell` adapter causing it to always invoke the `Get` operation.

  <details><summary>Related work items</summary>

  - Issues: [#445][#445]
  - PRs: [#480][#480]

  </details>

- Fixed a bug in the PowerShell adapters that caused errors when it discovered multiple modules
  with the same name. Starting with this release, the adapter chooses the version of the module
  with the latest version.

  <details><summary>Related work items</summary>

  - Issues: [#487][#487]
  - PRs: [#489][#489]

  </details>

- Fixed the error messaging when DSC doesn't get any input for a `Test` operation to clearly
  indicate the problem. Prior to this release, users received a difficult-to-decipher message about
  an unexpected end of file instead.

  <details><summary>Related work items</summary>

  - Issues: [#484][#484]
  - PRs: [#504][#504]

  </details>

- Fixed the behavior when a user specifies an invalid name or wildcard filter when calling
  `dsc resource list` with the `--adapter` option. Prior to this release, DSC returned no data.
  Starting with this release, DSC writes a message to STDERR indicating that no adapter was found.
  The operation still exits with exit code `0`.

  <details><summary>Related work items</summary>

  - Issues: [#477][#477]
  - PRs: [#506][#506]

  </details>

- Fixed the PowerShell adapters to correctly handle cache updates when a module containing
  resources is deleted externally.

  <details><summary>Related work items</summary>

  - Issues: [#495][#495]
  - PRs: [#497][#497]

  </details>

- Fixed the PowerShell adapters to return a clear error message when a user attempts to call the
  `Export` operation on an adapted resource that doesn't support it.

  <details><summary>Related work items</summary>

  - Issues: [#503][#503]
  - PRs: [#505][#505]

  </details>

<!-- Preview.9 links -->
[p9-01]: docs/reference/schemas/config/functions/variables.md
[p9-02]: docs/reference/schemas/config/functions/overview.md
[p9-03]: docs/reference/schemas/config/functions/overview.md#example-4---access-object-properties-and-array-items
[p9-04]: docs/reference/schemas/outputs/resource/list.md#capability-whatif
[p9-05]: docs/reference/cli/config/set.md#-w---what-if

## [v3.0.0-preview.8][release-v3.0.0-preview.8] - 2024-06-19

This section includes a summary of changes for the `preview.8` release. For the full list of changes
in this release, see the [diff on GitHub][compare-v3.0.0-preview.8].

<!-- Release links -->
[release-v3.0.0-preview.8]: https://github.com/PowerShell/DSC/releases/tag/v3.0.0-preview.8 "Link to the DSC v3.0.0-preview.8 release on GitHub"
[compare-v3.0.0-preview.8]: https://github.com/PowerShell/DSC/compare/v3.0.0-preview.7...v3.0.0-preview.8

### Changed

- Changed the `Microsoft.DSC/PowerShell` adapter to only handle PowerShell DSC Resources
  implemented as classes and remove the dependency on the **PSDesiredStateConfiguration** module.
  The `Microsoft.Windows/WindowsPowerShell` adapter continues to work with classic PSDSC resources.
  Neither adapter supports composite PSDSC resources. This change simplified the code and coincided
  with ensuring that the `Microsoft.DSC/PowerShell` adapter works correctly on Linux and macOS as
  well as Windows. This change also brought performance improvements to the adapter, speeding up
  resource invocation and discovery.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs:
    - [#435][#435]
    - [#439][#439]

  </details>

### Added

- Added the [`--what-if` (`-w`)][p8-01] option to the [dsc config set][cmd-cset] command. When you
  call `dsc config set` with the `--what-if` option, DSC doesn't actually invoke the resources to
  enforce the desired state. Instead, it returns the expected output for the command, showing the
  before and after state for each resource instance.

  The output for the `dsc config set` operation with the `--what-if` operation is the same as an
  [actual configuration set operation][p8-02], except that the metadata field
  [executionType][p8-03] is set to `WhatIf` instead of `Actual`.

  By default, the generated output is synthetic, based on the results of the resources' `test`
  operation. Resources can define the [whatIf][p8-04] property in their resource manifest to
  participate in what-if operations, reporting more specifically how they will change the system.
  For example, participating resources could indicate whether an actual set operation will require
  a reboot or whether the current user has the correct permissions to manage that resource
  instance.

  Participating resources have the [WhatIf capability][p8-05].

  <details><summary>Related work items</summary>

  - Issues: [#70][#70]
  - PRs:
    - [#400][#400]
    - [#441][#441]

  </details>

- Added support for [importer resources][p8-06]. These resources resolve external sources to a
  nested DSC Configuration document. The resolved instances are processed as nested resource
  instances.

  This required some updates to the schemas, all backwards-compatible:

  - Added a new [resourceKind][p8-07] named `Import`.
  - Added the [resolve][p8-08] command to resource manifests.
  - Added the new [`Resolve`][p8-09] capability, returned in the output for the
    [dsc resource list][cmd-rlist] command when DSC discovers an importer resource.

  <details><summary>Related work items</summary>

  - Issues: [#429][#429]
  - PRs:
    - [#412][#412]
    - [#464][#464]

  </details>

- Added the `Microsoft.DSC/Include` importer resource to resolve instances from an external
  configuration document. The resolved instances are processed as nested instances for the
  `Microsoft.DSC/Include` resource instance.
  
  You can use this resource to write smaller configuration documents and compose them as needed.
  For example, you could define a security baseline and a web server configuration separately, then
  combine them for a given application:

  ```yaml
  $schema: &schema https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
  resources:
  # Group of included baseline configurations
  - name: Baselines
    type: Microsoft.DSC/Group
    properties:
      $schema: *schema
      resources:
      - name: Security Baseline
        type: Microsoft.DSC/Include
        properties:
          configurationFile: security_baseline.dsc.yaml
          parametersFile:    security_baseline.parameters.yaml
      - name: Web Server Baseline
        type: Microsoft.DSC/Include
        properties:
          configurationFile: web_baseline.dsc.yaml
          parametersFile:    web_baseline.parameters.yaml
        dependsOn:
          - "[resourceId('Microsoft.DSC/Include', 'Security Baseline')]"

  # application configuration instances, all depend on the baselines
  - name: Application configuration
    type: MyApp/Settings
    properties:
      someSetting: someValue
    dependsOn:
      - "[resourceId('Microsoft.DSC/Group', 'Baselines')]"
  ```

  <details><summary>Related work items</summary>

  - Issues: [#429][#429]
  - PRs: [#412][#412]

  </details>

- Added caching for PowerShell Desired State Configuration (PSDSC) resources when using the
  `Microsoft.DSC/PowerShell` and `Microsoft.Windows/PowerShell` adapters. The adapters use the
  cache to speed up resource discovery. The performance improvement reduced the resource list time
  under tests from eight seconds to two seconds, and reduced invocation operation times by half.

  The adapters cache the resources in the following locations, depending on your platform:

  |            Adapter             | Platform |                      Path                       |
  | :----------------------------: | :------: | :---------------------------------------------- |
  |   `Microsoft.DSC/PowerShell`   |  Linux   | `$HOME/.dsc/PSAdapterCache.json`                |
  |   `Microsoft.DSC/PowerShell`   |  macOS   | `$HOME/.dsc/PSAdapterCache.json`                |
  |   `Microsoft.DSC/PowerShell`   | Windows  | `%LOCALAPPDATA%\dsc\PSAdapterCache.json`        |
  | `Microsoft.Windows/PowerShell` | Windows  | `%LOCALAPPDATA%\dsc\WindowsPSAdapterCache.json` |

  The adapters check whether the cache is stale on each run and refresh it if:

  - The `PSModulePath` environmental variable is updated.
  - Any module is added or removed from the `PSModulePath`.
  - Any related files in a cached PSDSC resource module has been updated since the cache was
    written. The adapter watches the `LastWriteTime` of module files with the following extensions:
    `.ps1`, `.psd1`, `.psm1`, and `.mof`.

  <details><summary>Related work items</summary>

  - Issues: [#371][#371]
  - PRs: [#432][#432]

  </details>

- Added the `DSC.PackageManagement/Apt` resource for managing software on systems that use the
  advanced package tool (APT). In this release, you can use the resource to:

  - Install the latest version of a package.
  - Uninstall a package.
  - Get the current state of a package.
  - Export every installed package as a DSC resource instance.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#434][#434]

  </details>

- Added the `Microsoft.DSC.Experimental/SystemctlService` class-based PSDSC resource. It has the
  `Get` and `Export` [capabilities][p8-10]. You can use it on Linux systems that manage services
  with SystemD and `systemctl`. In this release, it doesn't support setting services.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#454][#454]

  </details>

### Fixed

- Fixed the JSON Schema for [exit codes][p8-11] in the resource manifest to support negative
  integers. Prior to this release, the DSC engine supported negative exit codes but the JSON Schema
  forbid them.

  <details><summary>Related work items</summary>

  - Issues: [#407][#407]
  - PRs: [#410][#410]

  </details>

- Fixed the behavior of the [int()][int()] configuration function to error when given an input
  value other than a string or integer. Prior to this release, when you specified a number with
  a fractional part as input for the function, it coerced the input value to an integer representing
  the fractional part. Starting with this release, the `int()` function raises an invalid input
  error when the input value isn't a string or an integer.

  <details><summary>Related work items</summary>

  - Issues: [#390][#390]
  - PRs: [#438][#438]

  </details>

- Fixed the implementation to retrieve non-zero exit code descriptions for resource errors from the
  resource manifest, if defined. Prior to this release, these error descriptions weren't surfaced.

  <details><summary>Related work items</summary>

  - Issues: [#431][#431]
  - PRs: [#444][#444]

  </details>

<!-- Preview.8 links -->
[p8-01]: ./docs/reference/cli/config/set.md#-w---what-if
[p8-02]: ./docs/reference/schemas/outputs/config/set.md
[p8-03]: ./docs/reference/schemas/metadata/Microsoft.DSC/properties.md#executiontype
[p8-04]: ./docs/reference/schemas/resource/manifest/whatif.md
[p8-05]: ./docs/reference/schemas/outputs/resource/list.md#capability-whatif
[p8-06]: ./docs/reference/schemas/definitions/resourceKind.md#importer-resources
[p8-07]: ./docs/reference/schemas/definitions/resourceKind.md
[p8-08]: ./docs/reference/schemas/resource/manifest/resolve.md
[p8-09]: ./docs/reference/schemas/outputs/resource/list.md#capability-resolve
[p8-10]: ./docs/reference/schemas/outputs/resource/list.md#capabilities
[p8-11]: ./docs/reference/schemas/resource/manifest/root.md#exitcodes

## [v3.0.0-preview.7][release-v3.0.0-preview.7] - 2024-04-22

This section includes a summary of changes for the `preview.7` release. For the full list of changes
in this release, see the [diff on GitHub][compare-v3.0.0-preview.7].

<!-- Release links -->
[release-v3.0.0-preview.7]: https://github.com/PowerShell/DSC/releases/tag/v3.0.0-preview.7 "Link to the DSC v3.0.0-preview.7 release on GitHub"
[compare-v3.0.0-preview.7]: https://github.com/PowerShell/DSC/compare/v3.0.0-alpha.4...v3.0.0-preview.7

### Changed

- The version segment of the schema URIs for DSC have been updated from `2023/10` to `2024/04` to
  accommodate breaking schema changes from the schemas that `alpha.5` used. You can find more
  information about the specific changes to the schemas in the following changelog entries:

  - [Renamed 'providers' to 'adapters'](#rename-provider-to-adapter)
  - [Added the 'delete' operation for resources](#add-delete-operation)
  - [Added the option to specify a required security context for a configuration document](#add-elevation-requirement)
  - [Add option to specify a JSON input argument for resource commands](#add-json-input-arg)
  - [Add 'kind' property to resource manifests](#add-kind-property)
  - [Camel-cased 'SecureObject' and 'SecureString' parameter types](#camel-casing-secure-types)
  - [Add 'capabilities' to 'dsc resource list' output](#add-capabilities)
  - [Added metadata to config and resource output](#add-metadata-output)

  Update your configuration documents and resource manifests to use the following URIs for the
  `$schema` keyword:

  ```yaml
  Canonical URI for configuration documents: >-
    https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json

  Bundled URI for configuration documents: >-
    https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/bundled/config/document.json

  Enhanced Authoring in VS Code URI for configuration documents: >-
    https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/bundled/config/document.vscode.json

  Canonical URI for resource manifests: >-
    https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/resource/manifest.json

  Bundled URI for resource manifests: >-
    https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/bundled/resource/manifest.json

  Enhanced Authoring in VS Code URI for resource manifests: >-
    https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/bundled/resource/manifest.vscode.json
  ```

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#397][#397]

  </details>

- <a id="rename-provider-to-adapter"></a>

  In this release, the term `DSC Resource Provider` is replaced with the more semantically accurate
  `DSC Resource Adapter`. These resources enable users to leverage resources that don't define a
  DSC Resource Manifest with DSC, like PSDSC resources - they're _adapters_ between DSCv3 and
  resources defined in a different way.

  Beyond using different terminology in the documentation, this change also renamed the resource
  manifest property `provider` to [adapter][p7-01], and the `requires` property in the output for
  `dsc resource list` has been renamed to [requireAdapter][p7-02].

  <details><summary>Related work items</summary>

  - Issues: [#310][#310]
  - PRs:
    - [#334][#334]
    - [#373][#373]

  </details>

- <a id="camel-casing-secure-types"></a> Changed the casing for the [parameter type enums][p7-03]
  from `SecureString` to `secureString` and `SecureObject` to `secureObject`, to better match the
  type enumerations in ARM.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#364][#364]

  </details>

- The [envvar()][envvar()] function now raises an error when the specified environment variable
  doesn't exist instead of returning an empty string. This change helps reduce unexpected and
  difficult to diagnose errors when a configuration expects a value from the environment variable.

  <details><summary>Related work items</summary>

  - Issues: [#336][#336]
  - PRs: [#358][#358]

  </details>

- Renamed the `DscConfigRoot` environment variable to `DSC_CONFIG_ROOT`. DSC now correctly
  absolutizes the variable, even when the path to a configuration document is a relative path. DSC
  also raises a warning when you define the environment variable outside of DSC before overriding
  it.

  <details><summary>Related work items</summary>

  - Issues:
    - [#317][#317]
    - [#335][#335]
  - PRs: [#342][#342]

  </details>

- Updated the default behavior of the [dsc resource list][cmd-rlist] command and added the new
  [--adapter][p7-04] option to the command.

  Prior to this release, the command always called the `list` command for any discovered adapters,
  even when searching for a non-adapted resource by name. Enumerating the adapted resources can be
  a slow process, so the command no longer calls the adapters to list their adapted resources by
  default.

  Instead, you can use the `--adapter` option to specify a filter for the adapters you want to list
  adapted resources for. Specify the fully qualified type name of an adapter or a string including
  wildcards (`*`) to use as a filter for adapter names. You can specify the filter `*` to have DSC
  call the `list` operation for every discovered adapter, returning all adapted resources.

  For more information, see [dsc resource list][cmd-rlist].

  <details><summary>Related work items</summary>

  - Issues:
    - [#274][#274]
    - [#368][#368]
  - PRs: [#377][#377]

  </details>

- Updated the table view for the [dsc resource list][cmd-rlist] command to display the resource
  kind and capabilities. The capabilities column in the table uses bit flags for the display to
  keep the column width manageable.

  For more information, see the "Output" section of [dsc resource list][cmd-rlist].

  <details><summary>Related work items</summary>

  - Issues: [#329][#329]
  - PRs: [#346][#346]

  </details>

### Added

- <a id="add-delete-operation" /></a> Added the [dsc resource delete][cmd-rdelete] command and the
  [delete][p7-05] operation property to the resource manifest. Prior to this release, resources had
  to handle deleting resources as part of their `set` operation, and the development guidance was
  to use the [_exist][p7-06] standard property to indicate whether a resource should exist.

  Now, resource authors can indicate through the resource manifest whether the resource supports
  the `delete` operation with a separate command or as part of the `set` operation. It can be
  simpler to implement a separate `delete` operation than to handle deleting instances as part of
  `set`. You can implement your resource to have an explicit `delete` command and handle deleting
  instances as part of a `set` operation.

  You can also use the `dsc resource delete` command to delete instances one at a time. For this
  command, the JSON input defines the filter to pass to the resource for deleting the instance. For
  more information, see [dsc resource delete command reference][cmd-rdelete].

  If your resource handles deleting instances as part of `set`, use the [handlesExist][p7-07]
  property to tell DSC so. When this property is `true`, the resource has the
  [SetHandlesExist capability][p7-08].

  If your resource has a separate command for deleting instances, use the [delete][p7-05] property
  in your resource manifest to tell DSC and other tools how to invoke the operation. When this
  property is defined, the resource has the [Delete capability][p7-09].

  If your resource handles deleting instances, you should add the `_exist` standard property to the
  resource's [instance schema][p7-10]. While you can use any property name for this, DSC is only aware of
  deletion operations when you use the `_exist` property. DSC won't know to call the `delete`
  operation for resources that don't have the [SetHandlesExist][p7-08] capability.

  For resources that implement `delete` but don't handle `_exist` in the `set` operation, DSC can
  now invoke the delete operation as-needed in a configuration whenever it enforces the desired
  state for an instance of a resource with the `_exist` property set to `false`.

  <details><summary>Related work items</summary>

  - Issues: [#290][#290]
  - PRs:
    - [#379][#379]
    - [#382][#382]

  </details>

- <a id="add-elevation-requirement" /></a> Added the option to specify whether a configuration
  document requires root or elevated permissions. Now, you can define the `securityContext`
  metadata property under the `Microsoft.DSC` namespace in a configuration document to specify
  which security context to use:

  - `Current` - Any security context. This is the default if you don't specify this property in a
    configuration document.
  - `Elevated` - Elevated as root or an administrator.
  - `Restricted` - Not elevated as root or an administrator.

  For example, the following metadata at the top of a configuration document indicates that DSC
  must run as a normal user account, not root or administrator:

  ```yaml
  metadata:
    Microsoft.DSC:
      securityContext: restricted
  ```

  For more information, see [DSC Configuration document metadata schema][p7-11].

  <details><summary>Related work items</summary>

  - Issues: [#258][#258]
  - PRs: [#351][#351]

  </details>

- <a id="add-json-input-arg" /></a> Added the option to define a JSON input argument for resource
  commands. When you define the `args` list for the following commands, you can now define a
  special argument that the command expects to receive the compressed JSON data for:

  - [delete][p7-12]
  - [export][p7-13]
  - [get][p7-14]
  - [set][p7-15]
  - [test][p7-16]
  - [validate][p7-17]

  DSC sends data to these commands in three ways:

  1. When `input` is `stdin`, DSC sends the data as a string representing the data as a compressed
     JSON object without spaces or newlines between the object properties.
  1. When `input` is `env`, DSC sends the data as environment variables. It creates an environment
     variable for each property in the input data object, using the name and value of the property.
  1. When the `args` array includes a JSON input argument definition, DSC sends the data as a
     string representing the data as a compressed JSON object to the specified argument.

  If you don't define the `input` property and don't define a JSON input argument, DSC can't pass
  the input JSON to the resource. You can only define one JSON input argument for a command.

  You must define the `input` property, one JSON input argument in the `args` property array, or
  both. For more information, see the relevant schema documentation for the command property.

  <details><summary>Related work items</summary>

  - Issues: [#218][#218]
  - PRs: [#385][#385]

  </details>

- <a id="added-config-functions"/></a> Added configuration functions:

  - New mathematics functions include [add()][add()], [div()][div()], [max()][max()],
    [min()][min()], [mod()][mod()], [mul()][mul()], and [sub()][sub()]. The mathematics functions
    only operate on integer values.

  - The [reference()][reference()] function enables you to reference the result output for other
    resources, so you can use properties of one resource instance as values for another. The
    `reference()` function only works for resources that DSC has already managed in a
    configuration. You should always add the resource you're referencing with the `reference()`
    function to the [dependsOn][p7-18] list for the instance using the reference.

  - The [createArray()][createArray()] function enables you to create arrays of a given type from
    values.

  - The [int()][int()] function enables you to convert strings and numbers with fractional parts
    into integers.

  <details><summary>Related work items</summary>

  - Issues:
    - [#57][#57]
  - PRs:
    - [#347][#347]
    - [#349][#349]
    - [#352][#352]
    - [#353][#353]
    - [#354][#354]
    - [#360][#360]
    - [#361][#361]
    - [#375][#375]
    - [#376][#376]

  </details>

- <a id="add-kind-property" /></a> Added the [kind][p7-19] property to the resource manifest schema
  and the [output][p7-20] for the [dsc resource list][cmd-rlist] command. This property indicates
  whether the resource is a [group resource][p7-21] (`Group`), an [adapter resource][p7-22]
  (`Adapter`), or neither (`Resource`). For more information, see
  [DSC Resource kind schema reference][p7-23].

  This property is mandatory in the resource manifest for group resources. If your resource
  manifest doesn't define the `kind` property, DSC can infer whether the resource is an adapter
  resource or not. Microsoft recommends always explicitly defining this property in resource
  manifests, because the schema can apply enhanced validation based on the value of the `kind`
  property.

  <details><summary>Related work items</summary>

  - Issues: [#139][#139]
  - PRs: [#338][#338]

  </details>

- <a id="add-capabilities" /></a> Added the [capabilities][p7-24] property to the output for the
  [dsc resource list][cmd-rlist] command. The `capabilities` property indicates how you can use the
  DSC Resource and how DSC and other higher order tools should handle it.

  <details><summary>Related work items</summary>

  - Issues: [#356][#356]
  - PRs: [#357][#357]

  </details>

- <a id="add-metadata-output" /></a> Added the `metadata` property to the outputs for `dsc config`
  and `dsc resource` subcommands. This property in the output defines the context DSC was run under
  and information about the operation. See the output reference for each command for more
  information:

  - [dsc config get][p7-25]
  - [dsc config test][p7-26]
  - [dsc config set][p7-27]
  - [dsc resource get][p7-28]
  - [dsc resource test][p7-29]
  - [dsc resource set][p7-30]

  <details><summary>Related work items</summary>

  - Issues: [#401][#401]
  - PRs: [#405][#405]

  </details>

- Added parsing for [configuration functions][cfuncs] in the [default values][p7-31] of parameters.
  Prior to this release, DSC interpreted configuration functions in parameter default values as
  literal strings.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#364][#364]

  </details>

- Added type validation for parameter [default values][p7-31]. Prior to this release, DSC didn't
  validate that the default value for a parameter was valid for the parameter's [type][p7-32].

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#364]

  </details>

- Added support for resources to send trace information to DSC during command execution. DSC
  Resources can emit JSON objects to stderr. If the object has a property in the following list
  with a string value, DSC interprets the emitted object as a message of the matching level:
  `Error`, `Warning`, `Info`, `Debug`, `Trace`.

  For example, DSC would interpret a resource emitting the following JSON to stderr as a warning:

  ```json
  {"Warning":"Unable to access remote store, using cached local package data only"}
  ```

  DSC emits these messages along with its own messages when the specified trace level for the
  command is equal to or lower than the message's level.

  For more information about trace levels, see the [--trace-level][p7-33] option for the
  [dsc][cmd] root command.

  <details><summary>Related work items</summary>

  - Issues: [#89][#89]
  - PRs: [#287][#287]

  </details>

- Added validation to ensure resources return data for their instances that is valid against their
  own instance JSON schema. Prior to this release, the return data wasn't validated.

  <details><summary>Related work items</summary>

  - Issues: [#251][#251]
  - PRs: [#362][#362]

  </details>

- Added multi-line progress bars for the `dsc resource list` command to provide feedback to
  interactive users about the resource discovery process. Prior to this release, the command
  executed silently.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#323][#323]

  </details>

- Added functionality to insert metadata for adapter resources to indicate if the incoming data is
  for a configuration instead of direct resource invocation. Prior to this release, adapters had no
  way of discerning between a single-instance call for a configuration and a direct resource
  invocation.

  With this change, DSC inserts the following into the data object sent to the adapter during a
  `dsc config` command:

  ```json
  "metadata": {
    "Microsoft.DSC": {
      "context": "Configuration"
    }
  }
  ```

  Adapters can then check whether this value is set in the input data and handle it as-needed.

  <details><summary>Related work items</summary>

  - Issues: [#253][#253]
  - PRs: [#348][#348]

  </details>

- Added the `Microsoft.Windows/RebootPending` resource, which checks whether a Windows machine has
  a pending reboot. It can only be used for assertions, not to enforce state.

  <details><summary>Related work items</summary>

  - Issues: [#333][#333]
  - PRs: [#344][#344]

  </details>

- Added the `Microsoft.DSC.Transitional/RunCommandOnSet` resource, which runs a specified
  executable or script with given arguments during a `set` operation. This resource is intended as
  a temporary transitional resource while migrating to DSCv3 and implementing resources for your
  needs.

  <details><summary>Related work items</summary>

  - Issues: [#302][#302]
  - PRs: [#321][#321]

  </details>

<!-- preview.7 change links -->
[p7-01]: ./docs/reference/schemas/resource/manifest/adapter.md
[p7-02]: ./docs/reference/schemas/outputs/resource/list.md#requireadapter
[p7-03]: ./docs/reference/schemas/definitions/parameters/dataTypes.md
[p7-04]: ./docs/reference/cli/resource/list.md#-a---adapter
[p7-05]: ./docs/reference/schemas/resource/manifest/delete.md
[p7-06]: ./docs/reference/schemas/resource/properties/exist.md
[p7-07]: ./docs/reference/schemas/resource/manifest/set.md#handlesexist
[p7-08]: ./docs/reference/schemas/outputs/resource/list.md#capability-sethandlesexist
[p7-09]: ./docs/reference/schemas/outputs/resource/list.md#capability-delete
[p7-10]: ./docs/reference/schemas/resource/manifest/root.md#schema-1
[p7-11]: ./docs/reference/schemas/config/metadata.md
[p7-12]: ./docs/reference/schemas/resource/manifest/delete.md#json-input-argument
[p7-13]: ./docs/reference/schemas/resource/manifest/export.md#json-input-argument
[p7-14]: ./docs/reference/schemas/resource/manifest/get.md#json-input-argument
[p7-15]: ./docs/reference/schemas/resource/manifest/set.md#json-input-argument
[p7-16]: ./docs/reference/schemas/resource/manifest/test.md#json-input-argument
[p7-17]: ./docs/reference/schemas/resource/manifest/validate.md#json-input-argument
[p7-18]: ./docs/reference/schemas/config/resource.md#dependsOn
[p7-19]: ./docs/reference/schemas/resource/manifest/root.md#kind
[p7-20]: ./docs/reference/schemas/outputs/resource/list.md
[p7-21]: ./docs/reference/schemas/definitions/resourceKind.md#group-resources
[p7-22]: ./docs/reference/schemas/definitions/resourceKind.md#adapter-resources
[p7-23]: ./docs/reference/schemas/definitions/resourceKind.md
[p7-24]: ./docs/reference/schemas/outputs/resource/list.md#capabilities
[p7-25]: ./docs/reference/schemas/outputs/config/get.md#metadata-1
[p7-26]: ./docs/reference/schemas/outputs/config/test.md#metadata-1
[p7-27]: ./docs/reference/schemas/outputs/config/set.md#metadata-1
[p7-28]: ./docs/reference/schemas/outputs/resource/get.md#metadata-1
[p7-29]: ./docs/reference/schemas/outputs/resource/test.md#metadata-1
[p7-30]: ./docs/reference/schemas/outputs/resource/set.md#metadata-1
[p7-31]: ./docs/reference/schemas/config/parameter.md#defaultvalue
[p7-32]: ./docs/reference/schemas/config/parameter.md#type
[p7-33]: ./docs/reference/cli/dsc.md#-l---trace-level

## [v3.0.0-alpha.5][release-v3.0.0-alpha.5] - 2024-02-27

This section includes a summary of changes for the `alpha.5` release. For the full list of changes
in this release, see the [diff on GitHub][compare-v3.0.0-alpha.5].

<!-- Release links -->
[release-v3.0.0-alpha.5]: https://github.com/PowerShell/DSC/releases/tag/v3.0.0-alpha.5 "Link to the DSC v3.0.0-alpha.5 release on GitHub"
[compare-v3.0.0-alpha.5]: https://github.com/PowerShell/DSC/compare/v3.0.0-alpha.4...v3.0.0-alpha.5

### Changed

- Updated the options for the `dsc` root command:

  - Removed the global `--format` option, which controls the output format. Now, the relevant
    subcommands that return formattable output have the `--format` option (short option as `-f`)
    added to them.
  - Removed the global `--input` and `--input-file` options. Now, the `config` subcommands have the
    `--document` and `--path` options for specifying the configuration document as a string or from
    a file. The relevant `resource` subcommands have the `--input` and `--path` options for
    specifying the instance properties as a string or from a file.
  - The `--logging-level` option is renamed to [--trace-level][a5.05] with the short name `-l`. The
    default trace level is now `warning` instead of `info`.
  - Added the [--trace-format][a5.06] option with the `-f` short name. This option enables you to
    choose the format for the trace messages emitted to stderr. By default, the messages are
    emitted as lines of text with console colors. You can set this option to `plaintext` to emit
    the messages without console colors or to `json` to emit the messages as JSON objects.

    The trace messaging is also updated to only emit source files and line numbers for the `debug`
    and `trace` levels.

  <details><summary>Related work items</summary>

  - Issues:
    - [#286][#286]
    - [#227][#227]
    - [#226][#226]
  - PRs:
    - [#299][#299]
    - [#303][#303]
    - [#305][#305]
    - [#388][#388]

  </details>

- Updated the JSON schemas for the [get][a5.07], [set][a5.08], and [test][a5.09] output data. This
  change corrects an issue with how DSC surfaced information about instances nested inside group
  and adapter resources. Now when you review the output, you'll be able to see the results for each
  nested instance instead of a confusing object that loses the nested instance type and name
  information.

  This schema change is backwards compatible.

  <details><summary>Related work items</summary>

  - Issues:
    - [#165][#165]
    - [#266][#266]
    - [#284][#284]
  - PRs: [#318][#318]

  </details

- Changed the [concat][a5.10] configuration function to match the behavior of the ARM template
  function. The `concat()` function now only accepts strings or arrays of strings as input values.
  It raises an error if the input values are not of the same type.

  <details><summary>Related work items</summary>

  - Issues: [#271][#271]
  - PRs: [#322][#322]

  </details

### Added

- Implemented support for referencing parameters in a configuration with the [parameters()][a5.01]
  configuration function. This enables you to take advantage of parameterized configurations. Until
  this release, you could define but not reference parameters.

  Now, you can use the [--parameters][a5.02] and [--parameters-file][a5.03] options with the
  [dsc config][a5.04] commands to pass values for any parameter defined in the configuration
  document.

  <details><summary>Related work items</summary>

  - Issues: [#49][#49]
  - PRs:
    - [#291][#291]
    - [#294][#294]

  </details>

- Added support for authoring DSC Resource manifests in YAML. DSC now recognizes resource manifests
  that use the `.dsc.resource.yml` or `.dsc.resource.yaml` file extension instead of only
  `.dsc.resource.json`.

  <details><summary>Related work Items</summary>

  - Issues: [#129][#129]
  - PRs: [#311][#311]

  </details>

- Added the [DSCConfigRoot][a5.11] environment variable and the
  [envvar() configuration function][a5.12] to enable users to reference files and folders relative
  to the folder containing the configuration document. DSC automatically and only creates the
  `DSCConfigRoot` environment variable when you use the `--path` option to specify the
  configuration document instead of passing the document as a string from stdin or with the
  `--document` option.

  > [!NOTE]
  > In this release, DSC doesn't expand the specified path to an absolute path. You should always
  > specify the full path to the configuration document if you want to reference the
  > `DSCConfigRoot` variable in your configuration. Further, DSC always sets the value for this
  > environment variable when you use the `--path` option. If the environment variable is already
  > set, it overrides it silently.
  >
  > In a future release, the variable will be renamed to `DSC_CONFIG_ROOT` and DSC will
  > automatically expand relative paths into absolute paths when setting the environment variable.
  > It will also emit a warning when it overrides the variable.

  <details><summary>Related work Items</summary>

  - Issues: [#75][#75]
  - PRs: [#313][#313]

  </details>

- Added support for using the [dsc config export][cmd-cexport] and
  [dsc resource export][cmd-rexport] commands with the PowerShell adapter resource. PSDSC resources
  can now participate in the `export` command if they define a static method that returns an array
  of the PSDSC resource class.

  <details><summary>Related work Items</summary>

  - Issues: [#183][#183]
  - PRs: [#307][#307]

  </details>

- Added the `methods` column to the default table view for the console output of the
  [dsc resource list][cmd-rlist] command. This new column indicates which methods the resource
  explicitly implements. Valid values include `get`, `set`, `test`, and `export`. This information
  is only available in the table view. It isn't part of the output object for the command. If you
  use the [--format][a5.16] parameter, capture the command output, or redirect the output, the
  `methods` information isn't included.

  Resources that don't implement `test` rely on DSC's synthetic test behavior instead. They can
  still be used for test and set operations.

  Resources that don't implement `export` can't be used with the `dsc config export` or
  `dsc resource export` commands.

  Resources that don't implement `set` can be used for auditing, but not `dsc resource set`. They
  can be used with the `dsc config set` command, but only if they're nested inside a
  `DSC/AssertionGroup` instance.

  <details><summary>Related work Items</summary>

  - Issues: [#309][#309]
  - PRs: [#314][#314]

  </details>

- Added an prototype for a WMI resource adapter to enable users to query WMI. The adapter is
  disabled by default, as enumerating the WMI resources can have a performance impact. To enable
  it, rename the resource manifest from `wmigroup.dsc.resource.json.optout` to
  `wmigroup.dsc.resource.json`.

  <details><summary>Related work Items</summary>

  - Issues: [#263][#263]
  - PRs: [#279][#279]

  </details>

<!-- alpha.5 links -->
[a5.01]: docs/reference/schemas/config/functions/parameters.md
[a5.02]: docs/reference/cli/config/command.md#-p---parameters
[a5.03]: docs/reference/cli/config/command.md#-f---parameters_file
[a5.04]: docs/reference/cli/config/command.md
[a5.05]: docs/reference/cli/dsc.md#-l---trace-level
[a5.06]: docs/reference/cli/dsc.md#-f---trace-format
[a5.07]: docs/reference/schemas/outputs/resource/get.md
[a5.08]: docs/reference/schemas/outputs/resource/set.md
[a5.09]: docs/reference/schemas/outputs/resource/test.md
[a5.10]: docs/reference/schemas/config/functions/concat.md
[a5.11]: docs/reference/cli/config/command.md#environment-variables
[a5.12]: docs/reference/schemas/config/functions/envvar.md
[a5.16]: docs/reference/cli/resource/list.md#-f---format

## [v3.0.0-alpha.4][release-v3.0.0-alpha.4] - 2023-11-14

This section includes a summary of changes for the `alpha.4` release. For the full list of changes
in this release, see the [diff on GitHub][compare-v3.0.0-alpha.4].

<!-- Release links -->
[release-v3.0.0-alpha.4]: https://github.com/PowerShell/DSC/releases/tag/v3.0.0-alpha.4 "Link to the DSC v3.0.0-alpha.4 release on GitHub"
[compare-v3.0.0-alpha.4]: https://github.com/PowerShell/DSC/compare/v3.0.0-alpha.3...v3.0.0-alpha.4

### Changed

- Updated the canonical version of the schema URIs from `2023/08` to `2023/10`, as this release
  includes breaking changes for the schemas.

  As part of this change, the `$schema` keyword for both [configuration documents][a4.01] and
  [resource manifests][a4.02] accepts any valid URI for the schemas, instead of only one. Now, you
  can set the value for the keyword to the unbundled schema, the bundled schema, or the enhanced
  authoring schema for any supported version.

- Replaced the `_ensure` well-known property with the boolean [_exist][a4.03] property. This
  improves the semantics for users and simplifies implementation for resources, replacing the
  string enum values `Present` and `Absent` with `true` and `false` respectively.

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

- Added the `--input` and `--input-file` global options to the root `dsc` command. Now, you
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

- Added the new [completer][a4.04] command enables users to add shell completions for DSC to their
  shell. The command supports completions for Bash, Elvish, fish, PowerShell, and ZSH.

  <details><summary>Related work items</summary>

  - Issues: [#186][#186]
  - PRs: [#216][#216]

  </details>

- DSC now emits log messages to the stderr stream. This can make it easier to understand what DSC
  is doing. This doesn't affect the data output. By default, DSC emits errors, warnings, and
  informational messages, but not debug or trace messaging. You can control the level of the
  logging with the new `--logging-level` option on the root `dsc` command.

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

- Added initial [configuration document functions][a4.05] to DSC. You can now use the
  [base64()][a4.06], [concat()][a4.07], and [resourceId()][a4.08] functions in the configuration
  document.

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

<!-- alpha.4 links -->
[a4.01]: docs/reference/schemas/config/document.md#schema
[a4.02]: docs/reference/schemas/resource/manifest/root.md#schema
[a4.03]: docs/reference/schemas/resource/properties/exist.md
[a4.04]: docs/reference/cli/completer/command.md
[a4.05]: docs/reference/schemas/config/functions/overview.md
[a4.06]: docs/reference/schemas/config/functions/base64.md
[a4.07]: docs/reference/schemas/config/functions/concat.md
[a4.08]: docs/reference/schemas/config/functions/resourceId.md

## [v3.0.0-alpha.3][release-v3.0.0-alpha.3] - 2023-09-26

This section includes a summary of changes for the `alpha.3` release. For the full list of changes
in this release, see the [diff on GitHub][compare-v3.0.0-alpha.3].

<!-- Release links -->
[release-v3.0.0-alpha.3]: https://github.com/PowerShell/DSC/releases/tag/v3.0.0-alpha.3 "Link to the DSC v3.0.0-alpha.3 release on GitHub"
[compare-v3.0.0-alpha.3]: https://github.com/PowerShell/DSC/compare/v3.0.0-alpha.2...v3.0.0-alpha.3

### Changed

- Replaced the `manifestVersion` property for resource manifests with [$schema][a3.01]. Instead of
  specifying a semantic version, resources need to indicate which canonical schema DSC should use
  to validate and process the manifest.

  <details><summary>Related work items</summary>

  - Issues: [#127][#127]
  - PRs: [#199][#199]

  </details>

- Updated the `preTest` property for the `set` command in resource manifests to
  [implementsPretest][a3.02] to more make the manifest easier to read and understand.

  <details><summary>Related work items</summary>

  - PRs: [#197][#197]

  </details>

- The [dsc resource set][cmd-rset] command no longer tests the resource instance before invoking the
  `set` operation. This simplifies the behavior for the command and adheres more accurately to the
  implied contract for directly invoking a resource with DSC.

  <details><summary>Related work items</summary>

  - Issues: [#98][#98]
  - PRs: [#197][#197]

  </details>

- Replaced the `args` option with `env` for defining how a command-based resource expects to
  receive input for the [get][a3.04], [set][a3.05], and [test][a3.06] commands in the resource
  manifest.

  The `args` option was never implemented. Instead, resource authors can set the `input` property
  to `env` to indicate that the resource expects input as environmental variables.

  <details><summary>Related work items</summary>

  - PRs: [#198][#198]

  </details>

- The `input` property for the [get][a3.04] command in a resource manifest no longer has a default
  value. Instead, when a resource doesn't define `input` for the `get` command, DSC doesn't send
  any input to the resource for that command.

  <details><summary>Related work items</summary>

  - PRs: [#198][#198]

  </details>

<!-- alpha.3 links -->
[a3.01]: docs/reference/schemas/resource/manifest/root.md#schema
[a3.02]: docs/reference/schemas/resource/manifest/set.md#implementspretest
[a3.04]: docs/reference/schemas/resource/manifest/get.md#input
[a3.05]: docs/reference/schemas/resource/manifest/set.md#input
[a3.06]: docs/reference/schemas/resource/manifest/test.md#input

## [v3.0.0-alpha.2][release-v3.0.0-alpha.2] - 2023-09-05

This section includes a summary of changes for the `alpha.2` release. For the full list of changes
in this release, see the [diff on GitHub][compare-v3.0.0-alpha.2].

<!-- Release links -->
[release-v3.0.0-alpha.2]: https://github.com/PowerShell/DSC/releases/tag/v3.0.0-alpha.2 "Link to the DSC v3.0.0-alpha.2 release on GitHub"
[compare-v3.0.0-alpha.2]: https://github.com/PowerShell/DSC/compare/v3.0.0-alpha.1...v3.0.0-alpha.2

### Changed

- The [$schema][a2.14] value for configuration documents now points to the canonical published
  schema URI,
  `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json`.

  <details><summary>Related work items</summary>

  - PRs: [#156][#156]

  </details>

### Added

- Implemented functionality for the [dependsOn property of resource instances][a2.01] in
  configuration documents, enabling resource instances to depend on the successful processing of
  one or more other instances in the document.

  <details><summary>Related work items</summary>

  - Issues: [#45][#45]
  - PRs: [#175][#175]

  </details>

- Added the [export][a2.02] property to the resource manifest schema, indicating that the resource
  is exportable and defining how DSC can retrieve the current state for every instance of the
  resource.

  <details><summary>Related work items</summary>

  - Issues: [#73][#73]
  - PRs: [#171][#171]

  </details>

- Added the [dsc config export][cmd-cexport] command to convert an input configuration document
  defining a list of resource types into a usable configuration document that defines the current
  state for every instance of those resources.

  <details><summary>Related work items</summary>

  - Issues: [#73][#73]
  - PRs: [#171][#171]

  </details>

- Added the [dsc resource export][cmd-rexport] command to generate a usable configuration document
  that defines the current state for every instance of a specified resource.

  <details><summary>Related work items</summary>

  - Issues: [#73][#73]
  - PRs: [#171][#171]

  </details>

- Added the [--all][a2.05] option for the [dsc resource get][cmd-rget] command, enabling users to
  retrieve the current state for every instance of an exportable resource with a single command.

  <details><summary>Related work items</summary>

  - Issues:
    - [#73][#73]
    - [#174][#174]
  - PRs: [#171][#171]

  </details>

- Added handling for the <kbd>Ctrl</kbd>+<kbd>C</kbd> key combination to cancel a DSC operation.
  When `dsc` cancels an operation due to this key-press, it indicates that the operation was
  cancelled with [exit code 6][a2.07].

  <details><summary>Related work items</summary>

  - PRs: [#177][#177]
  - Issues: [#150][#150]

  </details>

- Added support for using the [DSC_RESOURCE_PATH environment variable][a2.08] to define a list of
  folders to search for command-based DSC Resource manifests. When `DSC_RESOURCE_PATH` is defined,
  DSC searches those folders for resources and ignores the `PATH` variable for resource discovery.

  <details><summary>Related work items</summary>

  - PRs: [#176][#176]
  - Issues: [#133][#133]

  </details>

- The `DSC/AssertionGroup`, `DSC/Group`, and `DSC/ParallelGroup` resources now define semantic exit
  codes in their manifests. These resources now indicate that they use the same
  [exit codes as the dsc command][a2.08].

  <details><summary>Related work items</summary>

  - PRs: [#182][#182]
  - Issues: [#181][#181]

  </details>

- Added type validation in the schema for the [defaultValue][a2.09] and [allowedValues][a2.10]
  properties of [configuration document parameters][a2.11] to improve the authoring experience.
  Now, when a parameter defines values for these properties that are incompatible with the defined
  data type, validation raises an error indicating that the values are invalid and why.

  <details><summary>Related work items</summary>

  - PRs: [#172][#172]

  </details>

- Enhanced VS Code-specific schemas for configuration documents and resource manifests to improve
  the authoring experience. The enhanced schemas use keywords only supported by VS Code to:

  - Render Markdown help information for properties and enums.
  - Provide contextual error messages when a value fails pattern validation.
  - Define default snippets to autocomplete values.

  These schemas are non-canonical and should only be used for authoring. For more information, see
  [Authoring with enhanced schemas][a2.12].

  <details><summary>Related work items</summary>

  - PRs: [#172][#172]

  </details>

- Documentation to the [Microsoft/OSInfo][a2.13] resource instance schema and command-line tool to
  provide contextual help about the properties the resource can validate.

  <details><summary>Related work items</summary>

  - PRs: [#168][#168]

  </details>

### Fixed

- The data-type conditionals for the [configuration parameters][a2.11] schema so that the `min*`
  and `max*` keywords apply to the correct data types. Previously, the logic prevented them from
  ever applying.

  <details><summary>Related work items</summary>

  - PRs: [#172][#172]

  </details>

- Using the `registry find` command no longer raises a panic error due to conflicting option
  definitions on the command.

  <details><summary>Related work items</summary>

  - PRs: [#163][#163]

  </details>

<!-- alpha.2 links -->
[a2.01]: docs/reference/schemas/config/resource.md#dependson
[a2.02]: docs/reference/schemas/resource/manifest/export.md
[a2.05]: docs/reference/cli/resource/get.md##a---all
[a2.07]: docs/reference/cli/dsc.md#exit-codes
[a2.08]: docs/reference/cli/dsc.md#environment-variables
[a2.09]: docs/reference/schemas/config/parameter.md#defaultvalue
[a2.10]: docs/reference/schemas/config/parameter.md#allowedvalues
[a2.11]: docs/reference/schemas/config/parameter.md
[a2.12]: https://learn.microsoft.com/powershell/dsc/concepts/enhanced-authoring?view=dsc-3.0&preserve-view=true
[a2.13]: https://learn.microsoft.com/powershell/dsc/reference/microsoft/osinfo/resource?view=dsc-3.0&preserve-view=true
[a2.14]: docs/reference/schemas/config/document.md#schema

## [v3.0.0-alpha.1][release-v3.0.0-alpha.1] - 2023-08-04

This is the first public release of DSC v3. Consider this release alpha quality. Use it only for
development evaluation, as it has known issues and isn't feature complete.

For the full list of changes in this release, see the [diff on GitHub][compare-v3.0.0-alpha.1].

<!-- Release comparison link -->
[release-v3.0.0-alpha.1]: https://github.com/PowerShell/DSC/releases/tag/v3.0.0-alpha.1 "Link to the DSC v3.0.0-alpha.1 release on GitHub"
[compare-v3.0.0-alpha.1]: https://github.com/PowerShell/DSC/compare/6090b1464bbf81fded5453351708482a4db35258...v3.0.0-alpha.1

<!-- CLI reference links -->
[cmd]:             ./docs/reference/cli/dsc.md
[cmd-completion]:  ./docs/reference/cli/completer/command.md
[cmd-schema]:      ./docs/reference/cli/schema/command.md
[cmd-c]:         ./docs/reference/cli/config/command.md
[cmd-cexport]:  ./docs/reference/cli/config/export.md
[cmd-cget]:     ./docs/reference/cli/config/get.md
[cmd-cset]:     ./docs/reference/cli/config/set.md
[cmd-ctest]:    ./docs/reference/cli/config/test.md
[cmd-r]:         ./docs/reference/cli/resource/command.md
[cmd-rdelete]:  ./docs/reference/cli/resource/delete.md
[cmd-rexport]:  ./docs/reference/cli/resource/export.md
[cmd-rget]:     ./docs/reference/cli/resource/get.md
[cmd-rlist]:    ./docs/reference/cli/resource/list.md
[cmd-rschema]:  ./docs/reference/cli/resource/schema.md
[cmd-rset]:     ./docs/reference/cli/resource/set.md
[cmd-rtest]:    ./docs/reference/cli/resource/test.md
<!-- Configuration function links -->
[cfuncs]: ./docs/reference/schemas/config/functions/overview.md
[add()]: ./docs/reference/schemas/config/functions/add.md
[base64()]: ./docs/reference/schemas/config/functions/base64.md
[concat()]: ./docs/reference/schemas/config/functions/concat.md
[createArray()]: ./docs/reference/schemas/config/functions/createArray.md
[div()]: ./docs/reference/schemas/config/functions/div.md
[envvar()]: ./docs/reference/schemas/config/functions/envvar.md
[int()]: ./docs/reference/schemas/config/functions/int.md
[max()]: ./docs/reference/schemas/config/functions/max.md
[min()]: ./docs/reference/schemas/config/functions/min.md
[mod()]: ./docs/reference/schemas/config/functions/mod.md
[mul()]: ./docs/reference/schemas/config/functions/mul.md
[parameters()]: ./docs/reference/schemas/config/functions/parameters.md
[reference()]: ./docs/reference/schemas/config/functions/reference.md
[resourceId()]: ./docs/reference/schemas/config/functions/resourceId.md
[sub()]: ./docs/reference/schemas/config/functions/sub.md

<!-- Issue and PR links -->
[#107]: https://github.com/PowerShell/DSC/issues/107
[#121]: https://github.com/PowerShell/DSC/issues/121
[#127]: https://github.com/PowerShell/DSC/issues/127
[#129]: https://github.com/PowerShell/DSC/issues/129
[#130]: https://github.com/PowerShell/DSC/issues/130
[#133]: https://github.com/PowerShell/DSC/issues/133
[#139]: https://github.com/PowerShell/DSC/issues/139
[#150]: https://github.com/PowerShell/DSC/issues/150
[#156]: https://github.com/PowerShell/DSC/issues/156
[#157]: https://github.com/PowerShell/DSC/issues/157
[#158]: https://github.com/PowerShell/DSC/issues/158
[#159]: https://github.com/PowerShell/DSC/issues/159
[#162]: https://github.com/PowerShell/DSC/issues/162
[#163]: https://github.com/PowerShell/DSC/issues/163
[#165]: https://github.com/PowerShell/DSC/issues/165
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
[#183]: https://github.com/PowerShell/DSC/issues/183
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
[#218]: https://github.com/PowerShell/DSC/issues/218
[#226]: https://github.com/PowerShell/DSC/issues/226
[#227]: https://github.com/PowerShell/DSC/issues/227
[#240]: https://github.com/PowerShell/DSC/issues/240
[#241]: https://github.com/PowerShell/DSC/issues/241
[#248]: https://github.com/PowerShell/DSC/issues/248
[#251]: https://github.com/PowerShell/DSC/issues/251
[#252]: https://github.com/PowerShell/DSC/issues/252
[#253]: https://github.com/PowerShell/DSC/issues/253
[#258]: https://github.com/PowerShell/DSC/issues/258
[#263]: https://github.com/PowerShell/DSC/issues/263
[#266]: https://github.com/PowerShell/DSC/issues/266
[#271]: https://github.com/PowerShell/DSC/issues/271
[#274]: https://github.com/PowerShell/DSC/issues/274
[#279]: https://github.com/PowerShell/DSC/issues/279
[#284]: https://github.com/PowerShell/DSC/issues/284
[#286]: https://github.com/PowerShell/DSC/issues/286
[#287]: https://github.com/PowerShell/DSC/issues/287
[#290]: https://github.com/PowerShell/DSC/issues/290
[#291]: https://github.com/PowerShell/DSC/issues/291
[#294]: https://github.com/PowerShell/DSC/issues/294
[#299]: https://github.com/PowerShell/DSC/issues/299
[#302]: https://github.com/PowerShell/DSC/issues/302
[#303]: https://github.com/PowerShell/DSC/issues/303
[#305]: https://github.com/PowerShell/DSC/issues/305
[#307]: https://github.com/PowerShell/DSC/issues/307
[#309]: https://github.com/PowerShell/DSC/issues/309
[#310]: https://github.com/PowerShell/DSC/issues/310
[#311]: https://github.com/PowerShell/DSC/issues/311
[#313]: https://github.com/PowerShell/DSC/issues/313
[#314]: https://github.com/PowerShell/DSC/issues/314
[#317]: https://github.com/PowerShell/DSC/issues/317
[#318]: https://github.com/PowerShell/DSC/issues/318
[#321]: https://github.com/PowerShell/DSC/issues/321
[#322]: https://github.com/PowerShell/DSC/issues/322
[#323]: https://github.com/PowerShell/DSC/issues/323
[#329]: https://github.com/PowerShell/DSC/issues/329
[#333]: https://github.com/PowerShell/DSC/issues/333
[#334]: https://github.com/PowerShell/DSC/issues/334
[#335]: https://github.com/PowerShell/DSC/issues/335
[#336]: https://github.com/PowerShell/DSC/issues/336
[#338]: https://github.com/PowerShell/DSC/issues/338
[#342]: https://github.com/PowerShell/DSC/issues/342
[#344]: https://github.com/PowerShell/DSC/issues/344
[#346]: https://github.com/PowerShell/DSC/issues/346
[#347]: https://github.com/PowerShell/DSC/issues/347
[#348]: https://github.com/PowerShell/DSC/issues/348
[#349]: https://github.com/PowerShell/DSC/issues/349
[#351]: https://github.com/PowerShell/DSC/issues/351
[#352]: https://github.com/PowerShell/DSC/issues/352
[#353]: https://github.com/PowerShell/DSC/issues/353
[#354]: https://github.com/PowerShell/DSC/issues/354
[#356]: https://github.com/PowerShell/DSC/issues/356
[#357]: https://github.com/PowerShell/DSC/issues/357
[#358]: https://github.com/PowerShell/DSC/issues/358
[#360]: https://github.com/PowerShell/DSC/issues/360
[#361]: https://github.com/PowerShell/DSC/issues/361
[#362]: https://github.com/PowerShell/DSC/issues/362
[#364]: https://github.com/PowerShell/DSC/issues/364
[#368]: https://github.com/PowerShell/DSC/issues/368
[#371]: https://github.com/PowerShell/DSC/issues/371
[#373]: https://github.com/PowerShell/DSC/issues/373
[#375]: https://github.com/PowerShell/DSC/issues/375
[#376]: https://github.com/PowerShell/DSC/issues/376
[#377]: https://github.com/PowerShell/DSC/issues/377
[#379]: https://github.com/PowerShell/DSC/issues/379
[#382]: https://github.com/PowerShell/DSC/issues/382
[#385]: https://github.com/PowerShell/DSC/issues/385
[#388]: https://github.com/PowerShell/DSC/issues/388
[#390]: https://github.com/PowerShell/DSC/issues/390
[#397]: https://github.com/PowerShell/DSC/issues/397
[#400]: https://github.com/PowerShell/DSC/issues/400
[#401]: https://github.com/PowerShell/DSC/issues/401
[#405]: https://github.com/PowerShell/DSC/issues/405
[#407]: https://github.com/PowerShell/DSC/issues/407
[#410]: https://github.com/PowerShell/DSC/issues/410
[#412]: https://github.com/PowerShell/DSC/issues/412
[#429]: https://github.com/PowerShell/DSC/issues/429
[#431]: https://github.com/PowerShell/DSC/issues/431
[#432]: https://github.com/PowerShell/DSC/issues/432
[#434]: https://github.com/PowerShell/DSC/issues/434
[#435]: https://github.com/PowerShell/DSC/issues/435
[#436]: https://github.com/PowerShell/DSC/issues/436
[#438]: https://github.com/PowerShell/DSC/issues/438
[#439]: https://github.com/PowerShell/DSC/issues/439
[#441]: https://github.com/PowerShell/DSC/issues/441
[#444]: https://github.com/PowerShell/DSC/issues/444
[#445]: https://github.com/PowerShell/DSC/issues/445
[#45]:  https://github.com/PowerShell/DSC/issues/45
[#452]: https://github.com/PowerShell/DSC/issues/452
[#454]: https://github.com/PowerShell/DSC/issues/454
[#457]: https://github.com/PowerShell/DSC/issues/457
[#462]: https://github.com/PowerShell/DSC/issues/462
[#464]: https://github.com/PowerShell/DSC/issues/464
[#465]: https://github.com/PowerShell/DSC/issues/465
[#468]: https://github.com/PowerShell/DSC/issues/468
[#469]: https://github.com/PowerShell/DSC/issues/469
[#475]: https://github.com/PowerShell/DSC/issues/475
[#477]: https://github.com/PowerShell/DSC/issues/477
[#480]: https://github.com/PowerShell/DSC/issues/480
[#481]: https://github.com/PowerShell/DSC/issues/481
[#482]: https://github.com/PowerShell/DSC/issues/482
[#484]: https://github.com/PowerShell/DSC/issues/484
[#485]: https://github.com/PowerShell/DSC/issues/485
[#488]: https://github.com/PowerShell/DSC/issues/488
[#487]: https://github.com/PowerShell/DSC/issues/487
[#489]: https://github.com/PowerShell/DSC/issues/489
[#49]:  https://github.com/PowerShell/DSC/issues/49
[#491]: https://github.com/PowerShell/DSC/issues/491
[#493]: https://github.com/PowerShell/DSC/issues/493
[#494]: https://github.com/PowerShell/DSC/issues/494
[#495]: https://github.com/PowerShell/DSC/issues/495
[#497]: https://github.com/PowerShell/DSC/issues/497
[#499]: https://github.com/PowerShell/DSC/issues/499
[#500]: https://github.com/PowerShell/DSC/issues/500
[#503]: https://github.com/PowerShell/DSC/issues/503
[#504]: https://github.com/PowerShell/DSC/issues/504
[#505]: https://github.com/PowerShell/DSC/issues/505
[#506]: https://github.com/PowerShell/DSC/issues/506
[#509]: https://github.com/PowerShell/DSC/issues/509
[#511]: https://github.com/PowerShell/DSC/issues/511
[#512]: https://github.com/PowerShell/DSC/issues/512
[#514]: https://github.com/PowerShell/DSC/issues/514
[#516]: https://github.com/PowerShell/DSC/issues/516
[#518]: https://github.com/PowerShell/DSC/issues/518
[#524]: https://github.com/PowerShell/DSC/issues/524
[#525]: https://github.com/PowerShell/DSC/issues/525
[#527]: https://github.com/PowerShell/DSC/issues/527
[#528]: https://github.com/PowerShell/DSC/issues/528
[#530]: https://github.com/PowerShell/DSC/issues/530
[#533]: https://github.com/PowerShell/DSC/issues/533
[#541]: https://github.com/PowerShell/DSC/issues/541
[#548]: https://github.com/PowerShell/DSC/issues/548
[#549]: https://github.com/PowerShell/DSC/issues/549
[#551]: https://github.com/PowerShell/DSC/issues/551
[#552]: https://github.com/PowerShell/DSC/issues/552
[#553]: https://github.com/PowerShell/DSC/issues/553
[#555]: https://github.com/PowerShell/DSC/issues/555
[#556]: https://github.com/PowerShell/DSC/issues/556
[#561]: https://github.com/PowerShell/DSC/issues/561
[#564]: https://github.com/PowerShell/DSC/issues/564
[#565]: https://github.com/PowerShell/DSC/issues/565
[#568]: https://github.com/PowerShell/DSC/issues/568
[#572]: https://github.com/PowerShell/DSC/issues/572
[#573]: https://github.com/PowerShell/DSC/issues/573
[#577]: https://github.com/PowerShell/DSC/issues/577
[#57]:  https://github.com/PowerShell/DSC/issues/57
[#70]:  https://github.com/PowerShell/DSC/issues/70
[#73]:  https://github.com/PowerShell/DSC/issues/73
[#75]:  https://github.com/PowerShell/DSC/issues/75
[#89]:  https://github.com/PowerShell/DSC/issues/89
[#98]:  https://github.com/PowerShell/DSC/issues/98
[#157]: https://github.com/PowerShell/DSC/issues/157
[#436]: https://github.com/PowerShell/DSC/issues/436
[#485]: https://github.com/PowerShell/DSC/issues/485
[#537]: https://github.com/PowerShell/DSC/issues/537
