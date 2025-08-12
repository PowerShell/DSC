# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
<#
    .SYNOPSIS
    Repro resource for trace messaging

    .PARAMETER Operation
    Operation to perform. Valid operations are:

    - `Get` - return the current state of the instance.

    .PARAMETER JsonInput
    Configuration or resource input in JSON format.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateSet('Get')]
    [string]$Operation,
    [Parameter(Mandatory = $false, Position = 1, ValueFromPipeline = $true)]
    [string]$jsonInput = '@{}'
)

begin {
    enum TestCase {
        SimpleMessage
        MinimalStruct
        StructWithMetadata
        StructWithAdditionalFields
        StructWithMetadataAndAdditionalFields
    }

    $script:ResourcePath = $MyInvocation.MyCommand.Path

    class TraceTesting {
        [TestCase]  $Case
        [ordered] $EmittedData

        [ordered] TraceData() {
            if ($null -ne $this.EmittedData) {
                return $this.EmittedData
            }
    
            if ($this.Case -eq [TestCase]::SimpleMessage) {
                $this.EmittedData = [ordered]@{
                    warn = "Simple message"
                }
                return $this.EmittedData
            }

            $date = Get-Date
            $date = $date.AddYears(-1)
            $this.EmittedData = [ordered]@{
                timestamp = $date
                level     = 'WARN'
                fields    = [ordered]@{
                    message = "structured trace message"
                }
            }

            if ($this.Case -in @([TestCase]::StructWithMetadata, [TestCase]::StructWithMetadataAndAdditionalFields)) {
                $this.EmittedData.target = $script:ResourcePath
                $this.EmittedData.lineNumber = 94
            }
            if ($this.Case -in @([TestCase]::StructWithAdditionalFields, [TestCase]::StructWithMetadataAndAdditionalFields)) {
                $this.EmittedData.fields.extraInteger = 10
                $this.EmittedData.fields.extraString  = "additional data"
            }

            return $this.EmittedData
        }

        [string] TraceJson() {
            return $this.TraceData() | ConvertTo-Json -Compress
        }
        [string] ToJson() {
            $jsonCase = $this.Case.ToString()
            $jsonCase = $jsonCase.Substring(0, 1).ToLowerInvariant() + $jsonCase.Substring(1)
            return [ordered] @{
                case = $jsonCase
                emittedData = $this.TraceData()
            } | ConvertTo-Json -Compress -Depth 99
        }
        static [TraceTesting] ParseJson([string]$inputJson) {
            $parsed = $inputJson | ConvertFrom-Json
            
            return [TraceTesting]@{
                Case = $parsed.case
            }
        }
    }
}

process {
    [TraceTesting]$parsedInput = [TraceTesting]::ParseJson($jsonInput)
    $host.ui.WriteErrorLine($parsedInput.TraceJson())

    switch ($Operation) {
        'Get' {
            $parsedInput.ToJson()
            exit 0
        }
    }
}
