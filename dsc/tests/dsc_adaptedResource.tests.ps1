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
}
