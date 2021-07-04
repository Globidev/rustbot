use crate::{Error, PrefixContext};

use reqwest::header;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

pub struct CommandFlags {
    pub channel: Channel,
    pub mode: Mode,
    pub edition: Edition,
    pub warn: bool,
}

#[derive(Debug, Serialize)]
pub struct PlaygroundRequest<'a> {
    pub channel: Channel,
    pub edition: Edition,
    pub code: &'a str,
    #[serde(rename = "crateType")]
    pub crate_type: CrateType,
    pub mode: Mode,
    pub tests: bool,
}

#[derive(Debug, Serialize)]
pub struct MiriRequest<'a> {
    pub edition: Edition,
    pub code: &'a str,
}

// has the same fields
pub type MacroExpansionRequest<'a> = MiriRequest<'a>;

#[derive(Debug, Serialize)]
pub struct ClippyRequest<'a> {
    pub edition: Edition,
    #[serde(rename = "crateType")]
    pub crate_type: CrateType,
    pub code: &'a str,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    Stable,
    Beta,
    Nightly,
}

impl FromStr for Channel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "stable" => Ok(Channel::Stable),
            "beta" => Ok(Channel::Beta),
            "nightly" => Ok(Channel::Nightly),
            _ => Err(format!("invalid release channel `{}`", s).into()),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Edition {
    #[serde(rename = "2015")]
    E2015,
    #[serde(rename = "2018")]
    E2018,
    #[serde(rename = "2021")]
    E2021,
}

impl FromStr for Edition {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "2015" => Ok(Edition::E2015),
            "2018" => Ok(Edition::E2018),
            "2021" => Ok(Edition::E2021),
            _ => Err(format!("invalid edition `{}`", s).into()),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum CrateType {
    #[serde(rename = "bin")]
    Binary,
    #[serde(rename = "lib")]
    Library,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Debug,
    Release,
}

impl FromStr for Mode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "debug" => Ok(Mode::Debug),
            "release" => Ok(Mode::Release),
            _ => Err(format!("invalid compilation mode `{}`", s).into()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PlayResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Returns a gist ID
pub async fn post_gist(ctx: PrefixContext<'_>, code: &str) -> Result<String, Error> {
    let mut payload = HashMap::new();
    payload.insert("code", code);

    let resp = ctx
        .data
        .http
        .post("https://play.rust-lang.org/meta/gist/")
        .header(header::REFERER, "https://discord.gg/rust-lang-community")
        .json(&payload)
        .send()
        .await?;

    let mut resp: HashMap<String, String> = resp.json().await?;
    log::info!("gist response: {:?}", resp);

    let gist_id = resp.remove("id").ok_or("no gist found")?;
    Ok(gist_id)
}

pub fn url_from_gist(flags: &CommandFlags, gist_id: &str) -> String {
    format!(
        "https://play.rust-lang.org/?version={}&mode={}&edition={}&gist={}",
        match flags.channel {
            Channel::Nightly => "nightly",
            Channel::Beta => "beta",
            Channel::Stable => "stable",
        },
        match flags.mode {
            Mode::Debug => "debug",
            Mode::Release => "release",
        },
        match flags.edition {
            Edition::E2015 => "2015",
            Edition::E2018 => "2018",
            Edition::E2021 => "2021",
        },
        gist_id
    )
}
