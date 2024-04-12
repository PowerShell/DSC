# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[CmdletBinding()]
param(
    [ValidateSet('Get', 'Validate')]
    $Operation,
    [Parameter(ValueFromPipeline)]
    $jsonInput
)

$ProgressPreference = 'Ignore'
$WarningPreference = 'Ignore'
$VerbosePreference = 'Ignore'

$objInput = $jsonInput | ConvertFrom-Json

switch ($Operation) {
    'Get' {
        $json = Get-Content -Path $objInput.JSON | ConvertFrom-Json
        $script = Get-Content -Path $objInput.Script -Raw
        $scriptblock = [scriptblock]::Create($script)
        $result = $scriptblock.Invoke() | ConvertFrom-Json
        
        $return = [System.Collections.Generic.List[Object]]::new()
        $result.psobject.properties.Name | ForEach-Object -Process {
            $settingName = $_
            $settingValue = $result.$settingName
            $rule = $json.rules | Where-Object settingname -EQ $settingName
            if ($rule) {
                $output = [rule]::new()
                # returning object should include JSON details so it is easy to understand what is being checked
                $rule.psobject.properties.name | ForEach-Object -Process { $output.$_ = $rule.$_ }
                # prove compliance
                $output.compliance = 'not_compliant'
                switch ($rule.Operator) {
                    'IsEquals'      { if ($settingValue -eq $rule.Operand) { $output.compliance = 'compliant' } }
                    'NotEquals'     { if ($settingValue -ne $rule.Operand) { $output.compliance = 'compliant' } }
                    'GreaterThan'   { if ($settingValue -gt $rule.Operand) { $output.compliance = 'compliant' } }
                    'GreaterEquals' { if ($settingValue -ge $rule.Operand) { $output.compliance = 'compliant' } }
                    'LessThan'      { if ($settingValue -lt $rule.Operand) { $output.compliance = 'compliant' } }
                    'LessEquals'    { if ($settingValue -le $rule.Operand) { $output.compliance = 'compliant' } }
                    'default'       { 'invalid operator' }
                }
                $output.remediationStrings = $rule.RemediationStrings | Where-Object Language -EQ (Get-WinSystemLocale).Name.replace('-','_') | ForEach-Object 'Description'
                $return += $output
            }
        }
        return $return | ConvertTo-Json -EnumsAsStrings
    }
    'Validate' {
        # TODO: this is placeholder
        @{ valid = $true } | ConvertTo-Json
    }
    'default' {
        'ERROR: Unsupported operation requested from powershell.resource.ps1'
    }
}

enum compliance {
    compliant
    not_compliant
}

class rule {
    [string] $settingname
    [string] $operator
    [string] $dataType
    [string] $operand
    [string] $moreInfoUrl
    [string] $remediationStrings
    [compliance] $compliance
}