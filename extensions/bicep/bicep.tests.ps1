# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    $foundBicep = if (Get-Command bicep -ErrorAction Ignore) {
        $true
    } else {
        $false
    }
}

Describe 'Bicep extension tests' -Skip:(!$foundBicep) {
    BeforeAll {
        $dateVersion = @'
{
    "$schema": "https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json",
    "type": "Microsoft.DSC.Debug/Echo",
    "version": "2025-08-27",
    "description": "Echo resource for testing and debugging purposes",
    "get": {
        "executable": "dscecho",
        "args": [
            {
                "jsonInputArg": "--input",
                "mandatory": true
            }
        ]
    },
    "set": {
        "executable": "dscecho",
        "args": [
            {
                "jsonInputArg": "--input",
                "mandatory": true
            }
        ]
    },
    "test": {
        "executable": "dscecho",
        "args": [
            {
                "jsonInputArg": "--input",
                "mandatory": true
            }
        ]
    },
    "export": {
        "executable": "dscecho",
        "args": [
            {
                "jsonInputArg": "--input",
                "mandatory": true
            }
        ]
    },
    "schema": {
        "command": {
            "executable": "dscecho"
        }
    }
}
'@
        Set-Content -Path "$TestDrive/dateVersion.dsc.resource.json" -Value $dateVersion
        $env:DSC_RESOURCE_PATH = "$env:PATH" + [System.IO.Path]::PathSeparator + "$TestDrive"
    }

    AfterAll {
        $env:DSC_RESOURCE_PATH = $null
    }

    It 'Example bicep file should work' {
        $bicepFile = Resolve-Path -Path "$PSScriptRoot\..\..\dsc\examples\hello_world.dsc.bicep"
        $out = dsc -l trace config get -f $bicepFile 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw | Out-String)
        $out.results[0].result.actualState.output | Should -BeExactly 'Hello, world!'
        $bicepFile = $bicepFile.ToString().Replace('\', '\\')
        (Get-Content -Path $TestDrive/error.log -Raw) | Should -Match "Importing file '$bicepFile' with extension 'Microsoft.DSC.Extension/Bicep'"
    }

    It 'Invalid bicep file returns error' {
        $bicepFile = "$TestDrive/invalid.bicep"
        Set-Content -Path $bicepFile -Value @"
targetScope = 'invalidScope'

resource invalid 'Microsoft.DSC.Extension/Bicep:1.0' = {
    name: 'invalid'
    properties: {
        output: 'This is invalid'
"@
        dsc -l trace config get -f $bicepFile 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 4 -Because (Get-Content -Path $TestDrive/error.log -Raw | Out-String)
        $content = (Get-Content -Path $TestDrive/error.log -Raw)
        $bicepFile = $bicepFile.ToString().Replace('\', '\\')
        $content | Should -Match "Importing file '$bicepFile' with extension 'Microsoft.DSC.Extension/Bicep'"
        $content | Should -Match "BCP033"
    }

    It 'Example bicep parameters file should work' {
        $bicepFile = Resolve-Path -Path "$PSScriptRoot\..\..\dsc\examples\hello_world.dsc.bicep"
        $bicepParamFile = Resolve-Path -Path "$PSScriptRoot\..\..\dsc\examples\hello_world.dsc.bicepparam"
        $out = dsc -l trace config --parameters-file $bicepParamFile get --file $bicepFile 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw | Out-String)
        $out.results[0].result.actualState.output | Should -BeExactly 'This is a parameterized hello world!'
    }
}
