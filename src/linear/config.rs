
use std::{
    fs,
    env,
    io::{stdin, Write},
    path::{Path, PathBuf},
};

use crate::constants::LINEAR_TOKEN_LEN;

const CONFIG_DIR: &str = ".config";
const APP_CONFIG_DIR: &str = "rust-cli";
const FILE_NAME: &str = "config.txt";


const DEFAULT_LINEAR_ISSUE_PAGE_SIZE: u32 = 50;
const DEFAULT_LINEAR_VIEW_PANEL_PAGE_SIZE: u32 = 50;
const DEFAULT_LINEAR_ISSUE_OP_PAGE_SIZE: u32 = 50;
const DEFAULT_LINEAR_CUSTOM_VIEW_PAGE_SIZE: u32 = 50;
const DEFAULT_LINEAR_TEAM_TIMEZONE_PAGE_SIZE: u32 = 50;
const DEFAULT_LINEAR_DUE_SOON_DAY_THRESHOLD: u32 = 5;

#[derive(Debug, Clone)]
pub struct LinearConfig {
    pub loaded: bool,
    pub api_key: Option<String>,

    pub issue_page_size: u32,
    pub view_panel_page_size: u32,
    pub issue_op_page_size: u32,
    pub custom_view_page_size: u32,
    pub team_timezone_page_size: u32,

    pub due_soon_day_threshold: u32,
}

impl Default for LinearConfig {
    fn default() -> LinearConfig {

        info!("{:?}", env::var("LINEAR_PERSONAL_API_KEY"));

        LinearConfig {
            loaded: false,
            api_key: None/*env::var("LINEAR_PERSONAL_API_KEY").ok().map(String::from)*/,
            issue_page_size: match env::var("LINEAR_ISSUE_PAGE_SIZE").ok() {
                Some(x) => *x.parse::<u32>().ok().get_or_insert(DEFAULT_LINEAR_ISSUE_PAGE_SIZE),
                None => DEFAULT_LINEAR_ISSUE_PAGE_SIZE,
            },
            view_panel_page_size: match env::var("LINEAR_VIEW_PANEL_PAGE_SIZE").ok() {
                Some(x) => *x.parse::<u32>().ok().get_or_insert(DEFAULT_LINEAR_VIEW_PANEL_PAGE_SIZE),
                None => DEFAULT_LINEAR_VIEW_PANEL_PAGE_SIZE,
            },
            issue_op_page_size: match env::var("LINEAR_ISSUE_OP_PAGE_SIZE").ok() {
                Some(x) => *x.parse::<u32>().ok().get_or_insert(DEFAULT_LINEAR_ISSUE_OP_PAGE_SIZE),
                None => DEFAULT_LINEAR_ISSUE_OP_PAGE_SIZE,
            },
            custom_view_page_size: match env::var("LINEAR_CUSTOM_VIEW_PAGE_SIZE").ok() {
                Some(x) => *x.parse::<u32>().ok().get_or_insert(DEFAULT_LINEAR_CUSTOM_VIEW_PAGE_SIZE),
                None => DEFAULT_LINEAR_CUSTOM_VIEW_PAGE_SIZE,
            },
            team_timezone_page_size: match env::var("LINEAR_TEAM_TIMEZONE_PAGE_SIZE").ok() {
                Some(x) => *x.parse::<u32>().ok().get_or_insert(DEFAULT_LINEAR_TEAM_TIMEZONE_PAGE_SIZE),
                None => DEFAULT_LINEAR_TEAM_TIMEZONE_PAGE_SIZE,
            },
            due_soon_day_threshold: match env::var("LINEAR_DUE_SOON_DAY_THRESHOLD").ok() {
                Some(x) => *x.parse::<u32>().ok().get_or_insert(DEFAULT_LINEAR_DUE_SOON_DAY_THRESHOLD),
                None => DEFAULT_LINEAR_DUE_SOON_DAY_THRESHOLD,
            }
        }
    }
}

impl LinearConfig {
    pub fn get_or_build_paths() -> PathBuf {
        match dirs::home_dir() {
            Some(home) => {
                let path = Path::new(&home);
                let home_config_dir = path.join(CONFIG_DIR);
                let app_config_dir = home_config_dir.join(APP_CONFIG_DIR);

                if !home_config_dir.exists() {
                    let res = fs::create_dir(&home_config_dir);
                    if res.is_err() {
                        error!("get_or_build_paths() fs::create_dir() failed: {:?}", home_config_dir);
                        panic!("get_or_build_paths() fs::create_dir() failed: {:?}", home_config_dir);
                    }
                }

                if !app_config_dir.exists() {
                    let res = fs::create_dir(&app_config_dir);
                    if res.is_err() {
                        error!("get_or_build_paths() fs::create_dir() failed: {:?}", app_config_dir);
                        panic!("get_or_build_paths() fs::create_dir() failed: {:?}", app_config_dir);
                    }
                }

                let config_file_path = &app_config_dir.join(FILE_NAME);

                config_file_path.to_path_buf()
            }
            None => {
                error!("No $HOME directory found for config");
                panic!("No $HOME directory found for config");
            },
        }
    }


    pub fn load_config(&mut self) -> Option<()> {
        let config_file_path = LinearConfig::get_or_build_paths();
        if config_file_path.exists() {
            let token = fs::read_to_string(&config_file_path);
            if token.is_err() {
                error!("load_config - fs::read_to_string() failed: {:?}", config_file_path);
                panic!("load_config - fs::read_to_string() failed: {:?}", config_file_path);
            }
    
            if let Ok(token_val) = token {
                // verify token is correct len
                let token_len: u16 = unicode_width::UnicodeWidthStr::width(token_val.as_str()) as u16;
                if token_len != LINEAR_TOKEN_LEN {
                    return None;
                }

                self.loaded = true;
                self.api_key = Some(token_val);
            }

            Some(())
        } else {
            None
        }
    }

    pub fn save_access_token(token: &str) {
        let config_file_path = LinearConfig::get_or_build_paths();
        fs::write(&config_file_path, token).expect("Unable to write file");
    }





}