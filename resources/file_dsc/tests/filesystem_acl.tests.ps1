# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Filesystem/ACL tests' -Skip:(!$IsWindows) {
    BeforeAll {
        $resourceType = 'Microsoft.Filesystem/ACL'
        $currentSid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
        $desiredSddl = "D:(D;;FR;;;AN)(A;;FA;;;$currentSid)"

        function ConvertTo-CanonicalSddl {
            param(
                [Parameter(Mandatory)]
                [string]$Sddl
            )

            return [Security.AccessControl.RawSecurityDescriptor]::new($Sddl).GetSddlForm('Access')
        }
    }

    BeforeEach {
        $testRoot = Join-Path $TestDrive "$([System.Guid]::NewGuid())"
        $null = New-Item -ItemType Directory -Path $testRoot
    }

    AfterEach {
        Remove-Item -LiteralPath $testRoot -Recurse -Force -ErrorAction Ignore
    }

    It 'Sets and gets Allow and Deny ACEs on a <itemType>' -ForEach @(
        @{ itemType = 'file' }
        @{ itemType = 'folder' }
    ) {
        $path = if ($itemType -eq 'file') {
            New-Item -ItemType File -Path (Join-Path $testRoot 'item.txt') |
                Select-Object -ExpandProperty FullName
        } else {
            New-Item -ItemType Directory -Path (Join-Path $testRoot 'folder') |
                Select-Object -ExpandProperty FullName
        }
        $json = @{ path = $path; SDDL = $desiredSddl } | ConvertTo-Json -Compress

        $setOutput = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $setState = ($setOutput | ConvertFrom-Json).afterState

        $getOutput = $json | dsc resource get -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $getState = ($getOutput | ConvertFrom-Json).actualState

        (ConvertTo-CanonicalSddl -Sddl $setState.SDDL) |
            Should -BeExactly (ConvertTo-CanonicalSddl -Sddl $desiredSddl)
        (ConvertTo-CanonicalSddl -Sddl $getState.SDDL) |
            Should -BeExactly (ConvertTo-CanonicalSddl -Sddl $desiredSddl)
        $getState.SDDL | Should -Match '\(D;'
        $getState.SDDL | Should -Match '\(A;'
    }

    It 'Tests whether Allow and Deny ACEs are in the desired state' {
        $path = New-Item -ItemType File -Path (Join-Path $testRoot 'test.txt') |
            Select-Object -ExpandProperty FullName
        $json = @{ path = $path; SDDL = $desiredSddl } | ConvertTo-Json -Compress
        $null = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

        $testOutput = $json | dsc resource test -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

        ($testOutput | ConvertFrom-Json).inDesiredState | Should -BeTrue
    }

    It 'Reports a mismatched valid DACL as not in the desired state' {
        $path = New-Item -ItemType File -Path (Join-Path $testRoot 'mismatch.txt') |
            Select-Object -ExpandProperty FullName
        $json = @{ path = $path; SDDL = $desiredSddl } | ConvertTo-Json -Compress
        $null = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

        $mismatchedJson = @{
            path = $path
            SDDL = "D:(D;;FW;;;AN)(A;;FA;;;$currentSid)"
        } | ConvertTo-Json -Compress
        $testOutput = $mismatchedJson |
            dsc resource test -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

        ($testOutput | ConvertFrom-Json).inDesiredState | Should -BeFalse
    }

    It 'Applies and tests the DACL protection state' {
        $path = New-Item -ItemType File -Path (Join-Path $testRoot 'protected.txt') |
            Select-Object -ExpandProperty FullName
        $protectedSddl = $desiredSddl.Replace('D:', 'D:P')
        $json = @{ path = $path; SDDL = $protectedSddl } | ConvertTo-Json -Compress

        $setOutput = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $setState = ($setOutput | ConvertFrom-Json).afterState

        $setState.SDDL | Should -Match '^D:P'
        $testOutput = $json | dsc resource test -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        ($testOutput | ConvertFrom-Json).inDesiredState | Should -BeTrue
    }

    It 'Rejects SDDL components that the resource does not manage' {
        $path = New-Item -ItemType File -Path (Join-Path $testRoot 'invalid.txt') |
            Select-Object -ExpandProperty FullName
        $json = @{ path = $path; SDDL = "O:BAD:(A;;FA;;;$currentSid)" } |
            ConvertTo-Json -Compress

        $null = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Not -Be 0
        (Get-Content -Raw $TestDrive/error.log) |
            Should -Match 'must contain only a discretionary ACL'
    }
}
