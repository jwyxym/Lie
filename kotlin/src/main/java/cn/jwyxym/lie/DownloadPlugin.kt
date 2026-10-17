package cn.jwyxym.lie

import android.Manifest
import android.app.Activity
import android.content.Intent
import android.os.Build
import app.tauri.PermissionState
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Permission
import app.tauri.annotation.PermissionCallback
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Channel
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin

@InvokeArg
class DownloadPrepareArgs {
	lateinit var events: Channel
}

@InvokeArg
class DownloadSnapshotArgs {
	var tasks: List<DownloadTask> = emptyList()
	var startedId: Long? = null
}

@InvokeArg
class DownloadTask {
	var id: Long = 0
	var animeTitle: String = ""
	var episodeTitle: String = ""
	var status: String = ""
	var downloadedBytes: Long = 0
	var totalBytes: Long? = null
	var completedSegments: Long = 0
	var totalSegments: Long = 0
	var error: String = ""

	fun active(): Boolean = status == "queued" || status == "downloading" || status == "merging"
}

@TauriPlugin(permissions = [Permission(strings = [Manifest.permission.POST_NOTIFICATIONS], alias = "notifications")])
class DownloadPlugin(private val activity: Activity) : Plugin(activity) {
	@Command
	fun prepare(invoke: Invoke) {
		DownloadService.events = invoke.parseArgs(DownloadPrepareArgs::class.java).events
		val preferences = activity.getSharedPreferences("download-notifications", 0)
		if (Build.VERSION.SDK_INT >= 33 && getPermissionState("notifications") != PermissionState.GRANTED && !preferences.getBoolean("asked", false)) {
			preferences.edit().putBoolean("asked", true).apply()
			requestPermissionForAlias("notifications", invoke, "notificationPermissionResult")
		} else {
			begin(invoke)
		}
	}

	@PermissionCallback
	fun notificationPermissionResult(invoke: Invoke) {
		// A denied notification permission does not prohibit a foreground service.
		begin(invoke)
	}

	private fun begin(invoke: Invoke) {
		DownloadService.begin(activity) { error ->
			if (error == null) invoke.resolve() else invoke.reject(error)
		}
	}

	@Command
	fun update(invoke: Invoke) {
		try {
			val snapshot = invoke.parseArgs(DownloadSnapshotArgs::class.java)
			val service = DownloadService.instance
			if (service == null && snapshot.tasks.any { it.active() }) {
				invoke.reject("后台下载服务已停止，请返回应用重试")
				return
			}
			service?.update(snapshot.tasks, snapshot.startedId)
			invoke.resolve()
		} catch (error: Exception) {
			invoke.reject("更新下载通知失败", error)
		}
	}

	@Command
	fun takeOpen(invoke: Invoke) {
		val open = activity.intent?.getBooleanExtra(DownloadService.OPEN_DOWNLOADS, false) == true
		activity.intent?.removeExtra(DownloadService.OPEN_DOWNLOADS)
		invoke.resolveObject(open)
	}

	override fun onNewIntent(intent: Intent) {
		if (intent.getBooleanExtra(DownloadService.OPEN_DOWNLOADS, false)) {
			activity.intent = intent
		}
	}
}
