use std::{fs, path::Path};
use crate::parser::config_item::ConfigItem;
use crate::parser::parser_state::ParserState;
use crate::parser::config::Configuration;


#[derive(Debug, Default)]
pub struct ConfigParser {
    pub name : String,
    pub state_stack: Vec<ParserState>,
    pub current_directive: String,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            name: String::new(),
            state_stack: vec![ParserState::Starting],
            current_directive: String::new(),
        }
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
        let mut i = 0;
        let mut parser_states : Vec<ParserState> = vec![ParserState::Starting];
        let mut raw_directive  = String::new();
        let mut found_directives : Vec<String> = vec![];
        let mut name = String::new();

        for val in content.chars() {
            let current_state = *parser_states.last().unwrap_or(&ParserState::Invalid);

            if current_state == ParserState::Invalid {
                break;
            }

            let next_state = current_state.next_state(val);
            if current_state != next_state {
                match current_state {
                    ParserState::Starting => {
                        match next_state {
                            ParserState::StartBlock => parser_states.push(next_state),
                            _ => {},
                        }
                    },
                    ParserState::Name => {
                        name.push(val);
                    }
                    ParserState::StartBlock => {
                        match next_state {
                            ParserState::Comment => parser_states.push(next_state),
                            ParserState::Seeking => parser_states.push(next_state),
                            ParserState::InDirective => parser_states.push(next_state),
                            ParserState::EndBlock => {
                                parser_states.pop();
                                parser_states.push(next_state);
                            }
                            _ => {},
                        }
                    },
                    ParserState::EndBlock => { },
                    ParserState::Seeking => {
                        match next_state {
                            ParserState::InDirective => parser_states.push(next_state),
                            ParserState::Comment => parser_states.push(next_state),
                            ParserState::EndBlock => {
                                parser_states.pop();
                                parser_states.push(next_state);
                            }
                            _ => {}
                        }
                    },
                    ParserState::Comment => {
                        parser_states.pop();
                    },
                    ParserState::InDirective => {
                        if next_state == ParserState::Seeking {
                            found_directives.push(raw_directive.clone());
                            raw_directive = String::new();

                            println!("{}", found_directives.last().unwrap());
                            parser_states.pop();
                        } else if next_state == ParserState::Comment {
                            parser_states.push(next_state);
                        }
                    }
                    _ => {}
                }
            } else if next_state == ParserState::InDirective {
                raw_directive.push(val);
            } else if next_state == ParserState::Name {
                name.push(val);
            }
        }

        for string_val in &found_directives {
            println!("{}", string_val)
        }

        Ok(items)
    }

    pub fn handle_transition(&mut self, config: &mut Configuration, token: char, from : ParserState, to : ParserState) {
        match from {
            ParserState::Starting => {
                match to {
                    ParserState::Name => self.start_name_transition(config, token),
                    ParserState::StartBlock => self.start_block_transition(config, token),
                    ParserState::Comment => self.start_comment_transition(config, token),
                    _ => {},
                }
            },
            ParserState::Name => {
                match to {
                    ParserState::Starting => self.end_name_transition(config, token),
                    ParserState::StartBlock => self.start_block_transition(config, token),
                    ParserState::Comment => self.start_comment_transition(config, token),
                    _ => {},
                }
            }
            ParserState::StartBlock => {
                match to {
                    ParserState::Seeking => self.seeking_transition(config, token),
                    ParserState::InDirective => self.in_directive_transition(config, token),
                    ParserState::Comment => self.start_comment_transition(config, token),
                    _ => {},
                }
            },
            ParserState::EndBlock => {

            },
            ParserState::Seeking => {
                match to {
                    ParserState::InDirective => self.in_directive_transition(config, token),
                    ParserState::Comment => self.start_comment_transition(config, token),
                    ParserState::EndBlock => self.end_block_transition(config, token),
                    _ => {},
                }
            },
            ParserState::Comment => {
                match to {
                    ParserState::Seeking => self.end_comment_transition(config, token),
                    _ => {},
                }
            },
            ParserState::InDirective => {
                match to {
                    ParserState::Comment => self.start_comment_transition(config, token),
                    ParserState::Seeking => self.seeking_transition(config, token),
                    ParserState::EndBlock => self.end_block_transition(config, token),
                    _ => {},
                }

            },
            ParserState::Invalid => {

            },
        }
    }

    pub fn start_name_transition(&mut self, config: &mut Configuration, token: char) {
        self.name = String::new();
        self.state_stack.push(ParserState::Name);
        self.name.push(token);
    }

    pub fn end_name_transition(&mut self, config: &mut Configuration, token: char) {
        let old_state = self.state_stack.pop();
        if old_state != Some(ParserState::Name) {
            panic!("Invalid state transition from {:?} to {:?}", old_state, token);
        }
    }

    pub fn start_block_transition(&mut self, _config: &mut Configuration, _token: char) {
        if self.state_stack.last() == Some(&ParserState::Name) {
            self.state_stack.pop();
        }
        self.state_stack.push(ParserState::StartBlock);
    }

    pub fn start_comment_transition(&mut self, _config: &mut Configuration, _token: char) {
        if self.state_stack.last() == Some(&ParserState::Name) {
            self.state_stack.pop();
        }
        self.state_stack.push(ParserState::Comment);
    }

    pub fn seeking_transition(&mut self, _config: &mut Configuration, _token: char) {
        if let Some(state) = self.state_stack.last() {
            if state == &ParserState::InDirective {
                self.state_stack.pop();
            } else {
                self.state_stack.push(ParserState::Seeking);
            }
        }
    }

    pub fn in_directive_transition(&mut self, config: &mut Configuration, token: char) {
        self.current_directive.push(token);
        self.state_stack.push(ParserState::InDirective);
    }

    pub fn end_block_transition(&mut self, _config: &mut Configuration, _token: char) {
        while let Some(state) = self.state_stack.pop() {
            if state == ParserState::StartBlock {
                break;
            }
        }

        self.state_stack.push(ParserState::EndBlock);
    }

    pub fn end_comment_transition(&mut self, _config: &mut Configuration, _token: char) {
        self.state_stack.pop();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_starting_stating_state() {
        let mut config_parser = ConfigParser::new();
        let mut configuration = Configuration::default();

        config_parser.handle_transition(&mut configuration, ' ', ParserState::Starting, ParserState::Starting);
        assert_eq!(config_parser.state_stack, vec![ParserState::Starting]);

        let mut config_parser = ConfigParser::new();
        let mut configuration = Configuration::default();
        config_parser.handle_transition(&mut configuration, 'n', ParserState::Starting, ParserState::Name);

        assert_eq!(config_parser.state_stack, vec![ParserState::Starting, ParserState::Name]);
        assert_eq!(config_parser.name, "n".to_string());

        let mut config_parser = ConfigParser::new();
        let mut configuration = Configuration::default();
        config_parser.handle_transition(&mut configuration, '#', ParserState::Starting, ParserState::Comment);

        assert_eq!(config_parser.state_stack, vec![ParserState::Starting, ParserState::Comment]);

        let mut config_parser = ConfigParser::new();
        let mut configuration = Configuration::default();

        config_parser.handle_transition(&mut configuration, ' ', ParserState::Starting, ParserState::StartBlock);

        assert_eq!(config_parser.state_stack, vec![ParserState::Starting, ParserState::StartBlock]);
    }

    #[test]
    fn test_name_state() {
        let mut config_parser = ConfigParser::new();
        let mut configuration = Configuration::default();
        config_parser.state_stack.push(ParserState::Name);
        config_parser.name = "a".to_string();

        config_parser.handle_transition(&mut configuration, ' ', ParserState::Name, ParserState::Starting);

        assert_eq!(config_parser.state_stack, vec![ParserState::Starting]);
        assert_eq!(config_parser.name, "a".to_string());

        let mut config_parser = ConfigParser::new();
        let mut configuration = Configuration::default();
        config_parser.state_stack.push(ParserState::Name);
        config_parser.name = "a".to_string();

        config_parser.handle_transition(&mut configuration, '#', ParserState::Name, ParserState::Comment);

        assert_eq!(config_parser.state_stack, vec![ParserState::Starting, ParserState::Comment]);
        assert_eq!(config_parser.name, "a".to_string());

        let mut config_parser = ConfigParser::new();
        let mut configuration = Configuration::default();
        config_parser.state_stack.push(ParserState::Name);
        config_parser.name = "a".to_string();

        config_parser.handle_transition(&mut configuration, '{', ParserState::Name, ParserState::StartBlock);

        assert_eq!(config_parser.state_stack, vec![ParserState::Starting, ParserState::StartBlock]);
        assert_eq!(config_parser.name, "a".to_string());
    }

    #[test]
    fn test_start_block() {
        let mut config_parser = ConfigParser::new();
        let mut configuration = Configuration::default();
        config_parser.state_stack.push(ParserState::StartBlock);

        config_parser.handle_transition(&mut configuration, ' ', ParserState::StartBlock, ParserState::Seeking);
        assert_eq!(config_parser.state_stack, vec![ParserState::Starting, ParserState::StartBlock, ParserState::Seeking]);

        let mut config_parser = ConfigParser::new();
        let mut configuration = Configuration::default();
        config_parser.state_stack.push(ParserState::StartBlock);

        config_parser.handle_transition(&mut configuration, 'a', ParserState::StartBlock, ParserState::InDirective);
        assert_eq!(config_parser.state_stack, vec![ParserState::Starting, ParserState::StartBlock, ParserState::InDirective]);
        assert_eq!(config_parser.current_directive, "a".to_string());

        let mut config_parser = ConfigParser::new();
        let mut configuration = Configuration::default();
        config_parser.state_stack.push(ParserState::StartBlock);

        config_parser.handle_transition(&mut configuration, ' ', ParserState::StartBlock, ParserState::Comment);
        assert_eq!(config_parser.state_stack, vec![ParserState::Starting, ParserState::StartBlock, ParserState::Comment]);
    }

    #[test]
    fn test_seeking() {
        let mut config_parser = ConfigParser::new();
        let mut configuration = Configuration::default();
        config_parser.state_stack.push(ParserState::StartBlock);
        config_parser.state_stack.push(ParserState::Seeking);

        config_parser.handle_transition(&mut configuration, 'a', ParserState::Seeking, ParserState::InDirective);
        assert_eq!(config_parser.state_stack, vec![ParserState::Starting, ParserState::StartBlock, ParserState::Seeking, ParserState::InDirective]);

        let mut config_parser = ConfigParser::new();
        let mut configuration = Configuration::default();
        config_parser.state_stack.push(ParserState::StartBlock);
        config_parser.state_stack.push(ParserState::Seeking);

        config_parser.handle_transition(&mut configuration, '#', ParserState::Seeking, ParserState::Comment);
        assert_eq!(config_parser.state_stack, vec![ParserState::Starting, ParserState::StartBlock, ParserState::Seeking, ParserState::Comment]);

        let mut config_parser = ConfigParser::new();
        let mut configuration = Configuration::default();
        config_parser.state_stack.push(ParserState::StartBlock);
        config_parser.state_stack.push(ParserState::Seeking);

        config_parser.handle_transition(&mut configuration, '#', ParserState::Seeking, ParserState::EndBlock);
        assert_eq!(config_parser.state_stack, vec![ParserState::Starting, ParserState::EndBlock]);
    }

    #[test]
    fn test_comment() {
        let mut config_parser = ConfigParser::new();
        let mut configuration = Configuration::default();
        config_parser.state_stack.push(ParserState::StartBlock);
        config_parser.state_stack.push(ParserState::Seeking);
        config_parser.state_stack.push(ParserState::Comment);

        config_parser.handle_transition(&mut configuration, '\n', ParserState::Comment, ParserState::Seeking);
        assert_eq!(config_parser.state_stack, vec![ParserState::Starting, ParserState::StartBlock, ParserState::Seeking]);
    }

    #[test]
    fn test_in_directive() {
        let mut config_parser = ConfigParser::new();
        let mut configuration = Configuration::default();
        config_parser.state_stack.push(ParserState::StartBlock);
        config_parser.state_stack.push(ParserState::Seeking);
        config_parser.state_stack.push(ParserState::InDirective);

        config_parser.handle_transition(&mut configuration, '#', ParserState::InDirective, ParserState::Comment);
        assert_eq!(config_parser.state_stack, vec![ParserState::Starting, ParserState::StartBlock, ParserState::Seeking,
                                                   ParserState::InDirective, ParserState::Comment]);

        let mut config_parser = ConfigParser::new();
        let mut configuration = Configuration::default();
        config_parser.state_stack.push(ParserState::StartBlock);
        config_parser.state_stack.push(ParserState::Seeking);
        config_parser.state_stack.push(ParserState::InDirective);

        config_parser.handle_transition(&mut configuration, ';', ParserState::InDirective, ParserState::Seeking);
        assert_eq!(config_parser.state_stack, vec![ParserState::Starting, ParserState::StartBlock, ParserState::Seeking]);

        let mut config_parser = ConfigParser::new();
        let mut configuration = Configuration::default();
        config_parser.state_stack.push(ParserState::StartBlock);
        config_parser.state_stack.push(ParserState::Seeking);
        config_parser.state_stack.push(ParserState::InDirective);

        config_parser.handle_transition(&mut configuration, ';', ParserState::InDirective, ParserState::EndBlock);
        assert_eq!(config_parser.state_stack, vec![ParserState::Starting, ParserState::EndBlock]);
    }
}