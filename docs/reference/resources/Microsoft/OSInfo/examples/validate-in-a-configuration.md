---
description: >
  Use the Microsoft/OSInfo resource to Use the Microsoft/OSInfo resource to validate operating
  system in a DSC Configuration Document.
ms.date: 03/25/2025
ms.topic: reference
title: Validate operating system information in a configuration
---

# Validate operating system information in a configuration

This example shows how you can use the `Microsoft/OSInfo` resource to assert that a configuration
document applies to a specific operating system.

> [!IMPORTANT]
> The `osinfo` command and `Microsoft/OSInfo` resource are a proof-of-concept example for use with
> DSC. Don't use it in production.

## Definition

The configuration document for this example defines a group resource instance called
`Operating System Assertion` with a nested instance of the `Microsoft/OSInfo` resource.

The configuration uses the `Microsoft.DSC/Assertion` group resource to ensure that the **Test**
operation is called for every instance in the group, regardless of the actual configuration
operation being applied. When DSC processes the group resource, it calls the **Test** operation for
the nested instances instead of the **Get** or **Set** operations. Instances in the group never
change the state of the system.

The instance of the `Microsoft/OSInfo` resource defines the `bitness` property to validate that the
configuration is applied on a 64-bit operating system.

:::code language="yaml" source="osinfo.config.dsc.yaml":::

## Getting the current state

To get the current state of the instances in the configuration document, use the
[dsc config get][01] command with the [--file][02] option.

# [Linux](#tab/linux)

```bash
dsc config get --file ./osinfo.config.dsc.yaml
```

# [macOS](#tab/macos)

```zsh
dsc config get --file ./osinfo.config.dsc.yaml
```

# [Windows](#tab/windows)

```powershell
dsc config get --file .\osinfo.config.dsc.yaml
```

---

The output depends on whether the operating system is 32-bit or 64-bit.

# [32-bit Linux](#tab/32bit/linux)

For a 32-bit Linux operating system, the get result shows that the `Is64BitOS` instance is
out of the desired state because the `bitness` property is `32`.

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: get
    executionType: actual
    startDatetime: 2025-03-07T13:32:15.787101400-06:00
    endDatetime: 2025-03-07T13:32:19.077737200-06:00
    duration: PT3.2906358S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT2.5803652S
  name: Operating System Assertion
  type: Microsoft.DSC/Assertion
  result:
  - name: Is64BitOS
    type: Microsoft/OSInfo
    result:
      actualState:
        $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
        family: Linux
        version: '20.04'
        codename: focal
        bitness: '32'
        architecture: i386
messages: []
hadErrors: false
```

# [64-bit Linux](#tab/64bit/linux)

For a 64-bit Linux operating system, the get result shows that the `Is64BitOS` instance is
in the desired state.

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: get
    executionType: actual
    startDatetime: 2025-03-07T13:32:15.787101400-06:00
    endDatetime: 2025-03-07T13:32:19.077737200-06:00
    duration: PT3.2906358S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT2.5803652S
  name: Operating System Assertion
  type: Microsoft.DSC/Assertion
  result:
  - name: Is64BitOS
    type: Microsoft/OSInfo
    result:
      actualState:
        $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
        family: Linux
        version: '20.04'
        codename: focal
        bitness: '64'
        architecture: x86_64
messages: []
hadErrors: false
```

# [32-bit macOS](#tab/32bit/macos)

For a 32-bit macOS operating system, the get result shows that the `Is64BitOS` instance is
out of the desired state because the `bitness` property is `32`.

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: get
    executionType: actual
    startDatetime: 2025-03-07T13:32:15.787101400-06:00
    endDatetime: 2025-03-07T13:32:19.077737200-06:00
    duration: PT3.2906358S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT2.5803652S
  name: Operating System Assertion
  type: Microsoft.DSC/Assertion
  result:
  - name: Is64BitOS
    type: Microsoft/OSInfo
    result:
      actualState:
        $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
        family: MacOS
        version: 13.5.0
        bitness: '32'
        architecture: arm
messages: []
hadErrors: false
```

# [64-bit macOS](#tab/64bit/macos)

For a 64-bit macOS operating system, the get result shows that the `Is64BitOS` instance is
in the desired state.

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: get
    executionType: actual
    startDatetime: 2025-03-07T13:32:15.787101400-06:00
    endDatetime: 2025-03-07T13:32:19.077737200-06:00
    duration: PT3.2906358S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT2.5803652S
  name: Operating System Assertion
  type: Microsoft.DSC/Assertion
  result:
  - name: Is64BitOS
    type: Microsoft/OSInfo
    result:
      actualState:
        $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
        family: MacOS
        version: 13.5.0
        bitness: '64'
        architecture: arm64
messages: []
hadErrors: false
```

# [32-bit Windows](#tab/32bit/windows)

For a 32-bit Windows operating system, the get result shows that the `Is64BitOS` instance is
out of the desired state because the `bitness` property is `32`.

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: get
    executionType: actual
    startDatetime: 2025-03-07T13:32:15.787101400-06:00
    endDatetime: 2025-03-07T13:32:19.077737200-06:00
    duration: PT3.2906358S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT2.5803652S
  name: Operating System Assertion
  type: Microsoft.DSC/Assertion
  result:
  - name: Is64BitOS
    type: Microsoft/OSInfo
    result:
      actualState:
        $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
        family: Windows
        version: 10.0.22621
        edition: Windows 11 Enterprise
        bitness: '32'
messages: []
hadErrors: false
```

# [64-bit Windows](#tab/64bit/windows)

For a 64-bit Windows operating system, the get result shows that the `Is64BitOS` instance is
in the desired state.

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: get
    executionType: actual
    startDatetime: 2025-03-07T13:32:15.787101400-06:00
    endDatetime: 2025-03-07T13:32:19.077737200-06:00
    duration: PT3.2906358S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT2.5803652S
  name: Operating System Assertion
  type: Microsoft.DSC/Assertion
  result:
  - name: Is64BitOS
    type: Microsoft/OSInfo
    result:
      actualState:
        $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
        family: Windows
        version: 10.0.22621
        edition: Windows 11 Enterprise
        bitness: '64'
messages: []
hadErrors: false
```

---

## Verify the desired state

DSC can use the resource to validate the operating system information in a configuration with the [dsc config test][03] command. When you use the `dsc config test` command, DSC invokes the **Test** operation against every resource in the configuration document.

The `Microsoft/OSInfo` resource doesn't implement the [test operation][04]. It relies on the
synthetic testing feature of DSC instead. The synthetic test uses a case-sensitive equivalency
comparison between the actual state of the instance properties and the desired state. If any
property value isn't an exact match, DSC considers the instance to be out of the desired state.

# [Linux](#tab/linux)

```bash
dsc config set --file ./osinfo.config.dsc.yaml
```

# [macOS](#tab/macos)

```zsh
dsc config set --file ./osinfo.config.dsc.yaml
```

# [Windows](#tab/windows)

```powershell
dsc config set --file .\osinfo.config.dsc.yaml
```

---

The output depends on whether the operating system is 32-bit or 64-bit. In all cases, the
`changedProperties` field for the result is an empty list. The `Microsoft.DSC/Assertion` group resource
never changes system state and the `Microsoft/OSInfo` resource doesn't implement the **Set**** operation.

# [32-bit Linux](#tab/32bit/linux)

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: set
    executionType: actual
    startDatetime: 2025-03-07T13:40:50.014780900-06:00
    endDatetime: 2025-03-07T13:40:54.009892500-06:00
    duration: PT3.9951116S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT3.2687015S
  name: Operating System Assertion
  type: Microsoft.DSC/Assertion
  result:
    beforeState:
    - name: Is64BitOS
      type: Microsoft/OSInfo
      result:
        actualState:
          $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
          family: Linux
          version: '20.04'
          codename: focal
          bitness: '32'
          architecture: i386
    afterState:
    - metadata:
        Microsoft.DSC:
          duration: PT0.0439438S
      name: Is64BitOS
      type: Microsoft/OSInfo
      result:
        desiredState:
          bitness: '64'
        actualState:
          $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
          family: Linux
          version: '20.04'
          codename: focal
          bitness: '32'
          architecture: i386
        inDesiredState: false
        differingProperties:
        - bitness
    changedProperties: []
messages: []
hadErrors: false

```

# [64-bit Linux](#tab/64bit/linux)

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: set
    executionType: actual
    startDatetime: 2025-03-07T13:39:27.847812500-06:00
    endDatetime: 2025-03-07T13:39:32.089234100-06:00
    duration: PT4.2414216S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT3.498287S
  name: Operating System Assertion
  type: Microsoft.DSC/Assertion
  result:
    beforeState:
    - name: Is64BitOS
      type: Microsoft/OSInfo
      result:
        actualState:
          $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
          family: Linux
          version: '20.04'
          codename: focal
          bitness: '64'
          architecture: i386
    afterState:
    - metadata:
        Microsoft.DSC:
          duration: PT0.0500784S
      name: Is64BitOS
      type: Microsoft/OSInfo
      result:
        desiredState:
          bitness: '64'
        actualState:
          $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
          family: Linux
          version: '20.04'
          codename: focal
          bitness: '64'
          architecture: i386
        inDesiredState: false
        differingProperties: []
    changedProperties: []
messages: []
hadErrors: false
```

# [32-bit macOS](#tab/32bit/macos)

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: set
    executionType: actual
    startDatetime: 2025-03-07T13:40:50.014780900-06:00
    endDatetime: 2025-03-07T13:40:54.009892500-06:00
    duration: PT3.9951116S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT3.2687015S
  name: Operating System Assertion
  type: Microsoft.DSC/Assertion
  result:
    beforeState:
    - name: Is64BitOS
      type: Microsoft/OSInfo
      result:
        actualState:
          $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
          family: MacOS
          version: 13.5.0
          bitness: '32'
          architecture: arm
    afterState:
    - metadata:
        Microsoft.DSC:
          duration: PT0.0439438S
      name: Is64BitOS
      type: Microsoft/OSInfo
      result:
        desiredState:
          bitness: '64'
        actualState:
          $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
          family: MacOS
          version: 13.5.0
          bitness: '32'
          architecture: arm
        inDesiredState: false
        differingProperties:
        - bitness
    changedProperties: []
messages: []
hadErrors: false
```

# [64-bit macOS](#tab/64bit/macos)

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: set
    executionType: actual
    startDatetime: 2025-03-07T13:39:27.847812500-06:00
    endDatetime: 2025-03-07T13:39:32.089234100-06:00
    duration: PT4.2414216S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT3.498287S
  name: Operating System Assertion
  type: Microsoft.DSC/Assertion
  result:
    beforeState:
    - name: Is64BitOS
      type: Microsoft/OSInfo
      result:
        actualState:
          $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
          family: MacOS
          version: 13.5.0
          bitness: '64'
          architecture: arm64
    afterState:
    - metadata:
        Microsoft.DSC:
          duration: PT0.0500784S
      name: Is64BitOS
      type: Microsoft/OSInfo
      result:
        desiredState:
          bitness: '64'
        actualState:
          $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
          family: MacOS
          version: 13.5.0
          bitness: '64'
          architecture: arm64
        inDesiredState: false
        differingProperties: []
    changedProperties: []
messages: []
hadErrors: false
```

# [32-bit Windows](#tab/32bit/windows)

```yaml
results:
- name: Operating System Assertion
  type: DSC/AssertionGroup
  result:
    beforeState:
      results:
      - name: Is64BitOS
        type: Microsoft/OSInfo
        result:
          desiredState:
            bitness: '64'
          actualState:
            $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
            family: Windows
            version: 10.0.22621
            edition: Windows 11 Enterprise
            bitness: '32'
          inDesiredState: false
          differingProperties:
          - bitness
      messages: []
      hadErrors: false
    afterState:
      results:
      - name: Is64BitOS
        type: Microsoft/OSInfo
        result:
          desiredState:
            bitness: '64'
          actualState:
            $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
            family: Windows
            version: 10.0.22621
            edition: Windows 11 Enterprise
            bitness: '32'
          inDesiredState: false
          differingProperties:
          - bitness
      messages: []
      hadErrors: false
    changedProperties: []
messages: []
hadErrors: false
```

# [64-bit Windows](#tab/64bit/windows)

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: set
    executionType: actual
    startDatetime: 2025-03-07T13:39:27.847812500-06:00
    endDatetime: 2025-03-07T13:39:32.089234100-06:00
    duration: PT4.2414216S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT3.498287S
  name: Operating System Assertion
  type: Microsoft.DSC/Assertion
  result:
    beforeState:
    - name: Is64BitOS
      type: Microsoft/OSInfo
      result:
        actualState:
          $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
          family: Windows
          version: 10.0.22621
          edition: Windows 11 Enterprise
          bitness: '64'
    afterState:
    - metadata:
        Microsoft.DSC:
          duration: PT0.0500784S
      name: Is64BitOS
      type: Microsoft/OSInfo
      result:
        desiredState:
          bitness: '64'
        actualState:
          $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
          family: Windows
          version: 10.0.22621
          edition: Windows 11 Enterprise
          bitness: '64'
        inDesiredState: false
        differingProperties: []
    changedProperties: []
messages: []
hadErrors: false
```

---

<!-- Link reference definitions -->
[01]: ../../../../cli/config/get.md
[02]: ../../../../cli/config/get.md#--file
[03]: ../../../../cli/config/test.md
[04]: ../../../../../concepts/resources/operations.md#test-operation
