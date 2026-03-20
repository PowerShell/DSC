---
description: Reference for the 'take' DSC configuration document function
ms.date:     11/01/2025
ms.topic:    reference
title:       take
---

## Synopsis

Returns an array with the specified number of elements from the start of an
array, or a string with the specified number of characters from the start of a
string.

## Syntax

```Syntax
take(originalValue, numberToTake)
```

## Description

The `take()` function extracts a specified number of elements from the beginning
of an array or characters from the beginning of a string. This is useful for
limiting results, implementing pagination, or extracting prefixes from larger
datasets.

- For arrays: returns a new array containing the first `n` elements
- For strings: returns a new string containing the first `n` characters

Both parameters are required. The `originalValue` must be an array or a string.
The `numberToTake` must be an integer. If the number is zero or negative, an
empty array or empty string is returned. If the number is larger than the length
of the array or string, all elements or characters are returned.

This function is particularly useful when you need to:

- Limit the number of items processed from a list
- Extract a fixed-length prefix from identifiers or paths
- Implement top-N selections without complex filtering
- Create pagination or batch processing logic

## Examples

### Example 1 - Limit deployment to top priority servers

Deploy configuration changes to only the highest priority servers first, limiting
risk during rollout. The `take()` function extracts the first N servers from
your priority list. This example uses [`parameters()`][00] to reference the
server list.

```yaml
# take.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  allServers:
    type: array
    defaultValue:
      - prod-web-01
      - prod-web-02
      - prod-web-03
      - prod-web-04
      - prod-web-05
resources:
- name: Priority Deployment
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      allServers: "[parameters('allServers')]"
      deployToFirst: "[take(parameters('allServers'), 2)]"
      description: Deploy to first 2 servers in priority order
```

```bash
dsc config get --file take.example.1.dsc.config.yaml
```

```yaml
results:
- name: Priority Deployment
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        allServers:
        - prod-web-01
        - prod-web-02
        - prod-web-03
        - prod-web-04
        - prod-web-05
        deployToFirst:
        - prod-web-01
        - prod-web-02
        description: Deploy to first 2 servers in priority order
messages: []
hadErrors: false
```

The function returns only the first two servers from the list, allowing you to
implement a staged rollout strategy.

### Example 2 - Extract environment prefix from resource names

When working with standardized naming conventions, extracting prefixes helps
with categorization and routing logic. This example shows how to use `take()` to
get environment codes from resource identifiers. This example uses
[`createArray()`][01] to build the resource name list.

```yaml
# take.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Resource Prefixes
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      resources: "[createArray('prod-db-east-01', 'dev-api-west-02', 'test-cache-central')]"
      prodPrefix: "[take('prod-db-east-01', 4)]"
      devPrefix: "[take('dev-api-west-02', 3)]"
      testPrefix: "[take('test-cache-central', 4)]"
```

```bash
dsc config get --file take.example.2.dsc.config.yaml
```

```yaml
results:
- name: Resource Prefixes
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        resources:
        - prod-db-east-01
        - dev-api-west-02
        - test-cache-central
        prodPrefix: prod
        devPrefix: dev
        testPrefix: test
messages: []
hadErrors: false
```

The function extracts the environment prefix from each resource name, enabling
environment-specific configuration logic.

### Example 3 - Implement batch processing with size limits

Processing items in controlled batches prevents resource exhaustion when dealing
with large datasets. By using `take()`, you can limit the number of items
processed in each run. This example uses [`parameters()`][00] to reference the
pending jobs array and [`length()`][02] to show the total count.

```yaml
# take.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  pendingJobs:
    type: array
    defaultValue:
      - job-001
      - job-002
      - job-003
      - job-004
      - job-005
      - job-006
      - job-007
      - job-008
resources:
- name: Batch Processing
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      totalPending: "[length(parameters('pendingJobs'))]"
      currentBatch: "[take(parameters('pendingJobs'), 3)]"
      batchSize: 3
      description: Process first 3 jobs from queue
```

```bash
dsc config get --file take.example.3.dsc.config.yaml
```

```yaml
results:
- name: Batch Processing
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        totalPending: 8
        currentBatch:
        - job-001
        - job-002
        - job-003
        batchSize: 3
        description: Process first 3 jobs from queue
messages: []
hadErrors: false
```

The function returns the first three jobs for processing, allowing you to
implement controlled batch processing with predictable resource usage.

### Example 4 - Select top-N log entries for monitoring

Pagination-style access to log entries or event streams can be implemented by
combining `take()` with [`skip()`][03]. This example shows how to get the most
recent entries while demonstrating the complementary relationship between these
functions. This example uses [`parameters()`][00] to reference the log entries
array.

```yaml
# take.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  recentLogs:
    type: array
    defaultValue:
      - 2025-11-01 10:00:00 - System started
      - 2025-11-01 10:05:32 - User login: admin
      - 2025-11-01 10:07:15 - Config updated
      - 2025-11-01 10:12:48 - Service restarted
      - 2025-11-01 10:15:03 - Backup completed
      - 2025-11-01 10:20:17 - Health check passed
      - 2025-11-01 10:25:44 - Cache cleared
resources:
- name: Log Monitoring
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      topThree: "[take(parameters('recentLogs'), 3)]"
      nextThree: "[take(skip(parameters('recentLogs'), 3), 3)]"
      description: Show first 3 and next 3 log entries
```

```bash
dsc config get --file take.example.4.dsc.config.yaml
```

```yaml
results:
- name: Log Monitoring
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        topThree:
        - 2025-11-01 10:00:00 - System started
        - 2025-11-01 10:05:32 - User login: admin
        - 2025-11-01 10:07:15 - Config updated
        nextThree:
        - 2025-11-01 10:12:48 - Service restarted
        - 2025-11-01 10:15:03 - Backup completed
        - 2025-11-01 10:20:17 - Health check passed
        description: Show first 3 and next 3 log entries
messages: []
hadErrors: false
```

By combining `take()` and `skip()`, you can implement pagination logic to
process logs or events in manageable chunks.

## Parameters

### originalValue

The array or string to take elements from. Required.

```yaml
Type:     array | string
Required: true
Position: 1
```

### numberToTake

The number of elements or characters to take from the start. Must be an integer.
If this value is 0 or less, an empty array or string is returned. If it's larger
than the length of the given array or string, all elements or characters are
returned. Required.

```yaml
Type:     integer
Required: true
Position: 2
```

## Output

Returns the same type as `originalValue`:

- If `originalValue` is an array, returns an array with up to `numberToTake`
  elements from the start
- If `originalValue` is a string, returns a string with up to `numberToTake`
  characters from the start

```yaml
Type: array | string
```

## Errors

The function returns an error in the following cases:

- **Invalid original value type**: The first argument is not an array or string
- **Invalid number type**: The second argument is not an integer

## Related functions

- [`skip()`][03] - Returns an array or string with elements skipped from the start
- [`first()`][04] - Returns the first element of an array or first character of a string
- [`last()`][05] - Returns the last element of an array or last character of a string
- [`length()`][02] - Returns the number of elements in an array or characters in a string
- [`createArray()`][01] - Creates an array from provided values
- [`parameters()`][00] - Returns the value of a specified configuration parameter

<!-- Link reference definitions -->
[00]: ./parameters.md
[01]: ./createArray.md
[02]: ./length.md
[03]: ./skip.md
[04]: ./first.md
[05]: ./last.md
