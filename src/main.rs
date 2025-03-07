use clap::{Parser, Subcommand};
use log::{error, info};
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::redirect;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::process::Command;

#[derive(Parser)]
#[command(name = "kernel-overlay")]
#[command(about = "Kernel overlay updater", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    UpdateSources,
    UpdateWorkflow,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Meta {
    categories: HashMap<String, serde_json::Value>,
    #[serde(default)]
    longterm: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Package {
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SourceMeta {
    #[serde(rename = "link")]
    link: String,
    #[serde(rename = "pgp")]
    pgp: String,
    #[serde(rename = "changelog")]
    changelog: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Source {
    category: String,
    checksum: String,
    date: String,
    url: String,
    package: Package,
    #[serde(rename = "meta")]
    meta: SourceMeta,
    version: String,
}

fn to_number(v: &str) -> f64 {
    let re = Regex::new(r"(\d+)").unwrap();
    let mut i = 100.0;
    let mut n = 0.0;
    for cap in re.captures_iter(v) {
        if let Some(m) = cap.get(1) {
            let start = m.start();
            let valid = start == 0 || matches!(v.chars().nth(start - 1), Some('.' | '-'));
            if valid {
                let d: f64 = m.as_str().parse().unwrap_or(0.0);
                n *= i;
                n += d;
                i *= i;
            }
        }
    }
    if let Some(rc_cap) = Regex::new(r"-rc(\d+)").unwrap().captures(v) {
        if let Some(rc) = rc_cap.get(1) {
            n += 0.1 * rc.as_str().parse::<f64>().unwrap_or(0.0);
        }
    }
    n
}

fn trim_version(version: &str) -> Option<String> {
    let re = Regex::new(r"^(\d+)[.](\d+)").unwrap();
    if let Some(cap) = re.captures(version) {
        let major = cap.get(1)?.as_str();
        let minor = cap.get(2)?.as_str();
        return Some(format!("{}_{}", major, minor));
    }
    None
}

fn first_href(items: &[scraper::ElementRef], idx: usize) -> String {
    if idx >= items.len() {
        return String::new();
    }
    let sel = Selector::parse("a[href]").unwrap();
    if let Some(a) = items[idx].select(&sel).next() {
        a.value().attr("href").unwrap_or("").to_string()
    } else {
        String::new()
    }
}

fn load_all_checksums(client: &Client, url: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    if let Ok(res) = client.get(url).send() {
        if res.status().is_success() {
            if let Ok(body) = res.text() {
                let re = Regex::new(r"([0-9a-f]+)\s+(\S+)").unwrap();
                for cap in re.captures_iter(&body) {
                    if let (Some(hash), Some(name)) = (cap.get(1), cap.get(2)) {
                        result.insert(name.as_str().to_string(), hash.as_str().to_string());
                    }
                }
            }
        }
    }
    result
}

fn checksum(client: &Client, checksum_url: &str, url: &str) -> String {
    let name = url.split('/').last().unwrap_or("");
    let mut checksums = load_all_checksums(client, checksum_url);
    if let Some(hash) = checksums.remove(name) {
        return hash;
    }

    let output = Command::new("wget")
        .args(["-q", "-O-", "--timeout=30", url])
        .output();

    if let Ok(output) = output {
        let mut hasher = Sha256::new();
        hasher.update(&output.stdout);
        let result = hasher.finalize();
        return hex::encode(result);
    }

    String::new()
}

fn cmd_update_sources() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .redirect(redirect::Policy::limited(10))
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let meta: Meta = serde_json::from_str(&fs::read_to_string("meta.json")?)?;

    let res = client.get("https://kernel.org").send()?;
    let body = res.text()?;
    let dom = Html::parse_document(&body);

    let releases_sel = Selector::parse("#releases").unwrap();
    let releases = dom
        .select(&releases_sel)
        .next()
        .ok_or("No releases table")?;

    let tr_sel = Selector::parse("tr").unwrap();
    let td_sel = Selector::parse("td").unwrap();

    let mut sources: Vec<Source> = Vec::new();

    for row in releases.select(&tr_sel) {
        let items: Vec<_> = row.select(&td_sel).collect();
        if items.len() < 10 {
            continue;
        }

        let category_text = items[0].text().collect::<String>();
        let category = category_text
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>();

        if !meta.categories.contains_key(&category) {
            continue;
        }

        let version_raw = items[1].text().collect::<String>();
        let version: String = version_raw
            .chars()
            .map(|c| if c == ' ' { '-' } else { c })
            .filter(|c| !c.is_ascii_punctuation() || *c == '-' || *c == '.')
            .collect::<String>();

        let date = items[2].text().collect::<String>();
        let tarball = first_href(&items, 3);
        let pgp = first_href(&items, 4);
        let browse = first_href(&items, 8);
        let changelog = first_href(&items, 9);

        let mut checksum_url = tarball.clone();
        if let Some(pos) = checksum_url.rfind('/') {
            checksum_url = format!("{}sha256sums.asc", &checksum_url[..pos + 1]);
        }

        let checksum = checksum(&client, &checksum_url, &tarball);

        let mut version = version;
        let re_rc = Regex::new(r"(\d+)[.](\d+)[-](\w+)").unwrap();
        if re_rc.is_match(&version) {
            version = re_rc.replace(&version, "$1.$2.0-$3").to_string();
        }
        let re_plain = Regex::new(r"^[0-9]+[.][0-9]+$").unwrap();
        if re_plain.is_match(&version) {
            version = format!("{}.0", version);
        }

        let pversion = trim_version(&version).unwrap_or_default();

        let mut category = category.clone();
        if version.to_lowercase().contains("eol") {
            version = version.to_lowercase().replace("-eol", "");
            category = "eol".to_string();
        }

        let package_name = if category == "stable" || category == "mainline" {
            category.clone()
        } else {
            pversion
        };

        sources.push(Source {
            category,
            checksum,
            date,
            url: tarball,
            package: Package { name: package_name },
            meta: SourceMeta {
                link: browse,
                pgp,
                changelog,
            },
            version,
        });
    }

    sources.sort_by(|a, b| a.version.cmp(&b.version));

    let json = serde_json::to_string_pretty(&sources)?;
    fs::write("sources.json", json)?;

    Ok(())
}

fn cmd_update_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let sources: Vec<Source> = serde_json::from_str(&fs::read_to_string("sources.json")?)?;
    let workflow_str = fs::read_to_string(".github/workflows/main.yml")?;
    let mut workflow: serde_json::Value = serde_json::from_str(&workflow_str)?;

    let mut versions: Vec<String> = sources.iter().map(|s| s.package.name.clone()).collect();
    versions.sort_by(|a, b| b.cmp(a));

    if let Some(matrix) = workflow
        .get_mut("jobs")
        .and_then(|j| j.get_mut("build"))
        .and_then(|b| b.get_mut("strategy"))
        .and_then(|s| s.get_mut("matrix"))
        .and_then(|m| m.get_mut("version"))
    {
        *matrix = serde_json::Value::Array(
            versions
                .iter()
                .map(|v| serde_json::Value::String(v.clone()))
                .collect(),
        );
    }

    let workflow_json = serde_json::to_string_pretty(&workflow)?;
    fs::write(".github/workflows/main.yml", workflow_json)?;

    update_readme(&sources)?;

    Ok(())
}

fn update_readme(sources: &[Source]) -> Result<(), Box<dyn std::error::Error>> {
    let mut sources = sources.to_vec();
    sources.sort_by(|a, b| {
        to_number(&b.version)
            .partial_cmp(&to_number(&a.version))
            .unwrap()
    });

    let mut table = String::from("|Version|Package|Date|\n|---|---|---|\n");
    for s in &sources {
        table.push_str(&format!("|{}|{}|{}|\n", s.version, s.package.name, s.date));
    }

    let content = fs::read_to_string("README.md")?;
    let re = Regex::new(r"(?s)<!--START-->.*?<!--END-->").unwrap();
    let new_content = re
        .replace(&content, format!("<!--START-->\n{}\n<!--END-->", table))
        .into_owned();
    fs::write("README.md", new_content)?;

    Ok(())
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match cli.command {
        Commands::UpdateSources => {
            info!("Starting update-sources");
            if let Err(e) = cmd_update_sources() {
                error!("update-sources failed: {}", e);
                std::process::exit(1);
            }
            info!("update-sources completed successfully");
        }
        Commands::UpdateWorkflow => {
            info!("Starting update-workflow");
            if let Err(e) = cmd_update_workflow() {
                error!("update-workflow failed: {}", e);
                std::process::exit(1);
            }
            info!("update-workflow completed successfully");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_number_stable() {
        assert!(to_number("6.18.3") > to_number("6.12.0"));
    }

    #[test]
    fn test_to_number_longterm() {
        assert!(to_number("6.1.159") > to_number("5.15.197"));
    }

    #[test]
    fn test_to_number_rc() {
        let rc = to_number("6.19.0-rc4");
        assert!(rc > 0.0);
        assert!(to_number("6.19.0-rc1") < to_number("6.19.0-rc4"));
    }

    #[test]
    fn test_trim_version() {
        assert_eq!(trim_version("6.18"), Some("6_18".to_string()));
        assert_eq!(trim_version("5.15"), Some("5_15".to_string()));
    }

    #[test]
    fn test_trim_version_invalid() {
        assert_eq!(trim_version("rc1"), None);
        assert_eq!(trim_version(""), None);
    }
}
