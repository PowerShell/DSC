# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'registry config set tests' {
    AfterEach {
        if ($IsWindows) {
            Remove-Item -Path 'HKCU:\1' -Recurse -ErrorAction Ignore
        }
    }

    It 'Can set a deeply nested key and value' -Skip:(!$IsWindows) {
        $json = @'
        {
            "keyPath": "HKCU\\1\\2\\3",
            "valueName": "Hello",
            "valueData": {
                "String": "World"
            }
        }
'@
        $out = registry config set --input $json 2>$null
        $LASTEXITCODE | Should -Be 0
        $out | Should -BeNullOrEmpty
        $result = registry config get --input $json 2>$null | ConvertFrom-Json
        $result.keyPath | Should -Be 'HKCU\1\2\3'
        $result.valueName | Should -Be 'Hello'
        $result.valueData.String | Should -Be 'World'
        ($result.psobject.properties | Measure-Object).Count | Should -Be 3

        $out = registry config get --input $json 2>$null
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.keyPath | Should -Be 'HKCU\1\2\3'
        $result.valueName | Should -Be 'Hello'
        $result.valueData.String | Should -Be 'World'
        ($result.psobject.properties | Measure-Object).Count | Should -Be 3
    }

    It 'delete called when _exist is false' -Skip:(!$IsWindows) {
        $config = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{
                    name = 'reg'
                    type = 'Microsoft.Windows/Registry'
                    properties = @{
                        keyPath = 'HKCU\1\2'
                        valueName = 'Test'
                        valueData = @{
                            String = 'Test'
                        }
                        _exist = $true
                    }
                }
            )
        }

        $out = dsc config set -i ($config | ConvertTo-Json -Depth 10)
        $LASTEXITCODE | Should -Be 0

        $config.resources[0].properties._exist = $false
        $out = dsc config set -i ($config | ConvertTo-Json -Depth 10) | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.afterState._exist | Should -Be $false

        Get-ItemProperty -Path 'HKCU:\1\2' -Name 'Test' -ErrorAction Ignore | Should -BeNullOrEmpty

        $config.resources[0].properties.valueName = $null
        $out = dsc config set -i ($config | ConvertTo-Json -Depth 10) | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.afterState._exist | Should -Be $false

        Get-Item -Path 'HKCU:\1\2' -ErrorAction Ignore | Should -BeNullOrEmpty
    }

    It 'Can set value without data' -Skip:(!$IsWindows) {
        $configYaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Key
              type: Microsoft.Windows/Registry
              properties:
                keyPath: 'HKCU\1'
                valueName: Test
                _exist: true
'@

        $out = dsc config set -i $configYaml | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.afterState.keyPath | Should -BeExactly 'HKCU\1'
        $out.results[0].result.afterState.valueName | Should -BeExactly 'Test'
        $out.results[0].result.afterState.valueData | Should -BeNullOrEmpty

        $out = dsc config get -i $configYaml | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.keyPath | Should -BeExactly 'HKCU\1'
        $out.results[0].result.actualState.valueName | Should -BeExactly 'Test'
        $out.results[0].result.actualState.valueData | Should -BeNullOrEmpty

        Remove-Item -Path 'HKCU:\1' -Recurse -ErrorAction Ignore
    }

    It 'Can set and retrieve a registry key access control list' -Skip:(!$IsWindows) {
        New-Item -Path 'HKCU:\1\AclSet' -Force | Out-Null
        $aclRequest = @{
            keyPath = 'HKCU\1\AclSet'
            accessControlList = @{
                discretionary          = @(
                    @{
                        principal  = $env:USERNAME
                        type       = 'allow'
                        accessMask = @('fullControl')
                    }
                )
                discretionaryProtected = $true
            }
        }
        $json = $aclRequest | ConvertTo-Json -Depth 10 -Compress

        $output = registry config set --input $json 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        $output | Should -BeNullOrEmpty

        $getJson = @{
            keyPath = 'HKCU\1\AclSet'
            accessControlList = @{ discretionary = @() }
        } | ConvertTo-Json -Depth 10 -Compress
        $result = registry config get --input $getJson 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        $result.accessControlList.discretionaryProtected | Should -BeTrue
        $result.accessControlList.discretionary.Count | Should -Be 1
        $result.accessControlList.discretionary[0].type | Should -BeExactly 'allow'
        $result.accessControlList.discretionary[0].accessMask | Should -Contain 'fullControl'

        $aclRequest.accessControlList.discretionaryProtected = $false
        $json = $aclRequest | ConvertTo-Json -Depth 10 -Compress

        registry config set --input $json 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        $result = registry config get --input $getJson 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        $result.accessControlList.discretionaryProtected | Should -BeFalse
    }

    It 'Replaces the discretionary section by default when _purge is omitted' -Skip:(!$IsWindows) {
        # Anonymous Logon's well-known SID. It never matches the test process's own token, so an
        # Allow/Deny entry for it can't accidentally lock the test process out of the key.
        $anonymousSid = 'S-1-5-7'

        New-Item -Path 'HKCU:\1\AclPurgeDefault' -Force | Out-Null
        $initial = @{
            keyPath           = 'HKCU\1\AclPurgeDefault'
            accessControlList = @{
                discretionary = @(
                    @{ principal = $env:USERNAME; type = 'allow'; accessMask = @('fullControl') }
                    @{ principal = $anonymousSid; type = 'deny'; accessMask = @('read') }
                )
            }
        } | ConvertTo-Json -Depth 10 -Compress
        registry config set --input $initial 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        # The replacement keeps the current user's own entry (so the key stays manageable) but
        # omits the anonymous deny entry; with _purge omitted (default true) that entry must go.
        $replace = @{
            keyPath           = 'HKCU\1\AclPurgeDefault'
            accessControlList = @{
                discretionary = @(
                    @{ principal = $env:USERNAME; type = 'allow'; accessMask = @('fullControl') }
                )
            }
        } | ConvertTo-Json -Depth 10 -Compress
        registry config set --input $replace 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        $getJson = @{
            keyPath           = 'HKCU\1\AclPurgeDefault'
            accessControlList = @{ discretionary = @() }
        } | ConvertTo-Json -Depth 10 -Compress
        $result = registry config get --input $getJson 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        # The second set replaced the discretionary section outright, so the anonymous deny
        # entry from the original ACL is gone.
        $result.accessControlList.discretionary.Count | Should -Be 1
        $result.accessControlList.discretionary[0].principal | Should -BeExactly ([Security.Principal.NTAccount]::new($env:USERNAME)).Translate([Security.Principal.SecurityIdentifier]).Value
    }

    It 'Merges new ACEs into the existing discretionary section when _purge is false' -Skip:(!$IsWindows) {
        $anonymousSid = 'S-1-5-7'

        New-Item -Path 'HKCU:\1\AclPurgeMerge' -Force | Out-Null
        $initial = @{
            keyPath           = 'HKCU\1\AclPurgeMerge'
            accessControlList = @{
                discretionary = @(
                    @{ principal = $env:USERNAME; type = 'allow'; accessMask = @('fullControl') }
                )
            }
        } | ConvertTo-Json -Depth 10 -Compress
        registry config set --input $initial 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        $merge = @{
            keyPath           = 'HKCU\1\AclPurgeMerge'
            accessControlList = @{
                discretionary = @(
                    @{ principal = $anonymousSid; type = 'allow'; accessMask = @('read') }
                )
                _purge        = $false
            }
        } | ConvertTo-Json -Depth 10 -Compress
        registry config set --input $merge 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        $getJson = @{
            keyPath           = 'HKCU\1\AclPurgeMerge'
            accessControlList = @{ discretionary = @() }
        } | ConvertTo-Json -Depth 10 -Compress
        $result = registry config get --input $getJson 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        # Both the original entry and the newly merged entry are present.
        $result.accessControlList.discretionary.principal | Should -Contain $anonymousSid
        $result.accessControlList.discretionary.Count | Should -Be 2
    }

    It 'Does not duplicate a semantically equal ACE when merging' -Skip:(!$IsWindows) {
        New-Item -Path 'HKCU:\1\AclPurgeDedupe' -Force | Out-Null
        $initial = @{
            keyPath           = 'HKCU\1\AclPurgeDedupe'
            accessControlList = @{
                discretionary = @(
                    @{ principal = $env:USERNAME; type = 'allow'; accessMask = @('fullControl') }
                )
            }
        } | ConvertTo-Json -Depth 10 -Compress
        registry config set --input $initial 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        # Same principal, type, and access mask as the existing allow entry: a semantic
        # duplicate that merging must not add a second copy of.
        $merge = @{
            keyPath           = 'HKCU\1\AclPurgeDedupe'
            accessControlList = @{
                discretionary = @(
                    @{ principal = $env:USERNAME; type = 'allow'; accessMask = @('fullControl') }
                )
                _purge        = $false
            }
        } | ConvertTo-Json -Depth 10 -Compress
        registry config set --input $merge 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        $getJson = @{
            keyPath           = 'HKCU\1\AclPurgeDedupe'
            accessControlList = @{ discretionary = @() }
        } | ConvertTo-Json -Depth 10 -Compress
        $result = registry config get --input $getJson 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        $result.accessControlList.discretionary.Count | Should -Be 1
    }

    It 'Orders Deny ACEs before Allow ACEs after merging' -Skip:(!$IsWindows) {
        # Anonymous Logon's well-known SID. A Deny entry for it never applies to the test
        # process's own token, so merging it in can't lock the test process out of the key.
        $anonymousSid = 'S-1-5-7'

        New-Item -Path 'HKCU:\1\AclPurgeOrder' -Force | Out-Null
        $initial = @{
            keyPath           = 'HKCU\1\AclPurgeOrder'
            accessControlList = @{
                discretionary = @(
                    @{ principal = $env:USERNAME; type = 'allow'; accessMask = @('fullControl') }
                )
            }
        } | ConvertTo-Json -Depth 10 -Compress
        registry config set --input $initial 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        $merge = @{
            keyPath           = 'HKCU\1\AclPurgeOrder'
            accessControlList = @{
                discretionary = @(
                    @{ principal = $anonymousSid; type = 'deny'; accessMask = @('read') }
                )
                _purge        = $false
            }
        } | ConvertTo-Json -Depth 10 -Compress
        registry config set --input $merge 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        $getJson = @{
            keyPath           = 'HKCU\1\AclPurgeOrder'
            accessControlList = @{ discretionary = @() }
        } | ConvertTo-Json -Depth 10 -Compress
        $result = registry config get --input $getJson 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        # The newly merged Deny entry must be ordered before the pre-existing Allow entry.
        $result.accessControlList.discretionary[0].type | Should -BeExactly 'deny'
    }

    It 'Rejects an access control list request that specifies neither discretionary nor system' -Skip:(!$IsWindows) {
        $json = @{
            keyPath           = 'HKCU\1\AclEmptySet'
            accessControlList = @{}
        } | ConvertTo-Json -Depth 10 -Compress

        registry config set --input $json 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Not -Be 0
        (Get-Content -Path $TestDrive/error.log -Raw) | Should -Match 'Registry ACL'
        Get-Item -Path 'HKCU:\1\AclEmptySet' -ErrorAction Ignore | Should -BeNullOrEmpty
    }

    It 'Rejects the removed wow6464Key/wow6432Key access mask values' -Skip:(!$IsWindows) {
        $json = @{
            keyPath           = 'HKCU\1\AclWow64Set'
            accessControlList = @{
                discretionary = @(
                    @{ principal = $env:USERNAME; type = 'allow'; accessMask = @('wow6464Key') }
                )
            }
        } | ConvertTo-Json -Depth 10 -Compress

        registry config set --input $json 2>$TestDrive/error.log

        # `wow6464Key`/`wow6432Key` are registry view selectors, not ACE permissions, and are no
        # longer valid accessMask values.
        $LASTEXITCODE | Should -Not -Be 0
        Get-Item -Path 'HKCU:\1\AclWow64Set' -ErrorAction Ignore | Should -BeNullOrEmpty
    }

    It 'Defaults discretionaryProtected to false when purging and the property is omitted' -Skip:(!$IsWindows) {
        New-Item -Path 'HKCU:\1\AclPurgeProtectedDefault' -Force | Out-Null
        $protectedRequest = @{
            keyPath           = 'HKCU\1\AclPurgeProtectedDefault'
            accessControlList = @{
                discretionary          = @(
                    @{ principal = $env:USERNAME; type = 'allow'; accessMask = @('fullControl') }
                )
                discretionaryProtected = $true
            }
        } | ConvertTo-Json -Depth 10 -Compress
        registry config set --input $protectedRequest 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        # Purge (the default) again, this time without specifying discretionaryProtected.
        $unspecifiedRequest = @{
            keyPath           = 'HKCU\1\AclPurgeProtectedDefault'
            accessControlList = @{
                discretionary = @(
                    @{ principal = $env:USERNAME; type = 'allow'; accessMask = @('fullControl') }
                )
            }
        } | ConvertTo-Json -Depth 10 -Compress
        registry config set --input $unspecifiedRequest 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        $getJson = @{
            keyPath           = 'HKCU\1\AclPurgeProtectedDefault'
            accessControlList = @{ discretionary = @() }
        } | ConvertTo-Json -Depth 10 -Compress
        $result = registry config get --input $getJson 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        # Omitting the property during a purge write resets protection to false; it is not left
        # unchanged from the previously protected state.
        $result.accessControlList.discretionaryProtected | Should -BeFalse
    }

    It 'Should succeed when _exist is false and value does not exist' -Skip:(!$IsWindows) {
        $config = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{
                    name = 'reg'
                    type = 'Microsoft.Windows/Registry'
                    properties = @{
                        keyPath = 'HKCU'
                        valueName = 'Test'
                        valueData = @{
                            String = 'Test'
                        }
                        _exist = $false
                    }
                }
            )
        }

        $out = dsc config set -i ($config | ConvertTo-Json -Depth 10) | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.afterState._exist | Should -Be $false

        Get-ItemProperty -Path 'HKCU:\1\2' -Name 'Test' -ErrorAction Ignore | Should -BeNullOrEmpty
    }

    It 'Should succeed when _exist is false and key does not exist' -Skip:(!$IsWindows) {
        $config = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{
                    name = 'reg'
                    type = 'Microsoft.Windows/Registry'
                    properties = @{
                        keyPath = 'HKCU\1'
                        _exist = $false
                    }
                }
            )
        }

        $out = dsc config set -i ($config | ConvertTo-Json -Depth 10) | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.afterState._exist | Should -Be $false
    }

    It 'Can delete value from system-protected key with minimal permissions' -Skip:(!$IsWindows) {
        $testKeyPath = 'HKLM:\Software\Policies\Microsoft\Windows\Appx'
        if (-not (Test-Path $testKeyPath)) {
            Set-ItResult -Skipped -Because "Test key path '$testKeyPath' does not exist"
            return
        }

        $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
        $isElevated = $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
        
        if (-not $isElevated) {
            Set-ItResult -Skipped -Because "Test requires elevated privileges"
            return
        }

        $setJson = @'
        {
            "keyPath": "HKEY_LOCAL_MACHINE\\Software\\Policies\\Microsoft\\Windows\\Appx",
            "valueName": "DSCTestValue",
            "valueData": {
                "String": "TestData"
            }
        }
'@
        $out = registry config set --input $setJson 2>$null
        $LASTEXITCODE | Should -Be 0

        $getJson = @'
        {
            "keyPath": "HKEY_LOCAL_MACHINE\\Software\\Policies\\Microsoft\\Windows\\Appx",
            "valueName": "DSCTestValue"
        }
'@
        $result = registry config get --input $getJson 2>$null | ConvertFrom-Json
        $result.valueName | Should -Be 'DSCTestValue'
        $result.valueData.String | Should -Be 'TestData'

        $deleteJson = @'
        {
            "keyPath": "HKEY_LOCAL_MACHINE\\Software\\Policies\\Microsoft\\Windows\\Appx",
            "valueName": "DSCTestValue"
        }
'@
        $out = registry config delete --input $deleteJson 2>$null
        $LASTEXITCODE | Should -Be 0

        $result = registry config get --input $getJson 2>$null | ConvertFrom-Json
        $result._exist | Should -Be $false
        $result.valueData | Should -BeNullOrEmpty
    }

    It 'Can whatif-set a system ACL on an existing key without a privilege error' -Skip:(!$IsWindows) {
        # Requesting a `system` section means the resource must open the key with
        # ACCESS_SYSTEM_SECURITY -- including during the `can_set` check that a what-if preview
        # performs -- which itself requires the SeSecurityPrivilege privilege to already be
        # enabled. If the privilege weren't enabled before that check, this would fail with a
        # privilege error instead of returning a preview, even for a user who otherwise holds the
        # privilege (for example an Administrator who hasn't elevated).
        $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
        $isElevated = $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

        if (-not $isElevated) {
            Set-ItResult -Skipped -Because 'Test requires elevated privileges'
            return
        }

        New-Item -Path 'HKCU:\1\AclSaclWhatIf' -Force | Out-Null
        $json = @{
            keyPath           = 'HKCU\1\AclSaclWhatIf'
            accessControlList = @{
                system = @(
                    @{
                        principal  = $env:USERNAME
                        type       = 'audit'
                        accessMask = @('read')
                        auditFlags = @('success', 'failure')
                    }
                )
            }
        } | ConvertTo-Json -Depth 10 -Compress

        $output = registry config set -w --input $json 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        (Get-Content -Path $TestDrive/error.log -Raw) | Should -Not -Match 'SeSecurityPrivilege'

        $result = $output | ConvertFrom-Json
        $result._metadata.whatIf | Should -Contain "Would set ACL on key 'HKCU\1\AclSaclWhatIf'"
    }

    It 'Round-trips a system ACL without changing the discretionary ACL' -Skip:(!$IsWindows) {
        $principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
        if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
            Set-ItResult -Skipped -Because 'Test requires elevated privileges'
            return
        }

        $keyPath = 'HKCU:\1\AclSaclRoundTrip'
        $null = New-Item -Path $keyPath -Force
        $daclBefore = (Get-Acl -Path $keyPath).GetSecurityDescriptorSddlForm(
            [Security.AccessControl.AccessControlSections]::Access)
        $json = @{
            keyPath           = 'HKCU\1\AclSaclRoundTrip'
            accessControlList = @{
                system          = @(
                    @{
                        principal  = $env:USERNAME
                        type       = 'audit'
                        accessMask = @('read')
                        auditFlags = @('success', 'failure')
                    }
                )
                systemProtected = $true
            }
        } | ConvertTo-Json -Depth 10 -Compress

        registry config set --input $json 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        $actual = registry config get --input $json 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        $tested = registry config test --input $json 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        $actual.accessControlList.systemProtected | Should -BeTrue
        $actual.accessControlList.system[0].auditFlags | Should -Contain 'success'
        $actual.accessControlList.system[0].auditFlags | Should -Contain 'failure'
        $tested._inDesiredState | Should -BeTrue
        $daclAfter = (Get-Acl -Path $keyPath).GetSecurityDescriptorSddlForm(
            [Security.AccessControl.AccessControlSections]::Access)
        $daclAfter | Should -BeExactly $daclBefore
    }

    It 'Validates ACL principals before changing a registry value' -Skip:(!$IsWindows) {
        $keyPath = 'HKCU:\1\AclPreflight'
        $null = New-Item -Path $keyPath -Force
        Set-ItemProperty -Path $keyPath -Name 'Data' -Value 'original'
        $missingAccount = "DSC-Missing-$([guid]::NewGuid())"
        $json = @{
            keyPath           = 'HKCU\1\AclPreflight'
            valueName         = 'Data'
            valueData         = @{ String = 'changed' }
            accessControlList = @{
                discretionary = @(
                    @{ principal = $missingAccount; type = 'allow'; accessMask = @('read') }
                )
            }
        } | ConvertTo-Json -Depth 10 -Compress

        registry config set --input $json 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Not -Be 0
        (Get-ItemPropertyValue -Path $keyPath -Name 'Data') | Should -BeExactly 'original'
    }
}
