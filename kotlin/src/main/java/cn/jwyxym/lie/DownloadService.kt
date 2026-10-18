package cn.jwyxym.lie

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import android.os.SystemClock
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.app.ServiceCompat
import androidx.core.content.ContextCompat
import app.tauri.plugin.Channel
import app.tauri.plugin.JSObject
import java.util.Locale

class DownloadService : Service() {
	companion object {
		const val OPEN_DOWNLOADS = "lie.openDownloads"
		private const val CHANNEL = "video-downloads"
		private const val SUMMARY_ID = 6100
		private const val GROUP = "lie.downloads"
		private const val CANCEL = "cn.jwyxym.lie.CANCEL_DOWNLOAD"
		private val readyCallbacks = mutableListOf<(String?) -> Unit>()
		var instance: DownloadService? = null
		var events: Channel? = null

		fun begin(context: Context, callback: (String?) -> Unit) {
			val current = instance
			if (current != null) {
				current.preparedAt = SystemClock.elapsedRealtime()
				callback(null)
				return
			}
			readyCallbacks.add(callback)
			try {
				ContextCompat.startForegroundService(context, Intent(context, DownloadService::class.java))
			} catch (error: Exception) {
				finishPreparation("无法启动后台下载服务：${error.message}")
			}
		}

		private fun finishPreparation(error: String?) {
			val callbacks = readyCallbacks.toList()
			readyCallbacks.clear()
			callbacks.forEach { it(error) }
		}
	}

	private val tasks = linkedMapOf<Long, DownloadTask>()
	private var preparedAt = 0L
	private var stopping = false
	private var wakeLock: PowerManager.WakeLock? = null

	override fun onCreate() {
		super.onCreate()
		try {
			if (Build.VERSION.SDK_INT >= 26) {
				getSystemService(NotificationManager::class.java).createNotificationChannel(
					NotificationChannel(CHANNEL, "视频缓存", NotificationManager.IMPORTANCE_LOW).apply {
						description = "视频缓存进度和下载结果"
					}
				)
			}
			ServiceCompat.startForeground(this, SUMMARY_ID, summary(),
				if (Build.VERSION.SDK_INT >= 29) ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC else 0)
			wakeLock = (getSystemService(POWER_SERVICE) as PowerManager)
				.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "$packageName:video-downloads").apply {
					setReferenceCounted(false)
					acquire(10 * 60 * 1000L)
				}
			preparedAt = SystemClock.elapsedRealtime()
			instance = this
		} catch (error: Exception) {
			finishPreparation("无法保持后台下载：${error.message}")
			stopDownloads()
		}
	}

	override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
		if (stopping) return START_NOT_STICKY
		if (intent?.action == CANCEL) {
			val id = intent.getLongExtra("downloadId", 0)
			if (tasks.containsKey(id)) sendEvent("cancel", id)
		} else {
			finishPreparation(null)
		}
		// Rust workers live in this process. Restarting only the service cannot resume them.
		return START_NOT_STICKY
	}

	fun update(snapshot: List<DownloadTask>, startedId: Long?) {
		if (stopping) return
		val manager = NotificationManagerCompat.from(this)
		val visibleIds = snapshot.filter { it.status != "deleting" }.map { it.id }.toSet()
		for (id in tasks.keys.toList()) {
			if (id !in visibleIds) {
				tasks.remove(id)
				manager.cancel("download-$id", 1)
			}
		}
		for (task in snapshot) {
			if (task.active()) {
				tasks[task.id] = task
				post(task, true)
			} else if (tasks.remove(task.id) != null || task.id == startedId) {
				if (task.status == "completed" || task.status == "failed") post(task, false)
			}
		}
		if (tasks.isEmpty() && SystemClock.elapsedRealtime() - preparedAt >= 4000) {
			stopDownloads()
		} else {
			// Renew a bounded lock only while Rust is still sending task snapshots.
			wakeLock?.acquire(10 * 60 * 1000L)
			if (manager.areNotificationsEnabled()) manager.notify(SUMMARY_ID, summary())
		}
	}

	private fun openIntent(): PendingIntent {
		val intent = Intent(this, MainActivity::class.java).apply {
			flags = Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
			putExtra(OPEN_DOWNLOADS, true)
		}
		return PendingIntent.getActivity(this, 0, intent, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
	}

	private fun builder(title: String, text: String): NotificationCompat.Builder =
		NotificationCompat.Builder(this, CHANNEL)
			.setSmallIcon(android.R.drawable.stat_sys_download)
			.setContentTitle(title)
			.setContentText(text)
			.setContentIntent(openIntent())
			.setOnlyAlertOnce(true)
			.setGroup(GROUP)
			.setCategory(NotificationCompat.CATEGORY_PROGRESS)
			.setVisibility(NotificationCompat.VISIBILITY_PRIVATE)

	private fun summary(): Notification = builder("视频缓存",
		if (tasks.isEmpty()) "正在准备下载…" else "${tasks.size} 集正在缓存，点击查看")
		.setGroupSummary(true).setOngoing(true).setProgress(0, 0, true).build()

	private fun bytes(value: Long): String = String.format(Locale.ROOT, "%.1f MiB", value / 1048576.0)

	private fun post(task: DownloadTask, active: Boolean) {
		val manager = NotificationManagerCompat.from(this)
		if (!manager.areNotificationsEnabled()) return
		val percent = when {
			task.status == "merging" -> task.mergeProgress ?: -1
			task.totalSegments > 0 -> (task.completedSegments.toDouble() / task.totalSegments * 100).toInt()
			(task.totalBytes ?: 0) > 0 -> (task.downloadedBytes.toDouble() / task.totalBytes!! * 100).toInt()
			else -> -1
		}.coerceIn(-1, 100)
		val text = when (task.status) {
			"queued" -> "等待下载"
			"merging" -> if (percent < 0) "正在分析视频封装信息" else "正在合并视频 · $percent%"
			"completed" -> "缓存完成 · ${bytes(task.downloadedBytes)}"
			"failed" -> if (task.error.contains("取消")) "下载已取消" else "缓存失败：${task.error}"
			else -> if (percent >= 0) "$percent% · ${bytes(task.downloadedBytes)}" else "正在下载 · ${bytes(task.downloadedBytes)}"
		}
		val notification = builder("${task.animeTitle} · ${task.episodeTitle}", text)
			.setOngoing(active).setAutoCancel(!active)
		if (active) {
			notification.setProgress(100, percent.coerceAtLeast(0), percent < 0 || task.status == "queued")
			val cancel = Intent(this, DownloadService::class.java).apply {
				action = CANCEL
				data = android.net.Uri.parse("lie-download://cancel/${task.id}")
				putExtra("downloadId", task.id)
			}
			notification.addAction(android.R.drawable.ic_menu_close_clear_cancel, "取消",
				PendingIntent.getService(this, 0, cancel, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE))
		} else {
			notification.setSmallIcon(if (task.status == "completed") android.R.drawable.stat_sys_download_done else android.R.drawable.stat_notify_error)
				.setStyle(NotificationCompat.BigTextStyle().bigText(text))
		}
		manager.notify("download-${task.id}", 1, notification.build())
	}

	private fun sendEvent(action: String, id: Long = 0) {
		val event = JSObject()
		event.put("action", action)
		event.put("id", id)
		events?.send(event)
	}

	override fun onTimeout(startId: Int, fgsType: Int) {
		for (task in tasks.values) {
			task.status = "failed"
			task.error = "后台下载达到系统时限，请返回应用重试"
			post(task, false)
		}
		sendEvent("timeout")
		tasks.clear()
		stopDownloads()
	}

	private fun stopDownloads() {
		stopping = true
		instance = null
		ServiceCompat.stopForeground(this, ServiceCompat.STOP_FOREGROUND_REMOVE)
		stopSelf()
	}

	override fun onDestroy() {
		if (instance === this) instance = null
		if (!stopping) sendEvent("stopped")
		for (id in tasks.keys) NotificationManagerCompat.from(this).cancel("download-$id", 1)
		if (wakeLock?.isHeld == true) wakeLock?.release()
		wakeLock = null
		finishPreparation("后台下载服务已停止")
		super.onDestroy()
	}

	override fun onBind(intent: Intent?): IBinder? = null
}
