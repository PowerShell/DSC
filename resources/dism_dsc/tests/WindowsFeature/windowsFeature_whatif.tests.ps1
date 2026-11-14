# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'WindowsFeatureList what-if tests' -Skip:(!$IsWindows) {
    BeforeAll {
        $resourceType = 'Microsoft.Windows/WindowsFeatureList'
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
        $before = (dsc resource get -r $resourceType -i $json 2>$null | ConvertFrom-Json).actualState

        # Run what-if
        $result = (dsc resource set -r $resourceType -i $json -w 2>$null | ConvertFrom-Json).afterState
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log -ErrorAction SilentlyContinue)

        # Projected state echoes back the requested feature name and state
        $result.features[0].featureName | Should -Be $testFeature
        $result.features[0].state       | Should -Be 'Installed'

        # what-if metadata present
        $result.features[0]._metadata.whatIf[0] | Should -Match "Would enable feature '$testFeature'"

        # No mutation occurred
        $after = (dsc resource get -r $resourceType -i $json 2>$null | ConvertFrom-Json).actualState
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
        $before = (dsc resource get -r $resourceType -i $json 2>$null | ConvertFrom-Json).actualState

        # Run what-if
        $result = (dsc resource set -r $resourceType -i $json -w 2>$null | ConvertFrom-Json).afterState
        $LASTEXITCODE | Should -Be 0

        $result.features[0].featureName | Should -Be $testFeature
        $result.features[0].state       | Should -Be 'NotPresent'
        $result.features[0]._metadata.whatIf[0] | Should -Match "Would disable feature '$testFeature'"

        # No mutation occurred
        $after = (dsc resource get -r $resourceType -i $json 2>$null | ConvertFrom-Json).actualState
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
        $before = (dsc resource get -r $resourceType -i $json 2>$null | ConvertFrom-Json).actualState

        # Run what-if
        $result = (dsc resource set -r $resourceType -i $json -w 2>$null | ConvertFrom-Json).afterState
        $LASTEXITCODE | Should -Be 0

        $result.features[0].featureName | Should -Be $testFeature
        $result.features[0].state       | Should -Be 'Removed'
        $result.features[0]._metadata.whatIf[0] | Should -Match "Would remove feature '$testFeature'"

        # No mutation occurred
        $after = (dsc resource get -r $resourceType -i $json 2>$null | ConvertFrom-Json).actualState
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
        $before = (dsc resource get -r $resourceType -i $json 2>$null | ConvertFrom-Json).actualState

        $result = (dsc resource set -r $resourceType -i $json -w 2>$null | ConvertFrom-Json).afterState
        $LASTEXITCODE | Should -Be 0

        $result.features[0].featureName | Should -Be $testFeature
        $result.features[0].state       | Should -Be 'Installed'
        $result.features[0].enableAll   | Should -BeTrue
        $result.features[0].limitAccess | Should -BeTrue
        $result.features[0]._metadata.whatIf[0] | Should -Match "Would enable feature '$testFeature'"

        $after = (dsc resource get -r $resourceType -i $json 2>$null | ConvertFrom-Json).actualState
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
        $result = (dsc resource set -r $resourceType -i $json -w 2>$null | ConvertFrom-Json).afterState
        $LASTEXITCODE | Should -Be 0

        $result.features | Should -HaveCount 2
        $result.features[0]._metadata.whatIf[0] | Should -Match "Would enable"
        $result.features[1]._metadata.whatIf[0] | Should -Match "Would disable"
    }
}
