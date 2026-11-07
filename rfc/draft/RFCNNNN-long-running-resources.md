---
RFC:          RFCNNNN # WG will set the number after submission
Author:       @SteveL-MSFT
Sponsor:      null    # <@GitHubUserName>
Status:       Draft   # <Draft | Experimental | Accepted | Final>
SupercededBy: null    # <Superceding RFC Number>
Version:      1.0     # <Major>.<Minor>
Area:         Resources
CommentsDue:  December 10, 2026
---

# Long Running DSC Resources

## Motivation

> As a DSC Configuration author,
> I want deploy a configuration that uses the same resource many times,
> so that the deployment time is reduced.

Currently, if a large configuraiton file uses the same resource multiple times,
each invocation of the resource spawns a new process, leading to increased deployment time and resource consumption.

Although we introduced the pattern of a List resource accepting an array of items to process in a single resource invocation,
it can make the configuration more complex to understand and maintain.

## Proposed experience

For configuration authors, there may be a need for process isolation so a new ResourceDirective will be added to opt-out
of reusing a resource that supports long running operations.

For resource authors, they will need to implement a JSONRPC interface and declare in their resource manifest that the resource supports long running operations.

## Specification

### Configuration

At the resource scope, a new directive `ReuseResource: <bool>` will be introduced to indicate whether the resource should be reused for multiple invocations within the same configuration.
By default, `ReuseResource` will be set to `true`, indicating that the resource should be reused for multiple invocations within the same configuration unless explicitly opted out.

If a resource only implements the JSONRPC interface, then if this directive is used and set to `false`, a new instance of the resource will be created for each invocation, ensuring process isolation.

### Resource Manifest

A new `jsonrpc` field will be added as a top-level field in the resource manifest to indicate that the resource supports long running operations.

```yaml
type: Microsoft.Windows/Registry
jsonrpc:
  version: "2.0"
  invoke: true
```

The existing fields per operation such as `get` which also declare requirements like `requireSecurityContext` will be respected via JSONRPC.
Only JSONRPC version 2.0 is supported.

Resources implement a JSONRPC APIs:

At the start, the engine will send an `initialize` request to the resource to allow it to perform any necessary setup before handling subsequent `dsc_invoke` requests.
This API is optional.

```json
{
  "jsonrpc": "2.0",
  "method": "initialize"
}
```

The `dsc_invoke` API is used to perform operations on the resource. It requires the operation name, resource type, context, and properties as parameters.
The `context` is needed to provide additional information to adapters.

```json
{
  "jsonrpc": "2.0",
  "method": "dsc_invoke",
  "params": {
    "operation": "<operation_name>",
    "resourceType": "<resource_type>",
    "context": {
        "targetResource": "<target_resource>"
    },
    "properties": {
      "<property_name>": "<property_value>"
    }
  },
}
```

Before the process is terminated, the engine will send a `shutdown` request to the resource to allow it to perform any necessary cleanup.
This API is also optional.

```json
{
  "jsonrpc": "2.0",
  "method": "shutdown"
}
```

The engine will allow for a timeout of 500ms for the `initialize` and `shutdown` requests.

> Open Questions:
>
> - Should the timeout be configurable by the resource in the manifest?  Seems to be hard for the resource to know what type of system it is running on

### Engine Behavior

The DSC engine will instantiate a new process for the resource as it does today on first invocation.
If the `ReuseResource` directive is set to `true` (or not specified), the engine will keep the resource process alive and reuse it for subsequent invocations within the same configuration.
If the `ReuseResource` directive is set to `false`, the engine will terminate the resource process after each invocation, ensuring process isolation.

At the end of the configuration, the engine will terminate any remaining resource processes that were kept alive for reuse.

## Alternate Proposals and Considerations

<!--
    Include any alternate proposals and notes for the RFC in this section.
-->

## Related work items

- [Define JSONRPC contract for reusable resources #1782](https://github.com/PowerShell/DSC/issues/1782)
