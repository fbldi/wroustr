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
        let peaces = Self::tokenize(copy);
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

    fn tokenize(raw: String) -> Vec<String> {
        let chars = raw.chars().collect::<Vec<char>>();
        let mut pos = 0;
        let mut current:Vec<char> = Vec::new();
        let mut finals: Vec<String> = Vec::new();
        while pos < chars.len() {
            #[cfg(feature = "debug")]
            println!("TOKENIZE: running... pos: {}", pos);
            if chars[pos].is_whitespace() {
                #[cfg(feature = "debug")]
                println!("TOKENIZE: white_space ");
                finals.push(current.iter().collect());
                current = Vec::new();
                pos += 1;
            }
            else if chars[pos] == '"' || chars[pos] == '\'' {
                #[cfg(feature = "debug")]
                println!("TOKENIZE: string started ");
                pos+=1;
                while pos+1 < chars.len() && (chars[pos+1] != '"' || chars[pos+1] != '\'') {
                    current.push(chars[pos]);
                    pos += 1;
                }
                #[cfg(feature = "debug")]
                println!("TOKENIZE: string ended ");
                finals.push(current.iter().collect());
                current = Vec::new();

            }
            else {
                current.push(chars[pos]);
                pos += 1;
            }
        }

        finals
    }
}