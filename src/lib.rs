pub struct Nodes {
    pub contents: Vec<Node>,
}

impl Nodes {
    pub fn tags(&self) -> impl Iterator<Item = &Tag> {
        self.contents.iter().filter_map(|node| {
            if let Node::Tag(tag) = node {
                Some(tag)
            } else {
                None
            }
        })
    }
}

pub struct Tag {
    pub name: String,
    pub contents: Option<Nodes>,
}

pub enum Node {
    Tag(Tag),
    Text(String),
}

pub enum ParsingErrorKind {
    UnknownCharacterEscaped,
    UnclosedBracket,
    NothingWasEscaped,
    UnexpectedClosingBracket,
}

#[derive(Clone)]
pub struct Position {
    row_number: usize,
    column_number: usize,
}

pub struct ParsingError {
    position: Position,
    kind: ParsingErrorKind,
}

pub struct Parser<'parser> {
    input: &'parser str,
    position: Position,
}

pub struct ParsingResult<'pr> {
    pub node: Node,
    pub rest: &'pr str,
}

impl<'parser> Parser<'parser> {
    pub fn new(input: &'parser str) -> Self {
        Self {
            input,
            position: Position {
                row_number: 1,
                column_number: 1,
            },
        }
    }

    fn error_here(&mut self, kind: ParsingErrorKind) -> ParsingError {
        ParsingError {
            position: self.position.clone(),
            kind,
        }
    }

    fn parse_node(&mut self) -> Result<Option<Node>, ParsingError> {
        let iter = self.input.chars().enumerate();
        if let Some((i, c)) = iter.next() {
            let opening_parenthesis_position = self.position.clone();
            if c == '[' {
            } else if c == ']' {
                Err(self.error_here(ParsingErrorKind::UnexpectedClosingBracket))
            }
            while let Some((i, c)) = iter.next() {
                if c == '[' || ']' {}
            }
        } else {
            Ok(None)
        }
    }
}

pub fn parse_tag_contents(input: &str, mut position: Position) -> Result<Vec<Node>, ParsingError> {
    let mut nodes = Vec::new();
    let mut text = String::new();
    macro_rules! save_text {
        () => {{
            nodes.push(Node::Text(text));
            text = String::new();
        }};
    }
    let mut next = {
        let mut iter = input.chars();
        move || {
            let c = iter.next();
            if let Some('\n') = c {
                position.row_number += 1;
                position.column_number = 1;
            }
            c
        }
    };
    let error_here = |kind| Err(ParsingError { position, kind });
    while let Some(c) = next() {
        match c {
            '\\' => {
                let Some(c) = next() else {return error_here(ParsingErrorKind::NothingWasEscaped)};
                match c {
                    '[' | ']' | '\\' => text.push(c),
                    _ => return error_here(ParsingErrorKind::UnknownCharacterEscaped),
                }
            }
            '[' => {
                save_text!();
                let opening_bracket_position = position;
                let tag_name = String::new();
                let inner_nodes = Vec::new();
                nodes.push();
                inner_nodes.push();
            }
            ']' => return error_here(ParsingErrorKind::UnexpectedClosingBracket),
            _ => text.push(c),
        }
    }
    save_text!();
    Ok(nodes)
}

pub fn parse(input: &str) -> Result<Nodes, ParsingError> {
    parse_tag_contents(
        input,
        Position {
            row_number: 1,
            column_number: 1,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() {
        assert_eq!(parse("[tag:[abc def:contents]]"))
    }
}
