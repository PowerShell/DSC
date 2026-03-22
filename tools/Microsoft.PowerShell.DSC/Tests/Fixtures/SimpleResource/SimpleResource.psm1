# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[DscResource()]
class SimpleResource {
    [DscProperty(Key)]
    [string] $Name

    [DscProperty(Mandatory)]
    [string] $Value

    [DscProperty()]
    [bool] $Enabled

    [SimpleResource] Get() {
        return $this
    }

    [bool] Test() {
        return $true
    }

    [void] Set() {
    }
}
