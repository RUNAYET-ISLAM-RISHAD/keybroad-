package com.keybroad.data

import android.content.Context
import android.util.Log
import org.json.JSONArray
import org.json.JSONObject
import java.io.IOException

data class KeyData(
    val key: String,
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

        // Handle JSON array format: [{"key":"q","output":"ধ"}, ...]
        if (trimmed.startsWith("[")) {
            return parseArrayLayout(trimmed, defaultName)
        }

        // Handle JSON object format (legacy): {"layout_id":"phonetic","keys":{...}}
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
        return LayoutData("phonetic", "Phonetic", emptyList())
    }

    companion object {
        private var instance: LayoutManager? = null

        fun getInstance(context: Context): LayoutManager {
            return instance ?: LayoutManager(context.applicationContext).also { instance = it }
        }
    }
}
