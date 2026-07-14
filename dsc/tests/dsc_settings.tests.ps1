# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'tests for dsc settings' {
    BeforeAll {

        $script:policyFilePath = if ($IsWindows) {
            Join-Path $env:ProgramData "dsc" "dsc.settings.json"
        } else {
            "/etc/dsc/dsc.settings.json"
        }

        $script:dscHome = (Get-Command dsc).Path | Split-Path
        $script:dscSettingsFilePath = Join-Path $script:dscHome "dsc.settings.json"
        $script:dscDefaultSettingsFilePath = Join-Path $script:dscHome "dsc_default.settings.json"

        if ($IsWindows) { #"Setting policy on Linux requires sudo"
            $script:policyDirPath = $script:policyFilePath | Split-Path
            New-Item -ItemType Directory -Path $script:policyDirPath | Out-Null
            # Set secure ACLs: only SYSTEM and Administrators have write access
            $acl = Get-Acl -Path $script:policyDirPath
            $acl.SetAccessRuleProtection($true, $false)
            $acl.Access | ForEach-Object { $acl.RemoveAccessRule($_) } | Out-Null
            $systemRule = New-Object System.Security.AccessControl.FileSystemAccessRule(
                "NT AUTHORITY\SYSTEM", "FullControl", "ContainerInherit,ObjectInherit", "None", "Allow")
            $adminsRule = New-Object System.Security.AccessControl.FileSystemAccessRule(
                "BUILTIN\Administrators", "FullControl", "ContainerInherit,ObjectInherit", "None", "Allow")
            $usersReadRule = New-Object System.Security.AccessControl.FileSystemAccessRule(
                "BUILTIN\Users", "ReadAndExecute", "ContainerInherit,ObjectInherit", "None", "Allow")
            $acl.AddAccessRule($systemRule)
            $acl.AddAccessRule($adminsRule)
            $acl.AddAccessRule($usersReadRule)
            Set-Acl -Path $script:policyDirPath -AclObject $acl
        }

        #create backups of settings files
        $script:dscSettingsFilePath_backup = Join-Path $script:dscHome "dsc.settings.json.backup"
        $script:dscDefaultSettingsFilePath_backup = Join-Path $script:dscHome "dsc_default.settings.json.backup"
        Copy-Item -Force -Path $script:dscSettingsFilePath -Destination $script:dscSettingsFilePath_backup
        Copy-Item -Force -Path $script:dscDefaultSettingsFilePath -Destination $script:dscDefaultSettingsFilePath_backup
    }

    AfterAll {
        Remove-Item -Force -Path $script:dscSettingsFilePath_backup
        Remove-Item -Force -Path $script:dscDefaultSettingsFilePath_backup
        if ($IsWindows) { #"Setting policy on Linux requires sudo"
            Remove-Item -Recurse -Force -Path $script:policyDirPath
        }
    }

    BeforeEach {
        $script:dscDefaultSettings = Get-Content -Raw -Path $script:dscDefaultSettingsFilePath_backup | ConvertFrom-Json
        $script:dscDefaultv1Settings = (Get-Content -Raw -Path $script:dscDefaultSettingsFilePath_backup | ConvertFrom-Json)."1"
    }

    AfterEach {
        Copy-Item -Force -Path $script:dscSettingsFilePath_backup -Destination $script:dscSettingsFilePath
        Copy-Item -Force -Path $script:dscDefaultSettingsFilePath_backup -Destination $script:dscDefaultSettingsFilePath
        if ($IsWindows) { #"Setting policy on Linux requires sudo"
            Remove-Item -Path $script:policyFilePath -ErrorAction SilentlyContinue
        }
    }

    It 'ensure a new tracing value in settings has effect' {

        $script:dscDefaultv1Settings."tracing"."level" = "TRACE"
        $script:dscDefaultv1Settings | ConvertTo-Json -Depth 90 | Set-Content -Force -Path $script:dscSettingsFilePath

        dsc resource list 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly "Trace-level is Trace"
    }

    It 'ensure a new resource_path value in settings has effect' {

        $script:dscDefaultv1Settings."resourcePath"."directories" = @("TestDir1","TestDir2")
        $script:dscDefaultv1Settings | ConvertTo-Json -Depth 90 | Set-Content -Force -Path $script:dscSettingsFilePath
        dsc -l debug resource list 2> $TestDrive/tracing.txt
        $expectedString = 'Using Resource Path: TestDir1'+[System.IO.Path]::PathSeparator+'TestDir2'
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expectedString
    }

    It 'Confirm settings override priorities' {

        if (! $IsWindows) {
            Set-ItResult -Skip -Because "Setting policy requires sudo"
            return
        }

        $script:dscDefaultv1Settings."tracing"."level" = "TRACE"
        $script:dscDefaultv1Settings."resourcePath"."directories" = @("PolicyDir")
        $script:dscDefaultv1Settings | ConvertTo-Json -Depth 90 | Set-Content -Force -Path $script:policyFilePath

        $script:dscDefaultv1Settings."tracing"."level" = "TRACE"
        $script:dscDefaultv1Settings."resourcePath"."directories" = @("SettingsDir")
        $script:dscDefaultv1Settings | ConvertTo-Json -Depth 90 | Set-Content -Force -Path $script:dscSettingsFilePath

        $script:dscDefaultSettings."1"."tracing"."level" = "TRACE"
        $script:dscDefaultSettings."1"."resourcePath"."directories" = @("Defaultv1SettingsDir")
        $script:dscDefaultSettings | ConvertTo-Json -Depth 90 | Set-Content -Force -Path  $script:dscDefaultSettingsFilePath

        # ensure policy overrides everything
        dsc -l debug resource list 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly "Trace-level is Trace"
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Using Resource Path: PolicyDir'

        # without policy, command-line args have priority
        Remove-Item -Path $script:policyFilePath
        dsc -l debug resource list 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly "Trace-level is Debug"
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Using Resource Path: SettingsDir'

        # without policy and command-line args, settings file is used
        dsc resource list 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly "Trace-level is Trace"
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Using Resource Path: SettingsDir'

        # without policy and command-line args and settings file, the default settings file is used
        Remove-Item -Path $script:dscSettingsFilePath
        dsc resource list 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly "Trace-level is Trace"
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Using Resource Path: Defaultv1SettingsDir'
    }

    It 'DSC_IGNORE_SETTINGS_FILE environment variable disables settings file' {
        $oldEnv = $env:DSC_IGNORE_SETTINGS_FILE
        try {
            $env:DSC_IGNORE_SETTINGS_FILE = "1"
            $null = dsc -l warn resource list 2> $TestDrive/tracing.txt
            $errorLog = Get-Content "$TestDrive/tracing.txt" -Raw
            $errorLog | Should -BeLike "*WARN*Ignoring settings file due to environment variable 'DSC_IGNORE_SETTINGS_FILE' being set or '--ignore-settings-file' flag being used*"
        }
        finally {
            $env:DSC_IGNORE_SETTINGS_FILE = $oldEnv
        }
    }

    It '--ignore-settings-file command-line argument disables settings file' {
        $null = dsc --ignore-settings-file resource list 2> $TestDrive/tracing.txt
        $errorLog = Get-Content "$TestDrive/tracing.txt" -Raw
        $errorLog | Should -BeLike "*WARN*Ignoring settings file due to environment variable 'DSC_IGNORE_SETTINGS_FILE' being set or '--ignore-settings-file' flag being used*"
    }
}
