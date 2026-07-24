---
description: >-
  Demonstrates how to include a configuration document from a file with the Microsoft.DSC/Include
  resource and pass it a parameters file.
ms.date:     07/24/2026
ms.topic:    reference
title:       Include a configuration file
---

This example demonstrates how to use the `Microsoft.DSC/Include` resource to compose a configuration
from a separate configuration document on disk, and how to supply that document with a parameters
file.

## Author the nested configuration document

First, author the configuration document that you want to include. This document defines a parameter
named `osFamily` and uses it to check the operating system with the `Microsoft/OSInfo` resource.
Save it next to the parent document as `osinfo.dsc.yaml`.

```yaml
# osinfo.dsc.yaml
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
```

## Author the parameters file

Next, author the parameters file that supplies a value for the `osFamily` parameter. Save it next to
the parent document as `osinfo.parameters.yaml`.

```yaml
# osinfo.parameters.yaml
parameters:
  osFamily: macOS
```

## Include the configuration and parameters

Finally, author the parent configuration document. The `Microsoft.DSC/Include` instance references
the configuration document with the `configurationFile` property and the parameters file with the
`parametersFile` property. Because the paths are relative, DSC resolves them against the parent
document's directory.

```yaml
# main.dsc.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: os info
    type: Microsoft.DSC/Include
    properties:
      configurationFile: osinfo.dsc.yaml
      parametersFile: osinfo.parameters.yaml
```

Invoke the [dsc config get][01] command against the parent document to retrieve the state of the
included resources.

```powershell
dsc config get --file ./main.dsc.yaml
```

DSC resolves the included configuration, applies the `osFamily` value from the parameters file, and
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
> To run the included configuration with its default parameter values instead, omit the
> `parametersFile` property. The nested document then uses the `defaultValue` defined for each of
> its parameters.

## See also

- [Microsoft.DSC/Include resource](../index.md)
- [Include inline configuration content](./include-inline-configuration-content.md)
- [Microsoft/OSInfo resource][02]

<!-- Link reference definitions -->
[01]: ../../../../../cli/config/get.md
[02]: ../../../../Microsoft/OSInfo/index.md
