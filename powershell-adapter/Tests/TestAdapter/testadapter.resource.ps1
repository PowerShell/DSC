# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0, HelpMessage = 'Operation to perform. Choose from List, Get, Set, Test, Export, Validate.')]
    [ValidateSet('List', 'Get', 'Set', 'Test', 'Export', 'Validate')]
    [string]$Operation,
    [Parameter(Mandatory = $false, Position = 1, ValueFromPipeline = $true, HelpMessage = 'Configuration or resource input in JSON format.')]
    [string]$jsonInput = '@{}'
)

function Write-DscTrace {
    param(
        [Parameter(Mandatory = $false)]
        [ValidateSet('Error', 'Warn', 'Info', 'Debug', 'Trace')]
        [string]$Operation = 'Debug',
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [string]$Message
    )

    $trace = @{$Operation = $Message } | ConvertTo-Json -Compress
    $host.ui.WriteErrorLine($trace)
}

'Hello from TestAdapter.' | Write-DscTrace
'PSPath=' + $PSHome | Write-DscTrace
'PSModulePath=' + $env:PSModulePath | Write-DscTrace

if ($jsonInput -ne '@{}') {
    $inputobj = $jsonInput | ConvertFrom-Json
}

$jsonInput | Write-DscTrace

switch ($Operation) {
    'List' {
            @{
                type           = "Test/TestCase"
                kind           = 'resource'
                version        = '1'
                capabilities   = @('get', 'set', 'test', 'export')
                path           = $PSScriptRoot
                directory      = Split-Path $PSScriptRoot
                implementedAs  = 'adapter'
                author         = 'Test'
                properties     = @('TestCaseId', 'Input', 'Result')
                requireAdapter = 'Test/TestAdapter'
                description    = 'TestCase resource'
            } | ConvertTo-Json -Compress
    }
    { @('Get','Set','Test','Export') -contains $_ } {

        # TestCase 1 = 'Verify adapted_dsc_type field'
        if (($inputobj.TestCaseId -eq 1 ) -or ($_ -eq 'Export')){
            $result = $inputobj.adapted_dsc_type -eq 'Test/TestCase'
            $result = @{'TestCaseId'=1; 'Input'=''; result = $result } | ConvertTo-Json -Depth 10 -Compress
            return $result
        }

    }
    'Validate' {
        @{ valid = $true } | ConvertTo-Json
    }
}
