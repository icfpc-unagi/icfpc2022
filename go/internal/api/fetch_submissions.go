package api

import (
	"context"
	"io"
	"net/http"

	"github.com/golang/glog"
	"github.com/icfpc-unagi/icfpc2022/go/internal/official"

	"github.com/icfpc-unagi/icfpc2022/go/pkg/db"
)

func init() {
	http.HandleFunc("/"+PATH_PREFIX+"/fetch_submissions", fetchSubmissionsHandler)
}

func fetchSubmissionsHandler(w http.ResponseWriter, r *http.Request) {
	dbData := []struct {
		SubmissionID    int   `db:"submission_id"`
		SubmissionScore int64 `db:"submission_score"`
	}{}

	if err := db.Select(context.Background(), &dbData,
		`SELECT submission_id, submission_score FROM submissions ORDER BY submission_id`,
	); err != nil {
		w.WriteHeader(500)
		glog.Errorf("Failed to fetch submissions from DB: %+v", err)
		return
	}

	scores := map[int]int64{}
	for _, row := range dbData {
		scores[row.SubmissionID] = row.SubmissionScore
	}

	officialData, err := official.Submissions()
	if err != nil {
		w.WriteHeader(503)
		glog.Errorf("Official data is temporary unavailable: %+v", err)
		return
	}

	count := 0
	for _, s := range officialData.Submissions {
		if scores[s.ID] != s.Score {
			count++
			if count > 10 {
				glog.Errorf("Too many submissions are fetched.")
				return
			}
			if fat, err := official.Submission(s.ID); err != nil {
				glog.Errorf("Failed to fetch submission %d: %+v", s.ID, err)
				continue
			} else if resp, err := http.Get(fat.FileURL); err != nil {
				glog.Errorf("Failed to fetch ISL data: %+v", err)
				continue
			} else if data, err := io.ReadAll(resp.Body); err != nil {
				glog.Errorf("Failed to fetch ISL data: %+v", err)
				continue
			} else if _, err := db.Execute(context.Background(),
				`REPLACE INTO submissions(submission_id, problem_id, submission_score, submission_solution) VALUES(?, ?, ?, ?)`,
				fat.ID, fat.ProblemID, fat.Score, string(data)); err != nil {
				glog.Errorf("Failed to insert a submission to SQL: %+v", err)
				continue
			}
		}
	}
}
