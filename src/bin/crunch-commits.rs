// © Vlad Miricioiu <vlad@vlad.website>
// SPDX-License-Identifier: AGPL-3.0-only

use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use url::Url;

use publiccode::utils::read_lines;

const REPO_NAMES_PATH: &str = "./results/repo-names.txt";
const DOMAIN_COUNTS_PATH: &str = "./results/domain-counts.csv";

#[derive(Deserialize, Debug)]
struct RepoCommitter {
    // name: String,
    email: String,
    // login: Option<String>,
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
    let mut n_repos_fetched = 0;
    let mut domain_counter: HashMap<String, u64> = HashMap::new();
    let repo_names: Vec<_> = read_lines(REPO_NAMES_PATH)?.collect();
    let n_repos = repo_names.len();
    for (idx, repo_name) in repo_names.into_iter().map_while(Result::ok).enumerate() {
        let url = Url::parse(
            format!(
                "https://commits.ecosyste.ms/api/v1/hosts/GitHub/repositories/{}",
                repo_name
            )
            .as_str(),
        )?;
        let response = http_client.get(url.as_str()).send().await?;
        let status = response.status();
        let json_string = response.text().await?;
        eprintln!(
            "{:<30}{} {}",
            format!("{:05}/{:05} ({:05})", idx, n_repos, n_repos_fetched),
            status.as_u16(),
            repo_name
        );
        if status != 200 {
            if status == 404 || status == 202 {
                continue;
            } else {
                eprintln!("{:<30}{}", "res", json_string);
                return Err(anyhow!("got status {}", status));
            }
        }
        n_repos_fetched += 1;
        let commits: RepoCommits = serde_json::from_str(&json_string)?;
        for committer in &commits.committers {
            if !committer.email.contains("@") {
                continue;
            }
            let Some(prefix) = committer.email.split("@").nth(1) else {
                continue;
            };
            *domain_counter.entry(prefix.to_string()).or_insert(0) += committer.count;
        }
    }
    let mut domain_counter_vec: Vec<(&String, &u64)> = domain_counter.iter().collect();
    domain_counter_vec.sort_by(|a, b| b.1.cmp(&a.1));
    let mut file = File::create(DOMAIN_COUNTS_PATH)?;
    for domain_count in &domain_counter_vec {
        let _ = file.write_all(format!("{},{}\n", domain_count.0, domain_count.1).as_bytes());
    }
    eprintln!(
        "Wrote {} rows for {} fetched repos",
        domain_counter_vec.len(),
        n_repos_fetched
    );
    Ok(())
}
