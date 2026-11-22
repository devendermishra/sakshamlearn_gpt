package dev.dioxus.main

import android.net.Uri
import android.webkit.WebResourceResponse
import java.io.ByteArrayInputStream
import org.json.JSONObject

/** Dioxus's HTTPS origin is intercepted locally, never fetched from DNS/network. */
object OfflinePolicy {
    const val CSP = "default-src 'none'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; " +
        "style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; " +
        "connect-src https://dioxus.index.html ws://127.0.0.1:*; media-src 'self' data: blob:; " +
        "object-src 'none'; base-uri 'none'; form-action 'none'; frame-src 'none'; worker-src 'none'"

    fun allows(uri: Uri): Boolean {
        if (uri.userInfo != null) return false
        return when (uri.scheme) {
            "https" -> uri.host == "dioxus.index.html" && (uri.port == -1 || uri.port == 443)
            "dioxus" -> uri.host == "index.html" && uri.port == -1
            else -> false
        }
    }

    fun allowsNavigation(url: String): Boolean = url == "about:blank" || allows(Uri.parse(url))

    // Anchor clicks can ask Dioxus to open a browser through IPC instead of
    // WebView navigation. Stop that path before it reaches native Rust.
    fun allowsIpc(message: String): Boolean = try {
        JSONObject(message).optString("method") != "browser_open"
    } catch (_: Exception) {
        false
    }

    fun blocked(): WebResourceResponse = WebResourceResponse(
        "text/plain", "UTF-8", 403, "Offline only", mapOf("Cache-Control" to "no-store"),
        ByteArrayInputStream(ByteArray(0))
    )

    fun protect(response: WebResourceResponse?): WebResourceResponse {
        // Never let even a missing local asset fall through to the network.
        if (response == null) return blocked()
        response.responseHeaders = (response.responseHeaders ?: emptyMap()) +
            ("Content-Security-Policy" to CSP)
        return response
    }
}
