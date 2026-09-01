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
    val keys: List<KeyData> = emptyList(),
    val isShift: Boolean = false
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
            keys = layoutData.keys,
            isShift = false
        )
    }

    fun processKey(keyData: KeyData, isShift: Boolean = false, isCaps: Boolean = false) {
        viewModelScope.launch {
            // For phonetic, keyData.output is Roman "a", for fixed layouts it's Bengali "ক"
            // Engine expects Roman for phonetic, and stable QWERTY ID lookup for others
            // For long-press, isShift indicates secondary character
            val effectiveShift = isShift || _state.value.isShift
            // Determine keyCode: use stable QWERTY ID's first char
            val keyCode = if (keyData.key == "space") 32 else keyData.key[0].code

            // Special handling for phonetic: if effectiveShift, we need to send uppercase Roman?
            // For phonetic, shift may produce capital Roman that maps to retroflex, but our phonetic engine is case-sensitive
            // We'll send the character that corresponds to output: if effectiveShift, use shiftOutput's first char
            val charToSend = if (effectiveShift && keyData.shiftOutput.isNotEmpty()) {
                keyData.shiftOutput[0]
            } else if (keyData.output.isNotEmpty()) {
                keyData.output[0]
            } else {
                keyData.key[0]
            }
            val unicode = charToSend.code

            // For phonetic, engine will transliterate Roman char
            // For fixed, engine will lookup via unicode
            val newText = engine.processKey(keyCode, effectiveShift, isCaps)

            // Also, for grapheme-aware and smart kar, engine handles correctly

            val suggestions = engine.getSuggestions().toList()
            _state.value = _state.value.copy(
                text = newText,
                suggestions = suggestions,
                // Reset shift after single use if not caps
                isShift = if (effectiveShift && !isCaps) false else _state.value.isShift
            )
        }
    }

    fun processSpecialKey(keyCode: Int) {
        viewModelScope.launch {
            // Handle shift toggle
            if (keyCode == 59 || keyCode == 60) {
                _state.value = _state.value.copy(isShift = !_state.value.isShift)
                // Also notify engine
                engine.processKey(keyCode, false, false)
                return@launch
            }
            // Backspace (67), Enter (66), Space (32)
            // Backspace is grapheme-aware in engine
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
            // Append suggestion with space
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
