use std::io::Write;

use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag},
    character::complete::char as character,
    character::complete::one_of,
    combinator::{map, not, eof},
    multi::{many0, many1, many_till},
    sequence::{delimited, separated_pair, pair},
    IResult,
};

#[derive(Debug)]
pub enum Node<'a> {
    Tag {
        name: &'a str,
        contents: Option<Vec<Node<'a>>>,
    },
    Text(&'a str),
}

fn text(s: &str) -> IResult<&str, Node<'_>> {
    map(escaped(is_not("\\[]"), '\\', one_of("\\[]")), |text| {
        Node::Text(text)
    })(s)
}

fn tag_name(s: &str) -> IResult<&str, &str> {
    escaped(is_not("\\[]:"), '\\', one_of("\\[]:"))(s)
}

fn tag_without_contents(s: &str) -> IResult<&str, Node<'_>> {
    map(delimited(tag("["), tag_name, tag("]")), |name| Node::Tag {
        name,
        contents: None,
    })(s)
}

fn tag_contents(s: &str) -> IResult<&str, Vec<Node>> {
    map(many_till(alt((tag_without_contents, tag_with_contents, text)), eof), |(nodes, _)| nodes)(s)
}

fn tag_with_contents(s: &str) -> IResult<&str, Node<'_>> {
    map(
        delimited(
            tag("["),
            separated_pair(tag_name, tag(":"), tag_contents),
            tag("]"),
        ),
        |(name, contents)| Node::Tag {
            name,
            contents: Some(contents),
        },
    )(s)
}

fn main() {
    loop {
        let mut line = String::new();
        print!("> ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut line).unwrap();
        println!("{:?}", tag_contents(line.strip_suffix('\n').unwrap()));
    }
}
