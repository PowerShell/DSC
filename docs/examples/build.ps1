#!/usr/bin/env pwsh

[CmdletBinding()]
param (
  [Parameter()]
  [ValidateSet('build', 'package', 'test')]
  [string]$Target = 'build',
  [switch]$Initialize
)

begin {
    $AppBuilder = Resolve-Path "$PSScriptRoot/app/build.ps1"
    $GoBuilder  = Resolve-Path "$PSScriptRoot/gotstoy/build.ps1"
}

process {
    Push-Location "$PSScriptRoot/app"
    . $AppBuilder -Target $Target  -AddToPath @PSBoundParameters
    Pop-Location
    Push-Location "$PSScriptRoot/gotstoy"
    . $GoBuilder  -Target $Target  -AddToPath @PSBoundParameters
    Pop-Location
}