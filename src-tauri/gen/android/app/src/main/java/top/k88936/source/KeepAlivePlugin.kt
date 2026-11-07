package top.k88936.source

import android.app.Activity
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import androidx.core.app.NotificationCompat
import androidx.core.content.ContextCompat
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin

/** The one channel the service posts on; there is a single notification. */
private const val CHANNEL = "keep_alive"

/** The service's notification id: a pass or three, they share one. */
private const val NOTIFICATION_ID = 1

/** The wake lock's name, which is what the battery screen shows a user. */
private const val LOCK = "source:keep-alive"

@InvokeArg
internal class SyncArgs {
  var hold: Boolean = false
}

/**
 * What the caller last said, and whether the service has answered its start.
 *
 * The caller reports a state, but the platform counts starts: a start that was
 * requested and then cancelled before `startForeground` runs crashes the whole
 * app, so the two are tracked apart and a release only stops a service that has
 * already answered. The state being reported is still the caller's; nothing
 * about what is running is decided here.
 */
internal object Hold {
  @Volatile
  var wanted = false

  @Volatile
  var answered = false
}

/**
 * Holds the process open while background work is allowed.
 *
 * Android freezes, then kills, a background process, so work that outlives the
 * window - a pass over the albums, the batch it queued - has to run behind a
 * foreground service, and a partial wake lock keeps the CPU up under it while
 * the screen is off. The notification is what the platform charges for either.
 *
 * The caller reports one fact - whether the process should be held - and this
 * side only starts or stops the service to match. It owns no work, no set of
 * keys and no line to show: what is running never reaches this class.
 */
@TauriPlugin
class KeepAlivePlugin(private val context: Activity) : Plugin(context) {

  @Command
  fun sync(invoke: Invoke) {
    val args = invoke.parseArgs(SyncArgs::class.java)
    // The application context outlives the activity; the service lives as long
    // as the work does, which is a different lifetime.
    val app = context.applicationContext
    val service = Intent(app, KeepAliveService::class.java)
    Hold.wanted = args.hold
    if (args.hold) {
      // Starting an already-running service is how the platform is told the
      // work is still going: it turns it into one more callback, not a second.
      ContextCompat.startForegroundService(app, service)
    } else if (Hold.answered) {
      // Only a service that has already posted its notification may be
      // cancelled: stopping one whose start is still in flight makes the
      // platform kill the app over an answer that never came.
      app.stopService(service)
    }
    // A release that arrives mid-start is left to the service, which reads
    // `wanted` once it has answered and stops itself from there.
    invoke.resolve()
  }
}

/**
 * The process, held while background work is allowed.
 *
 * `dataSync` is the platform's word for exactly this: moving bytes for the user
 * with the screen off.
 */
class KeepAliveService : Service() {
  private var lock: PowerManager.WakeLock? = null

  override fun onBind(intent: Intent?): IBinder? = null

  override fun onCreate() {
    super.onCreate()
    val power = getSystemService(Context.POWER_SERVICE) as PowerManager
    lock = power.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, LOCK).apply { acquire() }
  }

  override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
    startForeground(NOTIFICATION_ID, notification())
    Hold.answered = true
    // The caller let go while this start was in flight, so the release could
    // not cancel it; the start is answered above, and the stop is taken here.
    if (!Hold.wanted) {
      stopSelf()
    }
    // A restart would come with no caller left to report to; the switch state is
    // durable on its own, and the next sync corrects this either way.
    return START_NOT_STICKY
  }

  override fun onDestroy() {
    Hold.answered = false
    lock?.release()
    lock = null
    stopForeground(STOP_FOREGROUND_REMOVE)
    super.onDestroy()
  }

  /**
   * The platform's data-sync budget is spent. The work is durable - an upload
   * batch resumes from its sessions, a pass resumes at the next start - so it is
   * left to that instead of being held past the limit.
   */
  override fun onTimeout(startId: Int, fgsType: Int) {
    stopSelf()
  }

  private fun notification(): Notification {
    channel()
    val open = PendingIntent.getActivity(
      this,
      0,
      Intent(this, MainActivity::class.java),
      PendingIntent.FLAG_IMMUTABLE
    )
    return NotificationCompat.Builder(this, CHANNEL)
      .setSmallIcon(android.R.drawable.stat_notify_sync)
      .setContentTitle(applicationInfo.loadLabel(packageManager))
      .setContentText("正在后台处理")
      .setContentIntent(open)
      .setOngoing(true)
      .setOnlyAlertOnce(true)
      .setSilent(true)
      .setForegroundServiceBehavior(NotificationCompat.FOREGROUND_SERVICE_IMMEDIATE)
      .build()
  }

  private fun channel() {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) {
      return
    }
    val manager = getSystemService(NotificationManager::class.java)
    manager.createNotificationChannel(
      NotificationChannel(CHANNEL, "后台任务", NotificationManager.IMPORTANCE_LOW)
    )
  }
}
