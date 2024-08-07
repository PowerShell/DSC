BeforeDiscovery {
    $projectPath = "$($PSScriptRoot)\..\.." | Convert-Path

    <#
        If the QA tests are run outside of the build script (e.g with Invoke-Pester)
        the parent scope has not set the variable $ProjectName.
    #>
    if (-not $ProjectName)
    {
        # Assuming project folder name is project name.
        $ProjectName = Get-SamplerProjectName -BuildRoot $projectPath
    }

    $script:moduleName = $ProjectName

    Remove-Module -Name $script:moduleName -Force -ErrorAction SilentlyContinue

    $mut = Get-Module -Name $script:moduleName -ListAvailable |
        Select-Object -First 1 |
            Import-Module -Force -ErrorAction Stop -PassThru
}

BeforeAll {
    # Convert-Path required for PS7 or Join-Path fails
    $projectPath = "$($PSScriptRoot)\..\.." | Convert-Path

    <#
        If the QA tests are run outside of the build script (e.g with Invoke-Pester)
        the parent scope has not set the variable $ProjectName.
    #>
    if (-not $ProjectName)
    {
        # Assuming project folder name is project name.
        $ProjectName = Get-SamplerProjectName -BuildRoot $projectPath
    }

    $script:moduleName = $ProjectName

    $sourcePath = (
        Get-ChildItem -Path $projectPath\*\*.psd1 |
            Where-Object -FilterScript {
                ($_.Directory.Name -match 'source|src' -or $_.Directory.Name -eq $_.BaseName) `
                    -and $(
                    try
                    {
                        Test-ModuleManifest -Path $_.FullName -ErrorAction Stop
                    }
                    catch
                    {
                        $false
                    }
                )
            }
    ).Directory.FullName
}

Describe 'Changelog Management' -Tag 'Changelog' {
    It 'Changelog has been updated' -Skip:(
        -not ([bool](Get-Command git -ErrorAction SilentlyContinue) -and
            [bool](&(Get-Process -Id $PID).Path -NoProfile -Command 'git rev-parse --is-inside-work-tree 2>$null'))
    ) {
        <#
            Get the list of changed files compared with branch main to verify
            that required files are changed.
        #>

        # Only run if there is a remote called origin
        if (((git remote) -match 'origin'))
        {
            $headCommit = &git rev-parse HEAD
            $defaultBranchCommit = &git rev-parse origin/main
            $filesChanged = &git @('diff', "$defaultBranchCommit...$headCommit", '--name-only')
        }

        $filesStagedAndUnstaged = &git @('diff', 'HEAD', '--name-only') 2>&1

        $filesChanged += $filesStagedAndUnstaged

        # Only check if there are any changed files.
        if ($filesChanged)
        {
            $filesChanged | Should -Contain 'CHANGELOG.md' -Because 'the CHANGELOG.md must be updated with at least one entry in the Unreleased section for each PR'
        }
    }

    It 'Changelog format compliant with keepachangelog format' -Skip:(![bool](Get-Command git -EA SilentlyContinue)) {
        { Get-ChangelogData -Path (Join-Path $ProjectPath 'CHANGELOG.md') -ErrorAction Stop } | Should -Not -Throw
    }

    It 'Changelog should have an Unreleased header' -Skip:$skipTest {
            (Get-ChangelogData -Path (Join-Path -Path $ProjectPath -ChildPath 'CHANGELOG.md') -ErrorAction Stop).Unreleased | Should -Not -BeNullOrEmpty
    }
}

Describe 'General module control' -Tags 'FunctionalQuality' {
    It 'Should import without errors' {
        { Import-Module -Name $script:moduleName -Force -ErrorAction Stop } | Should -Not -Throw

        Get-Module -Name $script:moduleName | Should -Not -BeNullOrEmpty
    }

    It 'Should remove without error' {
        { Remove-Module -Name $script:moduleName -ErrorAction Stop } | Should -Not -Throw

        Get-Module $script:moduleName | Should -BeNullOrEmpty
    }
}

BeforeDiscovery {
    # Must use the imported module to build test cases.
    $allModuleFunctions = & $mut { Get-Command -Module $args[0] -CommandType Function } $script:moduleName

    # Build test cases.
    $testCases = @()

    foreach ($function in $allModuleFunctions)
    {
        $testCases += @{
            Name = $function.Name
        }
    }
}

Describe 'Quality for module' -Tags 'TestQuality' {
    BeforeDiscovery {
        if (Get-Command -Name Invoke-ScriptAnalyzer -ErrorAction SilentlyContinue)
        {
            $scriptAnalyzerRules = Get-ScriptAnalyzerRule
        }
        else
        {
            if ($ErrorActionPreference -ne 'Stop')
            {
                Write-Warning -Message 'ScriptAnalyzer not found!'
            }
            else
            {
                throw 'ScriptAnalyzer not found!'
            }
        }
    }

    It 'Should have a unit test for <Name>' -ForEach $testCases {
        Get-ChildItem -Path 'tests\' -Recurse -Include "$Name.Tests.ps1" | Should -Not -BeNullOrEmpty
    }

    It 'Should pass Script Analyzer for <Name>' -ForEach $testCases -Skip:(-not $scriptAnalyzerRules) {
        $functionFile = Get-ChildItem -Path $sourcePath -Recurse -Include "$Name.ps1"

        $pssaResult = (Invoke-ScriptAnalyzer -Path $functionFile.FullName)
        $report = $pssaResult | Format-Table -AutoSize | Out-String -Width 110
        $pssaResult | Should -BeNullOrEmpty -Because `
            "some rule triggered.`r`n`r`n $report"
    }
}

Describe 'Help for module' -Tags 'helpQuality' {
    It 'Should have .SYNOPSIS for <Name>' -ForEach $testCases {
        $functionFile = Get-ChildItem -Path $sourcePath -Recurse -Include "$Name.ps1"

        $scriptFileRawContent = Get-Content -Raw -Path $functionFile.FullName

        $abstractSyntaxTree = [System.Management.Automation.Language.Parser]::ParseInput($scriptFileRawContent, [ref] $null, [ref] $null)

        $astSearchDelegate = { $args[0] -is [System.Management.Automation.Language.FunctionDefinitionAst] }

        $parsedFunction = $abstractSyntaxTree.FindAll( $astSearchDelegate, $true ) |
            Where-Object -FilterScript {
                $_.Name -eq $Name
            }

        $functionHelp = $parsedFunction.GetHelpContent()

        $functionHelp.Synopsis | Should -Not -BeNullOrEmpty
    }

    It 'Should have a .DESCRIPTION with length greater than 40 characters for <Name>' -ForEach $testCases {
        $functionFile = Get-ChildItem -Path $sourcePath -Recurse -Include "$Name.ps1"

        $scriptFileRawContent = Get-Content -Raw -Path $functionFile.FullName

        $abstractSyntaxTree = [System.Management.Automation.Language.Parser]::ParseInput($scriptFileRawContent, [ref] $null, [ref] $null)

        $astSearchDelegate = { $args[0] -is [System.Management.Automation.Language.FunctionDefinitionAst] }

        $parsedFunction = $abstractSyntaxTree.FindAll($astSearchDelegate, $true) |
            Where-Object -FilterScript {
                $_.Name -eq $Name
            }

        $functionHelp = $parsedFunction.GetHelpContent()

        $functionHelp.Description.Length | Should -BeGreaterThan 40
    }

    It 'Should have at least one (1) example for <Name>' -ForEach $testCases {
        $functionFile = Get-ChildItem -Path $sourcePath -Recurse -Include "$Name.ps1"

        $scriptFileRawContent = Get-Content -Raw -Path $functionFile.FullName

        $abstractSyntaxTree = [System.Management.Automation.Language.Parser]::ParseInput($scriptFileRawContent, [ref] $null, [ref] $null)

        $astSearchDelegate = { $args[0] -is [System.Management.Automation.Language.FunctionDefinitionAst] }

        $parsedFunction = $abstractSyntaxTree.FindAll( $astSearchDelegate, $true ) |
            Where-Object -FilterScript {
                $_.Name -eq $Name
            }

        $functionHelp = $parsedFunction.GetHelpContent()

        $functionHelp.Examples.Count | Should -BeGreaterThan 0
        $functionHelp.Examples[0] | Should -Match ([regex]::Escape($function.Name))
        $functionHelp.Examples[0].Length | Should -BeGreaterThan ($function.Name.Length + 10)

    }

    It 'Should have described all parameters for <Name>' -ForEach $testCases {
        $functionFile = Get-ChildItem -Path $sourcePath -Recurse -Include "$Name.ps1"

        $scriptFileRawContent = Get-Content -Raw -Path $functionFile.FullName

        $abstractSyntaxTree = [System.Management.Automation.Language.Parser]::ParseInput($scriptFileRawContent, [ref] $null, [ref] $null)

        $astSearchDelegate = { $args[0] -is [System.Management.Automation.Language.FunctionDefinitionAst] }

        $parsedFunction = $abstractSyntaxTree.FindAll( $astSearchDelegate, $true ) |
            Where-Object -FilterScript {
                $_.Name -eq $Name
            }

        $functionHelp = $parsedFunction.GetHelpContent()

        $parameters = $parsedFunction.Body.ParamBlock.Parameters.Name.VariablePath.ForEach({ $_.ToString() })

        foreach ($parameter in $parameters)
        {
            $functionHelp.Parameters.($parameter.ToUpper()) | Should -Not -BeNullOrEmpty -Because ('the parameter {0} must have a description' -f $parameter)
            $functionHelp.Parameters.($parameter.ToUpper()).Length | Should -BeGreaterThan 25 -Because ('the parameter {0} must have descriptive description' -f $parameter)
        }
    }
}

