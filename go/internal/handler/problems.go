package handler

import (
	"bytes"
	"context"
	"fmt"
	"html"
	"net/http"
	"sort"

	"github.com/icfpc-unagi/icfpc2022/go/internal/official"

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

type rankingRecord struct {
	TeamName string
	MinCost  int
	Rank     int
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

	scoreboard, err := official.Scoreboard()
	if err != nil {
		glog.Errorf("Failed to fetch scoreboard: %+v", err)
	}

	ranks := map[int][]*rankingRecord{}
	if scoreboard != nil {
		for _, u := range scoreboard.Users {
			for _, r := range u.Results {
				if r.SubmissionCount == 0 {
					continue
				}
				ranks[r.ProblemID] = append(ranks[r.ProblemID], &rankingRecord{
					TeamName: u.TeamName,
					MinCost:  r.MinCost,
				})
			}
		}
	}
	for _, r := range ranks {
		sort.SliceStable(r, func(i, j int) bool {
			return r[i].MinCost < r[j].MinCost
		})
		for i, _ := range r {
			r[i].Rank = i + 1
			if i > 0 && r[i].MinCost == r[i-1].MinCost {
				r[i].Rank = r[i-1].Rank
			}
		}
	}

	submissionMap := map[int]*submissionsRecord{}
	for _, s := range submissions {
		submissionMap[s.ProblemID] = s
	}

	for _, problem := range Problems() {
		showProblem(buf, submissionMap[problem.ID], &problem, ranks[problem.ID])
	}
}

func showProblem(buf *bytes.Buffer, record *submissionsRecord, problem *Problem, ranking []*rankingRecord) {
	fmt.Fprintf(buf, `<h2><a name="problem_%d"></a>Problem %d: %s</h2>`,
		problem.ID, problem.ID, problem.Name)

	resp := &api.EvaluateResponse{}
	if record != nil && record.SubmissionSolution != "" {
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
	if resp.Image == "" {
		resp.Image = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAAAAAA6fptVAAAACklEQVQIHWP4DwABAQEANl9ngAAAAABJRU5ErkJggg=="
	}

	fmt.Fprintf(buf, `
<table style="table-layout:fixed;width:100%%;margin-bottom: 2em;"><tr>
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
		problem.ID, resp.Image, resp.Image, problem.ID)
	fmt.Fprint(buf, `<table style="table-layout:fixed; width:100%;"><tr><td width="50%" style="vertical-align:top">`)
	if ranking != nil {
		fmt.Fprint(buf, `<table style="width: 100%; table-layout: fixed">`)
		for i, r := range ranking {
			style := ""
			if i >= 10 && r.TeamName != "Unagi" {
				continue
			}
			if r.TeamName == "Unagi" {
				style = `background: #cdf; color: red; font-weight: bold`
			}
			rankStr := ""
			if r.Rank == 1 {
				rankStr = "👑 "
			} else if r.Rank == 2 {
				rankStr = "🥈 "
			} else if r.Rank == 3 {
				rankStr = "🥉 "
			}
			diff := ""
			if r.Rank != 1 {
				diff = fmt.Sprintf("%+.1f%%", (float64(r.MinCost)/float64(ranking[0].MinCost)-1)*100)
			}
			fmt.Fprintf(buf, `<tr style="white-space: nowrap; %s"><td style="width:4ex;">%d位</td><td style="overflow-x:hidden; text-overflow: ellipsis; width: 50%%">%s%s</td><td style="text-align:right; width: 6ex;">%d</td><td style="text-align:right; width: 6ex;">%s</td>`,
				style,
				r.Rank, rankStr, html.EscapeString(r.TeamName), r.MinCost, diff)
		}
		fmt.Fprintf(buf, `</table>`)
	}
	fmt.Fprint(buf, `</td><td width="50%" style="vertical-align:top">`)
	if record != nil && record.SubmissionSolution != "" {
		fmt.Fprintf(buf, `<ul><li>提出ID: %d</li>`, record.SubmissionID)
		fmt.Fprintf(buf, `<li>スコア: %d (コスト: %d, 類似度: %d)</li>`,
			resp.Cost+resp.Similarity, resp.Cost, resp.Similarity)
		fmt.Fprintf(buf, "</ul>")
		fmt.Fprintf(buf, `<form action="/visualizer/" method="GET" style="text-align: center;"><input type="hidden" name="submission_id" value="%d"><input type="submit" value="可視化"></form>`, record.SubmissionID)
	}
	fmt.Fprint(buf, `</td></tr></table>`)
}
