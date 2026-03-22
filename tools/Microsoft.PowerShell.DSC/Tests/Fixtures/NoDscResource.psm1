# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

# A helper module with no DSC resources
function Get-SomeValue {
    return 'hello'
}

class NotADscResource {
    [string] $Name

    [string] GetName() {
        return $this.Name
    }
}
