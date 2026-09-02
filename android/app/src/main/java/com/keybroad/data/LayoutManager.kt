package com.keybroad.data

import android.content.Context

data class KeyData(
    val key: String, // Logical ID (QWERTY)
    val output: String,
    val shiftOutput: String,
    val display: String
)

data class LayoutData(
    val id: String,
    val name: String,
    val keys: List<KeyData>
)

class LayoutManager(private val context: Context) {

    // Gboard-style: always Roman QWERTY a-z (UI shows English letters, engine transliterates)
    fun getRomanKeys(): List<KeyData> {
        val keys = mutableListOf<KeyData>()
        for (c in 'a'..'z') {
            val s = c.toString()
            keys.add(KeyData(s, s, s.uppercase(), s))
        }
        // Digits 0-9
        for (c in '0'..'9') {
            val s = c.toString()
            keys.add(KeyData(s, s, s, s))
        }
        keys.add(KeyData("space", " ", " ", "space"))
        keys.add(KeyData(",", ",", ",", ","))
        keys.add(KeyData(".", ".", ".", "."))
        return keys
    }

    // Kept for compatibility — delegates to Roman keys
    fun loadLayout(layoutName: String): LayoutData {
        return LayoutData(layoutName.lowercase(), layoutName, getRomanKeys())
    }

    fun getBaseVisualLayout(): LayoutData {
        return LayoutData("roman", "Roman", getRomanKeys())
    }

    companion object {
        private var instance: LayoutManager? = null
        fun getInstance(context: Context): LayoutManager {
            return instance ?: LayoutManager(context.applicationContext).also { instance = it }
        }
    }
}
