use anyhow::Result;
use colored::*;
use cyrce_forge_core::config::ForgeConfig;
use std::fs;
use std::path::Path;

pub async fn cmd_ide(project_dir: &Path, target: &str) -> Result<()> {
    let config = match ForgeConfig::load(project_dir) {
        Ok(c) => c,
        Err(_) => {
            println!("{}","❌ Error: No se encontró 'forge.toml'. Inicializa el proyecto primero con 'forge init'.".red());
            return Ok(());
        }
    };

    match target.to_lowercase().as_str() {
        "vscode" => generate_vscode(project_dir, &config),
        "intellij" => generate_intellij(project_dir, &config),
        _ => {
            println!("{}","❌ Editor no soportado. Usa 'vscode' o 'intellij'.".red());
            Ok(())
        }
    }
}

fn generate_vscode(project_dir: &Path, config: &ForgeConfig) -> Result<()> {
    let vscode_dir = project_dir.join(".vscode");
    if !vscode_dir.exists() {
        fs::create_dir_all(&vscode_dir)?;
    }

    // settings.json
    let settings = r#"{
    "evenBetterToml.schema.associations": {
        "forge.toml": "https://raw.githubusercontent.com/enri312/forge/main/schemas/forge.schema.json",
        "tests/**/forge.toml": "https://raw.githubusercontent.com/enri312/forge/main/schemas/forge.schema.json"
    }
}"#;
    fs::write(vscode_dir.join("settings.json"), settings)?;

    // tasks.json
    let tasks = r#"{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Forge: Build",
            "type": "shell",
            "command": "forge build",
            "group": {
                "kind": "build",
                "isDefault": true
            },
            "presentation": {
                "reveal": "always",
                "panel": "shared"
            }
        },
        {
            "label": "Forge: Run",
            "type": "shell",
            "command": "forge run",
            "presentation": {
                "reveal": "always",
                "panel": "shared"
            }
        },
        {
            "label": "Forge: Test",
            "type": "shell",
            "command": "forge test",
            "group": "test",
            "presentation": {
                "reveal": "always",
                "panel": "shared"
            }
        }
    ]
}"#;
    fs::write(vscode_dir.join("tasks.json"), tasks)?;

    // launch.json dependent on lang
    let launch = match config.project.lang.as_str() {
        "java" | "kotlin" => r#"{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "java",
            "name": "Forge: Run Java/Kotlin",
            "request": "launch",
            "mainClass": "${command:java.resolveMainClass}",
            "projectName": "${workspaceFolderBasename}"
        }
    ]
}"#,
        "python" => r#"{
    "version": "0.2.0",
    "configurations": [
        {
            "name": "Forge: Run Python",
            "type": "debugpy",
            "request": "launch",
            "program": "${file}",
            "console": "integratedTerminal"
        }
    ]
}"#,
        _ => r#"{
    "version": "0.2.0",
    "configurations": []
}"#,
    };
    
    fs::write(vscode_dir.join("launch.json"), launch)?;

    println!("{}","✅ Archivos de configuración para VS Code generados en .vscode/".green());
    Ok(())
}

fn generate_intellij(project_dir: &Path, config: &ForgeConfig) -> Result<()> {
    let idea_dir = project_dir.join(".idea");
    if !idea_dir.exists() {
        fs::create_dir_all(&idea_dir)?;
    }

    let project_name = &config.project.name;
    
    let modules_xml = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<project version="4">
  <component name="ProjectModuleManager">
    <modules>
      <module fileurl="file://$PROJECT_DIR$/.idea/{project_name}.iml" filepath="$PROJECT_DIR$/.idea/{project_name}.iml" />
    </modules>
  </component>
</project>"#);
    fs::write(idea_dir.join("modules.xml"), modules_xml)?;

    let iml_content = match config.project.lang.as_str() {
        "java" | "kotlin" => r#"<?xml version="1.0" encoding="UTF-8"?>
<module type="JAVA_MODULE" version="4">
  <component name="NewModuleRootManager" inherit-compiler-output="true">
    <exclude-output />
    <content url="file://$MODULE_DIR$/..">
      <sourceFolder url="file://$MODULE_DIR$/../src/main/java" isTestSource="false" />
      <sourceFolder url="file://$MODULE_DIR$/../src/main/kotlin" isTestSource="false" />
      <sourceFolder url="file://$MODULE_DIR$/../src/test/java" isTestSource="true" />
      <sourceFolder url="file://$MODULE_DIR$/../src/test/kotlin" isTestSource="true" />
      <excludeFolder url="file://$MODULE_DIR$/../.forge" />
      <excludeFolder url="file://$MODULE_DIR$/../build" />
    </content>
    <orderEntry type="inheritedJdk" />
    <orderEntry type="sourceFolder" forTests="false" />
  </component>
</module>"#,
        "python" => r#"<?xml version="1.0" encoding="UTF-8"?>
<module type="PYTHON_MODULE" version="4">
  <component name="NewModuleRootManager" inherit-compiler-output="true">
    <exclude-output />
    <content url="file://$MODULE_DIR$/..">
      <sourceFolder url="file://$MODULE_DIR$/../src" isTestSource="false" />
      <sourceFolder url="file://$MODULE_DIR$/../tests" isTestSource="true" />
      <excludeFolder url="file://$MODULE_DIR$/../.forge" />
    </content>
    <orderEntry type="inheritedJdk" />
    <orderEntry type="sourceFolder" forTests="false" />
  </component>
</module>"#,
        _ => r#"<?xml version="1.0" encoding="UTF-8"?><module type="WEB_MODULE" version="4"></module>"#,
    };

    fs::write(idea_dir.join(format!("{}.iml", project_name)), iml_content)?;

    println!("{}","✅ Archivos de configuración para IntelliJ generados en .idea/".green());
    Ok(())
}
