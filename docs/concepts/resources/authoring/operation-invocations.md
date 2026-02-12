---
description: >-
  Considerations and guidance for defining exit codes for a resource.
ms.date: 08/15/2025
title: Defining DSC resource operation invocations
---

# Defining DSC resource operation invocations

For non-adapted resources, DSC expects the resource manifest to indicate how DSC should invoke
operations for the resource. The model for invoking resource operations is the same across all
operations.

At a minimum, the resource needs to define the name of the executable command for DSC to invoke
with the `executable` field. If the command requires any arguments, specify them with the `args`
field.

DSC sends data to the command in three ways:

1. When `input` is `stdin`, DSC sends the data as a string representing the data as a compressed
   JSON object without spaces or newlines between the object properties.
1. When `input` is `env`, DSC sends the data as environment variables. It creates an environment
   variable for each property in the input data object, using the name and value of the property.
1. When the `args` array includes a JSON input argument definition, DSC sends the data as a string
   representing the data as a compressed JSON object to the specified argument.

If you don't define the `input` property and don't define a JSON input argument, DSC can't pass the
input JSON to the resource. You can only define one JSON input argument for a command.

To ensure DSC can send input to your resource, you must define the `input` property, one JSON input
argument in the `args` property array, or both.

## Defining the executable and basic arguments

The `executable` field defines the name of the command to run. The value must be the name of a
command discoverable in the system's `PATH` environment variable or the full path to the command. A
file extension is only required when the command isn't recognizable by the operating system as an
executable.

The `args` property defines the list of arguments to pass to the command. DSC passes the arguments
to the command in the order you define them.

For example, consider the following operation definition snippet:

```json
{
    "executable": "my-app",
    "args": [
        "config",
        "get",
        "my-resource",
        "--output-format",
        "jsonline"
    ]
}
```

To invoke this resource, DSC will run the following effective command:

```sh
my-app config get my-resource --output-format jsonline
```

In this example, the resource hasn't defined how DSC should pass input to the command and therefore
can't accept any input.

## Input handling

Most resource operations need to be able to accept input. Deciding which model to use depends on
which option works best for your own development context.

You can have DSC send input to your resource over stdin, as environment variables, or as an input
argument.

### Handling input from stdin

When you define the `input` field as `stdin`, DSC sends the input data for the operation to the
defined `executable` over stdin as a compressed JSON object. When you use this input option, your
resource is responsible for deserializing the input JSON object.

Consider the following operation definition snippet:

```json
{
    "executable": "my-resource",
    "args": ["config", "get"],
    "input": "stdin"
}
```

To invoke this example resource operation definition, DSC will run the following effective
commands:

- POSIX shell

  ```sh
  my-resource config get < $inputData
  ```

- PowerShell

  ```powershell
  $inputData | my-resource config get
  ```

### Handling input from environment variables

When you define the `input` field as `env`, DSC sends the input data for the operation to the
defined `executable` as one or more environment variables. This model enables you to author
resources without needing to handle parsing the input JSON but has some restrictions:

- Your resource properties can only be the following JSON types:

  - `string`
  - `boolean`
  - `integer`
  - `number`
  - `array` where every item is of type `string`, `integer`, or `number`.

  If the input value for a property is `null`, DSC ignores that property when defining the
  environment variables prior to invoking your resource.

  If the input value for a property is an object or an array containing any values that aren't a
  string, integer, or number, DSC raises an error.
- DSC creates an environment variable for every defined property in the input object.
- The created environment variable uses the same casing as the property name.
- The value for the created environment variable is always a string. Your resource is responsible
  for correctly interpreting non-string property values from their string representation.
- For array properties, the created environment variable separates each value with a comma. Your
  resource shouldn't accept string values that contain any commas. DSC doesn't provide any handling
  for escaping string values in an array that contain commas.
- DSC doesn't clear existing environment variables prior to invoking the command, so any
  environment variables already defined in the context where a user is invoking DSC will be
  inherited. Any existing environment variables with the same name are overwritten for the
  operation invocation, but not modified for the context where DSC itself is invoked.

The primary use case for this input model is to support authoring resources as POSIX shell scripts
without requiring dependencies for parsing JSON. Given the limitations for defining resources with
input as environment variables, you may want to use a different input handling model or require
users to have a JSON parsing tool (such as [jq](https://jqlang.org/)) available.

Consider the following definition snippet and input data:

```json
{
    "executable": "my-resource",
    "args": ["config", "get"],
    "input": "stdin"
}
```

```json
{
    "stringProperty": "foo",
    "booleanProperty": true,
    "integerProperty": 0,
    "numberProperty": 1.2,
    "arrayOfStringsProperty": ["a", "b", "c"],
    "arrayOfIntegersProperty": [1, 2, 3],
    "arrayOfNumbersProperty": [1.2, 2.3, 3.4],
    "arrayOfMixedTypesProperty": ["a", 1, 1.2],
    "arrayEmptyProperty": [],
    "nullProperty": null
}
```

To invoke this example resource operation definition, DSC will run the following effective commands
for the given input data:

- POSIX shell

  ```sh
  export stringProperty="foo"
  export booleanProperty="true"
  export integerProperty="0"
  export numberProperty="1.2"
  export arrayOfStringsProperty="a,b,c"
  export arrayOfIntegersProperty="1,2,3"
  export arrayOfNumbersProperty="1.2,2.3,3.4"
  export arrayOfMixedTypesProperty="a,1,1.2"
  export arrayEmptyProperty=""
  my-resource config get
  ```

- PowerShell

  ```powershell
  $env:stringProperty = 'foo'
  $env:booleanProperty = 'true'
  $env:integerProperty = '0'
  $env:numberProperty = '1.2'
  $env:arrayOfStringsProperty = 'a,b,c'
  $env:arrayOfIntegersProperty = '1,2,3'
  $env:arrayOfNumbersProperty = '1.2,2.3,3.4'
  $env:arrayOfMixedTypesProperty = 'a,1,1.2'
  $env:arrayEmptyProperty = ''
  my-resource config get
  ```

### Handling input from a JSON input argument

To support sending input for an operation as an argument, you can define a JSON input argument in
the `args` field. When you do, DSC passes the JSON input to the named argument when available. A
JSON input argument is defined as a JSON object with the following properties:

- `jsonInputArg` (required) - The argument to pass the JSON data to for the command, like `--input`.
- `mandatory` (optional) - Indicate whether DSC should always pass the argument to the command,
  even when there's no JSON input for the command. In that case, DSC passes an empty string to the
  JSON input argument.

You can only define one JSON input argument in the `args` field.

If you define a JSON input argument and an `input` kind for a command, DSC sends the JSON data both
ways:

- If you define `input` as `env` and a JSON input argument, DSC sets an environment variable for
  each property in the JSON input and passes the JSON input object as a string to the defined
  argument.
- If you define `input` as `stdin` and a JSON input argument, DSC passes the JSON input over stdin
  and as a string to the defined argument.
- If you define a JSON input argument without defining the `input` property, DSC only passes the
  JSON input as a string to the defined argument.

If you don't define the `input` field and don't define a JSON input argument, DSC can't pass the
input JSON to the resource. For any operation definition except `get` and `export`, this makes the
definition invalid.

Consider the following operation definition snippet:

```json
{
    "executable": "my-resource",
    "args": [
        "config",
        "get",
        { "jsonInputArg": "--input", "mandatory": true }
    ],
}
```

To invoke this example resource operation definition, DSC will run the following effective
commands:

- POSIX shell

  ```sh
  my-resource config get --input $inputData
  ```

- PowerShell

  ```powershell
  my-resource config get --input $inputData
  ```

## Related content

- [Designing a DSC resource](./index.md)
- [Command-based DSC Resource manifest schema reference](../../../reference/schemas/resource/manifest/root.md)
