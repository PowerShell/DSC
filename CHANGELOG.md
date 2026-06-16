---
title: "Desired State Configuration changelog"
description: >-
  A log of the changes for releases of DSC.
ms.topic: whats-new
ms.date: 03/25/2025
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

All notable changes to DSC after the `3.0.0` release are documented in this file. The format is
based on [Keep a Changelog][m1], and DSC adheres to [Semantic Versioning][m2].

To see the changes for the earlier development of DSC before version `3.0.0`, see the
[DSC prerelease changelog][m3] on GitHub.

<!-- Meta links -->
[m1]: https://keepachangelog.com/en/1.1.0/
[m2]: https://semver.org/spec/v2.0.0.html
[m3]: https://github.com/PowerShell/DSC/blob/main/docs/prerelease-changelog.md

## Unreleased

This section includes a summary of user-facing changes since the last release. For the full list of
changes since the last release, see the [diff on GitHub][unreleased].

<!-- Unreleased comparison link - always update version to match last release tag -->
[unreleased]: https://github.com/PowerShell/DSC/compare/v3.1.1...main

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

## [v3.2.2][release-v3.2.2] - 2026-06-16

This section includes a summary of changes for the `3.2.2` release. For the full list of changes
in this release, see the [diff on GitHub][compare-v3.2.2].

<!-- Release links -->
[release-v3.2.2]: https://github.com/PowerShell/DSC/releases/tag/v3.2.2 "Link to the DSC v3.2.2 release on GitHub"
[compare-v3.2.2]: https://github.com/PowerShell/DSC/compare/v3.2.1...v3.2.2

### Fixed

- Fixed several bugs with the `Microsoft.DSC.Transitional/PowerShellScript` and
`Microsoft.DSC.Transitional/WindowsPowerShellScript` resources:

  - Scripts can now raise non-terminating errors and continue processing. In previous releases the
    resources considered a script that raised _any_ errors - even if the author specifically
    handled them or set the error action preference to `Ignore` - caused the resource to mark the
    script as failing. This prevented scripts for these resources from doing any error handling
    because _any_ error stopped further execution.
  - The resources now emit all trace messages from the script when it fails _before_ writing the
    final error message and exiting. In previous releases a race condition prevented some messages
    from emitting before the resource exited.

  <details><summary>Related work items</summary>

  - Issues:
    - [#1547][#1547]
  - PRs:
    - [#1554][#1554]
    - [#1557][#1557]
    - [#1558][#1558]
    - [#1562][#1562]

  </details>

## [v3.2.1][release-v3.2.1] - 2026-06-16

This section includes a summary of changes for the `3.2.1` release. For the full list of changes
in this release, see the [diff on GitHub][compare-v3.2.1].

<!-- Release links -->
[release-v3.2.1]: https://github.com/PowerShell/DSC/releases/tag/v3.2.1 "Link to the DSC v3.2.1 release on GitHub"
[compare-v3.2.1]: https://github.com/PowerShell/DSC/compare/v3.2.0...v3.2.1

### Fixed

- Resolved a design issue that caused failures when passing a resource path to an adapter with the
  resource path argument when the path includes any spaces. Starting with this release the resource
  path argument now includes an optional `includeQuotes` field to wrap the path in quotes before
  constructing the command invocation for the resource.

  This new option is set for the `Microsoft.Adapter/PowerShell` and
  `Microsoft.Adapter/WindowsPowerShell` adapters to enable them to correctly invoke PSDSC resources
  installed in a path that contains spaces.

   <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs:
    - [#1534][#1534]
    - [#1535][#1535]

  </details>

## [v3.2.0][release-v3.2.0] - 2026-06-08

This section includes a summary of changes for the `3.2.0` minor release. For the full list of
changes in this release, see the [diff on GitHub][compare-v3.2.0].

<!-- Release links -->
[release-v3.2.0]: https://github.com/PowerShell/DSC/releases/tag/v3.2.0 "Link to the DSC v3.2.0 release on GitHub"
[compare-v3.2.0]: https://github.com/PowerShell/DSC/compare/v3.1.3..v3.2.0

### Added

- Added the `dsc mcp` command to start DSC in server mode. In this mode, DSC acts as a JSON RPC
  server. For this release, the server primarily enables use as a Model Context Protocol (MCP)
  provider. The server supports the following functions (tools in MCP):

  - `list_resources`
  - `list_adapted_resources`
  - `show_dsc_resource`
  - `list_dsc_resource`
  - `invoke_dsc_resource`
  - `invoke_dsc_config`

  <details><summary>Related work items</summary>

  - Issues:
    - [#1093][#1093]
  - PRs:
    - [#1092][#1092]
    - [#1101][#1101]
    - [#1105][#1105]
    - [#1162][#1162]
    - [#1174][#1174]

  </details>

- Added the `dsc function list` command to enumerate the available DSC configuration functions.

  <details><summary>Related work items</summary>

  - Issues: _None_
  - PRs: [#959][#959]

  </details>

- Added the `--what-if` CLI option to the `dsc resource set` and `dsc resource delete` commands
  to enable checking how a resource invocation will modify system state outside of operating on a
  configuration document.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs:
    - [#1407][#1407]

  </details>

- Added the `--output-format` CLI option to the `dsc resource delete` command, enabling you to
  control the output format for that command now that it supports invoking the **Delete** operation
  in what-if mode, which returns output. When you invoke the command without the `--what-if` option
  DSC returns no data to stdout.

  <details><summary>Related work items</summary>

  - Issues:
    - [#566][#566]
  - PRs:
    - [#1333][#1333]
    - [#1407][#1407]

  </details>

- Added the `--noop` and `--dry-run` aliases for the `--what-if` CLI option in the `dsc config set`,
  `dsc resource set`, and `dsc resource delete` commands.

  <details><summary>Related work items</summary>

  - Issues:
    - [#566][#566]
    - [#893][#893]
  - PRs:
    - [#1121][#1121]
    - [#1333][#1333]

  </details>

- Added support for using both the `--parameters-file` and `--parameters` CLI options for the same
  `dsc config` subcommands, enabling you to use a parameters file and _override_ the values in that
  file with the `--parameters` option. Prior to this release you could use _either_ `--parameters`
  or `--parameters-file` but not both with the same command execution.

  <details><summary>Related work items</summary>

  - Issues:
    - [#1213][#1213]
  - PRs:
    - [#1215][#1215]

  </details>

- Added the `--version` CLI option to the `dsc resource *` commands. Starting with this release,
  you can provide a version requirement to indicate to DSC which version of the resource to invoke.

  If DSC doesn't discover the resource with a version that is valid for the requirement defined by
  the `--version` option DSC raises an error.

  <details><summary>Related work items</summary>

  - Issues:
    - [#543][#543]
    - [#942][#942]
  - PRs:
    - [#1077][#1077]
    - [#1449][#1449]

  </details>

- Improved parsing and validation for CLI arguments and parameters that specify the fully qualified
  type name for a resource. Prior to this release, specifying an invalidly-constructed type name
  only raised an error when DSC failed to discover the given resource.

  Starting with this release, DSC validates the type name and provides detailed validation errors
  when the type name is incorrectly structured.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs:
    - [#1449][#1449]

  </details>

- Added the `executionInformation` field to replace the use of the `Microsoft.DSC` namespace in the
  `metadata` field for DSC command output. In this release, the execution information is duplicated
  in both `executionInformation` and `metadata.Microsoft.DSC` to maintain backwards compatibility
  for tools and scripts that process DSC output. In DSC version `4.0.0`, the results will no longer
  include the `metadata.Microsoft.DSC` field.

  <details><summary>Related work items</summary>

  - Issues:
    - [#1369][#1369]
  - PRs:
    - [#1387][#1387]

  </details>

- Added the `y2j` tool to the DSC package. This tool bidirectionaly converts JSON to YAML and YAML
  to JSON. It's included in the package for convenience and testing scenarios.

  <details><summary>Related work items</summary>

  - Issues:
    - [#1369][#1369]
  - PRs:
    - [#1387][#1387]

  </details>

- Added support for defining extensions that can retrieve secrets and the [`secret()`][`secret()`]
  configuration function for retrieving secrets from an extension.

  <details><summary>Related work items</summary>

  - Issues:
    - [#685][#685]
  - PRs:
    - [#908][#908]
    - [#1046][#1046]
    - [#1079][#1079]
    - [#1083][#1083]

  </details>

- Added support for defining extensions that can retrieve configuration documents defined in any
  format and use them with DSC. These extensions can convert a file that defines a configuration in
  a way DSC can't parse into a configuration document, enabling you to use different data formats
  or domain specific languages (DSLs).

  <details><summary>Related work items</summary>

  - Issues:
    - [#976][#976]
  - PRs:
    - [#997][#997]

  </details>

- Added the experimental `Microsoft.PowerShell/Discover` extension to find DSC manifests packaged
  with PowerShell modules. This enables PowerShell developers to define and publish DSC extensions
  and resources with their PowerShell modules. THe extension also discovers adapted resource
  manifests and manifest lists.

  <details><summary>Related work items</summary>

  - Issues:
    - [#913][#913]
  - PRs:
    - [#1071][#1071]

  </details>

- Added support for synthetic **Export** operations for resources that don't have the `export`
  capability. For these resources, DSC instead invokes the **Get** operation on the resource, using
  the defined `properties` for the instance. The actual state of the resource is then inserted into
  the export configuration document.

  <details><summary>Related work items</summary>

  - Issues:
    - [#428][#428]
  - PRs:
    - [#1035][#1035]

  </details>

- Added the top-level `function` field to configuration documents, enabling you to define custom
  functions you can use in the rest of the configuration document.

  <details><summary>Related work items</summary>

  - Issues:
    - [#1018][#1018]
  - PRs:
    - [#1096][#1096]

  </details>

- Added the top-level `outputs` field to configuration documents, enabling you to retrieve custom
  representations of the data emitted by resources in the document for final output.

  <details><summary>Related work items</summary>

  - Issues:
    - [#898][#898]
  - PRs:
    - [#1010][#1010]

  </details>

- Added the top-level `directives` field to configuration documents, enabling you to control how
  DSC processes the document. DSC supports the following directives:

  - `resourceDiscovery` - Choose whether DSC raises an error for an undiscovered resource during
    initial validation of a configuration document.

    By default and when you set this directive to `preDeployment`, DSC performs discovery for
    resources and extensions and then validates whether every resource used in the configuration
    document exists and is available. If any resource is used in the document but not discovered,
    DSC raises an error _without_ invoking any of the resources.

    Set this directive to `duringDeployment` to defer raising an error for a missing resource when
    the configuration document itself installs a resource that the configuration depends on. When
    you do, DSC performs discovery again when processing a resource that wasn't initially
    discovered and only raises an error if the resource isn't available at that time.

  - `securityContext` - Choose whether DSC validates that the configuration is being processed in a
    specific security context prior to invoking any resources in the configuration document. This
    replaces the usage of the `securityContext` field in the `Microsoft.DSC` namespace of the
    document's `metadata` field. Using metadata to define this directive now raises a warning that
    indicates the field is deprecated.

    If you define both `directives.securityContext` and `metadata.Microsoft.DSC.securityContext`,
    DSC uses the value from the directive.

  - `version` - Choose whether DSC validates that the configuration is being processed by a
    compatible version of DSC itself. When you don't define this directive, DSC processes the
    configuration document as normal. When you define this directive as a semantic version
    requirement, DSC raises an error if the version of DSC processing the document isn't valid for
    that requirement.

    This enables you to require specific versions of DSC in production and raise an error for
    incompatible configuration documents and DSC versions _without_ invoking any resources in the
    document.

  <details><summary>Related work items</summary>

  - Issues:
    - [#1170][#1170]
    - [#1369][#1369]
    - [#1343][#1343]
  - PRs:
    - [#1366][#1366]
    - [#1387][#1387]

  </details>

- Added new fields to resource instances in a configuration document:

  - `condition` enables you to only invoke the instance when the value for this field evaluates to
    `true`.
  - `copy` enables you to define one resource instance that expands into multiple instances with
    shared property values.

    > [!NOTE]
    > This field was added during the preview releases for DSC 3.2.0. Some early adopters began
    > using the `copy` field. To avoid breaking those users the functionality remains in the DSC
    > engine but is _deprecated_. If you use the `copy` field DSC raises a warning to indicate that
    > you should not rely on this functionality as it will be removed in DSC version `4.0.0`.

  - `requireVersion` enables you to pin a resource instance to a specific version or a range of
    versions. This field is aliased to `apiVersion` for compatibility purposes.

  <details><summary>Related work items</summary>

  - Issues:
    - [#496][#496]
    - [#543][#543]
    - [#942][#942]
    - [#972][#972]
  - PRs:
    - [#978][#978]
    - [#1077][#1077]
    - [#1099][#1099]
    - [#1430][#1430]

  </details>

- Added new configuration functions:

  - `and()` - evaluates to `true` if all parameters evaluate to `true`
  - `bool()` - converts a string or numerical value to `true` or `false`
  - `true()` - returns the boolean value `true`
  - `false()` - returns the boolean value `false`
  - `not()` - returns the opposite of the input boolean value
  - `or()` - evaluates to `true` if any parameter evaluates to `true`
  - `less()` - evaluates to `true` if the first parameter is less than the second parameter.
    Supports comparing numbers and strings.
  - `lessOrEquals()` - evaluates to `true` if the first parameter is less than or equal to the
    second parameter. Supports comparing numbers and strings.
  - `greater()` - evaluates to `true` if the first parameter is greater than the second parameter.
    Supports comparing numbers and strings.
  - `greaterOrEquals()` - evaluates to `true` if the first parameter is greater than or equal to
    the second parameter. Supports comparing numbers and strings.
  - `coalesce()`
  - `createObject()`
  - `null()`
  - `contains()`
  - `union()`
  - `length()`
  - `empty()`
  - `secret()`
  - `endsWith()`
  - `startsWith()`
  - `utcNow()`
  - `uniqueString()`
  - `string()`
  - `array()`
  - `first()`
  - `indexOf()`
  - `lastIndexOf()`
  - `skip()`
  - `join()`
  - `context()`
  - `intersection()`
  - `range()`
  - `substring()`
  - `base64ToString()`
  - `toLower()`
  - `toUpper()`
  - `trim()`
  - `items()`
  - `tryGet()`
  - `uriComponent()`
  - `uriComponentToString()`
  - `json()`
  - `uri()`
  - `last()`
  - `copyIndex()`
  - `tryIndexFromEnd()`
  - `take()`
  - `parseCidr()`
  - `cidrHost()`
  - `cidrSubnet()`
  - `objectKeys()`
  - `tryWhich()`
  - `shallowMerge()`
  - `dataUri()`
  - `dataUriToString()`
  - `lambda()`
  - `lambdaVariables()`
  - `map()`
  - `filter()`

  <details><summary>Related work items</summary>

  - Issues:
    - [#57][#57]
    - [#976][#976]
    - [#980][#980]
  - PRs:
    - [#908][#908]
    - [#979][#979]
    - [#990][#990]
    - [#999][#999]
    - [#1005][#1005]
    - [#1032][#1032]
    - [#1040][#1040]
    - [#1041][#1041]
    - [#1087][#1087]
    - [#1085][#1085]
    - [#1086][#1086]
    - [#1099][#1099]
    - [#1103][#1103]
    - [#1138][#1138]
    - [#1142][#1142]
    - [#1145][#1145]
    - [#1148][#1148]
    - [#1156][#1156]
    - [#1175][#1175]
    - [#1178][#1178]
    - [#1183][#1183]
    - [#1176][#1176]
    - [#1190][#1190]
    - [#1192][#1192]
    - [#1194][#1194]
    - [#1211][#1211]
    - [#1219][#1219]
    - [#1227][#1227]
    - [#1230][#1230]
    - [#1238][#1238]
    - [#1274][#1274]
    - [#1332][#1332]

  </details>

- Added support for using configuration functions in the `name` field for resource instances in a
  configuration document. Prior to this release, specifying a configuration function for this field
  raised an error.

  <details><summary>Related work items</summary>

  - Issues:
    - [#1116][#1116]
  - PRs:
    - [#1117][#1117]

  </details>

- Added in-memory caching for discovered resources, including adapted resources, and extensions.
  This improves performance when DSC needs to run discovery more than once, such as when using
  implicitly adapted resource instances.

  <details><summary>Related work items</summary>

  - Issues:
    - _None_.
  - PRs:
    - [#1132][#1132]

  </details>

- Added the `metadata` field to resource and extension manifests, enabling authors to define any
  additional data they want to include with their resources and extensions.

  <details><summary>Related work items</summary>

  - Issues:
    - _None_.
  - PRs:
    - [#1198][#1198]

  </details>

- Added the `condition` field to resource and extension manifests, enabling authors to define use
  DSC configuration expressions to define a check for whether the resource is usable on a system.
  This field affects whether DSC discards the resource or extension during discovery:

  - If the manifest doesn't define `condition` or defines `condition` with an expression that
    evaluates to `true`, DSC discovers the resource or extension as normal.
  - If the manifest defines `condition` and the expression evaluates to `false`, DSC discards the
    resource or extension during discovery and writes a debug message notifying the user that the
    manifest's condition wasn't met.

  This is particularly useful for not displaying or attempting to use resources and extensions that
  have external prerequisites, like the PowerShell adapters depending on `pwsh` being available on
  the system.

  Starting with this release, built-in DSC resources use this field to prevent confusion and errors
  for users where a resource or extension isn't functional because of missing external
  dependencies.

  <details><summary>Related work items</summary>

  - Issues:
    - _None_.
  - PRs:
    - [#1194][#1194]

  </details>

- Added the `directives` field to resource instances in a configuration document to provide
  per-instance overrides for how DSC should process the resource. In this release, you can define
  the following directives for a resource instance:

  - `requireAdapter` - indicates that DSC should use the defined fully qualified type name for the
    adapted resource instance. When this directive isn't specified, DSC uses the first discovered
    adapter that can invoke the adapted resource instance. This directive has no effect on
    nonadapted resource instances.
  - `securityContext` - indicates that DSC should validate the current security context against
    this directive before invoking the resource. This value overrides the
    `directives.securityContext` for the top level of the configuration document. This enables you
    to selectively require or forbid elevated security contexts for a specific resource instance.

- Added a new "what-if" argument type for the `set.args` and `delete.args` fields in resource manifests,
  enabling authors to define an argument to pass to the resource command to indicate it should
  operate in what-if mode.

  <details><summary>Related work items</summary>

  - Issues:
    - [#1361][#1361]
  - PRs:
    - [#1374][#1374]
    - [#1377][#1377]

  </details>

- Added support for resources returning the `metadata._refreshEnv` field in the output for **Set**
  operations on Windows systems to indicate that DSC should update the environment variables before
  invoking the next resource. This enables resources that install software or modify environment
  variables that other resource instances depend on to advertise that environment variables ned to
  be refreshed. This is a common requirement when a configuration document both installs and invokes
  software in the same document.

  <details><summary>Related work items</summary>

  - Issues:
    - [#430][#430]
  - PRs:
    - [#1408][#1408]

  </details>

- Added support for deprecating resources and extensions by defining the `deprecationMessage` field
  in resource and extension manifests. When a manifest defines this field, DSC raises a warning to
  users for that extension or resource indicating that it's deprecated and surfacing the message to
  the user.

  <details><summary>Related work items</summary>

  - Issues:
    - _[#1368][#1368]_
  - PRs:
    - [#1398][#1398]

  </details>

- Added the `set.whatIfReturns` field to resource manifest enabling a resource to return
  differently shaped data for **Set** operations depending on whether the user invokes the command
  in what-if mode. Prior to this release the resource needed to return the same data structure for
  actual **Set** operations and those operations in what-if mode. The field has the same possible
  values as `set.returns`.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs:
    - [#1386][#1386]

  </details>

- Added support for defining and discovering adapted resource manifests. Providing an adapted
  resource manifest:
  
  - Improves the performance for discovering adapted resources because it doesn't rely on the adapter
    to find, parse, and surface resources to DSC.
  - Improves the validation for adapted resources because it enables the adapted resource author to
    provide a JSON Schema for adapted resource instances instead of delegating validation of the
    instance to the adapter, which must dynamically validate the instance.

  In this release the new `Microsoft.Adapter/PowerShell` adapter uses adapted resource manifests
  for discovery. The `Microsoft.Adapter/WindowsPowerShell` adapter still has to dynamically
  discover PSDSC resources because it depends on the PSDSC v1.1 engine.

  <details><summary>Related work items</summary>

  - Issues:
    - [#872][#872]
    - [#1352][#1352]
  - PRs:
    - [#1375][#1375]
    - [#1401][#1401]

  </details>

- Added support for defining multiple resource and extension manifests in files that use the naming
  convention `.dsc.manifests.<json|yaml>`. The files are _manifest lists_. Manifest lists simplify
  distribution for resources and extensions where you can keep the manifests in a single file
  instead of defining one file for every extension, resource, and adapted resource manifest.

  <details><summary>Related work items</summary>

  - Issues:
    - [#1160][#1160]
  - PRs:
    - [#1187][#1187]

  </details>

- Added new resources for configuring SSHD:

  - `Microsoft.OpenSSH.SSHD/sshd_config` - Manages the configuration file for an SSH server.
  - `Microsoft.OpenSSH.SSHD/Subsystem` - Manages an entry for the `Subsystem` keyword in the
    configuration file for an SSH server.
  - `Microsoft.OpenSSH.SSHD/SubsystemList` - Manages multiple entries for the `Subsystem` keyword
    in the configuration file for an SSH server.
  - `Microsoft.OpenSSH.SSHD/Windows` - Manages global settings for an SSH server on Windows.

  <details><summary>Related work items</summary>

  - Issues:
    - [#1030][#1030]
    - [#1340][#1340]
  - PRs:
    - [#869][#869]
    - [#958][#958]
    - [#1004][#1004]
    - [#1275][#1275]
    - [#1284][#1284]
    - [#1307][#1307]
    - [#1327][#1327]
    - [#1348][#1348]
    - [#1367][#1367]

  </details>

- Added new resources for migrating from imperative configuration scripts to idempotent DSC
  configuration documents:

  - `Microsoft.DSC.Transitional/PowerShellScript` - Define PowerShell script blocks to invoke for
    **Get**, **Set**, and **Test** operations.
  - `Microsoft.DSC.Transitional/WindowsPowerShellScript` - Define Windows PowerShell script blocks
    to invoke for **Get**, **Set**, and **Test** operations.

  <details><summary>Related work items</summary>

  - Issues:
    - [#885][#885]
  - PRs:
    - [#869][#869]

  </details>

- Added new adapters for PowerShell and Windows PowerShell, replacing the existing adapters which
  are now marked as deprecated:

  - `Microsoft.Adapter/PowerShell` replaces `Microsoft.DSC/PowerShell`.
  - `Microsoft.Adapter/WindowsPowerShell` replaces `Microsoft.Windows/WindowsPowerShell`.

  All improvements and new functionality will be implemented for the `Microsoft.Adapter/*` types.
  You should begin migrating usage to the new adapters. In DSC version `4.0.0`, the deprecated
  adapters will be removed.

  You can explicitly select which adapter to use for a resource instance in a configuration document
  with the `requireAdapter` directive. By default, DSC will use the new adapters when the instance
  doesn't specify a directive.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs:
    - [#1331][#1331]

  </details>

- Added new built-in DSC resources for managing Windows systems:

  - `Microsoft.Windows/UpdateList`
  - `Microsoft.Windows/OptionalFeatureList`
  - `Microsoft.Windows/Service`
  - `Microsoft.Windows/FeatureOnDemandList`
  - `Microsoft.Windows/FirewallRuleList`

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs:
    - [#1351][#1351]
    - [#1426][#1426]
    - [#1432][#1432]
    - [#1433][#1433]
    - [#1453][#1453]

  </details>

- Added support for the **Set** operation to the `Microsoft.Windows/WMI` adapter.

  <details><summary>Related work items</summary>

  - Issues:
    - [#475][#475]
  - PRs:
    - [#976][#976]

  </details>

- Added support for filtered **Export** operations to the `Microsoft.DSC/PowerShell` adapter.
  Starting with this release, class-based PSDSC resources can implement a filtered export operation
  to return a subset of instances that exist on the system.

  <details><summary>Related work items</summary>

  - Issues:
    - [#1277][#1277]
  - PRs:
    - [#1278][#1278]

  </details>

- Added the canonical properties `_name` and `_securityContext`. These canonical properties are
  only emitted by resources during an **Export** operation. When a resource defines either of these
  properties and includes them in the output data for an instance during an **Export** operation
  DSC hoists those values to the `name` and `metadata.Microsoft.Dsc.securityContext` in the
  exported resource instance.

  <details><summary>Related work items</summary>

  - Issues: [#598][#598]
  - PRs: [#884][#884]

  </details>

- Added the `_metadata` canonical property. When a resource includes this canonical property in
  its resource instance JSON Schema:

  - DSC passes the properties defined in the `metadata` field of the resource instance to the
    resource by inserting the metadata into the `_metadata` key of the resource properties.
  - DSC hoists any data returned by the resource in the `_metadata` property from an **Export**
    operation into the `metadata` field in the exported resource instance.

  <details><summary>Related work items</summary>

  - Issues: [#467][#467]
  - PRs:
    - [#947][#947]
    - [#1069][#1069]

  </details>

- Added the `_restartRequired` canonical property. When a resource includes this canonical property
  in its resource instance JSON Schema, the resource can include the canonical property in its
  output to indicate that the machine, specific services, or specific processes need to be
  restarted.

  This canonical property replaces `_rebootRequested` which was defined but had no integration with
  the DSC engine.

  <details><summary>Related work items</summary>

  - Issues:
    - [#50][#50]
    - [#1236][#1236]
    - [#1372][#1372]
  - PRs:
    - [#975][#975]
    - [#1260][#1260]
    - [#1417][#1417]

  </details>

- Added support for defining resource type arguments in the `args` field for resource adapter
  manifests to simplify implementing adapters that operate on a single resource instance. You can
  define this argument to pass the fully qualified type name of the adapted resource instance to
  the adapter.

  <details><summary>Related work items</summary>

  - Issues: [#931][#931]
  - PRs: [#1124][#1124]

  </details>

- Renamed the `adapter.config` field in resource manifests to `inputKind`. The old name is retained
  for compatibility purposes.

  <details><summary>Related work items</summary>

  - Issues: [#931][#931]
  - PRs: [#1124][#1124]

  </details>

- Improved the error message for duplicate resource instances to include the fully qualified type
  name for the duplicated instance. DSC raises an error for duplicate instances when more than one
  instance has the same fully qualified type name (`type` field) and instance name (`name`) field.

  Prior to this release, DSC only included the instance name in the error message, implying that
  defining _any_ two instances with the same name, even with different resource types, is invalid.

  <details><summary>Related work items</summary>

  - Issues: [#1022][#1022]
  - PRs: [#1029][#1029]

  </details>

- Added the canonical `_name` property to the `Microsoft/OSInfo` resource, so exporting that
  resource automatically defines the exported instance's `name` field as:
  
  `<OS Family> <OS version> [<OS architecture>]`

  Where the final segment is only defined if the operating system architecture is discoverable.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#1038][#1038]

  </details>

### Fixed

- Fixed handling for secure strings and objects in the engine to prevent accidentally leaking
  secure data in trace messaging and output.

  <details><summary>Related work items</summary>

  - Issues:
    - [#1084][#1084]
    - [#1123][#1123]
  - PRs: [#1127][#1127]

  </details>

- Fixed a bug that prevented DSC from correctly deserializing data from UTF-8 files with a byte
  order mark (BOM) by removing the BOM prior to deserialization.

  <details><summary>Related work items</summary>

  - Issues: [#829][#829]
  - PRs: [#924][#924]

  </details>

- Fixed a bug that caused DSC to raise a terminating error when it encounters a DSC manifest that
  the engine can't parse during discovery. This caused an invalid manifest to break all DSC
  operations. Starting with this release DSC raises an informational message about manifests it
  can't correctly read and continues processing.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#1445][#1445]

  </details>

- Fixed a bug that caused DSC to try writing to a broken pipe when piped to commands that stop
  processing when the piped-to command no longer needs input from the `dsc` CLI, like
  `dsc resource list | Select-Object -First 1` in PowerShell.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#1154][#1154]

  </details>

- Fixed a bug that prevented referencing other parameters in the `defaultValue` field for a
  parameter definition in a configuration document.

  <details><summary>Related work items</summary>

  - Issues: [#1129][#1129]
  - PRs: [#1159][#1159]

  </details>

- Fixed the automatic name generation for exported instances of resources that don't define the
  canonical `_name` property. Prior to this release, the exported instances defined the `name`
  field as `<fully qualified type name>-<zero-index>`, like `Microsoft/OSInfo-0`. This generated
  string is invalid for the JSON Schema that validates the `name` field, which forbids forward
  slashes (`/`).

  Starting with this release the automatic name generation uses only the final segment of the fully
  qualified type name and starting the index at `1` instead of `0` (`<name>-<index>`), like
  `OSInfo-1`.

  <details><summary>Related work items</summary>

  - Issues: [#845][#845]
  - PRs: [#1038][#1038]

  </details>

- Fixed a bug that sometimes caused unnecessary parsing of configuration expressions for implicit
  adapted resource instances, causing DSC to raise an erroneous error.

  <details><summary>Related work items</summary>

  - Issues: [#1024][#1024]
  - PRs: [#1031][#1031]

  </details>

- Fixed a bug that prevented DSC from correctly invoking the **Delete** operation for resources
  that have the `_exist` canonical property, have the `delete` capability, and don't have the
  `setHandlesExist` capability. Prior to this release, invoking the `dsc resource set` command
  to remove an instance by setting `_exist` to `false` failed to correctly invoke the **Delete**
  operation.

  Starting with this release, DSC correctly invokes the **Delete** operation to remove the resource
  instance. This ensures the behavior of a resource instance is consistent when using `dsc config
  set` and `dsc resource set` to remove a specific resource instance.

  <details><summary>Related work items</summary>

  - Issues: [#1268][#1268]
  - PRs: [#1317][#1317]

  </details>

- Fixed discovery of the `Export()` method for class-based PSDSC resources in the
  `Microsoft.Windows/WindowsPowerShell` and `Microsoft.DSC/PowerShell` adapters.

  <details><summary>Related work items</summary>

  - Issues: [#853][#853]
  - PRs:
    - [#876][#876]
    - [#877][#877]

  </details>

- Fixed property discovery for PSDSC resources in the `Microsoft.Windows/WindowsPowerShell`
  adapter to no longer emit the `DependsOn` or `PSDscRunAsCredential` common properties, bringing
  the resource property definitions into alignment with `Microsoft.DSC/PowerShell`.

  <details><summary>Related work items</summary>

  - Issues: [#878][#878]
  - PRs: [#879][#879]

  </details>

- Fixed a bug for class-based PSDSC resources in the `Microsoft.DSC/PowerShell` adapter that
  caused PSDSC resource classes with a `[SecureString]` type property from instantiating.

  <details><summary>Related work items</summary>

  - Issues: [#1207][#1207]
  - PRs: [#1208][#1208]

  </details>

- Fixed handling for passing username and password (`[pscredential]` PowerShell type) to adapted
  PSDSC resources to correctly convert secure objects into credentials before invoking the adapted
  resource.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#1308][#1308]

  </details>

- Fixed the `Microsoft.Windows/WindowsPowerShell` adapter to prevent raising an error when the
  `PSModulePath` environmental variable contains any empty path segments. Prior to this release,
  the adapter would raise an error because PSDSC discovery fails when `PSModulePath` defines any
  empty segments.

  <details><summary>Related work items</summary>

  - Issues: [#1095][#1095]
  - PRs: [#1097][#1097]

  </details>

- Fixed the `Microsoft.Windows/WindowsPowerShell` adapter to suppress writing progress information
  from PSDSC. DSC already includes its own progress reporting and the Windows PowerShell progress
  implementation can consume extra resources, impacting the system.

  <details><summary>Related work items</summary>

  - Issues: [#923][#923]
  - PRs: [#964][#964]

  </details>

- Fixed a bug in the build for the `Microsoft.Windows/WMI` adapter to ensure the required PowerShell
  files are included in the installation archive. In previous releases the archive included the
  resource manifest but not the related script and data files the adapter depends on, making it
  unusable.

  <details><summary>Related work items</summary>

  - Issues: [#967][#967]
  - PRs: [#969][#969]

  </details>

- Fixed a bug in the `Microsoft.DSC.Debug/Echo` resource that caused the JSON Schema to incorrectly
  represent a resource instance and prevent any validation errors, even when the `output` includes
  malformed secure string or secure object values.

  <details><summary>Related work items</summary>

  - Issues: [#1202][#1202]
  - PRs: [#1205][#1205]

  </details>

## [v3.1.3][release-v3.1.3] - 2026-06-15

This section includes a summary of changes for the `3.1.3` patch release. For the full list of
changes in this release, see the [diff on GitHub][compare-v3.1.3].

<!-- Release links -->
[release-v3.1.3]: https://github.com/PowerShell/DSC/releases/tag/v3.1.3 "Link to the DSC v3.1.3 release on GitHub"
[compare-v3.1.3]: https://github.com/PowerShell/DSC/compare/v3.1.2...v3.1.3

### Fixed

- Fixed a bug that causes older versions of DSC to raise errors when installed on the same system
  as newer versions with manifests that define new fields.

  Starting with this release, DSC emits info messages when it discovers incompatible manifests and
  skips processing those manifests instead of failing the execution immediately. This enables you
  to install multiple versions of DSC for testing, such as preparing for an upgrade.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs: [#1446][#1446]

  </details>

## [v3.1.2][release-v3.1.2] - 2026-06-15

This section includes a summary of changes for the `3.1.2` patch release. For the full list of
changes in this release, see the [diff on GitHub][compare-v3.1.2].

<!-- Release links -->
[release-v3.1.2]: https://github.com/PowerShell/DSC/releases/tag/v3.1.2 "Link to the DSC v3.1.2 release on GitHub"
[compare-v3.1.2]: https://github.com/PowerShell/DSC/compare/v3.1.1...v3.1.2

### Fixed

- Fixed a bug that caused errors when specifying the relative path to an executable in a resource
  manifest. Starting with this release, DSC correctly resolves relative paths for resource manifest
  executables.

  <details><summary>Related work items</summary>

  - Issues: _None_.
  - PRs:
    - [#1224][#1224]
    - [#1235][#1235]

  </details>


## [v3.1.1][release-v3.1.1] - 2025-07-14

This section includes a summary of changes for the `3.1.1` patch release. For the full list of
changes in this release, see the [diff on GitHub][compare-v3.1.1].

<!-- Release links -->
[release-v3.1.1]: https://github.com/PowerShell/DSC/releases/tag/v3.1.1 "Link to the DSC v3.1.1 release on GitHub"
[compare-v3.1.1]: https://github.com/PowerShell/DSC/compare/v3.1.0...v3.1.1

### Fixed

- Fixed a bug that caused DSC to default output format to JSON instead of YAML when you use a
  `dsc resource` command without capturing or redirecting the output.

  <details><summary>Related work items</summary>

  - Issues: [#918][#918]
  - PRs:
    - [#920][#920]
    - [#960][#960]

  </details>

## [v3.1.0][release-v3.1.0] - 2025-06-18

This section includes a summary of changes for the `3.1.0` release. For the full list of changes
in this release, see the [diff on GitHub][compare-v3.1.0].

<!-- Release links -->
[release-v3.1.0]: https://github.com/PowerShell/DSC/releases/tag/v3.1.0 "Link to the DSC v3.1.0 release on GitHub"
[compare-v3.1.0]: https://github.com/PowerShell/DSC/compare/v3.0.2...v3.1.0

### Added

- Added support for defining adapted resource instances in configuration documents without
  explicitly nesting them inside an instance of the adapter resource. This simplifies the authoring
  for configuration documents, but comes with a potential performance tradeoff, as DSC has to invoke
  the adapter for each adapted instance separately.

  <details><summary>Related work items</summary>

  - Issues: [#693][#693]
  - PRs: [#720][#720]

  </details>

- Added new configuration functions:

  - Use the `equals()` function to compare two values, returning `true` if the values are equal and
    otherwise `false`. The comparison is always `false` if the two values are of a different type.
    When comparing strings, the comparison is case-sensitive.

    For example, `equals('a', 'A')` would evaluate to `false`.
  - Use the `if()` function to conditionally return one of two values. The first argument to the
    function defines the condition and must evaluate to a boolean value. If the conditional
    argument evaluates to `true`, the function returns the second argument. If the conditional
    argument evaluates to `false`, the function returns the third argument.

    For example, `if(equals('a', 'b'), 'left', 'right')` would evaluate to `right`.
  - Use the experimental `format()` function to create a string that interpolates one or more
    values. When you use this experimental function, DSC currently emits a warning. This function
    only supports a subset of the data types supported in the ARM template syntax.

    For example, `format('hello {0} - {1:X}', 'world', 12)` would evaluate to `hello world - c`.

  <details><summary>Related work items</summary>

  - Issues: [#767][#767]
  - PRs:
    - [#770][#770]
    - [#776][#776]
    - [#779][#779]

  </details>

- Added support for extensions to DSC. You can now use the `dsc extension list` command to
  enumerate available extensions. DSC now supports a single extension capability, `discover`, which
  returns JSON objects indicating where to find DSC resource manifests that aren't in the `PATH` or
  `DSC_RESOURCE_PATH`, as with resources installed as Appx packages.

  Now when DSC performs discovery, it recursively discovers extensions and resources on the system.

  This release also includes an experimental extension for discovering resources installed as Appx
  packages.

  - Issues:
    - [#461][#461]
    - [#681][#681]
  - PRs:
    - [#760][#760]
    - [#762][#762]

- Adds support for passing parameters to the `dsc config` commands from stdin. You can pass
  _either_ the configuration document or parameters file contents to the command from stdin, but
  not both. This enables securely passing sensitive parameters to DSC without writing them to a
  file or defining them as an environment variable.

  <details><summary>Related work items</summary>

  - Issues: [#834][#834]
  - PRs: [#863][#863]

  </details>

- Added the `--input` and `--file` options to the [dsc resource export][cli.resource.export]
  command to enable filtering the exported instances.

  Prior to this change, DSC would send resources the defined properties for filtering when a user
  invoked the `dsc config export` command, but the same behavior wasn't available when directly
  invoking the **Export** operation on a resource.

  <details><summary>Related work items</summary>

  - Issues: [#678][#678]
  - PRs: [#680][#680]

  </details>

- Added the YAML document separator (`---`) between output objects when you invoke a DSC command
  with YAML as the output format. Prior to this change, it was dificult to distinguish between
  output objects and to parse the output programmatically.

  <details><summary>Related work items</summary>

  - Issues: [#628][#628]
  - PRs: [#635][#635]

  </details>

- Added the `table-no-truncate` format option to the `dsc resource list`
  command to avoid truncating the table due to the width of the console.

  <details><summary>Related work items</summary>

  - Issues: [#763][#763]
  - PRs: [#823][#823]

  </details>

- Added the `json-array` format option to the `dsc resource get --all` command for easier
  integration and scripting.

  <details><summary>Related work items</summary>

  - Issues: [#813][#813]
  - PRs: [#861][#861]

  </details>

- Added the `pass-through` format option to the `dsc resource get` command to return the data
  directly from the resource without wrapping it in a result. This enables usage by higher order
  tools without requiring them to unwrap the result object.

  <details><summary>Related work items</summary>

  - Issues: _None_
  - PRs: [#862][#862]

  </details>

- Added the `exporter` [resource kind][schema.definitions.resourceKind]. You can now define a DSC
  resource that only implements the **Export** operation to use for inventory and fact-gathering
  scenarios. Resources with the `kind` field in their manifest set to `exporter` must return full
  [resource instances][schema.config.resource] as JSONLINEs. DSC then recursively invokes the
  **Export** operation for those returned instances, enabling you to dynamically generate an
  inventory without specifying every single resource type you want to retrieve from the system.

  <details><summary>Related work items</summary>

  - Issues: [#515][#515]
  - PRs: [#682][#682]

  </details>

- Added support for the `Microsoft.Windows/Registry` resource to support defining a registry value
  without any data (`RZ_NONE`). Prior to this change, users were required to specify both the
  `valueName` and `valueData` properties when defining a registry value. Starting with this
  release, you can define an instance of the resource without `valueData`.

  <details><summary>Related work items</summary>

  - Issues: [#683][#683]
  - PRs: [#684][#684]

  </details>

- Added a warning message during resource discovery when DSC finds a resource manifest that
  includes an executable that doesn't exist. This helps inform a user about whether a resource
  manifest is invalid before making any attempts to invoke that resource directly or starting a
  broader configuration operation.

  <details><summary>Related work items</summary>

  - Issues: _None_
  - PRs: [#688][#688]

  </details>

- Added support for the **Export** operation to the `Microsoft.Windows/WindowsPowerShell` adapter.
  This functionality mirrors the capability of the `Microsoft.DSC/PowerShell` adapter and only
  supports **Export** for adapted PSDSC resources implemented as PowerShell classes.

  <details><summary>Related work items</summary>

  - Issues: [#811][#811]
  - PRs: [#848][#848]

  </details>

### Fixed

- Fixed a bug in the `Microsoft.DSC/PowerShell` and `Microsoft.Windows/WindowsPowerShell` adapters
  that caused an error when a previously cached PSDSC resource module no longer exists on the
  system.

  <details><summary>Related work items</summary>

  - Issues: [#640][#640]
  - PRs: [#647][#647]

  </details>

- Fixed a bug that incorrectly handled resource instances without any properties defined. Prior to
  this change, specifying a resource instance without properties raised an error in the engine.

  <details><summary>Related work items</summary>

  - Issues: _None_
  - PRs: [#675][#675]

  </details>

- Fixed the `Microsoft.DSC.Transitional/RunCommandOnSet` resource to invoke correctly during
  `dsc config` operations. Prior to this change, the resource never reported as being out of the
  desired state, so DSC never invoked the resource with the `dsc config set` command.

  <details><summary>Related work items</summary>

  - Issues: [#658][#658]
  - PRs: [#659][#659]

  </details>

- Fixed a bug in the engine that erroneously dropped any metadata returned by a resource that
  wasn't part of the defined [Microsoft.DSC][schema.metadata] metadata object. Starting with this
  release, all metadata emitted by a resource is correctly returned in the output for a command.

  <details><summary>Related work items</summary>

  - Issues: [#668][#668]
  - PRs: [#679][#679]

  </details>

- Fixed the behavior of the `DSC_RESOURCE_PATH` environment variable to limit discovery for both
  resource manifests and executables. Prior to this change, DSC searched the `PATH` for executables
  referenced in manifests, even when `DSC_RESOURCE_PATH` is defined. Now when you set the
  `DSC_RESOURCE_PATH` variable, DSC only uses those paths for discovery, as intended.

  <details><summary>Related work items</summary>

  - Issues: [#814][#814]
  - PRs: [#825][#825]

  </details>

- Fixed the `Microsoft.Windows/Registry` resource to correctly handle hives with a single subkey.

  <details><summary>Related work items</summary>

  - Issues: _None_
  - PRs: [#684][#684]

  </details>

- Fixed the `Microsoft.Windows/Registry` resource to correctly handle being called to delete a
  nonexistant key instead of erroring.

  <details><summary>Related work items</summary>

  - Issues: [#843][#843]
  - PRs: [#847][#847]

  </details>

- Fixed the behavior of configuration expressions in nested instances. Prior to this change, DSC
  attempted to recursively resolve configuration expressions before invoking group resources,
  causing errors when referencing not-yet-invoked nested instances. DSC no longer recursively
  resolves configuration expressions, requiring the group resource to resolve nested expressions
  instead. Every built-in group resource provided with DSC supports resolving nested expressions.

  <details><summary>Related work items</summary>

  - Issues: [#692][#692]
  - PRs: [#695][#695]

  </details>

- Fixed error messaging for duplicate resource instance names. Prior to this change, the error
  message didn't correctly indicate the name of the duplicate resource instance.

  <details><summary>Related work items</summary>

  - Issues: [#841][#841]
  - PRs: [#844][#844]

  </details>

- Fixed the JSON Schema of the `Microsoft.DSC/PendingReboot` resource to allow specifying whether a
  pending reboot is expected with the `Microsoft.DSC/Assertion` resource.

  <details><summary>Related work items</summary>

  - Issues: [#858][#858]
  - PRs: [#859][#859]

  </details>

- Fixed a bug in the `Microsoft.DSC/PowerShell` and `Microsoft.Windows/WindowsPowerShell` adapters
  that caused failures when a PSDSC resource defined any property with subproperties, like a
  hashtable or custom class. The adapters now correctly handle creating complex properties for
  adapted PSDSC resources.

  <details><summary>Related work items</summary>

  - Issues: [#709][#709]
  - PRs: [#713][#713]

  </details>

- Fixed a bug in the caching for the `Microsoft.DSC/PowerShell` and
  `Microsoft.Windows/WindowsPowerShell` adapters that caused errors when PowerShell modules are
  installed, updated, or removed during a configuration operation. Starting with this release, the
  adapters correctly handle cache invalidation.

  <details><summary>Related work items</summary>

  - Issues:
    - [#745][#745]
    - [#807][#807]
  - PRs:
    - [#748][#748]
    - [#787][#787]

  </details>

- Fixed support in the `Microsoft.DSC/PowerShell` and `Microsoft.Windows/WindowsPowerShell`
  adapters for passing credentials to adapted PSDSC resources. Previously, using any PSDSC
  resources with a **PSCredential** property failed because the adapters didn't correctly cast the
  input data.

  <details><summary>Related work items</summary>

  - Issues: [#328][#328]
  - PRs: [#758][#758]

  </details>

- Fixed the handling of enum values in the `Microsoft.DSC/PowerShell` adapter to return them as
  strings, not integers, for readability.

  <details><summary>Related work items</summary>

  - Issues: [#791][#791]
  - PRs: [#800][#800]

  </details>

- Fixed the handling of hidden properties in the `Microsoft.Windows/WindowsPowerShell` adapter to
  prevent them from being erroneously included in the output for a resource.

  <details><summary>Related work items</summary>

  - Issues: [#832][#832]
  - PRs: [#855][#855]

  </details>

- Fixed bugs in the discovery for the `Microsoft.Windows/WindowsPowerShell` adapter to:

  - Prepend the built-in module path (`$Env:SystemRoot\System32\WindowsPowerShell\1.0\Modules`).
  - Remove PowerShell modules from the path.
  - Ensure PSDSC resources implemented as classes are discoverable.
  - Iindicate when a PSDSC resource can't be found that the adapter requires PSDSC resource modules
    to be installed in the `AllUsers` scope.

  <details><summary>Related work items</summary>

  - Issues:
    - [#707][#707]
    - [#798][#798]
  - PRs:
    - [#764][#764]
    - [#777][#777]
    - [#797][#797]
    - [#812][#812]

  </details>

<!-- Unreleased change links -->

## [v3.0.2][release-v3.0.2] - 2025-04-08

This section includes a summary of changes for the `3.0.2` release. For the full list of changes
in this release, see the [diff on GitHub][compare-v3.0.2].

<!-- Release links -->
[release-v3.0.2]: https://github.com/PowerShell/DSC/releases/tag/v3.0.2 "Link to the DSC v3.0.2 release on GitHub"
[compare-v3.0.2]: https://github.com/PowerShell/DSC/compare/v3.0.1...v3.0.2

### Fixed

- Fixed the `Microsoft.DSC/PowerShell` and `Microsoft.Windows/WindowsPowerShell` resources to
  bypass execution policy when invoked. Prior to this change, the adapters would raise a
  nondescriptive error when the PowerShell execution policy on a machine is restricted, which is
  the default state.

  <details><summary>Related work items</summary>

  - Issues: [#714][#714]
  - PRs: [#715][#715]

  </details>

- Fixed a bug in the `Microsoft.DSC/Assertion` group resource that prevented it from reporting a
  failure when nested resource instances aren't in the desired state. Now when any nested instance
  for the group fails, the group reports a failure, preventing dependent resources from invoking
  needlessly.

  <details><summary>Related work items</summary>

  - Issues: [#731][#731]
  - PRs: [#736][#736]

  </details>

## [v3.0.1][release-v3.0.1] - 2025-03-27

This section includes a summary of changes for the `3.0.1` release. For the full list of changes
in this release, see the [diff on GitHub][compare-v3.0.1].

<!-- Release links -->
[release-v3.0.1]: https://github.com/PowerShell/DSC/releases/tag/v3.0.1 "Link to the DSC v3.0.1 release on GitHub"
[compare-v3.0.1]: https://github.com/PowerShell/DSC/compare/v3.0.0...v3.0.1

### Fixed

- Fixed the build for ARM64 Linux to correctly produce a `.tar.gz` file. Prior to this change, the
  artifact couldn't be unzipped.

  <details><summary>Related work items</summary>

  - Issues: [#687][#687]
  - PRs: [#690][#690]

  </details>

- Fixed a bug in the DSC engine to correctly propagate the `_inDesiredState` canonical property for
  resources that implement the **Test** operation. Prior to this change, the engine would
  erroneously perform a synthetic test, which lead to misreporting.

  <details><summary>Related work items</summary>

  - Issues: [#674][#674]
  - PRs: [#676][#676]

  </details>

- Fixed the implementation for the `Microsoft.DSC/PowerShell` and
  `Microsoft.Windows/WindowsPowerShell` adapters to correctly insert the `_inDesiredState`
  canonical property when returning data for the **Test** operation. Prior to this release, the
  results for adapted PSDSC resources would incorrectly report their status.

  <details><summary>Related work items</summary>

  - Issues: _None_
  - PRs: [#699][#699]

  </details>

- Fixed a bug in tracing that inspected messages for incorrectly cased keys, preventing DSC from
  surfacing those messages.

  <details><summary>Related work items</summary>

  - Issues: _None_
  - PRs: [#699][#699]

  </details>

- Fixed tracing for the `Microsoft.DSC/PowerShell` and `Microsoft.Windows/WindowsPowerShell`
  adapters to surface more useful messages during operations, particularly for debugging and error
  messaging.

  <details><summary>Related work items</summary>

  - Issues: _None_
  - PRs: [#699][#699]

  </details>

- Fixed the progress reporting when the [--progress-format][cli.option.p] option is set to `json`
  by no longer displaying the progress bar when you invoke DSC interactively.

  <details><summary>Related work items</summary>

  - Issues: _None_
  - PRs: [#644][#644]

  </details>

- Fixed writing progress when reporting on an indeterminate number of items by starting at `1` and
  incrementing the counter. Prior to this change it was difficult to track progress for these
  items.

  <details><summary>Related work items</summary>

  - Issues: _None_
  - PRs: [#644][#644]

  </details>

## [v3.0.0][release-v3.0.0] - 2025-02-28

Version `3.0.0` is the first generally available release of DSC.

<!-- Release links -->
[release-v3.0.0]: https://github.com/PowerShell/DSC/releases/tag/v3.0.0 "Link to the DSC v3.0.0 release on GitHub"

<!-- Reference doc links -->
[cli.option.p]: https://learn.microsoft.com/powershell/dsc/reference/cli/?view=dsc-3.0&preserveView=true#--progress-format
[cli.resource.export]: https://learn.microsoft.com/powershell/dsc/reference/cli/resource/export?view=dsc-3.0&preserveView=true
[schema.config.resource]: https://learn.microsoft.com/powershell/dsc/reference/schemas/config/resource?view=dsc-3.0&preserveView=true
[schema.definitions.resourceKind]: https://learn.microsoft.com/powershell/dsc/reference/schemas/definitions/resourcekind?view=dsc-3.0&preserveView=true
[schema.metadata]: https://learn.microsoft.com/powershell/dsc/reference/schemas/metadata/microsoft.dsc/properties?view=dsc-3.0&preserveView=true

<!-- Issue and PR links -->
[#328]: https://github.com/PowerShell/DSC/issues/328
[#428]: https://github.com/PowerShell/DSC/issues/428
[#430]: https://github.com/PowerShell/DSC/issues/430
[#461]: https://github.com/PowerShell/DSC/issues/461
[#467]: https://github.com/PowerShell/DSC/issues/467
[#475]: https://github.com/PowerShell/DSC/issues/475
[#496]: https://github.com/PowerShell/DSC/issues/496
[#50]: https://github.com/PowerShell/DSC/issues/50
[#515]: https://github.com/PowerShell/DSC/issues/515
[#543]: https://github.com/PowerShell/DSC/issues/543
[#566]: https://github.com/PowerShell/DSC/issues/566
[#57]: https://github.com/PowerShell/DSC/issues/57
[#598]: https://github.com/PowerShell/DSC/issues/598
[#628]: https://github.com/PowerShell/DSC/issues/628
[#635]: https://github.com/PowerShell/DSC/issues/635
[#640]: https://github.com/PowerShell/DSC/issues/640
[#644]: https://github.com/PowerShell/DSC/issues/644
[#647]: https://github.com/PowerShell/DSC/issues/647
[#658]: https://github.com/PowerShell/DSC/issues/658
[#659]: https://github.com/PowerShell/DSC/issues/659
[#668]: https://github.com/PowerShell/DSC/issues/668
[#674]: https://github.com/PowerShell/DSC/issues/674
[#675]: https://github.com/PowerShell/DSC/issues/675
[#676]: https://github.com/PowerShell/DSC/issues/676
[#678]: https://github.com/PowerShell/DSC/issues/678
[#679]: https://github.com/PowerShell/DSC/issues/679
[#680]: https://github.com/PowerShell/DSC/issues/680
[#681]: https://github.com/PowerShell/DSC/issues/681
[#682]: https://github.com/PowerShell/DSC/issues/682
[#683]: https://github.com/PowerShell/DSC/issues/683
[#684]: https://github.com/PowerShell/DSC/issues/684
[#685]: https://github.com/PowerShell/DSC/issues/685
[#687]: https://github.com/PowerShell/DSC/issues/687
[#688]: https://github.com/PowerShell/DSC/issues/688
[#690]: https://github.com/PowerShell/DSC/issues/690
[#692]: https://github.com/PowerShell/DSC/issues/692
[#693]: https://github.com/PowerShell/DSC/issues/693
[#695]: https://github.com/PowerShell/DSC/issues/695
[#699]: https://github.com/PowerShell/DSC/issues/699
[#707]: https://github.com/PowerShell/DSC/issues/707
[#709]: https://github.com/PowerShell/DSC/issues/709
[#713]: https://github.com/PowerShell/DSC/issues/713
[#714]: https://github.com/PowerShell/DSC/issues/714
[#715]: https://github.com/PowerShell/DSC/issues/715
[#720]: https://github.com/PowerShell/DSC/issues/720
[#731]: https://github.com/PowerShell/DSC/issues/731
[#736]: https://github.com/PowerShell/DSC/issues/736
[#745]: https://github.com/PowerShell/DSC/issues/745
[#748]: https://github.com/PowerShell/DSC/issues/748
[#758]: https://github.com/PowerShell/DSC/issues/758
[#760]: https://github.com/PowerShell/DSC/issues/760
[#762]: https://github.com/PowerShell/DSC/issues/762
[#763]: https://github.com/PowerShell/DSC/issues/763
[#764]: https://github.com/PowerShell/DSC/issues/764
[#767]: https://github.com/PowerShell/DSC/issues/767
[#770]: https://github.com/PowerShell/DSC/issues/770
[#776]: https://github.com/PowerShell/DSC/issues/776
[#777]: https://github.com/PowerShell/DSC/issues/777
[#779]: https://github.com/PowerShell/DSC/issues/779
[#787]: https://github.com/PowerShell/DSC/issues/787
[#791]: https://github.com/PowerShell/DSC/issues/791
[#797]: https://github.com/PowerShell/DSC/issues/797
[#798]: https://github.com/PowerShell/DSC/issues/798
[#800]: https://github.com/PowerShell/DSC/issues/800
[#807]: https://github.com/PowerShell/DSC/issues/807
[#811]: https://github.com/PowerShell/DSC/issues/811
[#812]: https://github.com/PowerShell/DSC/issues/812
[#813]: https://github.com/PowerShell/DSC/issues/813
[#814]: https://github.com/PowerShell/DSC/issues/814
[#823]: https://github.com/PowerShell/DSC/issues/823
[#825]: https://github.com/PowerShell/DSC/issues/825
[#829]: https://github.com/PowerShell/DSC/issues/829
[#832]: https://github.com/PowerShell/DSC/issues/832
[#834]: https://github.com/PowerShell/DSC/issues/834
[#841]: https://github.com/PowerShell/DSC/issues/841
[#843]: https://github.com/PowerShell/DSC/issues/843
[#844]: https://github.com/PowerShell/DSC/issues/844
[#845]: https://github.com/PowerShell/DSC/issues/845
[#847]: https://github.com/PowerShell/DSC/issues/847
[#848]: https://github.com/PowerShell/DSC/issues/848
[#853]: https://github.com/PowerShell/DSC/issues/853
[#855]: https://github.com/PowerShell/DSC/issues/855
[#858]: https://github.com/PowerShell/DSC/issues/858
[#859]: https://github.com/PowerShell/DSC/issues/859
[#861]: https://github.com/PowerShell/DSC/issues/861
[#862]: https://github.com/PowerShell/DSC/issues/862
[#863]: https://github.com/PowerShell/DSC/issues/863
[#869]: https://github.com/PowerShell/DSC/issues/869
[#872]: https://github.com/PowerShell/DSC/issues/872
[#876]: https://github.com/PowerShell/DSC/issues/876
[#877]: https://github.com/PowerShell/DSC/issues/877
[#878]: https://github.com/PowerShell/DSC/issues/878
[#879]: https://github.com/PowerShell/DSC/issues/879
[#884]: https://github.com/PowerShell/DSC/issues/884
[#885]: https://github.com/PowerShell/DSC/issues/885
[#893]: https://github.com/PowerShell/DSC/issues/893
[#898]: https://github.com/PowerShell/DSC/issues/898
[#908]: https://github.com/PowerShell/DSC/issues/908
[#913]: https://github.com/PowerShell/DSC/issues/913
[#918]: https://github.com/PowerShell/DSC/issues/918
[#920]: https://github.com/PowerShell/DSC/issues/920
[#923]: https://github.com/PowerShell/DSC/issues/923
[#924]: https://github.com/PowerShell/DSC/issues/924
[#931]: https://github.com/PowerShell/DSC/issues/931
[#942]: https://github.com/PowerShell/DSC/issues/942
[#947]: https://github.com/PowerShell/DSC/issues/947
[#958]: https://github.com/PowerShell/DSC/issues/958
[#959]: https://github.com/PowerShell/DSC/issues/959
[#960]: https://github.com/PowerShell/DSC/issues/960
[#964]: https://github.com/PowerShell/DSC/issues/964
[#967]: https://github.com/PowerShell/DSC/issues/967
[#969]: https://github.com/PowerShell/DSC/issues/969
[#972]: https://github.com/PowerShell/DSC/issues/972
[#975]: https://github.com/PowerShell/DSC/issues/975
[#976]: https://github.com/PowerShell/DSC/issues/976
[#978]: https://github.com/PowerShell/DSC/issues/978
[#979]: https://github.com/PowerShell/DSC/issues/979
[#980]: https://github.com/PowerShell/DSC/issues/980
[#990]: https://github.com/PowerShell/DSC/issues/990
[#997]: https://github.com/PowerShell/DSC/issues/997
[#999]: https://github.com/PowerShell/DSC/issues/999
[#1004]: https://github.com/PowerShell/DSC/issues/1004
[#1005]: https://github.com/PowerShell/DSC/issues/1005
[#1010]: https://github.com/PowerShell/DSC/issues/1010
[#1018]: https://github.com/PowerShell/DSC/issues/1018
[#1022]: https://github.com/PowerShell/DSC/issues/1022
[#1024]: https://github.com/PowerShell/DSC/issues/1024
[#1029]: https://github.com/PowerShell/DSC/issues/1029
[#1030]: https://github.com/PowerShell/DSC/issues/1030
[#1031]: https://github.com/PowerShell/DSC/issues/1031
[#1032]: https://github.com/PowerShell/DSC/issues/1032
[#1035]: https://github.com/PowerShell/DSC/issues/1035
[#1038]: https://github.com/PowerShell/DSC/issues/1038
[#1040]: https://github.com/PowerShell/DSC/issues/1040
[#1041]: https://github.com/PowerShell/DSC/issues/1041
[#1046]: https://github.com/PowerShell/DSC/issues/1046
[#1069]: https://github.com/PowerShell/DSC/issues/1069
[#1071]: https://github.com/PowerShell/DSC/issues/1071
[#1077]: https://github.com/PowerShell/DSC/issues/1077
[#1079]: https://github.com/PowerShell/DSC/issues/1079
[#1083]: https://github.com/PowerShell/DSC/issues/1083
[#1084]: https://github.com/PowerShell/DSC/issues/1084
[#1085]: https://github.com/PowerShell/DSC/issues/1085
[#1086]: https://github.com/PowerShell/DSC/issues/1086
[#1087]: https://github.com/PowerShell/DSC/issues/1087
[#1092]: https://github.com/PowerShell/DSC/issues/1092
[#1093]: https://github.com/PowerShell/DSC/issues/1093
[#1095]: https://github.com/PowerShell/DSC/issues/1095
[#1096]: https://github.com/PowerShell/DSC/issues/1096
[#1097]: https://github.com/PowerShell/DSC/issues/1097
[#1099]: https://github.com/PowerShell/DSC/issues/1099
[#1101]: https://github.com/PowerShell/DSC/issues/1101
[#1103]: https://github.com/PowerShell/DSC/issues/1103
[#1105]: https://github.com/PowerShell/DSC/issues/1105
[#1116]: https://github.com/PowerShell/DSC/issues/1116
[#1117]: https://github.com/PowerShell/DSC/issues/1117
[#1121]: https://github.com/PowerShell/DSC/issues/1121
[#1123]: https://github.com/PowerShell/DSC/issues/1123
[#1124]: https://github.com/PowerShell/DSC/issues/1124
[#1127]: https://github.com/PowerShell/DSC/issues/1127
[#1129]: https://github.com/PowerShell/DSC/issues/1129
[#1132]: https://github.com/PowerShell/DSC/issues/1132
[#1138]: https://github.com/PowerShell/DSC/issues/1138
[#1142]: https://github.com/PowerShell/DSC/issues/1142
[#1145]: https://github.com/PowerShell/DSC/issues/1145
[#1148]: https://github.com/PowerShell/DSC/issues/1148
[#1154]: https://github.com/PowerShell/DSC/issues/1154
[#1156]: https://github.com/PowerShell/DSC/issues/1156
[#1159]: https://github.com/PowerShell/DSC/issues/1159
[#1160]: https://github.com/PowerShell/DSC/issues/1160
[#1162]: https://github.com/PowerShell/DSC/issues/1162
[#1170]: https://github.com/PowerShell/DSC/issues/1170
[#1174]: https://github.com/PowerShell/DSC/issues/1174
[#1175]: https://github.com/PowerShell/DSC/issues/1175
[#1176]: https://github.com/PowerShell/DSC/issues/1176
[#1178]: https://github.com/PowerShell/DSC/issues/1178
[#1183]: https://github.com/PowerShell/DSC/issues/1183
[#1187]: https://github.com/PowerShell/DSC/issues/1187
[#1190]: https://github.com/PowerShell/DSC/issues/1190
[#1192]: https://github.com/PowerShell/DSC/issues/1192
[#1194]: https://github.com/PowerShell/DSC/issues/1194
[#1198]: https://github.com/PowerShell/DSC/issues/1198
[#1202]: https://github.com/PowerShell/DSC/issues/1202
[#1205]: https://github.com/PowerShell/DSC/issues/1205
[#1207]: https://github.com/PowerShell/DSC/issues/1207
[#1208]: https://github.com/PowerShell/DSC/issues/1208
[#1211]: https://github.com/PowerShell/DSC/issues/1211
[#1213]: https://github.com/PowerShell/DSC/issues/1213
[#1215]: https://github.com/PowerShell/DSC/issues/1215
[#1219]: https://github.com/PowerShell/DSC/issues/1219
[#1224]: https://github.com/PowerShell/DSC/issues/1224
[#1227]: https://github.com/PowerShell/DSC/issues/1227
[#1230]: https://github.com/PowerShell/DSC/issues/1230
[#1235]: https://github.com/PowerShell/DSC/issues/1235
[#1236]: https://github.com/PowerShell/DSC/issues/1236
[#1238]: https://github.com/PowerShell/DSC/issues/1238
[#1260]: https://github.com/PowerShell/DSC/issues/1260
[#1268]: https://github.com/PowerShell/DSC/issues/1268
[#1274]: https://github.com/PowerShell/DSC/issues/1274
[#1275]: https://github.com/PowerShell/DSC/issues/1275
[#1277]: https://github.com/PowerShell/DSC/issues/1277
[#1278]: https://github.com/PowerShell/DSC/issues/1278
[#1284]: https://github.com/PowerShell/DSC/issues/1284
[#1307]: https://github.com/PowerShell/DSC/issues/1307
[#1308]: https://github.com/PowerShell/DSC/issues/1308
[#1317]: https://github.com/PowerShell/DSC/issues/1317
[#1327]: https://github.com/PowerShell/DSC/issues/1327
[#1331]: https://github.com/PowerShell/DSC/issues/1331
[#1332]: https://github.com/PowerShell/DSC/issues/1332
[#1333]: https://github.com/PowerShell/DSC/issues/1333
[#1340]: https://github.com/PowerShell/DSC/issues/1340
[#1343]: https://github.com/PowerShell/DSC/issues/1343
[#1348]: https://github.com/PowerShell/DSC/issues/1348
[#1351]: https://github.com/PowerShell/DSC/issues/1351
[#1352]: https://github.com/PowerShell/DSC/issues/1352
[#1361]: https://github.com/PowerShell/DSC/issues/1361
[#1366]: https://github.com/PowerShell/DSC/issues/1366
[#1367]: https://github.com/PowerShell/DSC/issues/1367
[#1368]: https://github.com/PowerShell/DSC/issues/1368
[#1369]: https://github.com/PowerShell/DSC/issues/1369
[#1372]: https://github.com/PowerShell/DSC/issues/1372
[#1374]: https://github.com/PowerShell/DSC/issues/1374
[#1375]: https://github.com/PowerShell/DSC/issues/1375
[#1377]: https://github.com/PowerShell/DSC/issues/1377
[#1386]: https://github.com/PowerShell/DSC/issues/1386
[#1387]: https://github.com/PowerShell/DSC/issues/1387
[#1398]: https://github.com/PowerShell/DSC/issues/1398
[#1401]: https://github.com/PowerShell/DSC/issues/1401
[#1407]: https://github.com/PowerShell/DSC/issues/1407
[#1408]: https://github.com/PowerShell/DSC/issues/1408
[#1417]: https://github.com/PowerShell/DSC/issues/1417
[#1426]: https://github.com/PowerShell/DSC/issues/1426
[#1430]: https://github.com/PowerShell/DSC/issues/1430
[#1432]: https://github.com/PowerShell/DSC/issues/1432
[#1433]: https://github.com/PowerShell/DSC/issues/1433
[#1445]: https://github.com/PowerShell/DSC/issues/1445
[#1446]: https://github.com/PowerShell/DSC/issues/1446
[#1449]: https://github.com/PowerShell/DSC/issues/1449
[#1453]: https://github.com/PowerShell/DSC/issues/1453
[#1534]: https://github.com/PowerShell/DSC/issues/1534
[#1535]: https://github.com/PowerShell/DSC/issues/1535
[#1547]: https://github.com/PowerShell/DSC/issues/1547
[#1554]: https://github.com/PowerShell/DSC/issues/1554
[#1557]: https://github.com/PowerShell/DSC/issues/1557
[#1558]: https://github.com/PowerShell/DSC/issues/1558
[#1562]: https://github.com/PowerShell/DSC/issues/1562


