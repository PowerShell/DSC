---
description: Reference for the 'range' DSC configuration document function
ms.date:     09/26/2025
ms.topic:    reference
title:       range
---

## Synopsis

Creates an array of integers from a starting integer and containing a number of
items.

## Syntax

```Syntax
range(startIndex, count)
```

## Description

The `range()` function generates a sequence of consecutive integers starting
from `startIndex` and containing `count` number of items. This is useful for
creating numeric sequences, iterating over indices, or generating test data.

The sum of `startIndex` and `count` must not exceed 2,147,483,647 (the maximum
value for a 32-bit signed integer). The `count` parameter must be a
non-negative integer up to 10,000.

## Examples

### Example 1 - Generate server port numbers for load balancer configuration

Use `range()` to create a sequence of port numbers for configuring multiple
backend servers in a load balancer. This ensures consistent port allocation
across your infrastructure.

```yaml
# range.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Load Balancer Ports
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      backendPorts: "[range(8080, 5)]"
      alternativePorts: "[range(9000, 3)]"
```

```bash
dsc config get --file range.example.1.dsc.config.yaml
```

```yaml
results:
- name: Load Balancer Ports
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        backendPorts:
        - 8080
        - 8081
        - 8082
        - 8083
        - 8084
        alternativePorts:
        - 9000
        - 9001
        - 9002
messages: []
hadErrors: false
```

### Example 2 - Create worker node identifiers for container orchestration

Generate sequential identifiers for worker nodes in a container cluster. This
is helpful when provisioning multiple identical workers that need unique
numeric identifiers.

```yaml
# range.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Worker Node IDs
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      workerIds: "[range(1, 8)]"
      testNodeIds: "[range(100, 3)]"
```

```bash
dsc config get --file range.example.2.dsc.config.yaml
```

```yaml
results:
- name: Worker Node IDs
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        workerIds:
        - 1
        - 2
        - 3
        - 4
        - 5
        - 6
        - 7
        - 8
        testNodeIds:
        - 100
        - 101
        - 102
messages: []
hadErrors: false
```

### Example 3 - Generate database partition numbers with negative starting values

Create partition identifiers that include negative numbers, useful for
time-series data partitioning or when working with offset-based indexing
systems.

```yaml
# range.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Database Partitions
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      timeOffsets: "[range(-3, 7)]"
      emptyRange: "[range(50, 0)]"
```

```bash
dsc config get --file range.example.3.dsc.config.yaml
```

```yaml
results:
- name: Database Partitions
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        timeOffsets:
        - -3
        - -2
        - -1
        - 0
        - 1
        - 2
        - 3
        emptyRange: []
messages: []
hadErrors: false
```

### Example 4 - Create year sequences for data archiving policies

Generate sequences of years for implementing data retention policies or
creating time-based archive structures. This example shows practical year
ranges for typical business scenarios.

```yaml
# range.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Archive Years
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      recentYears: "[range(2020, 5)]"
      fiscalYears: "[range(2022, 3)]"
```

```bash
dsc config get --file range.example.4.dsc.config.yaml
```

```yaml
results:
- name: Archive Years
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        recentYears:
        - 2020
        - 2021
        - 2022
        - 2023
        - 2024
        fiscalYears:
        - 2022
        - 2023
        - 2024
messages: []
hadErrors: false
```

## Parameters

### startIndex

The first integer in the array.

```yaml
Type:     int
Required: true
Position: 1
```

### count

The number of integers in the array. Must be a non-negative integer up to
10,000. The sum of `startIndex` and `count` must not exceed 2,147,483,647.

```yaml
Type:     int
Required: true
Position: 2
```

## Output

Returns an array of consecutive integers starting from `startIndex`.

```yaml
Type: array
```

## Related functions

- [`createArray()`][00] - Creates an array from individual values
- [`length()`][01] - Returns the number of elements in an array
- [`first()`][02] - Returns the first element of an array
- [`skip()`][03] - Returns array elements after skipping a specified number

<!-- Link reference definitions -->
[00]: ./createArray.md
[01]: ./length.md
[02]: ./first.md
[03]: ./skip.md
