use anyhow::{anyhow, Context, Result};
use chrono::{offset::TimeZone, NaiveDateTime};
use chrono_tz::US::Pacific;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::copy;
use std::path::PathBuf;
use toml;

static LIMIT: &str = "10";
const VID_URL: &str = "https://www.giantbomb.com/api/videos/?format=json&limit=";

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Config {
    path: PathBuf,
    time: chrono::DateTime<chrono::Utc>,
    gbkey: String,
    exclude: Vec<String>, //excludes certain names (partial matches)
    locked: bool,
}

#[derive(Debug, Deserialize)]
struct GiantBombResult<T> {
    results: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct GiantBombThumbnail {
    pub medium_url: String,
    pub small_url: String,
}

#[derive(Debug, Deserialize)]
struct GiantBombVideoShow {
    pub title: String,
}

#[derive(Debug, Deserialize)]
struct GiantBombVideo {
    pub guid: String,
    pub deck: String,
    pub hd_url: Option<String>,
    pub high_url: Option<String>,
    pub low_url: Option<String>,
    pub youtube_id: Option<String>,
    pub name: String,
    pub length_seconds: i64,
    pub publish_date: String,
    pub site_detail_url: String,
    pub image: GiantBombThumbnail,
    pub video_show: Option<GiantBombVideoShow>,
}

pub fn from_giantbomb_datetime_to_timestamp(s: &str) -> Option<i64> {
    from_giantbomb_datetime(s).map(|v| v.timestamp())
}

fn from_giantbomb_datetime(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    let dt = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .unwrap_or_else(|e| panic!("Error: {}", e));
    Pacific
        .from_local_datetime(&dt)
        .map(|v| v.with_timezone(&chrono::Utc))
        .single()
}

/// query giantbomb api and returns the result in a `GiantBombResult<T>`
fn query_giantbomb<T>(t: &str, url: String) -> Result<GiantBombResult<T>>
where
    T: serde::de::DeserializeOwned,
{
    let mut q = url;
    q.push_str("&api_key=");
    q.push_str(&t);
    //println!("Query: {}", q);
    reqwest::get(q.as_str())
        .and_then(|mut s| s.json())
        .context("Error in parsing json to get list of videos")
}

/// query the giantbomb videos, filter them according to `date` and returns them
fn query_videos(config: &Config) -> Result<Vec<GiantBombVideo>> {
    let qstring = VID_URL.to_string() + LIMIT;
    let res = query_giantbomb(&config.gbkey, qstring).map(|v| v.results)?;
    Ok(res
        .into_iter()
        .filter(|v: &GiantBombVideo| !config.exclude.iter().any(|substr| v.name.contains(substr)))
        .filter(|v: &GiantBombVideo| {
            chrono::Utc.timestamp(
                from_giantbomb_datetime_to_timestamp(&v.publish_date).expect("Timestamp error"),
                0,
            ) > config.time
        })
        .collect())
}

fn get_config() -> Result<Config> {
    if let Ok(string) = fs::read_to_string("config.toml") {
        toml::from_str(&string).context("Error in reading config")
    } else {
        let c = Config {
            gbkey: "NOTHING".to_string(),
            path: PathBuf::new(),
            time: chrono::Utc::now(),
            exclude: Vec::new(),
            locked: false,
        };
        write_config(&c)?;
        panic!("Please adjust config");
    }
}

fn write_config(c: &Config) -> Result<()> {
    let string = toml::to_string(c).context("Error in serializing config")?;
    fs::write("config.toml", string).context("Error in writing config")
}

fn download_video(config: &Config, vid: &GiantBombVideo) -> Result<()> {
    let mut path = config.path.clone();

    let url = vid
        .hd_url
        .to_owned()
        .map(|v| v + "?api_key=" + &config.gbkey);

    if let Some(url) = url {
        //println!("Url: {}", url);
        let mut response = reqwest::get(&url).context("Could not find url")?;

        path.push(format!(
            "{}-{}-{}-{}.mp4",
            vid.guid,
            vid.publish_date,
            &vid.video_show
                .as_ref()
                .map(|s| &s.title)
                .unwrap_or(&"".to_string()),
            vid.name.replace("/", "")
        ));

        println!("Downloading {:?}", path.to_str());

        let mut dest = File::create(&path).context("Could not do file")?;
        copy(&mut response, &mut dest).context("error in copy")?;
        Ok(())
    } else {
        Err(anyhow!("Could not read do url"))
    }
}

fn run(config: &mut Config) -> Result<()> {
    let videos = query_videos(config)?;
    println!("Found {} new videos", videos.len());
    for vid in videos {
        download_video(config, &vid).with_context(|| format!("Error in video {}", vid.name))?;
        config.time = from_giantbomb_datetime(&vid.publish_date).expect("Error in parsing time");
        write_config(config)?;
    }
    Ok(())
}

/// Update giantbomb videos and put them into the database
pub fn main() {
    let config = get_config();
    if let Ok(mut config) = config {
        if config.locked {
            println!("Another instance is running (config file is locked). Aborting");
            return;
        }

        config.locked = true;
        write_config(&config).expect("Config in weird state, aborting");
        if let Err(e) = run(&mut config) {
            println!("Error in downloading");
            println!("E: {}", e);
        }

        config.time = chrono::Utc::now();
        config.locked = false;
        write_config(&config).expect("Error in writing config, possible corrupted");
        println!("Finished downloading files");
    } else {
        println!("Config could not be loaded");
    }
}
