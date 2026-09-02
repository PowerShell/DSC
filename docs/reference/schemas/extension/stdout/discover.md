---
description: JSON schema reference for the 'discover' operation output in a DSC extension
ms.date:     09/01/2026
ms.topic:    reference
title:       DSC extension discover operation stdout schema reference
---

# DSC extension discover operation stdout schema reference

## Synopsis

Represents a manifest not discoverable in the `PATH` or `DSC_RESOURCE_PATH` environment variables.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/extension/stdout/discover.json
Type:          object
```

## Description

Represents a DSC manifest that the extension discovered, either as the absolute path to the
manifest file on the system or as the content of the manifest. DSC expects every JSON Line emitted
to stdout for the **Discover** operation to adhere to this schema.

The output must be a JSON object. The object must define exactly one of the [manifestPath][01] or
[manifestContent][02] properties. If an extension returns JSON that is invalid against this schema,
DSC raises an error.

Each discovered manifest must be emitted as a separate JSON Line to stdout. If the extension
doesn't discover any manifests, it must return nothing to stdout and exit with code `0`. An empty
output with a zero exit code indicates no resources were found. A non-zero exit code indicates an
error, even if stdout is empty.

DSC uses the discovered manifests to find resources, including adapted resources and resources
defined in manifest lists. Extensions can't currently discover other extensions. If a discovered
manifest defines an extension, DSC ignores it.

## Required properties

The output for the `discover` operation must include exactly one of these properties:

- [manifestPath](#manifestpath)
- [manifestContent](#manifestcontent)

## Properties

### manifestPath

The value for this property must be the absolute path to a manifest file on the system. DSC
determines how to load the manifest from the file name, so the file name must follow one of the
manifest naming conventions, like `<name>.dsc.resource.json`, `<name>.dsc.adaptedresource.json`,
or `<name>.dsc.manifests.json`.

If the returned path isn't absolute, DSC raises an error. If DSC can't load the manifest at the
returned path, it writes an informational message and skips that manifest.

```yaml
Type:     string
Required: true (when manifestContent isn't defined)
```

### manifestContent

The value for this property must be the content of a manifest as a JSON object. DSC processes the
value the same way it processes the content of a manifest file, including evaluating the
`condition` property of the manifest. The value can be a resource manifest, an adapted resource
manifest, or a manifest list. If the value isn't a valid manifest, DSC raises an error. This
property was added in DSC version 3.3.0.

```yaml
Type:     object
Required: true (when manifestPath isn't defined)
```

## Exit codes

The extension must return one of the following exit codes:

- `0` - Success. The extension completed discovery. If no manifests were found, stdout is empty.
- Non-zero - Error. DSC treats any non-zero exit code as a failure and surfaces the extension's
  stderr output as an error message.

<!-- Link reference definitions -->
[01]: #manifestpath
[02]: #manifestcontent
