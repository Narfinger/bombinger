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

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    path: PathBuf,
    time: chrono::DateTime<chrono::Utc>,
    gbkey: String,
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
    pub deck: String,
    pub hd_url: Option<String>,
    pub youtube_id: Option<String>,
    pub name: String,
    pub length_seconds: i64,
    pub publish_date: String,
    pub site_detail_url: String,
    pub image: GiantBombThumbnail,
    pub video_show: GiantBombVideoShow,
}

pub fn from_giantbomb_datetime_to_timestamp(s: &str) -> i64 {
    let dt = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .unwrap_or_else(|e| panic!("Error: {}", e));
    let dttz = Pacific.from_local_datetime(&dt).unwrap();
    dttz.timestamp()
}

/// query giantbomb api and returns the result in a `GiantBombResult<T>`
fn query_giantbomb<T>(t: &str, url: String) -> GiantBombResult<T>
where
    T: serde::de::DeserializeOwned,
{
    let mut q = url;
    q.push_str("&api_key=");
    q.push_str(&t);
    //println!("Query: {}", q);
    reqwest::get(q.as_str())
        .and_then(|mut s| s.json())
        .unwrap_or_else(|e| panic!("error in json parsing: {}", e))
}

/// query the giantbomb videos, filter them according to `date` and returns them
fn query_videos(config: &Config) -> Vec<GiantBombVideo> {
    let qstring = VID_URL.to_string() + LIMIT;
    let res = query_giantbomb(&config.gbkey, qstring).results;
    res.into_iter()
        .filter(|v: &GiantBombVideo| {
            chrono::Utc.timestamp(from_giantbomb_datetime_to_timestamp(&v.publish_date), 0)
                > config.time
        })
        .collect()
}

fn get_config() -> Config {
    if let Ok(string) = fs::read_to_string("config.toml") {
        toml::from_str(&string).expect("Error in reading config")
    } else {
        let c = Config {
            gbkey: "NOTHING".to_string(),
            path: PathBuf::new(),
            time: chrono::Utc::now(),
        };
        let string = toml::to_string(&c).expect("Error in formating default config");
        fs::write("config.toml", string).expect("Error in writing default config");
        panic!("Please adjust config");
    }
}

fn write_config(c: &Config) {
    let string = toml::to_string(c).expect("Error in serializing config");
    fs::write("config.toml", string).expect("Error in writing config");
}
fn download_video(config: &Config, vid: &GiantBombVideo) -> reqwest::Result<()> {
    let mut path = config.path.clone();
    let mut response = reqwest::get(vid.hd_url.as_ref().expect("Could not find url"))?;

    path.push(format!(
        "{}-{}-{}.mp4",
        vid.publish_date, vid.video_show.title, vid.name
    ));

    let mut dest = File::create(&path).expect("Could not do file");
    copy(&mut response, &mut dest).expect("error in copy");
    Ok(())
}

/// Update giantbomb videos and put them into the database
pub fn main() {
    let mut config = get_config();
    let videos = query_videos(&config);
    for vid in videos {
        let res = download_video(&config, &vid);
        if res.is_err() {
            println!(
                "download failed of {}, {}",
                vid.name,
                vid.hd_url.unwrap_or("".to_string())
            );
        }
        config.time = chrono::Utc::now();
    }
    write_config(&config);
}
