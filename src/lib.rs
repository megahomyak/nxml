use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct Text {
    pub content: String,
    /// Ended with a vertical bar (`|`).
    pub ended_explicitly: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Sequence {
    pub contents: Vec<Node>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    Sequence(Sequence),
    Text(Text),
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum Error {
    #[error("there was an escaping character at the end of the input string (the string ends at {input_end:?})")]
    EscapeAtTheEndOfInput { input_end: parco::Position },
    #[error("a character that is not escapable escaped at {character_position:?}")]
    UnknownCharacterEscaped { character_position: parco::Position },
    #[error("a bracket is not closed. The opening bracket is at {pos:?}")]
    UnclosedBracket { pos: parco::Position },
    #[error("an unexpected closing bracket (it has no corresponding opening bracket) appeared at {pos:?}")]
    UnexpectedClosingBracket { pos: parco::Position },
}

pub fn parse_one_node(
    input: &str,
) -> Result<Option<(Node, parco::Rest<(&str, parco::Position)>)>, Error> {
    match node::parse(input.into()) {
        ParsingResult::Ok((node, rest)) => {
            Ok(Some((node, parco::Rest((rest.0.src(), rest.0.pos())))))
        }
        ParsingResult::Err => Ok(None),
        ParsingResult::Fatal(e) => Err(e.into()),
    }
}

pub fn parse_sequential_nodes(input: &str) -> Result<Sequence, Error> {
    match sequence_of_nodes::parse(input.into()) {
        ParsingResult::Ok((nodes, rest)) => {
            if rest.0.src().is_empty() {
                Ok(nodes)
            } else {
                Err(Error::UnexpectedClosingBracket { pos: rest.0.pos() })
            }
        }
        ParsingResult::Err => Ok(Sequence {
            contents: Vec::new(),
        }),
        ParsingResult::Fatal(e) => Err(e.into()),
    }
}

type ParsingResult<'s, T> = parco::Result<T, parco::PositionedString<'s>, FatalError>;

enum FatalError {
    EscapeAtTheEndOfInput { pos: parco::Position },
    UnknownCharacterEscaped { pos: parco::Position },
    UnclosedBracket { pos: parco::Position },
}

impl From<FatalError> for Error {
    fn from(e: FatalError) -> Self {
        match e {
            FatalError::UnclosedBracket { pos } => Self::UnclosedBracket { pos },
            FatalError::EscapeAtTheEndOfInput { pos } => Self::EscapeAtTheEndOfInput { input_end: pos },
            FatalError::UnknownCharacterEscaped { pos } => Self::UnknownCharacterEscaped { character_position: pos },
        }
    }
}

mod text_character {
    use super::*;

    pub enum Character {
        VerticalBar,
        Other(char),
    }

    pub(crate) fn parse(input: parco::PositionedString) -> ParsingResult<Character> {
        parco::one_part(input).and(|(c, input)| match c {
            '[' | ']' => parco::Result::Err,
            '\\' => parco::one_part(input.0)
                .and(|(c, rest)| {
                    if "\\|[]".contains(c) {
                        parco::Result::Ok((Character::Other(c), rest))
                    } else {
                        parco::Result::Fatal(FatalError::UnknownCharacterEscaped {
                            pos: input.0.pos(),
                        })
                    }
                })
                .or(|| {
                    parco::Result::Fatal(FatalError::EscapeAtTheEndOfInput { pos: input.0.pos() })
                }),
            '|' => parco::Result::Ok((Character::VerticalBar, input)),
            _ => parco::Result::Ok((Character::Other(c), input)),
        })
    }
}

mod text {
    use super::*;

    pub(crate) fn parse(mut input: parco::PositionedString) -> ParsingResult<Text> {
        let mut content = String::new();
        loop {
            match text_character::parse(input) {
                parco::Result::Ok((c, rest)) => {
                    input = rest.0;
                    match c {
                        text_character::Character::VerticalBar => {
                            content.shrink_to_fit();
                            return parco::Result::Ok((
                                Text {
                                    content,
                                    ended_explicitly: true,
                                },
                                parco::Rest(input),
                            ));
                        }
                        text_character::Character::Other(c) => {
                            content.push(c);
                        }
                    }
                }
                parco::Result::Err => {
                    if content.is_empty() {
                        return parco::Result::Err;
                    }
                    content.shrink_to_fit();
                    return parco::Result::Ok((
                        Text {
                            content,
                            ended_explicitly: false,
                        },
                        parco::Rest(input),
                    ));
                }
                parco::Result::Fatal(e) => return parco::Result::Fatal(e),
            }
        }
    }
}

mod sequence_of_nodes {
    use super::*;

    pub(crate) fn parse(input: parco::PositionedString) -> ParsingResult<Sequence> {
        let result: ParsingResult<Vec<_>> =
            parco::collect_repeating(input, |input| node::parse(*input)).into();
        result.map(|mut contents| {
            contents.shrink_to_fit();
            Sequence { contents }
        })
    }
}

mod node {
    use super::*;

    pub(crate) fn parse(input: parco::PositionedString) -> ParsingResult<Node> {
        text::parse(input).map(|text| Node::Text(text)).or(|| {
            parco::one_matching_part(input, |c| *c == '[').and(|(_c, rest)| {
                parco::one_matching_part(rest.0, |c| *c == ']')
                    .map(|_c| {
                        Node::Sequence(Sequence {
                            contents: Vec::new(),
                        })
                    })
                    .or(|| {
                        sequence_of_nodes::parse(rest.0).and(|(sequence, input)| {
                            parco::one_matching_part(input.0, |c| *c == ']')
                                .map(|_c| Node::Sequence(sequence))
                        })
                    })
                    .or(|| parco::Result::Fatal(FatalError::UnclosedBracket { pos: input.pos() }))
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(s: &'static str, ended_explicitly: bool) -> Node {
        Node::Text(Text {
            content: String::from(s),
            ended_explicitly,
        })
    }
    fn seq(contents: Vec<Node>) -> Node {
        Node::Sequence(Sequence { contents })
    }

    #[test]
    fn test_parsing() {
        assert_eq!(
            parse_sequential_nodes(
                r#"text[first sequence item|second sequence item[subsequence]|third sequence item[]with an empty in-between sequence]"#
            ),
            Ok(Sequence {
                contents: vec![
                    text("text", false),
                    seq(vec![
                        text("first sequence item", true),
                        text("second sequence item", false),
                        seq(vec![text("subsequence", false)]),
                        text("", true),
                        text("third sequence item", false),
                        seq(vec![]),
                        text("with an empty in-between sequence", false)
                    ])
                ]
            })
        );
    }

    #[test]
    fn test_escaping() {
        assert_eq!(
            parse_sequential_nodes(
                r#"[some vertical bars: \|\|\|][some brackets: \]\[\[\]][some backslashes: \\ \\ \\]"#
            ),
            Ok(Sequence {
                contents: vec![
                    seq(vec![text(r#"some vertical bars: |||"#, false)]),
                    seq(vec![text(r#"some brackets: ][[]"#, false)]),
                    seq(vec![text(r#"some backslashes: \ \ \"#, false)]),
                ]
            })
        );
    }

    #[test]
    fn test_parsing_two_text_nodes() {
        assert_eq!(
            parse_sequential_nodes(r#"first text node|second text node"#),
            Ok(Sequence {
                contents: vec![
                    text("first text node", true),
                    text("second text node", false)
                ]
            })
        );
    }

    #[test]
    fn test_parsing_complicated_input() {
        assert_eq!(
            parse_sequential_nodes(r#"[|a[]]|[a||b]"#),
            Ok(Sequence {
                contents: vec![
                    seq(vec![text("", true), text("a", false), seq(vec![])]),
                    text("", true),
                    seq(vec![text("a", true), text("", true), text("b", false)])
                ]
            })
        );
    }

    #[test]
    fn test_parsing_empty_input() {
        assert_eq!(
            parse_sequential_nodes(r#""#),
            Ok(Sequence { contents: vec![] })
        );
    }

    #[test]
    fn test_unexpected_closing_bracket_detection() {
        assert_eq!(
            parse_sequential_nodes(r#"abc]"#),
            Err(Error::UnexpectedClosingBracket {
                pos: parco::Position { col: 4, row: 1 }
            })
        );
    }

    #[test]
    fn test_immediate_unexpected_closing_bracket_detection() {
        assert_eq!(
            parse_sequential_nodes(r#"]"#),
            Err(Error::UnexpectedClosingBracket {
                pos: parco::Position { col: 1, row: 1 }
            })
        );
    }

    #[test]
    fn test_parsing_a_single_node_with_incorrect_syntax_after_the_node() {
        assert_eq!(
            parse_one_node(r#"a["#),
            Ok(Some((
                text("a", false),
                parco::Rest(("[", parco::Position { row: 1, col: 2 }))
            )))
        );
    }

    #[test]
    fn test_parsing_empty_input_as_one_node() {
        assert_eq!(parse_one_node(r#""#), Ok(None));
    }

    #[test]
    fn test_immediate_escape_at_the_end_of_input_detection() {
        assert_eq!(
            parse_sequential_nodes(r#"\"#),
            Err(Error::EscapeAtTheEndOfInput {
                input_end: parco::Position { col: 2, row: 1 }
            })
        );
    }

    #[test]
    fn test_escape_at_the_end_of_input_detection() {
        assert_eq!(
            parse_sequential_nodes(r#"abc\"#),
            Err(Error::EscapeAtTheEndOfInput {
                input_end: parco::Position { col: 5, row: 1 }
            })
        );
    }

    #[test]
    fn test_parsing_only_a_text_terminator() {
        assert_eq!(
            parse_sequential_nodes(r#"|"#),
            Ok(Sequence {
                contents: vec![text("", true)]
            })
        );
    }

    #[test]
    fn test_unknown_character_escape_detection() {
        assert_eq!(
            parse_sequential_nodes(r#"a\b"#),
            Err(Error::UnknownCharacterEscaped {
                character_position: parco::Position { col: 3, row: 1 }
            })
        );
    }

    #[test]
    fn test_immediate_unclosed_bracket_detection_with_one_node_parsing() {
        assert_eq!(
            parse_one_node(r#"[abcd"#),
            Err(Error::UnclosedBracket {
                pos: parco::Position { col: 1, row: 1 }
            })
        );
    }

    #[test]
    fn test_unclosed_bracket_detection_after_a_new_line() {
        assert_eq!(
            parse_sequential_nodes("\n[abcd"),
            Err(Error::UnclosedBracket {
                pos: parco::Position { col: 1, row: 2 }
            })
        );
    }

    #[test]
    fn test_unknown_character_escape_detection_after_a_new_line() {
        assert_eq!(
            parse_sequential_nodes("\n\\a"),
            Err(Error::UnknownCharacterEscaped {
                character_position: parco::Position { col: 2, row: 2 }
            })
        );
    }

    #[test]
    fn test_unclosed_bracket_detection() {
        assert_eq!(
            parse_sequential_nodes(r#"a[bcd"#),
            Err(Error::UnclosedBracket {
                pos: parco::Position { col: 2, row: 1 }
            })
        );
    }
}
