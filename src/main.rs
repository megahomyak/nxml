use std::{io::Write, ops::RangeFrom};

use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag},
    character::complete::char as character,
    character::complete::one_of,
    combinator::{eof, map, not},
    error::ParseError,
    multi::{many0, many1, many_till},
    sequence::{delimited, pair, separated_pair},
    Compare, IResult, InputIter, InputLength, InputTake, InputTakeAtPosition, Offset, Slice, AsChar,
};

#[derive(Debug)]
pub enum Node<Input> {
    Tag {
        name: Input,
        contents: Option<Vec<Node<Input>>>,
    },
    Text(Input),
}

fn text<Input, Error>(s: Input) -> IResult<Input, Node<Input>, Error>
where
    Input: Clone
        + Offset
        + InputTake
        + InputLength
        + InputTakeAtPosition
        + Slice<RangeFrom<usize>>
        + InputIter,
    <Input as InputIter>::Item: AsChar,
    Error: ParseError<Input>,
{
    map(escaped(is_not("\\[]"), '\\', one_of("\\[]")), |text| {
        Node::Text(text)
    })(s)
}

fn tag_name<Input: InputIter, Error: ParseError<Input>>(s: Input) -> IResult<Input, Input, Error> {
    escaped(is_not("\\[]:"), '\\', one_of("\\[]:"))(s)
}

fn tag_without_contents<Input: InputIter, Error: ParseError<Input>>(
    s: Input,
) -> IResult<Input, Node<Input>, Error> {
    map(delimited(tag("["), tag_name, tag("]")), |name| Node::Tag {
        name,
        contents: None,
    })(s)
}

fn tag_contents<Input: InputIter, Error: ParseError<Input>>(
    s: Input,
) -> IResult<Input, Vec<Node<Input>>, Error> {
    map(
        many_till(alt((tag_without_contents, tag_with_contents, text)), eof),
        |(nodes, _)| nodes,
    )(s)
}

fn tag_with_contents<
    Input: InputIter + InputTake + Compare<&'static str>,
    Error: ParseError<Input>,
>(
    s: Input,
) -> IResult<Input, Node<Input>, Error> {
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
        println!(
            "{:?}",
            tag_contents::<&str, nom::error::VerboseError<&str>>(line.strip_suffix('\n').unwrap())
        );
    }
}
