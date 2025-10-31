# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

using module ./psadapter_helpers.psm1

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0, HelpMessage = 'Operation to perform. Choose from List, Get, Set, Test, Export, Validate, ClearCache.')]
    [ValidateSet('List', 'Get', 'Set', 'Test', 'Export', 'Validate', 'ClearCache')]
    [string]$Operation,
    [Parameter(Mandatory = $false, ValueFromPipeline = $true, HelpMessage = 'Configuration or resource input in JSON format.')]
    [string]$jsonInput = '@{}',
    [Parameter(Mandatory = $true)]
    [string]$ResourceType
)

switch ($Operation) {
    'List' {
        # TODO: Implement List operation
    },
    { @('Get','Set','Test','Export') -contains $_ } {

    }

}
