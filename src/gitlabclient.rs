use reqwest;
use serde_json::{Value, from_str};
use std::collections::HashMap;

pub struct GitlabIssue {
    pub title: String,
    pub description: String,
    pub assignee: String,
    pub labels: Vec<String>,
    pub project_url: String,
    pub comments: Vec<String>
}


pub struct GitlabClient {
    client: reqwest::Client,
    gitlab_url: String,
    project: String,
    private_token: String,
}


impl GitlabClient {
    pub fn new(gitlab_url: String, project: String, private_token: String) -> GitlabClient {
        let client = reqwest::Client::new();

        GitlabClient {
            client: client,
            gitlab_url: gitlab_url,
            project: project,
            private_token: private_token
        }
    }

    pub fn generate_issue(&self, issue: &GitlabIssue) {
        let url = format!("{}/api/v4/projects/{}/issues?private_token={}", self.gitlab_url, self.project, self.private_token);
        println!("{:?}", url);

        // Generate first post
        let mut post = HashMap::new();
        post.insert("title", issue.title.clone());
        post.insert("description", issue.description.clone());
        post.insert("assignee", issue.assignee.clone());
        post.insert("labels", issue.labels[0].clone());

        // Create issue and retrieve iid
        let mut req = self.client.post(&*url)
                                 .json(&post)
                                 .send().unwrap();
        let body = match req.text() {
            Ok(body) => body,
            Err(_) => String::from("")
        };
        let result: Value = from_str(&*body).unwrap();
        let iid = result["iid"].to_string();

        // Post comments
        let url = format!("{}/api/v4/projects/{}/issues/{}/notes?private_token={}", self.gitlab_url, self.project, iid, self.private_token);
        for comment in issue.comments.clone() {
            println!("Post comment to issue {:?}", iid);
            let mut post = HashMap::new();
            post.insert("body", comment);

            // Create issue and retrieve iid
            self.client.post(&*url)
                       .json(&post)
                       .send().unwrap();
        }
    }
}
