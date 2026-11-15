param(
    [string]$Output = (Join-Path $PSScriptRoot "target/Saksham-Android.apk"),
    [switch]$UseExistingBuild
)

$ErrorActionPreference = "Stop"
Push-Location $PSScriptRoot
try {
    if (!$UseExistingBuild) {
        dx build --platform android --target aarch64-linux-android --no-default-features --features mobile --locked
        if ($LASTEXITCODE -ne 0) { throw "Dioxus Android build failed." }
    }

    $project = Join-Path $PSScriptRoot "target/dx/saksham/debug/android/app"
    $main = Join-Path $project "app/src/main"
    $activity = Join-Path $main "kotlin/dev/dioxus/main/MainActivity.kt"
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot "android/MainActivity.kt") -Destination $activity -Force
    $manifest = Join-Path $main "AndroidManifest.xml"
    [xml]$xml = Get-Content -LiteralPath $manifest -Raw
    [void]$xml.manifest.application.SetAttribute("icon", "http://schemas.android.com/apk/res/android", "@mipmap/saksham_launcher")
    $xml.Save($manifest)

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
        # Dioxus has already packaged the default icon. Clean that output so
        # Gradle writes a fresh APK when it adds the Saksham resources.
        & ./gradlew.bat --offline :app:clean :app:assembleDebug
        if ($LASTEXITCODE -ne 0) { throw "Gradle could not package the launcher artwork." }
    } finally {
        Pop-Location
    }
    Copy-Item -LiteralPath (Join-Path $project "app/build/outputs/apk/debug/app-debug.apk") -Destination $Output
    Write-Output "Signed APK: $Output"
} finally {
    Pop-Location
}
