param([Parameter(Mandatory)][string]$Main)
$ErrorActionPreference = 'Stop'

function Replace-Checked([string]$Text, [string]$Before, [string]$After) {
    if (($Text.Split(@($Before), [StringSplitOptions]::None).Count - 1) -ne 1) {
        throw "Wry template changed; cannot safely apply offline guard: $Before"
    }
    return $Text.Replace($Before, $After)
}

$kotlin = Join-Path $Main 'kotlin/dev/dioxus/main'
$clientPath = Join-Path $kotlin 'RustWebViewClient.kt'
$client = (Get-Content -LiteralPath $clientPath -Raw).Replace("`r`n", "`n")
if (!$client.Contains('// Saksham offline guard')) {
    $client = Replace-Checked $client "        pendingUrlRedirect?.let {" @'
        // Saksham offline guard: run before JNI or any redirect/network fallback.
        if (!OfflinePolicy.allows(request.url)) return OfflinePolicy.blocked()
        pendingUrlRedirect?.let {
'@
    $client = Replace-Checked $client 'assetLoader.shouldInterceptRequest(request.url)' 'OfflinePolicy.protect(assetLoader.shouldInterceptRequest(request.url))'
    $client = Replace-Checked $client "            return response`n" "            return OfflinePolicy.protect(response)`n"
    $client = Replace-Checked $client '        return shouldOverride(request.url.toString())' @'
        if (!OfflinePolicy.allowsNavigation(request.url.toString())) return true
        return shouldOverride(request.url.toString())
'@
    Set-Content -LiteralPath $clientPath -Value $client -NoNewline
}
# Wry's redirect recovery used to return null, which means "try the network".
# Keep recovery on the local protocol path even if an old error queued it.
$redirectFallback = "            pendingUrlRedirect = null`n            return null"
if ($client.Contains($redirectFallback)) {
    $client = Replace-Checked $client $redirectFallback "            pendingUrlRedirect = null`n            return OfflinePolicy.blocked()"
    Set-Content -LiteralPath $clientPath -Value $client -NoNewline
}
# Wry calls native navigation handling directly from loadUrl, too. Guard both paths.
$viewPath = Join-Path $kotlin 'RustWebView.kt'
$view = (Get-Content -LiteralPath $viewPath -Raw).Replace("`r`n", "`n")
if (!$view.Contains('// Saksham offline guard')) {
    foreach ($signature in @('    override fun loadUrl(url: String) {', '    override fun loadUrl(url: String, additionalHttpHeaders: Map<String, String>) {')) {
        $view = Replace-Checked $view $signature ($signature + "`n        // Saksham offline guard`n        if (!OfflinePolicy.allowsNavigation(url)) return")
    }
    Set-Content -LiteralPath $viewPath -Value $view -NoNewline
}
if (($client | Select-String -AllMatches 'OfflinePolicy\.').Matches.Count -ne 6 -or
    ($view | Select-String -AllMatches 'OfflinePolicy\.').Matches.Count -ne 2) {
    throw 'Offline guard verification failed. Regenerate the Android project before packaging.'
}
$ipcPath = Join-Path $kotlin 'Ipc.kt'
$ipc = (Get-Content -LiteralPath $ipcPath -Raw).Replace("`r`n", "`n")
if (!$ipc.Contains('// Saksham offline guard')) {
    $ipc = Replace-Checked $ipc '        message?.let {m ->' @'
        message?.let {m ->
            // Saksham offline guard: Dioxus can open external links through IPC.
            if (!OfflinePolicy.allowsIpc(m)) return
'@
    Set-Content -LiteralPath $ipcPath -Value $ipc -NoNewline
}
if (($ipc | Select-String -AllMatches 'OfflinePolicy\.allowsIpc').Matches.Count -ne 1) {
    throw 'Offline IPC guard verification failed.'
}
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'OfflinePolicy.kt') -Destination $kotlin -Force
