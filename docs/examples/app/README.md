# The `tstoy` example app

This folder has the source code for a fictional application, `tstoy`. It's intended to serve as a functional mockup of a real application a user might want to configure.

It has two configuration options:

1. Should the application update automatically?
1. How frequently should the application check for updates?

These values are represented by keys in configuration files. The default configuration is:

```json
{
    "updates": {
        "automatic": false,
        "checkFrequency": 90
    }
}
```

The application is implemented so that it can merge configuration from:

1. The default configuration built into the code.
1. A machine-scope configuration file.
1. A user-scope configuration file.
1. Environment variables with the prefix `TSTOY_`.
1. Explicit flags passed to the application.

DSC Resources targeting this application should be able to ensure:

- Whether a configuration file in a specific scope exists.
- Whether a configuration file defines that the application should automatically update.
- The frequency of the automatic updates, if they're enabled.
