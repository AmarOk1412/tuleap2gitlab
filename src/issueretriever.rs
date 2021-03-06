use chrono::prelude::*;
use gitlabclient::{GitlabIssue, GitlabComment};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::remove_dir_all;
use tuleapclient::TuleapClient;

/**
 * Used to make the transition between tuleap and gitlab
 */
pub struct IssueRetriever {
    all_artifacts: Vec<Value>,
    assignees_map: HashMap<String, String>,
    project_map: HashMap<String, String>,
    file_dir: String
}

impl IssueRetriever {
    pub fn new(all_artifacts: Vec<Value>,
               assignees_map: HashMap<String, String>,
               project_map: HashMap<String, String>,
               file_dir: String) -> IssueRetriever {
        IssueRetriever {
            all_artifacts: all_artifacts,
            assignees_map: assignees_map,
            project_map: project_map,
            file_dir: file_dir
        }
    }

    /**
     * Migrate a text from tuleap to gitlab
     * @param string the text to migrate
     * @return string cleaned
     */
    fn clean_txt(&self, string: String) -> String {
        // First avoid weird line breaks from tuleap
        let mut result = string.replace("\\r\\n", "  \n");
        result = result.replace("\\n\\n", "  \n");
        result = result.replace("\\t", "\t");
        // will breaks some issues but will improve some others
        result = result.replace("\\n", "  \n");
        // protect json
        result = result.replace("\\\"", "\"");
        result = result.replace("\\\'", "'");
        result = result.replace("&#39;", "'");
        // avoid markdown's symbols... cause tuleap is not in md
        result = result.replace("^", "\\^");
        result = result.replace("#", "\\#");
        result = result.replace("*", "\\*");
        result = result.replace("_", "\\_");
        result = result.replace("~", "\\~");
        result
    }

    /**
     * Remove the first and last character if possible
     * @param string the String to clean
     * @return string without the first and last character
     */
    fn rm_first_and_last(&self, string: String) -> String {
        if string.len() < 2 {
            return string;
        }
        String::from(&string[1..(string.len()-1)])
    }

    /**
     * Prepare the migration via the tuleap client
     * @param tuleap the tuleap client
     * @return a vec of GitlabIssue for a GitlabClient
     */
    pub fn tuleap_to_gitlab(&self, mut tuleap: TuleapClient) -> Vec<GitlabIssue> {
        let _ = remove_dir_all(self.file_dir.clone());
        let mut gitlab_issues: Vec<GitlabIssue> = Vec::new();
        let release_date = DateTime::parse_from_rfc3339("2017-07-21T00:00:00+02:00").unwrap();

        // TODO improve with threads
        for issue in &self.all_artifacts {
            // Retrieve base issue
            let details = tuleap.get_artifact_details(issue["id"].to_string());
            let title = self.clean_txt(self.rm_first_and_last(issue["title"].to_string()));
            let created_at = self.clean_txt(self.rm_first_and_last(issue["submitted_on"].to_string()));
            let mut project_url: String = String::from(""); // for now store platform
            let mut labels: Vec<String> = Vec::new();
            let mut attachments: Vec<String> = Vec::new();
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
            if values.is_array() {
                for v in values.as_array().unwrap() {
                    let label = &v["label"];
                    if label == "Platform" {
                        project_url = v["values"][0]["label"].to_string();
                        project_url = match self.project_map.get(&self.rm_first_and_last(project_url)) {
                            Some(p) => p.clone(),
                            None => String::from("")
                        };
                        project_url = project_url;
                    } else if label == "Severity" {
                        let severity =  v["values"][0]["label"].to_string();
                        match self.rm_first_and_last(severity).chars().next() {
                            Some('1') => labels.push(String::from("S - Ordinary")),
                            Some('5') => labels.push(String::from("S - Major")),
                            Some('9') => labels.push(String::from("S - Critical")),
                            _ => continue
                        }
                    } else if label == "Original Submission" {
                        description += "\n\n";
                        let mut submission = v["value"].to_string();
                        description += &self.clean_txt(self.rm_first_and_last(submission));
                    } else if label == "Status" {
                        let status = v["values"][0]["label"].to_string();
                        let status = &status[1..(status.len()-1)];
                        closed = status == "Done" || status == "Declined";
                        if status == "Declined" {
                            labels.push(String::from("invalid"));
                            info!("mark issue {} as Declined", issue["id"].as_str().unwrap_or(""));
                        }
                        if !closed {
                            let date = details["last_modified_date"].to_string();
                            let end = date.len() - 1;
                            let date = DateTime::parse_from_rfc3339(&date[1..end]).unwrap_or(release_date);
                            if date.timestamp() < release_date.timestamp() {
                                labels.push(String::from("zombie"));
                                closed = true;
                                info!("mark issue {} as zombie", issue["id"].as_str().unwrap_or(""));
                            }
                        }
                    } else if label == "Attachments" {
                        let mut files_descriptions = &v["file_descriptions"];
                        if !files_descriptions.is_array() {
                            continue;
                        }
                        for desc in files_descriptions.as_array().unwrap() {
                            let name = self.rm_first_and_last(desc["name"].to_string());
                            let url = self.rm_first_and_last(desc["html_url"].to_string());
                            attachments.push(tuleap.get_file(url,
                                                             name,
                                                             self.file_dir.clone(),
                                                             issue["id"].to_string())
                                            );
                        }
                    } else if label == "Assigned to" {
                        assignee = v["values"][0]["username"].to_string();
                        assignee = match self.assignees_map.get(&self.rm_first_and_last(assignee)) {
                            Some(a) => a.clone(),
                            None => String::from("")
                        };
                    }
                }
            }
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
                comment_txt += &self.clean_txt(self.rm_first_and_last(body));
                comments.push(GitlabComment {
                    body: comment_txt,
                    created_at: self.rm_first_and_last(comment["submitted_on"].to_string())
                })
            }
            info!("New issue generated {}", title);
            debug!("{}", issue);
            let issue = GitlabIssue {
                title: title,
                closed: closed,
                description: description,
                assignee: assignee.clone(),
                labels: labels,
                project_url: project_url,
                created_at: created_at,
                comments: comments,
                attachments: attachments.clone()
            };
            gitlab_issues.push(issue);
        }
        gitlab_issues
    }
}
