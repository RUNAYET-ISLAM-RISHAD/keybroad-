package com.keybroad.update

import android.app.Activity
import android.app.AlertDialog
import android.app.DownloadManager
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.Build
import android.widget.Toast

class UpdateChecker(
    private val activity: Activity,
    private val updateManager: UpdateManager
) {
    private val downloadCompleteReceiver = DownloadCompleteReceiver()

    fun checkForUpdate() {
        updateManager.checkForUpdate { updateInfo ->
            if (updateInfo != null) {
                showUpdateDialog(updateInfo)
            }
        }
    }

    private fun showUpdateDialog(updateInfo: UpdateInfo) {
        activity.runOnUiThread {
            val dialog = AlertDialog.Builder(activity)
                .setTitle("Update Available")
                .setMessage("Version ${updateInfo.version} is available.\n\n${updateInfo.changelog}\n\nWould you like to update now?")
                .setPositiveButton("Update") { _, _ ->
                    downloadAndInstall(updateInfo)
                }
                .setNegativeButton("Later") { dialog, _ ->
                    dialog.dismiss()
                }
                .setCancelable(false)
                .create()

            dialog.show()
        }
    }

    private fun downloadAndInstall(updateInfo: UpdateInfo) {
        // Check if already downloaded
        val existingFile = updateManager.isUpdateDownloaded(updateInfo.version)
        if (existingFile != null) {
            updateManager.installApk(existingFile)
            return
        }

        // Register receiver for download complete
        try {
            val filter = IntentFilter(DownloadManager.ACTION_DOWNLOAD_COMPLETE)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                activity.registerReceiver(downloadCompleteReceiver, filter, Context.RECEIVER_NOT_EXPORTED)
            } else {
                activity.registerReceiver(downloadCompleteReceiver, filter)
            }
        } catch (e: Exception) {
            // Receiver might already be registered
        }

        updateManager.downloadAndInstall(updateInfo)
        Toast.makeText(activity, "Download started...", Toast.LENGTH_SHORT).show()
    }

    fun onDestroy() {
        try {
            activity.unregisterReceiver(downloadCompleteReceiver)
        } catch (e: Exception) {
            // Receiver not registered
        }
    }
}
