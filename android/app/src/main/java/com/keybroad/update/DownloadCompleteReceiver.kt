package com.keybroad.update

import android.app.DownloadManager
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import java.io.File

class DownloadCompleteReceiver : BroadcastReceiver() {

    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action == DownloadManager.ACTION_DOWNLOAD_COMPLETE) {
            val downloadId = intent.getLongExtra(DownloadManager.EXTRA_DOWNLOAD_ID, -1)

            val prefs = context.getSharedPreferences("update_prefs", Context.MODE_PRIVATE)
            val storedDownloadId = prefs.getInt("download_id", -1).toLong()

            if (downloadId == storedDownloadId) {
                val downloadManager = context.getSystemService(Context.DOWNLOAD_SERVICE) as DownloadManager
                val query = DownloadManager.Query().setFilterById(downloadId)
                val cursor = downloadManager.query(query)

                if (cursor.moveToFirst()) {
                    val statusIndex = cursor.getColumnIndex(DownloadManager.COLUMN_STATUS)
                    val status = cursor.getInt(statusIndex)

                    if (status == DownloadManager.STATUS_SUCCESSFUL) {
                        val version = prefs.getString("download_version", "") ?: ""
                        val apkFile = File(context.getExternalFilesDir("updates"), "keybroad-$version.apk")

                        if (apkFile.exists()) {
                            installApk(context, apkFile)
                        }
                    }
                }
                cursor.close()
            }
        }
    }

    private fun installApk(context: Context, apkFile: File) {
        try {
            val uri = androidx.core.content.FileProvider.getUriForFile(
                context,
                "${context.packageName}.fileprovider",
                apkFile
            )

            val intent = Intent(Intent.ACTION_VIEW).apply {
                setDataAndType(uri, "application/vnd.android.package-archive")
                flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_GRANT_READ_URI_PERMISSION
            }

            context.startActivity(intent)
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }
}
