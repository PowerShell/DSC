# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    $tomls = Get-ChildItem $PSScriptRoot/../../en-us.toml -Recurse -File
    $projects = @()
    $tomls | ForEach-Object {
        $projectName = (Split-Path $_ -Parent | Split-Path -Parent)
        $projects += @{ project = $projectName; toml = $_ }
    }
}

Describe 'Internationalization tests' {
    It 'Project <project> uses i18n strings from <toml>' -TestCases $projects {
        param($project, $toml)

        $i18n = [System.Collections.Hashtable]::new([System.StringComparer]::Ordinal)
        $prefix = ''
        Get-Content -Path $toml | ForEach-Object {
            if ($_ -match '\[(?<prefix>.*?)\]') {
                $prefix = $Matches['prefix']
            }
            elseif ($_ -match '^(?<key>\w+)\s?=\s?"(?<value>.*?)"') {
                $key = $prefix + '.' + $Matches['key']
                $i18n[$key] = 0
            }
        }

        $missing = @()
        Get-ChildItem -Recurse -Path $project -Include *.rs -File | ForEach-Object {
            # Write-Verbose -Verbose "File: $_"
            $line = 0
            Get-Content -Path $_ | ForEach-Object {
                $line++
                ($_ | Select-String -Pattern '[^\w]t\!\("(?<key>.*?)".*?\)' -AllMatches).Matches | ForEach-Object {
                    # write-verbose -verbose "Line: $_"
                    if ($null -ne $_) {
                        $key = $_.Groups['key'].Value
                        if ($i18n.ContainsKey($key)) {
                            $i18n[$key] = 1
                            # write-verbose -verbose "Found on line $line : $key"
                        }
                        else {
                            $missing += $key
                            # write-verbose -verbose "Missing: $key"
                        }
                    }
                }
            }
        }

        $missing | Should -BeNullOrEmpty -Because "The following i18n keys are missing from $toml :`n$($missing | Out-String)"
        $unused = $i18n.GetEnumerator() | Where-Object { $_.Value -eq 0 } | ForEach-Object { $_.Key }
        $unused | Should -BeNullOrEmpty -Because "The following i18n keys are unused in the project:`n$($unused | Out-String)"
    }
}
