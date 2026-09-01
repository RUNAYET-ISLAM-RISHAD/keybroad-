package com.keybroad.ui

import android.app.AlertDialog
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.keybroad.update.UpdateChecker
import com.keybroad.update.UpdateManager
import com.keybroad.ui.theme.KeybroadTheme

class MainActivity : ComponentActivity() {

    private lateinit var updateChecker: UpdateChecker
    private lateinit var updateManager: UpdateManager

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Initialize update manager and checker
        updateManager = UpdateManager(this)
        updateChecker = UpdateChecker(this, updateManager)

        // Check for updates on launch (in background)
        updateChecker.checkForUpdate()

        // Handle signature-mismatch bootstrap request (from UpdateInstallReceiver)
        if (intent.getBooleanExtra("bootstrap_required", false)) {
            showBootstrapDialog(intent.getStringExtra("bootstrap_details") ?: "")
        }

        setContent {
            KeybroadTheme {
                val viewModel: KeyboardViewModel = viewModel()
                val state by viewModel.state.collectAsState()

                Surface(modifier = Modifier.fillMaxSize()) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        TextField(
                            value = state.text,
                            onValueChange = {},
                            modifier = Modifier.fillMaxWidth(),
                            readOnly = true,
                            textStyle = LocalTextStyle.current.copy(fontSize = 20.sp)
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            state.suggestions.forEach { suggestion ->
                                SuggestionChip(
                                    onClick = { viewModel.selectSuggestion(suggestion) },
                                    label = { Text(suggestion) }
                                )
                            }
                        }
                        Spacer(modifier = Modifier.height(16.dp))
                        LayoutSwitcher(
                            currentLayout = state.currentLayout,
                            onLayoutSelected = { viewModel.switchLayout(it) }
                        )
                        Spacer(modifier = Modifier.height(16.dp))
                        KeyboardView(
                            keys = state.keys,
                            isShift = state.isShift,
                            onKeyPress = { keyData, isShift, isCaps ->
                                viewModel.processKey(keyData, isShift, isCaps)
                            },
                            onSpecialKey = { keyCode ->
                                viewModel.processSpecialKey(keyCode)
                            },
                            suggestions = state.suggestions,
                            isJoinMode = state.isJoinMode,
                            showKarPopup = state.showKarPopup,
                            onSelectKar = { kar ->
                                viewModel.selectKar(kar)
                            },
                            onSelectSuggestion = { suggestion ->
                                viewModel.selectSuggestion(suggestion)
                            }
                        )
                    }
                }
            }
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        updateChecker.onDestroy()
    }

    /**
     * One-time bootstrap dialog: the installed build has a legacy signature.
     * Uninstall + reinstall the stable-key APK; after that all OTA updates
     * install automatically without signature conflicts.
     */
    private fun showBootstrapDialog(details: String) {
        AlertDialog.Builder(this)
            .setTitle("One-Time Update Step")
            .setMessage(
                "Your installed version uses an old signing key that cannot be " +
                "updated directly.\n\n" +
                "1. Uninstall the current app\n" +
                "2. Download and install the new version from:\n" +
                "https://github.com/RUNAYET-ISLAM-RISHAD/keybroad-/releases/latest\n\n" +
                "After this one-time step, all future updates will install " +
                "automatically.\n\n($details)"
            )
            .setPositiveButton("Open Download Page") { _, _ ->
                updateManager.uninstallForBootstrap()
            }
            .setNegativeButton("Later") { d, _ -> d.dismiss() }
            .setCancelable(false)
            .show()
    }
}
