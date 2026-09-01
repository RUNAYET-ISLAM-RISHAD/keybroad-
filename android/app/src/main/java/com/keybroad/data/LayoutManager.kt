package com.keybroad.data

import android.content.Context
import android.util.Log
import org.json.JSONArray
import org.json.JSONObject
import java.io.IOException

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

    private val layouts = mutableMapOf<String, LayoutData>()

    // Base Bengali visual layout (identical across all layouts)
    // Rows as per spec: Vowels, Ka-varga, etc.
    fun getBaseVisualLayout(): LayoutData {
        val baseKeys = listOf(
            // Row1: Vowels অ আ ই ঈ উ ঊ এ ঐ ও ঔ (logical q-p)
            KeyData("q", "অ", "অ", "অ"), KeyData("w", "আ", "আ", "আ"), KeyData("e", "ই", "ই", "ই"), KeyData("r", "ঈ", "ঈ", "ঈ"), KeyData("t", "উ", "উ", "উ"),
            KeyData("y", "ঊ", "ঊ", "ঊ"), KeyData("u", "এ", "এ", "এ"), KeyData("i", "ঐ", "ঐ", "ঐ"), KeyData("o", "ও", "ও", "ও"), KeyData("p", "ঔ", "ঔ", "ঔ"),
            // Row2: ক খ গ ঘ ঙ চ ছ জ ঝ ঞ (logical a-;)
            KeyData("a", "ক", "ক", "ক"), KeyData("s", "খ", "খ", "খ"), KeyData("d", "গ", "গ", "গ"), KeyData("f", "ঘ", "ঘ", "ঘ"), KeyData("g", "ঙ", "ঙ", "ঙ"),
            KeyData("h", "চ", "চ", "চ"), KeyData("j", "ছ", "ছ", "ছ"), KeyData("k", "জ", "জ", "জ"), KeyData("l", "ঝ", "ঝ", "ঝ"), KeyData(";", "ঞ", "ঞ", "ঞ"),
            // Row3: ট ঠ ড ঢ ণ ত থ দ ধ ন (logical z-/)
            KeyData("z", "ট", "ট", "ট"), KeyData("x", "ঠ", "ঠ", "ঠ"), KeyData("c", "ড", "ড", "ড"), KeyData("v", "ঢ", "ঢ", "ঢ"), KeyData("b", "ণ", "ণ", "ণ"),
            KeyData("n", "ত", "ত", "ত"), KeyData("m", "থ", "থ", "থ"), KeyData(",", "দ", "দ", "দ"), KeyData(".", "ধ", "ধ", "ধ"), KeyData("/", "ন", "ন", "ন"),
            // Row4: প ফ ব ভ ম য র ল শ ষ (logical Q-P)
            KeyData("Q", "প", "প", "প"), KeyData("W", "ফ", "ফ", "ফ"), KeyData("E", "ব", "ব", "ব"), KeyData("R", "ভ", "ভ", "ভ"), KeyData("T", "ম", "ম", "ম"),
            KeyData("Y", "য", "য", "য"), KeyData("U", "র", "র", "র"), KeyData("I", "ল", "ল", "ল"), KeyData("O", "শ", "শ", "শ"), KeyData("P", "ষ", "ষ", "ষ"),
            // Row5: স হ ড় ঢ় য় ক্ষ জ্ঞ ৎ ং/ঁ (logical A-L)
            KeyData("A", "স", "স", "স"), KeyData("S", "হ", "হ", "হ"), KeyData("D", "ড়", "ড়", "ড়"), KeyData("F", "ঢ়", "ঢ়", "ঢ়"), KeyData("G", "য়", "য়", "য়"),
            KeyData("H", "ক্ষ", "ক্ষ", "ক্ষ"), KeyData("J", "জ্ঞ", "জ্ঞ", "জ্ঞ"), KeyData("K", "ৎ", "ৎ", "ৎ"), KeyData("L", "ং", "ঁ", "ং")
        )
        return LayoutData("base", "Base Bengali", baseKeys)
    }

    fun loadLayout(layoutName: String): LayoutData {
        layouts[layoutName.lowercase()]?.let { return it }

        val fileName = "${layoutName.lowercase()}.json"
        val json = try {
            context.assets.open(fileName).bufferedReader().use { it.readText() }
        } catch (e: IOException) {
            Log.e("LayoutManager", "Failed to load layout: $fileName", e)
            return getDefaultLayout()
        }

        val layoutData = parseLayout(json, layoutName)
        layouts[layoutName.lowercase()] = layoutData
        Log.d("LayoutManager", "Loaded layout '$layoutName' with ${layoutData.keys.size} keys")
        return layoutData
    }

    private fun parseLayout(json: String, defaultName: String): LayoutData {
        val trimmed = json.trim()
        if (trimmed.startsWith("[")) {
            return parseArrayLayout(trimmed, defaultName)
        }
        return parseObjectLayout(trimmed, defaultName)
    }

    private fun parseArrayLayout(json: String, defaultName: String): LayoutData {
        val arr = JSONArray(json)
        val keys = mutableListOf<KeyData>()
        for (i in 0 until arr.length()) {
            val obj = arr.getJSONObject(i)
            val key = obj.getString("key")
            val output = obj.getString("output")
            val shiftOutput = obj.optString("shift_output", output)
            keys.add(KeyData(
                key = key,
                output = output,
                shiftOutput = shiftOutput,
                display = output
            ))
        }
        return LayoutData(defaultName.lowercase(), defaultName, keys)
    }

    private fun parseObjectLayout(json: String, defaultName: String): LayoutData {
        val jsonObj = JSONObject(json)
        val layoutId = jsonObj.optString("layout_id", defaultName.lowercase())
        val name = jsonObj.optString("name", defaultName)
        val keysObj = jsonObj.optJSONObject("keys") ?: JSONObject()
        val keys = mutableListOf<KeyData>()
        keysObj.keys().forEach { key ->
            val keyObj = keysObj.getJSONObject(key)
            val output = keyObj.optString("output", key)
            val shiftOutput = keyObj.optString("shift_output", output)
            keys.add(KeyData(
                key = key,
                output = output,
                shiftOutput = shiftOutput,
                display = output
            ))
        }
        return LayoutData(layoutId, name, keys)
    }

    private fun getDefaultLayout(): LayoutData {
        return getBaseVisualLayout()
    }

    companion object {
        private var instance: LayoutManager? = null
        fun getInstance(context: Context): LayoutManager {
            return instance ?: LayoutManager(context.applicationContext).also { instance = it }
        }
    }
}
