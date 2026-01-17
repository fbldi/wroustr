
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
mod parser;



#[cfg(all(test))]
mod tests {
    use crate::parser::Parsed;

    #[test]
    fn test_parser() {
        println!("IM HERE!!");
        let text = "@IDK #param 'started string and end it now' #param2 'heheheheheh'".to_string();
        let parsed = Parsed::parse(text);
        if parsed.params.get("param").unwrap() == "started string and end it" {
            assert_eq!(parsed.params.get("param").unwrap(), "started string and end it");
        }
        println!("ENDED")
    }



}

