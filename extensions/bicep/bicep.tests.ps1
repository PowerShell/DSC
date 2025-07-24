# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    $foundBicep = if ($null -ne (Get-Command bicep -ErrorAction Ignore)) {
        $true
    } else {
        $false
    }
}

Describe 'Bicep extension tests' -Skip:(!$foundBicep) {
    It 'Example bicep file should work' {
        $bicepFile = Resolve-Path -Path "$PSScriptRoot\..\..\dsc\examples\hello_world.dsc.bicep"
        $out = dsc -l debug config get -f $bicepFile 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw | Out-String)
        $out.results[0].result.actualState.output | Should -BeExactly 'Hello, world!'
        (Get-Content -Path $TestDrive/error.log -Raw) | Should -Match "Importing file '$bicepFile' with extension 'Microsoft.DSC.Extension/Bicep'"
    }

    It 'Invalid bicep file returns error' {
        $bicepFile = "$TestDrive/invalid.bicep"
        Set-Content -Path $bicepFile -Value @"
        myresource invalid 'Microsoft.DSC.Extension/Bicep:1.0' = {
            name: 'invalid'
            properties: {
                output: 'This is invalid'
"@
        $out = dsc -l trace config get -f $bicepFile 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 4 -Because (Get-Content -Path $TestDrive/error.log -Raw | Out-String)
        $content = (Get-Content -Path $TestDrive/error.log -Raw)
        $content | Should -Match "Importing file '$bicepFile' with extension 'Microsoft.DSC.Extension/Bicep'"
        $content | Should -Match "BCP279: Expected a type at this location"
    }
}
