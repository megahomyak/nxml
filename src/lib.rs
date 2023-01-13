#[derive(Debug, PartialEq)]
pub enum Node {
    Sequence(Vec<Node>),
    Text(String),
}

#[derive(Debug, PartialEq)]
pub struct Rest<T>(T);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Position {
    pub column: usize,
    pub row: usize,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    EscapeAtTheEndOfInput,
    UnknownCharacterEscaped { pos: Position },
    UnclosedBracket { pos: Position },
    UnexpectedClosingBracket { pos: Position },
}

pub fn parse_one_node(input: &str) -> Result<Option<(Node, Rest<(&str, Position)>)>, Error> {
    match inner::node::parse(input.into()) {
        inner::ParsingResult::Ok((node, rest)) => Ok(Some((node, Rest((rest.0.s, rest.0.pos))))),
        inner::ParsingResult::Err => Ok(None),
        inner::ParsingResult::Fatal(e) => Err(e.into()),
    }
}

pub fn parse_sequential_nodes(input: &str) -> Result<Vec<Node>, Error> {
    match inner::sequence_of_nodes::parse(input.into()) {
        inner::ParsingResult::Ok((nodes, rest)) => {
            if rest.0.s.is_empty() {
                Ok(nodes)
            } else {
                Err(Error::UnexpectedClosingBracket { pos: rest.0.pos })
            }
        }
        inner::ParsingResult::Err => Ok(vec![]),
        inner::ParsingResult::Fatal(e) => Err(match e {
            inner::FatalError::UnclosedBracket { pos } => Error::UnclosedBracket { pos },
            inner::FatalError::EscapeAtTheEndOfInput => Error::EscapeAtTheEndOfInput,
            inner::FatalError::UnknownCharacterEscaped { pos } => {
                Error::UnknownCharacterEscaped { pos }
            }
        }),
    }
}

mod inner {
    use super::*;

    #[derive(Clone, Copy)]
    pub struct Input<'a> {
        pub s: &'a str,
        pub pos: Position,
    }

    impl<'a> From<&'a str> for Input<'a> {
        fn from(s: &'a str) -> Self {
            Self {
                s,
                pos: Position { column: 1, row: 1 },
            }
        }
    }

    pub enum FatalError {
        EscapeAtTheEndOfInput,
        UnknownCharacterEscaped { pos: Position },
        UnclosedBracket { pos: Position },
    }

    impl From<FatalError> for Error {
        fn from(e: FatalError) -> Self {
            match e {
                inner::FatalError::UnclosedBracket { pos } => Self::UnclosedBracket { pos },
                inner::FatalError::EscapeAtTheEndOfInput => Self::EscapeAtTheEndOfInput,
                inner::FatalError::UnknownCharacterEscaped { pos } => {
                    Self::UnknownCharacterEscaped { pos }
                }
            }
        }
    }

    pub enum ParsingResult<'a, T> {
        Ok((T, Rest<Input<'a>>)),
        Err,
        Fatal(FatalError),
    }

    use ParsingResult as PR;

    impl<'a, T> ParsingResult<'a, T> {
        pub fn and<O>(
            self,
            f: impl FnOnce((T, Rest<Input<'a>>)) -> ParsingResult<'a, O>,
        ) -> ParsingResult<'a, O> {
            match self {
                Self::Ok((value, rest)) => f((value, rest)),
                Self::Err => ParsingResult::Err,
                Self::Fatal(e) => ParsingResult::Fatal(e),
            }
        }

        pub fn or(self, f: impl FnOnce() -> Self) -> Self {
            if let Self::Err = self {
                f()
            } else {
                self
            }
        }

        pub fn map<O>(
            self,
            f: impl FnOnce(T) -> O,
        ) -> ParsingResult<'a, O> {
            match self {
                Self::Ok((value, rest)) => ParsingResult::Ok((f(value), rest)),
                Self::Err => ParsingResult::Err,
                Self::Fatal(fatal) => ParsingResult::Fatal(fatal),
            }
        }
    }

    fn first_character(input: Input<'_>) -> PR<'_, char> {
        let mut chars = input.s.chars();
        chars.next().map_or(PR::Err, |c| {
            PR::Ok((
                c,
                Rest(Input {
                    s: chars.as_str(),
                    pos: if c == '\n' {
                        Position {
                            row: input.pos.row + 1,
                            column: 1,
                        }
                    } else {
                        Position {
                            row: input.pos.row,
                            column: input.pos.column + 1,
                        }
                    },
                }),
            ))
        })
    }

    fn matches(input: Input<'_>, sample: char) -> PR<'_, char> {
        first_character(input).and(|(c, input)| {
            if c == sample {
                PR::Ok((c, input))
            } else {
                PR::Err
            }
        })
    }

    mod text_character {
        use super::*;

        pub enum Character {
            VerticalBar,
            Other(char),
        }

        pub fn parse(input: Input<'_>) -> PR<'_, Character> {
            first_character(input).and(|(c, input)| match c {
                '[' | ']' => PR::Err,
                '\\' => first_character(input.0)
                    .and(|(c, rest)| {
                        if "\\|[]".contains(c) {
                            PR::Ok((Character::Other(c), rest))
                        } else {
                            PR::Fatal(FatalError::UnknownCharacterEscaped { pos: input.0.pos })
                        }
                    })
                    .or(|| PR::Fatal(FatalError::EscapeAtTheEndOfInput)),
                '|' => PR::Ok((Character::VerticalBar, input)),
                _ => PR::Ok((Character::Other(c), input)),
            })
        }
    }

    mod text {
        use super::*;

        pub fn parse(mut input: Input<'_>) -> PR<'_, String> {
            let mut result = String::new();
            loop {
                match text_character::parse(input) {
                    PR::Ok((c, rest)) => {
                        input = rest.0;
                        match c {
                            text_character::Character::VerticalBar => {
                                result.shrink_to_fit();
                                return PR::Ok((result, Rest(input)));
                            }
                            text_character::Character::Other(c) => {
                                result.push(c);
                            }
                        }
                    }
                    PR::Err => {
                        if result.is_empty() {
                            return PR::Err;
                        }
                        result.shrink_to_fit();
                        return PR::Ok((result, Rest(input)));
                    }
                    PR::Fatal(e) => return PR::Fatal(e),
                }
            }
        }
    }

    pub mod sequence_of_nodes {
        use super::*;

        pub fn parse(mut input: Input<'_>) -> PR<'_, Vec<Node>> {
            let mut nodes = Vec::new();
            loop {
                match node::parse(input) {
                    PR::Err => {
                        nodes.shrink_to_fit();
                        return PR::Ok((nodes, Rest(input)));
                    }
                    PR::Ok((node, rest)) => {
                        input = rest.0;
                        nodes.push(node);
                    }
                    PR::Fatal(e) => return PR::Fatal(e),
                }
            }
        }
    }

    pub mod node {
        use super::*;

        pub fn parse(input: Input<'_>) -> PR<'_, Node> {
            text::parse(input)
                .map(|text| Node::Text(text))
                .or(|| {
                    matches(input, '[').and(|(_c, rest)| {
                        matches(rest.0, ']')
                            .map(|_c| Node::Sequence(vec![]))
                            .or(|| {
                                sequence_of_nodes::parse(rest.0).and(|(nodes, input)| {
                                    matches(input.0, ']')
                                        .map(|_c| Node::Sequence(nodes))
                                })
                            })
                            .or(|| PR::Fatal(FatalError::UnclosedBracket { pos: input.pos }))
                    })
                })
        }
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
                pos: Position { column: 4, row: 1 }
            })
        );
    }

    #[test]
    fn test_immediate_unexpected_closing_bracket_detection() {
        assert_eq!(
            parse_sequential_nodes(r#"]"#),
            Err(Error::UnexpectedClosingBracket {
                pos: Position { column: 1, row: 1 }
            })
        );
    }

    #[test]
    fn test_parsing_a_single_node_with_incorrect_syntax_after_the_node() {
        assert_eq!(
            parse_one_node(r#"a["#),
            Ok(Some((
                text("a"),
                Rest(("[", Position { row: 1, column: 2 }))
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
                pos: Position { column: 3, row: 1 }
            })
        );
    }

    #[test]
    fn test_immediate_unclosed_bracket_detection_with_one_node_parsing() {
        assert_eq!(
            parse_one_node(r#"["#),
            Err(Error::UnclosedBracket {
                pos: Position { column: 1, row: 1 }
            })
        );
    }

    #[test]
    fn test_unclosed_bracket_detection_after_a_new_line() {
        assert_eq!(
            parse_sequential_nodes("\n["),
            Err(Error::UnclosedBracket {
                pos: Position { column: 1, row: 2 }
            })
        );
    }

    #[test]
    fn test_unknown_character_escape_detection_after_a_new_line() {
        assert_eq!(
            parse_sequential_nodes("\n\\a"),
            Err(Error::UnknownCharacterEscaped {
                pos: Position { column: 2, row: 2 }
            })
        );
    }

    #[test]
    fn test_unclosed_bracket_detection() {
        assert_eq!(
            parse_sequential_nodes(r#"a[b"#),
            Err(Error::UnclosedBracket {
                pos: Position { column: 2, row: 1 }
            })
        );
    }
}
