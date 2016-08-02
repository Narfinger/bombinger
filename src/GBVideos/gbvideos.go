package GBVideos

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"net/http"
)

const api_url = "https://www.giantbomb.com/api"
const videos_url = api_url + "/videos"
const shows_url = api_url + "/video_shows?&format=json&api_key=" + api_key

func GetShows() (VideoShows, error) {
	var target VideoShows = nil
	r, err := http.Get(shows_url)
	if err != nil {
		return nil, err
	}
	defer r.Body.Close()

	body, _ := ioutil.ReadAll(r.Body)
	fmt.Println(body)
	json.NewDecoder(r.Body).Decode(target)
	return target, nil
}
