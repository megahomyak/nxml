#[derive(Debug, PartialEq)]
pub enum Node {
    Sequence(Vec<Node>),
    Text(String),
}

#[derive(Debug, PartialEq)]
pub enum ParsingError {
    EscapeCharacterEscapedNothing,
    NonSpecialCharacterWasEscaped,
}

struct Rest<T>(pub T);

fn split_until<'a>(input: &'a str, checker: impl FnMut(char) -> bool) -> (&'a str, Rest<&'a str>) {
    let mut char_indices = input.char_indices();
    let mut old_index = 0;
    loop {
        if let Some((new_index, c)) = char_indices.next() {
            if !checker(c) {
                return (&input[..old_index], Rest(&input[old_index..]));
            }
            old_index = new_index;
        } else {
            return (input, Rest(""));
        }
    }
}

fn take_first_character(input: &str) -> Option<(char, Rest<&str>)> {
    let mut chars = input.chars();
    if let Some(c) = chars.next() {
        Some((c, Rest(chars.as_str())))
    } else {
        None
    }
}

fn take_last_character(input: &str) -> Option<(char, Rest<&str>)> {
    let mut chars = input.chars();
    if let Some(c) = chars.next_back() {
        Some((c, Rest(chars.as_str())))
    } else {
        None
    }
}

fn take_matching<'a>(input: &'a str, sample: &'static str) -> Option<(&'a str, Rest<&'a str>)> {
    if input.starts_with(sample) {
        Some((&input[..sample.len()], Rest(&input[sample.len()..])))
    } else {
        None
    }
}

mod special_character {
    use super::*;

    pub enum Error {
        InputEnded,
        WrongCharacter,
    }

    pub fn parse(input: &str) -> Result<(char, Rest<&str>), Error> {
        take_first_character(input)
            .ok_or(Error::InputEnded)
            .and_then(|(c, rest)| {
                if "[]|\\".contains(c) {
                    Ok((c, rest))
                } else {
                    Err(Error::WrongCharacter)
                }
            })
    }
}

mod text_character {
    use super::*;

    pub enum Error {
        InputEnded,
        UnknownCharacterEscaped,
        SpecialCharacterEncountered,
        EscapeAtTheEndOfInput,
    }

    pub fn parse(input: &str) -> Result<(char, Rest<&str>), Error> {
        fn escaped<'a>(
            input: &'a str,
            sample: &'static str,
        ) -> Result<(char, Rest<&'a str>), Error> {
            take_matching(input, sample)
                .map(|(s, rest)| ((take_last_character(s).unwrap()).0, rest))
                .ok_or(Error::InputEnded)
        }

        take_first_character(input).and_then(|(c, rest)| {
            if c == '\\' {
                match take_first_character(rest.0) {
                    None => Err(Error::EscapeAtTheEndOfInput),
                    Some((c, rest)) => match c {
                        ''
                    }
                }
                take_first_character(rest.0)
                    .ok_or_else(|(c, rest)| if "\\[]|".contains(c) { Ok((c, rest)) } else { Err(Error::UnknownCharacterEscaped) })
            } else {
                Ok((c, rest))
            }
        });

        escaped(input, "\\\\")
            .or_else(|_err| escaped(input, "\\["))
            .or_else(|_err| escaped(input, "\\]"))
            .or_else(|_err| escaped(input, "\\|"))
            .or_else(|_err| {
                if special_character::parse(input).is_ok() {
                    Err(Error::SpecialCharacterEncountered)
                }
            });
        if special_character::parse(input).is_ok() {
            Err(Error::SpecialCharacterEncountered)
        } else {
        }
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

        assert_eq!(
            parse_sequential_nodes(
                r#"text[first sequence item|second sequence item[subsequence]|third sequence item[]with an empty in-between sequence]"#
            ),
            todo!()
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
                seq(vec![text(""), text(""), seq(vec![])]),
                seq(vec![text("a"), text(""),])
            ])
        );
        todo!("check for errors");
    }
}
