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
	http.HandleFunc("/programs", programsHandler)
}

func programsHandler(w http.ResponseWriter, r *http.Request) {
	buf := &bytes.Buffer{}
	defer func() { Template(w, buf.Bytes()) }()

	fmt.Fprintf(buf, "<h1>実行プログラム</h1>")
	fmt.Fprintf(buf, "<h2>プログラム提出</h2>")

	req := &api.InsertProgramRequest{
		ProgramName: r.PostFormValue("program_name"),
		ProgramCode: r.PostFormValue("program_code"),
	}
	if req.ProgramName != "" && req.ProgramCode != "" {
		if resp, err := api.InsertProgram(req); err != nil {
			fmt.Fprintf(buf, `<pre class="alert-danger">%s</pre>`,
				html.EscapeString(fmt.Sprintf("%+v", err)))
		} else {
			fmt.Fprintf(buf, `<div class="alert-success">ジョブを %d 件追加しました</div>`, len(resp.TaskIDs))
		}
	} else if r.Method != "GET" {
		fmt.Fprintf(buf, `<div class="alert-danger">プログラム名、コード両方とも必須です</div>`)
	}
	fmt.Fprintf(buf, `
<form action="?" method="POST">
<div>プログラム名 (必須): <input type="text" name="program_name" style="width:50%%"></div>
<div><textarea name="program_code" style="width:100%%; height: 200px;">%s</textarea><br>
※ 問題は <code>/work/problems/${PROBLEM_ID}.png</code> から取得できます。出力は標準出力へ。</div>
<input type="submit" value="実行開始" class="primary">
</form>
`, html.EscapeString(req.ProgramCode))

	fmt.Fprintf(buf, `<h2>プログラム一覧</h2>`)

	programs := make([]struct {
		ProgramID   int    `db:"program_id"`
		ProgramName string `db:"program_name"`
		ProgramCode string `db:"program_code"`
	}, 0)
	if err := db.Select(context.Background(), &programs,
		`SELECT program_id, program_name, program_code FROM programs ORDER BY program_id DESC`); err != nil {
		glog.Errorf("Failed to fetch programs: %+v", err)
	}

	fmt.Fprintf(buf, "<ul>")
	for _, p := range programs {
		fmt.Fprintf(buf, `<li>プログラム番号 %d: %s<pre>%s</pre></li>`, p.ProgramID, html.EscapeString(p.ProgramName), html.EscapeString(p.ProgramCode))
	}
	fmt.Fprintf(buf, "</ul>")
}
