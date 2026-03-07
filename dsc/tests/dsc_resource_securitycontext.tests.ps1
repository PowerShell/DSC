# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for resource manifest security context' {
    BeforeAll {
        $isAdmin = if ($IsWindows) {
            $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
            [System.Security.Principal.WindowsPrincipal]::new($identity).IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)
        }
        else {
            [System.Environment]::UserName -eq 'root'
        }
    }

    It 'Resource with <securityContext> security context for operation <operation>' -TestCases @(
        # since `set` and `test` rely on `get` to retrieve the current state, we need to always allow that
        # and have a separate resource to test the elevated and restricted contexts for get
        @{ securityContext = 'Elevated'; operation = 'get'; property = 'actualState'; type = 'Test/SecurityContextElevatedGet' },
        @{ securityContext = 'Elevated'; operation = 'set'; property = 'afterState' },
        @{ securityContext = 'Elevated'; operation = 'delete' },
        @{ securityContext = 'Elevated'; operation = 'test'; property = 'actualState' },
        @{ securityContext = 'Elevated'; operation = 'export' },
        @{ securityContext = 'Restricted'; operation = 'get'; property = 'actualState'; type = 'Test/SecurityContextRestrictedGet' },
        @{ securityContext = 'Restricted'; operation = 'set'; property = 'afterState' },
        @{ securityContext = 'Restricted'; operation = 'delete' },
        @{ securityContext = 'Restricted'; operation = 'test'; property = 'actualState' },
        @{ securityContext = 'Restricted'; operation = 'export' },
        @{ securityContext = 'Current'; operation = 'get'; property = 'actualState' },
        @{ securityContext = 'Current'; operation = 'set'; property = 'afterState' },
        @{ securityContext = 'Current'; operation = 'delete' },
        @{ securityContext = 'Current'; operation = 'test'; property = 'actualState' },
        @{ securityContext = 'Current'; operation = 'export' }
    ) {
        param($securityContext, $operation, $property, $type)
        
        if ($null -eq $type) {
            $type = "Test/SecurityContext$securityContext"
        }
        $inputObj = @{
            hello = "world"
            action = $operation
        }
        $out = dsc resource $operation -r $type --input ($inputObj | ConvertTo-Json -Compress) 2>$testdrive/error.log
        switch ($securityContext) {
            'Elevated' {
                if ($isAdmin) {
                    $LASTEXITCODE | Should -Be 0
                    if ($property) {
                        $result = $out | ConvertFrom-Json
                        $result.$property.action | Should -Be $operation
                    } elseif ($operation -eq 'export') {
                        $result = $out | ConvertFrom-Json
                        $result.resources.properties.action | Should -Be 'export'
                    }
                }
                else {
                    $LASTEXITCODE | Should -Be 2
                    (Get-Content "$testdrive/error.log") | Should -BeLike "*ERROR*Operation '$operation' for resource '$type' requires security context '$securityContext'*"
                }
            }
            'Restricted' {
                if ($isAdmin) {
                    $LASTEXITCODE | Should -Be 2
                    (Get-Content "$testdrive/error.log") | Should -BeLike "*ERROR*Operation '$operation' for resource '$type' requires security context '$securityContext'*"
                }
                else {
                    $LASTEXITCODE | Should -Be 0
                    if ($property) {
                        $result = $out | ConvertFrom-Json
                        $result.$property.action | Should -Be $operation
                    } elseif ($operation -eq 'export') {
                        $result = $out | ConvertFrom-Json
                        $result.resources.properties.action | Should -Be 'export'
                    }
                }
            }
            'Current' {
                $LASTEXITCODE | Should -Be 0
                if ($property) {
                    $result = $out | ConvertFrom-Json
                    $result.$property.action | Should -Be $operation
                } elseif ($operation -eq 'export') {
                    $result = $out | ConvertFrom-Json
                    $result.resources.properties.action | Should -Be 'export'
                }
            }
        }
    }
}