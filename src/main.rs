use std::io::Write;

use nom::{bytes::complete::{is_not, escaped}, IResult, character::complete::one_of};

fn parser(s: &str) -> IResult<&str, &str> {
    escaped(is_not("\\[]"), '\\', one_of("\\[]"))(s)
}

fn main() {
    loop {
        let mut line = String::new();
        print!("> ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut line).unwrap();
        println!("{:?}", parser(&line.strip_suffix('\n').unwrap()));
    }
}
