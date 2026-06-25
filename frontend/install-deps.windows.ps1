# Install specific npm packages by reading exact versions from package.json.
# Workaround for an npm bug in this environment that silently skips installing
# the project's own devDependencies even though it reports "up to date".
$ProgressPreference = 'SilentlyContinue'
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$pkg = Get-Content "$root\frontend\package.json" -Raw | ConvertFrom-Json

# Map of packages we want installed with their pinned version strings.
$targets = @{
    'vite'                  = $pkg.devDependencies.vite
    '@vitejs/plugin-react'  = $pkg.devDependencies.'@vitejs/plugin-react'
    'typescript'            = $pkg.devDependencies.typescript
}

function Resolve-ExactVersion($name, $range) {
    # range may be ^x.y.z or ~x.y.z - strip leading caret/tilde
    $clean = $range -replace '^[~^]', ''
    return $clean
}

function Install-Pkg($name, $version) {
    $dest = "node_modules\$($name.Replace('/','\'))"
    if (Test-Path "$dest\package.json") {
        $installed = (Get-Content "$dest\package.json" -Raw | ConvertFrom-Json).version
        if ($installed -eq $version) { Write-Host "  skip $name@$version (already installed)"; return }
    }
    Write-Host "Installing $name@$version"
    $leaf = $name.Split('/')[-1]
    $url = "https://registry.npmjs.org/$name/-/$leaf-$version.tgz"
    Invoke-WebRequest $url -OutFile tmp.tgz
    tar -xzf tmp.tgz -C node_modules
    if ($name.Contains('/')) {
        $scopeDir = "node_modules\$($name.Split('/')[0])"
        if (-not (Test-Path $scopeDir)) { New-Item -ItemType Directory $scopeDir -Force | Out-Null }
    }
    if (Test-Path $dest) { Remove-Item $dest -Recurse -Force }
    Move-Item node_modules\package $dest -Force
    Remove-Item tmp.tgz
    Write-Host "  OK: $dest"
}

Push-Location "$root\frontend"
foreach ($name in $targets.Keys) {
    $ver = Resolve-ExactVersion $name $targets[$name]
    Install-Pkg $name $ver
}
Pop-Location
Write-Host "Done."
