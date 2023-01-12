#[derive(Debug, PartialEq)]
pub enum Node {
    Sequence(Vec<Node>),
    Text(String),
}

#[derive(Debug, PartialEq)]
pub struct Rest<T>(T);

#[derive(Debug, PartialEq)]
pub enum FatalError {
    EscapeAtTheEndOfInput,
    UnknownCharacterEscaped,
}

#[derive(Debug, PartialEq)]
pub enum ParsingResult<'a, T> {
    Ok((T, Rest<&'a str>)),
    Err,
    Fatal(FatalError),
}

use ParsingResult as PR;

impl<'a, T> ParsingResult<'a, T> {
    pub fn and<O>(
        self,
        f: impl FnOnce((T, Rest<&'a str>)) -> ParsingResult<'a, O>,
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
        f: impl FnOnce((T, Rest<&'a str>)) -> (O, Rest<&'a str>),
    ) -> ParsingResult<'a, O> {
        match self {
            Self::Ok((value, rest)) => ParsingResult::Ok(f((value, rest))),
            Self::Err => ParsingResult::Err,
            Self::Fatal(fatal) => ParsingResult::Fatal(fatal),
        }
    }
}

fn first_character(input: &str) -> PR<'_, char> {
    let mut chars = input.chars();
    match chars.next() {
        None => PR::Err,
        Some(c) => PR::Ok((c, Rest(chars.as_str()))),
    }
}

fn matching(input: &str, sample: char) -> PR<'_, char> {
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

    pub fn parse(input: &str) -> PR<'_, Character> {
        first_character(input).and(|(c, input)| match c {
            '[' | ']' => PR::Err,
            '\\' => first_character(input.0).and(|(c, input)| {
                if "\\|[]".contains(c) {
                    PR::Ok((Character::Other(c), input))
                } else {
                    PR::Err
                }
            }),
            '|' => PR::Ok((Character::VerticalBar, input)),
            _ => PR::Ok((Character::Other(c), input)),
        })
    }
}

mod text {
    use super::*;

    pub fn parse(mut input: &str) -> PR<'_, String> {
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
                    } else {
                        result.shrink_to_fit();
                        return PR::Ok((result, Rest(input)));
                    }
                }
                PR::Fatal(e) => return PR::Fatal(e),
            }
        }
    }
}

mod sequence_of_nodes {
    use super::*;

    pub fn parse(mut input: &str) -> PR<'_, Vec<Node>> {
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

mod node {
    use super::*;

    pub fn parse(input: &str) -> PR<'_, Node> {
        text::parse(input)
            .map(|(text, input)| (Node::Text(text), input))
            .or(|| {
                matching(input, '[').and(|(c, input)| {
                    matching(input.0, ']')
                        .map(|(c, input)| (Node::Sequence(vec![]), input))
                        .or(|| {
                            sequence_of_nodes::parse(input.0).and(|(nodes, input)| {
                                matching(input.0, ']')
                                    .map(|(c, input)| (Node::Sequence(nodes), input))
                            })
                        })
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() {
        fn text(s: &'static str) -> Node {
            Node::Text(String::from(s))
        }
        fn seq(s: Vec<Node>) -> Node {
            Node::Sequence(s)
        }
        fn ok<T>(v: T) -> ParsingResult<'static, T> {
            ParsingResult::Ok((v, Rest("")))
        }

        assert_eq!(
            sequence_of_nodes::parse(
                r#"text[first sequence item|second sequence item[subsequence]|third sequence item[]with an empty in-between sequence]"#
            ),
            ok(vec![
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
        assert_eq!(
            sequence_of_nodes::parse(r#"[some vertical bars: \|\|\|][some brackets: \]\[\[\]]"#),
            PR::Ok((
                vec![
                    seq(vec![text(r#"some vertical bars: |||"#)]),
                    seq(vec![text(r#"some brackets: ][[]"#)])
                ],
                Rest("")
            ))
        );
        assert_eq!(
            sequence_of_nodes::parse(r#"first text node|second text node"#),
            ok(vec![text("first text node"), text("second text node")])
        );

        assert_eq!(
            sequence_of_nodes::parse(r#"[|a[]]|[a||b]"#),
            ok(vec![
                seq(vec![text(""), text("a"), seq(vec![])]),
                text(""),
                seq(vec![text("a"), text(""), text("b")])
            ])
        );
    }
}
