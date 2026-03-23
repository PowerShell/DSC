# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    #region    Rust i18n type definitions and functions
    class DscProjectRustTranslationInfo {
        [System.Collections.Generic.Dictionary[string, string]] $Table
        [System.Collections.Generic.HashSet[string]] $DuplicateTranslations
        [System.Collections.Generic.HashSet[string]] $MissingTranslations
        [System.Collections.Generic.HashSet[string]] $UnusedTranslations
        [System.Collections.Generic.HashSet[string]] $UsedTranslations

        [string] GetTranslationString([string]$translationKey) {
            if ($null -eq $this.Table) {
                $this.Initialize()
            }
            return $this.Table[$translationKey]
        }

        [void] Initialize() {
            if ($null -ne $this.Table) {
                return
            }
            $this.Table = [System.Collections.Generic.Dictionary[string, string]]::new()
        }

        [void] ProcessData(
            [System.Collections.Specialized.OrderedDictionary]$data,
            [string]$prefix,
            [int]$version
        ) {
            foreach ($key in $data.Keys) {
                $keyData = $data[$key]
                $workingKey = @($prefix, $key) -join '.'

                if ($keyData -is [System.Collections.Specialized.OrderedDictionary]) {
                    $this.ProcessData($keyData, $workingKey, $version)
                } elseif ($keyData -is [string]) {
                    if ($key -match '^[a-zA-Z]+-[a-zA-Z]+' -and $version -eq 2) {
                        if ($key -eq 'en-us') {
                            if (-not [string]::IsNullOrEmpty($this.Table[$prefix])) {
                                $this.DuplicateTranslations.Add($prefix)
                            }
                            $this.Table[$prefix] = $keyData
                        }
                        continue
                    }

                    if (-not [string]::IsNullOrEmpty($this.Table[$workingKey])) {
                        $this.DuplicateTranslations.Add($workingKey)
                    }
                    $this.Table[$workingKey] = $keyData
                }
            }
        }

        [void] LoadData([System.Collections.Specialized.OrderedDictionary]$data) {
            $this.Initialize()

            [ValidateRange(1,2)][int]$version = 1
            if ($data['_version'] -is [int]) {
                $version = $data['_version']
            }

            foreach ($key in $data.Keys) {
                if ($key -eq '_version') {
                    continue
                }
                $keyData = $data[$key]

                if ($keyData -is [string]) {
                    if (-not [string]::IsNullOrEmpty($this.Table[$key])) {
                        $this.DuplicateTranslations.Add($key)
                    }
                    $this.Table[$key] = $keyData
                } elseif ($keyData -is [System.Collections.Specialized.OrderedDictionary]) {
                    $this.ProcessData($keyData, $key, $version)
                }
            }
        }

        [void] LoadFile([System.IO.FileInfo]$file) {
            $content   = Get-Content -Path $file.FullName -Raw
            $extension = $file.Extension.Substring(1)

            $fileData = switch ($extension) {
                'toml' {
                    $content | PSToml\ConvertFrom-Toml
                    break
                }
                'yaml' {
                    $content | YaYaml\ConvertFrom-Yaml
                    break
                }
                default {
                    throw "Unsupported translation file format '$extension' - must be TOML or YAML."
                }
            }

            $this.LoadData($fileData)
        }

        [void] CheckTranslations([System.IO.DirectoryInfo]$projectFolder) {
            $this.UsedTranslations    = Get-TranslationKey -ProjectDirectory $projectFolder
            $definedKeys              = [System.Collections.Generic.HashSet[string]]$this.Table.Keys
            $this.MissingTranslations = $this.UsedTranslations.Where({ $_ -notin $definedKeys })
            $this.UnusedTranslations  = $definedKeys.Where({ $_ -notin $this.UsedTranslations })
        }

        DscProjectRustTranslationInfo([System.Collections.Specialized.OrderedDictionary]$data) {
            $this.LoadData($data)
        }

        DscProjectRustTranslationInfo([System.IO.FileInfo]$file) {
            $this.LoadFile($file)
        }
        DscProjectRustTranslationInfo([System.IO.DirectoryInfo]$directory) {
            $localesFolder = if ($directory.BaseName -eq 'locales') {
                $directory
            } else {
                Join-Path -Path $directory -ChildPath 'locales'
            }
            $projectFolder = Split-Path -Path $localesFolder -Parent

            if (-not (Test-Path $localesFolder)) {
                throw "Unable to find valid locales folder from in directory '$directory'"
            }

            $tomlFile = Join-Path -Path $localesFolder -ChildPath 'en-us.toml'
            if (Test-Path -Path $tomlFile) {
                $this.LoadFile((Get-Item -Path $tomlFile))
            }
            $yamlFiles = Get-ChildItem -Path $localesFolder | Where-Object Extension -match 'ya?ml'
            foreach ($yamlFile in $yamlFiles) {
                $this.LoadFile($yamlFile)
            }

            $this.CheckTranslations($projectFolder)
        }
    }

    function Get-TranslationKey {
        [cmdletbinding()]
        [OutputType([string[]])]
        param(
            [Parameter(Mandatory)]
            [string]$ProjectDirectory
        )

        begin {
            $patterns = @{
                t = '(?s)\bt\!\(\s*"(?<key>.*?)".*?\)'
                panic_t = '(?s)\bpanic_t\!\(\s*"(?<key>.*?)".*?\)'
                assert_t = '(?s)\bassert_t\!\(\s*.*?,\s*"(?<key>.*?)".*?\)'
            }
            [string[]]$keys = @()
        }

        process {
            if (-not (Test-Path $ProjectDirectory -PathType Container)) {
                throw "Invalid target, '$ProjectDirectory' isn't a directory or doesn't exist."
            }

            Get-ChildItem -Recurse -Path $ProjectDirectory -Include *.rs -File | ForEach-Object {
                $file = $_
                $content = Get-Content -Path $file -Raw
                foreach ($pattern in $patterns.keys) {
                    ($content | Select-String -Pattern $patterns[$pattern] -AllMatches).Matches | ForEach-Object {
                        if ($null -ne $_) {
                            $key   = $_.Groups['key'].Value
                            $keys += $key
                        }
                    }
                }
            }
        }

        end {
            $keys
        }
    }
    #endregion Rust i18n type definitions and functions

    # Limit the folders to recursively search for rust i18n translation strings
    $rootFolders = @(
        'adapters'
        'dsc'
        'extensions'
        'grammars'
        'lib'
        'pal'
        'resources'
        'tools'
        'y2j'
    )
    $localeFolders = $rootFolders | ForEach-Object -Process {
        Get-ChildItem $PSScriptRoot/../../$_/locales -Recurse -Directory
    }
    
    $projects = @()
    $localeFolders | ForEach-Object -Process {
        $projects   += @{
            project         = Split-Path $_ -Parent
            translationInfo = [DscProjectRustTranslationInfo]::new($_)
        }
    }
}

Describe 'Internationalization tests' {
    Context '<project>' -ForEach $projects {
        It 'Uses translation strings' {
            $check = @{
                Not           = $true
                BeNullOrEmpty = $true
                Because       = "'$project' defines at least one translation file"
            }
            $translationInfo.UsedTranslations | Should @check
        }
        It 'Does not define any duplicate translation strings' {
            $check = @{
                BeNullOrEmpty = $true
                Because = (@(
                    "The following translation keys are defined more than once:"
                    $translationInfo.DuplicateTranslations | ConvertTo-Json -Depth 2
                ) -join ' ')
            }

            $translationInfo.DuplicateTranslations | Should @check
        }

        It 'Uses every defined translation string' {
            $check = @{
                BeNullOrEmpty = $true
                Because = (@(
                    "The following translation keys are defined but not used:"
                    $translationInfo.UnusedTranslations | ConvertTo-Json -Depth 2
                ) -join ' ')
            }
            $translationInfo.UnusedTranslations | Should @check
        }

        It 'Defines every used translation string' {
            $check = @{
                BeNullOrEmpty = $true
                Because = (@(
                    "The following translation keys are used but not defined:"
                    $translationInfo.MissingTranslations | ConvertTo-Json -Depth 2
                ) -join ' ')
            }
            $translationInfo.MissingTranslations | Should @check
        }
    }
}
