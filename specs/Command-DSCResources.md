# Command based resources for DSC v3

Configuration is based on a declarative idempotent model that allows users to get, set, and test settings values.
Configuration relies on resources to perform the actual domain specific work.
To enable customers and partners to more easily participate in authoring resources,
we need a simplified model that is not tied to any specific programming language.

However, we still need to support the existing resources that are written in PowerShell including
class based and script functions based resources.

## Commands as resources

Resources are not required to be standalone executables,
but they must be able to execute as a command-line with optional arguments.
This enables resources to be written in python, PowerShell script, or could be a standalone executable written in
Go, Rust, etc...

Communication to and from the resource will be via STDIN or arguments for input and STDOUT for output along
with using the exit code to indicate success or failure and STDERR for informative messages.

> [Note] We should reserve a range of exit codes for standard errors and leave the rest for custom resource use

### Resource discovery

Any "command" (which can be a script requiring a host runtime) that particpates would have a command manifest file with the name
"<command>.dscresource.json" that would be found within the `PATH` environment variable.

PowerShell based resources can have an optional `.dscresource.json` file to indicate if the resource should be hosted by powershell.exe or pwsh.exe (see below).
This file would be discovered via the existing `PSModulePath` environment variable along with the resource module.

For PSDesiredStateConfiguration module, in addition to searching for PowerShell class resources,
it would also search through the `PATH` environment variable for files with the name "<command>.dscresource.json".
The contents of this file would have a `dscresource` section indicating that commands participates in configuration (defined in next section).

Example of getting the metadata for a resource:

```output
PS> Get-DscResource MyResource
PS> config list MyResource

ImplementationDetail : CommandBased
ResourceType         : MyResource
Name                 : MyResource
FriendlyName         :
Module               : 
ModuleName           : 
Version              : 0.0.1
Path                 : /usr/bin/MyResource.command.json
ParentPath           : /usr/bin
ImplementedAs        : Command
CompanyName          : Microsoft
Settings             : # this is renamed from `Properties`, but in PS we can alias it
SHA256Hash           : 
SignerCertThumbprint :
exitCodes            : {}
requires             : {}
schema               : https://schemas.microsoft.com/configuration/myResource/20220621/schema.json
```

These members are carried over form existing `Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo` type with addition of
`SHA256Hash` and `SignerCertThumprint` optional members and `exitCodes`, `requires`, and `schema` are from the manifest file.

### Command manifest for configuration

The manifest file is a JSON file following specific schema (actual JSON schema file to be defined later).
The file name must end with `.dscresource.json` while the first part
An example "MyResource.dscresource.json":

```json
{
  "manifestVersion": "1.0",
  "dscresource": {
    "name": "Microsoft.DSC.MyResource",
    "version": "0.0.1-preview",
    "get": {
        "executable": "mycommand",
        "args": [ "getconfig" ],
        "inputViaStdin": true
    },
    "set": {
        "executable": "mycommand",
        "args": [ "setconfig" ],
        "inputAsArgs": true,
        "returnState": true
    },
    "test": {
        "caseSensitive": true
    },
    "requires": [
      {
        "resource": "name of dependency",
        "resourceType": "executable|OS",
        "version": "1.2.3"
      }
    ],
    "exitCodes": {
      "1": "Access Denied",
      "2": "Invalid setting value",
      "3": "Unknown setting"
    },
    "schema": "https://schemas.microsoft.com/configuration/myResource/20220621/schema.json"
  }
}
```

The resource `name` includes the namespace which is required to be unique.
The `version` allows for semver.

If `inputViaStdin` is `true`, then the input JSON will be sent to the command via STDIN.
This is preferred for resources that support nested JSON objects.

If `inputAsArgs` is `true`, then the input JSON will be deserialized into a list of arguments.
The top level members of the JSON become the argument names and the values become the argument values.
Any nested object values or arrays will be sent as JSON argument values.
When this is used, the `config` command will need to be careful escaping the arguments correctly
to not expose a code injection vulnerability.
This would make it easy to implement simple resources as BASH scripts which can parse arguments
more easily than JSON.

The `returnState` for `set` is optional but indicates that the resource will return the current state
of the settings.
Otherwise, the `config` command will execute a `get` upon successful `set` to get the current state.

Here, `mycommand` is an executable that is found within `PATH`.
Note that the executable name does not need to match the command name as part of the manifest file name.
This means that a single executable can implement multiple resources where each resource
would have its own manifest file.
The `args` member allows passing an arbitrary number of arguments to the executable for get, set, or test operations.
This allows the executable to be "python3", for example, and the args point to a specific python script with
other arguments as needed by the script if it implements get, set, or test within the same script.
Alternatively, the args could point to different scripts that implement get, set, or test.

In the example above, doing:

```powershell
Invoke-DscResource -Method Get -Name MyResource
```

would execute: "mycommand getconfig"

`set` and `test` are optional.
If `test` is not implemented, PSDesiredStateConfiguration will perform it's own comparison with desired state
with current state by performing a `get` operation.
In the example above, because there is no `executable` specified, then it does not implement `test` and relies on
higher level tooling to provide that operation.
The `caseSensitive` member defaults to `false` if not specified and is used for higher level tooling to determine
when comparing the input JSON with retrieved current JSON if values should be case sensitive or not.

In the case of of a resource that has properties whose status cannot be determined by simple equivalence, for example a resource that supports a version range,
then a simple comparison of desired configuration and current configuration JSON won't work.
For now, resources with properties that require any type of more complex check will need to implement `test` to perform it themselves.

> **Note**: JSON treats member names as case-sensitive.  Expectation is that JSON schema is published to help
> with authoring configuration files which will take care of case-sensitive member names so only the member values
> case-sensitivity is determined by thie `caseSensitive` member.

The `requires` section is optional and declares resources the configuration resource depends upon.
For example, version 1.0 of the sshdconfig resource may require a minimal version of sshd executable to work correctly.
Tooling like PSDesiredStateConfiguration may perform dependency versioning checks and fail fast,
but initially it may only serve as documentation.
Version can be a range using [nuget version range syntax](https://docs.microsoft.com/en-us/nuget/concepts/package-versioning#version-ranges).

The `exitCodes` section is optional and documents the exit codes that the resource can return.
An exit code of 0 is always success.  Any non-zero exit code is considered a failure.
This section enables more detailed error reporting by tooling.

> [Note] Need to think of how localization will work for `exitCode` messages.

The `schema` member should be a URL to a JSON schema that describes the configuration for the resource
so it can be used for intellisense while authoring configuration.

> [Note] Should we support embedded schema in the manifest file?

In the future, if we want to support long running host processes instead of spawning a new
process each time, we can use something like:

```json
{
  "manifestVersion": "1.0",
  "configuration": {
    "get": {
        "host": "powershell-host",
        "args": [ "" ],
        "acceptStdin": true
    }
  }
}
```

Here, the "host" member points to an executable that implements some well defined GRPC endpoints to be
defined in the future.
This is currently out of scope, but allows for this type of support in the future.

## PowerShell module based resources

Existing PowerShell based resources are packaged as modules whether they are script functions or classes.
Some require Windows PowerShell while others may require PowerShell 7.
Both will be supported, but a new resource manifest must be created to work in DSC v3.

### .ps1 as a resource manifest

A PowerShell based resource can be use a `.ps1` script and not part of a module.
A `.ps1` script works similar to a command based resource except that converstion to/from JSON will be handled by the `config` command
and the script will just need to work with PSObjects.
The `config` command will need to know whether to use Windows PowerShell or PowerShell 7 to execute the script and is specified explicitly
in the manifest file:

```json
{
  "manifestVersion": "1.0",
  "dscresource": {
    "name": "Microsoft.DSC.ScriptResource",
    "version": "0.0.1-preview",
    "get": {
        "executable": "pwsh",
        "type": "powershellscript",
        "file": "ScriptResource.ps1",
        "args": [ "-method", "get" ],
        "inputArg": "-inputObject"
    }
  }
}
```

In this fragment, the `executable` points to the PowerShell executable to use.
The `type` member informs the `config` command to special case the execution of the script as it would be base64 encoded.
The JSON input will be converted to a HashTable and passed to the script as a parameter.
The output object from the resource will be converted by the `config` command to JSON.
As `get`, `set`, and `test` could be different script files with different requires, each section may repeat similar property values.

### Module based resources

Existing PowerShell class and script function resources packaged as a module will need the addition of a manifest file.

```json
{
  "manifestVersion": "1.0",
  "dscresource": {
    "name": "Microsoft.DSC.MyResource",
    "version": "0.0.1-preview",
    "get": {
        "executable": "pwsh",
        "type": "powershellmodule",
        "module": "Microsoft.DSC.DSCResource",
        "name": "MyResource"
    }
  }
}
```

> [Note] Perhaps we can require this JSON be in the same folder as the module and then the module name doesn't need to be specified.

## Resource input

The JSON input to the resource will be just the `settings` for that resource.
See https://microsoft.ghe.com/AzureCore-Compute/PowerShellTeam-Docs/pull/36/files for a higher level configuration example and the
contents of the `settings` for a resource is expected to be passed as JSON.

For example, a `get` operation against a `SecretManagement` resource would be:
  
```json
{
  "name": "MySecret",
  "vault": "AzureKeyVault"
}
```

## Resource output

### Success output

The exit code must be 0 for success.
Legacy PowerShell resources don't set the exit code so will default to 0.
Optional JSON output via STDOUT provides additional information and may be used by the orchestrator.
For example, the result of a `set` with a resource to install software may return the path to the installed software
as well as the install date and version which may not be specifically requested by the input payload.
In this case, the request may be a range of versions and the response will be the specific version installed.

### Failure output

The exit code may be non-zero to indicate failure.
However, some languages may not make it easy to set the exit code so it's not required.
JSON output via STDERR is interpreted as a failure only if the top level member is `error`:

```json
{
  "error": {
    "code": 2,
    "message": "Access Denied"
  }
}
```

Since STDERR is used for other types of output (verbose/debug messages), the top level member indicates failure.

> [TODO] Should define a common schema for error messages so an orchestrator can more easily parse
> and report the errors to the user.

### Get operation

If no input JSON is provided as a filter, then the resource will return all managed settings as a single
JSON object to STDOUT.
In cases where it does not make sense to return the entirety of the domain as JSON (for example,
registry or file system), then the resource should return an error with a non-zero exit code indicating
such an operation is not supported.

If input JSON is provided, then the resource will return all managed settings that match the filter as a single
JSON object to STDOUT.
If multiple instances need to be returned, it would be represented as an array within the single JSON response object.

> [Note] Streaming of JSON is only needed if there's a scenario that would make use of streaming, otherwise
> it's simpler to always return a single JSON object as a container for multiple objects.

### Set operation

If the resource declares `returnState` as `true` in the manifest, then the resource shall return JSON
representing the current state of the settings after successful set operation.
Otherwise, no output is expected for success and the `config` command will invoke `get` to get the current state.
Failure follows "Failure output" section above.

### Test operation

If the input JSON matches the current state, then a 0 exit code is returned and no output is sent to STDOUT.
Otherwise, a non-zero exit code is returned and JSON representing settings where the values differ is sent to STDOUT
where the values are the current values.

### Extended information output

Resources that return extended information must conform to this format sent to STDERR.

Error messages have this syntax:

```json
{
  "error": {
    "code": "1",
    "message": "Access Denied"
  }
}
```

Verbose messages have this syntax:

```json
{
  "verbose": {
    "message": "Setting 'MySetting' changed from 'OldValue' to 'NewValue'"
  }
}
```

Warning messages have this syntax:

```json
{
  "warning": {
    "message": "Setting 'MySetting' is not valid"
  }
}
```

Note that any JSON object to STDERR needs to be on a single line even thought these
examples show them on multiple lines.
It is up to tooling to determine how to display this information and how to handle any
JSON objects that are returned via STDERR that don't match the above syntax.
Any non-standard JSON objects (or invalid JSON) will result in the `config` command emitting a warning.

## Common configuration members

For consistency, the following members are common to all configuration resource settings:

`_ensure`: Used to specify if that setting should be `present` or `absent`.
`_purge`: Indicate if unmanaged settings should be removed (`true`) or left alone (`false`).  Default is `false`.
`_validation`: Indicate if the resource should fail if there are unknown settings (`strict`) or ignore them (`loose`).  Default is `strict`.

Common configuration members start with a leading underscore to avoid conflicts with existing resource settings
and indicate consistent definitions for these settings.
