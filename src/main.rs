use anyhow::{anyhow, Context, Result};
use chrono::{offset::TimeZone, NaiveDateTime};
use chrono_tz::US::Pacific;
use clap::{App, Arg};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::copy;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;

static LIMIT: &str = "10";
static LIMIT_TEXT_OUTPUT: usize = 10;
const VID_URL: &str = "https://www.giantbomb.com/api/videos/?format=json&limit=";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
enum Resolution {
    HD,
    High,
    Low,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ReadConfig {
    path: PathBuf,
    time: chrono::DateTime<chrono::Utc>,
    gbkey: String,
    exclude: Vec<String>, //excludes certain names (partial matches)
    locked: bool,
    resolution: Resolution,
    write_to: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Config {
    config_path: PathBuf,
    read_config: ReadConfig,
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
    reqwest::blocking::get(q.as_str())
        .and_then(|s| s.json())
        .context("Error in parsing json to get list of videos")
}

/// query the giantbomb videos, filter them according to `date` and returns them
fn query_videos(config: &Config) -> Result<Vec<GiantBombVideo>> {
    let qstring = VID_URL.to_string() + LIMIT;
    let res = query_giantbomb(&config.read_config.gbkey, qstring).map(|v| v.results)?;
    Ok(res
        .into_iter()
        .filter(|v: &GiantBombVideo| {
            !config
                .read_config
                .exclude
                .iter()
                .any(|substr| v.name.contains(substr))
        })
        .filter(|v: &GiantBombVideo| {
            chrono::Utc.timestamp(
                from_giantbomb_datetime_to_timestamp(&v.publish_date).expect("Timestamp error"),
                0,
            ) > config.read_config.time
        })
        .collect())
}

fn get_config(path: &Path) -> Result<Config> {
    if let Ok(string) = fs::read_to_string(path) {
        let read_config: ReadConfig = toml::from_str(&string).context("Error in reading config")?;
        Ok(Config {
            config_path: path.to_owned(),
            read_config,
        })
    } else {
        let c = ReadConfig {
            gbkey: "NOTHING".to_string(),
            path: PathBuf::new(),
            time: chrono::Utc::now(),
            exclude: Vec::new(),
            locked: false,
            resolution: Resolution::HD,
            write_to: String::new(),
        };
        write_config(&Config {
            read_config: c,
            config_path: path.to_owned(),
        })?;
        panic!("Please adjust config");
    }
}

fn write_config(c: &Config) -> Result<()> {
    let string = toml::to_string(&c.read_config).context("Error in serializing config")?;
    fs::write(&c.config_path, string).context("Error in writing config")
}

fn download_video(config: &Config, vid: &GiantBombVideo) -> Result<()> {
    let mut path = config.read_config.path.clone();

    let url_to_grab = match config.read_config.resolution {
        Resolution::HD => &vid.hd_url,
        Resolution::High => &vid.high_url,
        Resolution::Low => &vid.low_url,
    };

    let url = url_to_grab
        .to_owned()
        .map(|v| v + "?api_key=" + &config.read_config.gbkey);

    if let Some(url) = url {
        //println!("Url: {}", url);
        let mut response = reqwest::blocking::get(&url).context("Could not find url")?;

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
        if let Err(e) = copy(&mut response, &mut dest) {
            fs::remove_file(path)?;
            return Err(e).context("Error in copying file");
        }
        Ok(())
    } else {
        Err(anyhow!("Could not read do url"))
    }
}

fn run(config: &mut Config) -> Result<()> {
    let videos = query_videos(config)?;
    println!("Found {} new videos", videos.len());
    for vid in &videos {
        download_video(config, vid).with_context(|| format!("Error in video {}", vid.name))?;
        config.read_config.time =
            from_giantbomb_datetime(&vid.publish_date).expect("Error in parsing time");
        write_config(config)?;
    }

    //write log file
    if !config.read_config.write_to.is_empty() {
        let f = File::open(&config.read_config.write_to);
        let new = videos.into_iter().map(|v| v.name);
        let all = if let Ok(mut f) = f {
            let mut buff = String::new();
            f.read_to_string(&mut buff)?;
            let old = buff.lines().take(LIMIT_TEXT_OUTPUT).map(String::from);
            new.chain(old).map(|s| s + "\n").collect::<String>()
        } else {
            new.map(|s| s + "\n").collect::<String>()
        };

        let mut write_f = File::create(&config.read_config.write_to)?;
        write_f.write_all(all.as_bytes())?;
    }

    Ok(())
}

/// Update giantbomb videos and put them into the database
pub fn main() {
    let matches = App::new("bombinger")
        .arg(
            Arg::with_name("config")
                .short("c")
                .required(true)
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .get_matches();

    let config_path: PathBuf = matches.value_of("config").unwrap_or("config.toml").into();

    let config = get_config(&config_path);
    if let Ok(mut config) = config {
        if config.read_config.locked {
            println!("Another instance is running (config file is locked). Aborting");
            return;
        }
        let checked_time = chrono::Utc::now();

        config.read_config.locked = true;
        write_config(&config).expect("Config in weird state, aborting");
        if let Err(e) = run(&mut config) {
            println!("Error in downloading");
            println!("E: {}", e);
            config.read_config.locked = false;
            write_config(&config).expect("Error in writing config, possible corrupt");
            println!("Not updating config time as we aborted");
            return;
        }

        config.read_config.time = checked_time;
        config.read_config.locked = false;
        write_config(&config).expect("Error in writing config, possible corrupted");
        println!("Finished downloading files");
    } else {
        println!("Config could not be loaded");
    }
}
