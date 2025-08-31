# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'tests for metadata versioning' {
  It 'returns the correct dsc semantic version in metadata' {
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: 'Hello, World!'
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $version = $out.metadata.'Microsoft.DSC'.version -as [System.Management.Automation.SemanticVersion]
    $version | Should -Not -BeNullOrEmpty
    $dscVersion = (dsc --version).Split(" ")[1]
    $version | Should -Be $dscVersion
  }
}