# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
        $testCases = if ($IsWindows) {
            @(
                @{
                    resourceType = 'Microsoft.DSC.Transitional/PowerShellScript'
                }
                @{
                    resourceType = 'Microsoft.DSC.Transitional/WindowsPowerShellScript'
                }
            )
        } else {
            @(
                @{
                    resourceType = 'Microsoft.DSC.Transitional/PowerShellScript'
                }
            )
        }
}

Describe 'Tests for PSScript resource' {
    It 'Get operation returns the script content for <resourceType>' -TestCases $testCases {
        param($resourceType)

        $yaml = @"
        GetScript: |
          "Hello, World!"
          1+1
        SetScript: |
          throw 'This should not be executed'
"@
        $result = dsc resource get -r $resourceType -i $yaml 2> $TestDrive/error.txt | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.txt -Raw | Out-String)
        $result.actualState.output.Count | Should -Be 2 -Because ($result | ConvertTo-Json | Out-String)
        $result.actualState.output[0] | Should -BeExactly "Hello, World!"
        $result.actualState.output[1] | Should -BeExactly 2
    }

    It 'Set operation executes the script for <resourceType>' -TestCases $testCases {
        param($resourceType)

        $yaml = @"
        SetScript: |
          "Hello, World!"
          1+1
"@
        $result = dsc resource set -r $resourceType -i $yaml 2> $TestDrive/error.txt | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.txt -Raw | Out-String)
        $result.afterState.output.Count | Should -Be 2 -Because ($result | ConvertTo-Json | Out-String)
        $result.afterState.output[0] | Should -BeExactly "Hello, World!"
        $result.afterState.output[1] | Should -BeExactly 2
    }

    It 'Get w/ Set operation succeeds for <resourceType>' -TestCases $testCases {
        param($resourceType)

        $yaml = @"
        GetScript: |
          "Hello, World!"
          1+1
        SetScript: |
          "Hello, World!"
          2+2
"@
        $result = dsc resource set -r $resourceType -i $yaml 2> $TestDrive/error.txt | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.txt -Raw | Out-String)
        $result.beforeState.output.Count | Should -Be 2 -Because ($result | ConvertTo-Json | Out-String)
        $result.beforeState.output[0] | Should -BeExactly "Hello, World!"
        $result.beforeState.output[1] | Should -BeExactly 2
        $result.afterState.output.Count | Should -Be 2
        $result.afterState.output[0] | Should -BeExactly "Hello, World!"
        $result.afterState.output[1] | Should -BeExactly 4
    }

    It 'Test operation returns in desired state for <resourceType>' -TestCases $testCases {
        param($resourceType)

        $yaml = @'
        TestScript: |
            $true
'@
        $result = dsc resource test -r $resourceType -i $yaml 2> $TestDrive/error.txt | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.txt -Raw | Out-String)
        $result.InDesiredState | Should -BeTrue
    }

    It 'Test operation returns not in desired state for <resourceType>' -TestCases $testCases {
        param($resourceType)

        $yaml = @'
        TestScript: |
            $false
'@
        $result = dsc resource test -r $resourceType -i $yaml 2> $TestDrive/error.txt | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.txt -Raw | Out-String)
        $result.InDesiredState | Should -BeFalse
    }

    It 'Test operation returns error for non-boolean result for <resourceType>' -TestCases $testCases {
        param($resourceType)

        $yaml = @'
        TestScript: |
            "This is not a boolean"
'@
        $result = dsc resource test -r $resourceType -i $yaml 2> $TestDrive/error.txt | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 2 -Because (Get-Content $TestDrive/error.txt -Raw | Out-String)
        $result | Should -BeNullOrEmpty -Because "Test operation should return an error"
        (Get-Content $TestDrive/error.txt -Raw) | Should -BeLike '*Test operation did not return a single boolean value.*'
    }

    It 'Test operation returns error for multiple boolean results for <resourceType>' -TestCases $testCases {
        param($resourceType)

        $yaml = @'
        TestScript: |
            $true
            $false
'@
        $result = dsc resource test -r $resourceType -i $yaml 2> $TestDrive/error.txt | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 2 -Because (Get-Content $TestDrive/error.txt -Raw | Out-String)
        $result | Should -BeNullOrEmpty -Because "Test operation should return an error"
        (Get-Content $TestDrive/error.txt -Raw) | Should -BeLike '*Test operation did not return a single boolean value.*'
    }

    It 'Empty SetScript is ignored for <resourceType>' -TestCases $testCases {
        param($resourceType)

        $yaml = @'
        GetScript: |
          "Hello, World!"
          1+1
        TestScript: |
          $true
'@

        $result = dsc resource set -r $resourceType -i $yaml 2> $TestDrive/error.txt | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.txt -Raw | Out-String)
        $result.beforeState.output.Count | Should -Be 2 -Because ($result | ConvertTo-Json | Out-String)
        $result.beforeState.output[0] | Should -BeExactly "Hello, World!"
        $result.beforeState.output[1] | Should -BeExactly 2
        $result.afterState.output.Count | Should -Be 0
    }

    It 'Empty GetScript is ignored for <resourceType>' -TestCases $testCases {
        param($resourceType)

        $yaml = @'
        SetScript: |
          "Hello, World!"
          1+1
        TestScript: |
          $true
'@
        $result = dsc resource get -r $resourceType -i $yaml 2> $TestDrive/error.txt | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.txt -Raw | Out-String)
        $result.actualState.output.Count | Should -Be 0 -Because ($result | ConvertTo-Json | Out-String)
    }

    It 'Empty TestScript is ignored for <resourceType>' -TestCases $testCases {
        param($resourceType)

        $yaml = @'
        GetScript: |
          "Hello, World!"
          1+1
        SetScript: |
          "Hello, World!"
          2+2
'@
        $result = dsc resource test -r $resourceType -i $yaml 2> $TestDrive/error.txt | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.txt -Raw | Out-String)
        $result.InDesiredState | Should -BeTrue
    }

    It 'Write-Warning shows up as warn traces for <resourceType>' -TestCases $testCases {
        param($resourceType)

        $yaml = @'
        GetScript: |
          Write-Warning "This is a warning"
'@

        $result = dsc resource get -r $resourceType -i $yaml 2> $TestDrive/error.txt | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.txt -Raw | Out-String)
        $result.actualState.output.Count | Should -Be 0 -Because ($result | ConvertTo-Json | Out-String)
        (Get-Content $TestDrive/error.txt -Raw) | Should -BeLike '*WARN*:*This is a warning*'
    }

    It 'Write-Error shows up as error traces for <resourceType>' -TestCases $testCases {
        param($resourceType)

        $yaml = @'
        GetScript: |
          Write-Error "This is an error"
'@

        $result = dsc resource get -r $resourceType -i $yaml 2> $TestDrive/error.txt | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 2 -Because (Get-Content $TestDrive/error.txt -Raw | Out-String)
        $result.actualState.output.Count | Should -Be 0 -Because ($result | ConvertTo-Json | Out-String)
        (Get-Content $TestDrive/error.txt -Raw) | Should -BeLike '*ERROR*:*This is an error*'
    }

    It 'Write-Verbose shows up as info traces for <resourceType>' -TestCases $testCases {
        param($resourceType)

        $yaml = @'
        GetScript: |
          Write-Verbose "This is a verbose message"
'@
        $result = dsc -l info resource get -r $resourceType -i $yaml 2> $TestDrive/error.txt | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.txt -Raw | Out-String)
        $result.actualState.output.Count | Should -Be 0 -Because ($result | ConvertTo-Json | Out-String)
        (Get-Content $TestDrive/error.txt -Raw) | Should -BeLike '*INFO*:*This is a verbose message*'
    }

    It 'Write-Debug shows up as debug traces for <resourceType>' -TestCases $testCases {
        param($resourceType)

        $yaml = @'
        GetScript: |
          Write-Debug "This is a debug message"
'@
        $result = dsc -l debug resource get -r $resourceType -i $yaml 2> $TestDrive/error.txt | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.txt -Raw | Out-String)
        $result.actualState.output.Count | Should -Be 0 -Because ($result | ConvertTo-Json | Out-String)
        (Get-Content $TestDrive/error.txt -Raw) | Should -BeLike '*DEBUG*:*This is a debug message*'
    }

    It 'Write-Information shows up as trace traces for <resourceType>' -TestCases $testCases {
        param($resourceType)

        $yaml = @'
        GetScript: |
          $InformationPreference = 'Continue'
          Write-Information "This is an information message"
'@
        $result = dsc -l trace resource get -r $resourceType -i $yaml 2> $TestDrive/error.txt | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.txt -Raw | Out-String)
        $result.actualState.output.Count | Should -Be 0 -Because ($result | ConvertTo-Json | Out-String)
        (Get-Content $TestDrive/error.txt -Raw) | Should -BeLike '*TRACE*:*This is an information message*'
    }

    It 'A thrown exception results in an error for <resourceType>' -TestCases $testCases {
        param($resourceType)

        $yaml = @'
        GetScript: |
          throw "This is an exception"
'@
        $result = dsc resource get -r $resourceType -i $yaml 2> $TestDrive/error.txt | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 2 -Because (Get-Content $TestDrive/error.txt -Raw | Out-String)
        $result.actualState.output.Count | Should -Be 0 -Because ($result | ConvertTo-Json | Out-String)
        (Get-Content $TestDrive/error.txt -Raw) | Should -BeLike '*ERROR*:*This is an exception*'
    }

    It 'Sample config works' {
        $configPath = Join-Path $PSScriptRoot '../../dsc/examples/psscript.dsc.yaml'
        $result = dsc config get -f $configPath 2> $TestDrive/error.txt | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.txt -Raw | Out-String)
        $result.results[0].result.actualState.output.Count | Should -Be 1 -Because ($result | ConvertTo-Json -Depth 10 | Out-String)
        $result.results[0].result.actualState.output[0].PSEdition | Should -BeExactly 'Core'
        $result.results[0].result.actualState.output[0].PSVersion.Major | Should -Be 7
    }
}
