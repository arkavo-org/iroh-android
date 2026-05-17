plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
    `maven-publish`
}

val libVersion = file("../../VERSION").readText().trim()

group = "net.arkavo"
version = libVersion

android {
    namespace = "net.arkavo.iroh"
    compileSdk = 34

    defaultConfig {
        minSdk = 28
        ndk {
            // Keep in sync with scripts/build-aar.sh
            abiFilters += setOf("arm64-v8a", "armeabi-v7a", "x86_64")
        }
        consumerProguardFiles("consumer-rules.pro")
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }

    kotlinOptions {
        jvmTarget = "11"
    }

    sourceSets {
        getByName("main") {
            // build-aar.sh drops .so files here before Gradle assembles.
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }

    publishing {
        singleVariant("release") {
            withSourcesJar()
        }
    }
}

dependencies {
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.7.3")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.5")
    androidTestImplementation("androidx.test:runner:1.5.2")
    androidTestImplementation("org.jetbrains.kotlinx:kotlinx-coroutines-test:1.7.3")
}

publishing {
    publications {
        register<MavenPublication>("release") {
            groupId = "net.arkavo"
            artifactId = "iroh-android"
            version = libVersion
            afterEvaluate {
                from(components["release"])
            }
            pom {
                name.set("iroh-android")
                description.set("Kotlin/JNI bindings for Iroh blob storage on Android")
                url.set("https://github.com/arkavo-org/iroh-android")
                licenses {
                    license {
                        name.set("MIT")
                        url.set("https://opensource.org/licenses/MIT")
                    }
                    license {
                        name.set("Apache-2.0")
                        url.set("https://www.apache.org/licenses/LICENSE-2.0")
                    }
                }
                scm {
                    url.set("https://github.com/arkavo-org/iroh-android")
                    connection.set("scm:git:https://github.com/arkavo-org/iroh-android.git")
                }
            }
        }
    }
    repositories {
        maven {
            name = "GitHubPackages"
            url = uri("https://maven.pkg.github.com/arkavo-org/iroh-android")
            credentials {
                username = (findProperty("gpr.user") as String?)
                    ?: System.getenv("GITHUB_ACTOR")
                password = (findProperty("gpr.token") as String?)
                    ?: System.getenv("GITHUB_TOKEN")
            }
        }
    }
}
