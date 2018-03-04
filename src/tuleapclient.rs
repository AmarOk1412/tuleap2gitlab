use reqwest;

use serde_json::{Value, from_str};

pub struct TuleapClient {
    client: reqwest::Client,
    tracker_url: String,
    tracker_nb: u64

}

impl TuleapClient {
    pub fn new(tracker_url: String, tracker_nb: u64) -> TuleapClient {
        let client = reqwest::Client::new();

        TuleapClient {
            client: client,
            tracker_url: tracker_url,
            tracker_nb: tracker_nb
        }
    }

    pub fn get_artifacts(&mut self) -> Vec<Value> {
        let mut i = 0;
        let mut finish = false;
        let mut all_artifacts: Vec<Value> = Vec::new();
        while !finish {
            let url = format!("{}/api/trackers/{}/artifacts?offset={}", self.tracker_url, self.tracker_nb, i*100);
            let mut req = self.client.get(&*url)
                         .send().unwrap();
            let body = match req.text() {
                Ok(body) => body,
                Err(_) => String::from("")
            };
            let mut artifacts: Vec<Value> = from_str(&*body).unwrap();
            finish = artifacts.is_empty();
            all_artifacts.append(&mut artifacts);
            i += 1;
        }
        all_artifacts
    }

    pub fn get_artifact_details(&mut self, id: String) -> Value {
        let url = format!("{}/api/artifacts/{}", self.tracker_url, id);
        println!("{:?}", url);
        let mut req = self.client.get(&*url)
                     .send().unwrap();
        let body = match req.text() {
            Ok(body) => body,
            Err(_) => String::from("")
        };
        from_str(&*body).unwrap()
    }

    pub fn get_artifact_comments(&mut self, id: String) -> Vec<Value> {
        let url = format!("{}/api/artifacts/{}/changesets?fields=comments", self.tracker_url, id);
        println!("{:?}", url);
        let mut req = self.client.get(&*url)
                     .send().unwrap();
        let body = match req.text() {
            Ok(body) => body,
            Err(_) => String::from("")
        };
        from_str(&*body).unwrap()
    }
}
