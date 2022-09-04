package api

import (
	"context"
	"fmt"
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
	fmt.Fprintf(w, "%d submissions were fetched.\n", count)

	if result, err := db.Execute(context.Background(), `
INSERT IGNORE
INTO runs(submission_id, program_id, problem_id, run_command, run_name, run_score, run_created)
SELECT
    submission_id,
    0 AS program_id,
    problem_id,
    "" AS run_command,
    CASE WHEN submission_solution LIKE "#%" THEN TRIM(
        SUBSTRING(
            SUBSTRING_INDEX(submission_solution, "\n", 1)
        FROM
            2
        )
    ) ELSE ""
END AS run_name,
submission_score AS run_score,
submission_created AS run_created
FROM
    (
        submissions
    NATURAL JOIN(
        SELECT
            submission_id
        FROM
            (
            SELECT
                submissions.submission_id AS submission_id,
                (t.submission_id IS NULL) AS flag
            FROM
                (
                    submissions NATURAL LEFT
                JOIN(
                    SELECT
                        submission_id
                    FROM
                        runs
                    WHERE
                        submission_id IS NOT NULL
                ) t
                )
        ) t
    WHERE
        flag
    ) t
    )
ORDER BY
    submission_id
DESC
`); err != nil {
		glog.Errorf("Failed to update runs from submissions: %+v", err)
	} else if n, err := result.RowsAffected(); err != nil {
		glog.Errorf("Failed to get # of affected rows: %+v", err)
	} else {
		fmt.Fprintf(w, "%d submissions have been merged into runs.\n", n)
	}
}
