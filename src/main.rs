use std::io::Write;

#[derive(Debug)]
pub enum Node {
    Tag {
        name: String,
        contents: Option<Vec<Node>>,
    },
    Text(String),
}

#[derive(Debug)]
pub enum ParsingResult<'a, T, E> {
    Ok {
        rest: &'a str,
        value: T,
        next_error: E,
    },
    Err(E),
}

mod text {
    use crate::ParsingResult;

    enum Character {
        Colon,
        OpeningBracket,
        ClosingBracket,
        Escape,
        Other(char),
    }

    impl From<char> for Character {
        fn from(value: char) -> Self {
            match value {
                ':' => Self::Colon,
                '[' => Self::OpeningBracket,
                ']' => Self::ClosingBracket,
                '\\' => Self::Escape,
                _ => Self::Other(value),
            }
        }
    }

    impl From<Character> for char {
        fn from(value: Character) -> Self {
            match value {
                Character::Escape => '\\',
                Character::OpeningBracket => '[',
                Character::ClosingBracket => ']',
                Character::Colon => ':',
                Character::Other(c) => c,
            }
        }
    }

    #[derive(Debug)]
    pub enum ParsingError {
        UnknownCharacterEscaped,
        EscapeCharacterIsAtTheEndOfTheString,
        StringIsEmpty,
        StringStartsWithAnOpeningBracket,
        StringStartsWithAClosedBracket,
        StringStartsWithAColon,
    }

    pub fn parse_text(input: &str) -> ParsingResult<'_, String, ParsingError> {
        fn ok(
            mut text: String,
            rest: &str,
            next_error: ParsingError,
        ) -> ParsingResult<String, ParsingError> {
            text.shrink_to_fit();
            ParsingResult::Ok {
                rest,
                value: text,
                next_error,
            }
        }

        let mut char_indices = input
            .char_indices()
            .map(|(index, character)| (index, character.into()));
        let mut text = String::from(match char_indices.next() {
            None => return ParsingResult::Err(ParsingError::StringIsEmpty),
            Some((_index, character)) => match character {
                Character::OpeningBracket => {
                    return ParsingResult::Err(ParsingError::StringStartsWithAnOpeningBracket)
                }
                Character::ClosingBracket => {
                    return ParsingResult::Err(ParsingError::StringStartsWithAClosedBracket)
                }
                Character::Colon => {
                    return ParsingResult::Err(ParsingError::StringStartsWithAColon)
                }
                Character::Escape => match char_indices.next() {
                    None => {
                        return ParsingResult::Err(
                            ParsingError::EscapeCharacterIsAtTheEndOfTheString,
                        )
                    }
                    Some((_index, character)) => match character {
                        Character::OpeningBracket
                        | Character::ClosingBracket
                        | Character::Colon
                        | Character::Escape => character.into(),
                        Character::Other(_character) => {
                            return ParsingResult::Err(ParsingError::UnknownCharacterEscaped)
                        }
                    },
                },
                Character::Other(character) => character,
            },
        });

        loop {
            text.push(match char_indices.next() {
                None => return ok(text, "", ParsingError::StringIsEmpty),
                Some((index, Character::ClosingBracket)) => {
                    return ok(
                        text,
                        &input[index..],
                        ParsingError::StringStartsWithAClosedBracket,
                    )
                }
                Some((index, Character::OpeningBracket)) => {
                    return ok(
                        text,
                        &input[index..],
                        ParsingError::StringStartsWithAnOpeningBracket,
                    )
                }
                Some((index, Character::Colon)) => {
                    return ok(text, &input[index..], ParsingError::StringStartsWithAColon)
                }
                Some((_index, Character::Escape)) => match char_indices.next() {
                    None => {
                        return ParsingResult::Err(
                            ParsingError::EscapeCharacterIsAtTheEndOfTheString,
                        )
                    }
                    Some((
                        _index,
                        character @ (Character::ClosingBracket
                        | Character::OpeningBracket
                        | Character::Colon
                        | Character::Escape),
                    )) => character.into(),
                    Some((_index, Character::Other(_character))) => {
                        return ParsingResult::Err(ParsingError::UnknownCharacterEscaped)
                    }
                },
                Some((_index, Character::Other(character))) => character,
            });
        }
    }
}

#[derive(Debug)]
pub enum ParsingError {
    UnknownCharacterEscaped,
    UnclosedBracket,
    EscapeCharacterIsAtTheEndOfTheString,
    UnpairedClosingBracket,
    AColonIsNotPlacedImmediatelyAfterTagName,
    TagInsideTagName,
}

pub fn parse_sequential_nodes(mut input: &str) -> Result<Vec<Node>, ParsingError> {
    #[derive(Debug)]
    enum ProcessingResult {
        ParsingIsDone,
        Error(ParsingError),
        NewNode(Node),
    }

    fn process_error(error: text::ParsingError, input: &mut &str) -> ProcessingResult {
        println!("{:?}, {:?}", error, input);
        match error {
            text::ParsingError::StringStartsWithAColon => {
                return ProcessingResult::Error(
                    ParsingError::AColonIsNotPlacedImmediatelyAfterTagName,
                )
            }
            text::ParsingError::StringIsEmpty => ProcessingResult::ParsingIsDone,
            text::ParsingError::StringStartsWithAClosedBracket => {
                return ProcessingResult::Error(ParsingError::UnpairedClosingBracket)
            }
            text::ParsingError::UnknownCharacterEscaped => {
                return ProcessingResult::Error(ParsingError::UnknownCharacterEscaped)
            }
            text::ParsingError::EscapeCharacterIsAtTheEndOfTheString => {
                return ProcessingResult::Error(ParsingError::EscapeCharacterIsAtTheEndOfTheString)
            }
            text::ParsingError::StringStartsWithAnOpeningBracket => {
                let mut chars = input.chars();
                chars.next();
                *input = chars.as_str();
                match text::parse_text(input) {
                    ParsingResult::Ok {
                        rest,
                        value: name,
                        next_error,
                    } => {
                        *input = rest;
                        match next_error {
                            text::ParsingError::StringStartsWithAnOpeningBracket => {
                                ProcessingResult::Error(ParsingError::TagInsideTagName)
                            }
                            text::ParsingError::EscapeCharacterIsAtTheEndOfTheString => {
                                ProcessingResult::Error(
                                    ParsingError::EscapeCharacterIsAtTheEndOfTheString,
                                )
                            }
                            text::ParsingError::UnknownCharacterEscaped => {
                                ProcessingResult::Error(ParsingError::UnknownCharacterEscaped)
                            }
                            text::ParsingError::StringIsEmpty => {
                                ProcessingResult::Error(ParsingError::UnclosedBracket)
                            }
                            text::ParsingError::StringStartsWithAColon => {
                                let contents = match parse_sequential_nodes(input) {
                                    Ok(contents) => contents,
                                    Err(error) => return ProcessingResult::Error(error),
                                };
                                ProcessingResult::NewNode(Node::Tag {
                                    name,
                                    contents: Some(contents),
                                })
                            }
                            text::ParsingError::StringStartsWithAClosedBracket => {
                                ProcessingResult::NewNode(Node::Tag {
                                    name,
                                    contents: None,
                                })
                            }
                        }
                    }
                    ParsingResult::Err(error) => match error {
                        text::ParsingError::StringStartsWithAColon => {
                            let contents = match parse_sequential_nodes(input) {
                                Ok(contents) => contents,
                                Err(error) => return ProcessingResult::Error(error),
                            };
                            ProcessingResult::NewNode(Node::Tag {
                                name: String::new(),
                                contents: Some(contents),
                            })
                        }
                        text::ParsingError::StringIsEmpty => {
                            ProcessingResult::Error(ParsingError::UnclosedBracket)
                        }
                        text::ParsingError::StringStartsWithAClosedBracket => {
                            ProcessingResult::NewNode(Node::Tag {
                                name: String::new(),
                                contents: None,
                            })
                        }
                        text::ParsingError::UnknownCharacterEscaped => {
                            ProcessingResult::Error(ParsingError::UnknownCharacterEscaped)
                        }
                        text::ParsingError::EscapeCharacterIsAtTheEndOfTheString => {
                            ProcessingResult::Error(
                                ParsingError::EscapeCharacterIsAtTheEndOfTheString,
                            )
                        }
                        text::ParsingError::StringStartsWithAnOpeningBracket => {
                            ProcessingResult::Error(ParsingError::TagInsideTagName)
                        }
                    },
                }
            }
        }
    }

    let mut nodes = Vec::new();
    loop {
        match text::parse_text(input) {
            ParsingResult::Ok {
                rest,
                value,
                next_error,
            } => {
                input = rest;
                nodes.push(Node::Text(value));
                match process_error(next_error, &mut input) {
                    ProcessingResult::ParsingIsDone => return Ok(nodes),
                    ProcessingResult::NewNode(node) => nodes.push(node),
                    ProcessingResult::Error(error) => return Err(error),
                }
            }
            ParsingResult::Err(error) => match process_error(error, &mut input) {
                ProcessingResult::ParsingIsDone => return Ok(nodes),
                ProcessingResult::NewNode(node) => nodes.push(node),
                ProcessingResult::Error(error) => return Err(error),
            },
        };
    }
}

fn main() {
    loop {
        let mut line = String::new();
        print!("> ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut line).unwrap();
        println!(
            "{:?}",
            parse_sequential_nodes(line.strip_suffix('\n').unwrap())
        );
    }
}
