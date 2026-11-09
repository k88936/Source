package top.k88936.source.mobile.mediastore

import android.Manifest
import android.app.Activity
import android.content.ContentUris
import android.content.pm.PackageManager
import android.database.Cursor
import android.graphics.Bitmap
import android.net.Uri
import android.os.Build
import android.provider.MediaStore
import android.util.Base64
import android.util.Size
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Permission
import app.tauri.annotation.PermissionCallback
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.io.ByteArrayOutputStream
import java.util.concurrent.Executors

/**
 * Address prefix the Rust side stores as a local album: MediaStore is a flat
 * list of buckets, so a bucket's `RELATIVE_PATH` is the whole identity and the
 * prefix only keeps it apart from the desktop adapter's paths.
 */
private const val BUCKET_SCHEME = "mediastore:"

private const val PERMISSION_STORAGE = "storage"
private const val PERMISSION_MEDIA = "media"

private const val GRANT_FULL = "full"
private const val GRANT_PARTIAL = "partial"
private const val GRANT_DENIED = "denied"

private const val ASKED = "asked"

@InvokeArg
internal class AlbumArgs {
  lateinit var uri: String
}

@InvokeArg
internal class AssetPageArgs {
  lateinit var uri: String
  var cursor: String? = null
  var limit: Int = 0
}

@InvokeArg
internal class AssetArgs {
  lateinit var id: String
}

@InvokeArg
internal class ThumbArgs {
  lateinit var id: String
  var edge: Int = 0
}

/**
 * The device photo library, reached the only way Android allows: MediaStore
 * rows behind a runtime permission.
 *
 * Android has no `$HOME` and no directories a plain file API may walk, so the
 * desktop adapter's model - a directory the user adds - becomes a bucket here:
 * a `RELATIVE_PATH` such as `DCIM/Camera/` that the user picks out of a flat
 * list. Opening that list is what raises the system permission dialog; nothing
 * else in the app asks for it.
 */
@TauriPlugin(
  permissions = [
    Permission(strings = [Manifest.permission.READ_EXTERNAL_STORAGE], alias = PERMISSION_STORAGE),
    Permission(
      strings = [Manifest.permission.READ_MEDIA_IMAGES, Manifest.permission.READ_MEDIA_VIDEO],
      alias = PERMISSION_MEDIA
    )
  ]
)
class MediaStorePlugin(private val context: Activity) : Plugin(context) {

  /**
   * Every command lands here from the main thread, but a MediaStore query on
   * the main thread is a stall the system will complain about, so the work is
   * serialized onto one background thread. One thread, not a pool: cursors
   * and the plugin's own state then never interleave.
   */
  private val io = Executors.newSingleThreadExecutor { Thread(it, "mediastore") }

  private val files: Uri = MediaStore.Files.getContentUri(MediaStore.VOLUME_EXTERNAL)

  private val columns = arrayOf(
    MediaStore.MediaColumns._ID,
    MediaStore.MediaColumns.DISPLAY_NAME,
    MediaStore.MediaColumns.MIME_TYPE,
    MediaStore.MediaColumns.SIZE,
    MediaStore.MediaColumns.DATE_TAKEN,
    MediaStore.MediaColumns.DATE_MODIFIED,
    MediaStore.MediaColumns.WIDTH,
    MediaStore.MediaColumns.HEIGHT,
    MediaStore.MediaColumns.DURATION,
    MediaStore.MediaColumns.RELATIVE_PATH,
    MediaStore.MediaColumns.BUCKET_DISPLAY_NAME
  )

  /**
   * Photos and videos only: the same rows the gallery shows.
   */
  private val mediaSelection =
    "${MediaStore.Files.FileColumns.MEDIA_TYPE} IN (" +
      "${MediaStore.Files.FileColumns.MEDIA_TYPE_IMAGE}, " +
      "${MediaStore.Files.FileColumns.MEDIA_TYPE_VIDEO})"

  private val bucketSelection =
    "$mediaSelection AND ${MediaStore.MediaColumns.RELATIVE_PATH} = ?"

  private val prefs by lazy { context.getSharedPreferences("MediaStore", Activity.MODE_PRIVATE) }

  private val resolver get() = context.contentResolver

  // ---------------------------------------------------------------------------
  // Permission
  // ---------------------------------------------------------------------------

  @Command
  fun permissionState(invoke: Invoke) {
    invoke.resolve(stateJson())
  }

  @Command
  fun requestPermission(invoke: Invoke) {
    ask(invoke, "onRequested")
  }

  @PermissionCallback
  fun onRequested(invoke: Invoke) {
    invoke.resolve(stateJson())
  }

  private fun grant(): String {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
      return if (check(Manifest.permission.READ_EXTERNAL_STORAGE)) GRANT_FULL else GRANT_DENIED
    }
    if (check(Manifest.permission.READ_MEDIA_IMAGES) && check(Manifest.permission.READ_MEDIA_VIDEO)) {
      return GRANT_FULL
    }
    // Android 14 lets the user hand over a selection instead of the library,
    // which is a real grant: the rows they picked are readable, nothing else.
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE &&
      check(Manifest.permission.READ_MEDIA_VISUAL_USER_SELECTED)
    ) {
      return GRANT_PARTIAL
    }
    return GRANT_DENIED
  }

  private fun stateJson(): JSObject {
    val grant = grant()
    val json = JSObject()
    json.put("granted", grant)
    json.put("canRequest", when (grant) {
      GRANT_FULL -> false
      GRANT_PARTIAL -> true
      else -> !permanentlyDenied()
    })
    return json
  }

  /**
   * Whether the system still shows the dialog. A denial that came with
   * "don't ask again" is only recognizable afterwards, from the missing
   * rationale, so the fact that we asked is recorded when we do.
   */
  private fun permanentlyDenied(): Boolean =
    prefs.getBoolean(ASKED, false) &&
      !ActivityCompat.shouldShowRequestPermissionRationale(context, primaryPermission())

  private fun primaryPermission(): String =
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
      Manifest.permission.READ_MEDIA_IMAGES
    } else {
      Manifest.permission.READ_EXTERNAL_STORAGE
    }

  private fun check(permission: String): Boolean =
    ContextCompat.checkSelfPermission(context, permission) == PackageManager.PERMISSION_GRANTED

  private fun ask(invoke: Invoke, callback: String) {
    prefs.edit().putBoolean(ASKED, true).apply()
    val alias =
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) PERMISSION_MEDIA else PERMISSION_STORAGE
    requestPermissionForAlias(alias, invoke, callback)
  }

  // ---------------------------------------------------------------------------
  // Library
  // ---------------------------------------------------------------------------

  /**
   * The flat bucket list. Asking for it is what opens the permission dialog,
   * because there is nothing to show before the answer.
   */
  @Command
  fun browse(invoke: Invoke) {
    when (grant()) {
      GRANT_DENIED -> if (permanentlyDenied()) {
        invoke.reject("相册权限已被拒绝, 请在系统设置中允许", "denied")
      } else {
        ask(invoke, "onBrowseRequested")
      }

      else -> loadBuckets(invoke)
    }
  }

  @PermissionCallback
  fun onBrowseRequested(invoke: Invoke) {
    if (grant() == GRANT_DENIED) {
      invoke.reject("未授予相册权限", "denied")
      return
    }
    loadBuckets(invoke)
  }

  private fun loadBuckets(invoke: Invoke) {
    io.execute {
      try {
        val names = HashMap<String, String>()
        resolver.query(
          files,
          arrayOf(
            MediaStore.MediaColumns.RELATIVE_PATH,
            MediaStore.MediaColumns.BUCKET_DISPLAY_NAME
          ),
          mediaSelection,
          null,
          null
        )?.use { cursor ->
          while (cursor.moveToNext()) {
            val path = cursor.getString(0) ?: continue
            // The volume root is not an album: rows dropped straight into
            // storage report "/" (or nothing) and no bucket name at all.
            if (path == "/" || path.isEmpty()) continue
            val name = cursor.getString(1).orEmpty().ifBlank { dirName(path) }
            if (!names.containsKey(path)) {
              names[path] = name
            }
          }
        }
        val entries = names.entries
          .map { (path, name) -> entry(path, name) }
          .sortedBy { it.getString("name").lowercase() }
        val page = JSObject()
        page.put("entries", JSArray(entries))
        invoke.resolve(page)
      } catch (error: Exception) {
        reject(invoke, error)
      }
    }
  }

  private fun entry(path: String, name: String): JSObject {
    val json = JSObject()
    json.put("uri", BUCKET_SCHEME + path)
    json.put("name", name)
    json.put("path", path.trimEnd('/'))
    return json
  }

  /**
   * One page of a bucket, newest first. The library has no cursor to seek, so
   * a page is the bucket's rows walked up to its offset - the alternative is
   * a full read of every page into memory.
   */
  @Command
  fun assets(invoke: Invoke) {
    val args = invoke.parseArgs(AssetPageArgs::class.java)
    io.execute {
      try {
        if (grant() == GRANT_DENIED) {
          invoke.reject("未授予相册权限", "permission")
          return@execute
        }
        val path = bucketPath(args.uri)
        val offset = args.cursor?.toIntOrNull() ?: 0
        val limit = args.limit.coerceIn(1, 200)
        val items = JSArray()
        var more = false
        var seen = 0
        resolver.query(
          files,
          columns,
          bucketSelection,
          arrayOf(path),
          "${MediaStore.MediaColumns.DATE_MODIFIED} DESC"
        )?.use { cursor ->
          while (cursor.moveToNext()) {
            if (seen < offset) {
              seen += 1
              continue
            }
            if (items.length() == limit) {
              more = true
              break
            }
            items.put(assetJson(cursor))
            seen += 1
          }
        }
        val page = JSObject()
        page.put("items", items)
        if (more) {
          page.put("next", (offset + limit).toString())
        }
        invoke.resolve(page)
      } catch (error: Exception) {
        reject(invoke, error)
      }
    }
  }

  @Command
  fun count(invoke: Invoke) {
    val args = invoke.parseArgs(AlbumArgs::class.java)
    io.execute {
      try {
        if (grant() == GRANT_DENIED) {
          invoke.reject("未授予相册权限", "permission")
          return@execute
        }
        val total = resolver.query(
          files,
          arrayOf(MediaStore.MediaColumns._ID),
          bucketSelection,
          arrayOf(bucketPath(args.uri)),
          null
        )?.use { it.count } ?: 0
        invoke.resolveObject(total)
      } catch (error: Exception) {
        reject(invoke, error)
      }
    }
  }

  @Command
  fun asset(invoke: Invoke) {
    val args = invoke.parseArgs(AssetArgs::class.java)
    io.execute {
      try {
        val row = resolver.query(Uri.parse(args.id), columns, null, null, null)?.use { cursor ->
          if (cursor.moveToFirst()) assetJson(cursor) else null
        }
        if (row == null) invoke.resolve() else invoke.resolve(row)
      } catch (error: Exception) {
        reject(invoke, error)
      }
    }
  }

  @Command
  fun thumb(invoke: Invoke) {
    val args = invoke.parseArgs(ThumbArgs::class.java)
    io.execute {
      try {
        val bitmap = resolver.loadThumbnail(Uri.parse(args.id), Size(args.edge, args.edge), null)
        val out = ByteArrayOutputStream()
        bitmap.compress(Bitmap.CompressFormat.JPEG, 85, out)
        bitmap.recycle()
        invoke.resolveObject(Base64.encodeToString(out.toByteArray(), Base64.NO_WRAP))
      } catch (error: Exception) {
        reject(invoke, error)
      }
    }
  }

  // ---------------------------------------------------------------------------
  // Rows
  // ---------------------------------------------------------------------------

  private fun assetJson(cursor: Cursor): JSObject {
    val id = cursor.getLong(cursor.getColumnIndexOrThrow(MediaStore.MediaColumns._ID))
    val uri = ContentUris.withAppendedId(files, id).toString()
    val modified = cursor.getLong(cursor.getColumnIndexOrThrow(MediaStore.MediaColumns.DATE_MODIFIED)) * 1000
    val taken = cursor.getLong(cursor.getColumnIndexOrThrow(MediaStore.MediaColumns.DATE_TAKEN))
    val json = JSObject()
    json.put("id", uri)
    json.put("uri", uri)
    json.put("name", cursor.getString(cursor.getColumnIndexOrThrow(MediaStore.MediaColumns.DISPLAY_NAME)))
    json.put("size", cursor.getLong(cursor.getColumnIndexOrThrow(MediaStore.MediaColumns.SIZE)))
    json.put("dateTaken", if (taken > 0) taken else modified)
    val mime = cursor.getString(cursor.getColumnIndexOrThrow(MediaStore.MediaColumns.MIME_TYPE))
    if (!mime.isNullOrEmpty()) json.put("mime", mime)
    val width = cursor.getInt(cursor.getColumnIndexOrThrow(MediaStore.MediaColumns.WIDTH))
    if (width > 0) json.put("width", width)
    val height = cursor.getInt(cursor.getColumnIndexOrThrow(MediaStore.MediaColumns.HEIGHT))
    if (height > 0) json.put("height", height)
    val duration = cursor.getLong(cursor.getColumnIndexOrThrow(MediaStore.MediaColumns.DURATION))
    if (duration > 0) json.put("durationMs", duration)
    return json
  }

  private fun bucketPath(uri: String): String {
    require(uri.startsWith(BUCKET_SCHEME)) { "未知的相册地址: $uri" }
    return uri.substring(BUCKET_SCHEME.length)
  }

  private fun dirName(path: String): String = path.trimEnd('/').substringAfterLast('/')

  private fun reject(invoke: Invoke, error: Exception) {
    invoke.reject(error.message ?: error.toString(), "internal", error)
  }
}
