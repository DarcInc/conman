use std::process::{Command, Stdio};
use regex::Regex;
use log::{warn, error};

pub struct JlsCommand {}

#[derive(Debug, PartialEq)]
pub enum Parameters {
    BooleanParameter(String, bool),
    StringParameter(String, String),
    NumberParameter(String, i32),
}

impl JlsCommand {
    pub fn list_jails() -> std::result::Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut cmd = Command::new("jls")
            .arg("-nq")
            .stdout(Stdio::piped())
            .spawn()
            .expect("jls command failed to start");

        let output = cmd.wait_with_output()
            .expect("jls command failed to start");

        let text_representation = String::from_utf8(output.stdout)?;
        let lines : Vec<String> = text_representation.lines().map(ToOwned::to_owned).collect();
        println!("{:?}", lines.get(0).unwrap());

        Ok(vec![])
    }

    pub fn tokenize_jls_line(raw: &str) -> std::result::Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut result = vec![];
        let mut in_quotes = false;
        let mut current = String::new();

        for char in raw.chars() {
            if char != ' ' {
                current.push(char);
            } else if char == '"' && !in_quotes {
                in_quotes = true;
                current.push(char);
            } else if in_quotes {
                if char == '"' {
                    in_quotes = false;
                }
                current.push(char);
            } else {
                if current != "" {
                    result.push(current);
                }
                current = String::new();
            }
        }

        if current != "" {
            result.push(current);
        }

        Ok(result)
    }

    pub fn directive_to_paramter(expr: &Regex, directive: &str) -> std::result::Result<Parameters, Box<dyn std::error::Error>> {
        if let Some(caps) = expr.captures(directive) {
            let name = caps.name("name").map_or("", |m| m.as_str());
            if name != "" {
                if let Some(disabled) = caps.name("disabled") {
                    Ok(Parameters::BooleanParameter(name.to_string(), false))
                } else if let Some(numeric) = caps.name("numeric") {
                    let number = numeric.as_str().parse::<i32>();
                    if let Ok(n) = number {
                        Ok(Parameters::NumberParameter(name.to_string(), n))
                    } else {
                        warn!("Invalid numeric format: {} -> {}", name, numeric.as_str());
                        Ok(Parameters::NumberParameter(name.to_string(), -1))
                    }
                } else if let Some(quoted) = caps.name("quoted") {
                    Ok(Parameters::StringParameter(name.to_string(), quoted.as_str().to_string()))
                } else if let Some(unquoted) = caps.name("unquoted") {
                    Ok(Parameters::StringParameter(name.to_string(), unquoted.as_str().to_string()))
                } else {
                    Ok(Parameters::BooleanParameter(name.to_string(), true))
                }
            } else {
                warn!("Attempting to parse invalid row {}", directive);
                Ok(Parameters::StringParameter("NO NAME".to_string(), "NO VALUE".to_string()))
            }
        } else {
            Err("directive does not match regex")?
        }
    }

    pub fn convert_to_parameter_list(raw : &Vec<String>) -> Result<Vec<Parameters>, Box<dyn std::error::Error>> {
        let expr = Regex::new(r#"^(?<name>\w+\.)+=((?<disabled>disabled)|(?<numeric>\d+)|("(?<quoted>.*)")|(<?<unquoted>.+))?$"#)?;

        let result : Vec<Parameters> = raw.iter().map(|val| {
            if let Some(caps) = expr.captures(val) {
                let name = caps.name("name").map_or("", |m| m.as_str());
                if name != "" {
                    if let Some(disabled) = caps.name("disabled") {
                        Parameters::BooleanParameter(name.to_string(), false)
                    } else if let Some(numeric) = caps.name("numeric") {
                        let number = numeric.as_str().parse::<i32>();
                        if let Ok(n) = number {
                            Parameters::NumberParameter(name.to_string(), n)
                        } else {
                            warn!("Invalid numeric format: {} -> {}", name, numeric.as_str());
                            Parameters::NumberParameter(name.to_string(), -1)
                        }
                    } else if let Some(quoted) = caps.name("quoted") {
                        Parameters::StringParameter(name.to_string(), quoted.as_str().to_string())
                    } else if let Some(unquoted) = caps.name("unquoted") {
                        Parameters::StringParameter(name.to_string(), unquoted.as_str().to_string())
                    } else {
                        Parameters::BooleanParameter(name.to_string(), true)
                    }
                } else {
                    warn!("Attempting to parse invalid row {}", val);
                    Parameters::StringParameter("NO NAME".to_string(), "NO VALUE".to_string())
                }
            } else {
                error!("Regular expression miss-match {}", val);
                Parameters::StringParameter("NO NAME".to_string(), "NO VALUE".to_string())
            }
        }).collect();

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //
    const RE_EXPR: &str = r#"^(?<name>[\w+\.]+)(?:=(?:(?<disabled>disable)|(?<numeric>\d+)|(?:"(?<quoted>.*)")|(?<unquoted>\w*)))?$"#;

    #[test]
    fn test_tokenize_jls_line() {
        let line = "devfs_ruleset=5 nodying enforce_statfs=2 env=\"\" host=new ip4=disable";
        let expected : Vec<String> = vec![
            "devfs_ruleset=5".to_string(),
            "nodying".to_string(),
            "enforce_statfs=2".to_string(),
            "env=\"\"".to_string(),
            "host=new".to_string(),
            "ip4=disable".to_string()
        ];

        let result  = JlsCommand::tokenize_jls_line(line).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_directive_to_paramter_boolean() {
        let expr = Regex::new(RE_EXPR).unwrap();

        let expected = Parameters::BooleanParameter("nodying".to_string(), true);
        let test_case = "nodying";
        let result = JlsCommand::directive_to_paramter(&expr, test_case);
        assert!(result.is_ok());
        assert_eq!(expected, result.unwrap());

        let expected = Parameters::BooleanParameter("ip4".to_string(), false);
        let test_case = "ip4=disable";
        let result = JlsCommand::directive_to_paramter(&expr, test_case);
        assert!(result.is_ok());
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_directive_to_paramter_numeric() {
        let expr = Regex::new(RE_EXPR).unwrap();

        let expected = Parameters::NumberParameter("devfs.ruleset".to_string(), 5);
        let test_case = "devfs.ruleset=5";
        let result = JlsCommand::directive_to_paramter(&expr, test_case);
        assert!(result.is_ok());
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_directive_to_parameter_quoted_string() {
        let expr = Regex::new(RE_EXPR).unwrap();

        let expected = Parameters::StringParameter("env".to_string(), String::default());
        let test_case = r#"env="""#;
        let result = JlsCommand::directive_to_paramter(&expr, test_case);
        assert!(result.is_ok());
        assert_eq!(expected, result.unwrap());

        let expected = Parameters::StringParameter("env".to_string(), "FOO=bar".to_string());
        let test_case = r#"env="FOO=bar""#;
        let result = JlsCommand::directive_to_paramter(&expr, test_case);
        assert!(result.is_ok());
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_directive_to_parameter_raw_string() {
        let expr = Regex::new(RE_EXPR).unwrap();

        let expected = Parameters::StringParameter("host".to_string(), "new".to_string());
        let test_case = r#"host=new"#;
        let result = JlsCommand::directive_to_paramter(&expr, test_case);
        assert!(result.is_ok());
        assert_eq!(expected, result.unwrap());

        let expected = Parameters::StringParameter("nothing".to_string(), String::default());
        let test_case = r#"nothing="#;
        let result = JlsCommand::directive_to_paramter(&expr, test_case);
        assert!(result.is_ok());
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_to_parameter_list() {
        let line = "devfs_ruleset=5 nodying enforce_statfs=2 env=\"\" host=new ip4=disable";
        let expected : Vec<Parameters> = vec![
            Parameters::NumberParameter("devfs_ruleset".to_string(), 5),
            Parameters::BooleanParameter("nodying".to_string(), true),
            Parameters::NumberParameter("enforce_statfs".to_string(), 2),
            Parameters::StringParameter("env".to_string(), "".to_string()),
            Parameters::StringParameter("host".to_string(), "new".to_string()),
            Parameters::BooleanParameter("ip4".to_string(), false),
        ];

        let tokenized = JlsCommand::tokenize_jls_line(line).unwrap();

        let result = JlsCommand::convert_to_parameter_list(&tokenized);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }
}