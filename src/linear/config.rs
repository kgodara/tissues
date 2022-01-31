
use std::{
    fs,
    env,
    io::{stdin, Write},
    path::{Path, PathBuf},
    sync::{ Arc, 
        atomic::{ AtomicBool, Ordering }
    },
};

use serde_json::{ Value, Map };

use crate::constants::LINEAR_TOKEN_LEN;

const CONFIG_DIR: &str = ".config";
const APP_CONFIG_DIR: &str = "rust-cli";

const APP_CONFIG_FILE_NAME: &str = "config.txt";
const APP_VIEWER_OBJECT_FILE_NAME: &str = "viewer.txt";
const APP_DASHBOARD_VIEW_LIST: &str = "view_list.txt";


pub const MAX_PAGE_SIZE: u32 = 50;

const DEFAULT_LINEAR_ISSUE_PAGE_SIZE: u32 = 50;
const DEFAULT_LINEAR_VIEW_PANEL_PAGE_SIZE: u32 = 50;
const DEFAULT_LINEAR_ISSUE_OP_PAGE_SIZE: u32 = 50;
const DEFAULT_LINEAR_CUSTOM_VIEW_PAGE_SIZE: u32 = 50;
const DEFAULT_LINEAR_TEAM_TIMEZONE_PAGE_SIZE: u32 = 50;
const DEFAULT_LINEAR_DUE_SOON_DAY_THRESHOLD: u32 = 5;

#[derive(Debug, Clone)]
pub struct LinearConfig {
    pub is_valid_token: bool,
    pub api_key: Option<String>,
    pub viewer_object: Option<Map<String, Value>>,

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
            is_valid_token: false,
            api_key: None,
            viewer_object: None,
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

pub enum CachedDataFile {
    AccessToken,
    ViewerObject,
    ViewList,
}

impl LinearConfig {
    pub fn get_or_build_paths(data_file: CachedDataFile) -> PathBuf {
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
                let file_path = match data_file {
                    CachedDataFile::AccessToken => app_config_dir.join(APP_CONFIG_FILE_NAME),
                    CachedDataFile::ViewerObject => app_config_dir.join(APP_VIEWER_OBJECT_FILE_NAME),
                    CachedDataFile::ViewList => app_config_dir.join(APP_DASHBOARD_VIEW_LIST)
                };
                file_path.to_path_buf()
            }
            None => {
                error!("No $HOME directory found for config");
                panic!("No $HOME directory found for config");
            },
        }
    }


    pub fn load_config(&mut self) -> Option<()> {
        let config_file_path = LinearConfig::get_or_build_paths(CachedDataFile::AccessToken);
        if config_file_path.exists() {
            let token = fs::read_to_string(&config_file_path);
            if token.is_err() {
                error!("load_config - fs::read_to_string() failed: {:?}", config_file_path);
                panic!("load_config - fs::read_to_string() failed: {:?}", config_file_path);
            }

            // verify & set token fields
            if let Ok(token_val) = token {
                // verify token is correct len
                let token_len: u16 = unicode_width::UnicodeWidthStr::width(token_val.as_str()) as u16;
                if token_len != LINEAR_TOKEN_LEN {
                    return None;
                }

                self.is_valid_token = true;
                self.api_key = Some(token_val);
            }
        } else {
            return None;
        }

        // load & set viewer_object
        let viewer_obj_file_path = LinearConfig::get_or_build_paths(CachedDataFile::ViewerObject);
        if viewer_obj_file_path.exists() {
            let viewer_obj_result = fs::read_to_string(&viewer_obj_file_path);
            if viewer_obj_result.is_err() {
                error!("load_config - fs::read_to_string() failed: {:?}", viewer_obj_file_path);
                panic!("load_config - fs::read_to_string() failed: {:?}", viewer_obj_file_path);
            }

            if let Ok(viewer_obj_val) = viewer_obj_result {
                let viewer_obj = serde_json::from_str(&viewer_obj_val).unwrap();
                if let Value::Object(viewer_map) = viewer_obj {
                    self.viewer_object = Some(viewer_map);
                }
            }
        }


        Some(())
    }

    pub fn save_access_token(&mut self, token: &str) {
        let config_file_path = LinearConfig::get_or_build_paths(CachedDataFile::AccessToken);
        fs::write(&config_file_path, token).expect("Unable to write file");

        self.api_key = Some(String::from(token));
        self.is_valid_token = true;
    }

    pub fn save_viewer_object(&mut self, viewer_object: serde_json::Map<String, Value>) {
        let viewer_object_file_path = LinearConfig::get_or_build_paths(CachedDataFile::ViewerObject);
        let serialized = serde_json::to_string(&viewer_object).unwrap();
        fs::write(&viewer_object_file_path, serialized).unwrap();

        self.viewer_object = Some(viewer_object);
    }

    pub fn save_view_list(view_list: Vec<Option<Value>>) {
        let view_list_file_path = LinearConfig::get_or_build_paths(CachedDataFile::ViewList);
        let serialized = serde_json::to_string(&view_list).unwrap();
        fs::write(&view_list_file_path, serialized).unwrap();
    }

    // Attempt to read cached view list from file
    pub fn read_view_list() -> Option<Vec<Option<Value>>> {
        match dirs::home_dir() {
            Some(home) => {
                let path = Path::new(&home);
                let home_config_dir = path.join(CONFIG_DIR);
                let app_config_dir = home_config_dir.join(APP_CONFIG_DIR);
                let view_list_file_path = app_config_dir.join(APP_DASHBOARD_VIEW_LIST);

                let data = fs::read_to_string(view_list_file_path);

                match data {
                    Ok(data_str) => {
                        let deserialized: Vec<Option<Value>> = serde_json::from_str(&data_str).unwrap();
                        return Some(deserialized);
                    },
                    // Return None if file is not found, otherwise panic
                    Err(io_err) => {
                        // error!("read_view_list() error - {:?}", io_err);
                        // panic!("read_view_list() error - {:?}", io_err);
                        match io_err.kind() {
                            std::io::ErrorKind::NotFound => {
                                return None;
                            },
                            _ => {
                                error!("read_view_list() error - {:?}", io_err);
                                panic!("read_view_list() error - {:?}", io_err);
                            }
                        }
                    }
                }
            },
            None => { None }
        }
    }





}