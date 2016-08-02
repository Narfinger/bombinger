package GB

import (
	"encoding/json"
	"net/http"
)

const api_url = "https://www.giantbomb.com/api"
const videos_url = api_url + "/videos"
const shows_url = api_url + "/video_shows"

func GetShows() VideoShows {
	var target VideoShows = nil
	r, err := http.Get(shows_url)
	if err != nil {
		//return err
	}
	defer r.Body.Close()

	json.NewDecoder(r.Body).Decode(target)
	return target
}
