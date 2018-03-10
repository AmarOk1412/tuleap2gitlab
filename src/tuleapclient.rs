use reqwest;
use serde_json::{Value, from_str};
use std::io::prelude::*;
use std::fs::{File, create_dir, metadata};

/**
 * Represent a tuleap client use to manipulate the API
 */
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

    /**
     * Retrieve all artifacts from a tracker
     * @return a vec of Json values from the API
     */
    pub fn get_artifacts(&mut self) -> Vec<Value> {
        let mut i = 0;
        let mut finish = false;
        let mut all_artifacts: Vec<Value> = Vec::new();
        while !finish {
            let url = format!("{}/api/trackers/{}/artifacts?offset={}", self.tracker_url, self.tracker_nb, i*100);
            let mut req = self.client.get(&*url)
                         .send().ok().expect("Failed to get artifacts");
            let body = match req.text() {
                Ok(body) => body,
                Err(_) => String::from("")
            };
            let mut artifacts: Vec<Value> = from_str(&*body).ok().expect("Failed to parse artifacts");
            finish = artifacts.is_empty();
            all_artifacts.append(&mut artifacts);
            i += 1;
        }
        all_artifacts
    }

    /**
     * Retrieve a detailled artifact from a tracker
     * @param id the id of the artifact
     * @return a Json value from the API
     */
    pub fn get_artifact_details(&mut self, id: String) -> Value {
        let url = format!("{}/api/artifacts/{}", self.tracker_url, id);
        let mut req = self.client.get(&*url)
                     .send().ok().expect("Failed to get artifact's details");
        let body = match req.text() {
            Ok(body) => body,
            Err(_) => String::from("")
        };
        from_str(&*body).ok().expect("Failed to parse artifact's details")
    }

    /**
     * Retrieve a file
     * @param url the url of the file
     * @param name the name of the file
     * @param file_dir the base directory to save data
     * @param id of the issue
     * @return final path for the file
     */
    pub fn get_file(&mut self, url: String, filename: String, file_dir: String, id: String) -> String {
        let url = format!("{}{}", self.tracker_url, url);
        let mut req = self.client.get(&*url)
                     .send().ok().expect("Failed to get file");
        let mut buf: Vec<u8> = vec![];
        let _ = req.copy_to(&mut buf);
        let _ = create_dir("data");
        let _ = create_dir(format!("{}/{}",file_dir, id));
        let mut final_path = format!("{}/{}/{}",file_dir, id, filename);
        let mut i = 0;
        while metadata(final_path.clone()).is_ok() {
            final_path = format!("{}/{}/{}{}",file_dir, id, i, filename);
            i += 1;
        }
        info!("create file: {}", final_path);
        let mut buffer = File::create(final_path.clone()).ok().expect("Failed to create file");
        buffer.write(buf.as_slice()).ok().expect("Failed to write buffer");
        String::from(final_path)
    }

    /**
     * Retrieve all comments from an artifact
     * @param id the id of the artifact
     * @return a vec of Json values from the API
     */
    pub fn get_artifact_comments(&mut self, id: String) -> Vec<Value> {
        let url = format!("{}/api/artifacts/{}/changesets?fields=comments", self.tracker_url, id);
        let mut req = self.client.get(&*url)
                     .send().ok().expect("Failed to get comments");
        let body = match req.text() {
            Ok(body) => body,
            Err(_) => String::from("")
        };
        from_str(&*body).ok().expect("Failed to parse comments")
    }
}
