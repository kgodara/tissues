use std::fs;
use std::collections::HashMap;

use serde_json::Value;
use chrono::{DateTime, Utc, FixedOffset, TimeZone,
            Datelike, Timelike, Duration };

// use crate::errors::TimeZoneParseError;

use crate::linear::{
    LinearConfig,
    client::LinearClient,
    error::LinearClientError,
    view_resolver::FilterType,
};

use crate::app::Platform;

use crate::util::GraphQLCursor;


const TIMEZONE_JSON_PATH: &str = "data/timezones.json";


include!(concat!(env!("OUT_DIR"), "/tz_raw.rs"));


pub fn parse_timezones_from_file() -> HashMap<String, f64> {

    let query_contents = String::from(TIMEZONES);
    
    // Parse the string of data into serde_json::Value.
    let time_zone_array: serde_json::Value = serde_json::from_str(query_contents.as_str()).unwrap();

    let mut location_to_offset_map = HashMap::new();

    match time_zone_array.as_array() {
        Some(time_zone_vec) => {
            for time_zone_obj in time_zone_vec.iter() {
                
                let time_zone_location = time_zone_obj["location"].clone();
                let time_zone_offset = time_zone_obj["decimal_offset"].clone();

                let location_name: String;
                let offset: f64;

                match time_zone_location.as_str() {
                    Some(location_str) => { location_name = String::from(location_str) },
                    None => {
                        error!("TimeZone object 'location' not a str - time_zone_obj: {:?}", time_zone_obj);
                        panic!("TimeZone object 'location' not a str - time_zone_obj: {:?}", time_zone_obj);
                    },
                }

                match time_zone_offset.as_f64() {
                    Some(offset_float) => { offset = offset_float },
                    None => {
                        error!("TimeZone object 'decimal_offset' not a float - time_zone_obj: {:?}", time_zone_obj);
                        panic!("TimeZone object 'decimal_offset' not a float - time_zone_obj: {:?}", time_zone_obj);
                    },
                }

                location_to_offset_map.insert(location_name, offset);

            }
        },
        None => {
            error!("TimeZone JSON file is not an array");
            panic!("TimeZone JSON file is not an array");
        }
    }

    location_to_offset_map
}

pub async fn load_linear_team_timezones(linear_config: LinearConfig) -> Vec<(String, String)> {

    let mut cursor = GraphQLCursor::default();

    let mut team_tz_tuples: Vec<(String, String)> = Vec::new();

    // Paginate through all teams
    loop {
        if cursor.platform == Platform::Linear && !cursor.has_next_page {
            break;
        }

        let query_result: Result<Value, LinearClientError>;

        query_result = LinearClient::fetch_team_timezones(linear_config.clone(), Some(cursor.clone())).await;

        if let Ok(response) = query_result {


            let team_obj_list: Vec<Value>;
            let mut team_tz_page_list: Vec<(String, String)>;
            
            match response["team_nodes"].as_array() {
                Some(issue_data) => {
                    team_obj_list = issue_data.clone();
                },
                None => {
                    error!("'team_nodes' invalid format: {:?}", response["team_nodes"]);
                    panic!("'team_nodes' invalid format");
                }
            }

            // Format fetched data into (id, timezone) tuples
            team_tz_page_list = team_obj_list
                                    .into_iter()
                                    .map(|e| {
                                        let team_id: String = match &e["id"] {
                                            Value::String(id) => id.clone(),
                                            _ => {
                                                error!("load_linear_team_timezones team [ 'id' ] was not Value::String");
                                                panic!("load_linear_team_timezones team [ 'id' ] was not Value::String");
                                            }
                                        };

                                        let team_timezone: String = match &e["timezone"] {
                                            Value::String(id) => id.clone(),
                                            _ => {
                                                error!("load_linear_team_timezones team [ 'timezone' ] was not Value::String");
                                                panic!("load_linear_team_timezones team [ 'timezone' ] was not Value::String");
                                            }
                                        };

                                        (team_id, team_timezone)
                                    })
                                    .collect();

            // append to team_tz_tuples
            team_tz_tuples.append(&mut team_tz_page_list);
            
            // Update GraphQLCursor
            match GraphQLCursor::linear_cursor_from_page_info(response["cursor_info"].clone()) {
                Some(new_cursor) => {
                    cursor = new_cursor;
                },
                None => {
                    error!("GraphQLCursor could not be created from response['cursor_info']: {:?}", response["cursor_info"]);
                    panic!("GraphQLCursor could not be created from response['cursor_info']: {:?}", response["cursor_info"]);
                },
            }

        }
        else {
            error!("load_linear_team_timezones Query Failed: {:?}", query_result);
            panic!("load_linear_team_timezones Query Failed: {:?}", query_result);
        }

    }


    team_tz_tuples
}

// Accept: due_date_str: &str ("yyyy-mm-dd")
// Return: (i32, i32, i32) (y, m, d)
fn parse_linear_issue_due_date (due_date_str: &str) -> Vec<i32> {
    let split = due_date_str.split('-');

    let vec: Vec<i32> = split
                        .map(|e| {
                            e.parse::<i32>().expect("parse_linear_issue_due_date parsing {e:?} to i32 failed")
                        })
                        .collect();
    if vec.len() != 3 {
        error!("parse_linear_issue_due_date due_date_str '{:?}' split into more than 3 components", due_date_str);
        panic!("parse_linear_issue_due_date due_date_str '{:?}' split into more than 3 components", due_date_str);
    }
    vec
}

// Accept:
//    team_tz_lookup: &HashMap<String,String>,
//    tz_offset_lookup: &HashMap<String, f64>
//    team_id: Value::String() || Value::Null,
//    due_date: Value::String('YYYY-MM-DD') || Value::Null
//    linear_config: &LinearConfig
// Return: FilterType::{ DueToday, Overdue, HasDueDate, DueSoon, NoDueDate }
pub fn get_issue_due_date_category( team_tz_lookup: &HashMap<String, String>,
                                    tz_offset_lookup: &HashMap<String, f64>,
                                    team_id: Value,
                                    due_date: Value,
                                    linear_config: &LinearConfig
                                ) -> FilterType {

    let team_id_str: String;
    let due_date_str: String;

    // get team_id as a String, if unsuccessful: panic
    match team_id.as_str() {
        Some(team_str) => {team_id_str = String::from(team_str)},
        None => {
            error!("get_issue_due_date_category team_id was not a Value::String");
            panic!("get_issue_due_date_category team_id was not a Value::String");
        }
    }

    // get due_date as a String or Value::Null, if unsuccessful panic
    match due_date {
        Value::String(date_str) => { due_date_str = date_str },
        Value::Null => { return FilterType::NoDueDate; },
        _ => {
            error!("get_issue_due_date_category due_date was not a Value::String or Value::Null");
            panic!("get_issue_due_date_category due_date was not a Value::String or Value::Null");
        }
    }

    // Lookup the tz name using 'team_tz_lookup'
    let tz_name: String = match team_tz_lookup.get(&team_id_str) {
        Some(tz_name_str) => { tz_name_str.to_string() },
        None => {
            error!("get_issue_due_date_category no timezone name found for team_id {:?}", team_id_str);
            panic!("get_issue_due_date_category no timezone name found for team_id {:?}", team_id_str)
        }
    };

    // Fetch f64 offset by matching timezone name 
    let offset: f64 = match tz_offset_lookup.get(&tz_name) {
        Some(offset_float) => { *offset_float },
        None => {
            error!("get_issue_due_date_category no float offset found for timezone name {:?}", tz_name);
            panic!("get_issue_due_date_category no float offset found for timezone name {:?}", tz_name)
        }
    };

    let hour: f64 = 3600.0;
    let utc_offset: i32 = (offset*hour).floor() as i32;

    
    // Current time without UTC offset
    let utc: DateTime<Utc> = Utc::now();

    // Current time with UTC offset
    let now_offset = FixedOffset::east(utc_offset)
                                    .ymd( utc.date().year(), utc.date().month(), utc.date().day() )
                                    .and_hms(utc.time().hour(), utc.time().minute(), utc.time().second());


    // Due date with UTC offset

    let due_date_components: Vec<i32> = parse_linear_issue_due_date(&due_date_str);

    let due_date_offset = FixedOffset::east(utc_offset)
                                        .ymd(due_date_components[0], due_date_components[1] as u32, due_date_components[2] as u32)
                                        .and_hms(23, 59, 59);

    
    let duration_diff: Duration = due_date_offset.signed_duration_since(now_offset);

    // Overdue
    if duration_diff.num_seconds() <= 0  {
        FilterType::Overdue
    }
    // DueToday
    else if duration_diff.num_days() == 0 && duration_diff.num_seconds() > 0 {
        FilterType::DueToday
    }
    // DueSoon
    else if duration_diff.num_days() >= (linear_config.due_soon_day_threshold as i64) {
        FilterType::DueSoon
    }
    else {
        FilterType::HasDueDate
    }

}