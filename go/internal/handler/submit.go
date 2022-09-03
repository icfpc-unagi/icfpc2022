package handler

import (
	"bytes"
	"fmt"
	"html"
	"net/http"
	"strconv"

	"github.com/icfpc-unagi/icfpc2022/go/internal/auth"

	"github.com/icfpc-unagi/icfpc2022/go/internal/api"
)

func init() {
	http.HandleFunc("/submit", auth.BasicAuth(submitHandler))
}

func submitHandler(w http.ResponseWriter, r *http.Request) {
	buf := &bytes.Buffer{}
	defer func() { Template(w, buf.Bytes()) }()

	problems := Problems()

	fmt.Fprintf(buf, "<h1>評価・提出フォーム</h1>")

	resp := &api.EvaluateResponse{}

	if problemID, _ := strconv.ParseInt(
		r.FormValue("problem_id"), 10, 64); problemID > 0 {
		var err error
		resp, err = api.Evaluate(&api.EvaluateRequest{
			ProblemID: int(problemID),
			ISL:       r.FormValue("isl"),
		})
		if err != nil {
			resp.ProblemID = int(problemID)
			fmt.Fprintf(buf, "<pre>エラー: %s</pre>",
				html.EscapeString(fmt.Sprintf("%+v", err)))
		}
	}

	if resp.Image != "" {
		fmt.Fprintf(buf, `
<table style="table-layout:fixed;width:100%%"><tr>
<td width="30%%" style="text-align:center">
	<img src="data:image/png;base64,%s" style="width:100%%;"><br>
	提出画像
</td>
<td width="30%%" style="text-align:center">
	<img src="/problems/%d.png" style="width:100%%;"><br>
	元画像
</td>
<td width="30%%" style="text-align:center">
</td>
</tr></table>`,
			resp.Image, resp.ProblemID)
		fmt.Fprintf(buf, `<div>スコア: %d (コスト: %d, 類似度: %d)</div>`, resp.Cost+resp.Similarity, resp.Cost, resp.Similarity)
	}

	fmt.Fprintf(buf, `<form action="?" method="POST">`)
	fmt.Fprintf(buf, `<div>問題番号: <select name="problem_id">`)
	for _, p := range problems {
		selected := ""
		if r.FormValue("problem_id") == strconv.Itoa(p.ID) {
			selected = " selected"
		}
		fmt.Fprintf(buf, `<option value="%d"%s>Problem %d: %s</option>`,
			p.ID, selected, p.ID, p.Name)
	}
	fmt.Fprintf(buf, `</select></div>`)
	fmt.Fprint(buf, `<div><textarea name="isl" style="width:100%;height:500px;">`)
	fmt.Fprintf(buf, `%s</textarea></div>`, html.EscapeString(r.FormValue("isl")))
	fmt.Fprintf(buf, `<input type="submit" value="評価">`)
	fmt.Fprintf(buf, `</form>`)
}
