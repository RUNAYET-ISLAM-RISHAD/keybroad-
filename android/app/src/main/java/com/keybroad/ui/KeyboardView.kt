package com.keybroad.ui

import android.util.Log
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.keybroad.data.KeyData

@Composable
fun KeyboardView(
    keys: List<KeyData>,
    onKeyPress: (keyData: KeyData, isShift: Boolean, isCaps: Boolean) -> Unit,
    onSpecialKey: (keyCode: Int) -> Unit
) {
    if (keys.isEmpty()) {
        Text("Loading keyboard...")
        return
    }

    // Filter out "space" pseudo-key; space is handled by special row (keyCode 32)
    val filtered = keys.filter { it.key != "space" }
    Log.d("KeyboardView", "Displaying ${filtered.size}/${keys.size} keys in JSON order")

    // Split preserving JSON order into QWERTY-like rows
    // Phonetic (26 keys): 10 + 9 + 7  => qwerty..., asdf..., zxcv...
    // Jatiya (36 keys): 10 + 10 + 10 + 6 => digits + rows preserving array order
    val rows: List<List<KeyData>> = when (filtered.size) {
        26 -> listOf(
            filtered.subList(0, 10),
            filtered.subList(10, 19),
            filtered.subList(19, 26)
        )
        36 -> listOf(
            filtered.subList(0, 10),
            filtered.subList(10, 20),
            filtered.subList(20, 30),
            filtered.subList(30, 36)
        )
        else -> filtered.chunked(10)
    }

    Column {
        rows.forEach { row ->
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(2.dp)
            ) {
                row.forEach { keyData ->
                    KeyButton(
                        keyData = keyData,
                        modifier = Modifier.weight(1f),
                        onKeyPress = onKeyPress
                    )
                }
            }
            Spacer(modifier = Modifier.height(4.dp))
        }

        // Special keys row
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(2.dp)
        ) {
            SpecialKeyButton(
                label = "Space",
                keyCode = 32,
                modifier = Modifier.weight(4f),
                onSpecialKey = onSpecialKey
            )
            SpecialKeyButton(
                label = "Back",
                keyCode = 8,
                modifier = Modifier.weight(1f),
                onSpecialKey = onSpecialKey
            )
            SpecialKeyButton(
                label = "Enter",
                keyCode = 13,
                modifier = Modifier.weight(1f),
                onSpecialKey = onSpecialKey
            )
        }
    }
}

@Composable
fun KeyButton(
    keyData: KeyData,
    modifier: Modifier = Modifier,
    onKeyPress: (keyData: KeyData, isShift: Boolean, isCaps: Boolean) -> Unit
) {
    Button(
        onClick = { onKeyPress(keyData, false, false) },
        modifier = modifier.height(52.dp),
        colors = ButtonDefaults.buttonColors(
            containerColor = Color(0xFF6750A4),
            contentColor = Color.White
        ),
        contentPadding = PaddingValues(horizontal = 2.dp, vertical = 4.dp)
    ) {
        Text(
            text = keyData.output,
            fontSize = 20.sp,
            fontWeight = FontWeight.Medium,
            color = Color.White
        )
    }
}

@Composable
fun SpecialKeyButton(
    label: String,
    keyCode: Int,
    modifier: Modifier = Modifier,
    onSpecialKey: (keyCode: Int) -> Unit
) {
    Button(
        onClick = { onSpecialKey(keyCode) },
        modifier = modifier.height(52.dp),
        colors = ButtonDefaults.buttonColors(
            containerColor = Color(0xFF625B71),
            contentColor = Color.White
        )
    ) {
        Text(
            text = label,
            fontSize = 14.sp,
            color = Color.White
        )
    }
}
