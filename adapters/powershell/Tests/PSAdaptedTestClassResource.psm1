# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[DscResource()]
class PSAdaptedTestClass
{
    [DscProperty(Key)]
    [string] $Name

    [DscProperty()]
    [int] $Value

    [void] Set()
    {
    }

    [bool] Test()
    {
        return $true
    }

    [PSAdaptedTestClass] Get()
    {
        $this.Value = 42
        return $this
    }
}
