package GBVideos

import (
	"encoding/json"
	"net/http"
)

const api_url = "https://www.giantbomb.com/api"
const videos_url = api_url + "/videos/&format=json&api_key=" + api_key
const shows_url = api_url + "/video_shows/?&format=json&api_key=" + api_key

// func GetShows() (VideoShows, error) {
// 	var target VideoShows = nil
// 	r, err := http.Get(shows_url)
// 	if err != nil {
// 		return nil, err
// 	}
// 	defer r.Body.Close()

// 	body, _ := ioutil.ReadAll(r.Body)
// 	fmt.Println(body)
// 	json.NewDecoder(r.Body).Decode(target)
// 	return target, nil
// }

func GetVideos() (Videos, error) {
	var target VideosResponse
	r, err := http.Get(videos_url)
	if err != nil {
		return nil, err
	}
	defer r.Body.Close()
	json.NewDecoder(r.Body).Decode(target)
	return target.Results, nil
}
