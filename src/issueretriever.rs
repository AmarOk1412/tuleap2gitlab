use chrono::prelude::*;
use gitlabclient::{GitlabIssue, GitlabComment};
use serde_json::Value;
use std::collections::HashMap;
use tuleapclient::TuleapClient;

pub struct IssueRetriever {
    all_artifacts: Vec<Value>,
    assignees_map: HashMap<String, String>
}

impl IssueRetriever {
    pub fn new(all_artifacts: Vec<Value>, assignees_map: HashMap<String, String>) -> IssueRetriever {
        IssueRetriever {
            all_artifacts: all_artifacts,
            assignees_map: assignees_map
        }
    }

    pub fn select_issues(&self) -> Vec<Value> {
        let mut result : Vec<Value> = Vec::new();
        let release_date = DateTime::parse_from_rfc3339("2017-07-21T00:00:00+02:00").unwrap();
        for artifact in &self.all_artifacts {
            let date = artifact["last_modified_date"].to_string();
            let end = date.len() - 1;
            let date = DateTime::parse_from_rfc3339(&date[1..end]).unwrap();
            // Get all issues not declined modified after the 1.0 release
            if artifact["status"] != "Declined"
               && (date.timestamp() > release_date.timestamp()) {
                result.push(artifact.clone());
            }
        }

        result
    }

    fn clean_txt(&self, txt: String) -> String {
        let mut result = txt.replace("\\r\\n", "  \n");
        result = result.replace("\\t", "\t");
        result = result.replace("\\\"", "\"");
        result = result.replace("\\\'", "'");
        result
    }

    fn no_quotes(&self, string: String) -> String {
        String::from(&string[1..(string.len()-1)])
    }

    pub fn tuleap_to_gitlab(&self, mut tuleap: TuleapClient) -> Vec<GitlabIssue> {
        let mut gitlab_issues: Vec<GitlabIssue> = Vec::new();
        let selected_issues = self.select_issues();
        // TODO improve with threads
        for issue in &selected_issues {
            // Retrieve base issue
            let details = tuleap.get_artifact_details(issue["id"].to_string());
            let title = self.clean_txt(self.no_quotes(issue["title"].to_string()));
            let created_at = self.clean_txt(self.no_quotes(issue["submitted_on"].to_string()));
            let mut project_url: String = String::from(""); // for now store platform
            let mut labels: Vec<String> = Vec::new();
            let mut assignee: String = String::from("");
            let mut description = String::from("Issue generated from Tuleap's migration script.\n**Originally submitted by: ");
            let mut sender: String;
            let mut closed = false;
            if details["submitted_by_details"].is_object() {
                sender = details["submitted_by_details"]["display_name"].to_string();
            } else {
                sender = details["submitted_by_user"]["display_name"].to_string();
            }
            description += &sender[1..(sender.len()-1)];
            description += "**";
            let values = &details["values"];
            for v in values.as_array().unwrap() {
                let label = &v["label"];
                if label == "Platform" {
                    project_url = v["values"][0]["label"].to_string();
                    project_url = self.no_quotes(project_url);
                } else if label == "Severity" {
                    let severity =  v["values"][0]["label"].to_string();
                    labels.push(self.no_quotes(severity));
                } else if label == "Original Submission" {
                    description += "\n\n";
                    let mut submission = v["value"].to_string();
                    description += &self.clean_txt(self.no_quotes(submission));
                } else if label == "Status" {
                    let mut status = v["values"][0]["label"].to_string();
                    closed = &status[1..(status.len()-1)] == "Done";
                } else if label == "Assigned to" {
                    assignee = v["values"][0]["username"].to_string();
                    assignee = match self.assignees_map.get(&self.no_quotes(assignee)) {
                        Some(a) => a.clone(),
                        None => String::from("")
                    };
                }
            }
            // TODO get linked files?
            // Retrieve comments
            let comments_json = tuleap.get_artifact_comments(issue["id"].to_string());
            let mut comments: Vec<GitlabComment> = Vec::new();
            for comment in comments_json {
                let mut comment_txt: String = String::from("**Submitted by ");
                if comment["submitted_by_details"].is_object() {
                    sender = comment["submitted_by_details"]["display_name"].to_string();
                } else {
                    sender = comment["submitted_by_user"]["display_name"].to_string();
                }
                comment_txt += &sender[1..(sender.len()-1)];
                comment_txt += "**\n\n";
                let mut body = comment["last_comment"]["body"].to_string();
                comment_txt += &self.clean_txt(self.no_quotes(body));
                comments.push(GitlabComment {
                    body: comment_txt,
                    created_at: self.no_quotes(comment["submitted_on"].to_string())
                })
            }
            let issue = GitlabIssue {
                title: title,
                closed: closed,
                description: description,
                assignee: assignee.clone(),
                labels: labels,
                project_url: project_url,
                created_at: created_at,
                comments: comments
            };
            /*println!("Title: {}\nDescription: {}\nAssignee: {}\nLabels: {}\nPlatform: {}\n", issue.title, issue.description, issue.assignee, issue.labels[0], issue.project_url);
            for comment in &issue.comments {
                println!("{:?}", comment);
            }*/
            if assignee != "" {
                gitlab_issues.push(issue);
            }
        }
        gitlab_issues
    }
}
