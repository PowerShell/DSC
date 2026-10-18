# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'registry config test tests' {
    AfterEach {
        if ($IsWindows) {
            Remove-Item -Path 'HKCU:\1' -Recurse -ErrorAction Ignore
        }
    }

    It 'Reports in desired state when the value matches' -Skip:(!$IsWindows) {
        $json = @{
            keyPath   = 'HKCU\1\TestValue'
            valueName = 'Hello'
            valueData = @{ String = 'World' }
        } | ConvertTo-Json -Depth 10 -Compress
        registry config set --input $json 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        $result = registry config test --input $json 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        $result._inDesiredState | Should -BeTrue
    }

    It 'Reports not in desired state when the value differs' -Skip:(!$IsWindows) {
        $setJson = @{
            keyPath   = 'HKCU\1\TestValueMismatch'
            valueName = 'Hello'
            valueData = @{ String = 'World' }
        } | ConvertTo-Json -Depth 10 -Compress
        registry config set --input $setJson 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        $testJson = @{
            keyPath   = 'HKCU\1\TestValueMismatch'
            valueName = 'Hello'
            valueData = @{ String = 'Different' }
        } | ConvertTo-Json -Depth 10 -Compress
        $result = registry config test --input $testJson 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        $result._inDesiredState | Should -BeFalse
    }

    It 'Reports not in desired state when the key does not exist' -Skip:(!$IsWindows) {
        $json = @{
            keyPath = 'HKCU\1\TestDoesNotExist'
        } | ConvertTo-Json -Depth 10 -Compress

        $result = registry config test --input $json 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        $result._inDesiredState | Should -BeFalse
    }

    It 'Reports in desired state for an alias-equivalent access control list under _purge:true' -Skip:(!$IsWindows) {
        New-Item -Path 'HKCU:\1\TestAclAlias' -Force | Out-Null

        # Individually, these rights are bit-for-bit identical to `fullControl`.
        $decomposedMask = @(
            'queryValue', 'setValue', 'createSubKey', 'enumerateSubKeys', 'notify',
            'createLink', 'delete', 'readPermissions', 'changePermissions', 'takeOwnership'
        )
        $json = @{
            keyPath           = 'HKCU\1\TestAclAlias'
            accessControlList = @{
                discretionary          = @(
                    @{ principal = $env:USERNAME; type = 'allow'; accessMask = $decomposedMask }
                )
                discretionaryProtected = $false
            }
        } | ConvertTo-Json -Depth 10 -Compress
        registry config set --input $json 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        # A real system read always reports the most comprehensive alias that fits exactly, so
        # the actual entry now reports `fullControl`; a Test using the original, decomposed list
        # must still report satisfied under `_purge: true` (the default) instead of comparing the
        # access mask arrays for raw equality.
        $result = registry config test --input $json 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        $result._inDesiredState | Should -BeTrue
    }

    It 'Reports in desired state with _purge:false when the requested ACE already exists' -Skip:(!$IsWindows) {
        New-Item -Path 'HKCU:\1\TestAclMerge' -Force | Out-Null
        $setJson = @{
            keyPath           = 'HKCU\1\TestAclMerge'
            accessControlList = @{
                discretionary = @(
                    @{ principal = $env:USERNAME; type = 'allow'; accessMask = @('fullControl') }
                )
            }
        } | ConvertTo-Json -Depth 10 -Compress
        registry config set --input $setJson 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        $testJson = @{
            keyPath           = 'HKCU\1\TestAclMerge'
            accessControlList = @{
                discretionary = @(
                    @{ principal = $env:USERNAME; type = 'allow'; accessMask = @('fullControl') }
                )
                _purge        = $false
            }
        } | ConvertTo-Json -Depth 10 -Compress
        $result = registry config test --input $testJson 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        $result._inDesiredState | Should -BeTrue
    }

    It 'Rejects an access control list request that specifies neither discretionary nor system' -Skip:(!$IsWindows) {
        $json = @{
            keyPath           = 'HKCU\1\TestAclEmpty'
            accessControlList = @{}
        } | ConvertTo-Json -Depth 10 -Compress

        registry config test --input $json 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Not -Be 0
        (Get-Content -Path $TestDrive/error.log -Raw) | Should -Match 'Registry ACL'
    }

    It 'Can test a registry list and aggregates the in-desired-state result' -Skip:(!$IsWindows) {
        $setJson = @{
            registryEntries = @(
                @{ keyPath = 'HKCU\1\TestListA'; valueName = 'Hello'; valueData = @{ String = 'World' } }
                @{ keyPath = 'HKCU\1\TestListB'; valueName = 'Hello'; valueData = @{ String = 'World' } }
            )
        } | ConvertTo-Json -Depth 10 -Compress
        registry config set --list --input $setJson 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        $matchingTestJson = $setJson
        $result = registry config test --list --input $matchingTestJson 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        $result._inDesiredState | Should -BeTrue
        $result.registryEntries[0]._inDesiredState | Should -BeTrue
        $result.registryEntries[1]._inDesiredState | Should -BeTrue

        $mismatchTestJson = @{
            registryEntries = @(
                @{ keyPath = 'HKCU\1\TestListA'; valueName = 'Hello'; valueData = @{ String = 'World' } }
                @{ keyPath = 'HKCU\1\TestListB'; valueName = 'Hello'; valueData = @{ String = 'Different' } }
            )
        } | ConvertTo-Json -Depth 10 -Compress
        $mismatchResult = registry config test --list --input $mismatchTestJson 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        $mismatchResult._inDesiredState | Should -BeFalse
        $mismatchResult.registryEntries[0]._inDesiredState | Should -BeTrue
        $mismatchResult.registryEntries[1]._inDesiredState | Should -BeFalse
    }
}
