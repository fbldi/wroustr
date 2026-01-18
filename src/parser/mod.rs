use crate::routes::Params;


#[derive(Clone)]
pub struct Parsed {
    pub params: Params,
    pub command: String
}



impl Parsed {
    pub fn parse(msg: String) -> Self {
        let copy = msg.clone();
        println!("As string: {}", copy);
        let peaces = Self::tokenize(copy.as_str());
        #[cfg(feature = "debug")]
        println!("PARSE: {:?}", peaces);
        if peaces.len() < 1 {
            return Self {
                params: Params::new(),
                command: "".to_string()
            };
        }
        if !peaces[0].starts_with("@") {
            return Self {
                params: Params::new(),
                command: "".to_string()
            };
        }
        #[cfg(feature = "debug")]
        println!("Found {}", peaces[0]);
        let mut params = Params::new();
        let mut i = 1;
        while i + 1 < peaces.len() {
            let key = peaces[i].clone();
            let value = peaces[i + 1].clone();

            if key.starts_with("#") {
                params.insert(
                    key[1..].to_string(),
                    value
                        .trim()
                        .trim_matches('"')
                        .to_string(),
                );
            }

            i += 2;
        };
        #[cfg(feature = "debug")]
        println!("PARAMS: {:?}", params);

        Self {
            params,
            command: peaces[0].to_string()
        }
    }

    fn tokenize(raw: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut chars = raw.chars().peekable();

        while let Some(c) = chars.next() {
            if c.is_whitespace() {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            } else if c == '"' || c == '\'' {
                let quote = c;
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next == quote {
                        break;
                    }
                    current.push(next);
                }
                tokens.push(current.clone());
                current.clear();
            } else {
                current.push(c);
            }
        }

        if !current.is_empty() {
            tokens.push(current);
        }

        tokens
    }
}