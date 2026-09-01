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
    val isShift: Boolean = false,
    val isJoinMode: Boolean = false,
    val showKarPopup: Boolean = false
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
        val isBengali = layoutName != "Phonetic" && layoutName != "English"
        _state.value = _state.value.copy(
            currentLayout = layoutName,
            keys = layoutData.keys,
            isShift = false,
            isJoinMode = false
        )
        refreshSuggestions()
    }

    private fun refreshSuggestions() {
        // Sync join mode with engine state
        val isJoinMode = engine.isJoinMode()
        val suggestions = if (isJoinMode) {
            engine.getJoinSuggestions().toList()
        } else {
            engine.getSuggestions().toList()
        }
        _state.value = _state.value.copy(
            suggestions = suggestions,
            isJoinMode = isJoinMode
        )
    }

    fun processKey(keyData: KeyData, isShift: Boolean = false, isCaps: Boolean = false) {
        viewModelScope.launch {
            val effectiveShift = isShift || _state.value.isShift
            val keyCode = if (keyData.key == "space") 32 else keyData.key[0].code
            val charToSend = if (effectiveShift && keyData.shiftOutput.isNotEmpty()) {
                keyData.shiftOutput[0]
            } else if (keyData.output.isNotEmpty()) {
                keyData.output[0]
            } else {
                keyData.key[0]
            }
            val unicode = charToSend.code
            val newText = engine.processKey(keyCode, effectiveShift, isCaps)
            _state.value = _state.value.copy(
                text = newText,
                isShift = if (effectiveShift && !isCaps) false else _state.value.isShift
            )
            refreshSuggestions()
        }
    }

    fun processSpecialKey(keyCode: Int) {
        viewModelScope.launch {
            if (keyCode == 59 || keyCode == 60) {
                _state.value = _state.value.copy(isShift = !_state.value.isShift)
                engine.processKey(keyCode, false, false)
                refreshSuggestions()
                return@launch
            }
            if (keyCode == 100) {
                // যুক্ত (join) key
                engine.processKey(keyCode, false, false)
                refreshSuggestions()
                return@launch
            }
            if (keyCode == 101) {
                // কার key - toggle popup
                _state.value = _state.value.copy(showKarPopup = !_state.value.showKarPopup)
                return@launch
            }
            // Backspace (67), Enter (66), Space (32)
            val newText = engine.processKey(keyCode, false, false)
            refreshSuggestions()
            _state.value = _state.value.copy(
                text = newText,
                showKarPopup = false
            )
        }
    }

    fun selectKar(kar: String) {
        viewModelScope.launch {
            // Kar characters are Bengali vowel signs; pass their unicode directly.
            // The engine's fixed-layout lookup and smart kar replacement handle placement.
            val newText = engine.processKey(kar[0].code, false, false)
            _state.value = _state.value.copy(
                text = newText,
                showKarPopup = false
            )
            refreshSuggestions()
        }
    }

    fun dismissKarPopup() {
        _state.value = _state.value.copy(showKarPopup = false)
    }

    fun switchLayout(layoutName: String) {
        viewModelScope.launch {
            engine.switchLayout(layoutName)
            loadLayout(layoutName)
        }
    }

    fun selectSuggestion(suggestion: String) {
        viewModelScope.launch {
            if (_state.value.isJoinMode && suggestion.length >= 3) {
                // Join mode: suggestion is a conjunct like "ক্ষ" (ক + ্ + ষ).
                // Send the final consonant to complete the conjunct via the engine.
                val lastChar = suggestion.last()
                val newText = engine.processKey(lastChar.code, false, false)
                _state.value = _state.value.copy(text = newText)
                refreshSuggestions()
            } else {
                // Normal mode: append suggestion word with space
                val newText = _state.value.text + suggestion + " "
                _state.value = _state.value.copy(
                    text = newText,
                    suggestions = emptyList()
                )
            }
        }
    }

    override fun onCleared() {
        engine.destroy()
        super.onCleared()
    }
}