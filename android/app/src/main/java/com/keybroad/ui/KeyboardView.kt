package com.keybroad.ui

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.keybroad.data.KeyData

// Gboard-style palette
private val GboardBackground = Color(0xFFF1F3F4)
private val GboardKeyBackground = Color.White
private val GboardKeyText = Color(0xFF202124)
private val GboardSpecialKeyBackground = Color(0xFFD2D4D7)
private val GboardSuggestionBackground = Color.White
private val GboardSuggestionText = Color(0xFF202124)
private val GboardAccent = Color(0xFF1A73E8)

@Composable
fun KeyboardView(
    keys: List<KeyData>,
    onKeyPress: (keyData: KeyData, isShift: Boolean, isCaps: Boolean) -> Unit,
    onSpecialKey: (keyCode: Int) -> Unit,
    onToggleLanguage: () -> Unit = {},
    suggestions: List<String> = emptyList(),
    onSelectSuggestion: (String) -> Unit = {},
    isEnglishMode: Boolean = false,
    isShift: Boolean = false
) {
    if (keys.isEmpty()) {
        Text("Loading keyboard...")
        return
    }

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .background(GboardBackground)
            .padding(horizontal = 4.dp, vertical = 6.dp)
    ) {
        // Suggestion bar — Gboard style
        SuggestionBar(
            suggestions = suggestions,
            onSelectSuggestion = onSelectSuggestion
        )

        // QWERTY rows from Roman keys
        QwertyRows(
            keys = keys,
            onKeyPress = onKeyPress,
            onSpecialKey = onSpecialKey,
            isShift = isShift
        )

        // Bottom row: ?123  Globe  Space  Enter
        BottomRow(
            onSpecialKey = onSpecialKey,
            onToggleLanguage = onToggleLanguage,
            isEnglishMode = isEnglishMode
        )
    }
}

@Composable
fun SuggestionBar(
    suggestions: List<String>,
    onSelectSuggestion: (String) -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(GboardSuggestionBackground)
            .padding(horizontal = 4.dp, vertical = 6.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        if (suggestions.isEmpty()) {
            // Placeholder to keep height consistent
            Text(
                text = "",
                modifier = Modifier.height(20.dp)
            )
        } else {
            suggestions.take(3).forEach { item ->
                TextButton(
                    onClick = { onSelectSuggestion(item) },
                    modifier = Modifier.weight(1f),
                    contentPadding = PaddingValues(horizontal = 8.dp, vertical = 4.dp)
                ) {
                    Text(
                        text = item,
                        fontSize = 15.sp,
                        color = GboardSuggestionText,
                        fontWeight = FontWeight.Normal
                    )
                }
            }
        }
    }
    Divider(color = Color(0xFFE8EAED), thickness = 1.dp)
    Spacer(modifier = Modifier.height(4.dp))
}

@Composable
fun QwertyRows(
    keys: List<KeyData>,
    onKeyPress: (KeyData, Boolean, Boolean) -> Unit,
    onSpecialKey: (keyCode: Int) -> Unit,
    isShift: Boolean
) {
    val filtered = keys.filter { it.key != "space" && it.key != " " && it.key.length == 1 && it.key[0].isLetter() }
    // QWERTY order
    val order = listOf(
        "q","w","e","r","t","y","u","i","o","p",
        "a","s","d","f","g","h","j","k","l",
        "z","x","c","v","b","n","m"
    )
    val sorted = filtered.sortedBy { order.indexOf(it.key.lowercase()).let { idx -> if (idx == -1) 100 else idx } }
    val row1 = sorted.filter { it.key.lowercase() in order.subList(0,10) } // q-p
    val row2 = sorted.filter { it.key.lowercase() in order.subList(10,19) } // a-l
    val row3Letters = sorted.filter { it.key.lowercase() in order.subList(19,26) } // z-m

    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        // Row 1: Q W E R T Y U I O P
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            row1.forEach { kd ->
                GboardKey(
                    keyData = kd,
                    isShift = isShift,
                    modifier = Modifier.weight(1f),
                    onKeyPress = onKeyPress
                )
            }
        }
        // Row 2: A S D F G H J K L (with slight inset)
        Row(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 12.dp),
            horizontalArrangement = Arrangement.spacedBy(6.dp)
        ) {
            row2.forEach { kd ->
                GboardKey(
                    keyData = kd,
                    isShift = isShift,
                    modifier = Modifier.weight(1f),
                    onKeyPress = onKeyPress
                )
            }
        }
        // Row 3: Shift | Z X C V B N M | Backspace
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(6.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            GboardSpecialKey(
                label = if (isShift) "⇧" else "↑",
                containerColor = if (isShift) GboardAccent else GboardSpecialKeyBackground,
                contentColor = if (isShift) Color.White else GboardKeyText,
                modifier = Modifier.weight(1.3f),
                onClick = { onSpecialKey(59) }
            )
            row3Letters.forEach { kd ->
                GboardKey(
                    keyData = kd,
                    isShift = isShift,
                    modifier = Modifier.weight(1f),
                    onKeyPress = onKeyPress
                )
            }
            GboardSpecialKey(
                label = "⌫",
                containerColor = GboardSpecialKeyBackground,
                contentColor = GboardKeyText,
                modifier = Modifier.weight(1.3f),
                onClick = { onSpecialKey(67) }
            )
        }
    }
}

@Composable
fun BottomRow(
    onSpecialKey: (keyCode: Int) -> Unit,
    onToggleLanguage: () -> Unit,
    isEnglishMode: Boolean
) {
    Spacer(modifier = Modifier.height(6.dp))
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        // ?123 placeholder
        GboardSpecialKey(
            label = "?123",
            containerColor = GboardSpecialKeyBackground,
            contentColor = GboardKeyText,
            modifier = Modifier.weight(1.1f),
            onClick = { onSpecialKey(0) }
        )
        // Globe — toggles English / Bangla
        GboardSpecialKey(
            label = "🌐",
            containerColor = GboardSpecialKeyBackground,
            contentColor = GboardKeyText,
            modifier = Modifier.weight(1.1f),
            onClick = onToggleLanguage
        )
        // Space — shows EN / বাংলা
        GboardSpaceKey(
            label = if (isEnglishMode) "English" else "বাংলা",
            modifier = Modifier.weight(3.5f),
            onClick = { onSpecialKey(32) }
        )
        GboardSpecialKey(
            label = "↵",
            containerColor = GboardAccent,
            contentColor = Color.White,
            modifier = Modifier.weight(1.4f),
            onClick = { onSpecialKey(66) }
        )
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun GboardSpecialKey(
    label: String,
    containerColor: Color,
    contentColor: Color,
    modifier: Modifier = Modifier,
    onClick: () -> Unit
) {
    Card(
        modifier = modifier
            .height(48.dp)
            .shadow(1.dp, RoundedCornerShape(6.dp))
            .combinedClickable(onClick = onClick),
        shape = RoundedCornerShape(6.dp),
        colors = CardDefaults.cardColors(containerColor = containerColor)
    ) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text(text = label, fontSize = 16.sp, color = contentColor)
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun GboardSpaceKey(
    label: String,
    modifier: Modifier = Modifier,
    onClick: () -> Unit
) {
    Card(
        modifier = modifier
            .height(48.dp)
            .shadow(1.dp, RoundedCornerShape(24.dp))
            .combinedClickable(onClick = onClick),
        shape = RoundedCornerShape(24.dp),
        colors = CardDefaults.cardColors(containerColor = GboardKeyBackground)
    ) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text(
                text = label,
                fontSize = 14.sp,
                color = Color(0xFF5F6368),
                fontWeight = FontWeight.Medium
            )
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun GboardKey(
    keyData: KeyData,
    isShift: Boolean,
    modifier: Modifier = Modifier,
    onKeyPress: (KeyData, Boolean, Boolean) -> Unit
) {
    var showPopup by remember { mutableStateOf(false) }
    val display = if (isShift) keyData.shiftOutput.uppercase() else keyData.output.lowercase()
    val hint = if (isShift) keyData.output else keyData.shiftOutput

    Box(modifier = modifier.height(48.dp), contentAlignment = Alignment.Center) {
        Card(
            modifier = Modifier
                .fillMaxSize()
                .shadow(1.dp, RoundedCornerShape(6.dp))
                .combinedClickable(
                    onClick = { onKeyPress(keyData, isShift, false) },
                    onLongClick = {
                        showPopup = true
                        onKeyPress(keyData, !isShift, false)
                    }
                ),
            shape = RoundedCornerShape(6.dp),
            colors = CardDefaults.cardColors(containerColor = GboardKeyBackground)
        ) {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                Text(
                    text = display,
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Normal,
                    color = GboardKeyText
                )
                if (hint.isNotEmpty() && hint.lowercase() != display.lowercase() && hint.length == 1) {
                    Text(
                        text = hint,
                        fontSize = 9.sp,
                        color = Color(0xFF80868B),
                        modifier = Modifier.align(Alignment.TopEnd).padding(top = 2.dp, end = 4.dp)
                    )
                }
            }
        }
        if (showPopup) {
            Card(
                modifier = Modifier.offset(y = (-54).dp),
                shape = RoundedCornerShape(8.dp),
                colors = CardDefaults.cardColors(containerColor = Color(0xFF202124))
            ) {
                Text(
                    text = hint.ifEmpty { display },
                    modifier = Modifier.padding(horizontal = 12.dp, vertical = 8.dp),
                    color = Color.White,
                    fontSize = 18.sp
                )
            }
            LaunchedEffect(showPopup) {
                kotlinx.coroutines.delay(500)
                showPopup = false
            }
        }
    }
}


