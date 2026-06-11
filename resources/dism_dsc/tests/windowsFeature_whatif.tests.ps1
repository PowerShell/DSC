# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'WindowsFeatureList what-if tests' -Skip:(!$IsWindows) {
    BeforeAll {
        $testFeature = 'TelnetClient'
    }

    It 'Can what-if enable a feature without mutating state' {
        $json = @"
{
    "features": [
        { "featureName": "$testFeature", "state": "Installed" }
    ]
}
"@
        # Capture pre-state
        $before = $json | dism_dsc get windows-feature 2>$null | ConvertFrom-Json

        # Run what-if
        $result = $json | dism_dsc set windows-feature -w 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log -ErrorAction SilentlyContinue)

        # Projected state echoes back the requested feature name and state
        $result.features[0].featureName | Should -Be $testFeature
        $result.features[0].state       | Should -Be 'Installed'

        # what-if metadata present
        $result.features[0]._metadata.whatIf[0] | Should -Match "Would enable feature '$testFeature'"

        # No mutation occurred
        $after = $json | dism_dsc get windows-feature 2>$null | ConvertFrom-Json
        $before | ConvertTo-Json -Depth 10 | Should -Be ($after | ConvertTo-Json -Depth 10)
    }

    It 'Can what-if disable a feature without mutating state' {
        $json = @"
{
    "features": [
        { "featureName": "$testFeature", "state": "NotPresent" }
    ]
}
"@
        # Capture pre-state
        $before = $json | dism_dsc get windows-feature 2>$null | ConvertFrom-Json

        # Run what-if
        $result = $json | dism_dsc set windows-feature -w 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0

        $result.features[0].featureName | Should -Be $testFeature
        $result.features[0].state       | Should -Be 'NotPresent'
        $result.features[0]._metadata.whatIf[0] | Should -Match "Would disable feature '$testFeature'"

        # No mutation occurred
        $after = $json | dism_dsc get windows-feature 2>$null | ConvertFrom-Json
        $before | ConvertTo-Json -Depth 10 | Should -Be ($after | ConvertTo-Json -Depth 10)
    }

    It 'Can what-if remove a feature without mutating state' {
        $json = @"
{
    "features": [
        { "featureName": "$testFeature", "state": "Removed" }
    ]
}
"@
        # Capture pre-state
        $before = $json | dism_dsc get windows-feature 2>$null | ConvertFrom-Json

        # Run what-if
        $result = $json | dism_dsc set windows-feature -w 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0

        $result.features[0].featureName | Should -Be $testFeature
        $result.features[0].state       | Should -Be 'Removed'
        $result.features[0]._metadata.whatIf[0] | Should -Match "Would remove feature '$testFeature'"

        # No mutation occurred
        $after = $json | dism_dsc get windows-feature 2>$null | ConvertFrom-Json
        $before | ConvertTo-Json -Depth 10 | Should -Be ($after | ConvertTo-Json -Depth 10)
    }

    It 'Can what-if enable a feature with enableAll and limitAccess without mutating state' {
        $json = @"
{
    "features": [
        { "featureName": "$testFeature", "state": "Installed", "enableAll": true, "limitAccess": true }
    ]
}
"@
        $before = $json | dism_dsc get windows-feature 2>$null | ConvertFrom-Json

        $result = $json | dism_dsc set windows-feature -w 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0

        $result.features[0].featureName | Should -Be $testFeature
        $result.features[0].state       | Should -Be 'Installed'
        $result.features[0].enableAll   | Should -BeTrue
        $result.features[0].limitAccess | Should -BeTrue
        $result.features[0]._metadata.whatIf[0] | Should -Match "Would enable feature '$testFeature'"

        $after = $json | dism_dsc get windows-feature 2>$null | ConvertFrom-Json
        $before | ConvertTo-Json -Depth 10 | Should -Be ($after | ConvertTo-Json -Depth 10)
    }

    It 'Can what-if multiple features in one call without mutating state' {
        $json = @"
{
    "features": [
        { "featureName": "$testFeature", "state": "Installed" },
        { "featureName": "$testFeature", "state": "NotPresent" }
    ]
}
"@
        $result = $json | dism_dsc set windows-feature -w 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0

        $result.features | Should -HaveCount 2
        $result.features[0]._metadata.whatIf[0] | Should -Match "Would enable"
        $result.features[1]._metadata.whatIf[0] | Should -Match "Would disable"
    }
}
