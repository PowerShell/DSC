# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Filesystem/AccessControlList tests' -Skip:(!$IsWindows) {
    BeforeDiscovery {
        $isElevated = if ($IsWindows) {
            ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
                [Security.Principal.WindowsBuiltInRole]::Administrator)
        } else {
            $false
        }
    }

    BeforeAll {
        $resourceType = 'Microsoft.Filesystem/AccessControlList'
        $currentSid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
        # Anonymous Logon's well-known SID; also exercises resolving an account name to a SID.
        $anonymousSid = 'S-1-5-7'

        function New-DesiredAcl {
            param(
                [string]$AnonymousAccessMask = 'read',
                [string]$CurrentUserAccessMask = 'fullControl',
                [bool]$Protected
            )

            $entries = @(
                @{ principal = 'ANONYMOUS LOGON'; type = 'deny'; accessMask = @($AnonymousAccessMask) }
                @{ principal = $currentSid; type = 'allow'; accessMask = @($CurrentUserAccessMask) }
            )
            $acl = @{ discretionary = $entries }
            if ($PSBoundParameters.ContainsKey('Protected')) {
                $acl.discretionaryProtected = $Protected
            }
            return $acl
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
        $json = @{ path = $path; accessControlList = (New-DesiredAcl) } | ConvertTo-Json -Depth 10 -Compress

        $setOutput = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $setState = ($setOutput | ConvertFrom-Json).afterState

        $getOutput = $json | dsc resource get -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $getState = ($getOutput | ConvertFrom-Json).actualState

        foreach ($state in @($setState, $getState)) {
            $discretionary = $state.accessControlList.discretionary
            $denyEntry = $discretionary | Where-Object { $_.type -eq 'deny' }
            $allowEntry = $discretionary | Where-Object { $_.type -eq 'allow' }

            # The resolved SID is returned even though an account name was supplied as input.
            $denyEntry.principal | Should -BeExactly $anonymousSid
            $denyEntry.accessMask | Should -Contain 'read'
            $allowEntry.principal | Should -BeExactly $currentSid
            $allowEntry.accessMask | Should -Contain 'fullControl'
        }
    }

    It 'Tests whether Allow and Deny ACEs are in the desired state' {
        $path = New-Item -ItemType File -Path (Join-Path $testRoot 'test.txt') |
            Select-Object -ExpandProperty FullName
        $json = @{ path = $path; accessControlList = (New-DesiredAcl) } | ConvertTo-Json -Depth 10 -Compress
        $null = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

        $testOutput = $json | dsc resource test -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

        ($testOutput | ConvertFrom-Json).inDesiredState | Should -BeTrue
    }

    It 'Reports a mismatched DACL as not in the desired state' {
        $path = New-Item -ItemType File -Path (Join-Path $testRoot 'mismatch.txt') |
            Select-Object -ExpandProperty FullName
        $json = @{ path = $path; accessControlList = (New-DesiredAcl) } | ConvertTo-Json -Depth 10 -Compress
        $null = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

        $mismatchedAcl = New-DesiredAcl -AnonymousAccessMask 'write'
        $mismatchedJson = @{ path = $path; accessControlList = $mismatchedAcl } |
            ConvertTo-Json -Depth 10 -Compress
        $testOutput = $mismatchedJson |
            dsc resource test -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

        ($testOutput | ConvertFrom-Json).inDesiredState | Should -BeFalse
    }

    It 'Applies and tests the DACL protection state' {
        $path = New-Item -ItemType File -Path (Join-Path $testRoot 'protected.txt') |
            Select-Object -ExpandProperty FullName
        $json = @{ path = $path; accessControlList = (New-DesiredAcl -Protected $true) } |
            ConvertTo-Json -Depth 10 -Compress

        $setOutput = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $setState = ($setOutput | ConvertFrom-Json).afterState

        $setState.accessControlList.discretionaryProtected | Should -BeTrue
        $testOutput = $json | dsc resource test -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        ($testOutput | ConvertFrom-Json).inDesiredState | Should -BeTrue
    }

    Context '_purge behavior' {
        It 'Replaces the discretionary section outright when _purge is omitted (default true)' {
            $path = New-Item -ItemType File -Path (Join-Path $testRoot 'replace.txt') |
                Select-Object -ExpandProperty FullName
            $json = @{ path = $path; accessControlList = (New-DesiredAcl) } |
                ConvertTo-Json -Depth 10 -Compress
            $null = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

            $replacementAcl = @{
                discretionary = @(
                    @{ principal = $currentSid; type = 'allow'; accessMask = @('read') }
                )
            }
            $replaceJson = @{ path = $path; accessControlList = $replacementAcl } |
                ConvertTo-Json -Depth 10 -Compress
            $setOutput = $replaceJson | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            $setState = ($setOutput | ConvertFrom-Json).afterState

            # Only the replacement entry remains; the deny entry from the original ACL is gone.
            @($setState.accessControlList.discretionary).Count | Should -Be 1
            $setState.accessControlList.discretionary[0].principal | Should -BeExactly $currentSid
            $setState.accessControlList.discretionary[0].accessMask | Should -Contain 'read'
        }

        It 'Merges new ACEs into the existing discretionary section when _purge is false' {
            $path = New-Item -ItemType File -Path (Join-Path $testRoot 'merge.txt') |
                Select-Object -ExpandProperty FullName
            $json = @{ path = $path; accessControlList = (New-DesiredAcl) } |
                ConvertTo-Json -Depth 10 -Compress
            $null = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

            $everyoneSid = 'S-1-1-0'
            $mergeAcl = @{
                discretionary = @(
                    @{ principal = $everyoneSid; type = 'allow'; accessMask = @('read') }
                )
                _purge        = $false
            }
            $mergeJson = @{ path = $path; accessControlList = $mergeAcl } |
                ConvertTo-Json -Depth 10 -Compress
            $setOutput = $mergeJson | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            $setState = ($setOutput | ConvertFrom-Json).afterState

            $principals = $setState.accessControlList.discretionary.principal
            $principals | Should -Contain $anonymousSid
            $principals | Should -Contain $currentSid
            $principals | Should -Contain $everyoneSid
            @($setState.accessControlList.discretionary).Count | Should -Be 3
        }

        It 'Does not duplicate a semantically equal ACE when merging' {
            $path = New-Item -ItemType File -Path (Join-Path $testRoot 'dedupe.txt') |
                Select-Object -ExpandProperty FullName
            $json = @{ path = $path; accessControlList = (New-DesiredAcl) } |
                ConvertTo-Json -Depth 10 -Compress
            $null = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

            # Same principal, type, and access mask as the existing allow entry: a semantic
            # duplicate that merging must not add a second copy of.
            $duplicateAcl = @{
                discretionary = @(
                    @{ principal = $currentSid; type = 'allow'; accessMask = @('fullControl') }
                )
                _purge        = $false
            }
            $duplicateJson = @{ path = $path; accessControlList = $duplicateAcl } |
                ConvertTo-Json -Depth 10 -Compress
            $setOutput = $duplicateJson | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            $setState = ($setOutput | ConvertFrom-Json).afterState

            @($setState.accessControlList.discretionary).Count | Should -Be 2
        }

        It 'Orders Deny ACEs before Allow ACEs after merging' {
            $path = New-Item -ItemType File -Path (Join-Path $testRoot 'order.txt') |
                Select-Object -ExpandProperty FullName
            # Start with only an Allow entry; no Deny entry exists yet.
            $initialAcl = @{
                discretionary = @(
                    @{ principal = $currentSid; type = 'allow'; accessMask = @('fullControl') }
                )
            }
            $json = @{ path = $path; accessControlList = $initialAcl } | ConvertTo-Json -Depth 10 -Compress
            $null = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

            $mergeAcl = @{
                discretionary = @(
                    @{ principal = 'ANONYMOUS LOGON'; type = 'deny'; accessMask = @('read') }
                )
                _purge        = $false
            }
            $mergeJson = @{ path = $path; accessControlList = $mergeAcl } |
                ConvertTo-Json -Depth 10 -Compress
            $setOutput = $mergeJson | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            $setState = ($setOutput | ConvertFrom-Json).afterState

            # The Deny entry added by the merge must be ordered before the pre-existing Allow entry.
            @($setState.accessControlList.discretionary)[0].type | Should -Be 'deny'
        }

        It 'Tests true with _purge false when extra existing ACEs are present' {
            $path = New-Item -ItemType File -Path (Join-Path $testRoot 'subset.txt') |
                Select-Object -ExpandProperty FullName
            $json = @{ path = $path; accessControlList = (New-DesiredAcl) } |
                ConvertTo-Json -Depth 10 -Compress
            $null = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

            $partialAcl = @{
                discretionary = @(
                    @{ principal = $currentSid; type = 'allow'; accessMask = @('fullControl') }
                )
                _purge        = $false
            }
            $testJson = @{ path = $path; accessControlList = $partialAcl } |
                ConvertTo-Json -Depth 10 -Compress
            $testOutput = $testJson | dsc resource test -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

            ($testOutput | ConvertFrom-Json).inDesiredState | Should -BeTrue
        }

        It 'Tests false with _purge false when a requested ACE is missing' {
            $path = New-Item -ItemType File -Path (Join-Path $testRoot 'missing.txt') |
                Select-Object -ExpandProperty FullName
            $json = @{ path = $path; accessControlList = (New-DesiredAcl) } |
                ConvertTo-Json -Depth 10 -Compress
            $null = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

            $missingAcl = @{
                discretionary = @(
                    @{ principal = 'S-1-1-0'; type = 'allow'; accessMask = @('read') }
                )
                _purge        = $false
            }
            $testJson = @{ path = $path; accessControlList = $missingAcl } |
                ConvertTo-Json -Depth 10 -Compress
            $testOutput = $testJson | dsc resource test -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

            ($testOutput | ConvertFrom-Json).inDesiredState | Should -BeFalse
        }

        It 'Tests true with _purge true using an alias-equivalent access mask' {
            $path = New-Item -ItemType File -Path (Join-Path $testRoot 'alias.txt') |
                Select-Object -ExpandProperty FullName
            # Individually, these rights are bit-for-bit identical to `fullControl`.
            $decomposedMask = @(
                'readData', 'writeData', 'appendData', 'readExtendedAttributes',
                'writeExtendedAttributes', 'executeFile', 'deleteChild', 'readAttributes',
                'writeAttributes', 'delete', 'readPermissions', 'changePermissions',
                'takeOwnership', 'synchronize'
            )
            $acl = @{
                discretionary          = @(
                    @{ principal = $currentSid; type = 'allow'; accessMask = $decomposedMask }
                )
                discretionaryProtected = $false
            }
            $json = @{ path = $path; accessControlList = $acl } | ConvertTo-Json -Depth 10 -Compress
            $null = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

            # A real system read always reports the most comprehensive alias that fits exactly,
            # so the actual entry now reports `fullControl`; a Test using the original,
            # decomposed list must still report satisfied under `_purge: true` (the default)
            # instead of comparing the access mask arrays for raw equality.
            $testOutput = $json | dsc resource test -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            ($testOutput | ConvertFrom-Json).inDesiredState | Should -BeTrue
        }

        It 'Defaults discretionaryProtected to false when purging and the property is omitted' {
            $path = New-Item -ItemType File -Path (Join-Path $testRoot 'purgeprotected.txt') |
                Select-Object -ExpandProperty FullName
            $protectedJson = @{ path = $path; accessControlList = (New-DesiredAcl -Protected $true) } |
                ConvertTo-Json -Depth 10 -Compress
            $null = $protectedJson | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

            # Purge (the default) again, without specifying discretionaryProtected this time.
            $unspecifiedAcl = @{ discretionary = (New-DesiredAcl).discretionary }
            $unspecifiedJson = @{ path = $path; accessControlList = $unspecifiedAcl } |
                ConvertTo-Json -Depth 10 -Compress
            $setOutput = $unspecifiedJson | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            $setState = ($setOutput | ConvertFrom-Json).afterState

            # Omitting the property during a purge write resets protection to false; it is not
            # left unchanged from the previously protected state.
            $setState.accessControlList.discretionaryProtected | Should -BeFalse
        }

        It 'Accepts a generic access mask value without rejecting or silently dropping it' {
            $path = New-Item -ItemType File -Path (Join-Path $testRoot 'generic.txt') |
                Select-Object -ExpandProperty FullName
            $acl = @{
                discretionary = @(
                    @{ principal = $currentSid; type = 'allow'; accessMask = @('genericAll') }
                )
            }
            $json = @{ path = $path; accessControlList = $acl } | ConvertTo-Json -Depth 10 -Compress
            $setOutput = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            $setState = ($setOutput | ConvertFrom-Json).afterState

            # NTFS maps `GENERIC_ALL` to the file-specific `fullControl`-equivalent rights when a
            # security descriptor is actually applied to a real file, so the mask read back
            # reports the mapped alias instead of the original raw generic bit. The important
            # thing this test guards is that the request is accepted at all (not rejected as an
            # unrepresentable access mask) and that the resulting access is the expected, full
            # equivalent -- not that the literal `genericAll` value survives the round trip.
            $setState.accessControlList.discretionary[0].accessMask | Should -Contain 'fullControl'

            $testOutput = $json | dsc resource test -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            ($testOutput | ConvertFrom-Json).inDesiredState | Should -BeTrue

            $acl._purge = $false
            $mergeJson = @{ path = $path; accessControlList = $acl } |
                ConvertTo-Json -Depth 10 -Compress
            $mergeOutput = $mergeJson |
                dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            (($mergeOutput | ConvertFrom-Json).afterState.accessControlList.discretionary |
                Where-Object principal -EQ $currentSid) | Should -HaveCount 1
        }
    }


    It 'Leaves the system access control list unread when it is not requested' {
        $path = New-Item -ItemType File -Path (Join-Path $testRoot 'partial.txt') |
            Select-Object -ExpandProperty FullName
        $json = @{ path = $path; accessControlList = (New-DesiredAcl) } | ConvertTo-Json -Depth 10 -Compress
        $null = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

        $getOutput = $json | dsc resource get -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $getState = ($getOutput | ConvertFrom-Json).actualState

        # Only the discretionary section was requested, so the system section must not be
        # reported even though the file has one by default.
        $getState.accessControlList.PSObject.Properties.Name | Should -Not -Contain 'system'
    }

    It 'Rejects an accessControlList with no discretionary or system section' {
        $path = New-Item -ItemType File -Path (Join-Path $testRoot 'invalid.txt') |
            Select-Object -ExpandProperty FullName
        $json = @{ path = $path; accessControlList = @{} } | ConvertTo-Json -Depth 10 -Compress

        $null = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Not -Be 0
        (Get-Content -Raw $TestDrive/error.log) |
            Should -Match 'discretionary and/or system'
    }

    It 'Rejects an allow entry placed in the system section' {
        $path = New-Item -ItemType File -Path (Join-Path $testRoot 'badtype.txt') |
            Select-Object -ExpandProperty FullName
        $acl = @{
            system = @(
                @{ principal = $currentSid; type = 'allow'; accessMask = @('read') }
            )
        }
        $json = @{ path = $path; accessControlList = $acl } | ConvertTo-Json -Depth 10 -Compress

        $null = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Not -Be 0
        (Get-Content -Raw $TestDrive/error.log) |
            Should -Match "system access control entries must have type 'audit'"
    }

    It 'Reports path inspection errors without claiming the path is absent' {
        $json = @{
            path              = "invalid$([char]0)path"
            accessControlList = (New-DesiredAcl)
        } | ConvertTo-Json -Depth 10 -Compress

        $null = $json | dsc resource get -r $resourceType -f - 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Not -Be 0
        (Get-Content -Raw $TestDrive/error.log) | Should -Match 'Failed to inspect the path'
        (Get-Content -Raw $TestDrive/error.log) | Should -Not -Match 'does not exist'
    }

    Context 'System access control list (SACL)' -Skip:(!$isElevated) {
        It 'Sets and gets Audit ACEs on a file' {
            $path = New-Item -ItemType File -Path (Join-Path $testRoot 'audit.txt') |
                Select-Object -ExpandProperty FullName
            $acl = @{
                system = @(
                    @{
                        principal  = $currentSid
                        type       = 'audit'
                        accessMask = @('read', 'write')
                        auditFlags = @('success', 'failure')
                    }
                )
            }
            $json = @{ path = $path; accessControlList = $acl } | ConvertTo-Json -Depth 10 -Compress

            $testOutput = $json | dsc resource test -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            ($testOutput | ConvertFrom-Json).inDesiredState | Should -BeFalse

            $setOutput = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            $setState = ($setOutput | ConvertFrom-Json).afterState

            $getOutput = $json | dsc resource get -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            $getState = ($getOutput | ConvertFrom-Json).actualState

            foreach ($state in @($setState, $getState)) {
                $auditEntry = $state.accessControlList.system | Select-Object -First 1
                $auditEntry.principal | Should -BeExactly $currentSid
                $auditEntry.auditFlags | Should -Contain 'success'
                $auditEntry.auditFlags | Should -Contain 'failure'
            }
        }
    }
}
