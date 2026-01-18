
//use crate::routes::{Params, State};
#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "server")]
pub mod server;
pub mod routes;
#[cfg(feature = "interception")]
pub mod interceptor;
#[cfg(feature = "layers")]
pub mod layer;
pub mod command;
mod parser;


#[cfg(all(test))]
mod tests {
    use crate::command::Command;
    use crate::parser::Parsed;
    use crate::routes::Params;

    #[test]
    fn test_parser() {
        println!("IM HERE!!");
        let text = Command::from("JUHUU", Params::from([("asd".to_string(),"'pulu-lulu'".to_string())]) );
        let parsed = Parsed::parse(text);
        if parsed.params.get("asd").unwrap() == "started string and end it" {
            assert_eq!(parsed.params.get("param").unwrap(), "started string and end it");
        }
        let ikd = "asdasdasd";
        println!("ENDED")
    }



}

