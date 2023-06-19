# DSC Resource examples

These files and folders contain the fictional TSToys application and DSC Resources implemented in different languages to manage it.

You can build the application and the available resources by running the build script in this folder.

```powershell
# Just build and idempotently add to path
.\build.ps1
# Build, idempotently add to path, and load completions.
. .\build.ps1 -Initialize
```

With the binaries built and available, you can call them:

```powershell
tstoy help
gotstoy help
```

## `tstoy`

The fictional application. You can use its commands to inspect how the DSC Resources have affected
the configuration.

```powershell
# Show the application's settings and merged outcome.
tstoy show
```

```output
Default configuration: {
  "Updates": {
    "Automatic": false,
    "CheckFrequency": 90
  }
}
Machine configuration: {}
User configuration: {}
Final configuration: {
  "updates": {
    "automatic": false,
    "checkfrequency": 90
  }
}
```

You can retrieve the path to the configuration files with the `tstoy show path` subcommand.

```powershell
# Get both paths
tstoy show path
# get only machine-scope path
tstoy show path machine
# get only user-scope path
tstoy show path user
```

```output
C:\ProgramData\TailSpinToys\tstoy\tstoy.config.json
C:\Users\mlombardi\AppData\Local\TailSpinToys\tstoy\tstoy.config.json

C:\ProgramData\TailSpinToys\tstoy\tstoy.config.json

C:\Users\mlombardi\AppData\Local\TailSpinToys\tstoy\tstoy.config.json
```

## `gotstoy`

The golang implementation of a DSC Resource for managing the tstoy application.

### Getting current state

You can retrieve the current state of the resource with the `get` command.

```powershell
# Get current state with flags
gotstoy get --scope machine --ensure present --updateAutomatically=false
# Get with JSON over stdin
@'
{
    "scope": "user",
    "ensure": "present",
    "updateAutomatically": true,
    "updateFrequency": 45
}
'@ | gotstoy get
# Get current state of all scopes, pretty printed:
gotstoy get --all --pretty
```

```output
{"ensure":"absent","scope":"machine"}

{"ensure":"absent","scope":"user"}

{
  "ensure": "absent",
  "scope": "machine"
}
{
  "ensure": "absent",
  "scope": "user"
}
```

### Setting desired state

You can enforce the state of the resource with the `set` command.

```powershell
# Set the state with flags
gotstoy set --scope machine --ensure present --updateAutomatically=false
# Set with JSON over stdin
@'
{
    "scope": "user",
    "ensure": "present",
    "updateAutomatically": true,
    "updateFrequency": 45
}
'@ | gotstoy set
# Get new state of all scopes, pretty printed:
gotstoy get --all --pretty
```

```output
{"ensure":"present","scope":"machine","updateAutomatically":false}

{"ensure":"present","scope":"user","updateAutomatically":true,"updateFrequency":45}

{
  "ensure": "present",
  "scope": "machine",
  "updateAutomatically": false
}
{
  "ensure": "present",
  "scope": "user",
  "updateAutomatically": true,
  "updateFrequency": 45
}
```

### Verifying state

After you've enforced state, you should verify the changes with the `tstoy` application itself:

```powershell
tstoy show
```

```output
Default configuration: {
  "Updates": {
    "Automatic": false,
    "CheckFrequency": 90
  }
}
Machine configuration: {
  "updates": {
    "automatic": false
  }
}
User configuration: {
  "updates": {
    "automatic": true,
    "checkfrequency": 45
  }
}
Final configuration: {
  "updates": {
    "automatic": true,
    "checkfrequency": 45
  }
}
```
