---
description: JSON schema reference for the directives in a Desired State Configuration document.
ms.date:     10/13/2026
ms.topic:    reference
title:       DSC Configuration document directives schema
---

# DSC Configuration document directives schema

## Synopsis

Defines how DSC processes a configuration document as a whole.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/config/directive.json
Type:          object
```

## Description

The `directives` property of a DSC Configuration document defines how DSC processes the document
as a whole. Unlike the document's [metadata][01], which is informational, DSC validates every
directive and acts on it before invoking any resource instance. This property was added in DSC
version 3.2.

Every directive is optional. When the `directives` property isn't defined, DSC processes the
document with the default value for every directive.

A resource instance can define its own [directives][02] to override how DSC processes that
instance.

## Examples

### Example 1 - Require an elevated session and a minimum version of DSC

The following configuration document requires DSC version 3.2.0 or later and an elevated session.
DSC validates both requirements before it invokes any resource instance.

```yaml
# directives.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
directives:
  securityContext: elevated
  version: '>=3.2.0'
resources:
  - name: Echo
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: Running elevated on DSC 3.2.0 or later
```

When you invoke DSC in a non-elevated session, DSC raises an error and doesn't invoke the `Echo`
instance:

```sh
dsc config get --file directives.example.1.dsc.config.yaml
```

```text
ERROR Error: Security context: Elevated security context required
```

## Properties

### resourceDiscovery

The `resourceDiscovery` directive controls when DSC raises an error for a resource that it can't
find. When you don't define this directive or set it to `preDeployment`, DSC discovers resources
and extensions before invoking any resource instance and raises an error if any instance in the
document uses a resource that DSC didn't discover.

Set this directive to `duringDeployment` to defer this check until DSC processes each instance.
This is useful when the configuration document itself installs a resource that a later instance
depends on. With this setting, DSC performs discovery again when it processes an instance whose
resource wasn't initially discovered and only raises an error if the resource isn't available at
that time.

```yaml
Type:        string
Required:    false
Default:     preDeployment
ValidValues: [preDeployment, duringDeployment]
```

### securityContext

The `securityContext` directive defines the security context the configuration document requires.
Before invoking any resource instances, DSC validates that it's running in the required security
context and raises an error if it isn't:

- `current` - DSC can process the document in any security context. This is the default.
- `elevated` - DSC must be running as `root` (non-Windows) or in an elevated session with
  Administrator privileges (Windows).
- `restricted` - DSC must be running as a normal user or account in a non-elevated session.

This directive replaces the deprecated `Microsoft.DSC.securityContext` property in the document's
[metadata][03]. If you define both, the values must match or DSC raises an error. A resource
instance can override this directive with its own [securityContext][04] directive.

```yaml
Type:        string
Required:    false
Default:     current
ValidValues: [current, elevated, restricted]
```

### version

The `version` directive defines a semantic version requirement for DSC itself. When you define this
directive, DSC compares its own version to the requirement before invoking any resource instances
and raises an error if its version doesn't satisfy the requirement. This enables you to prevent a
configuration document from being processed by an incompatible version of DSC.

The value must be one or more comparators separated by commas. Each comparator is an operator
(`=`, `>`, `>=`, `<`, `<=`, `^`, or `~`) followed by a version, like `>=3.2.0, <4.0.0`. Build
metadata isn't allowed in the version. The syntax is the same as for the `requireVersion` property
of a resource instance. For more information, see [requireVersion][05].

```yaml
Type:     string
Required: false
```

<!-- Link reference definitions -->
[01]: metadata.md
[02]: resource.md#directives
[03]: metadata.md#securitycontext
[04]: resource.md#securitycontext
[05]: resource.md#requireversion
