use std::env;


const DEFAULT_LINEAR_ISSUE_PAGE_SIZE: u32 = 50;
const DEFAULT_LINEAR_VIEW_PANEL_PAGE_SIZE: u32 = 50;
const DEFAULT_LINEAR_CUSTOM_VIEW_PAGE_SIZE: u32 = 50;
const DEFAULT_LINEAR_TEAM_TIMEZONE_PAGE_SIZE: u32 = 50;
const DEFAULT_LINEAR_DUE_SOON_DAY_THRESHOLD: u32 = 5;

#[derive(Debug, Clone)]
pub struct LinearConfig {
    pub selected_team: String,
    pub api_key: Option<String>,
    pub issue_page_size: u32,
    pub view_panel_page_size: u32,
    pub custom_view_page_size: u32,
    pub team_timezone_page_size: u32,
    pub due_soon_day_threshold: u32,
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
            issue_page_size: match env::var("LINEAR_ISSUE_PAGE_SIZE").ok() {
                Some(x) => *x.parse::<u32>().ok().get_or_insert(DEFAULT_LINEAR_ISSUE_PAGE_SIZE),
                None => DEFAULT_LINEAR_ISSUE_PAGE_SIZE,
            },
            view_panel_page_size: match env::var("LINEAR_VIEW_PANEL_PAGE_SIZE").ok() {
                Some(x) => *x.parse::<u32>().ok().get_or_insert(DEFAULT_LINEAR_VIEW_PANEL_PAGE_SIZE),
                None => DEFAULT_LINEAR_VIEW_PANEL_PAGE_SIZE,
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