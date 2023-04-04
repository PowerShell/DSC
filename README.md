# DSCv3

[NOTICE] This repo is currently NOT accept contributions and is public and Open Source to show progress.  Once we are at a feature complete state, we can start taking contributions.

## High level design goals

- Cross-platform and Open Source
- Bring your own agent
  - No LCM support
  - Azure Guest Config, Azure Automanaged VM, WinGet partners as orchestration agents
- Author resources in PowerShell or any language
  - Need to be executable from command-line
  - Still supporting script based and class based resources
- Native `dsc` command removes dependency on PowerShell
  - Able to invoke PowerShell based resources (Windows PowerShell or PowerShell 7 runtimes)
- Move from MOF to JSON
- End users can author configuration in YAML or JSON and apply them using `dsc` command
