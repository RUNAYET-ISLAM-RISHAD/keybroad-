package com.keybroad.update

import android.app.Activity
import android.app.AlertDialog
import android.widget.Toast

/**
 * Orchestrates the OTA flow: check config -> prompt user -> download -> validate
 * signature & package name -> install via PackageInstaller Session API.
 */
class UpdateChecker(
    private val activity: Activity,
    private val updateManager: UpdateManager
) {

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
        // Reuse a fully downloaded file if present
        val existingFile = updateManager.isUpdateDownloaded(updateInfo.version)
        if (existingFile != null && existingFile.length() > 0) {
            validateAndInstall(existingFile, updateInfo)
            return
        }

        Toast.makeText(activity, "Downloading update...", Toast.LENGTH_SHORT).show()

        updateManager.downloadApk(updateInfo) { apkFile ->
            activity.runOnUiThread {
                if (apkFile == null) {
                    Toast.makeText(
                        activity,
                        "Download failed. Please try again later.",
                        Toast.LENGTH_LONG
                    ).show()
                    return@runOnUiThread
                }
                validateAndInstall(apkFile, updateInfo)
            }
        }
    }

    private fun validateAndInstall(apkFile: java.io.File, updateInfo: UpdateInfo) {
        // 1. Package name check: APK must belong to this app
        if (!updateManager.isValidApkForThisApp(apkFile, updateInfo.packageName)) {
            Toast.makeText(
                activity,
                "Update file is invalid (wrong package). Aborting install.",
                Toast.LENGTH_LONG
            ).show()
            return
        }

        // 2. Signature pre-check: warn (not block) on mismatch before install attempt
        if (!updateManager.isSignatureCompatible(apkFile)) {
            showMismatchDialog(apkFile)
            return
        }

        // 3. Install via PackageInstaller session
        updateManager.installApk(apkFile)
        Toast.makeText(activity, "Installing update...", Toast.LENGTH_SHORT).show()
    }

    /**
     * Pre-install signature mismatch: the installed build was signed with a
     * different (legacy) key. Offer the one-time bootstrap.
     */
    private fun showMismatchDialog(apkFile: java.io.File) {
        val dialog = AlertDialog.Builder(activity)
            .setTitle("Signature Mismatch")
            .setMessage(
                "The installed version was signed with an old key and cannot be " +
                "updated directly.\n\nTo switch to the new stable key (one-time step), " +
                "uninstall the current app, then install the new version. After this, " +
                "all future updates will be fully automatic."
            )
            .setPositiveButton("Uninstall & Reinstall") { _, _ ->
                // Step 1: kick off uninstall; the user then reinstalls the
                // already-downloaded APK from storage (also keep OTA URL available).
                updateManager.uninstallForBootstrap()
                Toast.makeText(
                    activity,
                    "After uninstalling, download and install: " +
                    "https://github.com/RUNAYET-ISLAM-RISHAD/keybroad-/releases/latest",
                    Toast.LENGTH_LONG
                ).show()
            }
            .setNegativeButton("Cancel") { d, _ -> d.dismiss() }
            .setCancelable(false)
            .create()
        dialog.show()
    }

    fun onDestroy() {
        // DownloadManager receivers removed; PackageInstaller receiver is
        // manifest-declared and does not need lifecycle handling.
    }
}
