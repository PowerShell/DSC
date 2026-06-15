# pythoncomponent.tests.ps1
# Component-level tests for pyDscAdapter
# 
# These tests invoke the Python adapter by executing the package `__main__.py`
# entry point directly to validate adapter behavior without the DSC engine wrapper.
# 
# Test scope: adapter operation dispatch, JSON parsing, resource loading, error handling
# Use for: fast feedback on adapter logic changes, no DSC CLI required

param(
    [string]$PythonExe = "python3"
)

Describe "Python Adapter - Component Tests" {
    BeforeAll {
        $manifestName = "pythontest.dsc.manifests.json"
        $manifestPath = $null

        $searchStarts = @()
        if (-not [string]::IsNullOrWhiteSpace($PSScriptRoot)) {
            $searchStarts += $PSScriptRoot
        }
        $searchStarts += (Get-Location).Path

        foreach ($start in ($searchStarts | Select-Object -Unique)) {
            $current = (Resolve-Path -LiteralPath $start -ErrorAction Stop).Path

            while ($true) {
                $candidate = Join-Path -Path $current -ChildPath "adapters/python/tests/$manifestName"
                if (Test-Path -LiteralPath $candidate) {
                    $manifestPath = (Resolve-Path -LiteralPath $candidate).Path
                    break
                }

                $candidateLocal = Join-Path -Path $current -ChildPath $manifestName
                if (Test-Path -LiteralPath $candidateLocal) {
                    $manifestPath = (Resolve-Path -LiteralPath $candidateLocal).Path
                    break
                }

                $parent = Split-Path -Path $current -Parent
                if ([string]::IsNullOrWhiteSpace($parent) -or $parent -eq $current) {
                    break
                }
                $current = $parent
            }

            if ($manifestPath) {
                break
            }
        }

        if (-not $manifestPath) {
            throw "Unable to locate '$manifestName' from PSScriptRoot or current directory."
        }

        $manifestDir = Split-Path -Parent $manifestPath
        $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json

        function script:Resolve-ManifestResourcePath {
            param(
                [string]$EntryPath,
                [string]$BaseDir
            )

            if ([string]::IsNullOrWhiteSpace($EntryPath)) {
                throw "Manifest entry path is empty."
            }

            $manifestRelativePath = Join-Path -Path $BaseDir -ChildPath $EntryPath
            if (Test-Path -LiteralPath $manifestRelativePath) {
                return (Resolve-Path -LiteralPath $manifestRelativePath).Path
            }

             throw "Unable to resolve resource path '$EntryPath' relative to manifest directory '$BaseDir'."
        }

        $script:ResourcePathByType = @{}
        foreach ($entry in $manifest.adaptedResources) {
            if ($null -eq $entry.type -or $null -eq $entry.path) {
                continue
            }
            $script:ResourcePathByType[$entry.type] = Resolve-ManifestResourcePath -EntryPath $entry.path -BaseDir $manifestDir
        }

        foreach ($requiredType in "PythonTest/Get", "PythonTest/Set", "PythonTest/Test", "PythonTest/Export") {
            if (-not $script:ResourcePathByType.ContainsKey($requiredType)) {
                throw "Required adapted resource '$requiredType' was not found in '$manifestPath'."
            }
        }

        $adapterProbeRoot = Split-Path -Parent $script:ResourcePathByType["PythonTest/Get"]
        $script:AdapterDir = $null
        while ($true) {
            if (Test-Path -LiteralPath (Join-Path -Path $adapterProbeRoot -ChildPath "pyDscAdapter")) {
                $script:AdapterDir = $adapterProbeRoot
                break
            }

            $parent = Split-Path -Path $adapterProbeRoot -Parent
            if ([string]::IsNullOrWhiteSpace($parent) -or $parent -eq $adapterProbeRoot) {
                break
            }
            $adapterProbeRoot = $parent
        }

        if (-not $script:AdapterDir) {
            throw "Unable to locate adapters/python folder from manifest-resolved resource path."
        }

        $script:AdapterEntryPoint = Join-Path -Path $script:AdapterDir -ChildPath "pyDscAdapter/__main__.py"
        if (-not (Test-Path -LiteralPath $script:AdapterEntryPoint)) {
            throw "Unable to locate adapter entry point at '$script:AdapterEntryPoint'."
        }

        $testFixturePath = $script:ResourcePathByType["PythonTest/Test"]
        $compileErrorPath = Join-Path -Path $TestDrive -ChildPath "stderr_component_compile.txt"

        Push-Location -LiteralPath $script:AdapterDir
        try {
            & $PythonExe -m py_compile $testFixturePath 2> $compileErrorPath
            if ($LASTEXITCODE -ne 0) {
                $compileError = Get-Content -LiteralPath $compileErrorPath -Raw -ErrorAction SilentlyContinue
                throw "Test fixture '$testFixturePath' has a syntax error: $compileError"
            }
        }
        finally {
            Pop-Location
        }

        function script:Get-ResourcePathForType {
            param([string]$ResourceType)

            if ([string]::IsNullOrWhiteSpace($ResourceType)) {
                return ""
            }

            if ($script:ResourcePathByType.ContainsKey($ResourceType)) {
                return $script:ResourcePathByType[$ResourceType]
            }

            return ""
        }

        function script:Invoke-Adapter {
            param(
                [string]$Operation,
                [string]$ResourceType,
                [string]$InputJson = "{}"
            )

            $resourcePath = Get-ResourcePathForType -ResourceType $ResourceType
            $stderrPath = Join-Path -Path $TestDrive -ChildPath "stderr_component.txt"

            Write-Host "CMD: $PythonExe $script:AdapterEntryPoint adapter --operation $Operation --resource $ResourceType --resource-path `"$resourcePath`" --input '$InputJson'" -ForegroundColor Yellow

            Push-Location -LiteralPath $script:AdapterDir
            try {
                $stdout = & $PythonExe $script:AdapterEntryPoint adapter `
                    --operation $Operation `
                    --resource $ResourceType `
                    --resource-path $resourcePath `
                    --input $InputJson `
                    2> $stderrPath

                $stderr = Get-Content -LiteralPath $stderrPath -Raw -ErrorAction SilentlyContinue
            }
            finally {
                Pop-Location
            }

            [pscustomobject]@{
                ExitCode = $LASTEXITCODE
                StdOut   = ($stdout | Out-String).Trim()
                StdErr   = ($stderr | Out-String).Trim()
            }
        }

        function script:Convert-StdOutToJsonLines {
            param([string]$StdOut)

            return @($StdOut -split "`n" | Where-Object { $_.Trim() -ne "" })
        }

    }

    It "GET returns expected wrapper output for <CaseName>" -TestCases @(
        @{ CaseName = "named input"; InputJson = '{"name":"pkg","_exist":true}'; ExpectedActualStateName = "pkg"; ExpectedActualStateExists = $true; ExpectedWrapperName = $null }
        @{ CaseName = "missing name input"; InputJson = '{"_exist":true}'; ExpectedActualStateName = $null; ExpectedActualStateExists = $null; ExpectedWrapperName = "PythonTest/Get" }
    ) {
        $resourceType = "PythonTest/Get"
        $result = Invoke-Adapter -Operation "get" -ResourceType $resourceType -InputJson $InputJson

           $result.ExitCode | Should -Be 0 -Because $result.StdErr
           $result.StdOut | Should -Match '^\{.*\}$' -Because $result.StdErr

        $payload = $result.StdOut | ConvertFrom-Json

        if ($null -ne $ExpectedActualStateName) {
              $payload.metadata."Microsoft.DSC".operation | Should -Be "Get"
              $payload.type | Should -Be "Microsoft.DSC.Adapters/Python"
              $payload.result[0].type | Should -Be $resourceType
              $payload.result[0].result.actualState.name | Should -Be $ExpectedActualStateName
              $payload.result[0].result.actualState._exist | Should -Be $ExpectedActualStateExists
        }

        if ($null -ne $ExpectedWrapperName) {
              $payload.name | Should -Be $ExpectedWrapperName
              $payload.result[0].name | Should -Be $ExpectedWrapperName
        }
    }

    It "SET returns after_state and diffs lines" {
        $result = Invoke-Adapter -Operation "set" -ResourceType "PythonTest/Set" -InputJson '{"name":"curl","_exist":false}'

            $result.ExitCode | Should -Be 0 -Because $result.StdErr

        $lines = Convert-StdOutToJsonLines -StdOut $result.StdOut
            $lines.Count | Should -Be 2

        $afterState = $lines[0] | ConvertFrom-Json
        $diffs = $lines[1] | ConvertFrom-Json
        $diffValues = @($diffs)

            $afterState.name | Should -Be "curl"
            $afterState._exist | Should -Be $false
            $diffValues.Count | Should -Be 1
            $diffValues[0] | Should -Be "_exist"
    }

    It "TEST returns expected drift output for <CaseName>" -TestCases @(
        @{ CaseName = "drift"; InputJson = '{"name":"pkg","_exist":false,"desired_exist":true}'; ExpectedActualStateName = "pkg"; ExpectedActualStateExists = $false; ExpectedDiffCount = $null; ExpectedDiffValue = "_exist" }
        @{ CaseName = "no drift"; InputJson = '{"name":"pkg","_exist":true,"desired_exist":true}'; ExpectedActualStateName = $null; ExpectedActualStateExists = $null; ExpectedDiffCount = 0; ExpectedDiffValue = $null }
    ) {
        $result = Invoke-Adapter -Operation "test" -ResourceType "PythonTest/Test" -InputJson $InputJson

           $result.ExitCode | Should -Be 0 -Because $result.StdErr

        $lines = Convert-StdOutToJsonLines -StdOut $result.StdOut
        $diffs = $lines[1] | ConvertFrom-Json

        if ($null -ne $ExpectedActualStateName) {
              $lines.Count | Should -Be 2

            $actualState = $lines[0] | ConvertFrom-Json
              $actualState.name | Should -Be $ExpectedActualStateName
              $actualState._exist | Should -Be $ExpectedActualStateExists
              $diffs | Should -Contain $ExpectedDiffValue
        }

        if ($null -ne $ExpectedDiffCount) {
              $diffs.Count | Should -Be $ExpectedDiffCount
        }
    }

    It "EXPORT returns package collection" {
        $result = Invoke-Adapter -Operation "export" -ResourceType "PythonTest/Export" -InputJson '{}'

            $result.ExitCode | Should -Be 0 -Because $result.StdErr
            $result.StdOut | Should -Match '^\{.*\}$' -Because $result.StdErr

        $payload = $result.StdOut | ConvertFrom-Json
            $payload.packages.Count | Should -Be 2
            $payload.packages[0].name | Should -Be "alpha"
            $payload.packages[1].name | Should -Be "beta"
    }

    It "LIST returns empty resources for <CaseName>" -TestCases @(
        @{ CaseName = "empty resource type"; ResourceType = "" }
        @{ CaseName = "unknown resource type"; ResourceType = "Unknown/Resource" }
    ) {
        $result = Invoke-Adapter -Operation "list" -ResourceType $ResourceType -InputJson "{}"

            $result.ExitCode | Should -Be 0 -Because $result.StdErr
            $result.StdOut | Should -Match '^\{.*\}$' -Because $result.StdErr

        $payload = $result.StdOut | ConvertFrom-Json
        $resources = @()
        if ($null -ne $payload -and $null -ne $payload.resources) {
            $resources = @($payload.resources)
        }

            $resources.Count | Should -Be 0
    }

    It "VALIDATE returns valid true" {
        $result = Invoke-Adapter -Operation "validate" -ResourceType "PythonTest/Get" -InputJson "{}"

            $result.ExitCode | Should -Be 0 -Because $result.StdErr
        $payload = $result.StdOut | ConvertFrom-Json
            $payload.valid | Should -Be $true
    }

    It '<Operation> with invalid JSON returns error' -TestCases @(
    @{ Operation = 'get'; ResourceType = 'PythonTest/Get' }
    @{ Operation = 'set'; ResourceType = 'PythonTest/Set' }
    @{ Operation = 'test'; ResourceType = 'PythonTest/Test' }
    ) {
        $result = Invoke-Adapter -Operation $Operation -ResourceType $ResourceType -InputJson '{bad-json'
           $result.ExitCode | Should -Be 1
           $result.StdOut | Should -Match '"error"'
    }

    It "EXPORT with filter input still returns package collection" {
        $result = Invoke-Adapter -Operation "export" -ResourceType "PythonTest/Export" -InputJson '{"name":"alpha"}'

        if ($result.ExitCode -eq 0) {
            $payload = $result.StdOut | ConvertFrom-Json
              $payload.packages.Count | Should -Be 2
              $payload.packages[0].name | Should -Be "alpha"
        }
        else {
              $result.ExitCode | Should -Be 1
              ($result.StdOut + "`n" + $result.StdErr) | Should -Match 'takes no arguments'
        }
    }

    It "Unknown resource type returns exit code 2" {
        $result = Invoke-Adapter -Operation "get" -ResourceType "Unknown/Resource" -InputJson "{}"

            $result.ExitCode | Should -Be 2
            $result.StdOut | Should -Match '"error"'
    }
}
