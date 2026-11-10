package com.sidetrack.utils

fun clamp(value: Int, min: Int, max: Int): Int {
    return value.coerceIn(min, max)
}

fun clamp(value: Float, min: Float, max: Float): Float {
    return value.coerceIn(min, max)
}