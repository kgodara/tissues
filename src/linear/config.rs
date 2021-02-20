use std::fs;



fn get_api_key(filename: &str) -> Result<String, std::io::Error> {

    let contents = fs::read_to_string(filename)?;
    Ok(contents)
}


#[derive(Debug)]
pub struct LinearConfig {
    pub selected_team: String,
    pub api_key: String,
}

impl Default for LinearConfig {
    fn default() -> LinearConfig {
        LinearConfig {
            selected_team: String::new(),
            api_key: String::from(get_api_key(&"linear_key.txt").expect("Failed to get Linear API Key")),
        }
    }
}