# Introduction

The `powershell-adapters` folder contains helper modules that can be loaded into your PowerShell session to assist you in familiarizing yourself with new DSC concepts. To see the availability of helper modules, see the following list:

- **DSC Configuration Migration Module**: - Aids in the assistance of grabbing configuration documents written in PowerShell code and transform them to valid configuration documents for the DSC version 3 core engine (e.g. YAML or JSON).

## Getting started

To get started using the helper modules, you can follow the below steps. This example uses the _DSC Configuration Migration Tool_ to be loaded into the session:

1. Open a PowerShell terminal session
2. Execute the following command: `Import-Module "powershell-helpers\dscConfigurationMigrationTool.psm1"`
3. Discover examples using: `Get-Help ConvertTo-DscYaml`
