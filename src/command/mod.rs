use crate::parser::Parsed;
use crate::routes::Params;

pub struct Command {
    parsed: Parsed
}
impl Command {
    pub fn from(command: impl Into<String>, params: Params) -> String {
        let command = command.into();
        let mut msg = format!("@{} ", command);
        for (key, value) in params.iter() {
            msg.push_str(&format!("#{} '{}' ", key, value));
        }
        msg.pop(); // Remove last space
        msg
    }
    pub fn parse(raw: String) -> Self  {
        // Implementation of parsing logic goes here
        let parsed = Parsed::parse(raw);
        Command {
            parsed,
        }
    }
    pub fn extract(&self) -> (String, Params) {
        (self.parsed.command.clone(), self.parsed.params.clone())
    }
}