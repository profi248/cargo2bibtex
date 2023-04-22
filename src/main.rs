use std::{fs, fmt::Write, env};

use chrono::Utc;
use toml::{Table, Value};
use crates_io_api::SyncClient;

fn main() {
    let mut path = "Cargo.toml".to_string();

    if env::args().len() > 1 {
        path = env::args().collect::<Vec<String>>()[1].clone();
    }

    let file = fs::read_to_string(path).expect("cannot open cargo.toml in the current directory (or it's invalid)");
    let table: Table = file.parse().expect("error parsing file");

    let mut deps: Vec<(String, String)> = Default::default();

    for (name, value) in table["dependencies"].as_table().expect("dependencies not found in file") {
        match value {
            Value::String(version) => deps.push((name.to_string(), version.to_string())),
            Value::Table(info) => {
                if info.contains_key("path") { continue };
                deps.push((name.to_string(), info["version"].as_str().expect("no version for: {name}").to_string()))
            },
            _ => panic!("invalid dependency: {name}")
        }
    }

    let client = SyncClient::new(
        "https://github.com/profi248/cargo2bibtex",
        std::time::Duration::from_millis(1000),
   ).unwrap();

   for dependency in deps {
        let authors = client.crate_owners(&dependency.0).expect("error retrieving crate owners");
        let info = client.get_crate(&dependency.0).expect("error retrieving crate info");

        if let Some(exact) = info.crate_data.exact_match { assert!(!exact, "crate {} not found", dependency.0); }

        let mut entry = String::from("@misc {");
        write!(entry, "rs-{},\n", dependency.0).unwrap();
        write!(entry, "\ttitle = {{{} {}}},\n", info.crate_data.name.replace("_", "\\_"), dependency.1).unwrap();
        write!(entry, "\turl = {{https://crates.io/crates/{}}},\n", dependency.0).unwrap();

        let users = authors.iter()
        .filter_map(|user| {
            match &user.kind {
                Some(k) if k == "user" => {
                    if let Some(name) = &user.name { Some(name.clone()) }
                    else { Some(format!("{{{}}}", &user.login)) }
                }
                _ => None
            } 
        }).collect::<Vec<String>>().join(" and ");

        write!(entry, "\tauthor = {{{}}},\n", users).unwrap();
        write!(entry, "\turldate = {{{}}},\n", Utc::now().format("%F")).unwrap();
        write!(entry, "\tyear = {{{}}},\n", info.crate_data.updated_at.format("%Y")).unwrap();
        write!(entry, "\tjournal = {{crates.io}},\n").unwrap();

        write!(entry, "}}").unwrap();

        println!("{entry}");
   }


}
