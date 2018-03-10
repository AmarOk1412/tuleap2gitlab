use reqwest;
use serde_json::{Value, from_str};
use std::collections::HashMap;
use std::fmt;

/**
 * Represent a gitlab comment
 **/
pub struct GitlabComment {
    pub body: String,
    pub created_at: String
}
// Used for println!
impl fmt::Display for GitlabComment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.body, self.created_at)
    }
}

/**
 * Represent a gitlab issue
 **/
pub struct GitlabIssue {
    pub title: String,
    pub closed: bool,
    pub description: String,
    pub assignee: String,
    pub labels: Vec<String>,
    pub project_url: String,
    pub created_at: String,
    pub comments: Vec<GitlabComment>,
    pub attachments: Vec<String>
}
// Used for println!
impl fmt::Display for GitlabIssue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({}) assigned to {}\n{}\n{}", self.title, self.closed, self.assignee, self.created_at, self.description)
    }
}

/**
 * client used to generate issues on gitlab
 **/
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

    /**
     * Generate a gitlab issue on a tracker from the API
     * @param issue to generate
     */
    pub fn generate_issue(&self, issue: &GitlabIssue) {
        let url = format!("{}/api/v4/projects/{}/issues?private_token={}", self.gitlab_url, self.project, self.private_token);

        let mut description: String = issue.description.clone();

        for attachment in issue.attachments.clone() {
            let url = format!("{}/api/v4/projects/{}/uploads?private_token={}", self.gitlab_url, self.project, self.private_token);

            let form = reqwest::multipart::Form::new()
                        .file("file", attachment.clone()).unwrap();
            let mut req = self.client.post(&*url)
                                 .multipart(form)
                                 .send().unwrap();
            let body = match req.text() {
                Ok(body) => body,
                Err(_) => String::from("")
            };
            let result: Value = from_str(&*body).unwrap();
            let md = result["markdown"].to_string();
            info!("Post new file: {}", attachment);
            debug!("{}", body);
            description += "  \n";
            description += &md[1..(md.len()-1)];
        }

        // Generate first post
        let mut post = HashMap::new();
        post.insert("title", issue.title.clone());
        post.insert("description", description);
        post.insert("assignee_id", issue.assignee.clone());
        post.insert("created_at", issue.created_at.clone());
        post.insert("labels", issue.labels[0].clone());
        info!("Generate new issue: {}", issue.title);
        debug!("{}", issue);

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
        for comment in issue.comments.iter() {
            let mut post = HashMap::new();
            post.insert("body", comment.body.clone());
            post.insert("created_at", comment.created_at.clone());
            info!("Generate new comment for {}", issue.title);
            debug!("{}", comment);

            // Create issue and retrieve iid
            self.client.post(&*url)
                       .json(&post)
                       .send().unwrap();
        }

        // Lock issue if done
        if issue.closed {
            let url = format!("{}/api/v4/projects/{}/issues/{}?private_token={}&state_event=close", self.gitlab_url, self.project, iid, self.private_token);
            info!("Close issue {}", issue.title);

            // Create issue and retrieve iid
            self.client.put(&*url)
                       .send().unwrap();
        }
    }
}
