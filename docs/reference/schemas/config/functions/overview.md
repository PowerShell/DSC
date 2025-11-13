---
description: Reference for available functions in a Desired State Configuration document.
ms.date:     02/28/2025
ms.topic:    reference
title:       DSC Configuration document functions reference
---

# DSC Configuration document functions reference

## Synopsis

Functions available within a configuration document for runtime processing.

## Description

DSC configuration documents support the use of functions that DSC processes at runtime to determine
values for the document. These functions enable you to define configurations that reuse values and
are easier to maintain.

For DSC to recognize a function, it must be placed within square brackets in a string. DSC
configuration document functions use the following syntax:

```Syntax
[<function-name>(<function-parameters>...)]
```

When using functions in YAML, you must specify the function with a string value that is wrapped in
double quotation marks or uses the [folded][01] or [literal][02] block syntax. When using the
folded or literal block syntaxes, always use the [block chomping indicator][03] (`-`) to trim
trailing line breaks and empty lines.

```yaml
# Double quoted syntax
<keyword>: "[<function-name>(<function-parameters>...)]"
# Folded block syntax
<keyword>: >-
  [<function-name>(<function-parameters>...)]
# Literal block syntax
<keyword>: |-
  [<function-name>(<function-parameters>...)]
```

You can nest functions, using the output of a nested function as a parameter value for an outer
function. DSC processes nested functions from the innermost function to outermost function.

```Syntax
[<outer-function-name>(<nested-function-name>(<nested-function-parameters>))]
```

It can be difficult to read long functions, especially when they're deeply nested. You can use
newlines to break long functions into a more readable format with the folded or literal block
syntax.

```yaml
# Multi-line folded block syntax
<keyword>: >-
  [<outer-function-name>(
    <nested-function-name>(
      <deeply-nested-function-name>(<deeply-nested-function-parameters>)
    )
  )]
# Multi-line literal block syntax
<keyword>: |-
  [<outer-function-name>(
    <nested-function-name>(
      <deeply-nested-function-name>(<deeply-nested-function-parameters>)
    )
  )]
```

When the output of a function is an array or object, you can access the properties of the object or
items in the array.

To access the property of an output object, follow the closing parenthesis of the function with a
period (`.`) and then the name of the property you want to access. You can also access the
properties of nested objects and arrays.

```yaml
# Accessing a top-level property syntax
<keyword>: "[<function-name>(<function-parameters>...).<property-name>]"
# Accessing a property nested in another object syntax
<keyword>: "[<function-name>(<function-parameters>...).<property-name>.<nested-property-name>]"
# Accessing a property nested in an array item syntax
<keyword>: "[<function-name>(<function-parameters>...)[<index>].<nested-property-name>]"
```

To access a specific item in an array output, follow the closing parenthesis of the function with
an opening square bracket (`[`), then the integer index of the item you want to access, and then a
closing square bracket (`]`). You can also access items in nested arrays.

```yaml
# Accessing a top-level array item syntax
<keyword>: "[<function-name>(<function-parameters>...)[<index>]]"
# Accessing an array item nested in a property syntax
<keyword>: "[<function-name>(<function-parameters>...).<property-name>[<nested-array-index>]]"
# Accessing an array item nested in another array syntax
<keyword>: "[<function-name>(<function-parameters>...)[<index>][nested-array-index]]"
```

## Examples

### Example 1 - Use a function with valid syntaxes

The following configuration document shows the three valid syntaxes for specifying a function in
a configuration document. In each resource instance, the `text` property is set to the output of
the [base64()][base64] function.

```yaml
# overview.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Double quoted syntax
    type: Microsoft.DSC.Debug/Echo
    properties:
      text: "[base64('ab')]"
  - name: Folded block syntax
    type: Microsoft.DSC.Debug/Echo
    properties:
      text: >-
        [base64('ab')]
  - name: Literal block syntax
    type: Microsoft.DSC.Debug/Echo
    properties:
      text: |-
        [base64('ab')]
```

```sh
dsc config get --file overview.example.1.dsc.config.yaml
```

```yaml
results:
- name: Double quoted syntax
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      text: YWI=
- name: Folded block syntax
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      text: YWI=
- name: Literal block syntax
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      text: YWI=
messages: []
hadErrors: false
```

### Example 2 - Concatenate two strings

The following configuration document sets the `text` property of the resource instance to the
output of the [concat()][concat] function, combining the strings `a` and `b` into `ab`.

```yaml
# overview.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo the concatenated strings 'a' and 'b'
    type: Microsoft.DSC.Debug/Echo
    properties:
      text: "[concat('a', 'b')]"
```

```sh
dsc config get --file overview.example.2.dsc.config.yaml
```

```yaml
results:
- name: Echo the concatenated strings 'a' and 'b'
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      text: ab
messages: []
hadErrors: false
```

### Example 3 - Using nested functions

The following configuration document shows how you can nest functions. The first two resource
instances use the output of the [concat()][concat] function as input to the [base64()][base64] function.
The third resource instance uses the output of the nested functions from the first two instances
as input to the `concat()` function. The last resource instance converts the output of the deeply
nested functions shown in the third instance to base64.

```yaml
# overview.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo the concatenated strings 'a' and 'b' as base64
    type: Microsoft.DSC.Debug/Echo
    properties:
      text: "[base64(concat('a', 'b'))]"
  - name: Echo the concatenated strings 'c' and 'd' as base64
    type: Microsoft.DSC.Debug/Echo
    properties:
      text: "[base64(concat('c', 'd'))]"
  - name: Echo the concatenated base64 of strings 'ab' and 'cd'
    type: Microsoft.DSC.Debug/Echo
    properties:
      text: "[concat(base64(concat('a', 'b')), base64(concat('c', 'd')))]"
  - name: Echo the concatenated base64 of strings 'ab' and 'cd' as base64
    type: Microsoft.DSC.Debug/Echo
    properties:
      # text: "[base64(concat(base64(concat('a', 'b')), base64(concat('c', 'd'))))]"
      text: >-
        [base64(
          concat(
            base64(concat('a', 'b')),
            base64(concat('c', 'd'))
          )
        )]
```

```sh
dsc config get --file overview.example.3.dsc.config.yaml
```

```yaml
results:
- name: Echo the concatenated strings 'a' and 'b' as base64
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      text: YWI=
- name: Echo the concatenated strings 'c' and 'd' as base64
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      text: Y2Q=
- name: Echo the concatenated base64 of strings 'ab' and 'cd'
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      text: YWI=Y2Q=
- name: Echo the concatenated base64 of strings 'ab' and 'cd' as base64
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      text: WVdJPVkyUT0=
messages: []
hadErrors: false
```

### Example 4 - Access object properties and array items

The following configuration documents show how you can access the properties of objects and items
in arrays. The example uses a shared parameter definition file to make it easier to reference a
single data source in each configuration document.

The parameters file defines two parameters:

- The `data` parameter is a complex object. The `message` property is a nested object. The
  `services` property is a nested array.
- The `list` parameter is a complex array. The third item in the array is an object. The fourth
  item in the array is a nested array.

```yaml
# overview.example.4.dsc.parameters.yaml
parameters:
  # Object parameter
  data:
    # Object property as string
    name:  Example 4
    # Object property as integer
    count: 1
    # Object property as nested object
    message:
      text:  Default message
      level: info
      context:
        location: DC01
    # Object property as array
    services:
      - web
      - database
      - application
  # Array parameter
  list:
    # Array item as string
    - first
    # Array item as integer
    - 2
    # array item as object
    - name:  third
      value: 3
    # Array item as nested array
    -
      - Nested first
      - Nested second
      - name: Nested third
```

The first configuration document defines an instance of the `Microsoft.DSC.Debug/Echo` resource to show how you
can access an object's properties in a configuration document.

```yaml
# overview.example.4.properties.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
# Minimal definition of the parameters
parameters:
  data: { type: object }
  list: { type: array }

resources:
  - name: Access the properties of an object
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        # Accessing output object
        data: "[parameters('data')]"
        # Accessing properties
        data.name:     "[parameters('data').name]"     # string
        data.count:    "[parameters('data').count]"    # integer
        data.message:  "[parameters('data').message]"  # nested object
        data.services: "[parameters('data').services]" # array
```

```sh
$params=overview.example.4.dsc.parameters.yaml
$config=overview.example.4.properties.dsc.config.yaml
dsc config --parameters-file $params get --file $config
```

```yaml
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.133791S
  name: Access the properties of an object
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        data:
          count: 1
          name: Example 4
          message:
            text: Default message
            level: info
            context:
              location: DC01
          services:
          - web
          - database
          - application
        data.name: Example 4
        data.count: 1
        data.message:
          text: Default message
          level: info
          context:
            location: DC01
        data.services:
        - web
        - database
        - application
```

The next configuration document shows how you can access nested object properties.

```yaml
# overview.example.4.nested.properties.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
# Minimal definition of the parameters
parameters:
  data: { type: object }
  list: { type: array }

resources:
  - name: Access the properties of a nested object
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        data.message.text:             "[parameters('data').message.text]"
        data.message.level:            "[parameters('data').message.level]"
        data.message.context:          "[parameters('data').message.context]"
        data.message.context.location: "[parameters('data').message.context.location]"
```

```sh
$params=overview.example.4.dsc.parameters.yaml
$config=overview.example.4.nested.properties.dsc.config.yaml
dsc config --parameters-file $params get --file $config
```

```yaml
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.0760186S
  name: Access the properties of an object
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        data.message.text: Default message
        data.message.level: info
        data.message.context:
          location: DC01
        data.message.context.location: DC01
```

The following configuration document shows how you can access items in an array.

```yaml
# overview.example.4.items.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
# Minimal definition of the parameters
parameters:
  data: { type: object }
  list: { type: array }

resources:
  - name: Access items in an array
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        # Accessing output array
        list: "[parameters('list')]"
        # Accessing array items
        list[0]: "[parameters('list')[0]]" # string
        list[1]: "[parameters('list')[1]]" # integer
        list[2]: "[parameters('list')[2]]" # object
        list[3]: "[parameters('list')[3]]" # nested array
```

```sh
$params=overview.example.4.dsc.parameters.yaml
$config=overview.example.4.items.dsc.config.yaml
dsc config --parameters-file $params get --path $config
```

```yaml
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.0750682S
  name: Access items in an array
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        list:
        - first
        - 2
        - name: third
          value: 3
        - - Nested first
          - Nested second
          - name: Nested third
        list[0]: first
        list[1]: 2
        list[2]:
          name: third
          value: 3
        list[3]:
        - Nested first
        - Nested second
        - name: Nested third
```

The following configuration document shows how you can access items in a nested array.

```yaml
# overview.example.4.nested.items.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
# Minimal definition of the parameters
parameters:
  data: { type: object }
  list: { type: array }

resources:
  - name: Access items in a nested array
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        list[3][0]: "[parameters('list')[3][0]]"
        list[3][1]: "[parameters('list')[3][1]]"
        list[3][2]: "[parameters('list')[3][2]]"
```

```sh
$params=overview.example.4.dsc.parameters.yaml
$config=overview.example.4.nested.items.dsc.config.yaml
dsc config --parameters-file $params get --file $config
```

```yaml
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.1349442S
  name: Access items in a nested array
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        list[3][0]: Nested first
        list[3][1]: Nested second
        list[3][2]:
          name: Nested third
```

The last configuration document shows how you can use the property and item access syntaxes
together to access values in complex objects.

```yaml
# overview.example.4.mixed.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
# Minimal definition of the parameters
parameters:
  data: { type: object }
  list: { type: array }

resources:
  - name: Access values in complex objects and arrays
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        # Accessing array items of an object property
        data.services[0]: "[parameters('data').services[0]]"
        data.services[1]: "[parameters('data').services[1]]"
        data.services[2]: "[parameters('data').services[2]]"
        # Accessing properties of an object in an array
        list[2].name:  "[parameters('list')[2].name]"
        list[2].value: "[parameters('list')[2].value]"
        # Accessing the property of an object in a nested array
        list[3][2].name: "[parameters('list')[3][2].name]"
```

```sh
$params=overview.example.4.dsc.parameters.yaml
$config=overview.example.4.mixed.dsc.config.yaml
dsc config --parameters-file $params get --file $config
```

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
# Minimal definition of the parameters
parameters:
  data: { type: object }
  list: { type: array }

resources:
  - name: Access values in complex objects and arrays
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        # Accessing array items of an object property
        data.services[0]: "[parameters('data').services[0]]"
        data.services[1]: "[parameters('data').services[1]]"
        data.services[2]: "[parameters('data').services[2]]"
        # Accessing properties of an object in an array
        list[2].name:  "[parameters('list')[2].name]"
        list[2].value: "[parameters('list')[2].value]"
        # Accessing the property of an object in a nested array
        list[3][2].name: "[parameters('list')[3][2].name]"
```

## Functions

The following sections include the available DSC configuration functions by purpose and input type.

### Array and collection functions

The following list of functions operate on arrays and collections:

- [array()][array] - Convert a value into an array containing that value.
- [concat()][concat] - Combine multiple arrays of strings into a single array of strings.
- [contains()][contains] - Check if an array contains a value or an object contains a key.
- [createArray()][createArray] - Create an array of a given type from zero or more values of the
  same type.
- [empty()][empty] - Check if a value (string, array, or object) is empty.
- [first()][first] - Return the first element of an array or the first character of a string.
- [indexOf()][indexOf] - Return the zero-based index of the first occurrence of a value in an array.
- [intersection()][intersection] - Return a single array or object with the common elements from the parameters.
- [items()][items] - Convert an object into an array of key-value pair objects.
- [join()][join] - Combine array elements into a single string with a specified delimiter.
- [last()][last] - Return the last element of an array or the last character of a string.
- [lastIndexOf()][lastIndexOf] - Return the zero-based index of the last occurrence of a value in an array.
- [length()][length] - Return the number of elements in an array, characters in a string, or top-level properties in an object.
- [max()][max] - Return the largest integer value from an array of integers.
- [min()][min] - Return the smallest integer value from an array of integers.
- [range()][range] - Create an array of integers within a specified range.
- [skip()][skip] - Return an array or string with elements skipped from the beginning.
- [take()][take] - Return an array or string with the specified number of elements from the start.
- [tryGet()][tryGet] - Safely retrieve a value from an array by index or an object by key without throwing an error.
- [tryIndexFromEnd()][tryIndexFromEnd] - Safely retrieve a value from an array by counting backward from the end.
- [union()][union] - Return a single array or object with all unique elements from the parameters.

### Comparison functions

The following list of functions compare values:

- [equals()][equals] - Check if two values are equal.
- [greater()][greater] - Check if the first value is greater than the second value.
- [greaterOrEquals()][greaterOrEquals] - Check if the first value is greater than or equal to the second value.
- [less()][less] - Check if the first value is less than the second value.
- [lessOrEquals()][lessOrEquals] - Check if the first value is less than or equal to the second value.

### Data functions

The following list of functions operate on data outside of a resource instance:

- [context()][context] - Return contextual information about the system and execution environment.
- [envvar()][envvar] - Return the value of a specified environment variable.
- [parameters()][parameters] - Return the value of a specified configuration parameter.
- [secret()][secret] - Retrieve a secret value from a secure store.
- [variables()][variables] - Return the value of a specified configuration variable.

### Logical functions

The following list of functions perform logical operations:

- [and()][and] - Return true if all boolean values are true.
- [bool()][bool] - Convert a value to a boolean.
- [false()][false] - Return the boolean value false.
- [if()][if] - Return one of two values based on a boolean condition.
- [not()][not] - Return the logical negation of a boolean value.
- [null()][null] - Return a null value.
- [or()][or] - Return true if any boolean value is true.
- [true()][true] - Return the boolean value true.

### Mathematics functions

The following list of functions operate on integer values or arrays of integer values:

- [add()][add] - Return the sum of two integers.
- [div()][div] - Return the dividend of two integers as an integer, dropping the remainder of the
  result, if any.
- [int()][int] - Convert a string or number with a fractional part into an integer.
- [max()][max] - Return the largest value from an array of integers.
- [min()][min] - Return the smallest value from an array of integers.
- [mod()][mod] - Return the remainder from the division of two integers.
- [mul()][mul] - Return the product from multiplying two integers.
- [sub()][sub] - Return the difference from subtracting one integer from another.

### Object functions

The following list of functions operate on objects:

- [coalesce()][coalesce] - Return the first non-null value from the provided arguments.
- [contains()][contains] - Check if an array contains a value or an object contains a key.
- [createObject()][createObject] - Create an object from key-value pairs.
- [empty()][empty] - Check if a value (string, array, or object) is empty.
- [items()][items] - Convert an object into an array of key-value pair objects.
- [json()][json] - Parse a JSON string and return the resulting value.
- [tryGet()][tryGet] - Safely retrieve a value from an array by index or an object by key without throwing an error.

### Resource functions

The following list of functions operate on resource instances:

- [copyIndex()][copyIndex] - Return the current iteration index of a copy loop.
- [reference()][reference] - Return the result data for another resource instance.
- [resourceId()][resourceId] - Return the ID of another resource instance to reference or depend
  on.

### String functions

The following list of functions are for manipulating strings:

- [base64()][base64] - Return the base64 representation of a string.
- [base64ToString()][base64ToString] - Decode a base64-encoded string and return the original string.
- [concat()][concat] - Return a combined string where the input strings are concatenated in the
  order they're specified.
- [contains()][contains] - Check if an array contains a value or an object contains a key.
- [empty()][empty] - Check if a value (string, array, or object) is empty.
- [endsWith()][endsWith] - Check if a string ends with a specified suffix.
- [first()][first] - Return the first element of an array or the first character of a string.
- [format()][format] - Create a formatted string from input values.
- [join()][join] - Combine array elements into a single string with a specified delimiter.
- [last()][last] - Return the last element of an array or the last character of a string.
- [length()][length] - Return the number of elements in an array, characters in a string, or top-level properties in an object.
- [skip()][skip] - Return an array or string with elements skipped from the beginning.
- [startsWith()][startsWith] - Check if a string starts with a specified prefix.
- [take()][take] - Return an array or string with the specified number of elements from the start.
- [string()][string] - Convert a value to its string representation.
- [substring()][substring] - Extract a portion of a string starting at a specified position.
- [toLower()][toLower] - Convert a string to lowercase.
- [toUpper()][toUpper] - Convert a string to uppercase.
- [trim()][trim] - Remove leading and trailing whitespace from a string.
- [uniqueString()][uniqueString] - Create a deterministic hash string based on provided values.
- [uri()][uri] - Create an absolute URI by combining a base URI with a relative URI string.
- [uriComponent()][uriComponent] - Encode a string for use as a URI component.
- [uriComponentToString()][uriComponentToString] - Decode a URI-encoded string.

### System functions

The following list of functions provide system-level information:

- [path()][path] - Construct a file system path from one or more path segments.
- [systemRoot()][systemRoot] - Return the system root directory path.
- [utcNow()][utcNow] - Return the current UTC datetime in a specified format.

### Type functions

The following list of functions create or convert values of a given type:

- [array()][array] - Convert a value into an array containing that value.
- [bool()][bool] - Convert a value to a boolean.
- [createArray()][createArray] - Create an array of a given type from zero or more values of the
  same type.
- [createObject()][createObject] - Create an object from key-value pairs.
- [int()][int] - Convert a string or number with a fractional part into an integer.
- [string()][string] - Convert a value to its string representation.

<!-- Link references -->
[01]: https://yaml.org/spec/1.2.2/#folded-style
[02]: https://yaml.org/spec/1.2.2/#literal-style
[03]: https://yaml.org/spec/1.2.2/#block-chomping-indicator
<!-- Function link references -->
[add]:                  ./add.md
[and]:                  ./and.md
[array]:                ./array.md
[base64]:               ./base64.md
[base64ToString]:       ./base64ToString.md
[bool]:                 ./bool.md
[coalesce]:             ./coalesce.md
[concat]:               ./concat.md
[contains]:             ./contains.md
[context]:              ./context.md
[copyIndex]:            ./copyIndex.md
[createArray]:          ./createArray.md
[createObject]:         ./createObject.md
[div]:                  ./div.md
[empty]:                ./empty.md
[endsWith]:             ./endsWith.md
[envvar]:               ./envvar.md
[equals]:               ./equals.md
[false]:                ./false.md
[first]:                ./first.md
[format]:               ./format.md
[greater]:              ./greater.md
[greaterOrEquals]:      ./greaterOrEquals.md
[if]:                   ./if.md
[indexOf]:              ./indexOf.md
[int]:                  ./int.md
[intersection]:         ./intersection.md
[items]:                ./items.md
[join]:                 ./join.md
[json]:                 ./json.md
[last]:                 ./last.md
[lastIndexOf]:          ./lastIndexOf.md
[length]:               ./length.md
[less]:                 ./less.md
[lessOrEquals]:         ./lessOrEquals.md
[max]:                  ./max.md
[min]:                  ./min.md
[mod]:                  ./mod.md
[mul]:                  ./mul.md
[not]:                  ./not.md
[null]:                 ./null.md
[or]:                   ./or.md
[parameters]:           ./parameters.md
[path]:                 ./path.md
[range]:                ./range.md
[reference]:            ./reference.md
[resourceId]:           ./resourceId.md
[secret]:               ./secret.md
[skip]:                 ./skip.md
[startsWith]:           ./startsWith.md
[string]:               ./string.md
[take]:                 ./take.md
[sub]:                  ./sub.md
[substring]:            ./substring.md
[systemRoot]:           ./systemRoot.md
[toLower]:              ./toLower.md
[toUpper]:              ./toUpper.md
[trim]:                 ./trim.md
[true]:                 ./true.md
[tryGet]:               ./tryGet.md
[tryIndexFromEnd]:      ./tryIndexFromEnd.md
[union]:                ./union.md
[uniqueString]:         ./uniqueString.md
[uri]:                  ./uri.md
[uriComponent]:         ./uriComponent.md
[uriComponentToString]: ./uriComponentToString.md
[utcNow]:               ./utcNow.md
[variables]:            ./variables.md
