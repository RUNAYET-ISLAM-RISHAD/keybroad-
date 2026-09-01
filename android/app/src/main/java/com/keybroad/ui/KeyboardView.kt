package com.keybroad.ui

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.keybroad.data.KeyData

private val QWERTY_ORDER = listOf(
    "q","w","e","r","t","y","u","i","o","p",
    "a","s","d","f","g","h","j","k","l",
    "z","x","c","v","b","n","m"
)

@Composable
fun KeyboardView(
    keys: List<KeyData>,
    onKeyPress: (keyData: KeyData, isShift: Boolean, isCaps: Boolean) -> Unit,
    onSpecialKey: (keyCode: Int) -> Unit,
    isShift: Boolean = false
) {
    if (keys.isEmpty()) {
        Text("Loading keyboard...")
        return
    }

    val filtered = keys.filter { it.key != "space" && it.key != " " }
    // Sort into QWERTY order to keep position identical across layouts
    val sorted = filtered.sortedBy { QWERTY_ORDER.indexOf(it.key.lowercase()).let { idx -> if (idx == -1) 100 else idx } }

    // Split into QWERTY rows: 10 + 9 + 7, or fallback
    val rows = when {
        sorted.size >= 26 -> listOf(
            sorted.filter { it.key.lowercase() in QWERTY_ORDER.subList(0,10) },
            sorted.filter { it.key.lowercase() in QWERTY_ORDER.subList(10,19) },
            sorted.filter { it.key.lowercase() in QWERTY_ORDER.subList(19,26) }
        ).filter { it.isNotEmpty() }
        else -> sorted.chunked(10)
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
                        isShift = isShift,
                        modifier = Modifier.weight(1f),
                        onKeyPress = onKeyPress
                    )
                }
            }
            Spacer(modifier = Modifier.height(4.dp))
        }

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(2.dp)
        ) {
            ShiftButton(
                isShift = isShift,
                modifier = Modifier.weight(1f),
                onShift = { onSpecialKey(59) }
            )
            SpecialKeyButton(
                label = "Space",
                keyCode = 32,
                modifier = Modifier.weight(4f),
                onSpecialKey = onSpecialKey
            )
            SpecialKeyButton(
                label = "Back",
                keyCode = 67,
                modifier = Modifier.weight(1f),
                onSpecialKey = onSpecialKey
            )
            SpecialKeyButton(
                label = "Enter",
                keyCode = 66,
                modifier = Modifier.weight(1f),
                onSpecialKey = onSpecialKey
            )
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun KeyButton(
    keyData: KeyData,
    isShift: Boolean,
    modifier: Modifier = Modifier,
    onKeyPress: (keyData: KeyData, isShift: Boolean, isCaps: Boolean) -> Unit
) {
    var showPopup by remember { mutableStateOf(false) }
    val display = if (isShift) keyData.shiftOutput else keyData.output
    val hint = if (isShift) keyData.output else keyData.shiftOutput

    Box(modifier = modifier.height(52.dp), contentAlignment = Alignment.Center) {
        Button(
            onClick = {},
            modifier = Modifier
                .fillMaxWidth()
                .combinedClickable(
                    onClick = { onKeyPress(keyData, isShift, false) },
                    onLongClick = {
                        // Long-press: show popup and send secondary
                        showPopup = true
                        onKeyPress(keyData, !isShift, false)
                    }
                ),
            colors = ButtonDefaults.buttonColors(
                containerColor = Color(0xFF6750A4),
                contentColor = Color.White
            ),
            contentPadding = PaddingValues(horizontal = 2.dp, vertical = 4.dp)
        ) {
            Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
                Text(
                    text = display,
                    fontSize = 20.sp,
                    fontWeight = FontWeight.Medium,
                    color = Color.White
                )
                if (hint.isNotEmpty() && hint != display) {
                    Text(
                        text = hint,
                        fontSize = 10.sp,
                        color = Color(0xFFD0BCFF),
                        modifier = Modifier.align(Alignment.TopEnd).padding(top = 2.dp, end = 2.dp)
                    )
                }
            }
        }
        if (showPopup) {
            Card(
                modifier = Modifier.offset(y = (-60).dp),
                colors = CardDefaults.cardColors(containerColor = Color(0xFF4F378B))
            ) {
                Text(
                    text = hint,
                    modifier = Modifier.padding(8.dp),
                    color = Color.White,
                    fontSize = 18.sp
                )
            }
            // Auto hide
            LaunchedEffect(showPopup) {
                kotlinx.coroutines.delay(600)
                showPopup = false
            }
        }
    }
}

@Composable
fun ShiftButton(
    isShift: Boolean,
    modifier: Modifier = Modifier,
    onShift: () -> Unit
) {
    Button(
        onClick = onShift,
        modifier = modifier.height(52.dp),
        colors = ButtonDefaults.buttonColors(
            containerColor = if (isShift) Color(0xFF7D5260) else Color(0xFF625B71),
            contentColor = Color.White
        )
    ) {
        Text(text = if (isShift) "⇧" else "Shift", fontSize = 14.sp, color = Color.White)
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
