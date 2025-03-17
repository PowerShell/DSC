---
description: >-
  Describes the components of a DSC command resource, how DSC uses them, and what's required
  of them.
ms.date: 03/18/2025
title:   Anatomy of a command-based DSC Resource
---

# Anatomy of a command-based DSC Resource

DSC Resources provide a standardized interface for managing the settings of a system. A resource
defines properties you can manage and implements the code needed to get an instance of the
resource.

DSC command resources are defined with at least two files:

1. A DSC resource manifest that tells DSC how to interact with the resource.
1. One or more executable files and their dependencies to manage instances of the resource.

## DSC resource manifests

DSC resource manifests are defined as JSON files. For DSC to recognize a JSON file as a manifest,
the file must meet the following criteria:

1. The file must be discoverable in the `PATH` environment variable.
1. The filename must end with `.dsc.resource.json`.

When DSC searches the local system for available command resources, it searches every folder in the
`PATH` for files that use the DSC resource manifest naming convention. DSC then parses each of
those discovered files and validates them against the [DSC Resource Manifest JSON schema][01].

If the JSON file validates against the schema, DSC can use the DSC Resource.

At a minimum, the manifest must define:

- The version of the DSC resource manifest JSON schema it's compatible with.
- The fully qualified name of the resource, like `Microsoft.Windows/Registry`. The fully qualified
  name syntax is `<owner>[.<group>][.<area>]/<name>`. The group and area components of the fully
  qualified name enable organizing resources into namespaces.
- How DSC can call the command to get the current state of a resource instance.
- A way to validate an instance. This can be one of the following:
  - A JSON schema that describes an instance
  - A command DSC must call to get the schema at runtime
  - A command to validate nested DSC Resources. This last option only applies to DSC group
    resources and DSC adapter resources.

The manifest may define:

- The kind of resource the manifest describes: `adapter`, `group`, `importer`, or `resource`.

  If the manifest doesn't define the resource kind, it defaults to `resource` and is interpreted as
  a typical resource that directly manages an instance.
- How DSC can call the command to test whether an instance is in the desired state.

  If the manifest doesn't define how to test an instance of the resource, DSC performs a synthetic
  test for resource instances. DSC's synthetic test always gets the actual state of an instance and
  does a strict case-sensitive comparison of the instance's properties to the desired state. If any
  of the properties aren't exactly the same as the defined desired state, DSC reports the instance
  as being non-compliant.
- How DSC can call the command to set an instance to the desired state.

  If the manifest doesn't define how to set an instance of the DSC Resource, DSC can't use the
  resource to enforce desired state.
- The meaning of the non-zero exit codes returned by the command.

  If the manifest doesn't define the meaning for exit codes, all non-zero exit codes are reported
  as a generic failure.
- How DSC can call the command to export a list of every instance of that resource on the machine.
- How DSC can call the command to delete a specific instance of the resource.
- Metadata about the resource, like its author and a short description.

The manifest doesn't need to specify the same executable file for every operation. The definition
for each operation is independent.

<!-- For more information on authoring DSC Resource manifests, see
[Authoring a DSC Resource Manifest][02]. -->

## DSC resource executables

Command resources always require an executable file for DSC to run. The manifest doesn't need to be
bundled with the executable. The executable can be any executable file, such as a binary
application or a shell script. A resource may use different executables for different operations.

For DSC to use an executable, it must be discoverable in the `PATH` environment variable. DSC calls
the executable once per operation, using the exit code returned by the executable to determine if
the command was successful. DSC treats exit code `0` as a success and all other exit codes as an
error.

<!-- For more information about error handling, see
[Handling errors in a DSC command resource][03]. -->

### Inputs

DSC sends input to command resources as one of the following:

- A JSON data blob over stdin.

  When DSC sends the input as JSON over stdin, the data blob is the JSON representation of an
  instance's desired state. This input option supports complex properties with nested objects.
- A JSON data blob as the value to a specific argument.

  When the resource defines a JSON input argument, DSC invokes the command with the defined
  argument and passes the JSON representation of an instance's desired state to that argument. This
  input option supports complex properties with nested objects.
- A set of argument flags and values.

  When DSC sends the input as arguments, it generates a pair of arguments for each of the specified
  properties. The first argument is the name of the property prefixed with `--`, such as
  `--duration`. The second argument is the property's value. The ordering of the argument pairs
  isn't guaranteed. This input method doesn't support complex properties.
- Environment variables.

  When DSC sends the input as environment variables, it defines an environment variable for each
  property of the resource instance and sets the value to the string representation of that
  property. These environment variables are only defined for the execution of the command for that
  specific operation. The environment variables don't affect any other processes.

Input handling is defined per-operation in the resource manifest. A resource can define differnt
input handling for the operations it supports.

### Outputs

The executable for a command resource must return JSON data to stdout when called by DSC. The
output encoding must be UTF-8. When the resource returns the state of an instance, DSC validates
the JSON data against the resource's instance schema.

For adapter resources, DSC expects the executable to pass through the instance states for the
resources it manages as either a single JSON array or as a series of [JSON Lines][04].

Command resources can report logging information to DSC by emitting JSON Lines to stderr.
Each log entry must be a JSON object that includes two keys:

1. The `message` key defines the human-readable string for the log entry.
1. The `level` key defines whether the message represents an `error`, a `warning`, or `information`.

DSC collects messages from resources and displays them in the results for a configuration
operation. When DSC invokes a resource directly outside of a configuration, it doesn't collect the
messages. Instead, DSC emits the messages to stderr.

<!-- For more information about logging from command-based DSC Resources, see
[Logging messages from a DSC command resources][05]. -->

## Related Content

<!-- - [Authoring a DSC Resource Manifest][02] -->
<!-- - [Write your first DSC Resource overview][06] -->
- [DSC Resource Manifest schema reference][01]

[01]: ../../reference/schemas/resource/manifest/root.md
<!-- [02]: authoring-a-manifest.md -->
<!-- [03]: handling-errors.md -->
[04]: https://jsonlines.org/
<!-- [05]: logging-messages.md -->
<!-- [06]: ../tutorials/first-resource/overview.md -->
