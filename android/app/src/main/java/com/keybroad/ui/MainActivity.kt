package com.keybroad.ui

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
}
