use std::collections::HashMap;

use thiserror::Error;

pub fn fields<'a>(iter: impl Iterator<Item = &'a Node>) -> HashMap<&'a str, &'a [Node]> {
    let mut fields = HashMap::new();
    for sequence in sequences(iter) {
        let mut iter = sequence.iter();
        if let Some(Node::Text(text)) = iter.next() {
            fields.insert(text.as_str(), iter.as_slice());
        }
    }
    fields
}

pub fn texts<'a>(iter: impl Iterator<Item = &'a Node>) -> impl Iterator<Item = &'a str> {
    iter.filter_map(|node| {
        if let Node::Text(text) = node {
            Some(text.as_str())
        } else {
            None
        }
    })
}

pub fn sequences<'a>(iter: impl Iterator<Item = &'a Node>) -> impl Iterator<Item = &'a Vec<Node>> {
    iter.filter_map(|node| {
        if let Node::Sequence(sequence) = node {
            Some(sequence)
        } else {
            None
        }
    })
}

#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    Sequence(Vec<Node>),
    Text(String),
}

impl Node {
    pub fn sequence(&self) -> Option<&Vec<Self>> {
        if let Self::Sequence(sequence) = self {
            Some(sequence)
        } else {
            None
        }
    }

    pub fn text(&self) -> Option<&String> {
        if let Self::Text(text) = self {
            Some(text)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum Error {
    #[error("there was a escaping character at the end of the input string")]
    EscapeAtTheEndOfInput,
    #[error("a character that is not escapable escaped at {pos:?}")]
    UnknownCharacterEscaped { pos: parco::Position },
    #[error("a bracket was not closed. The opening bracket is at {pos:?}")]
    UnclosedBracket { pos: parco::Position },
    #[error("an unexpected closing bracket (it does not have a corresponding opening bracket) appeared at {pos:?}")]
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

pub fn parse_sequential_nodes(input: &str) -> Result<Vec<Node>, Error> {
    match sequence_of_nodes::parse(input.into()) {
        ParsingResult::Ok((nodes, rest)) => {
            if rest.0.src().is_empty() {
                Ok(nodes)
            } else {
                Err(Error::UnexpectedClosingBracket { pos: rest.0.pos() })
            }
        }
        ParsingResult::Err => Ok(vec![]),
        ParsingResult::Fatal(e) => Err(match e {
            FatalError::UnclosedBracket { pos } => Error::UnclosedBracket { pos },
            FatalError::EscapeAtTheEndOfInput => Error::EscapeAtTheEndOfInput,
            FatalError::UnknownCharacterEscaped { pos } => Error::UnknownCharacterEscaped { pos },
        }),
    }
}

type ParsingResult<'s, T> = parco::Result<T, parco::PositionedString<'s>, FatalError>;

enum FatalError {
    EscapeAtTheEndOfInput,
    UnknownCharacterEscaped { pos: parco::Position },
    UnclosedBracket { pos: parco::Position },
}

impl From<FatalError> for Error {
    fn from(e: FatalError) -> Self {
        match e {
            FatalError::UnclosedBracket { pos } => Self::UnclosedBracket { pos },
            FatalError::EscapeAtTheEndOfInput => Self::EscapeAtTheEndOfInput,
            FatalError::UnknownCharacterEscaped { pos } => Self::UnknownCharacterEscaped { pos },
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
                .or(|| parco::Result::Fatal(FatalError::EscapeAtTheEndOfInput)),
            '|' => parco::Result::Ok((Character::VerticalBar, input)),
            _ => parco::Result::Ok((Character::Other(c), input)),
        })
    }
}

mod text {
    use super::*;

    pub(crate) fn parse(mut input: parco::PositionedString) -> ParsingResult<String> {
        let mut result = String::new();
        loop {
            match text_character::parse(input) {
                parco::Result::Ok((c, rest)) => {
                    input = rest.0;
                    match c {
                        text_character::Character::VerticalBar => {
                            result.shrink_to_fit();
                            return parco::Result::Ok((result, parco::Rest(input)));
                        }
                        text_character::Character::Other(c) => {
                            result.push(c);
                        }
                    }
                }
                parco::Result::Err => {
                    if result.is_empty() {
                        return parco::Result::Err;
                    }
                    result.shrink_to_fit();
                    return parco::Result::Ok((result, parco::Rest(input)));
                }
                parco::Result::Fatal(e) => return parco::Result::Fatal(e),
            }
        }
    }
}

mod sequence_of_nodes {
    use super::*;

    pub(crate) fn parse(mut input: parco::PositionedString) -> ParsingResult<Vec<Node>> {
        let mut nodes = Vec::new();
        loop {
            match node::parse(input) {
                parco::Result::Err => {
                    nodes.shrink_to_fit();
                    return parco::Result::Ok((nodes, parco::Rest(input)));
                }
                parco::Result::Ok((node, rest)) => {
                    input = rest.0;
                    nodes.push(node);
                }
                parco::Result::Fatal(e) => return parco::Result::Fatal(e),
            }
        }
    }
}

mod node {
    use super::*;

    pub(crate) fn parse(input: parco::PositionedString) -> ParsingResult<Node> {
        text::parse(input).map(|text| Node::Text(text)).or(|| {
            parco::one_matching_part(input, |c| *c == '[').and(|(_c, rest)| {
                parco::one_matching_part(rest.0, |c| *c == ']')
                    .map(|_c| Node::Sequence(vec![]))
                    .or(|| {
                        sequence_of_nodes::parse(rest.0).and(|(nodes, input)| {
                            parco::one_matching_part(input.0, |c| *c == ']')
                                .map(|_c| Node::Sequence(nodes))
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

    fn text(s: &'static str) -> Node {
        Node::Text(String::from(s))
    }
    fn seq(s: Vec<Node>) -> Node {
        Node::Sequence(s)
    }

    #[test]
    fn test_parsing() {
        assert_eq!(
            parse_sequential_nodes(
                r#"text[first sequence item|second sequence item[subsequence]|third sequence item[]with an empty in-between sequence]"#
            ),
            Ok(vec![
                text("text"),
                seq(vec![
                    text("first sequence item"),
                    text("second sequence item"),
                    seq(vec![text("subsequence")]),
                    text(""),
                    text("third sequence item"),
                    seq(vec![]),
                    text("with an empty in-between sequence")
                ])
            ])
        );
    }

    #[test]
    fn test_escaping() {
        assert_eq!(
            parse_sequential_nodes(r#"[some vertical bars: \|\|\|][some brackets: \]\[\[\]]"#),
            Ok(vec![
                seq(vec![text(r#"some vertical bars: |||"#)]),
                seq(vec![text(r#"some brackets: ][[]"#)])
            ])
        );
    }

    #[test]
    fn test_parsing_two_text_nodes() {
        assert_eq!(
            parse_sequential_nodes(r#"first text node|second text node"#),
            Ok(vec![text("first text node"), text("second text node")])
        );
    }

    #[test]
    fn test_parsing_complicated_input() {
        assert_eq!(
            parse_sequential_nodes(r#"[|a[]]|[a||b]"#),
            Ok(vec![
                seq(vec![text(""), text("a"), seq(vec![])]),
                text(""),
                seq(vec![text("a"), text(""), text("b")])
            ])
        );
    }

    #[test]
    fn test_parsing_empty_input() {
        assert_eq!(parse_sequential_nodes(r#""#), Ok(vec![]));
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
                text("a"),
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
            Err(Error::EscapeAtTheEndOfInput)
        );
    }

    #[test]
    fn test_escape_at_the_end_of_input_detection() {
        assert_eq!(
            parse_sequential_nodes(r#"abc\"#),
            Err(Error::EscapeAtTheEndOfInput)
        );
    }

    #[test]
    fn test_parsing_only_a_text_terminator() {
        assert_eq!(parse_sequential_nodes(r#"|"#), Ok(vec![text("")]));
    }

    #[test]
    fn test_unknown_character_escape_detection() {
        assert_eq!(
            parse_sequential_nodes(r#"a\b"#),
            Err(Error::UnknownCharacterEscaped {
                pos: parco::Position { col: 3, row: 1 }
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
                pos: parco::Position { col: 2, row: 2 }
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
