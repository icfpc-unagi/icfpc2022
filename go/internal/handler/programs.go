package handler

import (
	"bytes"
	"context"
	"fmt"
	"html"
	"net/http"
	"strconv"
	"time"

	"github.com/icfpc-unagi/icfpc2022/go/internal/api"

	"github.com/golang/glog"

	"github.com/icfpc-unagi/icfpc2022/go/pkg/db"
)

func init() {
	http.HandleFunc("/programs", programsHandler)
}

func programsHandler(w http.ResponseWriter, r *http.Request) {
	buf := &bytes.Buffer{}
	defer func() { Template(w, buf.Bytes()) }()

	fmt.Fprintf(buf, "<h1>実行プログラム</h1>")
	fmt.Fprintf(buf, "<h2>プログラム提出</h2>")

	pipelineID, _ := strconv.Atoi(r.PostFormValue("program_pipeline"))
	req := &api.InsertProgramRequest{
		ProgramName:     r.PostFormValue("program_name"),
		ProgramCode:     r.PostFormValue("program_code"),
		ProgramPipeline: pipelineID,
	}
	if req.ProgramName != "" && req.ProgramCode != "" {
		if resp, err := api.InsertProgram(req); err != nil {
			fmt.Fprintf(buf, `<pre class="alert-danger">%s</pre>`,
				html.EscapeString(fmt.Sprintf("%+v", err)))
		} else {
			fmt.Fprintf(buf, `<div class="alert-success">ジョブを %d 件追加しました</div>`, len(resp.RunIDs))
		}
	} else if r.Method != "GET" {
		fmt.Fprintf(buf, `<div class="alert-danger">プログラム名、コード両方とも必須です</div>`)
	}
	fmt.Fprintf(buf, `
<form action="?" method="POST">
<div>プログラム名 (必須): <input type="text" name="program_name" style="width:50%%; margin-right: 4em;">
パイプライン番号: <input type="text" name="program_pipeline" value="%d">
</div>
<div><textarea name="program_code" style="width:100%%; height: 200px;" class="code">%s</textarea><br>
※ <code>{{PROBLEM_ID}}</code> が問題番号と置換されます。<br>
※ 問題は <code>/work/problems/{{PROBLEM_ID}}.png</code> から取得できます。出力は標準出力へ。</div>
<input type="submit" value="実行開始" class="primary">
</form>
`, pipelineID, html.EscapeString(req.ProgramCode))

	fmt.Fprintf(buf, `<h2>プログラム一覧</h2>`)

	programs := make([]struct {
		ProgramID       int    `db:"program_id"`
		ProgramName     string `db:"program_name"`
		ProgramPipeline int    `db:"program_pipeline"`
		ProgramCode     string `db:"program_code"`
	}, 0)
	if err := db.Select(context.Background(), &programs,
		`SELECT program_id, program_name, program_pipeline, program_code FROM programs ORDER BY program_id DESC`); err != nil {
		glog.Errorf("Failed to fetch programs: %+v", err)
	}

	fmt.Fprintf(buf, "<ul>")
	for _, p := range programs {
		fmt.Fprintf(buf, `<li>プログラム番号 %d: %s (パイプライン=%d)<pre>%s</pre></li>`,
			p.ProgramID, html.EscapeString(p.ProgramName), p.ProgramPipeline, html.EscapeString(p.ProgramCode))
	}
	fmt.Fprintf(buf, "</ul>")

	fmt.Fprintf(buf, `<h2>最新ジョブ一覧</h2>`)
	listRuns(buf, "WHERE program_id > 0 ORDER BY run_id DESC LIMIT 1000")
}

func listRuns(buf *bytes.Buffer, where string) {
	records := make([]struct {
		RunID       int     `db:"run_id"`
		ProgramID   int     `db:"program_id"`
		ProgramName string  `db:"program_name"`
		ProblemID   int     `db:"problem_id"`
		RunPipeline int     `db:"run_pipeline"`
		RunScore    *int64  `db:"run_score"`
		RunLocked   *string `db:"run_locked"`
		RunCreated  string  `db:"run_created"`
	}, 0)
	if err := db.Select(context.Background(), &records, `
SELECT
    run_id,
    program_id,
    program_name,
    problem_id,
	run_pipeline,
    run_score,
    run_locked,
    run_created
FROM
    runs
NATURAL JOIN programs
`+where); err != nil {
		fmt.Fprintf(buf, `<pre class="alert-danger">%s</pre>`,
			html.EscapeString(fmt.Sprintf("%+v", err)))
	}

	fmt.Fprintf(buf, `<table style="width:100%%; table-layout: fixed; "">`)
	now := time.Now().UTC().Format("2006-01-02 15:04:05")
	for _, r := range records {
		scoreInfo := ""
		if r.RunScore != nil {
			scoreInfo = fmt.Sprintf(
				`<a href="/visualizer?run_id=%d" target="_blank">%d</a>`,
				r.RunID, *r.RunScore)
		} else if r.RunLocked != nil {
			if *r.RunLocked < now {
				scoreInfo = "実行待ち"
			} else {
				scoreInfo = "実行中"
			}
		} else {
			scoreInfo = "-"
		}
		fmt.Fprintf(buf, `<tr><td style="width: 17ex">ジョブ %d (P%d)</td><td style="width: 8ex">問題%d</td><td>%s</td><td style="width: 10ex; text-align: right">%s</td><td style="width:22ex; text-align: right">%s</td></tr>`,
			r.RunID,
			r.RunPipeline,
			r.ProblemID,
			html.EscapeString(r.ProgramName),
			scoreInfo,
			r.RunCreated,
		)
	}
	fmt.Fprintf(buf, "</table>")
}
