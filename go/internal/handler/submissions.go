package handler

import (
	"bytes"
	"context"
	"fmt"
	"net/http"

	"github.com/icfpc-unagi/icfpc2022/go/pkg/db"

	"github.com/icfpc-unagi/icfpc2022/go/internal/auth"
)

func init() {
	http.HandleFunc("/submissions", auth.BasicAuth(submissionsHandler))
}

func submissionsHandler(w http.ResponseWriter, r *http.Request) {
	buf := &bytes.Buffer{}
	defer func() { Template(w, buf.Bytes()) }()

	fmt.Fprintf(buf, "<h1>公式提出物一覧</h1>")

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
GROUP BY
    problem_id
) AS s
ORDER BY problem_id
`); err != nil {
		fmt.Fprintf(w, "<pre>エラー: %+v</pre>", err)
		return
	}

	fmt.Fprintf(buf, `<table><tr><td>問題番号</td><td>提出番号</td><td>スコア</td></tr>`)
	for _, s := range submissions {
		fmt.Fprintf(buf, `<tr><td>問題%d</td><td>提出%d</td><td>%d</td></tr>`, s.ProblemID, s.SubmissionID, s.SubmissionScore)
	}
	fmt.Fprintf(buf, `</table>`)

	for _, problem := range Problems() {
		fmt.Fprintf(buf, `<h2 id="problem_%d">Problem %d: %s</h2>`,
			problem.ID, problem.ID, problem.Name)
		fmt.Fprintf(buf, `<img src="/problems/%d.png">`, problem.ID)
	}
}
