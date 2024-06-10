# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

using namespace System.Collections.Generic

[DscResource()]
class Service
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

    [Service] Get()
    {
       
        return [Service]::GetServices() | ? {$_.Unit -eq $this.Unit }
    }

    static [Service[]] Export()
    {
        $resultList = [List[Service]]::new()
        $svcs = [Service]::GetServices()
        $svcs | %{
            $obj = New-Object Service
            $obj.Unit = $_.Unit
            $obj.Load = $_.Load
            $obj.Active = $_.Active
            $obj.Sub = $_.Sub
            $obj.Description = $_.Description
            
            $resultList.Add($obj)
        }

        return $resultList.ToArray()
    }

    static [pscustomobject[]] GetServices()
    {
        $out = systemctl -l --type service --all
        $resultList = [List[pscustomobject]]::new()
        $out | select-object -skip 1 | select-object -skiplast 7 | %{
            $arr = $_.split(" ", [System.StringSplitOptions]::RemoveEmptyEntries)

            $a = [pscustomobject]@{
                Unit = $arr[0]
                Load = $arr[1]
                Active = $arr[2]
                Sub = $arr[3]
                Description = ($arr | select-object -last ($arr.Count - 4)) -join " "
            }

            if ($a.active -ne 'not-found') { $resultList.Add($a)}
        }
        return $resultList.ToArray()
    }
}