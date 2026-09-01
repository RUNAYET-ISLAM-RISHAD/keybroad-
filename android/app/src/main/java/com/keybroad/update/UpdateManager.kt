package com.keybroad.update

import android.app.DownloadManager
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import androidx.core.content.FileProvider
import org.json.JSONObject
import java.io.File
import java.net.HttpURLConnection
import java.net.URL

data class UpdateInfo(
    val version: String,
    val versionCode: Int,
    val apkUrl: String,
    val changelog: String
)

class UpdateManager(private val context: Context) {

    companion object {
        private const val UPDATE_CONFIG_URL = "https://raw.githubusercontent.com/keybroad/keybroad/main/android/update_config.json"
        private const val UPDATE_CONFIG_FALLBACK_URL = "https://files.catbox.moe/pwpmy4.json"
    }

    private val prefs = context.getSharedPreferences("update_prefs", Context.MODE_PRIVATE)

    fun getCurrentVersionCode(): Int {
        return try {
            val packageInfo = context.packageManager.getPackageInfo(context.packageName, 0)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                packageInfo.longVersionCode.toInt()
            } else {
                @Suppress("DEPRECATION")
                packageInfo.versionCode
            }
        } catch (e: Exception) {
            0
        }
    }

    fun getCurrentVersionName(): String {
        return try {
            context.packageManager.getPackageInfo(context.packageName, 0).versionName ?: "1.0.0"
        } catch (e: Exception) {
            "1.0.0"
        }
    }

    fun checkForUpdate(callback: (UpdateInfo?) -> Unit) {
        Thread {
            try {
                val updateInfo = fetchUpdateInfo(UPDATE_CONFIG_URL) ?: fetchUpdateInfo(UPDATE_CONFIG_FALLBACK_URL)
                if (updateInfo != null && updateInfo.versionCode > getCurrentVersionCode() && updateInfo.apkUrl.isNotEmpty()) {
                    callback(updateInfo)
                } else {
                    callback(null)
                }
            } catch (e: Exception) {
                e.printStackTrace()
                callback(null)
            }
        }.start()
    }

    private fun fetchUpdateInfo(urlString: String): UpdateInfo? {
        return try {
            val url = URL(urlString)
            val connection = url.openConnection() as HttpURLConnection
            connection.requestMethod = "GET"
            connection.connectTimeout = 10000
            connection.readTimeout = 10000
            if (connection.responseCode == HttpURLConnection.HTTP_OK) {
                val response = connection.inputStream.bufferedReader().readText()
                val json = JSONObject(response)
                val info = UpdateInfo(
                    version = json.optString("version", "1.0.0"),
                    versionCode = json.optInt("version_code", 0),
                    apkUrl = json.optString("apk_url", ""),
                    changelog = json.optString("changelog", "")
                )
                connection.disconnect()
                info
            } else {
                connection.disconnect()
                null
            }
        } catch (e: Exception) {
            e.printStackTrace()
            null
        }
    }

    fun downloadAndInstall(updateInfo: UpdateInfo) {
        try {
            val downloadManager = context.getSystemService(Context.DOWNLOAD_SERVICE) as DownloadManager

            val request = DownloadManager.Request(Uri.parse(updateInfo.apkUrl)).apply {
                setTitle("Downloading Keybroad Update")
                setDescription("Version ${updateInfo.version}")
                setNotificationVisibility(DownloadManager.Request.VISIBILITY_VISIBLE_NOTIFY_COMPLETED)
                setDestinationInExternalFilesDir(context, "updates", "keybroad-${updateInfo.version}.apk")
            }

            val downloadId = downloadManager.enqueue(request)

            prefs.edit().apply {
                putInt("download_id", downloadId.toInt())
                putString("download_version", updateInfo.version)
                putString("download_url", updateInfo.apkUrl)
                apply()
            }
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    fun installApk(apkFile: File) {
        try {
            val uri = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
                FileProvider.getUriForFile(
                    context,
                    "${context.packageName}.fileprovider",
                    apkFile
                )
            } else {
                Uri.fromFile(apkFile)
            }

            val intent = Intent(Intent.ACTION_VIEW).apply {
                setDataAndType(uri, "application/vnd.android.package-archive")
                flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_GRANT_READ_URI_PERMISSION
            }

            context.startActivity(intent)
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    fun isUpdateDownloaded(version: String): File? {
        val file = File(context.getExternalFilesDir("updates"), "keybroad-$version.apk")
        return if (file.exists()) file else null
    }
}
