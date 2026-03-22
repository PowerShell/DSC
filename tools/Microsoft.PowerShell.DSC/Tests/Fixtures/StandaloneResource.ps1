# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[DscResource()]
class StandaloneResource {
    [DscProperty(Key)]
    [string] $Name

    [DscProperty()]
    [string] $Content

    [StandaloneResource] Get() {
        return $this
    }

    [bool] Test() {
        return $true
    }

    [void] Set() {
    }
}
