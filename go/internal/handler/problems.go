package handler

import (
	"bytes"
	"context"
	"fmt"
	"html"
	"net/http"

	"github.com/icfpc-unagi/icfpc2022/go/internal/api"

	"github.com/golang/glog"

	"github.com/icfpc-unagi/icfpc2022/go/pkg/db"
)

func init() {
	http.HandleFunc("/problems", problemsTemplate)
}

type submissionsRecord struct {
	ProblemID          int    `db:"problem_id"`
	SubmissionID       int    `db:"submission_id""`
	SubmissionScore    int64  `db:"submission_score""`
	SubmissionSolution string `db:"submission_solution"`
}

func problemsTemplate(w http.ResponseWriter, r *http.Request) {
	buf := &bytes.Buffer{}
	defer func() { Template(w, buf.Bytes()) }()

	fmt.Fprintf(buf, "<h1>Problems</h1>")

	submissions := make([]*submissionsRecord, 0)
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
		glog.Errorf("Failed to fetch submissions from DB: %+v", err)
	}

	submissionMap := map[int]*submissionsRecord{}
	for _, s := range submissions {
		submissionMap[s.ProblemID] = s
	}

	for _, problem := range Problems() {
		showProblem(buf, submissionMap[problem.ID], &problem)
	}
}

func showProblem(buf *bytes.Buffer, record *submissionsRecord, problem *Problem) {
	fmt.Fprintf(buf, `<h2><a name="problem_%d"></a>Problem %d: %s</h2>`,
		problem.ID, problem.ID, problem.Name)

	resp := &api.EvaluateResponse{}
	if record.SubmissionSolution != "" {
		var err error
		resp, err = api.Evaluate(&api.EvaluateRequest{
			ProblemID: problem.ID,
			ISL:       record.SubmissionSolution,
		})
		if err != nil {
			resp = &api.EvaluateResponse{}
			resp.ProblemID = problem.ID
			fmt.Fprintf(buf, "<pre>エラー: %s</pre>",
				html.EscapeString(fmt.Sprintf("%+v", err)))
		}
	}

	fmt.Fprintf(buf, `
<table style="table-layout:fixed;width:100%%"><tr>
<td width="30%%" style="text-align:center">
	<img src="/problems/%d.png" style="width:100%%;"><br>
	元画像
</td>
<td width="30%%" style="text-align:center">
	<img src="data:image/png;base64,%s" style="width:100%%;"><br>
	提出画像
</td>
<td width="30%%" style="text-align:center; isolation: isolate; position: relative">
	<div style="width:100%%; position:relative"><img src="data:image/png;base64,%s" style="width:100%%"><img src="/problems/%d.png" style="width:100%%; position:absolute; top: 0; left: 0; mix-blend-mode: difference;"></div>
	差分画像
<br>
</td>
</tr></table>`,
		resp.ProblemID, resp.Image, resp.Image, resp.ProblemID)
	if record.SubmissionSolution != "" {
		fmt.Fprintf(buf, `<ul><li>提出ID: %d</li>`, record.SubmissionID)
		fmt.Fprintf(buf, `<li>スコア: %d (コスト: %d, 類似度: %d)</li>`,
			resp.Cost+resp.Similarity, resp.Cost, resp.Similarity)
		fmt.Fprintf(buf, "</ul>")
	}
}
