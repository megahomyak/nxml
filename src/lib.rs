pub enum Node {
    Tag { name: String, contents: Vec<Node> },
    Text(String),
}

pub enum ParsingError {}

pub fn parse(s: &str) -> Result<Vec<Node>, ParsingError> {
    let results = Vec::new();
    let Some(character) = s.chars().next() else { return Ok(results); };
    if character == '[' {

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() {}
}
