---
description: >-
  Information about the list of canonical DSC Resource properties, including their purpose and how
  to add them to a resource's manifest.
ms.date:     09/01/2026
ms.topic:    reference
title:       DSC canonical properties
---

# DSC canonical properties

DSC has support for several canonical properties. Some canonical properties enable a DSC Resource
to use built-in processing. The canonical properties always start with an underscore (`_`) and DSC
Resources that use these properties may not override or extend them.

## _exist

The `_exist` property indicates that the resource can enforce whether instances exist, handling
whether an instance should be added, updated, or removed during a set operation. This property
provides shared semantics for DSC Resources and integrating tools.

For more information, see [DSC Resource _exist property schema][01].

## _inDesiredState

The read-only `_inDesiredState` property indicates whether a resource instance is in the desired
state. This property is mandatory for command-based DSC Resources that define the [test][02]
property in their [manifest][03].

For more information, see [DSC Resource _inDesiredState property schema][04].

## _purge

DSC Resources that need to distinguish between whether unmanaged entries in a list are valid or
must be removed can define the write-only `_purge` property. This property provides shared
semantics for DSC Resources and integrating tools, but doesn't enable any built-in processing with
DSC.

For more information, see [DSC Resource _purge property schema][05].

## _restartRequired

The read-only `_restartRequired` property indicates that the machine, specific services, or
specific processes need to be restarted after the resource enforces the desired state. When a
resource includes this property in the output of a set operation, DSC records the restart
requirements in the execution information for the operation and makes them available to the
[restartRequired()][06] configuration function.

The value of this property must be an array of objects. Each object defines exactly one of the
following properties:

- `system` - A string describing why the system needs to be restarted.
- `service` - The name of a service that needs to be restarted.
- `process` - An object with the `name` and `id` properties identifying a process that needs to be
  restarted.

This property replaces the `_rebootRequested` property, which earlier schemas defined but DSC
never processed.

<!-- Link reference definitions -->
[01]: exist.md
[02]: ../manifest/test.md
[03]: ../manifest/root.md
[04]: inDesiredState.md
[05]: purge.md
[06]: ../../config/functions/restartRequired.md
