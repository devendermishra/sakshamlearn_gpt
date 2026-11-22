param([string]$CargoHome = (Join-Path $env:USERPROFILE '.cargo'))
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot
$version = [regex]::Match((Get-Content (Join-Path $root 'Cargo.lock') -Raw), 'name = "wry"\r?\nversion = "([^"]+)"').Groups[1].Value
$template = Get-ChildItem (Join-Path $CargoHome 'registry/src') -Directory |
    ForEach-Object { Join-Path $_.FullName "wry-$version/src/android/kotlin" } |
    Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
if (!$template) { throw 'Run cargo fetch --locked to make the pinned Wry templates available.' }
$main = Join-Path $root ('target/patch-android-test-' + [guid]::NewGuid().ToString('N'))
$kotlin = Join-Path $main 'kotlin/dev/dioxus/main'
[void][IO.Directory]::CreateDirectory($kotlin)
$names = @('RustWebViewClient.kt', 'RustWebView.kt', 'Ipc.kt')
foreach ($name in $names) { Copy-Item -LiteralPath (Join-Path $template $name) -Destination $kotlin }
$patch = Join-Path $root 'android/patch-generated.ps1'
& $patch -Main $main
$once = @($names | ForEach-Object { (Get-FileHash (Join-Path $kotlin $_)).Hash }) -join ','
& $patch -Main $main
$twice = @($names | ForEach-Object { (Get-FileHash (Join-Path $kotlin $_)).Hash }) -join ','
if ($once -ne $twice) { throw 'Applying the offline patch twice changed the output.' }
# A new/unrecognized upstream request handler must stop the build.
$client = Join-Path $kotlin 'RustWebViewClient.kt'
$changed = (Get-Content (Join-Path $template 'RustWebViewClient.kt') -Raw).Replace('pendingUrlRedirect?.let {', 'newRedirectHandler?.let {')
Set-Content -LiteralPath $client -Value $changed
$rejected = $false
try { & $patch -Main $main } catch { $rejected = $_.ToString().Contains('Wry template changed') }
if (!$rejected) { throw 'Changed Wry template was accepted without review.' }
Write-Output 'Pinned Wry templates patch successfully; repeat application is stable; template drift fails closed.'
