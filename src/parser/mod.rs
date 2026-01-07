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
        let peaces = copy.split(" ").collect::<Vec<&str>>();
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
        println!("Found {}", peaces[0]);
        let mut params = Params::new();
        let mut i = 1;
        while i + 1 < peaces.len() {
            let key = peaces[i];
            let value = peaces[i + 1];

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

        Self {
            params,
            command: peaces[0].to_string()
        }
    }
}