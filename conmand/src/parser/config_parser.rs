use std::{fs, path::Path};

use crate::parser::config_item::ConfigItem;
use crate::parser::parser_state::ParserState;



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

}

#[cfg(test)]
mod test {
    use super::*;


}