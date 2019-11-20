use chrono::{offset::TimeZone, NaiveDateTime};
use chrono_tz::US::Pacific;
use serde::Deserialize;
use std::fs::File;
use std::io::copy;
use std::path::PathBuf;

static LIMIT: &str = "10";
const VID_URL: &str = "https://www.giantbomb.com/api/videos/?format=json&limit=";

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
fn query_giantbomb<T>(t: &String, url: String) -> GiantBombResult<T>
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
fn query_videos(t: &String, date: chrono::DateTime<chrono::Utc>) -> Vec<GiantBombVideo> {
    let qstring = VID_URL.to_string() + LIMIT;
    let res = query_giantbomb(t, qstring).results;
    res.into_iter()
        .filter(|v: &GiantBombVideo| {
            chrono::Utc.timestamp(from_giantbomb_datetime_to_timestamp(&v.publish_date), 0) > date
        })
        .collect()
}

fn get_config_date() -> Option<chrono::DateTime<chrono::Utc>> {
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("config"))
        .and_then(|m| m.get("datetime"))
        .ok()
}

fn get_config_key() -> String {
    let mut settings = config::Config::default();
    let merged_set = settings
        .merge(config::File::with_name("config"))
        .expect("Error in merging config");
    let k = merged_set.get("gbkey");
    if let Ok(key) = k {
        key
    } else {
        merged_set
            .set("gbkey", "PLEASEINSERTKEY")
            .expect("Could not write config");
        panic!("See settings file to get the key");
    }
}

fn get_config_download() -> PathBuf {
    let mut settings = config::Config::default();
    let merged_set = settings
        .merge(config::File::with_name("config"))
        .expect("Error in merge");
    let p: std::result::Result<String, config::ConfigError> = merged_set.get("path");
    if let Ok(path) = p {
        let mut thepath = PathBuf::new();
        thepath.push(path);
        if !thepath.exists() {
            panic!("Path does not exist: {:?}", thepath);
        }
        thepath
    } else {
        merged_set
            .set("path", "PLEASEINSERTPATHHERE")
            .expect("Could not write config");
        panic!("See settings file to insert path");
    }
}

fn update_config_date() {
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("config"))
        .and_then(|s| s.set("datetime", chrono::Utc::now().to_rfc3339()))
        .expect("Could not write config data");
}

fn download_video(vid: &GiantBombVideo) -> reqwest::Result<()> {
    let mut path = get_config_download();
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
    let key = get_config_key();
    if let Some(time) = get_config_date() {
        let videos = query_videos(&key, time);
        for vid in videos {
            let res = download_video(&vid);
            if res.is_err() {
                println!(
                    "download failed of {}, {}",
                    vid.name,
                    vid.hd_url.unwrap_or("".to_string())
                );
            }
        }
        update_config_date();
    } else {
        update_config_date();
    }
}
