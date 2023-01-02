pub struct Nodes(pub Vec<Node>);

pub struct Tag {
    pub name: String,
    pub contents: Nodes,
}

pub enum Node {
    Tag(Tag),
    Text(String),
}

impl Nodes {
    pub fn tags(&self) -> impl Iterator<Item = &Tag> {
        self.0.iter().filter_map(|node| {
            if let Node::Tag(tag) = node {
                Some(tag)
            } else {
                None
            }
        })
    }
}

pub enum ParsingError {}

pub struct Parser<'parser>(&'parser str);

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
