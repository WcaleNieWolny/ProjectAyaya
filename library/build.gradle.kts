import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    id("java")
    kotlin("jvm") version "1.7.10"
}

group = "me.wcaleniewolny.ayaya"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    testImplementation(kotlin("test"))
    implementation(kotlin("stdlib"))
}

tasks.getByName<Test>("test") {
    useJUnitPlatform()
    jvmArgs = listOf("-Djava.library.path=${rootProject.rootDir.path}/ayaya_native/target/debug/")
}

sourceSets {
    named("main") {
        java.srcDir("src/main/kotlin")
    }
    named("test") {
        java.srcDir("src/test/kotlin")
    }
}
val compileKotlin: KotlinCompile by tasks
compileKotlin.kotlinOptions {
    jvmTarget = "17"
}