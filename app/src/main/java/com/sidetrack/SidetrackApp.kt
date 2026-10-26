package com.sidetrack

import android.app.Application
import coil.ImageLoader
import coil.ImageLoaderFactory
import coil.disk.DiskCache
import coil.memory.MemoryCache
import com.sidetrack.bridge.NativeBridge

class SidetrackApp : Application(), ImageLoaderFactory {
    override fun onCreate() {
        super.onCreate()
        NativeBridge.init()
        NativeBridge.setTmpDir(cacheDir.absolutePath)
    }

    override fun newImageLoader(): ImageLoader {
        return ImageLoader.Builder(this)
            .memoryCache {
                MemoryCache.Builder(this)
                    .maxSizePercent(0.10) // 10% of app memory for images
                    .build()
            }
            .diskCache {
                DiskCache.Builder()
                    .directory(cacheDir.resolve("image_cache"))
                    .maxSizeBytes(20L * 1024 * 1024) // 20MB disk cache
                    .build()
            }
            .crossfade(false) // disable crossfade to save CPU on M308
            .build()
    }
}
