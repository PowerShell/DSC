# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[CmdletBinding()]
param(
    [Parameter()]
    [switch]$Second,
    [Parameter()]
    [string]$Name,
    [Parameter()]
    [string]$Vault
)

$secretsOne = @{
    Vault1 = @{
        MySecret = 'Hello'
        DuplicateSecret = 'World'
        DuplicateSame = 'SameSecret'
    }
    Vault2 = @{
        AnotherSecret = 'Foo'
    }
}

$secretTwo = @{
    VaultA = @{
        DifferentSecret = 'Hello2'
        DuplicateSecret = 'World2'
        DuplicateSame = 'SameSecret'
    }
}

function get-secret($hashtable, $name, $vault) {
    if ($vault) {
        return $hashtable[$vault][$name]
    } elseif ($name) {
        foreach ($vault in $hashtable.Keys) {
            if ($hashtable[$vault].ContainsKey($name)) {
                return $hashtable[$vault][$name]
            }
        }
    }
    return $null
}

if ($Second) {
    get-secret -hashtable $secretTwo -name $Name -vault $Vault
} else {
    get-secret -hashtable $secretsOne -name $Name -vault $Vault
}
