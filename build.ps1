param(
    [switch]$Release,
    [ValidateSet('none','aarch64-pc-windows-msvc',' x86_64-pc-windows-msvc')]
    $architecture = 'none',
    [switch]$Clippy
)

## Create the output folder
$configuration = $Release ? 'release' : 'debug'
$target = Join-Path $PSScriptRoot 'bin' $configuration
if (Test-Path $target) {
    Remove-Item $target -Recurse -ErrorAction Stop
}
New-Item -ItemType Directory $target > $null

$flags = @($Release ? '-r' : $null)
if ($architecture -ne 'none') {
    $flags += '--target'
    $flags += $architecture
    $path = ".\target\$architecture\$configuration"
}
else {
    $path = ".\target\$configuration"
}

$projects = "config", "registry"

foreach ($project in $projects) {
    ## Build format_json
    Write-Host -ForegroundColor Cyan "Building $project ..."
    try {
        Push-Location "$PSScriptRoot/$project"
        if ($Clippy) {
            cargo clippy @flags
        }
        else {
            cargo build @flags
        }

        Copy-Item "$path/$project.exe" $target
        Copy-Item *.command.json $target
    } finally {
        Pop-Location
    }
}

$relative = Resolve-Path $target -Relative
Write-Host -ForegroundColor Green "`nEXE's are copied to $target ($relative)"

$paths = $env:PATH.Split([System.IO.Path]::PathSeparator)
$found = $false
foreach ($path in $paths) {
    if ($path -eq $target) {
        $found = $true
        break
    }
}

# remove the other target in case switching between them
if ($Release) {
    $oldTarget = $target.Replace('\release', '\debug')
}
else {
    $oldTarget = $target.Replace('\debug', '\release')
}
$env:PATH = $env:PATH.Replace(';' + $oldTarget, '')

if (!$found) {
    Write-Host -ForegroundCOlor Yellow "Adding $target to `$env:PATH"
    $env:PATH += [System.IO.Path]::PathSeparator + $target
}

$env:RUST_BACKTRACE=1
