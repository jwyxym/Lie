package cn.jwyxym.lie

import android.content.pm.ActivityInfo
import android.os.Bundle
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import app.tauri.plugin.Plugin

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_PORTRAIT
    registerBrowserBridge()
    super.onCreate(savedInstanceState)
  }

  private fun hideBars(type: Int) {
    WindowCompat.setDecorFitsSystemWindows(window, false)

    WindowInsetsControllerCompat(window, window.decorView).let { controller ->
      controller.hide(type)
      controller.systemBarsBehavior =
        WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
    }
  }

  private fun showBars(type: Int) {
    WindowCompat.setDecorFitsSystemWindows(window, true)
    WindowInsetsControllerCompat(window, window.decorView).show(type)
  }

  private fun registerBrowserBridge() {
    getPluginManager().load(
      null,
      "lieAndroidBridge",
      object : Plugin(this) {
        override fun load(webView: WebView) {
          webView.addJavascriptInterface(AndroidBridge(), "LieAndroid")
        }
      },
      "{}"
    )
  }

  inner class AndroidBridge {
    // 前端调用 window.LieAndroid.lockLandscape()：把当前 Activity 锁定为横屏。
    @JavascriptInterface
    fun lockLandscape() {
      runOnUiThread {
        requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
      }
    }

    // 前端调用 window.LieAndroid.lockPortrait()：把当前 Activity 锁定为竖屏。
    @JavascriptInterface
    fun lockPortrait() {
      runOnUiThread {
        requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_PORTRAIT
      }
    }

    // 前端调用 window.LieAndroid.unlockOrientation()：取消方向锁定，交还给 Android 系统。
    @JavascriptInterface
    fun unlockOrientation() {
      runOnUiThread {
        requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED
      }
    }

    // 前端调用 window.LieAndroid.hideNavigation()：只隐藏底部导航栏。
    @JavascriptInterface
    fun hideNavigation() {
      runOnUiThread {
        hideBars(WindowInsetsCompat.Type.navigationBars())
      }
    }

    // 前端调用 window.LieAndroid.showNavigation()：显示底部导航栏。
    @JavascriptInterface
    fun showNavigation() {
      runOnUiThread {
        showBars(WindowInsetsCompat.Type.navigationBars())
      }
    }

    // 前端调用 window.LieAndroid.hideStatusBar()：只隐藏顶部状态栏。
    @JavascriptInterface
    fun hideStatusBar() {
      runOnUiThread {
        hideBars(WindowInsetsCompat.Type.statusBars())
      }
    }

    // 前端调用 window.LieAndroid.showStatusBar()：显示顶部状态栏。
    @JavascriptInterface
    fun showStatusBar() {
      runOnUiThread {
        showBars(WindowInsetsCompat.Type.statusBars())
      }
    }
  }
}
