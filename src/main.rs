extern crate chrono;
extern crate reqwest;
extern crate serde_json;

mod gitlabclient;
mod issueretriever;
mod tuleapclient;

use gitlabclient::GitlabClient;
use issueretriever::IssueRetriever;
use tuleapclient::TuleapClient;

fn main() {
    let mut tc = TuleapClient::new(String::from("https://tuleap.ring.cx"), 15);
    let retriever = IssueRetriever::new(tc.get_artifacts());
    let gitlab_issues = retriever.tuleap_to_gitlab(tc);
    // TODO get from config file
    let gc = GitlabClient::new(String::from("https://git.lab"), String::from("xxxx"), String::from("xxxxxxxxxxxxxxxxxx"));
    for issue in gitlab_issues {
        gc.generate_issue(&issue);
    }
}
