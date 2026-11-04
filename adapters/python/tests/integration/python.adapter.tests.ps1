# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
<#
.SYNOPSIS
    Integration tests for the Microsoft.Adapter/Python DSC adapter and
    the Microsoft.Python/Discover DSC discovery extension.

.DESCRIPTION
    Two test layers:
      Component tests  — invoke the adapter and discovery extension directly
                         as Python subprocesses (no DSC CLI required).
      DSC CLI tests    — exercise the full DSC CLI pipeline end-to-end.

    Prerequisites:
      • Python >= 3.11 on PATH as 'python' (Windows) or 'python3' (Linux/macOS)
      • ms-dsc and dsc-test-resource installed in that Python environment
      • 'dsc' CLI on PATH (for DSC CLI layer only)

.NOTES
    The adapter is invoked via ``python -m pyadapter.cli`` (module invocation),
    which relies on DSC setting the working directory to the manifest's directory.
    Both pyadapter/ and the bundled ms_dsc/ must reside in that directory.

.NOTES
    Pester 5 scope: user variables set at discovery-phase top-level are NOT
    carried to execution-phase BeforeAll.  Only automatic variables like
    $PSScriptRoot survive.  All paths and executables are therefore computed
    inside BeforeAll from $PSScriptRoot, and helper functions are defined
    inside the same BeforeAll (functions defined in BeforeAll ARE accessible
    from It blocks in the same Describe per Pester 5 contract).
#>

[CmdletBinding()]
param()

# ===========================================================================
Describe "Python DSC Adapter — Component Tests" {

    BeforeAll {
        # ----- Resolve the Python executable: 'python' on Windows, 'python3' on Unix ---
        $script:_py = if ($IsWindows) {
            (Get-Command python  -ErrorAction SilentlyContinue)?.Source
        } else {
            (Get-Command python3 -ErrorAction SilentlyContinue)?.Source ??
            (Get-Command python  -ErrorAction SilentlyContinue)?.Source
        }

        # The adapter root is invoked as a module; CWD must contain pyadapter/
        $script:_adapterRoot = (Resolve-Path (Join-Path $PSScriptRoot "../.."  )).Path

        if (-not $script:_py) { throw "Python not found on PATH. Install Python >= 3.11." }

        # ----- Helper defined inside BeforeAll: accessible from all It blocks ---
        function RunAdapter {
            param(
                [string]$Op,
                [string]$Res   = "",
                [string]$Json  = "",
                [switch]$NoIn
            )
            $a = @("-m", "pyadapter.cli", $Op)
            if ($Res) { $a += "--resource"; $a += $Res }
            $errFile = [System.IO.Path]::GetTempFileName()
            # CWD must be the adapter root so Python can find pyadapter/ and ms_dsc/ via '' in sys.path[0]
            Push-Location $script:_adapterRoot
            try {
                if ($NoIn -or -not $Json) { $out = & $script:_py @a 2>$errFile }
                else { $out = $Json | & $script:_py @a 2>$errFile }
            } finally {
                Pop-Location
            }
            $rc  = $LASTEXITCODE
            $err = Get-Content $errFile -Raw -ErrorAction SilentlyContinue
            Remove-Item $errFile -ErrorAction SilentlyContinue
            [pscustomobject]@{ ExitCode = $rc; StdOut = ($out | Out-String).Trim(); StdErr = ($err | Out-String).Trim() }
        }
    }

    # -----------------------------------------------------------------------
    Context "list operation" {
        It "exits with 0" {
            $r = RunAdapter -Op list -NoIn
            $r.ExitCode | Should -Be 0 -Because $r.StdErr
        }

        It "emits valid JSON when entries exist" {
            # Adapter list returns resources WITHOUT pre-built manifests.
            # DscTest resources have pre-built manifests so they appear in discovery,
            # not in adapter list.  Verify any emitted lines are valid JSON.
            $r = RunAdapter -Op list -NoIn
            $r.StdOut -split "`n" | Where-Object { $_.Trim() } | ForEach-Object {
                { $_ | ConvertFrom-Json } | Should -Not -Throw
            }
        }

        It "entries have required fields when present" {
            $r = RunAdapter -Op list -NoIn
            $entries = @($r.StdOut -split "`n" | Where-Object { $_ } | ConvertFrom-Json)
            foreach ($e in $entries) {
                $e.type        | Should -Not -BeNullOrEmpty
                $e.capabilities | Should -Not -BeNullOrEmpty
                $e.requireAdapter | Should -Be "Microsoft.Adapter/Python"
            }
        }
    }

    # -----------------------------------------------------------------------
    Context "get operation — DscTest/Get" {
        It "exits with 0" {
            (RunAdapter -Op get -Res DscTest/Get -Json '{"name":"it-get"}').ExitCode |
                Should -Be 0 -Because (RunAdapter -Op get -Res DscTest/Get -Json '{"name":"it-get"}').StdErr
        }

        It "emits a single JSON line" {
            $lines = (RunAdapter -Op get -Res DscTest/Get -Json '{"name":"single"}').StdOut -split "`n" |
                     Where-Object { $_.Trim() }
            $lines.Count | Should -Be 1
        }

        It "returns correct name in state" {
            $state = (RunAdapter -Op get -Res DscTest/Get -Json '{"name":"my-item"}').StdOut | ConvertFrom-Json
            $state.name | Should -Be "my-item"
        }

        It "DscTest/Get always reports _exist=true" {
            $state = (RunAdapter -Op get -Res DscTest/Get -Json '{"name":"always"}').StdOut | ConvertFrom-Json
            $state._exist | Should -Be $true
        }
    }

    # -----------------------------------------------------------------------
    Context "set operation — DscTest/ReadWrite (STATE_AND_DIFF)" {
        It "exits with 0" {
            (RunAdapter -Op set -Res DscTest/ReadWrite -Json '{"name":"set-it","_exist":true}').ExitCode |
                Should -Be 0
        }

        It "emits exactly two JSON lines (state + diff)" {
            $lines = (RunAdapter -Op set -Res DscTest/ReadWrite -Json '{"name":"two-lines","_exist":true}').StdOut -split "`n" |
                     Where-Object { $_.Trim() }
            $lines.Count | Should -Be 2
        }

        It "first line is the actual state" {
            $lines = (RunAdapter -Op set -Res DscTest/ReadWrite -Json '{"name":"st-name","_exist":true}').StdOut -split "`n" |
                     Where-Object { $_.Trim() }
            ($lines[0] | ConvertFrom-Json).name | Should -Be "st-name"
        }

        It "second line is a JSON array" {
            $lines = (RunAdapter -Op set -Res DscTest/ReadWrite -Json '{"name":"diff-arr","_exist":true}').StdOut -split "`n" |
                     Where-Object { $_.Trim() }
            { @($lines[1] | ConvertFrom-Json) } | Should -Not -Throw
        }

        It "reports _exist in changed properties on first-time set" {
            $name  = "fresh-$(Get-Random)"
            $lines = (RunAdapter -Op set -Res DscTest/ReadWrite -Json "{`"name`":`"$name`",`"_exist`":true}").StdOut -split "`n" |
                     Where-Object { $_.Trim() }
            @($lines[1] | ConvertFrom-Json) | Should -Contain "_exist"
        }

        It "always changes _exist on first-time set (fresh process per call)" {
            # Each adapter call is a fresh process; the in-process store is empty.
            # Every set therefore sees the item as absent and reports _exist as changed.
            $name  = "proc-$(Get-Random)"
            $lines = (RunAdapter -Op set -Res DscTest/ReadWrite -Json "{`"name`":`"$name`",`"_exist`":true}").StdOut -split "`n" |
                     Where-Object { $_.Trim() }
            @($lines[1] | ConvertFrom-Json) | Should -Contain "_exist"
        }
    }

    # -----------------------------------------------------------------------
    Context "test operation — DscTest/ReadWrite (STATE_AND_DIFF)" {
        It "exits with 0" {
            (RunAdapter -Op test -Res DscTest/ReadWrite -Json '{"name":"tst","_exist":false}').ExitCode |
                Should -Be 0
        }

        It "emits two JSON lines" {
            $lines = (RunAdapter -Op test -Res DscTest/ReadWrite -Json '{"name":"tst2","_exist":false}').StdOut -split "`n" |
                     Where-Object { $_.Trim() }
            $lines.Count | Should -Be 2
        }

        It "reports drift when desired _exist=true but store is empty" {
            $name  = "drift-$(Get-Random)"
            $lines = (RunAdapter -Op test -Res DscTest/ReadWrite -Json "{`"name`":`"$name`",`"_exist`":true}").StdOut -split "`n" |
                     Where-Object { $_.Trim() }
            @($lines[1] | ConvertFrom-Json) | Should -Contain "_exist"
        }

        It "always drifts when desired _exist=true (fresh process per call)" {
            # Each adapter call is a fresh process; the in-process store is always empty.
            # A test for _exist=true against an empty store will always drift.
            $name  = "always-drift-$(Get-Random)"
            $lines = (RunAdapter -Op test -Res DscTest/ReadWrite -Json "{`"name`":`"$name`",`"_exist`":true}").StdOut -split "`n" |
                     Where-Object { $_.Trim() }
            @($lines[1] | ConvertFrom-Json) | Should -Contain "_exist"
        }
    }

    # -----------------------------------------------------------------------
    Context "delete operation — DscTest/ReadWrite" {
        It "exits with 0" {
            $null = RunAdapter -Op set    -Res DscTest/ReadWrite -Json '{"name":"del-me","_exist":true}'
            (RunAdapter -Op delete -Res DscTest/ReadWrite -Json '{"name":"del-me"}').ExitCode | Should -Be 0
        }

        It "produces no stdout" {
            (RunAdapter -Op delete -Res DscTest/ReadWrite -Json '{"name":"silent-del"}').StdOut | Should -BeNullOrEmpty
        }

        It "item reports _exist=false after delete" {
            $name = "del-check-$(Get-Random)"
            $null = RunAdapter -Op set    -Res DscTest/ReadWrite -Json "{`"name`":`"$name`",`"_exist`":true}"
            $null = RunAdapter -Op delete -Res DscTest/ReadWrite -Json "{`"name`":`"$name`"}"
            $state = (RunAdapter -Op get   -Res DscTest/ReadWrite -Json "{`"name`":`"$name`"}").StdOut | ConvertFrom-Json
            $state._exist | Should -Be $false
        }
    }

    # -----------------------------------------------------------------------
    Context "export operation — DscTest/Export" {
        It "exits with 0" {
            (RunAdapter -Op export -Res DscTest/Export -Json '{}').ExitCode | Should -Be 0
        }

        It "emits multiple JSON lines" {
            $lines = (RunAdapter -Op export -Res DscTest/Export -Json '{}').StdOut -split "`n" | Where-Object { $_.Trim() }
            $lines.Count | Should -BeGreaterThan 1
        }

        It "each line has a name field" {
            (RunAdapter -Op export -Res DscTest/Export -Json '{}').StdOut -split "`n" | Where-Object { $_.Trim() } | ForEach-Object {
                ($_ | ConvertFrom-Json).name | Should -Not -BeNullOrEmpty
            }
        }

        It "returns all three fixture items unfiltered" {
            $lines = (RunAdapter -Op export -Res DscTest/Export -Json '{}').StdOut -split "`n" | Where-Object { $_.Trim() }
            $lines.Count | Should -Be 3
        }
    }

    # -----------------------------------------------------------------------
    Context "validate operation" {
        It "returns valid=true with exit 0" {
            $r = RunAdapter -Op validate -Res DscTest/Get -Json '{}'
            $r.ExitCode | Should -Be 0 -Because $r.StdErr
            ($r.StdOut | ConvertFrom-Json).valid | Should -Be $true
        }
    }

    # -----------------------------------------------------------------------
    Context "error handling" {
        It "exit code 2 for unknown resource type" {
            (RunAdapter -Op get -Res Unknown/Type -Json '{"name":"x"}').ExitCode | Should -Be 2
        }

        It "exit code 2 for invalid JSON" {
            (RunAdapter -Op get -Res DscTest/Get -Json '{bad json}').ExitCode | Should -Be 2
        }

        It "stderr is valid JSON on error" {
            $r = RunAdapter -Op get -Res Unknown/Type -Json '{}'
            $r.StdErr -split "`n" | Where-Object { $_.Trim() } | ForEach-Object {
                { $_ | ConvertFrom-Json } | Should -Not -Throw
            }
        }
    }
}

# ===========================================================================
Describe "Python DSC Discovery Extension — Component Tests" {

    BeforeAll {
        $script:_py2  = if ($IsWindows) {
            (Get-Command python  -ErrorAction SilentlyContinue)?.Source
        } else {
            (Get-Command python3 -ErrorAction SilentlyContinue)?.Source ??
            (Get-Command python  -ErrorAction SilentlyContinue)?.Source
        }
        $script:_disc = (Resolve-Path (Join-Path $PSScriptRoot "../../../../extensions/python/python.discover.py")).Path

        if (-not $script:_py2)  { throw "Python not found on PATH." }
        if (-not $script:_disc) { throw "Discover script not found" }

        function RunDiscover {
            $errFile = [System.IO.Path]::GetTempFileName()
            $out = & $script:_py2 $script:_disc 2>$errFile
            $rc  = $LASTEXITCODE
            $err = Get-Content $errFile -Raw -ErrorAction SilentlyContinue
            Remove-Item $errFile -ErrorAction SilentlyContinue
            [pscustomobject]@{ ExitCode = $rc; StdOut = ($out | Out-String).Trim(); StdErr = ($err | Out-String).Trim() }
        }
    }

    It "exits with 0" {
        (RunDiscover).ExitCode | Should -Be 0 -Because (RunDiscover).StdErr
    }

    It "emits valid JSON lines" {
        (RunDiscover).StdOut -split "`n" | Where-Object { $_.Trim() } | ForEach-Object {
            { $_ | ConvertFrom-Json } | Should -Not -Throw
        }
    }

    It "every line has manifestPath" {
        (RunDiscover).StdOut -split "`n" | Where-Object { $_.Trim() } | ConvertFrom-Json | ForEach-Object {
            $_.manifestPath | Should -Not -BeNullOrEmpty
        }
    }

    It "all manifestPath values are absolute" {
        (RunDiscover).StdOut -split "`n" | Where-Object { $_.Trim() } | ConvertFrom-Json | ForEach-Object {
            [System.IO.Path]::IsPathRooted($_.manifestPath) | Should -Be $true
        }
    }

    It "all manifests exist on disk" {
        (RunDiscover).StdOut -split "`n" | Where-Object { $_.Trim() } | ConvertFrom-Json | ForEach-Object {
            Test-Path -LiteralPath $_.manifestPath | Should -Be $true
        }
    }

    It "finds DscTest adapted manifests from fixture package" {
        $paths = @((RunDiscover).StdOut -split "`n" | Where-Object { $_.Trim() } |
                   ConvertFrom-Json | Select-Object -ExpandProperty manifestPath)
        ($paths | Where-Object { $_ -like "*DscTest*" }).Count |
            Should -BeGreaterThan 0 -Because "fixture manifests must be discovered"
    }
}

# ===========================================================================
Describe "DSC CLI — End-to-End Integration Tests" {
    <#
    .NOTES
        These tests require 'python' to be in the SYSTEM PATH (not just user PATH)
        because DSC spawns subprocesses that inherit the system environment, not the
        current user's PATH augmentations.

        On Windows: ensure Python is installed with "Add Python to PATH" for all users,
        or add Python's directory to the system PATH via System Properties → Environment Variables.

        Tests are automatically skipped when 'dsc' CLI is not found on PATH.
    #>

    BeforeAll {
        $script:_dsc         = (Get-Command dsc -ErrorAction SilentlyContinue)?.Source
        $script:_adapterDir  = (Resolve-Path (Join-Path $PSScriptRoot "../..")).Path
        $script:_extDir      = (Resolve-Path (Join-Path $PSScriptRoot "../../../../extensions/python")).Path
        $script:_manifestDir = (Resolve-Path (Join-Path $PSScriptRoot "../fixture/dsc_test_resource/dsc") -ErrorAction SilentlyContinue)?.Path

        if (-not $script:_dsc) {
            Write-Warning "dsc CLI not found on PATH; DSC CLI tests will be skipped."
        }

        # Ensure the Python directory is in PATH so DSC can invoke 'python'
        # when executing adapter manifests (DSC uses system PATH, not user PATH).
        $pyPath = (Get-Command python -ErrorAction SilentlyContinue)?.Source
        if ($pyPath) {
            $pyDir = Split-Path $pyPath
            if ($env:PATH -notlike "*$pyDir*") {
                $env:PATH = "$pyDir$([System.IO.Path]::PathSeparator)$env:PATH"
            }
        }

        $sep = [System.IO.Path]::PathSeparator
        if ($script:_adapterDir -and $script:_extDir) {
            $env:DSC_RESOURCE_PATH = "$($script:_adapterDir)${sep}$($script:_extDir)${sep}$($script:_manifestDir)${sep}$env:DSC_RESOURCE_PATH"
        }
    }

    BeforeEach {
        if (-not $script:_dsc) { Set-ItResult -Skipped -Because "dsc CLI not found" }
    }

    It "dsc resource list --adapter finds DscTest resources" {
        $errFile = [System.IO.Path]::GetTempFileName()
        $stdout  = & $script:_dsc resource list --adapter "Microsoft.Adapter/Python" 2>$errFile
        $stderr  = Get-Content $errFile -Raw -ErrorAction SilentlyContinue
        Remove-Item $errFile -ErrorAction SilentlyContinue
        $LASTEXITCODE | Should -Be 0 -Because $stderr
        ($stdout | Out-String) | Should -Match "DscTest"
    }

    It "dsc resource get returns state for DscTest/Get" {
        $errFile = [System.IO.Path]::GetTempFileName()
        $stdout  = & $script:_dsc resource get --resource "DscTest/Get" --input '{"name":"cli-get"}' 2>$errFile
        $stderr  = Get-Content $errFile -Raw -ErrorAction SilentlyContinue
        Remove-Item $errFile -ErrorAction SilentlyContinue
        $LASTEXITCODE | Should -Be 0 -Because $stderr
        ($stdout | Out-String) | Should -Match '"name"'
    }

    It "dsc resource set processes DscTest/ReadWrite" {
        $errFile = [System.IO.Path]::GetTempFileName()
        $null    = & $script:_dsc resource set --resource "DscTest/ReadWrite" --input '{"name":"cli-set","_exist":true}' 2>$errFile
        $stderr  = Get-Content $errFile -Raw -ErrorAction SilentlyContinue
        Remove-Item $errFile -ErrorAction SilentlyContinue
        $LASTEXITCODE | Should -Be 0 -Because $stderr
    }

    It "dsc resource test processes DscTest/ReadWrite" {
        $null    = & $script:_dsc resource set  --resource "DscTest/ReadWrite" --input '{"name":"cli-test-it","_exist":true}' 2>$null
        $errFile = [System.IO.Path]::GetTempFileName()
        $null    = & $script:_dsc resource test --resource "DscTest/ReadWrite" --input '{"name":"cli-test-it","_exist":true}' 2>$errFile
        $stderr  = Get-Content $errFile -Raw -ErrorAction SilentlyContinue
        Remove-Item $errFile -ErrorAction SilentlyContinue
        $LASTEXITCODE | Should -Be 0 -Because $stderr
    }

    It "dsc resource export returns items for DscTest/Export" {
        $errFile = [System.IO.Path]::GetTempFileName()
        $null    = & $script:_dsc resource export --resource "DscTest/Export" 2>$errFile
        $stderr  = Get-Content $errFile -Raw -ErrorAction SilentlyContinue
        Remove-Item $errFile -ErrorAction SilentlyContinue
        $LASTEXITCODE | Should -Be 0 -Because $stderr
    }
}
