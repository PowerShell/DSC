# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'User rights assignment export tests' -Skip:(!$IsWindows) {
    BeforeDiscovery {
        $isAdmin = if ($IsWindows) {
            $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
            $principal = [Security.Principal.WindowsPrincipal]$identity
            $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
        }
        else {
            $false
        }
    }

    BeforeAll {
        $resourceType = 'Microsoft.Windows/UserRightsAssignment'
    }

    Context 'Export all rights' -Skip:(!$isAdmin) {
        BeforeAll {
            $raw = dsc resource export -r $resourceType 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $script:export = $raw | ConvertFrom-Json
        }

        It 'Returns every user right exactly once' {
            $script:export.resources.Count | Should -Be 45
            $rights = @($script:export.resources.properties.right)
            @($rights | Select-Object -Unique).Count | Should -Be 45
            foreach ($right in $rights) {
                $right | Should -Match '^Se\w+(Privilege|Right)$'
            }
            $rights[0] | Should -BeExactly 'SeTrustedCredManAccessPrivilege'
            $rights[-1] | Should -BeExactly 'SeTakeOwnershipPrivilege'
        }

        It 'Returns complete entries with the correct type' {
            foreach ($resource in $script:export.resources) {
                $resource.type | Should -BeExactly $resourceType
                $resource.properties.PSObject.Properties.Name | Should -Contain 'identity'
                foreach ($name in @($resource.properties.identity)) {
                    $name | Should -Not -BeNullOrEmpty
                }
            }
        }

        It 'Reports Everyone for bypass traverse checking' {
            $entry = $script:export.resources | Where-Object { $_.properties.right -eq 'SeChangeNotifyPrivilege' }
            @($entry.properties.identity) | Should -Contain 'Everyone'
        }

        It 'Exported entries round-trip through get' {
            foreach ($resource in $script:export.resources | Select-Object -First 3) {
                $json = @{ right = $resource.properties.right } | ConvertTo-Json -Compress
                $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
                $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
                $actual = ($out | ConvertFrom-Json).actualState
                @($actual.identity) | Should -Be @($resource.properties.identity)
            }
        }
    }
}
