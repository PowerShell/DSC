#!/usr/bin/env pwsh

[CmdletBinding()]
param (
  [Parameter()]
  [ValidateSet('build', 'package', 'test')]
  [string]$Target = 'build',
  [switch]$Initialize,
  [switch]$AddToPath
)

function Build-Project {
    [cmdletbinding()]
    [OutputType([System.Management.Automation.ApplicationInfo])]
    param(
        [switch]$All
    )
    if ($All) {
        goreleaser release --skip-publish --skip-announce --skip-validate --clean --release-notes ./RELEASE_NOTES.md
    } else {
        goreleaser build --snapshot --clean --single-target
    }
    Get-Command "./dist/tstoy*/tstoy*" -ErrorAction Stop
}

switch ($Target) {
    'build' {
        $Application = Build-Project
        if ($AddToPath) {
            $ApplicationFolder = Split-Path -Parent $Application.Path
            $PathSeparator = [System.IO.Path]::PathSeparator
            if ($ApplicationFolder -notin ($env:PATH -split $PathSeparator)) {
                $env:PATH = $ApplicationFolder + $PathSeparator + $env:PATH
            }
        }
        if ($Initialize) {
            $Alias = Set-Alias -Name tstoy -Value $Application.Path -PassThru
            Invoke-Expression $(tstoy completion powershell | Out-String)
            $Alias
        } else {
            $Application
        }
    }
    'package' {
        Build-Project -All
    }
    'test' {
        $Application = Build-Project
        $TestContainer = New-PesterContainer -Path 'acceptance.tests.ps1' -Data @{
            Application = $Application
        }
        Invoke-Pester -Container $TestContainer -Output Detailed
    }
}