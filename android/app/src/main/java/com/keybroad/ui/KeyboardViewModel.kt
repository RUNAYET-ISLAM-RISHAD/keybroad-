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
    val keys: List<KeyData> = emptyList(),
    val isEnglishMode: Boolean = false, // false = Bangla Avro, true = English
    val isShift: Boolean = false
)

class KeyboardViewModel(application: Application) : AndroidViewModel(application) {
    private val engine = KeyboardEngine()
    private val layoutManager = LayoutManager.getInstance(application)
    private val _state = MutableStateFlow(KeyboardState())
    val state: StateFlow<KeyboardState> = _state

    // Small English word list for English-mode suggestions
    private val englishWords = listOf(
        "hello", "help", "held", "helium", "helm",
        "hero", "her", "here", "hey", "he",
        "how", "house", "home", "hope", "happy",
        "world", "word", "work", "well", "we",
        "you", "your", "yes", "yet",
        "the", "this", "that", "there", "they"
    )

    init {
        // Gboard-style: always Roman QWERTY keys, engine starts in Bangla Avro (Phonetic)
        val keys = layoutManager.getRomanKeys()
        _state.value = KeyboardState(keys = keys, isEnglishMode = false)
        engine.switchLayout("Phonetic")
        refreshSuggestions()
    }

    private fun refreshSuggestions() {
        val suggestions = if (_state.value.isEnglishMode) {
            val lastWord = _state.value.text.split(" ", "\n").lastOrNull()?.lowercase() ?: ""
            if (lastWord.length >= 2) {
                englishWords.filter { it.startsWith(lastWord) }.take(3)
            } else if (lastWord.length == 1) {
                englishWords.filter { it.startsWith(lastWord) }.take(3)
            } else {
                emptyList()
            }
        } else {
            // Bangla Avro: Bengali suggestions from engine (current_word via JNI)
            engine.getSuggestions().toList()
        }
        // Also include join suggestions in Bangla if needed (kept for compatibility)
        _state.value = _state.value.copy(suggestions = suggestions)
    }

    fun toggleLanguage() {
        viewModelScope.launch {
            val newMode = !_state.value.isEnglishMode
            engine.switchLayout(if (newMode) "English" else "Phonetic")
            _state.value = _state.value.copy(isEnglishMode = newMode, isShift = false)
            refreshSuggestions()
        }
    }

    fun processKey(keyData: KeyData, isShift: Boolean = false, isCaps: Boolean = false) {
        viewModelScope.launch {
            val effectiveShift = isShift || _state.value.isShift
            val keyCode = if (keyData.key == "space") 32 else keyData.key[0].code
            // Engine handles both layouts: Phonetic (Avro -> Bengali) and English (direct)
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
            // Backspace (67), Enter (66), Space (32)
            val newText = engine.processKey(keyCode, false, false)
            _state.value = _state.value.copy(text = newText)
            refreshSuggestions()
        }
    }

    fun selectSuggestion(suggestion: String) {
        viewModelScope.launch {
            // Let engine replace current partial word with full suggestion (Bengali)
            // For English mode we also go through engine so state stays consistent
            val newText = engine.applySuggestion(suggestion)
            _state.value = _state.value.copy(text = newText)
            refreshSuggestions()
        }
    }

    override fun onCleared() {
        engine.destroy()
        super.onCleared()
    }
}
