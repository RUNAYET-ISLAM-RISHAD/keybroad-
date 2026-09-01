package com.keybroad.update

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.pm.PackageInstaller
import android.widget.Toast

/**
 * Receives the PackageInstaller session commit result and surfaces it to the user.
 * On INSTALL_FAILED_UPDATE_INCOMPATIBLE (signature mismatch with the legacy
 * locally-signed build), prompts the one-time bootstrap: uninstall + reinstall.
 */
class UpdateInstallReceiver : BroadcastReceiver() {

    override fun onReceive(context: Context, intent: Intent) {
        when (intent.action) {
            UpdateManager.ACTION_INSTALL_STATUS -> {
                val status = intent.getIntExtra(
                    PackageInstaller.EXTRA_STATUS,
                    PackageInstaller.STATUS_FAILURE
                )
                when (status) {
                    PackageInstaller.STATUS_SUCCESS -> {
                        Toast.makeText(
                            context,
                            "Keybroad updated successfully",
                            Toast.LENGTH_LONG
                        ).show()
                        // App process is killed on install; this toast shows if we
                        // are re-installed as the same process (rare). No-op otherwise.
                    }
                    PackageInstaller.STATUS_FAILURE,
                    PackageInstaller.STATUS_FAILURE_ABORTED,
                    PackageInstaller.STATUS_FAILURE_BLOCKED,
                    PackageInstaller.STATUS_FAILURE_CONFLICT,
                    PackageInstaller.STATUS_FAILURE_INCOMPATIBLE,
                    PackageInstaller.STATUS_FAILURE_INVALID,
                    PackageInstaller.STATUS_FAILURE_STORAGE -> {
                        val message = intent.getStringExtra(
                            PackageInstaller.EXTRA_STATUS_MESSAGE
                        ) ?: "Installation failed"
                        // Signature mismatch => offer one-time bootstrap
                        if (message.contains("INSTALL_FAILED_UPDATE_INCOMPATIBLE", ignoreCase = true) ||
                            status == PackageInstaller.STATUS_FAILURE_CONFLICT ||
                            status == PackageInstaller.STATUS_FAILURE_INCOMPATIBLE
                        ) {
                            showBootstrapDialog(context, message)
                        } else {
                            Toast.makeText(context, "Update failed: $message", Toast.LENGTH_LONG).show()
                        }
                    }
                    else -> {
                        Toast.makeText(context, "Update failed (status $status)", Toast.LENGTH_LONG).show()
                    }
                }
            }
        }
    }

    private fun showBootstrapDialog(context: Context, details: String) {
        val intent = Intent(context, com.keybroad.ui.MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP
            putExtra("bootstrap_required", true)
            putExtra("bootstrap_details", details)
        }
        context.startActivity(intent)
        Toast.makeText(
            context,
            "Signature mismatch detected. Please uninstall and reinstall the new version.",
            Toast.LENGTH_LONG
        ).show()
    }
}
