# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

using namespace System.Collections.Generic

enum EnumPropEnumeration {
    Unexpected
    Expected
}

enum Ensure {
    Present
    Absent
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
    [hashtable] $HashTableProp

    [DscProperty()]
    [string] $EnumProp

    [DscProperty()]
    [PSCredential] $Credential

    [DscProperty()]
    [Ensure] $Ensure

    [DscProperty()]
    [SecureString] $SecureStringProp

    [string] $NonDscProperty # This property shouldn't be in results data

    hidden
    [string] $HiddenNonDscProperty # This property shouldn't be in results data

    hidden
    [DscProperty()]
    [string] $HiddenDscProperty # This property should be in results data, but is an anti-pattern.

    [void] Set()
    {
        Write-Host "This is a Host message"
        Write-Information "This is an Information message"
        Write-Error "This is an Error message"
    }

    [bool] Test()
    {
        Write-Warning "This is a Warning message"
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
        Write-Verbose "This is a Verbose message"
        if ($this.Name -eq "TestClassResource1")
        {
            $this.Prop1 = "ValueForProp1"
        }
        elseif ($this.Name -eq 'EchoBack')
        {
            # don't change the property, just echo it back
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
        Write-Debug "This is a Debug message"
        $resultList = [List[TestClassResource]]::new()
        $resultCount = 5
        if ($env:TestClassResourceResultCount) {
            $resultCount = $env:TestClassResourceResultCount
        }
        1..$resultCount | %{
            $obj = New-Object TestClassResource
            $obj.Name = "Object$_"
            $obj.Prop1 = "Property of object$_"
            $resultList.Add($obj)
        }

        return $resultList.ToArray()
    }

    static [TestClassResource[]] Export([bool]$UseExport)
    {
        if ($UseExport)
        {
            return [TestClassResource]::Export()
        }
        else
        {
            $resultList = [List[TestClassResource]]::new()
            $resultCount = 5
            if ($env:TestClassResourceResultCount) {
                $resultCount = $env:TestClassResourceResultCount
            }
            1..$resultCount | %{
                $obj = New-Object TestClassResource
                $obj.Name = "Object$_"
                $obj.Prop1 = "Property of object$_"
                $resultList.Add($obj)
            }
        }

        return $resultList.ToArray()
    }
}

[DscResource()]
class NoExport: BaseTestClass
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
        return $true
    }

    [NoExport] Get()
    {
        return $this
    }
}

[DscResource()]
class FilteredExport : BaseTestClass
{
    [DscProperty(Key)]
    [string] $Name

    [DscProperty()]
    [string] $Prop1

    [void] Set()
    {
    }

    [bool] Test()
    {
        return $true
    }

    [FilteredExport] Get()
    {
        return $this
    }

    static [FilteredExport[]] Export()
    {
        $resultList = [List[FilteredExport]]::new()
        $obj = New-Object FilteredExport
        $obj.Name = "DefaultObject"
        $obj.Prop1 = "Default Property"
        $resultList.Add($obj)

        return $resultList.ToArray()
    }

    static [FilteredExport[]] Export([FilteredExport]$Name)
    {
        $resultList = [List[FilteredExport]]::new()
        $obj = New-Object FilteredExport
        $obj.Name = $Name
        $obj.Prop1 = "Filtered Property for $Name"
        $resultList.Add($obj)

        return $resultList.ToArray()
    }
}

function Test-World()
{
    "Hello world from PSTestModule!"
}
