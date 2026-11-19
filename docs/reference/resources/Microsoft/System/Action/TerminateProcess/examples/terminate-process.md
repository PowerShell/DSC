---
description: Terminate an explicitly selected process using a DSC configuration.
ms.date: 11/09/2026
ms.topic: reference
title: Terminate a selected process
---

# Terminate a selected process

Inspect processes first with `Microsoft.System/Process`. Replace `1234` below with the PID
of a disposable process that you intend to terminate.

Save the following configuration as [terminate.config.dsc.yaml](./terminate.config.dsc.yaml):

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Terminate selected process
    type: Microsoft.System.Action/TerminateProcess
    properties:
      pid: 1234
```

Run it on any supported platform:

```powershell
dsc config set -f terminate.config.dsc.yaml
```

The action's `afterState.terminated` array identifies the processes whose termination requests
succeeded.

To terminate all processes with one exact name, replace `pid` with `name`:

```yaml
properties:
  name: my-disposable-process
```

Use the name returned by `Microsoft.System/Process`. On Windows this typically includes `.exe`.
Don't specify both selectors. Wildcards aren't supported by the termination action.
