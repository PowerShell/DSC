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
class PSTestClassResource : BaseTestClass
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
    }

    [bool] Test()
    {
        if (($this.Name -eq "PSTestClassResource1") -and ($this.Prop1 -eq "ValueForProp1"))
        {
            return $true
        }
        else
        {
            return $false
        }
    }

    [PSTestClassResource] Get()
    {
        if ($this.Name -eq "PSTestClassResource1")
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

    static [PSTestClassResource[]] Export()
    {
        $resultList = [List[PSTestClassResource]]::new()
        $resultCount = 5
        if ($env:PSTestClassResourceResultCount) {
            $resultCount = $env:PSTestClassResourceResultCount
        }
        1..$resultCount | %{
            $obj = New-Object PSTestClassResource
            $obj.Name = "Object$_"
            $obj.Prop1 = "Property of object$_"
            $resultList.Add($obj)
        }

        return $resultList.ToArray()
    }

    static [PSTestClassResource[]] Export([bool]$UseExport)
    {
        if ($UseExport)
        {
            return [PSTestClassResource]::Export()
        }
        else
        {
            $resultList = [List[PSTestClassResource]]::new()
            $resultCount = 5
            if ($env:PSTestClassResourceResultCount) {
                $resultCount = $env:PSTestClassResourceResultCount
            }
            1..$resultCount | %{
                $obj = New-Object PSTestClassResource
                $obj.Name = "Object$_"
                $obj.Prop1 = "Property of object$_"
                $resultList.Add($obj)
            }
        }

        return $resultList.ToArray()
    }
}

[DscResource()]
class PSNoExport: BaseTestClass
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

    [PSNoExport] Get()
    {
        return $this
    }
}

function Test-World()
{
    "Hello world from PSTestModule!"
}
