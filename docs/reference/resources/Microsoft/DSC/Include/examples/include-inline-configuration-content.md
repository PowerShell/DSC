---
description: >-
  Demonstrates how to embed a configuration document and its parameters inline with the
  Microsoft.DSC/Include resource.
ms.date:     07/24/2026
ms.topic:    reference
title:       Include inline configuration content
---

This example demonstrates how to use the `Microsoft.DSC/Include` resource to embed a configuration
document and its parameters directly in the parent document. Use inline content when you want to keep
a self-contained configuration in a single file rather than referencing separate files on disk.

## Embed the configuration and parameters

Author the parent configuration document. Instead of referencing files, the `Microsoft.DSC/Include`
instance defines the nested configuration with the `configurationContent` property and the parameter
values with the `parametersContent` property. Both properties accept YAML or JSON as text. The
following example uses YAML block scalars (`|`) to keep the nested documents readable.

```yaml
# main.dsc.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: os info
    type: Microsoft.DSC/Include
    properties:
      configurationContent: |
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        parameters:
          osFamily:
            type: string
            defaultValue: Windows
            allowedValues:
              - Windows
              - Linux
              - macOS
        resources:
          - name: os
            type: Microsoft/OSInfo
            properties:
              family: "[parameters('osFamily')]"
      parametersContent: |
        parameters:
          osFamily: macOS
```

Invoke the [dsc config get][01] command against the parent document to retrieve the state of the
included resources.

```powershell
dsc config get --file ./main.dsc.yaml
```

DSC parses the inline configuration, applies the `osFamily` value from the inline parameters, and
returns the result of the nested `Microsoft/OSInfo` instance. On a macOS machine, the output is
similar to the following YAML:

```yaml
results:
- name: os info
  type: Microsoft.DSC/Include
  result:
  - name: os
    type: Microsoft/OSInfo
    result:
      actualState:
        $id: https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json
        family: macOS
        version: 15.5.0
        bitness: '64'
        architecture: arm64
messages: []
hadErrors: false
```

> [!TIP]
> Inline content is convenient for small configurations and for generating configurations
> programmatically. For larger or reusable configurations, reference a file with the
> `configurationFile` property instead. For that approach, see
> [Include a configuration file](./include-a-configuration-file.md).

## See also

- [Microsoft.DSC/Include resource](../index.md)
- [Include a configuration file](./include-a-configuration-file.md)
- [Microsoft/OSInfo resource][02]

<!-- Link reference definitions -->
[01]: ../../../../../cli/config/get.md
[02]: ../../../../Microsoft/OSInfo/index.md
