package com.keybroad.update

import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.os.Build
import android.util.Base64
import org.json.JSONObject
import java.io.File
import java.io.FileInputStream
import java.net.HttpURLConnection
import java.net.URL
import java.security.MessageDigest
import java.security.cert.Certificate

data class UpdateInfo(
    val version: String,
    val versionCode: Int,
    val apkUrl: String,
    val changelog: String,
    val packageName: String = "com.keybroad"
)

class UpdateManager(private val context: Context) {

    companion object {
        private const val UPDATE_CONFIG_URL = "https://raw.githubusercontent.com/RUNAYET-ISLAM-RISHAD/keybroad-/main/android/update_config.json"
        private const val UPDATE_CONFIG_FALLBACK_URL = "https://files.catbox.moe/pwpmy4.json"
        const val ACTION_INSTALL_STATUS = "com.keybroad.update.INSTALL_STATUS"
        const val EXTRA_INSTALL_STATUS = "install_status"
        const val EXTRA_INSTALL_MESSAGE = "install_message"
        const val INSTALL_SUCCEEDED = 1
        const val INSTALL_FAILED = 2
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
            context.packageManager.getPackageInfo(context.packageName, 0).versionName ?: "1.1.1"
        } catch (e: Exception) {
            "1.1.1"
        }
    }

    /**
     * SHA-256 fingerprint of the currently installed app's signing certificate.
     * Used to detect signature mismatch before attempting an install.
     */
    fun getCurrentSignature(): String? {
        return try {
            val cert: Certificate? = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                val packageInfo = context.packageManager.getPackageInfo(
                    context.packageName,
                    android.content.pm.PackageManager.GET_SIGNING_CERTIFICATES
                )
                packageInfo.signingInfo?.apkContentsSigners?.firstOrNull()?.let { sig ->
                    java.security.cert.CertificateFactory.getInstance("X.509")
                        .generateCertificate(java.io.ByteArrayInputStream(sig.toByteArray()))
                }
            } else {
                @Suppress("DEPRECATION")
                val packageInfo = context.packageManager.getPackageInfo(
                    context.packageName,
                    android.content.pm.PackageManager.GET_SIGNATURES
                )
                packageInfo.signatures?.firstOrNull()?.let { sig ->
                    java.security.cert.CertificateFactory.getInstance("X.509")
                        .generateCertificate(java.io.ByteArrayInputStream(sig.toByteArray()))
                }
            }
            cert ?: return null
            val digest = MessageDigest.getInstance("SHA-256").digest(cert.encoded)
            Base64.encodeToString(digest, Base64.NO_WRAP)
        } catch (e: Exception) {
            e.printStackTrace()
            null
        }
    }

    /**
      * SHA-256 fingerprint of a downloaded APK's signing certificate.
      * Returns null if the APK cannot be parsed.
      */
    fun getApkSignature(apkFile: File): String? {
        return try {
            val pm = context.packageManager
            val packageInfo = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                pm.getPackageArchiveInfo(
                    apkFile.absolutePath,
                    android.content.pm.PackageManager.PackageInfoFlags.of(
                        android.content.pm.PackageManager.GET_SIGNING_CERTIFICATES.toLong()
                    )
                )
            } else if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                @Suppress("DEPRECATION")
                pm.getPackageArchiveInfo(apkFile.absolutePath, android.content.pm.PackageManager.GET_SIGNING_CERTIFICATES)
            } else {
                @Suppress("DEPRECATION")
                pm.getPackageArchiveInfo(apkFile.absolutePath, android.content.pm.PackageManager.GET_SIGNATURES)
            }
            val sigs = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                packageInfo?.signingInfo?.apkContentsSigners
            } else {
                @Suppress("DEPRECATION")
                packageInfo?.signatures
            }
            val sig = sigs?.firstOrNull() ?: return null
            val cert = java.security.cert.CertificateFactory.getInstance("X.509")
                .generateCertificate(java.io.ByteArrayInputStream(sig.toByteArray()))
            val digest = MessageDigest.getInstance("SHA-256").digest(cert.encoded)
            Base64.encodeToString(digest, Base64.NO_WRAP)
        } catch (e: Exception) {
            e.printStackTrace()
            null
        }
    }

    /**
     * True if the downloaded APK is signed with the same key as the installed app.
     * If either signature cannot be read, returns true (allow attempt; PackageInstaller
     * will surface INSTALL_FAILED_UPDATE_INCOMPATIBLE if truly mismatched).
     */
    fun isSignatureCompatible(apkFile: File): Boolean {
        val current = getCurrentSignature() ?: return true
        val incoming = getApkSignature(apkFile) ?: return true
        return current == incoming
    }

    /**
      * Validate the downloaded APK: correct package name and installable.
      * Uses PackageInfoFlags on API 33+ (Tiramisu) where the int-flag overload
      * is deprecated and may return null on Android 16 (API 36).
      */
    fun isValidApkForThisApp(apkFile: File, expectedPackage: String): Boolean {
        return try {
            val pm = context.packageManager
            val packageInfo = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                pm.getPackageArchiveInfo(
                    apkFile.absolutePath,
                    android.content.pm.PackageManager.PackageInfoFlags.of(0)
                )
            } else {
                @Suppress("DEPRECATION")
                pm.getPackageArchiveInfo(apkFile.absolutePath, 0)
            }
            android.util.Log.d(
                "UpdateManager",
                "isValidApk: file=${apkFile.absolutePath} pkg=${packageInfo?.packageName} expected=$expectedPackage"
            )
            if (packageInfo == null) {
                // Cannot read package info (e.g., file not yet fully written) —
                // let PackageInstaller decide rather than aborting with wrong-package toast.
                android.util.Log.w("UpdateManager", "getPackageArchiveInfo returned null, allowing install")
                return true
            }
            packageInfo.packageName == expectedPackage
        } catch (e: Exception) {
            android.util.Log.e("UpdateManager", "isValidApk check failed", e)
            // On exception, don't block install with wrong-package error
            true
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
                    version = json.optString("version", "1.1.1"),
                    versionCode = json.optInt("version_code", 0),
                    apkUrl = json.optString("apk_url", ""),
                    changelog = json.optString("changelog", ""),
                    packageName = json.optString("package_name", context.packageName)
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

    /**
     * Download the APK to app-private storage. Calls [onComplete] with the file,
     * or null on failure. Runs on a background thread; callback on same thread.
     */
    fun downloadApk(updateInfo: UpdateInfo, onComplete: (File?) -> Unit) {
        Thread {
            val apkFile = File(context.getExternalFilesDir("updates"), "keybroad-${updateInfo.version}.apk")
            apkFile.parentFile?.mkdirs()
            // Delete stale file so a partial download cannot be reused
            if (apkFile.exists()) apkFile.delete()
            var connection: HttpURLConnection? = null
            try {
                val url = URL(updateInfo.apkUrl)
                connection = url.openConnection() as HttpURLConnection
                connection.requestMethod = "GET"
                connection.connectTimeout = 15000
                connection.readTimeout = 60000
                if (connection.responseCode == HttpURLConnection.HTTP_OK) {
                    connection.inputStream.use { input ->
                        apkFile.outputStream().use { output ->
                            input.copyTo(output)
                        }
                    }
                    onComplete(if (apkFile.exists() && apkFile.length() > 0) apkFile else null)
                } else {
                    onComplete(null)
                }
            } catch (e: Exception) {
                e.printStackTrace()
                if (apkFile.exists()) apkFile.delete()
                onComplete(null)
            } finally {
                connection?.disconnect()
            }
        }.start()
    }

    /**
      * Install the APK via the PackageInstaller Session API.
      * Result is delivered to UpdateInstallReceiver as a broadcast
      * (ACTION_INSTALL_STATUS with EXTRA_INSTALL_STATUS).
      */
    fun installApk(apkFile: File) {
        val packageInstaller = context.packageManager.packageInstaller
        val params = android.content.pm.PackageInstaller.SessionParams(
            android.content.pm.PackageInstaller.SessionParams.MODE_FULL_INSTALL
        ).apply {
            // Must match applicationId exactly; without this PackageInstaller
            // may reject the session or UpdateChecker's pre-check may mis-fire.
            setAppPackageName(context.packageName) // "com.keybroad"
        }

        val sessionId = packageInstaller.createSession(params)
        val session = packageInstaller.openSession(sessionId)

        try {
            session.openWrite("keybroad-update", 0, -1).use { output ->
                FileInputStream(apkFile).use { input ->
                    input.copyTo(output)
                }
            }
        } catch (e: Exception) {
            e.printStackTrace()
            session.abandon()
            return
        }

        // PendingIntent target: broadcast receiver that surfaces install result
        val intent = Intent(context, UpdateInstallReceiver::class.java).apply {
            action = ACTION_INSTALL_STATUS
        }
        val statusIntent = PendingIntent.getBroadcast(
            context,
            sessionId,
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE
        )

        session.commit(statusIntent.intentSender)
        session.close()

        // Track for the receiver / UI
        prefs.edit().apply {
            putInt("active_session", sessionId)
            apply()
        }
    }

    fun isUpdateDownloaded(version: String): File? {
        val file = File(context.getExternalFilesDir("updates"), "keybroad-$version.apk")
        return if (file.exists()) file else null
    }

    /**
     * Uninstall the app (used only for the one-time signature bootstrap when
     * INSTALL_FAILED_UPDATE_INCOMPATIBLE is detected).
     */
    fun uninstallForBootstrap() {
        try {
            val intent = Intent(Intent.ACTION_DELETE).apply {
                data = android.net.Uri.parse("package:${context.packageName}")
                flags = Intent.FLAG_ACTIVITY_NEW_TASK
            }
            context.startActivity(intent)
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }
}
