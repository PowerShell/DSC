---
description: >
  Example showing how to inspect, create, verify, and remove UTF-8 content in a local file with
  the Microsoft.Filesystem.File/Content resource.
ms.date:     10/19/2026
ms.topic:    reference
title:       Manage a local file's content
---

# Manage a local file's content

This example follows the supplied `Microsoft.Filesystem.File/Content` sample configurations. It
reads an existing file, sets the content of a file, and verifies the result.

## Get the content of an existing file

The following command retrieves the actual content and hashes for the sample `test.txt` file with
the [dsc resource get][03] command. Update the path if you saved the sample files in a different
location.

```powershell
$instance = @{
  path = 'D:\DSC\config\file-content\test.txt'
} | ConvertTo-Json -Compress

dsc resource get --resource Microsoft.Filesystem.File/Content --input $instance
```

The result includes the file's UTF-8 `content`, `sha256`, `sha512`, and `_exist: true` properties.
For the sample file containing `Hello World` and `Hello World 2`, the output is:

```yaml
actualState:
  path: D:\DSC\config\file-content\test.txt
  content: "Hello World\r\nHello World 2"
  sha256: 4cb711e6d67352a139c55c3ae57a7d8a84f454479bd48f031fbcc08f254a24da
  sha512: 2ebbe8171d841341fa9c4d00414a385200ac9d1308fd53ec6b45322f9ca7f3ceb6fa6f1bf2ac578017a311c2730139718598c2c287d64f421a72e01f1217e6e6
  _exist: true
```

## Export the configured file

Although the resource doesn't provide a resource-level export operation for discovering every
file on the system, you can use [dsc config export][06] to export the current state of a file
declared in a configuration document. The input configuration identifies the file by its `path`.

:::code language="yaml" source="export-file-content.config.dsc.yaml":::

Save the configuration as `export-file-content.config.dsc.yaml`, then run:

```powershell
dsc config export --file ./export-file-content.config.dsc.yaml
```

You can also pass the configuration document as JSON using `--input`:

```powershell
$instance = @{
  resources = @(
    @{
      name       = 'Test File'
      type       = 'Microsoft.Filesystem.File/Content'
      properties = @{
        path = 'D:\DSC\config\file-content\test.txt'
      }
    }
  )
} | ConvertTo-Json -Depth 10 -Compress

dsc config export --input $instance
```

For the sample file containing `Hello World` and `Hello World 2`, the command returns a
configuration with its current content and hashes:

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
contentVersion: 1.0.0
resources:
- type: Microsoft.Filesystem.File/Content
  name: Content-0
  properties:
    path: C:\dsc\config\file-content\test.txt
    content: "Hello World\r\nHello World 2"
    sha256: 4cb711e6d67352a139c55c3ae57a7d8a84f454479bd48f031fbcc08f254a24da
    sha512: 2ebbe8171d841341fa9c4d00414a385200ac9d1308fd53ec6b45322f9ca7f3ceb6fa6f1bf2ac578017a311c2730139718598c2c287d64f421a72e01f1217e6e6
    _exist: true
```

## Define the desired file content

The configuration document below follows the sample that sets the content of `TestFile.txt`.
Save it as `file-content.config.dsc.yaml` and update `path` if needed.

:::code language="yaml" source="file-content.config.dsc.yaml":::

## Set a file with multiline content

For files with multiple lines, use a YAML literal block (`|`) for the `content` property. This
example creates the parent directories and writes a CA `.inf` file with the specified lines.

:::code language="yaml" source="complex-file-content.config.dsc.yaml":::

Save the configuration as `complex-file-content.config.dsc.yaml`, then apply it with
[dsc config set][04]:

```powershell
dsc config set --file ./complex-file-content.config.dsc.yaml
```

## Set the file content

The configuration sets `_exist` to `true` and provides `content`. When you apply it with the
[dsc config set][04] command, the resource creates any missing parent directories and then creates
the file with the specified content. If the file already exists, the resource replaces its
content.

```powershell
$desired = @{
  path    = 'D:\DSC\config\file-content\TestFile.txt'
  _exist  = $true
  content = 'This is a test file.'
} | ConvertTo-Json -Compress

dsc resource set --resource Microsoft.Filesystem.File/Content --input $desired
```

The result's `afterState` contains the configured content and its calculated hashes. You can also
apply the included configuration document with `dsc config set --file ./file-content.config.dsc.yaml`.

```yaml

beforeState:                                                                                                                                                                                            
  path: D:\DSC\config\file-content\TestFile.txt
  _exist: false
afterState:
  path: D:\DSC\config\file-content\TestFile.txt
  content: This is a test file.
  sha256: f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de
  sha512: b1df216b5b05e3965c469492744a5de0c945e0b103c42eb1e57476fbed8f1d489f5cae9b792db37c5d823bc0c6c7d06b056176d6abe5ce076eeadaed414e17a3
  _exist: true
changedProperties:
- content
- sha256
- sha512
- _exist

```

## Test the file content

Use the [dsc resource test][02] command to verify that the file matches the desired content.

```powershell
dsc resource test --resource Microsoft.Filesystem.File/Content --input $desired
```

The result reports the resource instance in the desired state:

```yaml
desiredState:                                                                                                                                                                                           
  content: This is a test file.
  _exist: true
  path: D:\DSC\config\file-content\TestFile.txt
actualState:
  path: D:\DSC\config\file-content\TestFile.txt
  content: This is a test file.
  sha256: f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de
  sha512: b1df216b5b05e3965c469492744a5de0c945e0b103c42eb1e57476fbed8f1d489f5cae9b792db37c5d823bc0c6c7d06b056176d6abe5ce076eeadaed414e17a3
  _exist: true
  _inDesiredState: true
inDesiredState: true
differingProperties: []
```

To test the included configuration document instead, use [dsc config test][05].

## Remove the file

To remove the file, set `_exist` to `false`. This uses the resource's **Set** operation; the
resource doesn't expose a separate **Delete** capability.

```powershell
$cleanup = @{
  path   = 'D:\DSC\config\file-content\TestFile.txt'
  _exist = $false
} | ConvertTo-Json -Compress

dsc resource set --resource Microsoft.Filesystem.File/Content --input $cleanup
```

<!-- Link reference definitions -->
[01]: ../../../../../cli/resource/set.md
[02]: ../../../../../cli/resource/test.md
[03]: ../../../../../cli/resource/get.md
[04]: ../../../../../cli/config/set.md
[05]: ../../../../../cli/config/test.md
[06]: ../../../../../cli/config/export.md
