# Installs the snappea CLI on Windows (x86_64) from a GitHub release.
#
#   irm https://raw.githubusercontent.com/crowecawcaw/snappea/main/scripts/install.ps1 | iex
#
# Environment variables:
#   SNAPPEA_VERSION      release to install (default: the latest)
#   SNAPPEA_INSTALL_DIR  folder for snappea.exe (default: %LOCALAPPDATA%\snappea\bin)
#
# The archive is checked against the release's SHA256SUMS. The folder is added to the
# user PATH, and in GitHub Actions to the PATH of later steps.
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$repo = 'crowecawcaw/snappea'
$target = 'x86_64-pc-windows-msvc'
$version = $env:SNAPPEA_VERSION
$dir = if ($env:SNAPPEA_INSTALL_DIR) { $env:SNAPPEA_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'snappea\bin' }

if ([System.Environment]::Is64BitOperatingSystem -ne $true) { throw 'snappea install: no prebuilt binary for 32-bit Windows' }

if (-not $version) {
  $headers = @{ 'User-Agent' = 'snappea-install' }
  if ($env:GITHUB_TOKEN) { $headers['Authorization'] = "Bearer $env:GITHUB_TOKEN" }
  $latest = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest" -Headers $headers
  $version = $latest.tag_name
}
$version = $version.TrimStart('v')
$tag = "v$version"
$archive = "snappea-$tag-$target.zip"
$base = "https://github.com/$repo/releases/download/$tag"

$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid())
New-Item -ItemType Directory -Path $tmp | Out-Null
try {
  Write-Host "snappea install: downloading snappea $version for $target"
  Invoke-WebRequest "$base/$archive" -OutFile (Join-Path $tmp $archive) -UseBasicParsing
  Invoke-WebRequest "$base/SHA256SUMS" -OutFile (Join-Path $tmp 'SHA256SUMS') -UseBasicParsing

  $line = Get-Content (Join-Path $tmp 'SHA256SUMS') | Where-Object { ($_ -split '\s+')[1] -eq $archive }
  if (-not $line) { throw "snappea install: SHA256SUMS has no entry for $archive" }
  $want = ($line -split '\s+')[0]
  $got = (Get-FileHash (Join-Path $tmp $archive) -Algorithm SHA256).Hash.ToLower()
  if ($got -ne $want) { throw "snappea install: checksum mismatch for $archive" }

  Expand-Archive (Join-Path $tmp $archive) -DestinationPath $tmp
  New-Item -ItemType Directory -Force -Path $dir | Out-Null
  $exe = Join-Path $dir 'snappea.exe'
  Copy-Item (Join-Path $tmp "snappea-$tag-$target\snappea.exe") $exe -Force
  Write-Host "snappea install: installed $(& $exe --version) to $exe"
} finally {
  Remove-Item -Recurse -Force $tmp
}

if ($env:GITHUB_PATH) {
  Add-Content -Path $env:GITHUB_PATH -Value $dir
} else {
  $userPath = [System.Environment]::GetEnvironmentVariable('Path', 'User')
  if (-not (($userPath -split ';') -contains $dir)) {
    [System.Environment]::SetEnvironmentVariable('Path', "$dir;$userPath", 'User')
    Write-Host "snappea install: added $dir to your user PATH; open a new terminal to use it"
  }
}
