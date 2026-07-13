# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'tests for policy folder security validation' {
    BeforeDiscovery {
        $isElevated = if ($IsWindows) {
            ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
                [Security.Principal.WindowsBuiltInRole]::Administrator)
        } else {
            (id -u) -eq 0
        }
    }

    Context 'Windows policy folder with insecure ACL emits warning' -Skip:(!$IsWindows -or !$isElevated) {
        BeforeAll {
            $script:policyDirPath = Join-Path $env:ProgramData "dsc"
            $script:policyFilePath = Join-Path $script:policyDirPath "dsc.settings.json"
            $script:existedBefore = Test-Path $script:policyDirPath

            if ($script:existedBefore) {
                $script:originalAcl = Get-Acl -Path $script:policyDirPath
            } else {
                New-Item -ItemType Directory -Path $script:policyDirPath -Force | Out-Null
            }

            # Write a simple policy settings file
            @{ tracing = @{ level = "TRACE" } } | ConvertTo-Json -Depth 5 | Set-Content -Path $script:policyFilePath

            # Grant Everyone write access to make the folder insecure
            icacls $script:policyDirPath /grant "Everyone:(OI)(CI)(W)" /T | Out-Null
        }

        AfterAll {
            Remove-Item -Path $script:policyFilePath -ErrorAction SilentlyContinue

            if ($script:existedBefore) {
                Set-Acl -Path $script:policyDirPath -AclObject $script:originalAcl
            } else {
                Remove-Item -Recurse -Force -Path $script:policyDirPath -ErrorAction SilentlyContinue
            }
        }

        It 'Should emit a warning about insecure policy folder' {
            dsc -l warn resource list 2> $TestDrive/tracing.txt
            "$TestDrive/tracing.txt" | Should -FileContentMatch "is not secure, settings file will not be used"
        }

        It 'Should not exit with an error code' {
            dsc -l warn resource list 2> $TestDrive/tracing.txt
            $LASTEXITCODE | Should -Be 0
        }
    }

    Context 'Linux policy folder with insecure permissions emits warning' -Skip:($IsWindows -or !$isElevated) {
        BeforeAll {
            $script:policyDirPath = "/etc/dsc"
            $script:policyFilePath = Join-Path $script:policyDirPath "dsc.settings.json"
            $script:existedBefore = Test-Path $script:policyDirPath

            if ($script:existedBefore) {
                $script:originalMode = (stat -c '%a' $script:policyDirPath)
            } else {
                New-Item -ItemType Directory -Path $script:policyDirPath -Force | Out-Null
            }

            # Write a simple policy settings file
            @{ tracing = @{ level = "TRACE" } } | ConvertTo-Json -Depth 5 | Set-Content -Path $script:policyFilePath

            # Make the folder world-writable (insecure)
            chmod 777 $script:policyDirPath
        }

        AfterAll {
            if ($script:existedBefore) {
                chmod $script:originalMode $script:policyDirPath
            }

            Remove-Item -Path $script:policyFilePath -ErrorAction SilentlyContinue
            if (-not $script:existedBefore) {
                Remove-Item -Recurse -Force -Path $script:policyDirPath -ErrorAction SilentlyContinue
            }
        }

        It 'Should emit a warning about insecure policy folder' {
            dsc -l warn resource list 2> $TestDrive/tracing.txt
            "$TestDrive/tracing.txt" | Should -FileContentMatch "is not secure, settings file will not be used"
        }

        It 'Should not exit with an error code' {
            dsc -l warn resource list 2> $TestDrive/tracing.txt
            $LASTEXITCODE | Should -Be 0
        }
    }
}
