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

fn take_one_character(input: &str) -> Option<(char, Rest<&str>)> {
    let chars = input.chars();
    if let Some(c) = chars.next() {
        Some((c, Rest(chars.as_str())))
    } else {
        None
    }
}

enum SpecialCharacter {
    SequenceOpener,
    SequenceCloser,
    TextTerminator,
    SpecialCharacterEscaper,
}

impl SpecialCharacter {
    fn new(c: char) -> Option<Self> {
        match c {
            '[' => Some(Self::SequenceOpener),
            ']' => Some(Self::SequenceCloser),
            '|' => Some(Self::TextTerminator),
            '\\' => Some(Self::SpecialCharacterEscaper),
            _ => None
        }
    }
}

mod special_character {
    use super::{Rest, take_one_character};

    pub enum Error {
        InputEnded,
        SpecialCharacterNotFound,
    }

    pub enum Chars {
        
    }

    pub fn parse(input: &str) -> Result<Rest<&str>, Error> {
        take_one_character(input).map(|c, Rest(input)| )
    }
}

fn parse_text_without_escaping(input: &str) -> (&str, Option<SpecialCharacter>, &str) {
    let mut char_indices = input.char_indices();
    while let Some((index, c)) = char_indices.next() {
        if let Some(c) = SpecialCharacter::new(c) {
            return (&input[..index], Some(c), char_indices.as_str());
        }
    }
    (input, None, char_indices.as_str())
}

fn parse_text(input: &str) -> Result<(Option<String>, Option<char>, &str), ParsingError> {
    let mut text = String::new();
    let mut chars = input.chars();
    loop {
    let character = loop {
        match chars.next() {
            None => ,
            Some(c) => 
        }
    }
    }
    while let Some(c) = chars.next() {
        if let Some()
        match SpecialCharacter::new(c) {
            None => (),
            Some()
        }
        if c == '\\' {
            match chars.next() {
                None => return Err(ParsingError::EscapeCharacterEscapedNothing),
                Some(c) => match SpecialCharacter::new(c) {
                    None => return Err(ParsingError::NonSpecialCharacterWasEscaped),
                    Some(_) => ,
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SpecialCharacter {
    OpeningBracket,
    ClosingBracket,
    VerticalBar,
    EscapeCharacter,
}

impl SpecialCharacter {
    fn new(c: char) -> Option<Self> {
        match c {
            '\\' => Some(Self::EscapeCharacter),
            '[' => Some(Self::OpeningBracket),
            ']' => Some(Self::ClosingBracket),
            '|' => Some(Self::VerticalBar),
            _ => None,
        }
    }
}

fn parse_object(input: &str) -> Result<(Node, Option<SpecialCharacter>, &str), ParsingError> {
    let (text, c, rest) = parse_text(input);
}

pub fn parse_sequential_nodes(input: &str) -> Result<Vec<Node>, ParsingError> {
    let (text, c, rest) = parse_text(input);
    match c {
        None => if !text.is_empty() {},
        Some(_) => (),
    }
    todo!()
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
            Ok(vec![seq(vec![text(r#"some vertical bars: |||"#)]), seq(vec![text(r#"some brackets: ][[]"#)])])
        );
        assert_eq!(
            parse_sequential_nodes(r#"first text node|second text node"#),
            Ok(vec![text("first text node"), text("second text node")])
        );


        assert_eq!(
            parse_sequential_nodes(r#"[|a[]]|[a||b]"#),
            Ok(vec![seq(vec![text(""), text(""), seq(vec![])]), seq(vec![text("a"), text(""),])])
            );
        todo!("check for errors");
    }
}
