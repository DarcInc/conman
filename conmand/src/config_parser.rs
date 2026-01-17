use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    String(String),
    Boolean(bool),
    Array(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct ConfigItem {
    pub name: String,
    pub values: HashMap<String, ConfigValue>,
    pub directives: Vec<String>,
}

impl ConfigItem {
    pub fn new(name: String) -> Self {
        Self {
            name,
            values: HashMap::new(),
            directives: Vec::new(),
        }
    }

    pub fn add_value(&mut self, key: String, value: ConfigValue) {
        self.values.insert(key, value);
    }

    pub fn add_directive(&mut self, directive: String) {
        self.directives.push(directive);
    }
}

#[derive(Debug, Default)]
pub struct ConfigParser;

impl ConfigParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Vec<ConfigItem>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        self.parse_content(&content)
    }

    pub fn parse_content(
        &self,
        content: &str,
    ) -> Result<Vec<ConfigItem>, Box<dyn std::error::Error>> {
        let mut items = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                i += 1;
                continue;
            }

            // Look for container blocks (e.g., "legolas {")
            if let Some(container_name) = self.extract_container_name(line) {
                let mut config_item = ConfigItem::new(container_name);
                i += 1;

                // Parse the block content
                while i < lines.len() {
                    let block_line = lines[i].trim();

                    if block_line == "}" {
                        i += 1;
                        break;
                    }

                    if !block_line.is_empty() && !block_line.starts_with('#') {
                        self.parse_config_line(block_line, &mut config_item);
                    }
                    i += 1;
                }

                items.push(config_item);
            } else {
                i += 1;
            }
        }

        Ok(items)
    }

    fn extract_container_name(&self, line: &str) -> Option<String> {
        if line.ends_with('{') {
            let name_part = &line[..line.len() - 1].trim();
            Some(name_part.to_string())
        } else {
            None
        }
    }

    fn parse_config_line(&self, line: &str, config_item: &mut ConfigItem) {
        // Handle directives (standalone statements without =)
        if !line.contains('=') && !line.contains('+') {
            config_item.add_directive(line.to_string());
            return;
        }

        // Handle key-value pairs
        if let Some((key, value)) = self.parse_key_value(line) {
            let config_value = if value.contains(',') {
                // Array value
                let array_values: Vec<String> = value
                    .split(',')
                    .map(|v| v.trim().trim_matches('"').to_string())
                    .filter(|v| !v.is_empty())
                    .collect();
                ConfigValue::Array(array_values)
            } else {
                // String value
                let trimmed_value = value.trim().trim_matches('"');
                ConfigValue::String(trimmed_value.to_string())
            };
            config_item.add_value(key, config_value);
        }
    }

    fn parse_key_value(&self, line: &str) -> Option<(String, String)> {
        // Handle += operator (array append)
        if line.contains("+=") {
            let parts: Vec<&str> = line.splitn(2, "+=").collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().trim_matches(';').to_string();
                return Some((key, value));
            }
        }

        // Handle = operator
        if line.contains('=') {
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().trim_matches(';').to_string();
                return Some((key, value));
            }
        }

        None
    }
}
