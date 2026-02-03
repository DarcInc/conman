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

use std::{fs, path::Path};
use crate::parser::config_item::ConfigItem;
use crate::parser::parser_state::ParserState;
use crate::parser::config::Configuration;

/// The parsing state when breaking apart a container configuration.  The state tracked is
/// a stack of states.  States are pushed and popped off the stack, with the top-most state
/// being the 'current' state.
///
/// * `state_stack` - A stack of the structure.
#[derive(Debug, Default)]
pub struct ConfigParser {
    pub state_stack: Vec<ParserState>,
}

impl ConfigParser {

    /// Creates a new configuration parser.
    pub fn new() -> Self {
        ConfigParser {
            state_stack: vec![ParserState::Starting],
        }
    }

    /// Read the container configuration from a file.
    ///
    /// * `p` - The path to the file
    pub fn parse_file<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<Configuration, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        self.parse_content(&content)
    }

    /// Tokenize the content from a container configuration.  Process each character, one at a
    /// time, and use that to determine the next state as per the state transitions.  If there
    /// is a state change, dispatch into the state change handling functions. Returns the parsed
    /// configuration or an error.
    ///
    /// If we are reading a name, and there is no state transition, we preserve the read token
    /// as part of the name.  If we are in the `ParserState::InDirective` state, we save the
    /// token as part of the directive string.
    ///
    /// * `content` - The content as a string
    pub fn parse_content(
        &mut self,
        content: &str,
    ) -> Result<Configuration, Box<dyn std::error::Error>> {

        let mut config = Configuration::default();

        for val in content.chars() {
            let current_state = *self.state_stack.last().unwrap_or(&ParserState::Invalid);

            if current_state == ParserState::Invalid {
                break;
            }

            let next_state = current_state.next_state(val);
            if current_state != next_state {
                self.handle_transition(&mut config, val, current_state, next_state);
            } else if next_state == ParserState::InDirective {
                if let Some(directive) = config.directives.last_mut() {
                    directive.raw.push(val);
                }
            } else if next_state == ParserState::Name {
                config.name.push(val);
            }
        }

        Ok(config)
    }

    /// Handle the state transitions.  Given the current configuration, a token, a current state
    /// and the next state, execute a transition function for that transition.
    ///
    /// * `config` - The configuration as parsed to this point.
    /// * `token` - The token that initiated the state transition.
    /// * `from` - The stated we are transitioning from
    /// * `to` - The state we are transitioning to.
    fn handle_transition(&mut self, config: &mut Configuration, token: char, from : ParserState,
                         to : ParserState) {
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

    /// Handle the transition that indicates we've started parsing the name of the container.
    /// The `ParserState::Name` state is pushed onto the stack and the token is stored as the
    /// first letter in the container name.
    ///
    /// * `config` - The configuration parsed so far.
    /// * `token` - The token that initiated the transition.
    fn start_name_transition(&mut self, config: &mut Configuration, token: char) {
        config.name = String::new();
        self.state_stack.push(ParserState::Name);
        config.name.push(token);
    }

    /// Ends the name transition.  We pop off the name state as we are done reading the name
    /// and transition back to what should be the `ParserState::Starting` state.
    ///
    /// * `_config` - The configuration parsed so far (not used).
    /// * `token` - The token that initiated the transition.
    fn end_name_transition(&mut self, _config: &mut Configuration, token: char) {
        let old_state = self.state_stack.pop();
        if old_state != Some(ParserState::Name) {
            panic!("Invalid state transition from {:?} to {:?}", old_state, token);
        }
    }

    /// Start the block that contains the directives for the container definition.  This
    /// pushes the `ParserState::StartBlock` state onto the state stack, indicating we are
    /// now in a block of configuration directives.
    ///
    /// * `_config` - The configuration parsed so far (not used).
    /// * `_token` - The token that initiated the transition (not used).
    fn start_block_transition(&mut self, _config: &mut Configuration, _token: char) {
        if self.state_stack.last() == Some(&ParserState::Name) {
            self.state_stack.pop();
        }
        self.state_stack.push(ParserState::StartBlock);
    }

    /// Starts a comment, which extends from the octothorpe character ('#') to the end of the
    /// line.  The `PareserState::Comment` state is pushed onto the stack as the current state.
    /// A comment can be encountered anywhere in the file.
    ///
    /// * Before the name
    /// * After the name but before the start of the block.
    /// * Immediately after the start of the block.
    /// * On a line on its own.
    /// * In the middle of a multi-line directive
    ///
    /// * `_config` - The configuration parsed so far (not used).
    /// * `_token` - The token that initiated the transition (not used).
    fn start_comment_transition(&mut self, _config: &mut Configuration, _token: char) {
        if self.state_stack.last() == Some(&ParserState::Name) {
            self.state_stack.pop();
        }
        self.state_stack.push(ParserState::Comment);
    }

    /// Start seeking a new directive.  This pushes the seeking state that implies we're in the
    /// whitespace between directives or the end of the configuration block.  If we were in a
    /// directive, we pop the directive state off the state off the stack.  Otherwise, the
    /// `ParserState::Seeking` state is pushed onto the stack.
    ///
    /// This handles the following situations:
    ///
    /// * We started a block and hit any whitespace character, so we're now seeking
    ///   for a directive.
    /// * We started a block and immediately encountered a directive.  We read the directive
    ///   and we're seeking the next directive.
    /// * We were seeking, read a directive or comment, and now we're seeking again.
    ///
    /// * `_config` - The configuration parsed so far (not used).
    /// * `_token` - The token that initiated the transition (not used).
    fn seeking_transition(&mut self, _config: &mut Configuration, _token: char) {
        if let Some(state) = self.state_stack.last() {
            if state == &ParserState::InDirective {
                self.state_stack.pop();
            } else {
                self.state_stack.push(ParserState::Seeking);
            }
        }
    }

    /// This is a transition into reading a directive.  A directive contains either an imparitive
    /// command or an assignment of a value.  But at this level, we just care about the string
    /// up to the semicolon that marks the end of the directive.  The token is stored as the
    /// first letter in the directive string and the `ParserState::InDirective` state is pushed
    /// onto the state stack.
    ///
    /// * `config` - The configuration parsed so far.
    /// * `token` - The token that initiated the transition.
    fn in_directive_transition(&mut self, config: &mut Configuration, token: char) {
        let mut item = ConfigItem::new("");
        item.raw.push(token);
        config.directives.push(item);
        self.state_stack.push(ParserState::InDirective);
    }

    /// Ends a configuration block for a container.  This is the end of the configuration and
    /// nothing of note should come after.
    ///
    /// * `_config` - The configuration parsed so far (not used).
    /// * `_token` - The token that initiated the transition (not used).
    pub fn end_block_transition(&mut self, _config: &mut Configuration, _token: char) {
        while let Some(state) = self.state_stack.pop() {
            if state == ParserState::StartBlock {
                break;
            }
        }

        self.state_stack.push(ParserState::EndBlock);
    }

    /// Ends a comment.  Whatever was happening when we were interrupted by a comment, we return
    /// to that activity.  We just pop the comment state off teh stack.
    ///
    /// * `_config` - The configuration parsed so far (not used).
    /// * `_token` - The token that initiated the transition (not used).
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
        assert_eq!(configuration.name, "n".to_string());

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
        configuration.name = "a".to_string();

        config_parser.handle_transition(&mut configuration, ' ', ParserState::Name, ParserState::Starting);

        assert_eq!(config_parser.state_stack, vec![ParserState::Starting]);
        assert_eq!(configuration.name, "a".to_string());

        let mut config_parser = ConfigParser::new();
        let mut configuration = Configuration::default();
        config_parser.state_stack.push(ParserState::Name);
        configuration.name = "a".to_string();

        config_parser.handle_transition(&mut configuration, '#', ParserState::Name, ParserState::Comment);

        assert_eq!(config_parser.state_stack, vec![ParserState::Starting, ParserState::Comment]);
        assert_eq!(configuration.name, "a".to_string());

        let mut config_parser = ConfigParser::new();
        let mut configuration = Configuration::default();
        config_parser.state_stack.push(ParserState::Name);
        configuration.name = "a".to_string();

        config_parser.handle_transition(&mut configuration, '{', ParserState::Name, ParserState::StartBlock);

        assert_eq!(config_parser.state_stack, vec![ParserState::Starting, ParserState::StartBlock]);
        assert_eq!(configuration.name, "a".to_string());
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
        assert_eq!(configuration.directives.last().unwrap().raw, "a".to_string());

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

    #[test]
    fn test_basic_parsing() -> Result<(), Box<dyn std::error::Error>> {
        let mut config_parser = ConfigParser::new();
        let configuration = config_parser.parse_content(r#"fordo {
            # foo the bar
            bobo;
            coco="dodo";
        }"#)?;

        assert_eq!(configuration.name, "fordo".to_string());
        assert_eq!(configuration.directives.len(), 2);
        assert_eq!(configuration.directives[0].raw, "bobo".to_string());
        assert_eq!(configuration.directives[1].raw, "coco=\"dodo\"".to_string());

        Ok(())
    }
}