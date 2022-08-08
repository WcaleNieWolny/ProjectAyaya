import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    kotlin("jvm") version "1.7.10"
    id("java")
    id("com.github.johnrengelman.shadow") version "7.1.2"
}

group = "me.wcaleniewolny.ayaya"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

//dependencies {
//    testImplementation(kotlin("test"))
//}
allprojects {
    apply {
        plugin("java")
        plugin("com.github.johnrengelman.shadow")
    }

    tasks.build.get().dependsOn(tasks.shadowJar)

    java {
        toolchain.languageVersion.set(JavaLanguageVersion.of(17))
    }

    tasks.withType<KotlinCompile> {
        kotlinOptions.jvmTarget = "17"
    }

}

//sourceSets {
//    named("main") {
//        java.srcDir("src/main/kotlin")
//    }
//}

tasks.test {
    useJUnitPlatform()
}