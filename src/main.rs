extern crate chrono;
extern crate reqwest;
extern crate serde_json;
#[macro_use] extern crate log;
extern crate env_logger;

mod gitlabclient;
mod issueretriever;
mod tuleapclient;

use gitlabclient::GitlabClient;
use issueretriever::IssueRetriever;
use serde_json::{Value, from_str};
use std::fs::File;
use std::collections::HashMap;
use std::io::prelude::*;
use tuleapclient::TuleapClient;

fn main() {
    env_logger::init();

    // This script load config from config.json
    let mut file = File::open("config.json").ok()
        .expect("Config file not found");
    let mut config = String::new();
    file.read_to_string(&mut config).ok()
        .expect("failed to read!");
    let config: Value = from_str(&*config).ok()
                        .expect("Incorrect config file. Please check config.json");


    let tuleap_url = String::from(config["tuleap_url"].as_str().unwrap_or(""));
    let tuleap_tracker = config["tuleap_tracker"].as_u64().unwrap_or(0);
    info!("Will retrieve tuleap issues from {} tracker {}", tuleap_url, tuleap_tracker);
    let mut tc = TuleapClient::new(tuleap_url, tuleap_tracker);

    let mut assignees_map = HashMap::new();
    let assignees = &config["assignees"];
    match assignees.as_array() {
        Some(assignees) => {
            for a in assignees {
                assignees_map.insert(
                    String::from(a["username"].as_str().unwrap_or("")),
                    String::from(a["gitlab_id"].as_str().unwrap_or(""))
                );
            }
        },
        _ => {}
    }

    let mut projects_map = HashMap::new();
    let projects = &config["projects"];
    match projects.as_array() {
        Some(projects) => {
            for p in projects {
                projects_map.insert(
                    String::from(p["platform"].as_str().unwrap_or("")),
                    String::from(p["gitlab_id"].as_str().unwrap_or(""))
                );
            }
        },
        _ => {}
    }

    info!("Get interresting issues and build issues for gitlab");
    let retriever = IssueRetriever::new(tc.get_artifacts(),
                                        assignees_map, projects_map,
                                        String::from(config["file_dir"].as_str().unwrap_or("")));
    let gitlab_issues = retriever.tuleap_to_gitlab(tc);

    info!("Create gitlab issues");
    let gc = GitlabClient::new(String::from(config["gitlab_url"].as_str().unwrap_or("")),
                               String::from(config["gitlab_token"].as_str().unwrap_or("")));
    for issue in gitlab_issues {
        // TODO move into thread
        gc.generate_issue(&issue);
    }
}
