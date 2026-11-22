package dev.dioxus.main

import android.webkit.WebView
import android.os.Build

typealias BuildConfig = org.saksham.learn.BuildConfig

class MainActivity : WryActivity() {
    private var sakshamWebView: WebView? = null
    private var resumed = false
    private var lifecycleGeneration = 0

    override fun onWebViewCreate(webView: WebView) {
        super.onWebViewCreate(webView)
        sakshamWebView = webView
        webView.settings.setGeolocationEnabled(false)
        webView.settings.javaScriptCanOpenWindowsAutomatically = false
        webView.settings.mediaPlaybackRequiresUserGesture = true
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            webView.settings.safeBrowsingEnabled = false
        }
        if (!resumed) {
            webView.onPause()
            webView.pauseTimers()
        }
    }

    override fun onPause() {
        resumed = false
        val generation = ++lifecycleGeneration
        // Dispatch first so active strokes enter the existing local save channel.
        // onPause alone does not suspend JavaScript timers.
        sakshamWebView?.let { webView ->
            webView.evaluateJavascript(
                "window.dispatchEvent(new Event('saksham:pause'));"
            ) {
                if (!resumed && generation == lifecycleGeneration) webView.pauseTimers()
            }
            webView.onPause()
        }
        super.onPause()
    }

    override fun onResume() {
        super.onResume()
        resumed = true
        lifecycleGeneration++
        sakshamWebView?.let { webView ->
            webView.resumeTimers()
            webView.onResume()
            webView.evaluateJavascript(
                "window.dispatchEvent(new Event('saksham:resume'));" +
                    "window.dispatchEvent(new Event('saksham:android-focus'));", null
            )
        }
    }

    override fun onDestroy() {
        lifecycleGeneration++
        sakshamWebView = null
        super.onDestroy()
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        if (hasFocus) {
            sakshamWebView?.evaluateJavascript(
                "window.dispatchEvent(new Event('saksham:android-focus'));",
                null
            )
        }
    }
}

