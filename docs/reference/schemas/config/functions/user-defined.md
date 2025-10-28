---
description: Reference for user-defined functions in DSC configuration documents
ms.date:     09/26/2025
ms.topic:    reference
title:       User-defined functions
---

## Synopsis

Define and invoke custom functions within DSC configuration documents.

## Description

DSC supports user-defined functions that allow you to create reusable logic
within configuration documents. These functions can accept parameters, perform
computations or transformations, and return typed values that can be used
throughout your configuration.

User-defined functions provide several benefits:

- **Reusability**: Define logic once and use it multiple times throughout your
  configuration
- **Maintainability**: Centralize complex logic in named functions for easier
  updates
- **Type safety**: Functions enforce parameter and output types to prevent
  runtime errors
- **Isolation**: Functions run in their own execution context with limited
  access to global state

## Function definition

User-defined functions are defined at the top level of the configuration
document using the following structure:

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
functions:
- namespace: <namespace-name>
  members:
    <function-name>:
      parameters:
      - name: <parameter-name>
        type: <parameter-type>
      output:
        type: <output-type>
        value: <expression>
```

### Parameters

The `parameters` section defines the input parameters for the function:

- **name**: The name of the parameter, used to reference the parameter value
  within the function
- **type**: The expected data type for the parameter. Valid types are:
  - `string`: A string value
  - `secureString`: A secure string value (sensitive data)
  - `int`: An integer value
  - `bool`: A boolean value
  - `array`: An array of values
  - `object`: An object with properties
  - `secureObject`: A secure object (sensitive data)

### Namespaces

Functions are organized into namespaces to provide logical grouping and avoid
naming conflicts:

- **namespace**: A logical grouping name for related functions
- **members**: The collection of functions within the namespace

### Output

The `output` section defines the function's return value:

- **type**: The data type of the return value (same types as parameters)
- **value**: The DSC expression that defines the function's logic and return
  value

## Function invocation

User-defined functions are invoked using a namespace prefix followed by the
function name:

```yaml
property: "[NamespaceName.FunctionName(arg1, arg2, ...)]"
```

## Execution context

User-defined functions run in a restricted execution context:

- Functions can only access their own parameters, not global configuration
  parameters
- Functions cannot invoke other user-defined functions
- Functions cannot access variables or other configuration state

## Type validation

DSC performs type validation for user-defined functions:

- **Parameter validation**: All arguments passed to the function must match the
  expected parameter types
- **Output validation**: The function's return value must match the declared
  output type
- **Count validation**: The number of arguments must exactly match the number
  of defined parameters

## Examples

### Example 1 - Simple string transformation function

This example defines a function that formats a greeting message using
[`concat()`][04] to combine strings:

```yaml
# user-defined.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
functions:
- namespace: Greeting
  members:
    formatGreeting:
      parameters:
      - name: firstName
        type: string
      - name: lastName
        type: string
      output:
        type: string
        value: "[concat('Hello, ', parameters('firstName'), ' ', 
                 parameters('lastName'), '!')]"

resources:
- name: Echo formatted greeting
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[Greeting.formatGreeting('John', 'Doe')]"
```

```sh
dsc config get --file user-defined.example.1.dsc.config.yaml
```

```yaml
results:
- name: Echo formatted greeting
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Hello, John Doe!
messages: []
hadErrors: false
```

### Example 2 - Mathematical calculation function

This example defines a function that calculates the area of a rectangle using
[`mul()`][05] to multiply the dimensions:

```yaml
# user-defined.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
functions:
- namespace: Math
  members:
    calculateArea:
      parameters:
      - name: width
        type: int
      - name: height
        type: int
      output:
        type: int
        value: "[mul(parameters('width'), parameters('height'))]"

resources:
- name: Echo calculated area
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: 
      width: 10
      height: 5
      area: "[Math.calculateArea(10, 5)]"
```

```sh
dsc config get --file user-defined.example.2.dsc.config.yaml
```

```yaml
results:
- name: Echo calculated area
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        width: 10
        height: 5
        area: 50
messages: []
hadErrors: false
```

### Example 3 - Array processing function

This example defines a function that processes an array parameter using
[`concat()`][04] and [`join()`][06] to format the output. The example also
uses [`createArray()`][07] to create the input array:

```yaml
# user-defined.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
functions:
- namespace: Utility
  members:
    joinWithPrefix:
      parameters:
      - name: items
        type: array
      - name: prefix
        type: string
      output:
        type: string
        value: "[concat(parameters('prefix'), ': ', 
                 join(parameters('items'), ', '))]"

resources:
- name: Echo joined array
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[Utility.joinWithPrefix(createArray('apple', 'banana', 'cherry'), 
              'Fruits')]"
```

```sh
dsc config get --file user-defined.example.3.dsc.config.yaml
```

```yaml
results:
- name: Echo joined array
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: "Fruits: apple, banana, cherry"
messages: []
hadErrors: false
```

## Error handling

User-defined functions can encounter several types of errors:

### Unknown function error

Occurs when trying to invoke a function that is not defined:

```text
Unknown user function 'Namespace.FunctionName'
```

### Parameter validation error

Occurs when a function tries to access a parameter that doesn't exist:

```text
Parameter 'ParameterName' not found in context
```

### Output type mismatch

Occurs when the function's return value doesn't match the declared output type:

```text
Output of user function 'Namespace.FunctionName' did not return expected type 'expectedType'
```

### Restricted function access

Occurs when trying to use restricted built-in functions:

```text
The 'functionName()' function is not available in user-defined functions
```

## Limitations

User-defined functions have several important limitations:

1. **Restricted function access**: User functions cannot call certain built-in
   functions like `reference()`, `variables()`, and `utcNow()`
2. **No cross-namespace calls**: Functions cannot call user-defined functions
   from other namespaces
3. **No global state access**: Functions can only access their own parameters,
   not global configuration parameters
4. **No side effects**: Functions should be pure and not modify external state
5. **Type constraints**: All parameters and outputs must have explicitly
   declared types
6. **Expression-based**: Function logic must be expressed as a single DSC
   expression

## Best practices

When creating user-defined functions, follow these best practices:

1. **Use descriptive names**: Choose function names that clearly describe their
   purpose
2. **Keep functions simple**: Functions should have a single, well-defined
   responsibility
3. **Validate input types**: Always specify appropriate parameter types to
   prevent runtime errors
4. **Document your functions**: Use comments to explain complex logic within
   function expressions
5. **Test thoroughly**: Verify functions work correctly with various input
   values
6. **Avoid complex nesting**: Keep function expressions readable by avoiding
   deeply nested function calls

## Related topics

- [DSC Configuration document schema][01]
- [DSC Configuration functions overview][02]
- [Built-in DSC functions][03]

<!-- Link reference definitions -->
[01]: ../document.md
[02]: ./overview.md
[03]: ./overview.md#functions
[04]: ./concat.md
[05]: ./mul.md
[06]: ./join.md
[07]: ./createArray.md
