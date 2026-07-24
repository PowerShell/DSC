---
description: >-
  Learn how to configure DSC's behavior using the settings system, including default settings,
  user settings, and policy settings for administrative control.
ms.date: 08/12/2025
title: Configure DSC settings
---

# Configure DSC settings

Microsoft's Desired State Configuration (DSC) platform uses a layered settings system that allows
you to customize its behavior. You can control various aspects of DSC operation through JSON
configuration files that are evaluated in a specific priority order.

Understanding how to configure DSC settings enables you to:

- Customize DSC's default behavior for your environment
- Set user-specific preferences for development workflows  
- Implement organization-wide policies that override user settings
- Control resource discovery paths and execution parameters

## Settings file hierarchy

DSC evaluates settings files in the following priority order, with later files overriding earlier
ones:

1. **Default settings** - Built-in configuration shipped with DSC
2. **User settings** - Custom configuration you create
3. **Policy settings** - Administrative settings that override all others

This layered approach ensures that administrators can enforce organization-wide policies while
still allowing users to customize their local DSC experience within those constraints.

### Default settings file

DSC ships with a default settings file named `dsc_default.settings.json` located in the same
directory as the DSC executable. This file contains the baseline configuration that DSC uses when
no other settings are specified.

> [!IMPORTANT]
> Don't modify the default settings file directly. Your changes will be lost when you update DSC.
> Instead, use the user settings file to override specific values.

### User settings file

Create a file named `dsc.settings.json` in the same directory as the DSC executable to override
default settings. This file should contain only the settings you want to change, not a complete
copy of the default settings.

The user settings file is where you define your personal preferences and development-specific
configurations. Common use cases include:

- Customizing resource discovery paths
- Setting default execution parameters
- Configuring logging levels for development

### Policy settings file

Administrators can create a policy settings file that overrides both default and user settings.
This file ensures that certain organizational requirements are enforced regardless of user
preferences.

The policy settings file location depends on your operating system:

- **Windows**: `%ProgramData%\dsc\dsc.settings.json`
- **Linux/macOS**: `/etc/dsc/dsc.settings.json`

Policy files are typically writable only by administrators but readable by all users, ensuring
that organizational policies can't be bypassed by individual users.

> [!IMPORTANT]
> Policy settings are designed for organizational environments and illustrated purposes only.
> They cannot be enforced for users with administrator privileges and are not managed by
> enterprise tools like Group Policy Management. Users with sufficient privileges can modify
> or override policy settings on their local systems.

## Settings file format

DSC settings files use a versioned JSON format. The root object contains a version number as the
key, with all settings nested under that version. This structure allows DSC to maintain backward
compatibility as the settings schema evolves.

```json
{
  "1": {
    "resourcePath": {
      "allowEnvOverride": true,
      "appendEnvPath": true,
      "directories": []
    },
    "tracing": {
      "level": "WARN",
      "format": "Default",
      "allowOverride": true
    }
  }
}
```

> [!NOTE]
> Settings files must contain valid JSON. Comments are not supported in JSON, so document your
> configuration choices in separate documentation files. The version number "1" corresponds to
> the current settings schema version and must be included as the root key.

## Available settings

DSC supports the following configuration settings organized by category:

### Resource path settings

The `resourcePath` setting controls how DSC discovers and loads resources from the file system.

```json
{
  "1": {
    "resourcePath": {
      "allowEnvOverride": true,
      "appendEnvPath": true,
      "directories": [
        "C:\\CustomResources",
        "C:\\CompanyResources"
      ]
    }
  }
}
```

**Properties:**

- `allowEnvOverride` (boolean): Whether to allow the `DSC_RESOURCE_PATH` environment variable to
  override the configured directories. When `true`, the environment variable takes precedence over
  the `directories` setting. When `false`, only the configured directories are used.

- `appendEnvPath` (boolean): Whether to append the system's `PATH` environment variable to the
  resource search paths. When `true`, DSC searches both the configured directories and all
  directories in the system PATH. This setting is ignored when `allowEnvOverride` is `true` and
  the `DSC_RESOURCE_PATH` environment variable is set.

- `directories` (array of strings): Custom directories where DSC should search for resource
  manifests and executables. These paths are searched before the system PATH when
  `appendEnvPath` is `true`.

### Tracing settings

The `tracing` setting controls DSC's logging and diagnostic output behavior.

```json
{
  "1": {
    "tracing": {
      "level": "WARN",
      "format": "Default",
      "allowOverride": true
    }
  }
}
```

**Properties:**

- `level` (string): The minimum log level to output. Valid values are `ERROR`, `WARN`, `INFO`,
  `DEBUG`, and `TRACE`. Higher levels include all lower-level messages (e.g., `WARN` includes
  `ERROR` messages).

- `format` (string): The format for log output. Valid values are `default` for standard
  console output, `plaintext` for unformatted text output, and `json` for structured JSON logging.

- `allowOverride` (boolean): Whether command-line arguments or environment variables can override
  the configured tracing level. When `false`, the configured level is enforced regardless of
  other settings.

## Common settings scenarios

### Example 1 - Custom resource paths

Configure additional directories where DSC should search for resources:

```json
{
  "1": {
    "resourcePath": {
      "allowEnvOverride": true,
      "appendEnvPath": true,
      "directories": [
        "C:\\CustomResources",
        "C:\\CompanyResources\\Production"
      ]
    }
  }
}
```

This setting extends the default resource discovery mechanism to include your custom resource
locations, making them available for use in configuration documents.

### Example 2 - Development environment configuration

Create a user settings file optimized for development work:

```json
{
  "1": {
    "resourcePath": {
      "allowEnvOverride": true,
      "appendEnvPath": true,
      "directories": [
        "C:\\Development\\MyResources",
        "C:\\Development\\TestResources"
      ]
    },
    "tracing": {
      "level": "DEBUG",
      "format": "Default",
      "allowOverride": true
    }
  }
}
```

This configuration increases logging verbosity for troubleshooting, adds development resource
paths, and maintains flexibility for environment variable overrides.

### Example 3 - Organization-wide policy enforcement

An administrator might create a policy settings file to enforce security and compliance
requirements:

```json
{
  "1": {
    "resourcePath": {
      "allowEnvOverride": false,
      "appendEnvPath": false,
      "directories": [
        "\\\\CompanyShare\\ApprovedResources"
      ]
    },
    "tracing": {
      "level": "WARN",
      "format": "Default",
      "allowOverride": false
    }
  }
}
```

This policy configuration restricts resource usage to approved company resources, disables
environment variable overrides for security, and prevents users from changing tracing settings.

## Best practices

### Use minimal configuration files

Only include settings you need to change in your configuration files. This approach:

- Makes your configuration easier to understand and maintain
- Reduces conflicts when DSC updates change default values
- Clearly shows which settings you've customized

### Document your settings choices

Keep documentation alongside your settings files explaining:

- Why you changed specific settings
- What impact the changes have on DSC behavior
- Any dependencies between settings

### Test settings changes carefully

Before deploying settings changes:

- Test them in a development environment first
- Verify that existing configuration documents still work correctly
- Check that the settings produce the expected behavior changes

### Use policy settings judiciously

Policy settings override all user preferences, so use them only for:

- Security requirements that must be enforced
- Compliance settings that users shouldn't override
- Organization-wide standards that ensure consistency

## Troubleshooting settings

### Verify settings are being loaded

Use DSC's debug logging to verify which settings files are being read and which values are being
applied. Enable debug logging in your user settings:

```json
{
  "1": {
    "tracing": {
      "level": "DEBUG",
      "format": "Default",
      "allowOverride": true
    }
  }
}
```

When debug logging is enabled, DSC outputs detailed information about:

- Which settings files are being loaded and from what locations
- The final resolved values for each setting after applying the hierarchy
- Whether environment variables are overriding configured values

### Check file locations and permissions

Ensure that:

- Settings files are in the correct locations (same directory as DSC executable for user settings)
- Files have proper read permissions for the user running DSC
- JSON syntax is valid
- The version number "1" is included as the root key in the JSON structure

### Understanding setting precedence

Remember the priority order when troubleshooting unexpected behavior:

1. Policy settings always win
2. User settings override defaults
3. Default settings provide the baseline

If a setting isn't working as expected, check whether it's being overridden by a higher-priority
file.

## Security considerations

### Protect sensitive settings

If your settings files contain sensitive information:

- Use appropriate file system permissions
- Consider using environment variables or secure storage for sensitive values
- Avoid storing credentials directly in settings files

Currently, DSC's available settings do not contain sensitive data, but future versions may
introduce settings that include credentials, tokens, or other sensitive information.

> [!WARNING]
> When using `TRACE` level logging, DSC may output sensitive data including configuration values,
> resource parameters, and system information. Use TRACE logging only in secure development
> environments and avoid it in production systems where sensitive data could be exposed.

### Policy file security

Policy settings files should be:

- Writable only by administrators
- Readable by all users who need to run DSC
- Protected from unauthorized modification

The default policy file locations provide this security model, but verify permissions match your
organization's security requirements.

## Related content

- [DSC Configuration document overview][01]
- [DSC Resource anatomy][02]
- [Improve the accessibility of DSC output in PowerShell][03]

<!-- Link reference definitions -->
[01]: ./configuration-documents/overview.md
[02]: ./resources/anatomy.md
[03]: ./output-accessibility.md
