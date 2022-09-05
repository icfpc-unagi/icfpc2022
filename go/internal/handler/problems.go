package handler

import (
	"bytes"
	"fmt"
	"html"
	"net/http"
	"sort"
	"strings"

	"github.com/icfpc-unagi/icfpc2022/go/internal/api"
)

func init() {
	http.HandleFunc("/problems", problemsTemplate)
}

func problemsTemplate(w http.ResponseWriter, r *http.Request) {
	buf := &bytes.Buffer{}
	defer func() { Template(w, buf.Bytes()) }()

	fmt.Fprintf(buf, "<h1>Problems</h1>")

	records := getAllRecords(buf)
	problemToRecords := map[int][]*scoreboardRecord{}
	for _, r := range records {
		problemToRecords[r.ProblemID] = append(problemToRecords[r.ProblemID], r)
	}

	for _, problem := range Problems() {
		showProblem(buf, &problem, problemToRecords[problem.ID])
	}
}

func showProblem(buf *bytes.Buffer, problem *Problem, records []*scoreboardRecord) {
	fmt.Fprintf(buf, `<h2><a name="problem_%d"></a>Problem %d: %s</h2>`,
		problem.ID, problem.ID, problem.Name)

	sort.SliceStable(records, func(i, j int) bool {
		return records[i].Score < records[j].Score
	})

	var bestRecord *scoreboardRecord
	for _, r := range records {
		if r.RunID != 0 {
			bestRecord = r
			break
		}
	}

	var exportResp *api.ExportResponse

	if bestRecord.RunID != 0 {
		var err error
		exportResp, err = api.ExportRun(bestRecord.RunID)
		if err != nil {
			fmt.Fprintf(buf, `<pre class="alert-danger">%s</pre>`,
				html.EscapeString(fmt.Sprintf("%+v", err)))
		}
	}

	resp := &api.EvaluateResponse{}
	if exportResp != nil && exportResp.ISL != "" {
		var err error
		resp, err = api.Evaluate(&api.EvaluateRequest{
			ProblemID: problem.ID,
			ISL:       exportResp.ISL,
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
	externalCount := 0
	internalCount := 0
	if len(records) > 0 {
		fmt.Fprint(buf, `<table style="width: 100%; table-layout: fixed">`)
		for _, r := range records {
			style := ""
			if r.UserName == "Unagi" {
				// pass
			} else if r.IsInternal && internalCount > 5 {
				continue
			} else if !r.IsInternal && externalCount > 10 {
				continue
			}
			if r.IsInternal {
				internalCount++
			} else {
				externalCount++
			}
			if r.UserName == "Unagi" {
				style = `background: #cdf; color: red; font-weight: bold`
			}
			rankStr := ""
			if r.ProblemRank == 1 {
				rankStr = "👑 "
			} else if r.ProblemRank == 2 {
				rankStr = "🥈 "
			} else if r.ProblemRank == 3 {
				rankStr = "🥉 "
			}
			diff := ""
			if r.ProblemRank != 1 {
				diff = fmt.Sprintf("%+.1f%%", (float64(r.Score)/float64(records[0].Score)-1)*100)
			}
			field := rankStr + html.EscapeString(r.UserName)
			if r.RunID != 0 {
				field = fmt.Sprintf(`<a href="/visualizer?run_id=%d" target="_blank" style="%s">%s</a>`, r.RunID, style, field)
			}
			fmt.Fprintf(buf, `<tr style="white-space: nowrap; %s"><td style="width:4ex;">%d位</td><td style="overflow-x:hidden; text-overflow: ellipsis; width: 50%%">%s</td><td style="text-align:right; width: 6ex;">%d</td><td style="text-align:right; width: 6ex;">%s</td>`,
				style,
				r.ProblemRank, field, r.Score, diff)
		}
		fmt.Fprintf(buf, `</table>`)
	}
	fmt.Fprint(buf, `</td><td width="50%" style="vertical-align:top">`)
	if exportResp != nil && exportResp.ISL != "" {
		comment := strings.SplitN(exportResp.ISL, "\n", 2)[0]
		fmt.Fprintf(buf, `<ul>`)
		if strings.HasPrefix(comment, "#") {
			comment = strings.TrimSpace(strings.TrimPrefix(comment, "#"))
			fmt.Fprintf(buf, `<li>提出情報: %s</li>`, html.EscapeString(comment))
		}
		fmt.Fprintf(buf, `<li>提出ID: %d</li>`, exportResp.RunID)
		fmt.Fprintf(buf, `<li>スコア: %d (コスト: %d, 類似度: %d)</li>`,
			resp.Cost+resp.Similarity, resp.Cost, resp.Similarity)
		fmt.Fprintf(buf, "</ul>")
		fmt.Fprintf(buf, `<form action="/visualizer/" method="GET" style="text-align: center;"><input type="hidden" name="run_id" value="%d"><input type="submit" value="可視化"></form>`, exportResp.RunID)
	}
	fmt.Fprint(buf, `</td></tr></table>`)
}
