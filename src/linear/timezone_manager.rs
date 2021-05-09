use std::fs;
use std::collections::HashMap;

use crate::errors::TimeZoneParseError;

const TIMEZONE_JSON_PATH: &str = "data/timezones.json";


pub fn parse_timezones_from_file(file_path: &str) -> HashMap<String, f64> {

    let mut query_contents = fs::read_to_string(file_path).unwrap();

    
    query_contents = query_contents.as_str()
                                    .replace("\n", "");
    
    // Parse the string of data into serde_json::Value.
    let time_zone_array: serde_json::Value = serde_json::from_str(query_contents.as_str()).unwrap();

    let mut location_to_offset_map = HashMap::new();

    match time_zone_array.as_array() {
        Some(time_zone_vec) => {
            for time_zone_obj in time_zone_vec.iter() {
                
                let time_zone_location = time_zone_obj["location"].clone();
                let time_zone_offset = time_zone_obj["decimal_offset"].clone();

                let mut location_name: String;
                let mut offset: f64;

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



pub struct TimeZoneManager {
    pub location_name_offset_map: HashMap<String, f64>,
}


impl Default for TimeZoneManager {
    fn default() -> TimeZoneManager {
        TimeZoneManager {
            location_name_offset_map: parse_timezones_from_file(&TIMEZONE_JSON_PATH)
        }
    }
}