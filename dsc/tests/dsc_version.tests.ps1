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

    It 'returns error if configuration requires higher DSC version' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            directives:
              version: 999.0.0
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: 'Hello, World!'
"@
        $null = $config_yaml | dsc config get -f - 2>$testdrive/error.log
        $errorLog = Get-Content -Path $testdrive/error.log -Raw
        $errorLog | Should -BeLike "*Validation*Configuration requires DSC version '999.0.0', but the current version is '*"
        $LASTEXITCODE | Should -Be 2
    }

    It 'returns no error if DSC version satisfies configuration requirement' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            directives:
              version: '>=3.1'
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: 'Hello, World!'
"@
        $out = $config_yaml | dsc config get -f - 2>$testdrive/error.log
        $errorLog = Get-Content -Path $testdrive/error.log -Raw
        $errorLog | Should -BeNullOrEmpty
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.results[0].result.actualState.output | Should -BeExactly 'Hello, World!' -Because $out
    }
}
