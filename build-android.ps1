param(
    [string]$Output = (Join-Path $PSScriptRoot "target/Saksham-Android.apk"),
    [switch]$UseExistingBuild,
    [switch]$DebugBuild,
    [switch]$Offline,
    [string]$SigningKey = (Join-Path $env:USERPROFILE '.android/debug.keystore'),
    [string]$PreviousApk = (Join-Path $PSScriptRoot 'target/Saksham-Android.apk')
)

$ErrorActionPreference = "Stop"
$previousSigningKey = $env:SAKSHAM_SIGNING_KEY
Push-Location $PSScriptRoot
try {
    if (!(Test-Path -LiteralPath $SigningKey)) {
        throw "Existing installation signing key is missing: $SigningKey. Restore it before building an update."
    }
    $env:SAKSHAM_SIGNING_KEY = (Resolve-Path -LiteralPath $SigningKey).Path
    $sdk = if ($env:ANDROID_HOME) { $env:ANDROID_HOME } else { Join-Path $env:LOCALAPPDATA 'Android/Sdk' }
    $buildTools = Get-ChildItem (Join-Path $sdk 'build-tools') -Directory | Sort-Object { [version]$_.Name } -Descending | Select-Object -First 1
    $apksigner = Join-Path $buildTools.FullName 'apksigner.bat'
    $aapt = Join-Path $buildTools.FullName 'aapt.exe'
    function Certificate([string]$Apk) {
        $info = & $apksigner verify --print-certs $Apk
        if ($LASTEXITCODE -ne 0) { throw "APK signature verification failed: $Apk" }
        $digest = @($info | Where-Object { $_ -match '^Signer #1 certificate SHA-256 digest:' })
        if ($digest.Count -ne 1) { throw "Cannot determine signing certificate: $Apk" }
        return $digest[0]
    }
    $previousCertificate = if (Test-Path -LiteralPath $PreviousApk) { Certificate $PreviousApk } else { $null }
    $profile = if ($DebugBuild) { "debug" } else { "release" }
    $variant = if ($DebugBuild) { "Debug" } else { "Release" }
    if (!$UseExistingBuild) {
        $arguments = @("build", "--platform", "android", "--target", "aarch64-linux-android", "--no-default-features", "--features", "mobile", "--locked")
        if (!$DebugBuild) { $arguments += "--release" }
        if ($Offline) { $arguments += "--offline" }
        & dx @arguments
        if ($LASTEXITCODE -ne 0) { throw "Dioxus Android build failed." }
    }

    $project = Join-Path $PSScriptRoot "target/dx/saksham/$profile/android/app"
    $main = Join-Path $project "app/src/main"
    if (!(Test-Path -LiteralPath $main)) { throw "An existing $profile build is required with -UseExistingBuild." }
    $activity = Join-Path $main "kotlin/dev/dioxus/main/MainActivity.kt"
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot "android/MainActivity.kt") -Destination $activity -Force
    & (Join-Path $PSScriptRoot 'android/patch-generated.ps1') -Main $main
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'android/network_security_config.xml') -Destination (Join-Path $main 'res/xml/network_security_config.xml') -Force
    $manifest = Join-Path $main "AndroidManifest.xml"
    [xml]$xml = Get-Content -LiteralPath $manifest -Raw
    [void]$xml.manifest.application.SetAttribute("icon", "http://schemas.android.com/apk/res/android", "@mipmap/saksham_launcher")
    # AppCompat's transitive startup components include an emoji font fetcher
    # and deferred profile installation. This offline app uses bundled fonts.
    foreach ($component in @(
        @('provider', 'androidx.startup.InitializationProvider'),
        @('receiver', 'androidx.profileinstaller.ProfileInstallReceiver')
    )) {
        foreach ($existing in @($xml.manifest.application.SelectNodes($component[0]))) {
            if ($existing.GetAttribute('name', 'http://schemas.android.com/apk/res/android') -eq $component[1]) {
                [void]$xml.manifest.application.RemoveChild($existing)
            }
        }
        $node = $xml.CreateElement($component[0])
        [void]$node.SetAttribute('name', 'http://schemas.android.com/apk/res/android', $component[1])
        [void]$node.SetAttribute('node', 'http://schemas.android.com/tools', 'remove')
        [void]$xml.manifest.application.AppendChild($node)
    }
    $xml.Save($manifest)

    # Retain the existing update certificate; release does not imply a new key.
    # Retain resource shrinking and the generated Wry/JNI keep rules.
    $gradlePath = Join-Path $project 'app/build.gradle.kts'
    $gradle = (Get-Content -LiteralPath $gradlePath -Raw).Replace("`r`n", "`n")
    if (!$gradle.Contains('// Saksham release signing')) {
        $anchor = '        getByName("release") {'
        if (($gradle.Split(@($anchor), [StringSplitOptions]::None).Count - 1) -ne 1) { throw 'Gradle release template changed.' }
        $gradle = $gradle.Replace($anchor, $anchor + @'

            // Saksham release signing
            isDebuggable = false
            isJniDebuggable = false
            isShrinkResources = true
            signingConfig = signingConfigs.getByName("debug").apply {
                storeFile = file(System.getenv("SAKSHAM_SIGNING_KEY"))
            }
'@)
        Set-Content -LiteralPath $gradlePath -Value $gradle -NoNewline
    }

    Add-Type -AssemblyName System.Drawing
    $source = [System.Drawing.Image]::FromFile((Join-Path $PSScriptRoot "assets/saksham-icon.png"))
    try {
        $densities = [ordered]@{ mdpi = 48; hdpi = 72; xhdpi = 96; xxhdpi = 144; xxxhdpi = 192 }
        foreach ($density in $densities.Keys) {
            $size = $densities[$density]
            $directory = Join-Path $main "res/mipmap-$density"
            [void][System.IO.Directory]::CreateDirectory($directory)
            $bitmap = [System.Drawing.Bitmap]::new($size, $size)
            $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
            try {
                $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
                $graphics.DrawImage($source, 0, 0, $size, $size)
                $bitmap.Save((Join-Path $directory "saksham_launcher.png"), [System.Drawing.Imaging.ImageFormat]::Png)
            } finally {
                $graphics.Dispose()
                $bitmap.Dispose()
            }
        }
    } finally {
        $source.Dispose()
    }

    Push-Location $project
    try {
        # Resource and source changes invalidate Gradle outputs without cleaning.
        $gradleArguments = @("app:assemble$variant")
        if ($Offline) { $gradleArguments += "--offline" }
        & ./gradlew.bat @gradleArguments
        if ($LASTEXITCODE -ne 0) { throw "Gradle could not package the Android release." }
    } finally {
        Pop-Location
    }
    $apk = Join-Path $project "app/build/outputs/apk/$profile/app-$profile.apk"
    $certificate = Certificate $apk
    if ($previousCertificate -and $certificate -ne $previousCertificate) { throw 'Signing certificate differs from the previous APK; update would fail.' }
    $badging = & $aapt dump badging $apk
    if ($LASTEXITCODE -ne 0 -or (!$DebugBuild -and ($badging -match 'application-debuggable'))) { throw 'Final APK is invalid or debuggable.' }
    $manifestDump = & $aapt dump xmltree $apk AndroidManifest.xml
    if ($LASTEXITCODE -ne 0 -or ($manifestDump -match 'E: (service|receiver|provider)\b') -or
        ($manifestDump -match 'android.permission.(WAKE_LOCK|RECEIVE_BOOT_COMPLETED|FOREGROUND_SERVICE|SCHEDULE_EXACT_ALARM)')) {
        throw 'Unexpected background components or permissions in final APK.'
    }
    $permissions = & $aapt dump permissions $apk
    if ($LASTEXITCODE -ne 0 -or !($permissions -match "uses-permission: name='android.permission.INTERNET'")) { throw 'Required localhost permission is missing.' }
    $unexpected = $permissions | Where-Object { $_ -match '^uses-permission:' -and $_ -notmatch "name='(android.permission.INTERNET|org.saksham.learn.DYNAMIC_RECEIVER_NOT_EXPORTED_PERMISSION)'" }
    if ($unexpected) { throw "Unexpected APK permissions: $unexpected" }
    $outputDirectory = Split-Path -Parent $Output
    if ($outputDirectory) { [void][System.IO.Directory]::CreateDirectory($outputDirectory) }
    Copy-Item -LiteralPath $apk -Destination $Output
    Write-Output "Signed APK: $Output"
} finally {
    $env:SAKSHAM_SIGNING_KEY = $previousSigningKey
    Pop-Location
}
