---
description: Microsoft.OpenSSH.SSHD/sshd_config resource reference documentation
ms.date: 05/07/2026
ms.topic: reference
title: Microsoft.OpenSSH.SSHD/sshd_config
---

# Microsoft.OpenSSH.SSHD/sshd_config

## Synopsis

Manage SSH Server Configuration.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.OpenSSH.SSHD/sshd_config
    properties:
      # Any sshd_config directive as a key
      <directive>: <value>
```

## Condition

The resource only applies on systems where the `sshd` executable is available in PATH. DSC
evaluates this with the expression `[not(equals(tryWhich('sshd'), null()))]` and skips the
resource if `sshd` is not found.

## Description

The `Microsoft.OpenSSH.SSHD/sshd_config` resource enables you to idempotently manage SSH server
configuration settings stored in the `sshd_config` file. The resource can:

- Retrieve current SSH server configuration settings.
- Apply desired SSH server configuration settings.
- Export all current SSH server configuration settings as individual resource instances.

> [!NOTE]
> This resource is installed with DSC itself on systems.
>
> You can update this resource by updating DSC. When you update DSC, the updated version of this
> resource is automatically available.

## Requirements

- The resource requires OpenSSH server to be installed on the system.
- The resource must run in a process context that has permissions to read and write the `sshd_config`
  file.
- On Windows, the default configuration file path is `%ProgramData%\ssh\sshd_config`.
- On Linux, the default configuration file path is `/etc/ssh/sshd_config`.
- To manage a configuration file in a non-default location, specify the
  [sshd_config_filepath](#sshd_config_filepath) property for the instance.

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of an instance.
- `set` - You can use the resource to enforce the desired state for an instance.
- `export` - You can use the resource to export all current SSH server configuration settings as
  individual resource instances.

This resource uses the synthetic test functionality of DSC to determine whether an instance is in
the desired state. For more information about resource capabilities, see
[DSC resource capabilities][00].

## Examples

1. [Export OpenSSH configuration][01] - Shows how to export current OpenSSH configuration.
2. [Manage SSH server configuration settings][02] - Shows how to get and set specific sshd_config
   directives.

## Properties

The `Microsoft.OpenSSH.SSHD/sshd_config` resource uses an open-object schema where each property
corresponds to an `sshd_config` directive. There are no fixed required or key properties. Any
valid `sshd_config` keyword can be used as a property name with its corresponding value.

For example:

```yaml
PermitRootLogin: 'no'
PasswordAuthentication: 'no'
Port: 22
```

For the full list of supported directives and their values, see the
[sshd_config man page][05] or the OpenSSH documentation.

In addition to the `sshd_config` directives, the resource defines the following properties that
control how it reads and writes the configuration file.

- [sshd_config_filepath](#sshd_config_filepath) - The path to the `sshd_config` file to manage.
- [_includeDefaults](#_includedefaults) - Whether to include settings inherited from the OpenSSH
  defaults.
- [_inheritedDefaults](#_inheriteddefaults) - The directives that came from the OpenSSH defaults.
- [_purge](#_purge) - Whether to remove directives that aren't defined in the instance.

### sshd_config_filepath

<details><summary>Expand for <code>sshd_config_filepath</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the path to the `sshd_config` file that the resource reads from and writes to. When you
don't specify this property, the resource uses the default path for the operating system. Use this
property to manage a configuration file in a non-default location:

```yaml
resources:
- name: Non-default SSH server configuration
  type: Microsoft.OpenSSH.SSHD/sshd_config
  properties:
    sshd_config_filepath: 'C:\ProgramData\ssh\non_default_sshd_config'
    passwordauthentication: 'no'
```

When you specify this property, the resource returns it as part of the result for the `get` and
`export` operations.

### _includeDefaults

<details><summary>Expand for <code>_includeDefaults</code> property metadata</summary>

```yaml
Type             : boolean
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Determines whether the result includes the settings that the SSH server inherits from the OpenSSH
defaults, rather than only the directives explicitly set in the configuration file.

The default value for this property depends on the operation:

- For the `get` operation, the default value is `true`. Set the property to `false` to return only
  the directives explicitly set in the configuration file.
- For the `export` operation, the default value is `false`. Set the property to `true` to return
  the full effective configuration.

### _inheritedDefaults

<details><summary>Expand for <code>_inheritedDefaults</code> property metadata</summary>

```yaml
Type             : array
IsRequired       : false
IsKey            : false
IsReadOnly       : true
IsWriteOnly      : false
```

</details>

Lists the directives whose values come from the OpenSSH defaults instead of the configuration
file. The `get` operation returns this property when the result includes inherited defaults, in
addition to the returned directives and their values:

```yaml
actualState:
  port: '22'
  addressfamily: any
  passwordauthentication: 'no'
  _inheritedDefaults:
  - port
  - addressfamily
```

This property is read-only. The resource returns it in the result, but you can't set it as part
of the desired state.

### _purge

<details><summary>Expand for <code>_purge</code> property metadata</summary>

```yaml
Type             : boolean
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : true
```

</details>

Determines whether the `set` operation removes directives from the `sshd_config` file that aren't
defined in the instance. The default value is `false`.

When this property is `false`, the resource enforces only the directives defined in the instance
and leaves any other directives in the file untouched. When this property is `true`, the resource
rewrites the file to match the instance exactly, removing any directive that isn't defined,
including directives added outside of DSC.

## Instance validating schema

The resource uses an embedded open-object schema. Any `sshd_config` directive is a valid property.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "sshdconfig",
  "type": "object",
  "properties": {
    "sshd_config_filepath": {
      "type": "string",
      "description": "Path to the sshd_config file to be processed. If not specified, the default path for the OS is used."
    }
  },
  "additionalProperties": true
}
```

## Exit codes

The resource returns the following exit codes from operations:

- [0](#exit-code-0) - Success
- [1](#exit-code-1) - Invalid parameter

### Exit code 0

Indicates the resource operation completed without errors.

### Exit code 1

Indicates the resource operation failed due to an invalid parameter. When the resource returns this
exit code, it also emits an error message with details about the invalid parameter.

## See also

- [Microsoft.OpenSSH.SSHD/Windows resource][03]
- For more information about OpenSSH, see [OpenSSH Documentation][04]

<!-- Link definitions -->
[00]: ../../../../../concepts/resources/capabilities.md
[01]: examples/export-openssh-configuration.md
[02]: examples/manage-sshd-settings.md
[03]: ../Windows/index.md
[04]: /windowsserverdocs/WindowsServerDocs/administration/OpenSSH/openssh-overview
[05]: https://man.openbsd.org/sshd_config
