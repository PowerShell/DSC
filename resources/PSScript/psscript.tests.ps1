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
}
