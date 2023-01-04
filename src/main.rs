use std::io::Write;

use nom::{bytes::complete::tag, IResult};

fn hello_parser(s: &str) -> IResult<&str, &str> {
    tag("hello")(s)
}

fn main() {
    loop {
        let mut line = String::new();
        print!("> ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut line).unwrap();
        println!("{:?}", hello_parser(&line.strip_suffix('\n').unwrap()));
    }
}
