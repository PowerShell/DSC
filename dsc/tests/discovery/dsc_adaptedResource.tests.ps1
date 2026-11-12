# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for adapted resources' {
    BeforeAll {
        $isAdmin = if ($IsWindows) {
            $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
            [System.Security.Principal.WindowsPrincipal]::new($identity).IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)
        }
        else {
            [System.Environment]::UserName -eq 'root'
        }
    }

    It 'Security context <securityContext> for operation <operation> works' -TestCases @(
        @{ securityContext = 'Elevated'; operation = 'get' },
        @{ securityContext = 'Elevated'; operation = 'test' },
        @{ securityContext = 'Elevated'; operation = 'set' },
        @{ securityContext = 'Elevated'; operation = 'delete' },
        @{ securityContext = 'Elevated'; operation = 'export' },
        @{ securityContext = 'Restricted'; operation = 'get' },
        @{ securityContext = 'Restricted'; operation = 'test' },
        @{ securityContext = 'Restricted'; operation = 'set' },
        @{ securityContext = 'Restricted'; operation = 'delete' },
        @{ securityContext = 'Restricted'; operation = 'export' },
        @{ securityContext = 'Current'; operation = 'get' },
        @{ securityContext = 'Current'; operation = 'test' },
        @{ securityContext = 'Current'; operation = 'set' },
        @{ securityContext = 'Current'; operation = 'delete' },
        @{ securityContext = 'Current'; operation = 'export' }
    ) {
        param($securityContext, $operation)

        $resourceType = "Adapted/SecurityContext$securityContext"
        $json = @{
            one = $securityContext
        } | ConvertTo-Json -Compress
        $out = dsc -l trace resource $operation -r $resourceType 2>$TestDrive/error.txt -i $json
        $errorTxt = Get-Content -Path $TestDrive/error.txt -Raw
        if ($securityContext -eq 'Elevated' -and !$isAdmin) {
            $LASTEXITCODE | Should -Be 2 -Because $errorTxt
            $errorTxt | Should -BeLike "*Operation '$operation' for resource 'Adapted/SecurityContext$securityContext' requires security context 'elevated'*"
        }
        elseif ($securityContext -eq 'Restricted' -and $isAdmin) {
            $LASTEXITCODE | Should -Be 2
            $errorTxt | Should -BeLike "*Operation '$operation' for resource 'Adapted/SecurityContext$securityContext' requires security context 'restricted'*"
        }
        else {
            $LASTEXITCODE | Should -Be 0
            if ($operation -ne 'delete') {
                $out | Should -Not -BeNullOrEmpty
            } else {
                $out | Should -BeNullOrEmpty
            }
        }
    }

    Context 'Return kind for set and test' {
        BeforeAll {
            $desired = @{ one = 'desired' } | ConvertTo-Json -Compress
        }

        It 'Inherits the return kind of the adapter for <operation> when the adapted resource does not define it' -TestCases @(
            @{ operation = 'test'; stateProperty = 'actualState' }
            @{ operation = 'set'; stateProperty = 'afterState' }
        ) {
            param($operation, $stateProperty)

            $out = dsc resource $operation -r Adapted/Three -i $desired 2>$TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.txt -Raw)
            $out.$stateProperty.one | Should -BeExactly 'value3'
        }

        It 'Uses return kind state from the adapted resource manifest for test' {
            $out = dsc -l debug resource test -r Adapted/ReturnState -i $desired 2>$TestDrive/error.txt | ConvertFrom-Json
            $errorTxt = Get-Content -Path $TestDrive/error.txt -Raw
            $LASTEXITCODE | Should -Be 0 -Because $errorTxt
            $out.actualState.one | Should -BeExactly 'fromOperation'
            $out.inDesiredState | Should -BeFalse
            $out.differingProperties | Should -Be @('one')
            $errorTxt | Should -BeLike "*Adapted resource 'Adapted/ReturnState' defines return kind 'state' for operation 'test'*"
        }

        It 'Uses return kind stateAndDiff from the adapted resource manifest for test' {
            $out = dsc -l debug resource test -r Adapted/ReturnStateAndDiff -i $desired 2>$TestDrive/error.txt | ConvertFrom-Json
            $errorTxt = Get-Content -Path $TestDrive/error.txt -Raw
            $LASTEXITCODE | Should -Be 0 -Because $errorTxt
            $out.actualState.one | Should -BeExactly 'fromOperation'
            $out.inDesiredState | Should -BeFalse
            $out.differingProperties | Should -Be @('fromAdapter')
            $errorTxt | Should -BeLike "*Adapted resource 'Adapted/ReturnStateAndDiff' defines return kind 'stateAndDiff' for operation 'test'*"
        }

        It 'Uses return kind state from the adapted resource manifest for set' {
            $out = dsc -l debug resource set -r Adapted/ReturnState -i $desired 2>$TestDrive/error.txt | ConvertFrom-Json
            $errorTxt = Get-Content -Path $TestDrive/error.txt -Raw
            $LASTEXITCODE | Should -Be 0 -Because $errorTxt
            $out.beforeState.one | Should -BeExactly 'fromGet'
            $out.afterState.one | Should -BeExactly 'fromOperation'
            $out.changedProperties | Should -Be @('one')
            $errorTxt | Should -BeLike "*Adapted resource 'Adapted/ReturnState' defines return kind 'state' for operation 'set'*"
        }

        It 'Uses return kind stateAndDiff from the adapted resource manifest for set' {
            $out = dsc -l debug resource set -r Adapted/ReturnStateAndDiff -i $desired 2>$TestDrive/error.txt | ConvertFrom-Json
            $errorTxt = Get-Content -Path $TestDrive/error.txt -Raw
            $LASTEXITCODE | Should -Be 0 -Because $errorTxt
            $out.beforeState.one | Should -BeExactly 'fromGet'
            $out.afterState.one | Should -BeExactly 'fromOperation'
            $out.changedProperties | Should -Be @('fromAdapter')
            $errorTxt | Should -BeLike "*Adapted resource 'Adapted/ReturnStateAndDiff' defines return kind 'stateAndDiff' for operation 'set'*"
        }

        It 'Uses the return kind from the adapted resource manifest in a configuration' {
            $config = @{
                '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
                resources = @(
                    @{
                        name       = 'state and diff'
                        type       = 'Adapted/ReturnStateAndDiff'
                        properties = @{ one = 'desired' }
                    }
                )
            } | ConvertTo-Json -Depth 5 -Compress
            $out = dsc config test -i $config 2>$TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.txt -Raw)
            $out.results[0].result.actualState.one | Should -BeExactly 'fromOperation'
            $out.results[0].result.differingProperties | Should -Be @('fromAdapter')
        }

        It 'Rejects an adapted resource manifest with an invalid return kind for <operation>' -TestCases @(
            @{ operation = 'set' }
            @{ operation = 'test' }
        ) {
            param($operation)

            $manifest = @{
                '$schema'      = 'https://aka.ms/dsc/schemas/v3/bundled/resource/adapted/manifest.json'
                type           = 'Adapted/InvalidReturn'
                kind           = 'resource'
                version        = '1.0.0'
                capabilities   = @('get', 'set', 'test')
                requireAdapter = 'Test/Adapter'
                path           = 'invalidReturn.dsc.adaptedResource.json'
                $operation     = @{ return = 'diff' }
                schema         = @{ embedded = @{ type = 'object' } }
            } | ConvertTo-Json -Depth 5
            $manifestDir = Join-Path $TestDrive "invalidReturn-$operation"
            $null = New-Item -Path $manifestDir -ItemType Directory
            Set-Content -Path (Join-Path $manifestDir 'invalidReturn.dsc.adaptedResource.json') -Value $manifest

            $oldPath = $env:DSC_RESOURCE_PATH
            try {
                $env:DSC_RESOURCE_PATH = $manifestDir + [System.IO.Path]::PathSeparator + (Split-Path (Get-Command dsctest).Source)
                $out = dsc resource list 'Adapted/InvalidReturn' 2>$TestDrive/error.txt
                $out | Should -BeNullOrEmpty
                Get-Content -Path $TestDrive/error.txt -Raw | Should -BeLike '*unknown variant*diff*'
            }
            finally {
                $env:DSC_RESOURCE_PATH = $oldPath
            }
        }
    }
}
