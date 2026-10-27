// © Vlad Miricioiu <vlad@vlad.website>
// SPDX-License-Identifier: AGPL-3.0-only

use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

use publiccode::utils::read_lines;

const REPO_NAMES_PATH: &str = "./results/repo-names.txt";

#[derive(Deserialize, Debug)]
struct RepoCommitter {
    name: String,
    email: String,
    login: Option<String>,
    count: u64,
}

#[derive(Deserialize, Debug)]
struct RepoCommits {
    committers: Vec<RepoCommitter>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let http_client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("publiccode; contact vlad@vlad.website")
        .build()?;
    let mut commit_map: HashMap<String, u64> = HashMap::new();
    let repo_names: Vec<_> = read_lines(REPO_NAMES_PATH)?.collect();
    for repo_name in repo_names.into_iter().map_while(Result::ok) {
        let url = Url::parse(
            format!(
                "https://commits.ecosyste.ms/api/v1/hosts/GitHub/repositories/{}",
                repo_name
            )
            .as_str(),
        )?;
        eprintln!("url\t{:#?}", url.as_str());
        let response = http_client.get(url.as_str()).send().await?;
        let status = response.status();
        let json_string = response.text().await?;
        if status != 200 {
            eprintln!("got status {}", status);
            if status == 404 || status == 202 {
                continue;
            } else {
                eprintln!("res\t{:#?}", json_string);
                return Err(anyhow!("got status {}", status));
            }
        }
        eprintln!("json_string\t{:#?}", json_string);
        let commits: RepoCommits = serde_json::from_str(&json_string)?;
        eprintln!("commits\t{:#?}", commits);
        // *map.entry(key).or_insert(0) += 1
    }
    eprintln!("commit_map\t{:#?}", commit_map);
    Ok(())
}
