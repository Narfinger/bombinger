package GB

import (
	"net/url"
	"time"
)

type VideoShow struct {
	ApiDetailUrl  string  `json:"api_detail_url"`  //URL pointing to the video_show detail resource.
	Deck          string  `json:"deck"`            //Brief summary of the video_show.
	Id            int     `json:"id"`              //Unique ID of the video_show.
	Name          string  `json:"name"`            //Name of the video_show.
	SiteDetailUrl url.URL `json:"site_detail_url"` //URL pointing to the video_show on Giant Bomb.
}

type VideoShows []VideoShow

type Video struct {
	ApiDetailUrl  url.URL   `json:"api_detail_url"`  //URL pointing to the video detail resource.
	Deck          string    `json:"deck"`            //Brief summary of the video.
	URLHD         url.URL   `json:"hd_url"`          //URL to the HD version of the video.
	URLHigh       url.URL   `json:"high_url"`        //URL to the High Res version of the video.
	URLLow        url.URL   `json:"low_url"`         //URL to the Low Res version of the video.
	EmbededPlayer url.URL   `json:"embed_player"`    //URL for video embed player. To be inserted into an iFrame.
	Id            int       `json:"id"`              //Unique ID of the video.
	Image         url.URL   `json:"image"`           //???Main image of the video.
	Length        int       `json:"length_seconds"`  //Length (in seconds) of the video.
	Name          string    `json:"name"`            //Name of the video.
	Date          time.Time `json:"publish_date"`    //Date the video was published on Giant Bomb.
	SiteDetailUrl url.URL   `json:"site_detail_url"` //URL pointing to the video on Giant Bomb.
	URL           url.URL   `json:"url"`             //The video's filename.
	Author        string    `json:"user"`            //Author of the video.
	YoutubeID     string    `json:"youtube_id"`      //Youtube ID for the video.
}

type Videos []Video
