// © Vlad Miricioiu <vlad@vlad.website>
// SPDX-License-Identifier: AGPL-3.0-only

use anyhow::Result;
use std::fs::File;

const DOMAIN_COUNTS_PATH: &str = "./results/domain-counts.csv";

#[tokio::main]
async fn main() -> Result<()> {
    let mut n_gov_commits = 0;
    let mut n_civ_commits = 0;
    let file = File::open(DOMAIN_COUNTS_PATH)?;
    let mut reader = csv::Reader::from_reader(file);
    for result in reader.records() {
        let record = result?;
        let domain = record.get(0).unwrap();
        let count = record.get(1).unwrap().parse::<u64>()?;
        if domain.ends_with(".gov.uk") || domain.ends_with(".scot") {
            n_gov_commits += count;
        } else {
            n_civ_commits += count;
        }
    }
    println!("{} gov", n_gov_commits);
    println!("{} civ", n_civ_commits);
    Ok(())
}
