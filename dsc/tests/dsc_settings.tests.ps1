# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'tests for dsc settings' {
    BeforeAll {
        $env:DSC_RESOURCE_PATH = $testdrive

        $script:policyFilePath = if ($IsWindows) {
            Join-Path $env:ProgramData "dsc" "settings.dsc.json"
        } else {
            "/etc/.dsc/settings.dsc.json"
        }

        $script:dscHome = (Get-Command dsc).Path | Split-Path
        $script:dscSettingsFilePath = Join-Path $script:dscHome "settings.dsc.json"
        $script:dscDefaultv1SettingsFilePath = Join-Path $script:dscHome "default_settings.v1.dsc.json"
        $script:dscDefaultv1SettingsJson = Get-Content -Raw -Path $script:dscDefaultv1SettingsFilePath

        $script:policyDirPath = $script:policyFilePath | Split-Path
        New-Item -ItemType Directory -Path $script:policyDirPath | Out-Null

        #create backups of settings files
        $script:dscSettingsFilePath_backup = Join-Path $script:dscHome "settings.dsc.json.backup"
        $script:dscDefaultv1SettingsFilePath_backup = Join-Path $script:dscHome "default_settings.v1.dsc.json.backup"
        Copy-Item -Force -Path $script:dscSettingsFilePath -Destination $script:dscSettingsFilePath_backup
        Copy-Item -Force -Path $script:dscDefaultv1SettingsFilePath -Destination $script:dscDefaultv1SettingsFilePath_backup
    }

    AfterAll {
        Remove-Item -Force -Path $script:dscSettingsFilePath_backup
        Remove-Item -Force -Path $script:dscDefaultv1SettingsFilePath_backup
        Remove-Item -Recurse -Force -Path $script:policyDirPath
    }

    AfterEach {
        Copy-Item -Force -Path $script:dscSettingsFilePath_backup -Destination $script:dscSettingsFilePath
        Copy-Item -Force -Path $script:dscDefaultv1SettingsFilePath_backup -Destination $script:dscDefaultv1SettingsFilePath
        Remove-Item -Path $script:policyFilePath -ErrorAction SilentlyContinue
        $env:DSC_RESOURCE_PATH = $null
    }

    It 'ensure a new tracing value in settings has effect' {
        
        $script:dscDefaultv1SettingsJson.Replace('"level": "WARN"', '"level": "TRACE"') | Set-Content -Force -Path $script:dscSettingsFilePath

        dsc resource list 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly "Trace-level is Trace"
    }

    It 'ensure a new resource_path value in settings has effect' {
        
        $script:dscDefaultv1SettingsJson.Replace('"directories": []', '"directories": ["TestDir"]') | Set-Content -Force -Path $script:dscSettingsFilePath
        dsc -l debug resource list 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Using Resource Path: "TestDir'
    }

    It 'Confirm settings override priorities' {
        
        $v = $script:dscDefaultv1SettingsJson.Replace('"level": "WARN"', '"level": "TRACE"')
        $v = $v.Replace('"directories": []', '"directories": ["PolicyDir"]')
        $v | Set-Content -Force -Path $script:policyFilePath

        $v = $script:dscDefaultv1SettingsJson.Replace('"level": "WARN"', '"level": "TRACE"')
        $v = $v.Replace('"directories": []', '"directories": ["SettingsDir"]')
        $v | Set-Content -Force -Path $script:dscSettingsFilePath

        $v = $script:dscDefaultv1SettingsJson.Replace('"level": "WARN"', '"level": "TRACE"')
        $v = $v.Replace('"directories": []', '"directories": ["Defaultv1SettingsDir"]')
        $v | Set-Content -Force -Path  $script:dscDefaultv1SettingsFilePath

        # ensure policy overrides everything
        dsc -l debug resource list 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly "Trace-level is Trace"
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Using Resource Path: "PolicyDir'

        # without policy, command-line args have priority
        Remove-Item -Path $script:policyFilePath
        dsc -l debug resource list 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly "Trace-level is Debug"
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Using Resource Path: "SettingsDir'

        # without policy and command-line args, settings file is used
        dsc resource list 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly "Trace-level is Trace"
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Using Resource Path: "SettingsDir'

        # without policy and command-line args and settings file, the default settings file is used
        Remove-Item -Path $script:dscSettingsFilePath
        dsc resource list 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly "Trace-level is Trace"
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Using Resource Path: "Defaultv1SettingsDir'
    }
}
