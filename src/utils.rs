// © Vlad Miricioiu <vlad@vlad.website>
// SPDX-License-Identifier: AGPL-3.0-only

use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub fn read_lines(filename: &str) -> io::Result<io::Lines<BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}
