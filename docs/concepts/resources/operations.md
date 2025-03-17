---
description: >-
  Describes the operations available for DSC resources and how they are used.
ms.date:     03/18/2025
ms.topic:    conceptual
title:       DSC operations
---

# DSC operations

DSC defines a set of operations that resources can implement. DSC invokes the operations to use
resources for different tasks. Not every resource implements every operation.

The general operations for managing system state are [Get](#get-operation),
[Test](#test-operation), and [Set](#set-operation).

The rest of this document describes the available resource operations.

## Get operation

The **Get** operation returns the actual state of a specific resource instance on the system.

This operation is only available for resources that have the [get capability](capabilities.md#get)

DSC invokes the **Test** operation when you use the following commands:

- `dsc resource get` to return the actual state for a resource instance.
- `dsc config get` to return the actual state for every instance in a configuration document.

## Test operation

The **Test** operation compares the actual state of a specific resource instance on the system to a specified desired state. The result indicates not only whether the instance is in the desired state but also _how_ the actual state differs from the desired state.

If a resource doesn't have the [test capability](capabilities.md#test), DSC synthetically tests the resource.

DSC invokes the **Test** operation when you use the following commands:

- `dsc resource test` to test the desired state of a specific resource instance.
- `dsc config test` to test the desired state of every instance in a configuration document.

## Set operation

The **Set** operation enforces the desired state of a resource instance on a system. The result indicates how the resource modified the system.

This operation is only available for resources with the [set capability](capabilities.md#set).

DSC invokes the **Set** operation when you use the following commands:

- `dsc resource set` to enforce the desired state of a specific resource instance.
- `dsc config get` to enforce the desired state defined by a configuration document.

## Delete operation

The **Delete** operation removes a resource instance from a system. The operation returns no output.

This operation is only available for resources with the [delete capability](capabilities.md#delete).

DSC invokes the **Delete** operation when you use the following commands:

- `dsc resource delete` to remove a specific resource instance.

## Export operation

The **Export** operation retrieves the actual state for every instance of the resource on a system. The result is a configuration document that includes the exported instances.

This operation is only available for resources with the [export capability](capabilities.md#export).

DSC invokes the **Export** operation when you use the following commands:

- `dsc resource export` to return a configuration document that enumerates the actual state for every instance of a specific resource.
- `dsc config export` to return a configuration document that enumerates the actual state for every instance in a configuration document.
- `dsc resource get` with the `--all` option to return the actual state for every instance of a specific resource as an array of **Get** operation results.

## List operation

The **List** operation retrieves the available adapted resources for a specific DSC adapter resource.

This operation is only available for adapter resources.

## Validate operation

The **Validate** operation indicates whether an instance of the resource is validly defined. Command resources use their resource instance schema for validation. Adapter resources implement the **Validate** operation to enable DSC to validate adapted resources, which may not have a defined JSON Schema.

DSC invokes the **Validate** operation on adapter resources when validating adapted resource instances in a configuration document or when you use the `dsc resource` commands to directly invoke an adapted resource.

## Resolve operation

The **Resolve** operation processes an importer resource instance to return a configuration document.

This operation is only available for importer resources.

## See also

- [DSC resource capabilities](capabilities.md)
- [DSC resource kinds](kinds.md)
- [DSC resource properties](properties.md)
