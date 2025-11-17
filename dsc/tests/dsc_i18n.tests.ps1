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

        $patterns = @{
            t = '(?s)\bt\!\(\s*"(?<key>.*?)".*?\)'
            panic_t = '(?s)\bpanic_t\!\(\s*"(?<key>.*?)".*?\)'
            assert_t = '(?s)\bassert_t\!\(\s*.*?,\s*"(?<key>.*?)".*?\)'
        }

        $missing = @()
        Get-ChildItem -Recurse -Path $project -Include *.rs -File | ForEach-Object {
            $content = Get-Content -Path $_ -Raw
            foreach ($pattern in $patterns.keys) {
                ($content | Select-String -Pattern $patterns[$pattern] -AllMatches).Matches | ForEach-Object {
                    # write-verbose -verbose "Line: $_"
                    if ($null -ne $_) {
                        $key = $_.Groups['key'].Value
                        if ($i18n.ContainsKey($key)) {
                            $i18n[$key] = 1
                        }
                        else {
                            $missing += $key
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
