# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'dsc graph sample plug-in tests' {
    BeforeAll {
        $script:schema = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'

        function New-TestDocument {
            <#
                .SYNOPSIS
                Writes a configuration document with the resources as JSON and returns its path.
            #>
            param(
                [Parameter(Mandatory)][string]$Name,
                [Parameter(Mandatory)][object[]]$Resources
            )

            $path = Join-Path $TestDrive $Name
            [ordered]@{ '$schema' = $script:schema; resources = $Resources } |
                ConvertTo-Json -Depth 20 |
                Set-Content -Path $path
            $path
        }

        $script:osId = "[resourceId('Microsoft/OSInfo','os')]"
        $script:reference = "[concat('The OS is ', reference(resourceId('Microsoft/OSInfo','os')).family)]"

        # Echo depends on os through dependsOn and through reference().
        $script:documentPath = New-TestDocument -Name 'reference.dsc.json' -Resources @(
            [ordered]@{ name = 'os'; type = 'Microsoft/OSInfo'; properties = @{} }
            [ordered]@{
                name       = 'Echo'
                type       = 'Microsoft.DSC.Debug/Echo'
                properties = @{ output = $script:reference }
                dependsOn  = @($script:osId)
            }
        )
        $script:expected = @(
            'flowchart TD'
            '    n0["os<br/>Microsoft/OSInfo"]'
            '    n1["Echo<br/>Microsoft.DSC.Debug/Echo"]'
            '    n0 --> n1'
        )

        $script:yamlPath = Join-Path $TestDrive 'reference.dsc.yaml'
        Set-Content -Path $script:yamlPath -Value @"
`$schema: $script:schema
resources:
- name: os
  type: Microsoft/OSInfo
  properties: {}
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "$script:reference"
  dependsOn:
  - "$script:osId"
"@
    }

    Context 'Finding the plug-in' {
        It 'Is listed as the plug-in that provides the graph command' {
            $out = dsc plugin list 'Microsoft.DSC.Samples/Graph' 2> $TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out.command | Should -BeExactly 'graph'
            $out.shadowed | Should -BeFalse
            $out.manifest.command.executable | Should -BeExactly 'pwsh'
        }

        It 'Is listed in the top-level help' {
            $out = dsc --help 2> $TestDrive/error.txt | Out-String
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -Match '(?m)^  graph\s+Show the dependency graph of a configuration document\r?$'
        }

        It 'Shows its help for: dsc <arguments>' -TestCases @(
            @{ arguments = @('help', 'graph'); usage = 'Usage: dsc graph [OPTIONS]' }
            @{ arguments = @('graph', '--help'); usage = 'Usage: dsc graph [OPTIONS]' }
            @{ arguments = @('Microsoft.DSC.Samples/Graph', '--help'); usage = 'Usage: dsc Microsoft.DSC.Samples/Graph [OPTIONS]' }
        ) {
            param($arguments, $usage)

            $out = dsc @arguments 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -Contain $usage
            ($out | Out-String) | Should -Match '--format <FORMAT>'
        }
    }

    Context 'Reading the document' {
        It 'Reads a JSON document from a file' {
            $out = dsc graph --file $documentPath 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -BeExactly $expected
        }

        It 'Reads a YAML document from a file' {
            $out = dsc graph --file $yamlPath 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -BeExactly $expected
        }

        It 'Reads YAML with powershell-yaml, then Yayaml, then DSC' {
            $module = 'powershell-yaml', 'Yayaml' |
                Where-Object { Get-Module -ListAvailable -Name $_ } |
                Select-Object -First 1
            $reader = if ($module) { "with the '$module' module" } else { 'No YAML module found' }

            $out = dsc --trace-level debug graph --file $yamlPath 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -BeExactly $expected
            (Get-Content -Raw $TestDrive/error.txt) | Should -Match ([regex]::Escape($reader))
        }

        It 'Returns exit code 2 for a file with more than one YAML document' {
            $path = Join-Path $TestDrive 'multiple.dsc.yaml'
            Set-Content -Path $path -Value "resources: []`n---`nresources: []"

            $out = dsc graph --file $path 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 2
            $out | Should -BeNullOrEmpty
        }

        It 'Returns exit code 2 for a document that is not valid YAML' {
            $path = Join-Path $TestDrive 'invalid.dsc.yaml'
            Set-Content -Path $path -Value "resources: [`n  name: }"

            $out = dsc graph --file $path 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 2
            $out | Should -BeNullOrEmpty
        }

        It 'Reads the document from standard input when: <name>' -TestCases @(
            @{ name = 'no option is used'; arguments = @() }
            @{ name = 'the file is -'; arguments = @('--file', '-') }
        ) {
            param($arguments)

            $out = Get-Content -Raw $documentPath | dsc graph @arguments 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -BeExactly $expected
        }

        It 'Shows the resources of an operation result without dependencies' {
            $result = [ordered]@{
                results = @(
                    [ordered]@{ name = 'os'; type = 'Microsoft/OSInfo'; result = @{ actualState = @{} } }
                    [ordered]@{ name = 'Echo'; type = 'Microsoft.DSC.Debug/Echo'; result = @{ actualState = @{} } }
                )
                hadErrors = $false
            } | ConvertTo-Json -Depth 10

            $out = $result | dsc --trace-level info graph 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -BeExactly $expected[0..2]
            Get-Content -Raw $TestDrive/error.txt | Should -Match 'INFO graph: The input is the result of an operation'
        }
    }

    Context 'Finding dependencies' {
        It 'Draws a dotted arrow for a reference without dependsOn' {
            $path = New-TestDocument -Name 'reference-only.dsc.json' -Resources @(
                [ordered]@{ name = 'os'; type = 'Microsoft/OSInfo'; properties = @{} }
                [ordered]@{ name = 'Echo'; type = 'Microsoft.DSC.Debug/Echo'; properties = @{ output = $reference } }
            )

            $out = dsc graph --file $path 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -Contain '    n0 -.-> n1'
            $out | Should -Not -Contain '    n0 --> n1'
        }

        It 'Matches the type of a dependency without regard to case' {
            $path = New-TestDocument -Name 'type-case.dsc.json' -Resources @(
                [ordered]@{ name = 'os'; type = 'microsoft/osinfo'; properties = @{} }
                [ordered]@{ name = 'Echo'; type = 'Microsoft.DSC.Debug/Echo'; properties = @{}; dependsOn = @($osId) }
            )

            $out = dsc graph --file $path 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -Contain '    n0 --> n1'
            ($out | Out-String) | Should -Not -Match 'missing'
        }

        It 'Draws groups as sub-graphs and keeps dependencies inside their group' {
            $echo = 'Microsoft.DSC.Debug/Echo'
            $path = New-TestDocument -Name 'groups.dsc.json' -Resources @(
                [ordered]@{
                    name       = 'Last Group'
                    type       = 'Microsoft.DSC/Group'
                    properties = [ordered]@{
                        '$schema' = $schema
                        resources = @(
                            [ordered]@{ name = 'Second'; type = $echo; properties = @{ output = 2 }; dependsOn = @("[resourceId('$echo','First')]") }
                            [ordered]@{ name = 'First'; type = $echo; properties = @{ output = 1 } }
                        )
                    }
                    dependsOn  = @("[resourceId('Microsoft.DSC/Group','First Group')]")
                }
                [ordered]@{
                    name       = 'First Group'
                    type       = 'Microsoft.DSC/Group'
                    properties = [ordered]@{
                        '$schema' = $schema
                        resources = @(
                            [ordered]@{ name = 'First'; type = $echo; properties = @{ output = 0 } }
                        )
                    }
                }
            )

            $out = dsc graph --file $path 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -BeExactly @(
                'flowchart TD'
                '    subgraph n0["Last Group (Microsoft.DSC/Group)"]'
                '        n2["Second<br/>Microsoft.DSC.Debug/Echo"]'
                '        n3["First<br/>Microsoft.DSC.Debug/Echo"]'
                '    end'
                '    subgraph n1["First Group (Microsoft.DSC/Group)"]'
                '        n4["First<br/>Microsoft.DSC.Debug/Echo"]'
                '    end'
                '    n1 --> n0'
                '    n3 --> n2'
            )
        }

        It 'Marks a dependency that is not in the document and warns about it' {
            $path = New-TestDocument -Name 'missing.dsc.json' -Resources @(
                [ordered]@{ name = 'Echo'; type = 'Microsoft.DSC.Debug/Echo'; properties = @{}; dependsOn = @($osId) }
            )

            $out = dsc graph --file $path 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -Contain '    n1["os<br/>Microsoft/OSInfo"]:::missing'
            $out | Should -Contain '    n1 --> n0'
            $out | Should -Contain '    classDef missing stroke-dasharray: 5 5'
            Get-Content -Raw $TestDrive/error.txt | Should -Match "WARN graph: 'Echo' depends on 'os' of type 'Microsoft/OSInfo'"
        }

        It 'Skips a dependency that is not a literal resourceId and warns about it' {
            $path = New-TestDocument -Name 'computed.dsc.json' -Resources @(
                [ordered]@{ name = 'os'; type = 'Microsoft/OSInfo'; properties = @{} }
                [ordered]@{
                    name       = 'Echo'
                    type       = 'Microsoft.DSC.Debug/Echo'
                    properties = @{}
                    dependsOn  = @("[resourceId('Microsoft/OSInfo', concat('o', 's'))]")
                }
            )

            $out = dsc graph --file $path 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            ($out | Out-String) | Should -Not -Match '-->'
            Get-Content -Raw $TestDrive/error.txt | Should -Match 'WARN graph: Skipped the dependency'
        }

        It 'Does not write warnings when the trace level is error' {
            $path = New-TestDocument -Name 'quiet.dsc.json' -Resources @(
                [ordered]@{ name = 'Echo'; type = 'Microsoft.DSC.Debug/Echo'; properties = @{}; dependsOn = @($osId) }
            )

            $null = dsc --trace-level error graph --file $path 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0
            Get-Content -Raw $TestDrive/error.txt | Should -Not -Match 'graph:'
        }
    }

    Context 'Writing the output' {
        It 'Uses the direction for the flowchart' {
            $out = dsc graph --file $documentPath --direction LR 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out[0] | Should -BeExactly 'flowchart LR'
        }

        It 'Writes the nodes and edges as JSON' {
            $out = dsc graph --file $documentPath --format json 2> $TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out.nodes.Count | Should -Be 2
            $out.nodes[0].id | Should -BeExactly 'n0'
            $out.nodes[0].name | Should -BeExactly 'os'
            $out.nodes[0].type | Should -BeExactly 'Microsoft/OSInfo'
            $out.edges.Count | Should -Be 1
            $out.edges[0].from | Should -BeExactly 'n0'
            $out.edges[0].to | Should -BeExactly 'n1'
            $out.edges[0].kind | Should -BeExactly 'dependsOn'
        }

        It 'Escapes characters that end a Mermaid label' {
            $path = New-TestDocument -Name 'escape.dsc.json' -Resources @(
                [ordered]@{ name = 'Say "hi" <now> #1'; type = 'Microsoft.DSC.Debug/Echo'; properties = @{} }
            )

            $out = dsc graph --file $path 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out[1] | Should -BeExactly '    n0["Say #quot;hi#quot; #lt;now#gt; #35;1<br/>Microsoft.DSC.Debug/Echo"]'
        }
    }

    Context 'Exit codes' {
        It 'Returns exit code 1 for invalid arguments: <name>' -TestCases @(
            @{ name = 'unknown option'; arguments = @('--unknown'); message = "Unknown argument '--unknown'" }
            @{ name = 'unknown format'; arguments = @('--format', 'dot'); message = "Unknown format 'dot'" }
            @{ name = 'unknown direction'; arguments = @('--direction', 'UP'); message = "Unknown direction 'UP'" }
            @{ name = 'option without a value'; arguments = @('--format'); message = "The option '--format' needs a value" }
            @{ name = 'file and input together'; arguments = @('--file', 'a.json', '--input', '{}'); message = 'Use either --file or --input' }
        ) {
            param($arguments, $message)

            $out = dsc graph @arguments 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 1
            $out | Should -BeNullOrEmpty
            Get-Content -Raw $TestDrive/error.txt | Should -Match ([regex]::Escape("ERROR graph: $message"))
        }

        It 'Returns exit code 2 when the file does not exist' {
            $out = dsc graph --file (Join-Path $TestDrive 'does-not-exist.json') 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 2
            $out | Should -BeNullOrEmpty
            Get-Content -Raw $TestDrive/error.txt | Should -Match "ERROR graph: The file .* doesn't exist"
        }

        It 'Returns exit code 2 when the document has no resources' {
            $out = '{ "parameters": {} }' | dsc graph 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 2
            $out | Should -BeNullOrEmpty
            Get-Content -Raw $TestDrive/error.txt | Should -Match "ERROR graph: The document has no 'resources' or 'results' property"
        }
    }
}
