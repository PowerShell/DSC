# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'WMI adapter resource tests' {

    BeforeAll {
        if ($IsWindows)
        {
            $OldPSModulePath = $env:PSModulePath
            $env:PSModulePath += ";" + $PSScriptRoot

            $configPath = Join-path $PSScriptRoot "test_wmi_config.dsc.yaml"
        }
    }
    AfterAll {
        if ($IsWindows)
        {
            $env:PSModulePath = $OldPSModulePath
        }
    }

    Context 'List WMI resources' {
        It 'List shows WMI resources' -Skip:(!$IsWindows) {

            $r = dsc resource list *OperatingSystem* -a Microsoft.Windows/WMI
            $LASTEXITCODE | Should -Be 0
            $res = $r | ConvertFrom-Json
            $res.Count | Should -BeGreaterOrEqual 1
        }   
    }

    Context 'Get WMI resources' {
        It 'Get works on an individual WMI resource' -Skip:(!$IsWindows) {

            $r = dsc resource get -r root.cimv2/Win32_OperatingSystem
            $LASTEXITCODE | Should -Be 0
            $res = $r | ConvertFrom-Json
            $res.actualState.result.type | Should -BeLike "*Win32_OperatingSystem"
        }
    
        It 'Get works on a config with WMI resources' -Skip:(!$IsWindows) {
    
            $r = Get-Content -Raw $configPath | dsc config get
            $LASTEXITCODE | Should -Be 0
            $res = $r | ConvertFrom-Json
            $res.results.result.actualstate.result[0].properties.LastBootUpTime | Should -Not -BeNull
            $res.results.result.actualstate.result[0].properties.Caption | Should -Not -BeNull
            $res.results.result.actualstate.result[0].properties.NumberOfProcesses | Should -Not -BeNull
        }
    
        It 'Example config works' -Skip:(!$IsWindows) {
            $configPath = Join-Path $PSScriptRoot '..\..\dsc\examples\wmi.dsc.yaml'
            $r = dsc config get -p $configPath
            $LASTEXITCODE | Should -Be 0
            $r | Should -Not -BeNullOrEmpty
            $res = $r | ConvertFrom-Json
            $res.results.result.actualstate.result[0].properties.Model | Should -Not -BeNullOrEmpty
            $res.results.result.actualstate.result[0].properties.Description | Should -Not -BeNullOrEmpty
        }
    }

    # TODO: work on set test configs
    Context "Set WMI resources" {
        It 'Set a resource' -Skip:(!$IsWindows) {
            $inputs = @{
                adapted_dsc_type = "root.cimv2/Win32_Process"
                properties       = @{
                    MethodName  = 'Create'
                    CommandLine = 'powershell.exe'
                }
            }
            # get the start of processes
            $ref = Get-Process 

            # run the creation of process
            $r = ($inputs | ConvertTo-Json -Compress) | dsc resource set -r root.cimv2/Win32_Process

            # handle the output as we do not have a filter yet on the get method
            $diff = Get-Process

            $comparison = (Compare-Object -ReferenceObject $ref -DifferenceObject $diff | Where-Object { $_.SideIndicator -eq '=>' })
            $process = foreach ($c in $comparison)
            {
                if ($c.InputObject.Path -like "*$($inputs.properties.CommandLine)*")
                {
                    $c.InputObject
                }
            }
            $res = $r | ConvertFrom-Json
            $res.afterState.result | Should -Not -BeNull
            $LASTEXITCODE | Should -Be 0
            $process | Should -Not -BeNullOrEmpty
            $process.Path | Should -BeLike "*powershell.exe*"
        }
        AfterAll {
            $process = Get-Process -Name "powershell" -ErrorAction SilentlyContinue | Sort-Object StartTime -Descending -Top 1
            Stop-Process $process
        }
    }

    Context "Export WMI resources" {
        It 'Exports all resources' -Skip:(!$IsWindows) {
            $r = dsc resource export -r root.cimv2/Win32_Process
            $LASTEXITCODE | Should -Be 0
            $res = $r | ConvertFrom-Json
            $res.resources.properties.result.properties.value.count | Should -BeGreaterThan 1
            $res.resources.properties.result.properties.value[0].CreationClassName | Should -Be 'Win32_Process'
        }
    }
}
