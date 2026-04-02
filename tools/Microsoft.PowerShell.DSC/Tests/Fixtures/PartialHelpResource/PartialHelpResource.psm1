# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

<#
    .SYNOPSIS
        Manages a partially documented resource.

    .PARAMETER Name
        The unique name for the resource.
#>
[DscResource()]
class PartialHelpResource {
    [DscProperty(Key)]
    [string] $Name

    [DscProperty()]
    [string] $Value

    [DscProperty()]
    [int] $Count

    [PartialHelpResource] Get() {
        return $this
    }

    [bool] Test() {
        return $true
    }

    [void] Set() {
    }
}
