package com.sidetrack.audio

import android.media.audiofx.Equalizer
import com.sidetrack.utils.clamp
import java.util.concurrent.locks.ReentrantLock

class AudioEffectsManager {
    private var equalizer: Equalizer? = null
    private var supported: Boolean = false
    private var cachedBandCount: Int = 0
    private var cachedFrequencies: List<Float> = emptyList()
    private var cachedRange: Pair<Int, Int> = Pair(0, 0)
    var settingsEnabled: Boolean = false
        get() = field
        set(value) {
            field = value
            if (!value && equalizer != null) {
                equalizer?.release()
                equalizer = null
            }
        }
    private val lock = ReentrantLock()
    private var currentLevels: List<Int> = emptyList()
    private var sessionId: Int = 0

    fun generateSessionId(context: android.content.Context): Int {
        val audioManager = context.getSystemService(android.media.AudioManager::class.java)
        return audioManager.generateAudioSessionId()
    }

    fun setSessionId(id: Int) {
        lock.lock()
        try { sessionId = id } finally { lock.unlock() }
    }

    fun checkSupport(context: android.content.Context): Boolean {
        return try {
            val sid = if (sessionId != 0) sessionId else {
                val am = context.getSystemService(android.media.AudioManager::class.java)
                am.generateAudioSessionId()
            }
            val eq = Equalizer(0, sid)
            cachedBandCount = eq.numberOfBands.toInt()
            cachedFrequencies = List(cachedBandCount) { eq.getCenterFreq(it.toShort()).toFloat() / 1000f }
            val r = eq.getBandLevelRange()
            cachedRange = Pair(r[0].toInt(), r[1].toInt())
            eq.release()
            supported = true
            true
        } catch (e: Exception) {
            supported = false
            false
        }
    }

    fun onTrackCreated(track: android.media.AudioTrack) {
        lock.lock()
        try {
            if (!settingsEnabled || !supported) return
            equalizer?.release()
            val eq = try {
                Equalizer(0, track.audioSessionId).also { it.enabled = true }
            } catch (e: Exception) {
                supported = false
                null
            }
            equalizer = eq
            if (eq != null) applyLevels(eq, currentLevels)
        } finally { lock.unlock() }
    }

    fun onTrackReleased() {
        lock.lock()
        try {
            equalizer?.release()
            equalizer = null
        } finally { lock.unlock() }
    }

    fun setLevels(levels: List<Int>) {
        lock.lock()
        try {
            val eq = equalizer ?: return
            val range = eq.getBandLevelRange()
            val minLevel = range[0].toInt()
            val maxLevel = range[1].toInt()
            currentLevels = levels.map { clamp(it, minLevel, maxLevel) }
            applyLevels(eq, currentLevels)
        } finally { lock.unlock() }
    }

    private fun applyLevels(eq: Equalizer, levels: List<Int>) {
        val range = eq.getBandLevelRange()
        val minLevel = range[0].toInt()
        val maxLevel = range[1].toInt()
        for (i in 0 until eq.numberOfBands.toInt()) {
            if (i < levels.size) {
                eq.setBandLevel(i.toShort(), clamp(levels[i], minLevel, maxLevel).toShort())
            }
        }
    }

    fun release() {
        lock.lock()
        try {
            settingsEnabled = false
            equalizer?.release()
            equalizer = null
        } finally { lock.unlock() }
    }

    val isSupported: Boolean get() = supported

    val bandCount: Int get() = equalizer?.numberOfBands?.toInt() ?: 0

    val centerFrequencies: List<Float>
        get() = if (equalizer != null) {
            val eq = equalizer!!
            List(eq.numberOfBands.toInt()) { eq.getCenterFreq(it.toShort()).toFloat() / 1000f }
        } else cachedFrequencies

    val levelRange: Pair<Int, Int>
        get() = if (equalizer != null) {
            val eq = equalizer!!
            val range = eq.getBandLevelRange()
            Pair(range[0].toInt(), range[1].toInt())
        } else cachedRange
}

val audioEffectsManager = AudioEffectsManager()