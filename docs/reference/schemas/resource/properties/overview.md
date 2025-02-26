---
description: >-
  Information about the list of canonical DSC Resource properties, including their purpose and how
  to add them to a resource's manifest.
ms.date:     02/28/2025
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

## _rebootRequested

The read-only `_rebootRequested` property indicates whether a resource instance requires a reboot
after a set operation. To use DSC's built-in reboot notification processing, resources must define
this property in their manifest.

For more information, see [DSC Resource _rebootRequested property schema][06].

[01]: exist.md
[02]: ../manifest/test.md
[03]: ../manifest/root.md
[04]: inDesiredState.md
[05]: purge.md
[06]: rebootRequested.md
