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

Resources implement a JSONRPC api:

```json
{
  "jsonrpc": "2.0",
  "method": "dsc_invoke",
  "params": {
    "operation": "<operation_name>",
    "resourceType": "<resource_type>",
    "properties": {
      "<property_name>": "<property_value>"
    }
  },
}
```

## Alternate Proposals and Considerations

<!--
    Include any alternate proposals and notes for the RFC in this section.
-->

## Related work items

- [Define JSONRPC contract for reusable resources #1782](https://github.com/PowerShell/DSC/issues/1782)
