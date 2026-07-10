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
        }

        #create backups of settings files
        $script:dscSettingsFilePath_backup = Join-Path $script:dscHome "dsc.settings.json.backup"
        $script:dscDefaultSettingsFilePath_backup = Join-Path $script:dscHome "dsc_default.settings.json.backup"
        Copy-Item -Force -Path $script:dscSettingsFilePath -Destination $script:dscSettingsFilePath_backup
        Copy-Item -Force -Path $script:dscDefaultSettingsFilePath -Destination $script:dscDefaultSettingsFilePath_backup

        $script:originalXdgConfigHome = $env:XDG_CONFIG_HOME
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
        $env:XDG_CONFIG_HOME = $script:originalXdgConfigHome
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

    It 'ensure a user settings file via XDG_CONFIG_HOME has effect' {

        $env:XDG_CONFIG_HOME = Join-Path $TestDrive 'xdg'
        $userSettingsDir = Join-Path $env:XDG_CONFIG_HOME 'dsc'
        New-Item -ItemType Directory -Path $userSettingsDir -Force | Out-Null

        $script:dscDefaultv1Settings."resourcePath"."directories" = @("UserDir")
        $script:dscDefaultv1Settings | ConvertTo-Json -Depth 90 | Set-Content -Force -Path (Join-Path $userSettingsDir 'dsc.settings.json')

        dsc -l debug resource list 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Using Resource Path: UserDir'
    }

    It 'ensure a workspace settings file has effect' {

        $workspaceDir = Join-Path $TestDrive 'workspace'
        New-Item -ItemType Directory -Path $workspaceDir -Force | Out-Null

        $script:dscDefaultv1Settings."resourcePath"."directories" = @("WorkspaceDir")
        $script:dscDefaultv1Settings | ConvertTo-Json -Depth 90 | Set-Content -Force -Path (Join-Path $workspaceDir 'dsc.settings.json')

        try {
            Push-Location $workspaceDir
            dsc -l debug resource list 2> $TestDrive/tracing.txt
        } finally {
            Pop-Location
        }
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Using Resource Path: WorkspaceDir'
    }

    It 'ensure workspace settings override user and install settings' {

        $script:dscDefaultv1Settings."resourcePath"."directories" = @("InstallDir")
        $script:dscDefaultv1Settings | ConvertTo-Json -Depth 90 | Set-Content -Force -Path $script:dscSettingsFilePath

        $env:XDG_CONFIG_HOME = Join-Path $TestDrive 'xdg'
        $userSettingsDir = Join-Path $env:XDG_CONFIG_HOME 'dsc'
        New-Item -ItemType Directory -Path $userSettingsDir -Force | Out-Null
        $script:dscDefaultv1Settings."resourcePath"."directories" = @("UserDir")
        $script:dscDefaultv1Settings | ConvertTo-Json -Depth 90 | Set-Content -Force -Path (Join-Path $userSettingsDir 'dsc.settings.json')

        $workspaceDir = Join-Path $TestDrive 'workspace'
        New-Item -ItemType Directory -Path $workspaceDir -Force | Out-Null
        $script:dscDefaultv1Settings."resourcePath"."directories" = @("WorkspaceDir")
        $script:dscDefaultv1Settings | ConvertTo-Json -Depth 90 | Set-Content -Force -Path (Join-Path $workspaceDir 'dsc.settings.json')

        # user settings override install settings
        dsc -l debug resource list 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Using Resource Path: UserDir'

        # workspace settings override user settings
        try {
            Push-Location $workspaceDir
            dsc -l debug resource list 2> $TestDrive/tracing.txt
        } finally {
            Pop-Location
        }
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Using Resource Path: WorkspaceDir'
    }

    It 'ensure policy overrides workspace settings' {

        if (! $IsWindows) {
            Set-ItResult -Skip -Because "Setting policy requires sudo"
            return
        }

        $script:dscDefaultv1Settings."resourcePath"."directories" = @("PolicyDir")
        # only define resourcePath as policy so the tracing field stays overridable by '-l debug'
        @{ resourcePath = $script:dscDefaultv1Settings."resourcePath" } | ConvertTo-Json -Depth 90 | Set-Content -Force -Path $script:policyFilePath

        $workspaceDir = Join-Path $TestDrive 'workspace'
        New-Item -ItemType Directory -Path $workspaceDir -Force | Out-Null
        $script:dscDefaultv1Settings."resourcePath"."directories" = @("WorkspaceDir")
        $script:dscDefaultv1Settings | ConvertTo-Json -Depth 90 | Set-Content -Force -Path (Join-Path $workspaceDir 'dsc.settings.json')

        try {
            Push-Location $workspaceDir
            dsc -l debug resource list 2> $TestDrive/tracing.txt
        } finally {
            Pop-Location
        }
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Using Resource Path: PolicyDir'
    }
}
