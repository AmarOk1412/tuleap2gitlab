extern crate chrono;
extern crate reqwest;
extern crate serde_json;

mod tuleapclient;
mod issueretriever;
mod gitlabclient;

use tuleapclient::TuleapClient;
use issueretriever::IssueRetriever;

fn main() {
    let mut tc = TuleapClient::new(String::from("https://tuleap.ring.cx"), 15);
    let retriever = IssueRetriever::new(tc.get_artifacts());
    retriever.tuleap_to_gitlab(tc);
    // TODO: publish all GitlabIssue via a gitlab client.
}
