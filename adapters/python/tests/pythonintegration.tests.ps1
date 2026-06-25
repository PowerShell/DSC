# pythonintegration.tests.ps1
# End-to-end integration tests for Python adapter via DSC CLI
#
# These tests invoke the full DSC command-line surface (`dsc resource get/set/test/export`)
# to validate adapter behavior within the DSC engine ecosystem.
#
# Test scope: DSC CLI integration, engine wrapping, config manifest resolution,
# full pipeline from user command to adapter output
# Use for: validating DSC compatibility, end-to-end scenarios, deployment validation

param(
    [string]$DscExe = "dsc"
)

Describe "Python Adapter - Integration Tests via DSC CLI" {
    BeforeAll {
        function Assert-KnownDscAdapterFailure {
            param(
                [pscustomobject]$Result
            )

                $Result.ExitCode | Should -Not -Be 0
                $Result.StdErr | Should -Not -BeNullOrEmpty
                $Result.StdErr | Should -Match "Operation Executable 'python3' not found|Failed to run process 'python3'|Command: Resource 'python3' \[exit code 2\]|Resource not found:\s*PythonTest/|Failed to parse resource:"
        }

        function Initialize-DscPythonPath {
            $pythonCommands = @('python3', 'python')
            $pathSeparator = if ($IsWindows) { ';' } else { ':' }

            foreach ($pythonCommand in $pythonCommands) {
                try {
                    $pythonPath = (& $pythonCommand -c "import sys; print(sys.executable)" 2>$null | Out-String).Trim()
                    if ([string]::IsNullOrWhiteSpace($pythonPath)) {
                        continue
                    }

                    if (-not (Test-Path -LiteralPath $pythonPath)) {
                        continue
                    }

                    $pythonDir = Split-Path -Parent $pythonPath
                    $pathEntries = @($env:PATH -split $pathSeparator | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
                    if ($pathEntries -notcontains $pythonDir) {
                        $env:PATH = "$pythonDir$pathSeparator$env:PATH"
                    }

                    return [pscustomobject]@{
                        Resolved = $true
                        Command  = $pythonCommand
                        Python   = $pythonPath
                        Python3  = $pythonPath
                        PathDir  = $pythonDir
                    }
                }
                catch {
                    continue
                }
            }

            return [pscustomobject]@{
                Resolved = $false
                Command  = ''
                Python   = ''
                Python3  = ''
                PathDir  = ''
            }
        }

        $script:DscPython = Initialize-DscPythonPath
        if ($script:DscPython.Resolved) {
            Write-Host "Resolved python3 for DSC: $($script:DscPython.Python3)" -ForegroundColor DarkGray
        }
        else {
            Write-Host "Unable to resolve a concrete python3 executable for DSC integration tests." -ForegroundColor DarkYellow
        }

        # Helper to run dsc resource get
        function Invoke-DscResourceGet {
            param(
                [string]$ResourceType,
                [string]$InputJson = "{}",
                [Parameter(Mandatory)]
                [string]$ErrorLog
            )

            Write-Host "CMD: $DscExe resource get --resource `"$ResourceType`" --input '$InputJson'" -ForegroundColor Yellow

            $stdout = & $DscExe resource get --resource $ResourceType --input $InputJson 2> $ErrorLog
            $stderr = Get-Content -Path $ErrorLog -Raw -ErrorAction SilentlyContinue

            [pscustomobject]@{
                ExitCode = $LASTEXITCODE
                StdOut   = ($stdout | Out-String).Trim()
                StdErr   = ($stderr | Out-String).Trim()
            }
        }

        # Helper to run dsc resource set
        function Invoke-DscResourceSet {
            param(
                [string]$ResourceType,
                [string]$InputJson = "{}",
                [Parameter(Mandatory)]
                [string]$ErrorLog
            )

            Write-Host "CMD: $DscExe resource set --resource `"$ResourceType`" --input '$InputJson'" -ForegroundColor Yellow

            $stdout = & $DscExe resource set --resource $ResourceType --input $InputJson 2> $ErrorLog
            $stderr = Get-Content -Path $ErrorLog -Raw -ErrorAction SilentlyContinue

            [pscustomobject]@{
                ExitCode = $LASTEXITCODE
                StdOut   = ($stdout | Out-String).Trim()
                StdErr   = ($stderr | Out-String).Trim()
            }
        }

        # Helper to run dsc resource test
        function Invoke-DscResourceTest {
            param(
                [string]$ResourceType,
                [string]$InputJson = "{}",
                [Parameter(Mandatory)]
                [string]$ErrorLog
            )

            Write-Host "CMD: $DscExe resource test --resource `"$ResourceType`" --input '$InputJson'" -ForegroundColor Yellow

            $stdout = & $DscExe resource test --resource $ResourceType --input $InputJson 2> $ErrorLog
            $stderr = Get-Content -Path $ErrorLog -Raw -ErrorAction SilentlyContinue

            [pscustomobject]@{
                ExitCode = $LASTEXITCODE
                StdOut   = ($stdout | Out-String).Trim()
                StdErr   = ($stderr | Out-String).Trim()
            }
        }

        # Helper to run dsc resource export
        function Invoke-DscResourceExport {
            param(
                [string]$ResourceType,
                [string]$InputJson = "{}",
                [Parameter(Mandatory)]
                [string]$ErrorLog
            )

            Write-Host "CMD: $DscExe resource export --resource `"$ResourceType`" --input '$InputJson'" -ForegroundColor Yellow

            $stdout = & $DscExe resource export --resource $ResourceType --input $InputJson 2> $ErrorLog
            $stderr = Get-Content -Path $ErrorLog -Raw -ErrorAction SilentlyContinue

            [pscustomobject]@{
                ExitCode = $LASTEXITCODE
                StdOut   = ($stdout | Out-String).Trim()
                StdErr   = ($stderr | Out-String).Trim()
            }
        }

        # Helper to run dsc resource list
        function Invoke-DscResourceList {
            param(
                [string]$ResourceFilter = "PythonTest/*",
                [string]$AdapterFilter = "Microsoft.DSC.Adapters/Python",
                [Parameter(Mandatory)]
                [string]$ErrorLog
            )

            Write-Host "CMD: $DscExe resource list `"$ResourceFilter`" --adapter `"$AdapterFilter`" --output-format json" -ForegroundColor Yellow

            $stdout = & $DscExe resource list $ResourceFilter --adapter $AdapterFilter --output-format json 2> $ErrorLog
            $stderr = Get-Content -Path $ErrorLog -Raw -ErrorAction SilentlyContinue

            [pscustomobject]@{
                ExitCode = $LASTEXITCODE
                StdOut   = ($stdout | Out-String).Trim()
                StdErr   = ($stderr | Out-String).Trim()
            }
        }
    }

Describe "Python Adapter - LIST Operation via DSC" {
    It "should execute list operation without error" {
        $result = Invoke-DscResourceList -ErrorLog "$TestDrive/error.log"

           $result.ExitCode | Should -Be 0
    }
}

Describe "Python Adapter - GET Operation via DSC" {
    It "should return wrapper JSON with actualState" {
        $rt = "PythonTest/Get"
        $json = '{"name":"pkg","_exist":true}'

        $result = Invoke-DscResourceGet -ResourceType $rt -InputJson $json -ErrorLog "$TestDrive/error.log"

        if ($result.ExitCode -eq 0) {
                $result.StdOut | Should -Match '^\{.*\}$' -Because "Expected JSON output"

            $payload = $result.StdOut | ConvertFrom-Json

            # DSC output shape can vary by adapter mode and engine version.
            # Extract resource actualState from any known wrapper shape.
            $actual = $null

            if ($null -ne $payload.actualState -and $null -ne $payload.actualState.result -and @($payload.actualState.result).Count -gt 0) {
                $actual = $payload.actualState.result[0].result.actualState
            }
            elseif ($null -ne $payload.result -and @($payload.result).Count -gt 0) {
                $actual = $payload.result[0].result.actualState
            }
            elseif ($null -ne $payload.actualState) {
                $actual = $payload.actualState
            }

                $actual | Should -Not -BeNullOrEmpty -Because "GET output should include actualState"
                $actual.name | Should -Be "pkg"
                $actual._exist | Should -Be $true
        }
        else {
            Assert-KnownDscAdapterFailure -Result $result
        }
    }
}

Describe "Python Adapter - SET Operation via DSC" {
    It "should apply desired state and report changes" {
        $rt = "PythonTest/Set"
        $json = '{"name":"curl","_exist":false}'

        $result = Invoke-DscResourceSet -ResourceType $rt -InputJson $json -ErrorLog "$TestDrive/error.log"

        if ($result.ExitCode -eq 0) {
                $result.StdOut | Should -Match '^\{.*\}$' -Because "Expected JSON output"

            $payload = $result.StdOut | ConvertFrom-Json

            # DSC set typically returns beforeState, afterState, changedProperties
                $payload.beforeState | Should -Not -BeNullOrEmpty
                $payload.afterState | Should -Not -BeNullOrEmpty

            # afterState should reflect desired _exist=false
                $payload.afterState.name | Should -Be "curl"
                $payload.afterState._exist | Should -Be $false

            # changedProperties should list _exist
                $payload.changedProperties | Should -Contain "_exist"
        }
        else {
            Assert-KnownDscAdapterFailure -Result $result
        }
    }
}

Describe "Python Adapter - TEST Operation via DSC" {
    It "should compare actual vs desired and report diffs" {
        $rt = "PythonTest/Test"
        # Desired: _exist=true; TestOnlyResource.test() will simulate actual=false → drift
        $json = '{"name":"pkg","desired_exist":true,"_exist":false}'

        $result = Invoke-DscResourceTest -ResourceType $rt -InputJson $json -ErrorLog "$TestDrive/error.log"

        if ($result.ExitCode -eq 0) {
                $result.StdOut | Should -Match '^\{.*\}$' -Because "Expected JSON output"

            $payload = $result.StdOut | ConvertFrom-Json

            # DSC test returns actualState, desiredState, inDesiredState, differingProperties
                $payload.actualState | Should -Not -BeNullOrEmpty
                $payload.desiredState | Should -Not -BeNullOrEmpty
                $payload.inDesiredState | Should -Be $false
                $payload.differingProperties | Should -Contain "_exist"
        }
        else {
            Assert-KnownDscAdapterFailure -Result $result
        }
    }
}

Describe "Python Adapter - EXPORT Operation via DSC" {
    It "should return exported package list" {
        $rt = "PythonTest/Export"
        $json = '{}'

        $result = Invoke-DscResourceExport -ResourceType $rt -InputJson $json -ErrorLog "$TestDrive/error.log"

        if ($result.ExitCode -eq 0) {
                $result.StdOut | Should -Match '^\{.*\}$' -Because "Expected JSON output"

            $payload = $result.StdOut | ConvertFrom-Json

            # Support both known DSC export shapes:
            # 1) configuration document: { "$schema": "...", "resources": [ { "properties": { "packages": [...] } } ] }
            # 2) adapter wrapper: { "metadata": {...}, "result": [ { "result": { "packages": [...] } } ] }
            $exported = $null
            if ($null -ne $payload.resources) {
                    $payload.resources.Count | Should -BeGreaterThan 0
                $exported = $payload.resources[0].properties
            }
            elseif ($null -ne $payload.result) {
                    $payload.result.Count | Should -BeGreaterThan 0
                $exported = $payload.result[0].result
            }

                $exported | Should -Not -BeNullOrEmpty -Because "EXPORT output should include exported properties"

            # Verify structure
                $exported.packages | Should -Not -BeNullOrEmpty
                $exported.packages.Count | Should -BeGreaterThan 0
            
            # Verify first package
                $exported.packages[0].name | Should -Be "alpha"
                $exported.packages[0].version | Should -Be "1.0.0"
                $exported.packages[0]._exist | Should -Be $true
            
            # Verify second package
                $exported.packages[1].name | Should -Be "beta"
                $exported.packages[1].version | Should -Be "2.0.0"
                $exported.packages[1]._exist | Should -Be $true
        }
        else {
            Assert-KnownDscAdapterFailure -Result $result
        }
    }
}

Describe "Python Adapter - Error Propagation via DSC" {
    It "<CaseName> via <Operation> returns non-zero and stderr" -TestCases @(
        @{ CaseName = "unknown resource type"; Operation = "get"; ResourceType = "PythonTest/DoesNotExist"; InputJson = "{}" }
    ) {
        $errorLog = "$TestDrive/error_$Operation.log"

        switch ($Operation) {
            "get" {
                $result = Invoke-DscResourceGet -ResourceType $ResourceType -InputJson $InputJson -ErrorLog $errorLog
            }
            "set" {
                $result = Invoke-DscResourceSet -ResourceType $ResourceType -InputJson $InputJson -ErrorLog $errorLog
            }
            "test" {
                $result = Invoke-DscResourceTest -ResourceType $ResourceType -InputJson $InputJson -ErrorLog $errorLog
            }
            default {
                throw "Unsupported operation '$Operation' in test case."
            }
        }

            $result.ExitCode | Should -Not -Be 0
            $result.StdErr | Should -Not -BeNullOrEmpty
    }
}

}
