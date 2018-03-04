use chrono::prelude::*;
use gitlabclient::GitlabIssue;
use serde_json::Value;
use tuleapclient::TuleapClient;
// TODO add log. avoid bad unwrapping

pub struct IssueRetriever {
    all_artifacts: Vec<Value>,
}

impl IssueRetriever {
    pub fn new(all_artifacts: Vec<Value>) -> IssueRetriever {
        IssueRetriever {
            all_artifacts: all_artifacts
        }
    }

    pub fn select_issues(&self) -> Vec<Value> {
        let mut result : Vec<Value> = Vec::new();
        let release_date = DateTime::parse_from_rfc3339("2017-07-21T00:00:00+02:00").unwrap();
        for artifact in &self.all_artifacts {
            let date = artifact["last_modified_date"].to_string();
            let end = date.len() - 1;
            let date = DateTime::parse_from_rfc3339(&date[1..end]).unwrap();
            // Get all issues not done or declined modified after the 1.0 release
            if artifact["status"] != "Done"
               && artifact["status"] != "Declined"
               && (date.timestamp() > release_date.timestamp()) {
                result.push(artifact.clone());
            }
        }

        result
    }

    pub fn tuleap_to_gitlab(&self, mut tuleap: TuleapClient) -> Vec<GitlabIssue> {
        let mut gitlab_issues: Vec<GitlabIssue> = Vec::new();
        let selected_issues = self.select_issues();
        // TODO improve with threads
        for issue in &selected_issues {
            // Retrieve base issue
            let details = tuleap.get_artifact_details(issue["id"].to_string());
            let mut project_url: String = String::from(""); // for now store platform
            let mut labels: Vec<String> = Vec::new();
            let mut assignee: String = String::from("");
            let mut description = String::from("Issue generated from Tuleap's migration script.\nOriginally submitted by: ");
            let mut sender: String;
            if details["submitted_by_details"].is_object() {
                sender = details["submitted_by_details"]["display_name"].to_string();
            } else {
                sender = details["submitted_by_user"]["display_name"].to_string();
            }
            description += &sender[1..(sender.len()-1)];
            let values = &details["values"];
            for v in values.as_array().unwrap() {
                let label = &v["label"];
                if label == "Platform" {
                    project_url = v["values"][0]["label"].to_string();
                } else if label == "Severity" {
                    labels.push(v["values"][0]["label"].to_string());
                } else if label == "Original Submission" {
                    description += "\n\n";
                    description += &v["value"].to_string();
                } else if label == "Assigned to" {
                    assignee = v["values"][0]["display_name"].to_string();
                }
            }
            // TODO get linked files?
            // Retrieve comments
            let comments_json = tuleap.get_artifact_comments(issue["id"].to_string());
            let mut comments: Vec<String> = Vec::new();
            for comment in comments_json {
                let mut comment_txt: String = String::from("Submitted by ");
                if comment["submitted_by_details"].is_object() {
                    sender = comment["submitted_by_details"]["display_name"].to_string();
                } else {
                    sender = comment["submitted_by_user"]["display_name"].to_string();
                }
                comment_txt += &sender[1..(sender.len()-1)];
                comment_txt += "\n\n";
                comment_txt += &comment["last_comment"]["body"].to_string();
                comments.push(comment_txt)
            }
            let issue = GitlabIssue {
                id: issue["id"].to_string(),
                title: issue["title"].to_string(),
                description: description,
                assignee: assignee,
                labels: labels,
                project_url: project_url,
                comments: comments
            };
            println!("Title: {}\nDescription: {}\nAssignee: {}\nLabels: {}\nPlatform: {}\n", issue.title, issue.description, issue.assignee, issue.labels[0], issue.project_url);
            for comment in &issue.comments {
                println!("{:?}", comment);
            }
            gitlab_issues.push(issue);
        }
        gitlab_issues
    }
}
