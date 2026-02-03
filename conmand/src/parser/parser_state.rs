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

/// The current parser state (really a tokenizer).
///
/// * Starting - The initial state
/// * Name - Indicates we're reading the configuration name.
/// * StartBlock - Indicates we've started reading a block.
/// * EndBlock - We've completed reading the configuration block.
/// * Seeking - We're seeking the next directive.
/// * Comment - We're reading comment text.
/// * InDirective - We're reading a configuration directive.
/// * Invalid - An illegal state transition.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ParserState {
    Starting,
    Name,
    StartBlock,
    EndBlock,
    Seeking,
    Comment,
    InDirective,
    Invalid,
}

impl ParserState {
    /// Given a state and a token, returns the next state
    ///
    /// ## State Transition Table
    ///
    /// | From State   | Input         | Next State   |
    /// |--------------|---------------|--------------|
    /// | Starting     | whitespace    | Starting     |
    /// | Starting     | alpha-numeric | Name         |
    /// | Starting     | '{'           | Start Block  |
    /// | Starting     | '#'           | Comment      |
    /// | Name         | alpha-numeric | Name         |
    /// | Name         | whitespace    | Starting     |
    /// | Name         | '{'           | Start Block  |
    /// | Name         | '#'           | Comment      |
    /// | Start Block  | whitespace    | Seeking      |
    /// | Start Block  | alpha-numeric | In Directive |
    /// | Start Block  | '#'           | Comment      |
    /// | Start Bock   | '}'           | End Block    |
    /// | End Block    | whitespace    | End Block    |
    /// | End Block    | '#'           | Comment      |
    /// | Seeking      | whitespace    | Seeking      |
    /// | Seeking      | '}'           | End Block    |
    /// | Seeking      | '#'           | Comment      |
    /// | Seeking      | alpha-numeric | In Directive |
    /// | Comment      | '\n'          | Comment      |
    /// | Comment      | .             | Comment      |
    /// | In Directive | ';'           | Seeking      |
    /// | In Directive | '#'           | Comment      |
    /// | In Directive | .             | In Directive |
    pub fn next_state(&self, token: char) -> ParserState {
        match self {
            ParserState::Starting => {
                if token.is_ascii_whitespace() {
                    ParserState::Starting
                } else if token.is_ascii_alphanumeric() {
                    ParserState::Name
                } else if token == '{' {
                    ParserState::StartBlock
                } else if token == '#' {
                    ParserState::Comment
                } else {
                    ParserState::Invalid
                }
            },
            ParserState::Name => {
                if token.is_alphanumeric() {
                    ParserState::Name
                } else if token.is_ascii_whitespace() {
                    ParserState::Starting
                } else if token == '#' {
                    ParserState::Comment
                } else if token == '{' {
                    ParserState::StartBlock
                } else {
                    ParserState::Invalid
                }
            }
            ParserState::StartBlock => {
                if token.is_ascii_whitespace() {
                    ParserState::Seeking
                } else if token.is_alphabetic() {
                    ParserState::InDirective
                } else if token == '#' {
                    ParserState::Comment
                } else if token == '}' {
                    ParserState::EndBlock
                } else {
                    ParserState::Invalid
                }
            },
            ParserState::EndBlock => {
                if token.is_ascii_whitespace() {
                    ParserState::EndBlock
                } else if token == '#' {
                    ParserState::Comment
                } else {
                    ParserState::Invalid
                }
            },
            ParserState::Seeking => {
                if token.is_ascii_whitespace() {
                    ParserState::Seeking
                } else if token == '#' {
                    ParserState::Comment
                } else if token.is_ascii_alphanumeric() {
                    ParserState::InDirective
                } else if token == '}' {
                    ParserState::EndBlock
                } else {
                    ParserState::Invalid
                }
            },
            ParserState::Comment => {
                if token == '\n' {
                    ParserState::Seeking
                } else {
                    ParserState::Comment
                }
            },
            ParserState::InDirective => {
                if token == ';' {
                    ParserState::Seeking
                } else if token == '#' {
                    ParserState::Comment
                } else {
                    ParserState::InDirective
                }
            },
            ParserState::Invalid => {
                ParserState::Invalid
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_start_state() {
        let current_state = ParserState::Starting;
        let next_state = current_state.next_state(' ');
        assert_eq!(next_state, ParserState::Starting);

        let next_state = current_state.next_state('\n');
        assert_eq!(next_state, ParserState::Starting);

        let next_state = current_state.next_state('a');
        assert_eq!(next_state, ParserState::Name);

        let next_state = current_state.next_state('{');
        assert_eq!(next_state, ParserState::StartBlock);

        let next_state = current_state.next_state('#');
        assert_eq!(next_state, ParserState::Comment);

        let next_state = current_state.next_state('.');
        assert_eq!(next_state, ParserState::Invalid);
    }

    #[test]
    fn test_name_state() {
        let current_state = ParserState::Name;
        let next_state = current_state.next_state('a');
        assert_eq!(next_state, ParserState::Name);

        let next_state = current_state.next_state(' ');
        assert_eq!(next_state, ParserState::Starting);

        let next_state = current_state.next_state('#');
        assert_eq!(next_state, ParserState::Comment);

        let next_state = current_state.next_state('{');
        assert_eq!(next_state, ParserState::StartBlock);
    }

    #[test]
    fn test_start_block() {
        let current_state = ParserState::StartBlock;
        let next_state = current_state.next_state(' ');
        assert_eq!(next_state, ParserState::Seeking);
        
        let next_state = current_state.next_state('#');
        assert_eq!(next_state, ParserState::Comment);

        let next_state = current_state.next_state('a');
        assert_eq!(next_state, ParserState::InDirective);

        let next_state = current_state.next_state('}');
        assert_eq!(next_state, ParserState::EndBlock);
    }

    #[test]
    fn test_end_block() {
        let current_state = ParserState::EndBlock;
        let next_state = current_state.next_state('{');
        assert_eq!(next_state, ParserState::Invalid);

        let next_state = current_state.next_state(' ');
        assert_eq!(next_state, ParserState::EndBlock);

        let next_state = current_state.next_state('a');
        assert_eq!(next_state, ParserState::Invalid);

        let next_state = current_state.next_state('#');
        assert_eq!(next_state, ParserState::Comment);
    }

    #[test]
    fn test_seeking() {
        let current_state = ParserState::Seeking;
        let next_state = current_state.next_state(' ');
        assert_eq!(next_state, ParserState::Seeking);

        let next_state = current_state.next_state('#');
        assert_eq!(next_state, ParserState::Comment);

        let next_state = current_state.next_state('a');
        assert_eq!(next_state, ParserState::InDirective);

        let next_state = current_state.next_state('}');
        assert_eq!(next_state, ParserState::EndBlock);

        let next_state = current_state.next_state('{');
        assert_eq!(next_state, ParserState::Invalid);
    }

    #[test]
    fn test_comment() {
        let current_state = ParserState::Comment;
        let next_state = current_state.next_state(' ');
        assert_eq!(next_state, ParserState::Comment);

        let next_state = current_state.next_state('#');
        assert_eq!(next_state, ParserState::Comment);

        let next_state = current_state.next_state('a');
        assert_eq!(next_state, ParserState::Comment);

        let next_state = current_state.next_state(';');
        assert_eq!(next_state, ParserState::Comment);

        let next_state = current_state.next_state('\n');
        assert_eq!(next_state, ParserState::Seeking);
    }

    #[test]
    fn test_in_directive() {
        let current_state = ParserState::InDirective;
        let next_state = current_state.next_state(' ');
        assert_eq!(next_state, ParserState::InDirective);

        let next_state = current_state.next_state('#');
        assert_eq!(next_state, ParserState::Comment);

        let next_state = current_state.next_state('{');
        assert_eq!(next_state, ParserState::InDirective);

        let next_state = current_state.next_state('a');
        assert_eq!(next_state, ParserState::InDirective);

        let next_state = current_state.next_state('}');
        assert_eq!(next_state, ParserState::InDirective);

        let next_state = current_state.next_state(';');
        assert_eq!(next_state, ParserState::Seeking);
    }

    #[test]
    fn test_invalid() {
        let current_state = ParserState::Invalid;
        let next_state = current_state.next_state(' ');
        assert_eq!(next_state, ParserState::Invalid);

        let next_state = current_state.next_state('#');
        assert_eq!(next_state, ParserState::Invalid);

        let next_state = current_state.next_state('{');
        assert_eq!(next_state, ParserState::Invalid);

        let next_state = current_state.next_state('}');
        assert_eq!(next_state, ParserState::Invalid);

        let next_state = current_state.next_state('a');
        assert_eq!(next_state, ParserState::Invalid);
    }
}