param(
    [string]$Output = (Join-Path $PSScriptRoot "target/Saksham-Android.apk"),
    [switch]$UseExistingBuild,
    [switch]$DebugBuild,
    [switch]$Offline
)

$ErrorActionPreference = "Stop"
Push-Location $PSScriptRoot
try {
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
    $activity = Join-Path $main "kotlin/dev/dioxus/main/MainActivity.kt"
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot "android/MainActivity.kt") -Destination $activity -Force
    $manifest = Join-Path $main "AndroidManifest.xml"
    [xml]$xml = Get-Content -LiteralPath $manifest -Raw
    [void]$xml.manifest.application.SetAttribute("icon", "http://schemas.android.com/apk/res/android", "@mipmap/saksham_launcher")
    $xml.Save($manifest)

    if (!$DebugBuild) {
        # Preserve the installed tablet's signing identity while using optimized,
        # non-debuggable Rust and Android release builds. Public distribution
        # should configure its own release signing key here.
        $gradle = Join-Path $project "app/build.gradle.kts"
        $configuration = Get-Content -LiteralPath $gradle -Raw
        $releaseMarker = 'getByName("release") {'
        $signingMarker = '// Saksham tablet release signing'
        if (!$configuration.Contains($signingMarker)) {
            if (!$configuration.Contains($releaseMarker)) { throw "Generated Gradle release configuration was not found." }
            $configuration = $configuration.Replace($releaseMarker, "$releaseMarker`n            $signingMarker`n            signingConfig = signingConfigs.getByName(`"debug`")`n            isShrinkResources = true")
            Set-Content -LiteralPath $gradle -Value $configuration -Encoding utf8
        }
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
        # Resource changes invalidate packaging without deleting Gradle outputs.
        $gradleArguments = @(":app:assemble$variant")
        if ($Offline) { $gradleArguments += "--offline" }
        & ./gradlew.bat @gradleArguments
        if ($LASTEXITCODE -ne 0) { throw "Gradle could not package the launcher artwork." }
    } finally {
        Pop-Location
    }
    $outputDirectory = Split-Path -Parent $Output
    if ($outputDirectory) { [void][System.IO.Directory]::CreateDirectory($outputDirectory) }
    Copy-Item -LiteralPath (Join-Path $project "app/build/outputs/apk/$profile/app-$profile.apk") -Destination $Output
    Write-Output "Signed APK: $Output"
} finally {
    Pop-Location
}
