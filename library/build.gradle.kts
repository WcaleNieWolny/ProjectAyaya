import com.github.jengelman.gradle.plugins.shadow.tasks.ShadowJar
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
    testImplementation(project(":library"))
}

tasks.getByName<Test>("test") {
    useJUnitPlatform()
    jvmArgs = listOf("-Djava.library.path=${rootProject.rootDir.path}/ayaya_native/target/release/")
}

sourceSets {
    named("main") {
        java.srcDir("src/main/kotlin")
    }
    named("test") {
        java.srcDir("src/test/kotlin")
    }
}

tasks.register<Jar>("packageTests") {
    from(sourceSets.test.get().output, sourceSets.main.get().output)

    manifest {
//        attributes(
//            "Class-Path": "",
//            'Main-Class': 'hello.HelloWorld'
//        )
        attributes(
            Pair("Main-Class", "me.wcaleniewolny.ayaya.MainAppTestKt")
        )
    }
}

tasks.register<ShadowJar>("shadowTests"){
    archiveClassifier.set("tests")
    from(sourceSets.test.get().output, sourceSets.main.get().output)

    manifest{
        attributes(
            Pair("Main-Class", "me.wcaleniewolny.ayaya.MainAppTestKt")
        )
    }
}


tasks.getByName("packageTests").dependsOn(tasks.shadowJar)

val compileKotlin: KotlinCompile by tasks
compileKotlin.kotlinOptions {
    jvmTarget = "17"
}