plugins {
    id("java")
    kotlin("jvm") version "1.7.10"
    id("kr.entree.spigradle") version "2.4.2"
    id("xyz.jpenilla.run-paper") version "1.0.6"
}

group = "me.wcaleniewolny.ayaya"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
    maven { url = uri("https://repo.papermc.io/repository/maven-public/") }
    maven { url = uri("https://repo.dmulloy2.net/repository/public/") }
}

dependencies {
    testImplementation("org.junit.jupiter:junit-jupiter-api:5.8.1")
    testRuntimeOnly("org.junit.jupiter:junit-jupiter-engine:5.8.1")
    compileOnly("io.papermc.paper:paper-api:1.18.2-R0.1-SNAPSHOT")
    compileOnly(group = "com.comphenix.protocol", name = "ProtocolLib", version = "4.7.0")
    implementation(kotlin("stdlib"))
    implementation(project(":library"))
}

tasks.getByName<Test>("test") {
    useJUnitPlatform()
}

val compileKotlin: org.jetbrains.kotlin.gradle.tasks.KotlinCompile by tasks
compileKotlin.kotlinOptions {
    jvmTarget = "17"
}

val runServer = tasks.runServer
runServer {
    minecraftVersion("1.18.2")
    jvmArgs = listOf("-Djava.library.path=${rootProject.rootDir.path}/native/build")
}

val pluginName = "ProjectAyaya"

spigot {
    name = pluginName
    authors = listOf("WcaleNieWolny")
    depends = listOf("ProtocolLib")
    version = this.version
    website = "https://github.com/WcaleNieWolny/"
    apiVersion = "1.18"
    excludeLibraries = listOf("*")
    permissions {
        create("ayaya.use") {
            description = "Allow usage of /ayaya command"
            defaults = "op"
        }
    }
    commands {
        create("test") {
            description = "Test command"
        }
    }
}