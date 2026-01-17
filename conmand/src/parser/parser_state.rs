
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


    }
}