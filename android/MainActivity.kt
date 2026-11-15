package dev.dioxus.main

import android.webkit.WebView

typealias BuildConfig = org.saksham.learn.BuildConfig

class MainActivity : WryActivity() {
    private var sakshamWebView: WebView? = null

    override fun onWebViewCreate(webView: WebView) {
        super.onWebViewCreate(webView)
        sakshamWebView = webView
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

