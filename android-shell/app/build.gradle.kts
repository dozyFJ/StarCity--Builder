plugins { id("com.android.application") }
val runNumber = System.getenv("GITHUB_RUN_NUMBER")?.toIntOrNull() ?: 1
android {
    namespace = "com.dozyfj.starcity"
    compileSdk = 35
    defaultConfig {
        applicationId = "com.dozyfj.starcity"
        minSdk = 26
        targetSdk = 35
        versionCode = 200000 + runNumber
        versionName = "0.2.$runNumber"
        ndk { abiFilters += listOf("arm64-v8a") }
        manifestPlaceholders["otaSaveFormat"] = "1"
        manifestPlaceholders["otaShellHash"] = System.getenv("OTA_SHELL_HASH") ?: "development"
        manifestPlaceholders["otaBuild"] = System.getenv("GITHUB_SHA") ?: "builtin"
        manifestPlaceholders["otaSequence"] = System.getenv("OTA_SEQUENCE") ?: "0"
    }
    sourceSets.getByName("main").java.srcDir("../../android-shared/src/main/java")
    signingConfigs { create("release") {
        storeFile = file(System.getenv("STAR_CITY_KEYSTORE") ?: "../../ci/starcity-builder.keystore")
        storePassword = System.getenv("STAR_CITY_KEYSTORE_PASSWORD")
        keyAlias = "androiddebugkey"
        keyPassword = System.getenv("STAR_CITY_KEYSTORE_PASSWORD")
    } }
    buildTypes { release { isMinifyEnabled = false; isDebuggable = false; signingConfig = if (project.findProperty("otaUnsigned") == "true") null else signingConfigs.getByName("release") } }
    compileOptions { sourceCompatibility = JavaVersion.VERSION_17; targetCompatibility = JavaVersion.VERSION_17 }
    packaging { jniLibs { useLegacyPackaging = true } }
}
