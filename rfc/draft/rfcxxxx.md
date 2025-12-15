---
RFC:          RFCNNNN # WG will set the number after submission
Author:       jahelmic # <@GitHubUserName>
Sponsor:      michaeltlombardi    # <@GitHubUserName>
Status:       Draft   # <Draft | Experimental | Accepted | Final>
SupercededBy: null    # <Superceding RFC Number>
Version:      1.0     # <Major>.<Minor>
Area:         DSC     # <Area within the DSC project>
CommentsDue:  null    # <Date for submitting comments to current draft (minimum 1 month)>
---

# Class-based PSDSC resource contract for DSC v3

This RFC defines a contract for PowerShell **class-based** PSDSC resources so that a single
implementation can:

- Continue to work with **PSDSC v1/v2**, and
- Participate fully in **DSC v3 semantics** via the PSDSC adapter,

without requiring a hard dependency on a Microsoft-shipped "DSC types" or RDK module.

The RFC focuses on:

- The **method signatures** and shapes DSC v3 cares about for PowerShell class-based resources.
- The **expected return structures** (results, messages, and streams).
- How this contract **aligns with JSON schema and manifest generation**, so that tooling (RDK,
  Sampler, analyzers) can build on it.

## Motivation

> As a DSC resource author with existing PSDSC class-based resources,
> I want to augment those resources to participate in DSC v3 semantics,
> so that I can support both PSDSC v1/v2 and DSC v3 without maintaining separate codebases.

Additional motivations:

- Many resources already implemented as **class-based PSDSC resources** have active users and are
  expensive to rewrite.
- DSC v3 introduces richer semantics and tooling:
  - Structured results from `Test`, `Set`, and `Get` (not just a "Reason" string).
  - Better "diff" semantics (for latest-version / package-like scenarios).
  - JSON-schema-based validation and manifest-driven discovery.
- Resource authors should be able to:
  - Incrementally add **V3-only capabilities** (e.g., richer test results, schema) to existing
    class-based resources.
  - Avoid taking a **mandatory dependency** on a Microsoft-shipped types or RDK module.
- The DSC community (including RDK / Sampler users) needs a **clear, documented contract** so that:
  - Static analysis is feasible,
  - Schema/manifest generation is consistent,
  - ScriptAnalyzer rules can be updated coherently for V1/V2/V3 resources.

## Proposed experience

This section describes how the contract feels for a resource author and for DSC v3 consumers.

### Authoring a class-based resource that works for PSDSC v1/v2 and DSC v3

A resource author today might have:

```powershell
[DscResource()]
class ChocolateyPackage {
    [DscProperty(Key)]
    [string] $Name

    [DscProperty(Mandatory)]
    [ValidateSet('Present', 'Absent')]
    [string] $Ensure = 'Present'

    [ChocolateyPackage] Get() { ... }
    [void] Set() { ... }
    [bool] Test() { ... }
}
```

With this RFC, the author can introduce:

```powershell
class ChocolateyPackage {
    static [System.Tuple[bool, ChocolateyPackage, String[]]] Test(
      [ChocolateyPackage]$instance
    ) {
        return Test-ChocolateyPackageResource -Instance $instance
    }

    static [System.Tuple[ChocolateyPackage, String[]]] Set(
      [ChocolateyPackage]$instance
    ) {
        Set-ChocolateyPackageResource -Instance $instance
    }

    static [ChocolateyPackage] Get(
      [ChocolateyPackage]$instance
    ) {
        return Get-ChocolateyPackageResource -Instance $instance
    }

    static [void] Delete(
      [ChocolateyPackage]$instance
    ) {
      Remove-ChocolateyPackageResource -Instance $instance
    }

    static [ChocolateyPackage[]] Export(
      [ChocolateyPackage]$filteringInstance
    ) {
      return Export-ChocolateyPackageResource -FilteringInstance $filteringInstance
    }
}
```

Key changes:

- All DSC v3-relevant methods are **static** and _optional_. If the class doesn't implement a static
  method for an operation, DSC can use the PSDSC instance method for that operation.
- A single class supports **both PSDSC and DSC v3**.
- Authors may optionally return richer structured data for DSC v3.
- No mandatory dependency on Microsoft-owned types.

### Using the resource in DSC v3

Example configuration:

```yaml
resources:
- type: Contoso.DSC/ChocolateyPackage
  name: InstallGit
  properties:
    Name: git
```

DSC v3 (via the PSDSC adapter):

- Validates JSON against the generated schema.
- Calls `Test`, `Get`, `Set` with class-based resource instances.
- Accepts both simple and structured returns.
- Emits structured messages and differences.

Resource consumers see **consistent behavior**, regardless of whether the implementation is native
DSC v3 or an adapted PSDSC class-based resource.

## Specification

> [!NOTE]
> Some aspects are deliberately scoped as "MVP" so that a pilot implementation (e.g., Chocolatey
> resources) can validate the design. Where details are not finalized, they are explicitly called
> out.

### Resource class shape

A DSC v3-compliant class-based resource MUST:

- Declare a schema class representing the resource instance.
- Use **static methods** for DSC v3 interaction.
- Accept the schema class instance as the parameter to all methods.

Skeleton:

```powershell
[DscResource()]
class <ResourceClass> {
    [DscProperty(Key)]
    [string] $Name

    [DscProperty(Mandatory)]
    [ValidateSet('Present', 'Absent')]
    [string] $Ensure = 'Present'

    static [<ResourceClass>] Get([<ResourceClass>]$instance) {}
    static [<SetReturn>]     Set([<ResourceClass>]$instance) {}
    static [<TestReturn>]    Test([<ResourceClass>]$instance) {}

    # optional
    static [<ExportReturn>] Export(...)
    static [void]           Delete(...)
    static [<SchemaReturn>] Schema(...)
}
```

### DSC operation method selection

If a class has the `[DscResource()]` attribute, DSC and the adapter know that the resource class
implements the traditional PSDSC resource methods `Get()`, `Set()`, and `Test()`.

When selecting the method to use for an operation, the adapter:

1. Checks for the existance of a DSC static method for that operation.
1. If the resource class implements a static method for the operation, DSC invokes that method.
1. If the class doesn't implement a static method for the operation and the operation is part of
   the PSDSC resource API, DSC uses the appropriate PSDSC instance method.
1. If the class doesn't implement a static method or instance method for the operation, the resource
   can't be used for that operation and DSC raises an error.

> [!NOTE]
> In this model, we _can_ support classes defined for DSC that don't have the `[DscResource()]`
> attribute and thus may not have the PSDSC instance methods. Supporting these classes is out of
> scope for the MVP.

### Method signatures (MVP)

For the MVP, this RFC proposes the following new method signatures. Each section defines method
signatures for a different DSC resource operation. In this proposal, we use the `Tuple` type for
structured return data. This enables static analysis and implementation without any dependencies
for defined types.

Future revisions of this RFC may:

- Introduce strongly-typed result classes (e.g., `DscTestResult`) that map to the same shape.
- Define a shared types module that authors can optionally reference.

#### Get operation method

Signature:

```pwsh
static [<ResourceClass>] Get([<ResourceClass>]$instance)
```

The `get` operation must always return the actual state of the instance with all discoverable
properties populated.

#### Set operation method

Signatures:

- No return data (DSC invokes `get` after `set` to generate after state and
  changed properties):

  ```pwsh
  static [void] Set([<ResourceClass>]$instance)
  ```

- Return state only (DSC generates the changed properties arrray):

  ```pwsh
  static [<ResourceClass>] Set([<ResourceClass>]$instance)
  ```

- Return state and changed properties (DSC uses the result without processing):

  ```pwsh
  static [System.Tuple[<ResourceClass>, String[]]] Set([<ResourceClass>]$instance)
  ```

- To indicate that the resource supports `whatIf` mode operations as well as `actual`, the class
  should define a method signature that expects a boolean parameter after the instance parameter:

  ```pwsh
  # No return data
  static [void] Set([<ResourceClass>]$instance, [bool]$whatIf)
  # state return kind
  static [<ResourceClass>] Set([<ResourceClass>]$instance, [bool]$whatIf)
  # stateAndDiff return kind
  static [System.Tuple[<ResourceClass>, String[]]] Set([<ResourceClass>]$instance, [bool]$whatIf)
  ```

The `set` operation may use one of three return types:

- The `[void]` return type maps to the same behavior and handling as the PSDSC `Set()` instance
  method.
- The `[<ResourceClass>]` return type maps to the DSC `state` return kind for a command resource.
- The `[System.Tuple[<ResourceClass>, String[]]]` return type maps to the DSC `stateAndDiff`
  return kind for a command resource.

The `set` operation may support `whatIf` mode invocations. In this mode, the resource doesn't change the system. Instead, it reports _how_ it would modify the system. The return data for this
operation is the _expected_ final state and changed properties. The return type for the what-if method _must_ be the same as the actual method signature, such as:

```pwsh
static [System.Tuple[ChocolateyPackage, String[]] Set(
  [ChocolateyPackage]$instance,
  [bool]$whatIf
) {
  # Implementation
}
static [System.Tuple[ChocolateyPackage, String[]] Set(
  [ChocolateyPackage]$instance
) {
  [ChocolateyPackage]::Set($instance, $false)
}
```

#### Test operation method

Signatures:

- Return state only (DSC generates the differing properties array):

  ```pwsh
  static [System.Tuple[bool, <ResourceClass>]] Test([<ResourceClass>] $instance)
  ```

- Return state and differing properties (DSC uses the result without processing):

  ```pwsh
  static [System.Tuple[bool, <ResourceClass>, String[]]] Test([<ResourceClass>] $instance)
  ```

The `test` operation may use one of two return types:

- The `[System.Tuple[bool, <ResourceClass>]]` return type maps to the DSC `state` return kind for
  a command resource. Instead of requiring the class to define the `InDesiredState` read-only
  property, DSC expects the resource to return the boolean value _and_ the actual state of the
  resource. The adapter munges the result for DSC.
- The `[System.Tuple[bool, <ResourceClass>, String[]]]` return type maps to the DSC `stateAndDiff`
  return kind for a command resource.

#### Export operation method

Signatures:

- Non-filtering export (resource returns every discovered instance):

  ```pwsh
  static [<ResourceClass>[]] Export()
  ```

- Filtered export (resource uses the input instance to limit the return data):

  ```pwsh
  static [<ResourceClass>[]] Export([<ResourceClass>]$filteringInstance)
  ```

The return type for the `export` operation is always an array of instances of the resource class.

The export functionality depends on which method signatures are implemented:

- If the class implements both signatures, it supports filtered and unfiltered exports.
- If the class implements only the parameterless signature, it doesn't support filtered exports.
- If the class implements only the signature with a filtering instance, it doesn't support
  unfiltered exports.
- If the class doesn't implement either signature, it doesn't support the `export` operation.

#### Delete operation method

Signature:

```pwsh
static [void] Delete([<ResourceClass>]$instance)
```

In the current data model for DSC, the `delete` method returns no data. Only messages and
execution status (success or failure) are reported back to the engine.

#### Schema method

Signatures:

```pwsh
static [string] Schema()
```

DSC expects the resource to return a string representation of the resource instance JSON Schema.
The output must validate against the resource instance meta schema at the following JSON pointer
URI:

```text
https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/resource/manifest.schema.json#/properties/embedded
```

If the resource doesn't implement this method, DSC generates a JSON Schema by inspecting the
resource class itself.

> [!NOTE]
> Currently, DSC doesn't distinguish between the instance schema and the filtering instance
> schema. Issue #1232 proposes a model for separately validating resource properties for the
> `export` operation. Until that proposal is implemented, the only schema we can emit for a
> resource is the resource instance schema.
>
> In the future, we may add a second method, like `FilteringSchema()`, to account for this.

### Attributes and metadata

Some information cannot be derived from method signatures alone. This RFC proposes a small set of
**optional attributes** that can be applied to the class, properties and methods.

These attributes MAY live in a shared types/module (e.g., RDK or `Microsoft.DSC.Types`). DSC v3
MUST treat this module as an **optional dependency**:

- If present, the adapter and tooling can leverage the attributes.
- If absent, equivalent metadata MAY be provided via structured return types or manifest entries.

> [!NOTE]
> When considering alternatives to attributes, it would be _possible_ to retrieve this information
> with a set of new metadata methods or static properties. We could accept hashtables for those
> methods or properties to minimize the number of items to check, but that would then require
> validating the keys and data types. Each of the following sections has a collapsible details
> block that enumerates the possible metadata static properties, but in general using attributes is
> preferable for this purpose.

#### `[DscResourceClass()]` attribute

The `[DscResourceClass()]` attribute annotates the class itself to define metadata about the class:

- `[DscResourceClass(DscVersion = '<dsc-version>')]` - Indicates which version of DSC the resource
  was developed with and to use for the `$schema` in the generated manifest. Values must match
  the following regex:

  ```regex
  ^v(?<major>\d+)\.(?<minor>\d+).(?<patch>\d+)$
  ```

  When this option isn't specified, the default value is `v3`.
- `[DscResourceClass(Type = '<fully-qualified-type-name>')]` - Defines the type name for the
  resource. Values must be a valid resource type name, like `Contoso.Chocolatey/Package`. When
  this option isn't specified, the default is `<ModuleName>/<ResourceClassName>`.
- `[DscResourceClass(Version = '<semantic-version>')]` - Defines the semantic version for the
  resource. Values must be a string that parses as a valid semantic version, like `1.2.3`. When
  this option isn't specified, the default is the module version.
- `[DscResourceClass(Description = '<description>')]` - Defines a short description for the
  resource, surfaced in the `dsc resource list` command. No default value.
- `[DscResourceClass(Tags = ('<tag>', ..., '<tag>'))]` - Defines the tags for the resource.

<details><summary>Metadata static properties</summary>

```pwsh
# Single method for all data, all fields optional:
static [hashtable] $ResourceMetadata = {
  @{
    DscVersion = '<DscVersion>'
    Type = '<fully-qualified-type-name>'
    Version = '<semantic-version>'
    Description = '<description>'
    Tags = @('<tag>', ..., '<tag>')
  }
}
# Individual properties
static [string]$DscVersion  = '<DscVersion>'
static [string]$Type        = '<fully-qualified-type-name>'
static [string]$Version     = '<semantic-version>'
static [string]$Description = '<description>'
static [string[]]$Tags      = @('<tag>', ..., '<tag>')
```

</details>

#### `[DscResourceProperty()]` attribute

The `[DscResourceProperty()]` attribute enables authors to annotate their resource properties with DSC semantics:

- `[DscResourceProperty(Canonical)]` - indicates that the property is a canonical DSC resource
  property. This attribute is only valid on properties that have the same name. For example. the following is valid, annotating the `_exist` canonical property:

  ```pwsh
  [DscResourceProperty(Canonical)]
  [bool]$Exist = $false
  ```

  And the following snippet would be invalid, because `_ensure` isn't a canonical property:

  ```pwsh
  [DscResourceProperty(Canonical)]
  [string]$Ensure
  ```

  > [!NOTE]
  > Ideally, we would have a way for the resource to use either the shorthand,
  > `[DscResourceProperty(Canonical)]` or specify the name of the canonical property to help
  > with property name conflicts, especially given the existence of canonical properties like
  > `_name`, which may conflict with the ergonomic design of the resource (like a chocolatey
  > package name).
  >
  > That would make the following definitions valid:
  >
  > ```pwsh
  > [DscResourceProperty(Canonical)]
  > [bool]$Exist = $false
  >
  > [DscResourceProperty(Canonical='_name')]
  > [string]$InstanceName
  > ```

- `[DscResourceProperty(ReadOnly)]` - indicates that the property is read-only and can be
  returned from the resource but is never used as input _to_ the resource.
- `[DscResourceProperty(WriteOnly)]` - indicates that the property is write-only and can be
  passed to the resource as input but is never returned in the output data.
- `[DscResourceProperty(Sensitive)]` - inidcates that the property is sensitive and should be
  redacted from messaging and output. Only valid on properties that have a string, object, or
  enum type.
- `[DscResourceProperty(Key)]` - indicates that the property uniquely identifies an instance of
  the resource.
- `[DscResourceProperty(Required)]` - indicates that the property is mandatory for non-export
  operations.

The `[DscResourceProperty()]` can inherit values when the class defines the `[DscProperty()]`
attribute on the same property:

- `[DscProperty(Key)]` - maps to `[DscResourceProperty(Key)]`.
- `[DscProperty(Mandatory)]` - maps to `[DscResourceProperty(Required)]`.
- `[DscProperty(NotConfigurable)]` - maps to `[DscResourceProperty(ReadOnly)]`.

<details><summary>Metadata static properties</summary>

```pwsh
# Single property for all data, each key a different property, all fields optional:
static [hashtable] $ResourcePropertyMetadata = @{
  @{
    <PropertyName> = @{
      ReadOnly  = $false
      WriteOnly = $false
      Sensitive = $false
      Key       = $false
      Required  = $false
    }
  }
}
# Per property metadata, all fields optional:
static [hashtable]$<PropertyName>Metadata  = @{
  ReadOnly  = $false
  WriteOnly = $false
  Sensitive = $false
  Key       = $false
  Required  = $false
}
# Individual properties for each resource property and option:
static [bool]$<PropertyName>ReadOnly  = $false
static [bool]$<PropertyName>WriteOnly = $false
static [bool]$<PropertyName>Sensitive = $false
static [bool]$<PropertyName>Key       = $false
static [bool]$<PropertyName>Required  = $false
```

</details>

#### `[DscResourceSet()]` attribute

The `[DscResourceSet()]` attribute defines handling for the `set` method. Must be attached to a
static set method signature. If the resource defines a signature that indicates support for
`whatIf` mode, the attribute must be on that method.

- `[DscResourceSet(implementsPretest)]` - Indicates that the resource is implemented to check
  whether it needs to change system state before making any changes. This maps to the
  `set.implementsPretest` resource manifest field. When this option isn't specified, the
  default is `false`.
- `[DscResourceSet(handlesExist)]` - Indicates that the resource directly handles the `_exist`
  canonical property. This maps to the `set.handlesExist` resource manifest field. When this
  option isn't specified, the default is `false`.

<details><summary>Metadata static properties</summary>

```pwsh
# Single property for all data, all fields optional:
static [hashtable] $SetOperation = @{
  @{
    ImplementsPretest = $false
    HandlesExist      = $false
  }
}
# Per option metadata
static [bool] $SetOperationImplementsPretest = $false
static [bool] $SetOperationHandlesExist      = $false
```

</details>

### JSON schema and manifest alignment

The class contract MUST align with a JSON schema and manifest model:

- **JSON schema**:

  - Represents properties, types, constraints, and read-only / write-only / sensitive flags.
  - Ideally generated at **build time** from the PowerShell class (e.g., using `System.Text.Json`).
  - May be embedded in a manifest or file alongside the resource module.

- **Manifest**:

  - Describes which methods are implemented and what capabilities the resource supports (e.g.,
    supports `Export`, has rich `Test` results, etc.).
  - Enables faster discovery and avoids heavy runtime analysis.

This RFC does **not** fully define the JSON schema format or manifest schema, but it requires:

- The class-based contract to expose enough information to:

  - Generate JSON schema with correct property naming and constraints.
  - Generate a manifest that allows the adapter to skip expensive reflection where possible.

> OPEN:
>
> - Naming conventions for JSON properties (camelCase vs PascalCase).
> - How to annotate canonical properties and avoid conflicts with WMI/LCM constraints (e.g.,
> `__Name`).
> - Minimum set of fields a manifest must contain to support this contract.

### PSDSC v2 adapter considerations

This RFC assumes a PSDSC-based adapter for DSC v3 that:

- Can load class-based PSDSC resources and recognize the proposed contract.
- May be shipped:
  - As part of the PSDSC module, or
  - As a separate adapter module with a dependency on PSDSC.

> OPEN:
>
> - Final shipping model (in-module vs separate module) and versioning strategy.
> - How to clearly communicate to users what changed when PSDSC or the adapter is updated.

## Alternate Proposals and Considerations

### Functions as the primary contract

An alternative approach was to make top-level functions the DSC v3 contract surface, using the
class only for schema:

- Pros:
  - Familiar for PowerShell users who prefer functions over classes.
- Cons:
  - Requires DSC to reason about a more complex combination of functions and classes.
  - Static analysis and manifest generation are simpler with everything on the class.
  - Harder to express the contract as a single analyzable unit.

This RFC proposes **static class methods** as the primary contract, with authors free to delegate to functions internally.

### Mandatory shared types module

Another alternative was to require all resources to depend on a shared types module (for result types, attributes, etc.):

- Pros:
  - Strong typing and IntelliSense for result objects and attributes.
  - Clear place to evolve shared patterns.
- Cons:
  - Introduces "dependency hell" for resource authors and consumers.
  - Complicates versioning and servicing.
  - Not necessary for basic functionality; generic structured returns are sufficient.

This RFC opts for:

- Generic structured forms (hash tables / objects) as the **baseline**.
- Optional shared types for authors who want richer tooling.

### RDK on the critical path

The working group explicitly does **not** want the Resource Development Kit (RDK) on the critical path:

- RDK should be able to build on this contract once defined.
- The contract and adapter behavior must stand on their own.
- Community and Sampler-based tooling can implement schema/manifest generation independently.

## Related work items

- Issue: "Define method signatures for PSDSC resource classes" (link TBD)

  Describes the need to clarify what methods and signatures DSC v3 should look for on class-based resources.

- Future (potential separate RFCs):
  - PSDSC v2 adapter for DSC v3 (shipping model and behavior).
  - JSON schema/manifest specification for DSC v3 resources.
  - ScriptAnalyzer rule set for V1/V2/V3 DSC resources, including class-based patterns.
