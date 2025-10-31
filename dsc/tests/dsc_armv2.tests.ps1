# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'ARM Language 2.0 tests' {
    It 'config with ARM Language 2.0 format works' {
<#
This JSON config built from the following Bicep code using the
desiredStateConfiguration and moduleExtensionConfigs experimental features:

extension dsc
targetScope = 'desiredStateConfiguration'

resource echoResource 'Microsoft.DSC.Debug/Echo@1.0.0' = {
  output: 'Hello World'
}
#>
        $configJson = @'
{
  "$schema": "https://aka.ms/dsc/schemas/v3/bundled/config/document.json",
  "languageVersion": "2.2-experimental",
  "contentVersion": "1.0.0.0",
  "metadata": {
    "_EXPERIMENTAL_WARNING": "This template uses ARM features that are experimental. Experimental features should be enabled for testing purposes only, as there are no guarantees about the quality or stability of these features. Do not enable these settings for any production usage, or your production environment may be subject to breaking.",
    "_EXPERIMENTAL_FEATURES_ENABLED": [
      "Enable defining extension configs for modules"
    ],
    "_generator": {
      "name": "bicep",
      "version": "0.38.33.27573",
      "templateHash": "5233252217641859406"
    }
  },
  "extensions": {
    "dsc": {
      "name": "DesiredStateConfiguration",
      "version": "0.1.0"
    }
  },
  "resources": {
    "echoResource": {
      "extension": "dsc",
      "type": "Microsoft.DSC.Debug/Echo",
      "apiVersion": "1.0.0",
      "properties": {
        "output": "Hello World"
      }
    }
  }
}
'@
        $out = dsc config get -i $configJson | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $out.results | Should -HaveCount 1
        $out.results[0].type | Should -Be 'Microsoft.DSC.Debug/Echo'
        $out.results[0].result.actualState.output | Should -Be 'Hello World'
    }
}
