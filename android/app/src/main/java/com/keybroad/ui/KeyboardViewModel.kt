package com.keybroad.ui

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.keybroad.bridge.KeyboardEngine
import com.keybroad.data.LayoutManager
import com.keybroad.data.KeyData
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

data class KeyboardState(
    val text: String = "",
    val suggestions: List<String> = emptyList(),
    val currentLayout: String = "Phonetic",
    val keys: List<KeyData> = emptyList()
)

class KeyboardViewModel(application: Application) : AndroidViewModel(application) {
    private val engine = KeyboardEngine()
    private val layoutManager = LayoutManager.getInstance(application)
    private val _state = MutableStateFlow(KeyboardState())
    val state: StateFlow<KeyboardState> = _state

    init {
        loadLayout("Phonetic")
    }

    private fun loadLayout(layoutName: String) {
        val layoutData = layoutManager.loadLayout(layoutName)
        _state.value = _state.value.copy(
            currentLayout = layoutName,
            keys = layoutData.keys
        )
    }

    fun processKey(keyData: KeyData, isShift: Boolean = false, isCaps: Boolean = false) {
        viewModelScope.launch {
            // key field is source key (e.g. "q", "space"), output is Bengali display
            // LayoutManager.kt reads output for display and key for keycode - verified
            val keyCode = if (keyData.key == "space") 32 else keyData.key[0].code
            val newText = engine.processKey(keyCode, isShift, isCaps)
            val suggestions = engine.getSuggestions().toList()

            _state.value = _state.value.copy(
                text = newText,
                suggestions = suggestions
            )
        }
    }

    fun processSpecialKey(keyCode: Int) {
        viewModelScope.launch {
            val newText = engine.processKey(keyCode, false, false)
            val suggestions = engine.getSuggestions().toList()
            _state.value = _state.value.copy(
                text = newText,
                suggestions = suggestions
            )
        }
    }

    fun switchLayout(layoutName: String) {
        viewModelScope.launch {
            engine.switchLayout(layoutName)
            loadLayout(layoutName)
        }
    }

    fun selectSuggestion(suggestion: String) {
        viewModelScope.launch {
            val newText = _state.value.text + suggestion + " "
            _state.value = _state.value.copy(
                text = newText,
                suggestions = emptyList()
            )
        }
    }

    override fun onCleared() {
        engine.destroy()
        super.onCleared()
    }
}
