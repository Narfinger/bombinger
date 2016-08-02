package GBTypes

type VideoShow struct {
	ApiDetailUrl  string  `json:"api_detail_url"`  //URL pointing to the video_show detail resource.
	Deck          string  `json:"deck"`            //Brief summary of the video_show.
	Id            int     `json:"id"`              //Unique ID of the video_show.
	Name          string  `json:"name"`            //Name of the video_show.
	SiteDetailUrl net.URL `json:"site_detail_url"` //URL pointing to the video_show on Giant Bomb.
}

type VideoShows []VideoShow

type Video struct {
	ApiDetailUrl  net.URL `json:"api_detail_url"`  //URL pointing to the video detail resource.
	Deck          string  `json:"deck"`            //Brief summary of the video.
	URLHD         net.URL `json:"hd_url"`          //URL to the HD version of the video.
	URLHigh       net.URL `json:"high_url"`        //URL to the High Res version of the video.
	URLLow        net.URL `json:"low_url"`         //URL to the Low Res version of the video.
	EmbededPlayer net.URL `json:"embed_player"`    //URL for video embed player. To be inserted into an iFrame.
	Id            int     `json:"id"`              //Unique ID of the video.
	Image         net.URL `json:"image"`           //???Main image of the video.
	Length        int     `json:"length_seconds"`  //Length (in seconds) of the video.
	Name          string  `json:"name"`            //Name of the video.
	Date          time    `json:"publish_date"`    //Date the video was published on Giant Bomb.
	SiteDetailUrl net.URL `json:"site_detail_url"` //URL pointing to the video on Giant Bomb.
	URL           net.URL `json:"url"`             //The video's filename.
	Author        string  `json:"user"`            //Author of the video.
	YoutubeID     string  `json:"youtube_id"`      //Youtube ID for the video.
}

type Videos []Video
