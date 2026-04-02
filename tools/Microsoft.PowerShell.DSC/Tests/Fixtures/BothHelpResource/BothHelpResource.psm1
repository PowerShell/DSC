# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

<#
    .SYNOPSIS
        Manages the first resource.

    .PARAMETER Name
        The unique name of the first resource.

    .PARAMETER Mode
        The operating mode for the first resource.
#>
[DscResource()]
class FirstResource {
    [DscProperty(Key)]
    [string] $Name

    [DscProperty()]
    [string] $Mode

    [FirstResource] Get() {
        return $this
    }

    [bool] Test() {
        return $true
    }

    [void] Set() {
    }
}

<#
    .SYNOPSIS
        Manages the second resource.

    .PARAMETER Id
        The identifier for the second resource.

    .PARAMETER Label
        A label for the second resource.
#>
[DscResource()]
class SecondResource {
    [DscProperty(Key)]
    [string] $Id

    [DscProperty()]
    [string] $Label

    [SecondResource] Get() {
        return $this
    }

    [bool] Test() {
        return $true
    }

    [void] Set() {
    }
}
