# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

enum Ensure {
    Present
    Absent
}

class BaseResource {
    [DscProperty()]
    [string] $BaseProperty
}

[DscResource()]
class ResourceA : BaseResource {
    [DscProperty(Key)]
    [string] $Name

    [DscProperty()]
    [Ensure] $Ensure

    [DscProperty()]
    [int] $Count

    [DscProperty()]
    [string[]] $Tags

    [ResourceA] Get() {
        return $this
    }

    [bool] Test() {
        return $true
    }

    [void] Set() {
    }

    [void] Delete() {
    }

    static [ResourceA[]] Export() {
        return @()
    }
}

[DscResource()]
class ResourceB {
    [DscProperty(Key)]
    [string] $Id

    [DscProperty()]
    [hashtable] $Settings

    [ResourceB] Get() {
        return $this
    }

    [bool] Test() {
        return $false
    }

    [void] Set() {
    }

    [bool] WhatIf() {
        return $true
    }
}
