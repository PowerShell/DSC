---
description: Reference for the 'lastIndexOf' DSC configuration document function
ms.date:     08/29/2025
ms.topic:    reference
title:       lastIndexOf
---

## Synopsis

Returns an integer for the index of the last occurrence of an item in an array.
If the item isn't present, returns -1.

## Syntax

```Syntax
lastIndexOf(arrayToSearch, itemToFind)
```

## Description

The `lastIndexOf()` function searches an array from the end to the beginning
and returns the zero-based index of the last matching element. String
comparisons are case-sensitive. If no match is found, `-1` is returned.

Supported `itemToFind` types:

- string (case-sensitive)
- number (integer)
- array (deep equality)
- object (deep equality)

## Examples

### Example 1 - Find the last rollout slot for a server role (strings)

Use `lastIndexOf()` to locate where a particular role (like a web node)
appears last in a planned rollout sequence. This is handy when you need to
schedule a final step (for example, draining traffic) after the last matching
node has been processed. This example uses [`createArray()`][02] to build the
list of nodes.

```yaml
# lastindexof.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Rollout Plan
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      lastWebIndex: "[lastIndexOf(createArray('web01','db01','web02','cache01','web03'), 'web03')]"
      lastWebFamilyIndex: "[lastIndexOf(createArray('web01','db01','web02','cache01','web02'), 'web02')]"
```

```bash
dsc config get --file lastindexof.example.1.dsc.config.yaml
```

```yaml
results:
- name: Rollout Plan
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        lastWebIndex: 4
        lastWebFamilyIndex: 4
messages: []
hadErrors: false
```

Note that string comparison is case-sensitive. Searching for `WEB02` would
return `-1` in this example.

### Example 2 - Locate the last matching configuration object (objects)

Deep equality lets you search arrays of objects. Here we find the last
occurrence of a feature flag object with a specific name. This example uses
[`createObject()`][03] to build objects and [`createArray()`][10] to build the
collection.

```yaml
# lastindexof.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Feature Flags
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      lastBetaIndex: "[lastIndexOf(createArray(createObject('name','Beta'), createObject('name','Gamma'), createObject('name','Beta')), createObject('name','Beta'))]"
```

```bash
dsc config get --file lastindexof.example.2.dsc.config.yaml
```

```yaml
results:
- name: Feature Flags
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        lastBetaIndex: 2
messages: []
hadErrors: false
```

Property order in objects doesn't matter. The following also returns `1` due to
deep equality: `lastIndexOf(array(createObject('a',1,'b',2), createObject('b',2,'a',1)), createObject('a',1,'b',2))`.

## Parameters

### arrayToSearch

The array to search. Required.

```yaml
Type:     array
Required: true
Position: 1
```

### itemToFind

The item to search for. Required.

```yaml
Type:     string | number | array | object
Required: true
Position: 2
```

## Output

Returns a number representing the last index or -1 if not found.

```yaml
Type: number
```

## Related functions

- [`indexOf()`][00] - First occurrence index in an array
- [`contains()`][01] - Checks for presence in arrays/objects/strings

<!-- Link reference definitions -->
[00]: ./indexOf.md
[01]: ./contains.md
[02]: ./createArray.md
[03]: ./createObject.md
