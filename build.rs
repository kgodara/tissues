// build.rs

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// parse file contents and remove new-lines
pub fn single_line_str_from_file(file_path: &str) -> String {

    let mut query_contents = fs::read_to_string(file_path).unwrap();

    query_contents = query_contents.as_str()
                                    .replace("\n", "");

    query_contents
}

pub fn gen_file_str_const(file_path: &str, const_name: &str) -> String {
    let parse = single_line_str_from_file(file_path);
    
    let mut result_str = String::from("pub const ");
    result_str.push_str(&const_name.to_ascii_uppercase());
    result_str.push_str(": &str = r#\"");
    result_str.push_str(parse.as_str());
    result_str.push_str("\"#;\n");

    result_str
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let query_dest_path = Path::new(&out_dir).join("query_raw.rs");
    let tz_dest_path = Path::new(&out_dir).join("tz_raw.rs");

    let base = env::current_dir().unwrap();

    let root_target: PathBuf = base.join("queries").join("linear");
    let query_root_name_list = &[
        "fetch_custom_views",
        "fetch_team_timezones",
        "fetch_viewer",
        "fetch_workflow_states",
    ];

    let mod_target: PathBuf = base.join("queries").join("linear").join("issue_modifications");
    let query_issue_mod_name_list = &[
        "set_issue_workflow_state",
        "set_issue_assignee",
        "set_issue_project",
        "set_issue_cycle",
        "set_issue_title",
    ];

    let issue_fetch_target: PathBuf = base.join("queries").join("linear").join("issues");
    let query_issue_fetch_name_list = &[
        "fetch_all_issues",
        "fetch_issues_by_assignee",
        "fetch_issues_by_content",
        "fetch_issues_by_creator",
        "fetch_issues_by_label",
        "fetch_issues_by_project",
        "fetch_issues_by_team",
        "fetch_issues_by_workflow_state",
        "fetch_issues_single_query",
    ];

    let op_fetch_target: PathBuf = base.join("queries").join("linear").join("op_fetch");
    let query_op_fetch_name_list = &[
        "get_cycles_by_team",
        "get_projects_by_team",
        "get_users_by_team",
        "get_workflow_states_by_team"
    ];


    let mut query_result_str = String::default();
    let mut tz_result_str = String::default();


    for x in query_root_name_list.iter() {
        let query_file_name = format!("{}.graphql", x);
        let query_target = root_target.join(query_file_name);
        query_result_str.push_str(&gen_file_str_const(query_target.into_os_string().into_string().unwrap().as_str(), x));
    }

    for x in query_issue_mod_name_list.iter() {
        let query_file_name = format!("{}.graphql", x);
        let query_target = root_target.join("issue_modifications").join(query_file_name);
        query_result_str.push_str(&gen_file_str_const(query_target.into_os_string().into_string().unwrap().as_str(), x));
    }

    for x in query_issue_fetch_name_list.iter() {
        let query_file_name = format!("{}.graphql", x);
        let query_target = root_target.join("issues").join(query_file_name);
        query_result_str.push_str(&gen_file_str_const(query_target.into_os_string().into_string().unwrap().as_str(), x));
    }

    for x in query_op_fetch_name_list.iter() {
        let query_file_name = format!("{}.graphql", x);
        let query_target = root_target.join("op_fetch").join(query_file_name);
        query_result_str.push_str(&gen_file_str_const(query_target.into_os_string().into_string().unwrap().as_str(), x));
    }

    let tz_target = base.join("data").join("timezones.json");
    tz_result_str.push_str(&gen_file_str_const(tz_target.into_os_string().into_string().unwrap().as_str(), "timezones"));


    fs::write(
        &query_dest_path,
        query_result_str,
    ).unwrap();

    fs::write(
        &tz_dest_path,
        tz_result_str,
    ).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=queries");
}