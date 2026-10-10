# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    $dscVersion = (dsc --version).Split(" ")[1] -as [System.Management.Automation.SemanticVersion]
    $isPrerelease = -not [string]::IsNullOrEmpty($dscVersion.PreReleaseLabel)
}

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
              version: '=999.0.0'
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: 'Hello, World!'
"@
        $null = $config_yaml | dsc config get -f - 2>$testdrive/error.log
        $errorLog = Get-Content -Path $testdrive/error.log -Raw
        $errorLog | Should -BeLike "*Validation*Configuration requires DSC version '=999.0.0', but the current version is '*"
        $LASTEXITCODE | Should -Be 2
    }

    It 'returns no error if DSC version satisfies configuration requirement' {
        $dscVersion = (dsc --version).Split(" ")[1] -as [System.Management.Automation.SemanticVersion]
        # A prerelease build only satisfies a requirement that defines a prerelease segment for the
        # same release, so pin a prerelease build to its release cycle, like '^3.4.0-preview'.
        $versionReq = if ($dscVersion.PreReleaseLabel) {
            '^{0}.{1}.{2}-{3}' -f $dscVersion.Major, $dscVersion.Minor, $dscVersion.Patch, $dscVersion.PreReleaseLabel.Split('.')[0]
        } else {
            '>=3.1'
        }
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            directives:
              version: '$versionReq'
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

    It 'validates the version directive against the running DSC version' {
        $dscVersion = (dsc --version).Split(" ")[1]
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            directives:
              version: '=$dscVersion'
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

    It 'reports the running DSC version when the requirement is not satisfied' {
        $dscVersion = (dsc --version).Split(" ")[1]
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            directives:
              version: '<3.2.0'
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: 'Hello, World!'
"@
        $null = $config_yaml | dsc config get -f - 2>$testdrive/error.log
        $errorLog = Get-Content -Path $testdrive/error.log -Raw
        $errorLog | Should -BeLike "*Validation*Configuration requires DSC version '<3.2.0', but the current version is '$dscVersion'*"
        $LASTEXITCODE | Should -Be 2
    }

    It 'requires a prerelease segment to match a prerelease DSC version for requirement: <requirement>' -Skip:(-not $isPrerelease) -TestCases @(
        @{ requirement = '={0}.{1}.{2}'; satisfied = $false }
        @{ requirement = '>={0}.{1}.{2}'; satisfied = $false }
        @{ requirement = '<{0}.{1}.{2}'; satisfied = $false }
        @{ requirement = '^{0}.{1}.{2}-{3}'; satisfied = $true }
        @{ requirement = '>={0}.{1}.{2}-{4}'; satisfied = $true }
    ) {
        param($requirement, $satisfied)

        $dscVersion = (dsc --version).Split(" ")[1] -as [System.Management.Automation.SemanticVersion]
        $versionReq = $requirement -f $dscVersion.Major, $dscVersion.Minor, $dscVersion.Patch, $dscVersion.PreReleaseLabel.Split('.')[0], $dscVersion.PreReleaseLabel
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            directives:
              version: '$versionReq'
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: 'Hello, World!'
"@
        $null = $config_yaml | dsc config get -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be ($satisfied ? 0 : 2) -Because (Get-Content -Path $testdrive/error.log -Raw)
    }
}
