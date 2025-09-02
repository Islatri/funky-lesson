package com.funkylesson.app

import android.content.ActivityNotFoundException
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.util.Log
import android.view.View
import android.view.WindowManager
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat

class MainActivity : TauriActivity() {
    override fun onResume() {
        super.onResume()
        setupDisplayCutoutAndFullscreen()
        setupJavaScriptInterface()
    }

    private fun setupJavaScriptInterface() {
        val webView = findWebView()
        webView?.let {
            // 启用 JavaScript
            it.settings.javaScriptEnabled = true
            // 注册 JavaScript 接口
            it.addJavascriptInterface(this, "Android")
        }
    }

    private fun findWebView(): WebView? {
        return try {
            val activityClass = this::class.java
            val superClass = activityClass.superclass
            
            val fields = superClass?.declaredFields ?: arrayOf()
            for (field in fields) {
                field.isAccessible = true
                val value = field.get(this)
                if (value is WebView) {
                    return value
                }
            }
            
            val contentView = findViewById<View>(android.R.id.content)
            return findWebViewRecursive(contentView)
        } catch (e: Exception) {
            Log.e("MainActivity", "Error finding WebView", e)
            null
        }
    }
    
    private fun findWebViewRecursive(view: View?): WebView? {
        if (view is WebView) {
            return view
        }
        if (view is android.view.ViewGroup) {
            for (i in 0 until view.childCount) {
                val found = findWebViewRecursive(view.getChildAt(i))
                if (found != null) return found
            }
        }
        return null
    }
    
    @JavascriptInterface
    fun openInExternalBrowser(url: String) {
        try {
            val intent = Intent(Intent.ACTION_VIEW, Uri.parse(url))
            // 确保用外部浏览器打开，而不是 WebView
            intent.addCategory(Intent.CATEGORY_BROWSABLE)
            intent.flags = Intent.FLAG_ACTIVITY_NEW_TASK
            startActivity(intent)
        } catch (e: ActivityNotFoundException) {
            // 可以添加日志或 Toast 提示
            Log.e("WebView", "No browser found to open URL: $url")
        }
    }
    
    private fun setupDisplayCutoutAndFullscreen() {
        try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                WindowCompat.setDecorFitsSystemWindows(window, false)
                val controller = WindowInsetsControllerCompat(window, window.decorView)
                controller.hide(WindowInsetsCompat.Type.systemBars())
                controller.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            } else {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                    window.attributes.layoutInDisplayCutoutMode = 
                        WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES
                }
                
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
                    window.statusBarColor = android.graphics.Color.TRANSPARENT
                    window.navigationBarColor = android.graphics.Color.TRANSPARENT
                }
                
                @Suppress("DEPRECATION")
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.KITKAT) {
                    window.decorView.systemUiVisibility = (
                        View.SYSTEM_UI_FLAG_LAYOUT_STABLE
                        or View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                        or View.SYSTEM_UI_FLAG_FULLSCREEN
                        or View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
                    )
                }
            }
            
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }
}