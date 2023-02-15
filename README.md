# DSCv3

Consider all current content Microsoft Confidential.  Eventually, this project will be OSS, but some examples are based on unannounced partner projects.

## High level design goals

- PSDesiredStateConfiguration v3 module
  - Separate from PowerShell engine to enable independent iteration
  - Get-DscResource, Invoke-DscResource can query and invoke native resources
- Cross-platform and Open Source
- Bring your own agent
  - No LCM support
  - Azure Guest Config, Azure Automanaged VM, WinGet partners as orchestration agents
- Author resources in PowerShell or any language
  - Need to be executable from command-line
  - Still supporting script based and class based resources
  - Adding .ps1 based resource for simpler authoring
- Native `config` command removes dependency on PowerShell
  - Able to invoke PowerShell based resources (Windows PowerShell or PowerShell 7 runtimes)
  - `config list`, `config get`, `config set`, `config test`
- Move from MOF to JSON
- End users can author configuration in YAML

## High level architecture

```
+------------------------------------+
|                                    |
|     Config/Orchestration YAML      |
|                                    |
+------------------------------------+
                   |
+------------------------------------+
|                                    |
|     Agent (e.g. Machine Config)    |   # An agent can either host PS and call Invoke-DscResource or Config command/API 
|                                    |
+------------------------------------+
                   |
        +----------+----------------------------+
        |                                       |
+---------------+                       +--------------+
|               +-----------------------+              |
| ConfigExe/API |                       |   PSDSC v3   |  # PSDSC v3 would call Config to call a resource
|               +-----+------------+    |              |
+---------------+     |            |    +--------------+
        |             |            |
+--------------+      |            |
|              |      |     +------+-------+ 
|   Command    |      |     |              |
|   Resource   |      |     | PowerShell 7 |
|              |      |     |              |
+--------------+      |     +-------+------+
                      |             |
             +--------+-----+   +---+--------+
             |              |   |            |
             |   Windows    +---|  PSDSC v3  |  # Config would use PSDSC v3 hosted in WinPS or PS7 as needed to actually invoke the resource
             |  PowerShell  |   |            |
             |              |   +------+-----+
             +--------------+          |
                                +------+--------+
                                |               |
                                | PS script,    |
                                | PS class,     |
                                | .ps1 resource |
                                |               |
                                +---------------+
```                                
