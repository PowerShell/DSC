# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

<#
    .SYNOPSIS
        Manages a help-documented resource.

    .DESCRIPTION
        The HelpResource DSC resource is used to manage a help-documented resource
        with full comment-based help including synopsis, description, and all
        parameter documentation.

    .PARAMETER Name
        The unique name identifying this resource instance.

    .PARAMETER Value
        The value to assign to this resource.

    .PARAMETER Enabled
        Whether this resource is active.
#>
[DscResource()]
class HelpResource {
    [DscProperty(Key)]
    [string] $Name

    [DscProperty(Mandatory)]
    [string] $Value

    [DscProperty()]
    [bool] $Enabled

    [HelpResource] Get() {
        return $this
    }

    [bool] Test() {
        return $true
    }

    [void] Set() {
    }
}
