// © Vlad Miricioiu <vlad@vlad.website>
// SPDX-License-Identifier: AGPL-3.0-only

use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use url::Url;

use publiccode::utils::read_lines;

const ORGS_LIST_PATH: &str = "./data/gb-orgs.txt";
const REPO_NAMES_PATH: &str = "./results/repo-names.txt";

#[derive(Deserialize, Debug)]
struct GithubRepo {
    full_name: String,
}

async fn get_repo_names_for_org(http_client: &Client, org_name: &str) -> Result<Vec<String>> {
    let mut page = 1;
    let mut repo_names: Vec<String> = Vec::new();
    loop {
        let url = Url::parse(
            format!(
                "https://api.github.com/orgs/{}/repos?per_page=100&page={}",
                org_name, page
            )
            .as_str(),
        )?;
        let response = http_client.get(url.as_str());
        let response = match env::var("GH_TOKEN") {
            Ok(val) => response.bearer_auth(val),
            Err(_) => response,
        };
        let response = response.send().await?;
        let status = response.status();
        let json_string = response.text().await?;
        if status != 200 {
            eprintln!("got status {}", status);
            if status == 404 {
                break;
            } else {
                eprintln!("res\t{:#?}", json_string);
                return Err(anyhow!("got status {}", status));
            }
        }
        let new_repos: Vec<GithubRepo> = serde_json::from_str(&json_string)?;
        let mut new_repo_names: Vec<_> = new_repos.into_iter().map(|d| d.full_name).collect();
        if new_repo_names.len() == 0 {
            break;
        }
        eprintln!("...\t{:#?}", new_repo_names.len());
        repo_names.append(&mut new_repo_names);
        page += 1;
    }
    Ok(repo_names)
}

async fn get_repo_names(http_client: &Client) -> Result<Vec<String>> {
    let mut repo_names: Vec<String> = Vec::new();
    let org_names: Vec<_> = read_lines(ORGS_LIST_PATH)?.collect();
    for org_name in org_names.into_iter().map_while(Result::ok) {
        eprintln!("org\t{:#?}", org_name);
        let mut new_repo_names = get_repo_names_for_org(&http_client, &org_name).await?;
        eprintln!("repo_names\t{:#?}", new_repo_names.len());
        repo_names.append(&mut new_repo_names);
        eprintln!("");
    }
    Ok(repo_names)
}

#[tokio::main]
async fn main() -> Result<()> {
    let http_client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("publiccode; contact vlad@vlad.website")
        .build()?;
    let repo_names = get_repo_names(&http_client).await?;
    eprintln!("repo_names\t{:#?}", repo_names.len());
    let mut file = File::create(REPO_NAMES_PATH)?;
    let _ = file.write_all(repo_names.join("\n").as_bytes());
    Ok(())
}
