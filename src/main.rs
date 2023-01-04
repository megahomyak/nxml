use std::io::Write;

use nom::{bytes::complete::is_not, IResult};

fn parser(s: &str) -> IResult<&str, &str> {
    is_not("[]\\")(s)
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
