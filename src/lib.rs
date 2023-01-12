pub enum Node {
    Sequence(Vec<Node>),
    Text(String),
}

pub enum ParsingError {
    EscapeAtTheEndOfInput,
    UnknownCharacterEscaped,
}

pub struct Rest<T>(T);

mod text_character {
    use super::*;

    pub enum RecoverableError {
        NoMoreText,
    }

    pub enum UnrecoverableError {
        EscapeAtTheEndOfInput,
        UnknownCharacterEscaped,
    }

    pub fn parse(
        input: &str,
    ) -> Result<Result<(char, Rest<&str>), RecoverableError>, UnrecoverableError> {
        let mut chars = input.chars();
        match chars.next() {
            None => Ok(Err(RecoverableError::NoMoreText)),
            Some(c) => match c {
                '|' | '[' | ']' => Ok(Err(RecoverableError::NoMoreText)),
                '\\' => match chars.next() {
                    Some(c @ ('|' | '\\' | '[' | ']')) => Ok(Ok((c, Rest(chars.as_str())))),
                    Some(_) => Err(UnrecoverableError::UnknownCharacterEscaped),
                    None => Err(UnrecoverableError::EscapeAtTheEndOfInput),
                },
                _ => Ok(Ok((c, Rest(chars.as_str())))),
            },
        }
    }
}

mod text {
    use super::*;
    use text_character::{RecoverableError, UnrecoverableError};

    pub fn parse(
        mut input: &str,
    ) -> Result<Result<(String, Rest<&str>), RecoverableError>, UnrecoverableError> {
        let mut result = String::new();
        loop {
            match text_character::parse(input) {
                Ok((c, rest)) => {
                    result.push(c);
                    input = rest.0;
                }
                Err(error @ text_character::Error::NoMoreText) => {
                    if result.is_empty() {
                        return Err(error);
                    } else {
                        result.shrink_to_fit();
                        return Ok((result, Rest(input)));
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }
}

mod node {}

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

        assert_eq!(
            parse_sequential_nodes(
                r#"text[first sequence item|second sequence item[subsequence]|third sequence item[]with an empty in-between sequence]"#
            ),
            vec![
                text("text"),
                seq(vec![
                    text("first sequence item"),
                    text("second sequence item"),
                    vec(seq![text("subsequence")]),
                    text("third sequence item"),
                    seq(vec![]),
                    text("with an empty in-between sequence")
                ])
            ]
        );
        assert_eq!(
            parse_sequential_nodes(r#"[some vertical bars: \|\|\|][some brackets: \]\[\[\]]"#),
            Ok(vec![
                seq(vec![text(r#"some vertical bars: |||"#)]),
                seq(vec![text(r#"some brackets: ][[]"#)])
            ])
        );
        assert_eq!(
            parse_sequential_nodes(r#"first text node|second text node"#),
            Ok(vec![text("first text node"), text("second text node")])
        );

        assert_eq!(
            parse_sequential_nodes(r#"[|a[]]|[a||b]"#),
            Ok(vec![
                seq(vec![text(""), text("a"), seq(vec![])]),
                seq(vec![text("a"), text(""), text("b")])
            ])
        );
        todo!("check error cases");
    }
}
