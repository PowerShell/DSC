---
description: >-
  Considerations and guidance for emitting messages from a DSC resource.
ms.date: 08/15/2025
title: Emitting messages from a DSC resource
---

# Emitting messages from a DSC resource

DSC uses a messaging system to provide runtime information to users. By default, DSC emits messages
as strings to stderr with ANSI console coloring for the timestamp, message level, and line number.
You can use the [--trace-format](../../../reference/cli/index.md#--trace-format) option to have DSC
emit the messages to stderr without console coloring or as JSON Lines.

DSC supports consuming and surfacing messages from resources to provide context and helpful
information to users.

This document provides an overview of the messaging system and guidance for how a resource should
integrate with DSC to emit useful messages.

## DSC message levels

Messages in DSC are categorized by their level. The following list shows the valid message levels
from highest to lowest level.

- `error` - messages at this level indicate critical problems.
- `warn` (default) - messages at this level provide important information that may affect usage
  and behavior.
- `info` - messages at this level provide useful but less important information about operational
  behavior.
- `debug` - messages at this level surface verbose information primarily useful for troubleshooting
  and investigating.
- `trace` - messages at this level surface extremely verbose and low-level information for
  troubleshooting and investigating.

> [!WARNING]
> The `trace` level output emits all JSON input/output that DSC processes during execution. DSC
> doesn't sanitize the JSON before emitting it. This trace level is only intended for developer
> use. Never redirect `trace` level output to storage as it might contain sensitive information.

Users can configure the trace level for DSC by specifying the
[--trace-level](../../../reference/cli/index.md#--trace-level) command option, setting the
[DSC_TRACE_LEVEL](../../../reference/cli/index.md#environment-variables) environmental variable, or
by modifying the settings for DSC.

The trace level defines the minimum message type to emit when using DSC. Because the default trace
level is `warn`, DSC only emits error and warning messages by default. The following table shows
how the trace level determines which messages DSC emits:

| Trace level | Emitted message levels                    |
|:-----------:|:------------------------------------------|
|   `error`   | `error`                                   |
|   `warn`    | `error`, `warn`                           |
|   `info`    | `error`, `warn`, `info`                   |
|   `debug`   | `error`, `warn`, `info`, `debug`          |
|   `trace`   | `error`, `warn`, `info`, `debug`, `trace` |

## DSC message output

By default, DSC emits trace messages to stderr with ANSI console coloring.

When a user specifies the [--trace-format](../../../reference/cli/index.md#--trace-format) option
as `plaintext`, DSC emits the messages in the same format but without console colors.

When a user specifies the format as `json`, DSC emits each message as a JSON Line to stderr.

When the [--trace-level](../../../reference/cli/index.md#--trace-level) option is set to `debug` or
`trace`, DSC includes additional information about where in _DSC's code_ the message was emitted.

### Console message output

When the trace level is set to `info`, `warn`, or `eror`, DSC emits messages from resources to
stderr with the following syntax:

```Syntax
<timestamp>  <level> PID <pid>: <resource-message>
```

- `<timestamp>` is the ISO8601 timestamp for when DSC emitted the message from the resource. It
  doesn't represent the timestamp for when the _resource emitted_ the message.
- `<level>` is the all-caps representation of the messaging level: `ERROR`, `WARN`, `INFO`,
  `DEBUG`, or `TRACE`.
- `<pid>` is the process ID for the process DSC spawned to invoke the resource.
- `<resource-message>` is the string value of the message emitted by the resource.

For example:

```console
2025-08-15T14:42:53.970726Z  WARN PID 29072: Message from resource
```

When the trace level is set to `debug` or `trace`, DSC emits messages from resources to stderr with
the following syntax:

```Syntax
<timestamp>  <level> <dsc-source-module>: <dsc-source-line>: PID <pid>: <resource-message>
```

- `<timestamp>` is the ISO8601 timestamp for when DSC emitted the message from the resource. It
  doesn't represent the timestamp for when the _resource emitted_ the message.
- `<level>` is the all-caps representation of the messaging level: `ERROR`, `WARN`, `INFO`,
  `DEBUG`, or `TRACE`.
- `<dsc-source-module>` indicates where in the DSC source code the message was emitted from, using
  Rust module path syntax.
- `<dsc-source-line>` indicates which line in the DSC source code the message was emitted from.
- `<pid>` is the process ID for the process DSC spawned to invoke the resource.
- `<resource-message>` is the string value of the message emitted by the resource.

For example:

```console
2025-08-15T14:57:02.218913Z  WARN dsc_lib::dscresources::command_resource: 901: PID 34460: Message from resource
```

### JSON message output

When the trace format is `json` and the trace level is set to `info`, `warn`, or `eror`, DSC emits
messages from resources to stderr as JSON Line objects with the following properties:

- `timestamp` - the ISO8601 timestamp for when DSC emitted the message from the resource. It
  doesn't represent the timestamp for when the _resource emitted_ the message.
- `level` - is the all-caps representation of the messaging level: `ERROR`, `WARN`, `INFO`,
  `DEBUG`, or `TRACE`.
- `fields` - is an object with the `message` string subproperty containing the message from the
  resource. The message is prefixed with `PID <pid>:` where `<pid>` is the process ID for the
  process DSC spawned to invoke the resource.

For example:

```json
{
  "timestamp": "2025-08-18T19:47:43.376148Z",
  "level": "WARN",
  "fields": {
    "message": "PID 1508: Message from resource"
  }
}
```

When the trace format is `json` and the trace level is set to `debug` or `trace`, DSC emits
messages from resources to stderr as JSON Line objects with the following properties:

- `timestamp` - the ISO8601 timestamp for when DSC emitted the message from the resource. It
  doesn't represent the timestamp for when the _resource emitted_ the message.
- `level` - is the all-caps representation of the messaging level: `ERROR`, `WARN`, `INFO`,
  `DEBUG`, or `TRACE`.
- `fields` - is an object with the `message` string subproperty containing the message from the
  resource. The message is prefixed with `PID <pid>:` where `<pid>` is the process ID for the
  process DSC spawned to invoke the resource.
- `target` - indicates where in the DSC source code the message was emitted from, using
  Rust module path syntax.
- `line_number` - indicates which line in the DSC source code the message was emitted from.

For example:

```json
{
  "timestamp": "2025-08-18T20:37:55.100565Z",
  "level": "WARN",
  "fields": {
    "message": "PID 16160: Message from resource"
  },
  "target": "dsc_lib::dscresources::command_resource",
  "line_number": 921
}
```

## How DSC consumes messages from resources

DSC processes lines of text emitted to stderr by a resource using the following procedure:

1. If the emitted line is JSON, DSC deserializes it and checks whether it defines a level property:

   - If the object defines the `error` property, DSC emits an error-level message, using the value
     of `error` as the message text.
   - If the object defines the `warn` property, DSC emits a warning-level message, using the value
     of `warn` as the message text.
   - If the object defines the `info` property, DSC emits an info-level message, using the value of
     `info` as the message text.
   - If the object defines the `debug` property, DSC emits a debug-level message, using the value
     of `debug` as the message text.
   - If the object defines the `trace` property, DSC emits a trace-level message, using the value
     of `trace` as the message text.

   If the object doesn't define any of the level properties, DSC emits a trace-level message, using
   the emitted line as the message text without unwrapping the JSON.
1. If DSC can't parse the line as JSON, DSC emits a trace-level message, using the line as the
   message text.

DSC always prefixes the message text with the `PID <pid>:` prefix, where `<pid>` is the ID for the
process DSC spawned to invoke the resource.

### Emitting structured data

Currently, DSC only supports emitting string messages at the defined levels. DSC can't bubble up
additional structured data with the message due to limitations in the tracing library DSC uses.
Instead, to send structured data, use the following convention:

1. Create a JSON object representing your data:

  ```json
  {
    "key1": true,
    "key2": "key2 value"
  }
  ```

1. Convert the JSON to an escaped string representation of the data as compressed JSON:

  ```json
  "{\"key1\":true,\"key2\":\"key2 value\"}"
  ```

1. Emit a message with the appropriate level as a JSON Line:

  ```json
  {"info":"{\"key1\":true,\"key2\":\"key2 value\"}"}
  ```

This convention requires the caller to unwrap the emitted data from the trace that DSC emits.

## Emitting error messages

By default, DSC recognizes failures for resource operations by inspecting the exit code your
resource returns for an operation invocation. If your manifest defines useful
[exit codes](./exit-codes.md), DSC can provide a human-readable synopsis for why the operation
failed.

Resources should emit error messages prior to returning a nonzero exit code to provide more
detailed information that a user can reference for troubleshooting.

To emit a `error` level message from your resource, emit a JSON Line to stderr with a single
property, `error`, where the value is a string representing the message you want to emit.

For example:

```json
{"error":"Message contents"}
```

## Emitting warning messages

Resources should emit warning messages to indicate non-critical issues and information about the
resource's behavior to the user.

Consider emitting warning messages under the following guidelines:

- Emit a warning message when your resource is aware of an important concern that won't cause the
  execution of the current operation to fail but may affect other operations.
- Emit a warning message when your resource needs to inform a user about effects of an operation
  that aren't obvious or expected by default.

To emit a `warn` level message from your resource, emit a JSON Line to stderr with a single
property, `warn`, where the value is a string representing the message you want to emit.

For example:

```json
{"warn":"Message contents"}
```

## Emitting info messages

Resources should emit info messages to provide non-critical information about the resource's
behavior to the user.

Consider emitting info messages under the following guidelines:

- Emit an info message when you want to surface context or information to a user about the behavior
  of the resource.
- Don't emit info messages for low-level details about the internals of your resource.

To emit a `info` level message from your resource, emit a JSON Line to stderr with a single
property, `info`, where the value is a string representing the message you want to emit.

For example:

```json
{"info":"Message contents"}
```

## Emitting debug messages

Resources can emit debug messages to help users and integrating developers troubleshoot resource
behavior.

Consider emitting debug messages under the following guidelines:

- Emit a debug message when you want to provide context for troubleshooting the resource.
- Emit debug messages consistently. If you emit a debug message indicating the start of an internal
  function or loop, also emit a debug message when that internal operation ends.

To emit a `debug` level message from your resource, emit a JSON Line to stderr with a single
property, `debug`, where the value is a string representing the message you want to emit.

For example:

```json
{"debug":"Message contents"}
```

## Emitting trace messages

Generally, resources shouldn't emit trace messages. If you need to provide more detailed
information than implied by debug messages, emit that information as trace messages.

Consider emitting trace messages under the following guidelines:

- Emit trace messages to provide extremely low-level information about the internal processing of
  the resource.
- Emit trace messages for maintainers and integrating developers to use when investigating and
  troubleshooting resource behavior.

To emit a `trace` level message from your resource, emit a JSON Line to stderr with a single
property, `trace`, where the value is a string representing the message you want to emit.

For example:

```json
{"trace":"Message contents"}
```

## Related content

- [Designing a DSC resource](./index.md)
- [Defining exit codes for a DSC resource](./exit-codes.md)
