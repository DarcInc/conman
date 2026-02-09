//!
//! Copyright (c) 2026, Paul C. Hoehne
//!
//! Redistribution and use in source and binary forms, with or without modification, are
//! permitted provided that the following conditions are met:
//!
//!   Redistributions of source code must retain the above copyright notice, this list of
//!   conditions and the following disclaimer.
//!
//!   Redistributions in binary form must reproduce the above copyright notice, this list of
//!   conditions and the following disclaimer in the documentation and/or other materials
//!   provided with the distribution.
//!
//! THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY
//! EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF
//! MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL
//! THE COPYRIGHT OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
//! SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT
//! OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
//! HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
//! OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
//! SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//!

use std::process::{Command, Stdio};
use log::{warn, error};
use crate::jls::configuration::Configuration;
use crate::jls::parameters::Parameters;
use regex::Regex;

pub struct JlsCommand {}

const CONFIG_DIRECTIVE_RE: &str = r#"^(?<name>[\w+\.]+)(?:=(?:(?<disabled>disable)|(?<numeric>\d+)|(?:"(?<quoted>.*)")|(?<unquoted>\w*)))?$"#;

impl JlsCommand {
    pub fn new() -> JlsCommand {
        JlsCommand {}
    }

    pub fn list_jails(&self) -> std::result::Result<Vec<Configuration>, Box<dyn std::error::Error>> {
        let cmd = Command::new("jls")
            .arg("-nq")
            .stdout(Stdio::piped())
            .spawn()
            .expect("jls command failed to start");

        let output = cmd.wait_with_output()
            .expect("jls command failed to start");

        let text_representation = String::from_utf8(output.stdout)?;
        let lines : Vec<String> = text_representation.lines().map(ToOwned::to_owned).collect();

        let jails = lines.iter().map(|line: &String| {
            let parts = self.tokenize_jls_line(line).unwrap_or_default();
            if let Ok(parameters) = self.convert_to_parameter_list(&parts) {
                Configuration::new(parameters)
            } else {
                Configuration::new(vec![])
            }
        }).collect();

        Ok(jails)
    }

    pub fn tokenize_jls_line(&self, raw: &str) -> std::result::Result<Vec<String>, Box<dyn std::error::Error>> {
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

    fn directive_to_paramter(&self, expr: &Regex, directive: &str) -> std::result::Result<Parameters, Box<dyn std::error::Error>> {
        if let Some(caps) = expr.captures(directive) {
            let name = caps.name("name").map_or("", |m| m.as_str());
            if name != "" {
                if let Some(_disabled) = caps.name("disabled") {
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

    pub fn convert_to_parameter_list(&self, raw : &Vec<String>) -> Result<Vec<Parameters>, Box<dyn std::error::Error>> {
        let expr = Regex::new(CONFIG_DIRECTIVE_RE)?;

        let result : Vec<Parameters> = raw.iter().map(|val| {
            let parsed = self.directive_to_paramter(&expr, val);
            match parsed {
                Ok(p) => p,
                Err(error) => {
                    error!("Failed to parse configuration directive {}: {}", val, error);
                    Parameters::StringParameter("NO NAME".to_string(), "NO VALUE".to_string())
                }
            }
        }).collect();

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_jls_line() {
        let line = "devfs_ruleset=5 nodying enforce_statfs=2 env=\"\" host=new ip4=disable";
        let expected: Vec<String> = vec![
            "devfs_ruleset=5".to_string(),
            "nodying".to_string(),
            "enforce_statfs=2".to_string(),
            "env=\"\"".to_string(),
            "host=new".to_string(),
            "ip4=disable".to_string()
        ];

        let jls = JlsCommand::new();
        let result = jls.tokenize_jls_line(line).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_directive_to_paramter_boolean() {
        let expr = Regex::new(CONFIG_DIRECTIVE_RE).unwrap();
        let jls = JlsCommand::new();

        let expected = Parameters::BooleanParameter("nodying".to_string(), true);
        let test_case = "nodying";
        let result = jls.directive_to_paramter(&expr, test_case);
        assert!(result.is_ok());
        assert_eq!(expected, result.unwrap());

        let expected = Parameters::BooleanParameter("ip4".to_string(), false);
        let test_case = "ip4=disable";
        let result = jls.directive_to_paramter(&expr, test_case);
        assert!(result.is_ok());
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_directive_to_paramter_numeric() {
        let expr = Regex::new(CONFIG_DIRECTIVE_RE).unwrap();
        let jls = JlsCommand::new();

        let expected = Parameters::NumberParameter("devfs.ruleset".to_string(), 5);
        let test_case = "devfs.ruleset=5";
        let result = jls.directive_to_paramter(&expr, test_case);
        assert!(result.is_ok());
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_directive_to_parameter_quoted_string() {
        let expr = Regex::new(CONFIG_DIRECTIVE_RE).unwrap();
        let jls = JlsCommand::new();

        let expected = Parameters::StringParameter("env".to_string(), String::default());
        let test_case = r#"env="""#;
        let result = jls.directive_to_paramter(&expr, test_case);
        assert!(result.is_ok());
        assert_eq!(expected, result.unwrap());

        let expected = Parameters::StringParameter("env".to_string(), "FOO=bar".to_string());
        let test_case = r#"env="FOO=bar""#;
        let result = jls.directive_to_paramter(&expr, test_case);
        assert!(result.is_ok());
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_directive_to_parameter_raw_string() {
        let expr = Regex::new(CONFIG_DIRECTIVE_RE).unwrap();
        let jls = JlsCommand::new();

        let expected = Parameters::StringParameter("host".to_string(), "new".to_string());
        let test_case = r#"host=new"#;
        let result = jls.directive_to_paramter(&expr, test_case);
        assert!(result.is_ok());
        assert_eq!(expected, result.unwrap());

        let expected = Parameters::StringParameter("nothing".to_string(), String::default());
        let test_case = r#"nothing="#;
        let result = jls.directive_to_paramter(&expr, test_case);
        assert!(result.is_ok());
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_to_parameter_list() {
        let line = "devfs_ruleset=5 nodying enforce_statfs=2 env=\"\" host=new ip4=disable";
        let expected: Vec<Parameters> = vec![
            Parameters::NumberParameter("devfs_ruleset".to_string(), 5),
            Parameters::BooleanParameter("nodying".to_string(), true),
            Parameters::NumberParameter("enforce_statfs".to_string(), 2),
            Parameters::StringParameter("env".to_string(), "".to_string()),
            Parameters::StringParameter("host".to_string(), "new".to_string()),
            Parameters::BooleanParameter("ip4".to_string(), false),
        ];
        let jls = JlsCommand::new();

        let tokenized = jls.tokenize_jls_line(line).unwrap();

        let result = jls.convert_to_parameter_list(&tokenized);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }
}