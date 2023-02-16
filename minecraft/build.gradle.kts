plugins {
    kotlin("jvm") version "1.7.10"
    id("kr.entree.spigradle") version "2.4.2"
    id("xyz.jpenilla.run-paper") version "1.0.6"
    id("io.papermc.paperweight.userdev") version "1.3.8"
    id("org.jlleitschuh.gradle.ktlint") version "11.2.0"
}

group = "me.wcaleniewolny.ayaya"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
    maven { url = uri("https://repo.papermc.io/repository/maven-public/") }
    maven { url = uri("https://repo.aikar.co/content/groups/aikar/") }
}

dependencies {
    paperDevBundle("1.18.2-R0.1-SNAPSHOT")
    testImplementation("org.junit.jupiter:junit-jupiter-api:5.8.1")
    testRuntimeOnly("org.junit.jupiter:junit-jupiter-engine:5.8.1")
    compileOnly("io.papermc.paper:paper-api:1.18.2-R0.1-SNAPSHOT")
    // compileOnly(group = "com.comphenix.protocol", name = "ProtocolLib", version = "4.7.0")
    implementation(kotlin("stdlib"))
    implementation(project(":library"))
    implementation("co.aikar:acf-paper:0.5.1-SNAPSHOT")
    implementation("net.kyori:adventure-text-minimessage:4.11.0-SNAPSHOT")
}

tasks.getByName<Test>("test") {
    useJUnitPlatform()
}

tasks.getByName("build").dependsOn(tasks.shadowJar)

tasks.getByName("assemble").dependsOn(tasks.reobfJar)

val compileKotlin: org.jetbrains.kotlin.gradle.tasks.KotlinCompile by tasks

compileKotlin.kotlinOptions {
    javaParameters = true
    jvmTarget = "17"
}

configure<org.jlleitschuh.gradle.ktlint.KtlintExtension> {
    outputToConsole.set(true)
    disabledRules.set(setOf("no-wildcard-imports"))
}

val runServer = tasks.runServer
runServer {
    minecraftVersion("1.18.2")
    jvmArgs = listOf("-Xmx20480M", "-Djava.library.path=${rootProject.rootDir.path}/ayaya_native/target/release", "-Dme.wcaleniewolny.ayaya.unsafe=true")
}

val shadowJar = tasks.shadowJar
shadowJar {
    relocate("co.aikar.commands", "me.wcaleniewolny.ayaya.minecraft.acf")
    relocate("co.aikar.locales", "me.wcaleniewolny.ayaya.minecraft.locales")
}

val pluginName = "ProjectAyaya"

spigot {
    name = pluginName
    authors = listOf("WcaleNieWolny")
    // depends = listOf("ProtocolLib")
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
}
