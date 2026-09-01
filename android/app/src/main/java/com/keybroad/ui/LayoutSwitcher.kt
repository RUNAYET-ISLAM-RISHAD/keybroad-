package com.keybroad.ui

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun LayoutSwitcher(
    currentLayout: String,
    onLayoutSelected: (String) -> Unit
) {
    val layouts = listOf("Phonetic", "Jatiya", "Probhat", "Unijoy", "English")
    Row {
        layouts.forEach { layout ->
            Button(
                onClick = { onLayoutSelected(layout) },
                modifier = Modifier.padding(4.dp)
            ) {
                Text(if (layout == currentLayout) "*$layout" else layout)
            }
        }
    }
}