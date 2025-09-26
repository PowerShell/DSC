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

## [v3.1.1][release-v3.1.1] - 2025-07-14

This section includes a summary of changes for the `3.1.1` patch release. For the
full list of changes in this release, see the
[diff on GitHub][compare-v3.1.1].

<!-- Release links -->
[release-v3.1.1]: https://github.com/PowerShell/DSC/releases/tag/v3.1.1 "Link to the DSC v3.1.1 release on GitHub"
[compare-v3.1.1]: https://github.com/PowerShell/DSC/compare/v3.1.0...v3.1.1

### Fixed

- Backport: Fix default output to YAML when used interactively.

  <details><summary>Related work items</summary>

  - Issues: [#918][#918]
  - PRs: [#960][#960]

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
[#515]: https://github.com/PowerShell/DSC/issues/515
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
[#682]: https://github.com/PowerShell/DSC/issues/682
[#683]: https://github.com/PowerShell/DSC/issues/683
[#684]: https://github.com/PowerShell/DSC/issues/684
[#687]: https://github.com/PowerShell/DSC/issues/687
[#688]: https://github.com/PowerShell/DSC/issues/688
[#690]: https://github.com/PowerShell/DSC/issues/690
[#692]: https://github.com/PowerShell/DSC/issues/692
[#693]: https://github.com/PowerShell/DSC/issues/693
[#695]: https://github.com/PowerShell/DSC/issues/695
[#699]: https://github.com/PowerShell/DSC/issues/699
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
[#764]: https://github.com/PowerShell/DSC/issues/764
[#787]: https://github.com/PowerShell/DSC/issues/787
[#807]: https://github.com/PowerShell/DSC/issues/807
[#767]: https://github.com/PowerShell/DSC/issues/767
[#770]: https://github.com/PowerShell/DSC/issues/770
[#776]: https://github.com/PowerShell/DSC/issues/776
[#779]: https://github.com/PowerShell/DSC/issues/779
[#707]: https://github.com/PowerShell/DSC/issues/707
[#777]: https://github.com/PowerShell/DSC/issues/777
[#461]: https://github.com/PowerShell/DSC/issues/461
[#681]: https://github.com/PowerShell/DSC/issues/681
[#760]: https://github.com/PowerShell/DSC/issues/760
[#762]: https://github.com/PowerShell/DSC/issues/762
[#797]: https://github.com/PowerShell/DSC/issues/797
[#798]: https://github.com/PowerShell/DSC/issues/798
[#812]: https://github.com/PowerShell/DSC/issues/812
[#791]: https://github.com/PowerShell/DSC/issues/791
[#800]: https://github.com/PowerShell/DSC/issues/800
[#763]: https://github.com/PowerShell/DSC/issues/763
[#823]: https://github.com/PowerShell/DSC/issues/823
[#841]: https://github.com/PowerShell/DSC/issues/841
[#844]: https://github.com/PowerShell/DSC/issues/844
[#843]: https://github.com/PowerShell/DSC/issues/843
[#847]: https://github.com/PowerShell/DSC/issues/847
[#811]: https://github.com/PowerShell/DSC/issues/811
[#848]: https://github.com/PowerShell/DSC/issues/848
[#814]: https://github.com/PowerShell/DSC/issues/814
[#825]: https://github.com/PowerShell/DSC/issues/825
[#832]: https://github.com/PowerShell/DSC/issues/832
[#855]: https://github.com/PowerShell/DSC/issues/855
[#813]: https://github.com/PowerShell/DSC/issues/813
[#861]: https://github.com/PowerShell/DSC/issues/861
[#858]: https://github.com/PowerShell/DSC/issues/858
[#859]: https://github.com/PowerShell/DSC/issues/859
[#862]: https://github.com/PowerShell/DSC/issues/862
[#834]: https://github.com/PowerShell/DSC/issues/834
[#863]: https://github.com/PowerShell/DSC/issues/863
[#918]: https://github.com/PowerShell/DSC/issues/918
[#960]: https://github.com/PowerShell/DSC/issues/960
