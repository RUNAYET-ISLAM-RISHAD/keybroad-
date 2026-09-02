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
            // CRITICAL: Send LOGICAL KEY ID (Roman Q/W/E/R), NOT Bengali unicode.
            // Engine maps logical ID via active layout profile.
            val keyCode = if (keyData.key == "space") 32 else keyData.key[0].code
            val newText = engine.processKey(keyCode, effectiveShift, isCaps)
            // Engine is single source of truth — set full text, don't append
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
            if (keyCode == 1000) {
                // যুক্ত (join) key — use 1000 to avoid collision with 'd' (100)
                val newText = engine.processKey(keyCode, false, false)
                _state.value = _state.value.copy(text = newText)
                refreshSuggestions()
                return@launch
            }
            if (keyCode == 1001) {
                // কার key — use 1001 to avoid collision with 'e' (101); toggle popup
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
            // Direct Bengali char via processChar — bypasses layout lookup,
            // handled by smart kar system in engine.
            val newText = engine.processChar(kar[0].code)
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
            if (_state.value.isJoinMode) {
                // Join mode: suggestion is a conjunct like "ক্ষ" (ক + ্ + ষ).
                // Send the final consonant via processChar to complete the conjunct.
                val lastChar = suggestion.last()
                val newText = engine.processChar(lastChar.code)
                _state.value = _state.value.copy(text = newText)
                refreshSuggestions()
            } else {
                // Normal mode: let engine replace current partial word with full suggestion.
                // Engine handles buffer update; UI displays engine's full text.
                val newText = engine.applySuggestion(suggestion)
                _state.value = _state.value.copy(text = newText)
                refreshSuggestions()
            }
        }
    }

    override fun onCleared() {
        engine.destroy()
        super.onCleared()
    }
}