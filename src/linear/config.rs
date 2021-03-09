use std::fs;
use std::env;



/*
fn get_api_key(filename: &str) -> Result<String, std::io::Error> {

    let contents = fs::read_to_string(filename)?;
    Ok(contents)
}
*/


#[derive(Debug)]
pub struct LinearConfig {
    pub selected_team: String,
    pub api_key: Option<String>,
}

impl Default for LinearConfig {
    fn default() -> LinearConfig {

        info!("{:?}", env::var("LINEAR_PERSONAL_API_KEY"));

        LinearConfig {
            selected_team: String::new(),
            api_key: match env::var("LINEAR_PERSONAL_API_KEY").ok() {
                Some(x) =>  Some(String::from(x)),
                None => None,
            },
        }
    }
}