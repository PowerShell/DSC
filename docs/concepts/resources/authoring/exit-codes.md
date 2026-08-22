---
description: >-
  Considerations and guidance for defining exit codes for a resource.
ms.date: 08/15/2025
title: Defining exit codes for a DSC resource
---

# Defining exit codes for a DSC resource

DSC determines whether a resource operation executed successfully by checking the exit code for
the resource process:

- DSC interprets exit code `0` as a successful operation.
- DSC interprets any nonzero exit code as a failed operation.

Resources that don't define the `exitCodes` field in their manifest can only surface success or
failure to users. To provide a better user experience, you can define exit codes and their meaning
in the resource manifest.

## How DSC uses exit codes

When a resource returns a nonzero exit code for an operation, DSC marks the operation for that
resource as a failure and emits an error trace message indicating the failure to the user.

The error message text uses the following syntax:

```Syntax
Command: Resource '<executable>' [exit code <exit-code>] manifest description: <exit-code-description>
```

The text wrapped in angle brackets (`<>`) indicates placeholders that DSC replaces with information
from the resource:

- `<executable>` is the executable DSC called to invoke the operation for the resource, defined by
  the `<operation>.executable` field in the manifest, like `get.executable`.
- `<exit-code>` is the exit code returned by the resource for the operation, like `1`.
- `<exit-code-description>` is the descriptive text for that exit code defined in the resource manifest.

For example, given the following manifest snippet:

```jsonc
{
  // Ellided manifest fields
  "get": {
    "executable": "example-resource"
  },
  "exitCodes": {
    "0": "success",
    "1": "failure",
    "70": "Invalid data"
  }
}
```

If the resource returns exit code `70` for the `get` operation, DSC emits the following error
message:

```console
Command: Resource 'example-resource' [exit code 70] manifest description: Invalid data
```

For more information about trace messaging in DSC and how a resource can participate in messaging,
see [Emitting messages from a DSC resource](./emitting-messages.md).

## Minimal exit codes

The smallest definition of semantic exit codes for a resource is to map `0` to success and `1` to
failure. The snippet below shows the minimal definition in a resource manifest:

```json
{
  "exitCodes": {
    "0": "success",
    "1": "unknown failure",
  }
}
```

However, this definition doesn't actually provide helpful context to a user - nonzero exit codes
_definitionally_ represent failures for DSC resources.

The rest of this document provides guidance for defining useful resource exit codes.

## Defining useful exit codes

When you implement a resource, follow these guidelines:

- Define exit code `0` to indicate a successful operation.
- Define exit code `1` to indicate an unknown or otherwise unhandled failure.
- Define an exit code for every fatal error you explicitly handle in your resource implementation.
- When defining exit codes for a resource that supports Linux or Unix, limit your exit codes to
  using `0` through `255`. Consider only defining custom exit codes other than `0` and `1` in the
  range `64` through `113`.

  For more information, see
  ["Exit Status" in the GNU Bash Reference Manual](https://www.gnu.org/software/bash/manual/html_node/Exit-Status.html)
  and
  ["Appendix E. Exit Codes With Special Meanings" in the Advanced Bash-Scripting Guide](https://tldp.org/LDP/abs/html/exitcodes.html).
- Use a coherent model for how you define your exit codes. For example, you might determine that
  you want to use exit codes `10` through `19` for API-related failures and exit codes `20`
  through `29` for validation errors.

  If you don't have enough failure causes to define a coherent model, add new exit codes by
  incrementing the last-defined exit code by 1. If the last defined exit code was `4`, the next
  exit code you define should be `5`.
- Avoid renumbering your exit codes as much as possible. Any users or integrating developers
  relying on your exit codes will need to update their scripts or other integrations whenever you
  change the semantics of an existing exit code.
- If your resource is wrapping an API or external command that returns exit codes, consider
  defining your exit codes with the same numbers and ensure the description matches the underlying
  API.
- Keep your exit code descriptions relatively short - no more than 80 characters. You should provide
  additional context for users by
  [emitting error messages](./emitting-messages.md#emitting-error-messages) from your resource
  before exiting.

## Examples

The following examples show how you can define exit codes for a resource.

### Example 1: Incrementing exit codes

This example cross-platform resource starts by defining the basic exit codes for the resource:

```jsonc
{
  // ellided manifest definition
  "exitCodes": {
    "0": "success",
    "1": "unknown failure"
  }
}
```

When the resource developer adds handling for a data validation failure, they define the new exit
code as `64`, the first recommended custom exit code for a resource that supports Linux and Unix
operating systems:

```jsonc
{
  // ellided manifest definition
  "exitCodes": {
    "0": "success",
    "1": "unknown failure",
    "64": "invalid data"
  }
}
```

The next exit code the resource developer defines for this resource should use exit code `65`.

### Example 2: Semantic exit code ranges

For this example resource, the developer has defined ranges for the resource operation exit codes:

- `70` through `79` for validation errors, where `70` is a catch-all for validation errors that
  don't have their own exit code.
- `80` through `89` for authentication errors, where `80` is a catch-all for authentication errors
  that don't have their own exit code.
- `90` through `99` for operational errors, where `90` is a catch-all for operational errors that
  don't have their own exit code.

```jsonc
{
  // ellided manifest definition
  "exitCodes": {
    "0": "success",
    "1": "unknown failure",
    "70": "data validation failure",
    "80": "authentication failure",
    "90": "operational failure"
  }
}
```

When the resource adds handling for a specific data validation failure, the manifest would define
the exit code for that failure as `71`.

## Related content

- [Designing a DSC resource](./index.md)
- [Emitting messages from a DSC resource](./emitting-messages.md)
