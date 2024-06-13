# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

using namespace System.Collections.Generic

[DscResource()]
class SystemctlService
{
    [DscProperty(Key)]
    [string] $Unit

    [DscProperty()]
    [string] $Load

    [DscProperty()]
    [string] $Active

    [DscProperty()]
    [string] $Sub

    [DscProperty()]
    [string] $Description

    [void] Set()
    {
    }

    [bool] Test()
    {
        return $false
    }

    [SystemctlService] Get()
    {
       
        return [SystemctlService]::GetAll() | Where-Object {$_.Unit -eq $this.Unit }
    }

    static [SystemctlService[]] Export()
    {
        return [SystemctlService]::GetAll()
    }

    static [SystemctlService[]] GetAll()
    {
        $out = systemctl -l --type service --all
        $resultList = [List[SystemctlService]]::new()
        $out | select-object -skip 1 | select-object -skiplast 7 | %{
            $arr = $_.split(" ", [System.StringSplitOptions]::RemoveEmptyEntries)
            
            if ($arr[2] -ne 'not-found') {
                $obj = New-Object SystemctlService
                $obj.Unit = $arr[0]
                $obj.Load = $arr[1]
                $obj.Active = $arr[2]
                $obj.Sub = $arr[3]
                $obj.Description = ($arr | select-object -last ($arr.Count - 4)) -join " "

                $resultList.Add($obj)
            }
        }
        return $resultList.ToArray()
    }
}