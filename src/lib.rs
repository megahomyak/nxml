pub enum Node {
    Tag {
        name: String,
        contents: Option<Vec<Node>>,
    },
    Text(String),
}

pub enum ParsingErrorKind {
    UnknownCharacterEscaped,
    UnclosedBracket,
    NothingWasEscaped,
    UnexpectedClosingBracket,
}

pub enum ParsingResult<'a, T> {
    Ok {
        rest_of_input: RemainingString<'a>,
        value: T,
    },
    Err {
        kind: ParsingErrorKind,
    },
}

#[derive(Clone, Copy)]
pub struct RemainingString<'a> {
    pub contents: &'a str,
}

fn take_while<'a>(source: &'a mut RemainingString, checker: impl Fn(char) -> bool) -> &'a str {
    if let Some(parts_separator) = source.contents.find(|c| !checker(c)) {
        let (result, new_contents) = source.contents.split_at(parts_separator);
        source.contents = new_contents;
        result
    } else {
        let old_contents = source.contents;
        source.contents = "";
        old_contents
    }
}

pub fn parse_text<'a>(input: RemainingString<'a>) -> ParsingResult<'_, &'a str> {
    let text = take_while(&mut input, |c| !"\\".contains(c))
}

pub fn parse_sequential_nodes(input: RemainingString) -> ParsingResult<'_, Vec<Node>> {

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() {
        assert_eq!(parse("[tag:[abc def:contents]]"))
    }
}
