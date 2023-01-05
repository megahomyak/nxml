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

fn text<Input, Error>(s: Input) -> IResult<Input, Node<'_>, Error> {
    map(escaped(is_not("\\[]"), '\\', one_of("\\[]")), |text| {
        Node::Text(text)
    })(s)
}

fn tag_name<Input, Error>(s: Input) -> IResult<Input, &str, Error> {
    escaped(is_not("\\[]:"), '\\', one_of("\\[]:"))(s)
}

fn tag_without_contents<Input, Error>(s: Input) -> IResult<Input, Node<'_>, Error> {
    map(delimited(tag("["), tag_name, tag("]")), |name| Node::Tag {
        name,
        contents: None,
    })(s)
}

fn tag_contents<Input, Error>(s: Input) -> IResult<Input, Vec<Node>, Error> {
    map(many_till(alt((tag_without_contents, tag_with_contents, text)), eof), |(nodes, _)| nodes)(s)
}

fn tag_with_contents<Input, Error: >(s: Input) -> IResult<Input, Node<'_>, Error> {
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
        println!("{:?}", tag_contents::<&str, nom::error::VerboseError<&str>>(line.strip_suffix('\n').unwrap()));
    }
}
