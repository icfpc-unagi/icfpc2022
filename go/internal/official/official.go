package official

import (
	"bytes"
	"encoding/json"
	"github.com/golang/glog"
	"github.com/pkg/errors"
	"net/http"
	"os"
	"sync"
)

var apiKey string
var apiKeyOnce sync.Once

func APIKey() string {
	apiKeyOnce.Do(func() {
		key, err := fetchAPIKey()
		if err != nil {
			glog.Fatalf("Failed to fetch an API key: %+v", err)
		}
		glog.Infof("API key: %s", key)
		apiKey = key
	})
	return apiKey
}

type ScoreboardResult struct {
	ProblemID       int    `json:"problem_id""`
	ProblemName     string `json:"problem_name"`
	LastSubmittedAt string `json:"last_submitted_at"`
	SubmissionCount int    `json:"submission_count"`
	MinCost         int    `json:"min_cost"`
}

type ScoreboardUser struct {
	UserID             int                `json:"user_id"`
	TeamName           string             `json:"team_name"`
	Results            []ScoreboardResult `json:"results"`
	TotalCost          int                `json:"total_cost"`
	SolvedProblemCount int                `json:"solved_problem_count"`
}

type ScoreboardResponse struct {
	IsFrozen      bool             `json:"is_frozen"`
	LastUpdatedAt string           `json:"last_updated_at"`
	Users         []ScoreboardUser `json:"users"`
}

func Scoreboard() (*ScoreboardResponse, error) {
	req, _ := http.NewRequest(
		"GET", "https://robovinci.xyz/api/results/scoreboard", nil)
	req.Header.Set("Authorization", "Bearer "+APIKey())

	client := new(http.Client)
	resp, err := client.Do(req)
	if err != nil {
		return nil, errors.Errorf("failed to fetch the scoreboard: %+v", err)
	}
	defer resp.Body.Close()

	decoder := json.NewDecoder(resp.Body)
	result := ScoreboardResponse{}
	if err := decoder.Decode(&result); err != nil {
		return nil, errors.Errorf("failed to parse a response: %+v", err)
	}
	return &result, nil
}

func fetchAPIKey() (string, error) {
	creds, err := os.ReadFile("/work/secrets/login.json")
	if err != nil {
		return "", errors.Errorf("failed to read credentials: %+v", err)
	}
	glog.Info("Requesting an API key...")
	res, err := http.Post(
		"https://robovinci.xyz/api/users/login", "application/json",
		bytes.NewBuffer(creds))
	defer res.Body.Close()
	if err != nil {
		return "", errors.Errorf("failed to log in: %+v", err)
	}
	decoder := json.NewDecoder(res.Body)
	resp := struct {
		Token string `json:"token"`
	}{}
	if err := decoder.Decode(&resp); err != nil {
		return "", errors.Errorf("failed to parse a response: %+v", err)
	}
	return resp.Token, nil
}
