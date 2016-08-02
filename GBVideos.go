package GBVideos

import (
	"encoding/json"
	"net/http"
)

const api_url = "https://www.giantbomb.com/api"
const videos_url, _ = Parse(api_url + "/videos")
const shows_url, _ = Parse(api_url + "/video_shows")

func getJson(url string, target interface{}) error {

}

func GetShows() VideoShows {
	target = nil
	r, err := http.Get(url)
	if err != nil {
		//return err
	}
	defer r.Body.Close()

	return json.NewDecoder(r.Body).Decode(target)
}
