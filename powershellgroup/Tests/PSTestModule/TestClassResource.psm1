using namespace System.Collections.Generic

enum EnumPropEnumeration {
    Unexpected
    Expected
}

[DscResource()]
class TestClassResource
{
    [DscProperty(Key)]
    [string] $Name

    [DscProperty()]
    [string] $Prop1

    [DscProperty()]
    [EnumPropEnumeration] $EnumProp

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
        $this.EnumProp = [EnumPropEnumeration]::Expected
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

function Hello-World()
{
    "Hello world from PSTestModule!"
}
