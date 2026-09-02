package com.keybroad.bridge

data class KeyMapping(
    val key: String,
    val output: String,
    val shift: String,
    val display: String
)

class KeyboardEngine : AutoCloseable {
    private var nativePtr: Long = 0

    init {
        System.loadLibrary("keybroad_core")
        nativePtr = nativeInit()
        android.util.Log.d("KeyboardEngine", "nativeInit → ptr=$nativePtr")
    }

    fun processKey(keyCode: Int, isShift: Boolean = false, isCaps: Boolean = false): String {
        if (nativePtr == 0L) {
            android.util.Log.e("KeyboardEngine", "processKey called with null ptr!")
            return ""
        }
        return try {
            val result = nativeProcessKey(nativePtr, keyCode, isShift, isCaps)
            android.util.Log.d("KeyboardEngine", "Key=$keyCode, Shift=$isShift, Caps=$isCaps → Result='$result'")
            result
        } catch (e: UnsatisfiedLinkError) {
            android.util.Log.e("KeyboardEngine", "JNI UnsatisfiedLinkError: ${e.message}")
            ""
        } catch (e: Exception) {
            android.util.Log.e("KeyboardEngine", "JNI Error: ${e.message}")
            ""
        }
    }

    fun processChar(unicode: Int): String {
        if (nativePtr == 0L) return ""
        return try {
            val result = nativeProcessChar(nativePtr, unicode)
            android.util.Log.d("KeyboardEngine", "processChar unicode=$unicode ('${unicode.toChar()}') → Result='$result'")
            result
        } catch (e: UnsatisfiedLinkError) {
            android.util.Log.e("KeyboardEngine", "processChar UnsatisfiedLinkError: ${e.message}")
            ""
        } catch (e: Exception) {
            android.util.Log.e("KeyboardEngine", "processChar Error: ${e.message}")
            ""
        }
    }

    fun applySuggestion(suggestion: String): String {
        if (nativePtr == 0L) return suggestion + " "
        return try {
            val result = nativeApplySuggestion(nativePtr, suggestion)
            android.util.Log.d("KeyboardEngine", "applySuggestion '$suggestion' → Result='$result'")
            result
        } catch (e: UnsatisfiedLinkError) {
            android.util.Log.e("KeyboardEngine", "applySuggestion UnsatisfiedLinkError: ${e.message}")
            suggestion + " "
        } catch (e: Exception) {
            android.util.Log.e("KeyboardEngine", "applySuggestion Error: ${e.message}")
            suggestion + " "
        }
    }

    fun getSuggestions(): Array<String> {
        if (nativePtr == 0L) return emptyArray()
        return try {
            nativeGetSuggestions(nativePtr)
        } catch (e: UnsatisfiedLinkError) {
            emptyArray()
        } catch (e: Exception) {
            emptyArray()
        }
    }

    fun isJoinMode(): Boolean {
        if (nativePtr == 0L) return false
        return try {
            nativeIsJoinMode(nativePtr)
        } catch (e: UnsatisfiedLinkError) {
            false
        } catch (e: Exception) {
            false
        }
    }

    fun getJoinSuggestions(): Array<String> {
        if (nativePtr == 0L) return emptyArray()
        return try {
            nativeGetJoinSuggestions(nativePtr)
        } catch (e: UnsatisfiedLinkError) {
            emptyArray()
        } catch (e: Exception) {
            emptyArray()
        }
    }

    fun switchLayout(layoutName: String) {
        if (nativePtr == 0L) return
        try {
            nativeSwitchLayout(nativePtr, layoutName)
        } catch (e: UnsatisfiedLinkError) {
            // Function not available
        } catch (e: Exception) {
            // Ignore
        }
    }

    fun destroy() {
        if (nativePtr == 0L) return
        try {
            nativeDestroy(nativePtr)
        } catch (e: Exception) {
            // Ignore
        }
        nativePtr = 0
    }

    override fun close() {
        destroy()
    }

    private external fun nativeInit(): Long
    private external fun nativeProcessKey(ptr: Long, keyCode: Int, isShift: Boolean, isCaps: Boolean): String
    private external fun nativeProcessChar(ptr: Long, unicode: Int): String
    private external fun nativeApplySuggestion(ptr: Long, suggestion: String): String
    private external fun nativeGetSuggestions(ptr: Long): Array<String>
    private external fun nativeIsJoinMode(ptr: Long): Boolean
    private external fun nativeGetJoinSuggestions(ptr: Long): Array<String>
    private external fun nativeSwitchLayout(ptr: Long, layoutName: String)
    private external fun nativeDestroy(ptr: Long)
}
