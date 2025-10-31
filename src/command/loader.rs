use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::types::Command;

pub struct CommandRegistry {
    commands: HashMap<String, Command>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn load_from_dir<P: AsRef<Path>>(&mut self, dir: P) -> Result<()> {
        let dir_path = dir.as_ref();

        for entry in fs::read_dir(dir_path)
            .context("Failed to read commands directory")?
        {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read {:?}", path))?;

                let command: Command = serde_yaml::from_str(&content)
                    .with_context(|| format!("Failed to parse {:?}", path))?;

                eprintln!("Loaded command: {}", command.name);
                self.commands.insert(command.name.clone(), command);
            }
        }

        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Command> {
        self.commands.get(name)
    }

    pub fn list(&self) -> Vec<&Command> {
        self.commands.values().collect()
    }
}
