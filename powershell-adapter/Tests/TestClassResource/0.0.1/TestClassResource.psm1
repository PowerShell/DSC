# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

using namespace System.Collections.Generic

enum EnumPropEnumeration {
    Unexpected
    Expected
}

class BaseTestClass
{
    [DscProperty()]
    [string] $BaseProperty
}

[DscResource()]
class TestClassResource : BaseTestClass
{
    [DscProperty(Key)]
    [string] $Name

    [DscProperty()]
    [string] $Prop1

    [DscProperty()]
    [string] $EnumProp

    [void] Set()
    {
    }

    [bool] Test()
    {
        if (($this.Name -eq "TestClassResource1") -and ($this.Prop1 -eq "ValueForProp1"))
        {
            return $true
        }
        else
        {
            return $false
        }
    }

    [TestClassResource] Get()
    {
        if ($this.Name -eq "TestClassResource1")
        {
            $this.Prop1 = "ValueForProp1"
        }
        else
        {
            $this.Prop1 = $env:DSC_CONFIG_ROOT
        }
        $this.EnumProp = ([EnumPropEnumeration]::Expected).ToString()
        return $this
    }

    static [TestClassResource[]] Export()
    {
        $resultList = [List[TestClassResource]]::new()
        1..5 | %{
            $obj = New-Object TestClassResource
            $obj.Name = "Object$_"
            $obj.Prop1 = "Property of object$_"
            $resultList.Add($obj)
        }

        return $resultList.ToArray()
    }
}

function Test-World()
{
    "Hello world from PSTestModule!"
}
