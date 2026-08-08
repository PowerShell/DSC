---
description: >-
  Overview of the process and considerations for designing a DSC resource.
ms.date: 03/25/2025
title: Designing a DSC resource
---

# Designing a DSC resource

<!-- Introduction -->

## Choosing a resource kind

When you decide to implement a DSC resource, you need to determine what _kind_ of resource you're
developing. DSC supports several different kinds of resource:

- Typical resources manage the state of a configurable component. Most resources are typical
  resources.
- Adapter resources make noncommand resources available to DSC to enable resource authors to
  develop resources without defining a manifest and executable.
- Exporter resources enable recursive export operations to help users quickly export the current
  configuration of a system without having to know every available resource.

For specific guidance on authoring resources, see:

- [Authoring a typical DSC resource manifest](./typical/resource-manifest.md) and
  [Authoring a typical DSC resource instance JSON Schema](./typical/resource-instance-schema.md)
- [Authoring a DSC exporter resource manifest](./exporter/resource-manifest.md)

## Defining resource metadata

DSC relies on metadata defined in the resource manifest to identify and describe each resource. At
a minimum, resources must define their fully qualified type name and version. When you're creating
resources for shared usage or publishing the resource publicly, there are other useful metadata
fields to define, like the resource description and tags.

For more information, see [Defining DSC resource manifest metadata](./manifest-metadata.md).

## Defining exit codes

DSC determines whether a resource operation executed successfully by checking the exit code for
the resource process:

- DSC interprets exit code `0` as a successful operation.
- DSC interprets any nonzero exit code as a failed operation.

Resources that don't define the `exitCodes` field in their manifest can only surface success or
failure to users. To provide a better user experience, you can define exit codes and their meaning
in the resource manifest.

For more guidance on defining exit codes, see
[Defining exit codes for a DSC resource](./exit-codes.md).

## Resource messaging

DSC resources can emit messages to stderr that DSC consumes and surfaces to users. Resource authors
can emit these messages to provide context and helpful information to users.

## Related content

- [Defining DSC resource manifest metadata](./manifest-metadata.md)
- [Defining DSC resource operation invocations](./operation-invocations.md)
- [Defining exit codes for a DSC resource](./exit-codes.md)
- [Emitting messages from a DSC resource](./emitting-messages.md)
