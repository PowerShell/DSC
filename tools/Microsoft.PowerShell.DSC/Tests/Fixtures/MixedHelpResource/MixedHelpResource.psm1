# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

<#
    .SYNOPSIS
        A fully documented DSC resource.

    .DESCRIPTION
        The DocumentedResource DSC resource has complete comment-based help
        covering all parameters.

    .PARAMETER Name
        The unique identifier for the resource.

    .PARAMETER Setting
        The configuration setting to apply.
#>
[DscResource()]
class DocumentedResource {
    [DscProperty(Key)]
    [string] $Name

    [DscProperty()]
    [string] $Setting

    [DocumentedResource] Get() {
        return $this
    }

    [bool] Test() {
        return $true
    }

    [void] Set() {
    }
}

[DscResource()]
class UndocumentedResource {
    [DscProperty(Key)]
    [string] $Id

    [DscProperty()]
    [string] $Data

    [UndocumentedResource] Get() {
        return $this
    }

    [bool] Test() {
        return $true
    }

    [void] Set() {
    }
}
