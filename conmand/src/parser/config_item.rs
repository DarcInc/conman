use std::collections::HashMap;

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
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            values: HashMap::new(),
            directives: Vec::new(),
        }
    }

    pub fn add_value(&mut self, key: &str, value: &ConfigValue) {
        self.values.insert(key.to_string(), value.clone());
    }

    pub fn add_directive(&mut self, directive: &str) {
        self.directives.push(directive.to_string());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_test() {
        let item = ConfigItem::new("foo");
        assert_eq!("foo".to_string(), item.name);
        assert_eq!(0, item.values.len());
        assert_eq!(0, item.directives.len());
    }

    #[test]
    fn add_value_test() {
        let mut item = ConfigItem::new("foo");
        item.add_value("bar", &ConfigValue::String("baz".to_string()));

        assert_eq!(1, item.values.len());
        assert!(item.values.contains_key("bar"));
        assert_eq!(item.values.get("bar").unwrap(), &ConfigValue::String("baz".to_string()));

        item.add_value("alpha", &ConfigValue::Boolean(true));
        assert_eq!(2, item.values.len());
        assert!(item.values.contains_key("alpha"));
        assert_eq!(item.values.get("alpha").unwrap(), &ConfigValue::Boolean(true));

        item.add_value("baker", &ConfigValue::Array(vec!["charlie".to_string()]));
        assert_eq!(3, item.values.len());
        assert!(item.values.contains_key("baker"));
        assert_eq!(item.values.get("baker").unwrap(), &ConfigValue::Array(vec!["charlie".to_string()]));
    }

    #[test]
    fn add_directive_test() {
        let mut item = ConfigItem::new("foo");
        item.add_directive("foo");

        assert_eq!(1, item.directives.len());
        assert_eq!(vec!["foo".to_string()], item.directives);
    }
}