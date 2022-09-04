package api

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"

	"github.com/icfpc-unagi/icfpc2022/go/pkg/db"
)

func init() {
	http.HandleFunc("/"+PATH_PREFIX+"/submission", submissionHandler)
}

func submissionHandler(w http.ResponseWriter, r *http.Request) {
	// submissionID, _ := strconv.ParseInt(r.FormValue("submission_id"), 10, 64)
	problemID, _ := strconv.ParseInt(r.FormValue("problem_id"), 10, 64)
	submissions := make([]struct {
		ProblemID          int    `db:"problem_id"`
		SubmissionID       int    `db:"submission_id""`
		SubmissionScore    int64  `db:"submission_score""`
		SubmissionSolution string `db:"submission_solution"`
	}, 0)
	if err := db.Select(context.Background(), &submissions, `
SELECT
	problem_id,
	submission_id,
	submission_score,
	submission_solution
FROM
    submissions
NATURAL JOIN(
    SELECT
        MIN(submission_id) AS submission_id
    FROM
        submissions
    NATURAL JOIN(
        SELECT
            problem_id,
            MIN(submission_score) AS submission_score
        FROM
            submissions
        GROUP BY
            problem_id
    ) AS s
WHERE problem_id = ?
GROUP BY
    problem_id
) AS s
ORDER BY problem_id
`, problemID); err != nil {
		fmt.Fprintf(w, "<pre>エラー: %+v</pre>", err)
		return
	}
	res, err := json.Marshal(submissions[0])
	if err != nil {
		fmt.Fprintf(w, "<pre>エラー: %+v</pre>", err)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Access-Control-Allow-Origin", "*")
	w.Write(res)
}
