# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

<#
    .SYNOPSIS
    Sample DSC command plug-in that shows the dependency graph of a configuration document.

    .DESCRIPTION
    DSC runs this script for `dsc graph`. The script reads a configuration document, finds the
    resources and the dependencies between them, and writes a Mermaid flowchart or a JSON object
    to standard output.

    The script follows the conventions for command plug-ins:

    - It writes the result to standard output and messages to standard error.
    - It honors the trace level and trace format that DSC passes in the `DSC_TRACE_LEVEL` and
      `DSC_TRACE_FORMAT` environment variables.
    - It uses the `DSC_EXE` environment variable to call back into the DSC that started it.
    - It shows its usage with the command the user typed, from `DSC_PLUGIN_COMMAND`.
    - It returns exit code 0 for success, 1 for invalid arguments, and 2 for a document that
      can't be read.
#>

$ErrorActionPreference = 'Stop'

$exitSuccess = 0
$exitInvalidArgs = 1
$exitInvalidDocument = 2

# The trace levels DSC knows, from least to most verbose.
$traceLevels = @('error', 'warn', 'info', 'debug', 'trace')

# PowerShell has no YAML parser of its own. These modules provide one, in order of preference.
$yamlModules = @('powershell-yaml', 'Yayaml')

# Matches a call to resourceId() with a literal type and a literal name.
$resourceIdPattern = "resourceId\(\s*'(?<type>(?:[^']|'')*)'\s*,\s*'(?<name>(?:[^']|'')*)'\s*\)"

function Write-Trace {
    <#
        .SYNOPSIS
        Writes a message to standard error when the trace level DSC passed allows it.
    #>
    param(
        [Parameter(Mandatory)][ValidateSet('error', 'warn', 'info', 'debug', 'trace')][string]$Level,
        [Parameter(Mandatory)][string]$Message
    )

    $configured = "$env:DSC_TRACE_LEVEL".ToLowerInvariant()
    if ($configured -notin $traceLevels) {
        $configured = 'warn'
    }
    if ($traceLevels.IndexOf($Level) -gt $traceLevels.IndexOf($configured)) {
        return
    }

    if ($env:DSC_TRACE_FORMAT -eq 'json') {
        $line = [ordered]@{
            timestamp = [DateTime]::UtcNow.ToString('o')
            level     = $Level.ToUpperInvariant()
            fields    = @{ message = $Message }
        } | ConvertTo-Json -Compress
    } else {
        $line = '{0} graph: {1}' -f $Level.ToUpperInvariant(), $Message
    }
    [Console]::Error.WriteLine($line)
}

function Show-Help {
    <#
        .SYNOPSIS
        Writes the usage of the command to standard output.
    #>
    $command = if ($env:DSC_PLUGIN_COMMAND) { "dsc $env:DSC_PLUGIN_COMMAND" } else { 'graph.ps1' }
    @"
Show the dependency graph of a configuration document

Usage: $command [OPTIONS]

The document is read from standard input unless --file or --input is used. An arrow points
from a resource to the resource that depends on it, which is the order DSC runs them in.

Options:
  -f, --file <FILE>          The configuration document to read, '-' for standard input
  -i, --input <INPUT>        The configuration document as a string
      --format <FORMAT>      The output format: mermaid (default) or json
      --direction <DIR>      The direction of a Mermaid flowchart: TD (default) or LR
  -h, --help                 Print help
"@
}

function Read-Arguments {
    <#
        .SYNOPSIS
        Parses the arguments DSC passed on, and exits with the invalid arguments code on a mistake.
    #>
    param([string[]]$Arguments)

    $options = @{ File = $null; Input = $null; Format = 'mermaid'; Direction = 'TD'; Help = $false }
    for ($i = 0; $i -lt $Arguments.Count; $i++) {
        $argument = $Arguments[$i]
        if ($argument -in '-h', '--help') {
            $options.Help = $true
            continue
        }

        $name = switch ($argument) {
            { $_ -in '-f', '--file' } { 'File' }
            { $_ -in '-i', '--input' } { 'Input' }
            '--format' { 'Format' }
            '--direction' { 'Direction' }
            default {
                Write-Trace -Level error -Message "Unknown argument '$argument'. Use --help to see the options."
                exit $exitInvalidArgs
            }
        }
        if ($i + 1 -ge $Arguments.Count) {
            Write-Trace -Level error -Message "The option '$argument' needs a value."
            exit $exitInvalidArgs
        }
        $options[$name] = $Arguments[++$i]
    }

    if ($options.Format -notin 'mermaid', 'json') {
        Write-Trace -Level error -Message "Unknown format '$($options.Format)'. Use 'mermaid' or 'json'."
        exit $exitInvalidArgs
    }
    if ($options.Direction -notin 'TD', 'LR') {
        Write-Trace -Level error -Message "Unknown direction '$($options.Direction)'. Use 'TD' or 'LR'."
        exit $exitInvalidArgs
    }
    if ($null -ne $options.File -and $null -ne $options.Input) {
        Write-Trace -Level error -Message "Use either --file or --input, not both."
        exit $exitInvalidArgs
    }
    $options
}

function ConvertTo-Document {
    <#
        .SYNOPSIS
        Parses the text of a document as JSON, then as YAML with a module, then by asking DSC.
    #>
    param([Parameter(Mandatory)][string]$Text)

    $document = $null
    try {
        $document = $Text | ConvertFrom-Json -AsHashtable -Depth 100
    } catch {
        Write-Trace -Level debug -Message "The document isn't JSON, reading it as YAML."
    }

    # Read YAML with the first known module that's installed. The command is called by its module
    # name, so the reader is always one of the known modules and never another command that
    # happens to be named ConvertFrom-Yaml.
    $reader = $null
    if ($null -eq $document) {
        foreach ($module in $yamlModules) {
            $reader = Get-Command -Name "$module\ConvertFrom-Yaml" -ErrorAction Ignore
            if ($null -eq $reader) { continue }

            Write-Trace -Level debug -Message "Reading the document as YAML with the '$module' module."
            # Ask powershell-yaml to keep the key order and to return every YAML document.
            $arguments = if ($module -eq 'powershell-yaml') { @{ Ordered = $true; AllDocuments = $true } } else { @{} }
            try {
                $document = $Text | & $reader @arguments -ErrorAction Stop
            } catch {
                Write-Trace -Level error -Message "The document isn't valid JSON or YAML: $($_.Exception.Message)"
                exit $exitInvalidDocument
            }
            break
        }
    }

    # Without a module, ask the DSC that started the plug-in. Its resolve command takes the
    # document as the content of an include and returns it as JSON. DSC looks up the resource
    # types while it resolves, so this needs the resources to be installed.
    if ($null -eq $document -and $null -eq $reader) {
        $dsc = if ($env:DSC_EXE) { $env:DSC_EXE } else { 'dsc' }
        Write-Trace -Level debug -Message "No YAML module found, asking '$dsc' to read the document."
        $include = @{ configurationContent = $Text } | ConvertTo-Json -Compress
        $resolved = $include | & $dsc config resolve --file - --output-format json
        if ($LASTEXITCODE -ne 0) {
            Write-Trace -Level error -Message "DSC couldn't read the document as a configuration document."
            exit $exitInvalidDocument
        }
        $document = ($resolved | ConvertFrom-Json -AsHashtable -Depth 100).configuration
    }

    if ($document -isnot [System.Collections.IDictionary]) {
        Write-Trace -Level error -Message "The document isn't an object."
        exit $exitInvalidDocument
    }
    $document
}

function Get-StringValue {
    <#
        .SYNOPSIS
        Returns every string in a value, however deeply it's nested.
    #>
    param($Value, [string[]]$SkipKey = @())

    if ($Value -is [string]) {
        $Value
    } elseif ($Value -is [System.Collections.IDictionary]) {
        foreach ($key in $Value.Keys) {
            if ($key -notin $SkipKey) {
                Get-StringValue -Value $Value[$key]
            }
        }
    } elseif ($Value -is [System.Collections.IEnumerable]) {
        foreach ($item in $Value) {
            Get-StringValue -Value $item
        }
    }
}

function Get-ScopeKey {
    <#
        .SYNOPSIS
        Returns the key that identifies a resource among its siblings.
    #>
    param([string]$Type, [string]$Name)

    # Type names aren't case-sensitive in DSC, resource names are.
    '{0}|{1}' -f $Type.ToLowerInvariant(), $Name
}

function Add-Node {
    <#
        .SYNOPSIS
        Adds a node to the graph and returns it.
    #>
    param($Graph, [string]$Name, [string]$Type, [string]$Group, [switch]$IsGroup, [switch]$Missing)

    $node = [ordered]@{
        id   = 'n{0}' -f $Graph.nodes.Count
        name = $Name
        type = $Type
    }
    if ($Group) { $node.group = $Group }
    if ($IsGroup) { $node.isGroup = $true }
    if ($Missing) { $node.missing = $true }
    $Graph.nodes.Add($node)
    $node
}

function Add-Edge {
    <#
        .SYNOPSIS
        Adds a dependency to the graph. A dependsOn edge replaces a reference edge for the same pair.
    #>
    param($Graph, [string]$From, [string]$To, [ValidateSet('dependsOn', 'reference')][string]$Kind)

    $existing = $Graph.edges | Where-Object { $_.from -eq $From -and $_.to -eq $To }
    if ($existing) {
        if ($Kind -eq 'dependsOn') { $existing.kind = $Kind }
        return
    }
    $Graph.edges.Add([ordered]@{ from = $From; to = $To; kind = $Kind })
}

function Add-ResourceList {
    <#
        .SYNOPSIS
        Adds the resources of a document or a group to the graph, with their dependencies.
    #>
    param($Graph, $Resources, [string]$Group)

    # Dependencies are looked up among the resources of the same document or group.
    $scope = @{}
    $entries = foreach ($resource in $Resources) {
        if ($resource -isnot [System.Collections.IDictionary]) { continue }
        $nested = $null
        if ($resource['properties'] -is [System.Collections.IDictionary]) {
            $nested = $resource['properties']['resources']
        }
        $isGroup = $nested -is [System.Collections.IEnumerable] -and $nested -isnot [string]
        $node = Add-Node -Graph $Graph -Name "$($resource['name'])" -Type "$($resource['type'])" -Group $Group -IsGroup:$isGroup
        $scope[(Get-ScopeKey -Type $node.type -Name $node.name)] = $node
        @{ Resource = $resource; Node = $node; Nested = if ($isGroup) { $nested } }
    }

    foreach ($entry in $entries) {
        $resource = $entry.Resource
        $dependencies = [System.Collections.Generic.List[object]]::new()

        foreach ($dependsOn in @($resource['dependsOn'])) {
            if ($null -eq $dependsOn) { continue }
            if ("$dependsOn" -match "^\[\s*$resourceIdPattern\s*\]$") {
                $dependencies.Add(@{ Type = $Matches.type; Name = $Matches.name; Kind = 'dependsOn' })
            } else {
                Write-Trace -Level warn -Message "Skipped the dependency '$dependsOn' of '$($entry.Node.name)': only a literal resourceId('<type>', '<name>') is supported."
            }
        }

        # The strings of nested resources belong to those resources, not to the group.
        $skip = if ($entry.Nested) { 'resources' } else { @() }
        foreach ($string in Get-StringValue -Value $resource['properties'] -SkipKey $skip) {
            foreach ($match in [regex]::Matches($string, "reference\(\s*$resourceIdPattern\s*\)")) {
                $dependencies.Add(@{ Type = $match.Groups['type'].Value; Name = $match.Groups['name'].Value; Kind = 'reference' })
            }
        }

        foreach ($dependency in $dependencies) {
            # A quote in an expression string is written as two quotes.
            $type = $dependency.Type -replace "''", "'"
            $name = $dependency.Name -replace "''", "'"
            $key = Get-ScopeKey -Type $type -Name $name
            if (-not $scope.ContainsKey($key)) {
                Write-Trace -Level warn -Message "'$($entry.Node.name)' depends on '$name' of type '$type', which isn't in the document."
                $scope[$key] = Add-Node -Graph $Graph -Name $name -Type $type -Group $Group -Missing
            }
            Add-Edge -Graph $Graph -From $scope[$key].id -To $entry.Node.id -Kind $dependency.Kind
        }

        if ($entry.Nested) {
            Add-ResourceList -Graph $Graph -Resources $entry.Nested -Group $entry.Node.id
        }
    }
}

function ConvertTo-MermaidText {
    <#
        .SYNOPSIS
        Escapes text for use inside a quoted Mermaid label.
    #>
    param([string]$Text)

    $Text.Replace('#', '#35;').Replace('"', '#quot;').Replace('<', '#lt;').Replace('>', '#gt;')
}

function Write-MermaidNode {
    <#
        .SYNOPSIS
        Writes the Mermaid lines for the nodes of one group, and for the groups inside it.
    #>
    param($Graph, [string]$Group, [int]$Depth)

    $indent = '    ' * $Depth
    foreach ($node in $Graph.nodes | Where-Object { "$($_['group'])" -eq $Group }) {
        $name = ConvertTo-MermaidText -Text $node.name
        $type = ConvertTo-MermaidText -Text $node.type
        if ($node['isGroup']) {
            '{0}subgraph {1}["{2} ({3})"]' -f $indent, $node.id, $name, $type
            Write-MermaidNode -Graph $Graph -Group $node.id -Depth ($Depth + 1)
            '{0}end' -f $indent
        } elseif ($node['missing']) {
            '{0}{1}["{2}<br/>{3}"]:::missing' -f $indent, $node.id, $name, $type
        } else {
            '{0}{1}["{2}<br/>{3}"]' -f $indent, $node.id, $name, $type
        }
    }
}

function ConvertTo-Mermaid {
    <#
        .SYNOPSIS
        Returns the graph as the lines of a Mermaid flowchart.
    #>
    param($Graph, [string]$Direction)

    "flowchart $Direction"
    Write-MermaidNode -Graph $Graph -Group '' -Depth 1
    foreach ($edge in $Graph.edges) {
        $arrow = if ($edge.kind -eq 'dependsOn') { '-->' } else { '-.->' }
        '    {0} {1} {2}' -f $edge.from, $arrow, $edge.to
    }
    if ($Graph.nodes | Where-Object { $_['missing'] }) {
        '    classDef missing stroke-dasharray: 5 5'
    }
}

$options = Read-Arguments -Arguments $args
if ($options.Help) {
    Show-Help
    exit $exitSuccess
}

if ($null -ne $options.Input) {
    $text = $options.Input
} elseif ($null -ne $options.File -and $options.File -ne '-') {
    if (-not (Test-Path -LiteralPath $options.File -PathType Leaf)) {
        Write-Trace -Level error -Message "The file '$($options.File)' doesn't exist."
        exit $exitInvalidDocument
    }
    $text = Get-Content -LiteralPath $options.File -Raw
} elseif ([Console]::IsInputRedirected) {
    $text = @($input) -join "`n"
} else {
    Write-Trace -Level error -Message "No document. Use --file, --input, or pipe a document to the command."
    exit $exitInvalidArgs
}

if ([string]::IsNullOrWhiteSpace($text)) {
    Write-Trace -Level error -Message "The document is empty."
    exit $exitInvalidDocument
}

$document = ConvertTo-Document -Text $text
$graph = @{
    nodes = [System.Collections.Generic.List[object]]::new()
    edges = [System.Collections.Generic.List[object]]::new()
}

if ($document['resources'] -is [System.Collections.IEnumerable]) {
    Add-ResourceList -Graph $graph -Resources $document['resources'] -Group ''
} elseif ($document['results'] -is [System.Collections.IEnumerable]) {
    # The result of an operation names the resources, but not what they depend on.
    Write-Trace -Level info -Message "The input is the result of an operation, which doesn't list dependencies. Showing the resources only."
    foreach ($result in $document['results']) {
        if ($result -is [System.Collections.IDictionary]) {
            $null = Add-Node -Graph $graph -Name "$($result['name'])" -Type "$($result['type'])"
        }
    }
} else {
    Write-Trace -Level error -Message "The document has no 'resources' or 'results' property."
    exit $exitInvalidDocument
}

Write-Trace -Level debug -Message "Found $($graph.nodes.Count) resources and $($graph.edges.Count) dependencies."

if ($options.Format -eq 'json') {
    [ordered]@{ nodes = $graph.nodes; edges = $graph.edges } | ConvertTo-Json -Depth 5 -Compress
} else {
    ConvertTo-Mermaid -Graph $graph -Direction $options.Direction
}
exit $exitSuccess
