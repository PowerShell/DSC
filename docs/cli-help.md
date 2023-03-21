# DSC Cli Help

C:\git\Microsoft.Dsc\dsc\bin\Debug\net7.0> .\dsc.exe --help

```output
Description:
  Command line interface for working with Desired State Configuration.

Usage:
  dsc [command] [options]

Options:
  --version       Show version information
  -?, -h, --help  Show help and usage information

Commands:
  config    Invoke desired state configurations.
  module    Manage modules.
  repo      Manage repositories.
  resource  Invoke and find resources.
```

C:\git\Microsoft.Dsc\dsc\bin\Debug\net7.0> .\dsc.exe config --help

```output
Description:
  Invoke desired state configurations.

Usage:
  dsc config [command] [options]

Options:
  -?, -h, --help  Show help and usage information

Commands:
  test  test a machine's desired state configuration.
  set   Set a machine's desired state configuration.
```

C:\git\Microsoft.Dsc\dsc\bin\Debug\net7.0> .\dsc.exe config set --help

```output
Description:
  Set a machine's desired state configuration.

Usage:
  dsc config set [options]

Options:
  -f, --file <file> (REQUIRED)  The configuration file.
  -?, -h, --help                Show help and usage information
```

C:\git\Microsoft.Dsc\dsc\bin\Debug\net7.0> .\dsc.exe config test --help

```output
Description:
  test a machine's desired state configuration.

Usage:
  dsc config test [options]

Options:
  -f, --file <file> (REQUIRED)  The configuration file.
  -?, -h, --help                Show help and usage information

```

C:\git\Microsoft.Dsc\dsc\bin\Debug\net7.0> .\dsc.exe module --help

```output
Description:
  Manage modules.

Usage:
  dsc module [command] [options]

Options:
  -?, -h, --help  Show help and usage information

Commands:
  find     Find modules
  install  install modules

C:\git\Microsoft.Dsc\dsc\bin\Debug\net7.0> .\dsc.exe module find --help
```output
Description:
  Find modules

Usage:
  dsc module find [options]

Options:
  -n, --name <name> (REQUIRED)  The name of the module
  -v, --version <version>       The version of the module
  -r, --repo <repo>             The repo that contains the module
  -?, -h, --help                Show help and usage information


```

C:\git\Microsoft.Dsc\dsc\bin\Debug\net7.0> .\dsc.exe module install --help

```output
Description:
  install modules

Usage:
  dsc module install [options]

Options:
  -n, --name <name> (REQUIRED)  The name of the module
  -v, --version <version>       The version of the module
  -r, --repo <repo>             The repo that contains the module
  -?, -h, --help                Show help and usage information
```

C:\git\Microsoft.Dsc\dsc\bin\Debug\net7.0> .\dsc.exe resource --help

```output
Description:
  Invoke and find resources.

Usage:
  dsc resource [command] [options]

Options:
  -?, -h, --help  Show help and usage information

Commands:
  get   Get the current state of a resource
  test  Test the state of a resource
  set   Set the state of a resource
  find  Find a resource

```

C:\git\Microsoft.Dsc\dsc\bin\Debug\net7.0> .\dsc.exe resource get --help

```output
Description:
  Get the current state of a resource

Usage:
  dsc resource get [options]

Options:
  -n, --name <name> (REQUIRED)           The name of the resource
  -m, --module <module> (REQUIRED)       The module that contains the resource
  -v, --version <version>                The version of the module to use
  -p, --properties <properties>          Property of the resource. Format as key:value
  -j, --jsonProperties <jsonProperties>  A JSON string that will be passed as input to the resource
  -?, -h, --help                         Show help and usage information


```

C:\git\Microsoft.Dsc\dsc\bin\Debug\net7.0> .\dsc.exe resource set --help

```output
Description:
  Set the state of a resource

Usage:
  dsc resource set [options]

Options:
  -n, --name <name> (REQUIRED)           The name of the resource
  -m, --module <module> (REQUIRED)       The module that contains the resource
  -v, --version <version>                The version of the module to use
  -p, --properties <properties>          Property of the resource. Format as key:value
  -j, --jsonProperties <jsonProperties>  A JSON string that will be passed as input to the resource
  -?, -h, --help                         Show help and usage information


```

C:\git\Microsoft.Dsc\dsc\bin\Debug\net7.0> .\dsc.exe resource test --help

```output
Description:
  Test the state of a resource

Usage:
  dsc resource test [options]

Options:
  -n, --name <name> (REQUIRED)           The name of the resource
  -m, --module <module> (REQUIRED)       The module that contains the resource
  -v, --version <version>                The version of the module to use
  -p, --properties <properties>          Property of the resource. Format as key:value
  -j, --jsonProperties <jsonProperties>  A JSON string that will be passed as input to the resource
  -?, -h, --help                         Show help and usage information


```

C:\git\Microsoft.Dsc\dsc\bin\Debug\net7.0> .\dsc.exe resource find --help

```output
Description:
  Find a resource

Usage:
  dsc resource find [options]

Options:
  -v, --version <version>  The version of the module to use
  -?, -h, --help           Show help and usage information

```

C:\git\Microsoft.Dsc\dsc\bin\Debug\net7.0> .\dsc.exe repo --help
```output
Description:
  Manage repositories.

Usage:
  dsc repo [command] [options]

Options:
  -?, -h, --help  Show help and usage information

Commands:
  set     Add or update a repository
  remove  Remove a repository

```

C:\git\Microsoft.Dsc\dsc\bin\Debug\net7.0> .\dsc.exe repo set --help

```output
Description:
  Add or update a repository

Usage:
  dsc repo set [options]

Options:
  -n, --name <name> (REQUIRED)  The name of the repository
  -u, --uri <uri> (REQUIRED)    The uri of the repository
  -?, -h, --help                Show help and usage information


```

C:\git\Microsoft.Dsc\dsc\bin\Debug\net7.0> .\dsc.exe repo remove --help

```output
Description:
  Remove a repository

Usage:
  dsc repo remove [options]

Options:
  -n, --name <name> (REQUIRED)  The name of the repository
  -?, -h, --help                Show help and usage information

```
