package handler

import (
	"bytes"
	"fmt"
	"html"
	"net/http"
	"os"
	"regexp"
	"strconv"
	"strings"

	"github.com/icfpc-unagi/icfpc2022/go/internal/api"

	"github.com/icfpc-unagi/icfpc2022/go/internal/util"

	"github.com/icfpc-unagi/icfpc2022/go/internal/auth"
)

func init() {
	webDir := http.StripPrefix("/visualizer/", http.FileServer(http.Dir("/work/web/")))
	http.HandleFunc("/visualizer/", auth.BasicAuth(func(w http.ResponseWriter, r *http.Request) {
		if r.Method == "GET" && strings.HasPrefix(r.URL.Path, "/visualizer/") {
			if r.URL.Path == "/visualizer/" {
				visualizerHandler(w, r)
				return
			}
			webDir.ServeHTTP(w, r)
		} else {
			http.NotFound(w, r)
		}
	}))
}

func visualizerHandler(w http.ResponseWriter, r *http.Request) {
	buf := &bytes.Buffer{}
	defer func() { Template(w, buf.Bytes()) }()

	params := struct {
		ProblemID int
		Solution  string
	}{}

	fmt.Fprintf(buf, "<h1>ビジュアライザ</h1>")

	if id, _ := strconv.Atoi(r.URL.Query().Get("run_id")); id != 0 {
		resp, err := api.ExportRun(id)
		if err != nil {
			fmt.Fprintf(buf, `<pre class="alert-danger">%s</pre>`,
				html.EscapeString(fmt.Sprintf("%+v", err)))
		} else {
			params.ProblemID = resp.ProblemID
			params.Solution = resp.ISL
		}
	}

	page := func() string {
		p, _ := os.ReadFile("/work/web/index.html")
		s := string(p)
		ss := strings.SplitN(s, "<body>", 2)
		s = ss[len(ss)-1]
		s = strings.SplitN(s, "</body>", 2)[0]
		return s
	}()

	page = func() string {
		buf := &bytes.Buffer{}
		fmt.Fprintf(buf, `<div><select name="problem_id" id="problem_id" onchange="updateOutput()" style="margin-left: 0">`)
		for _, p := range util.Problems() {
			selected := ""
			if params.ProblemID == p.ID {
				selected = " selected"
			}
			fmt.Fprintf(buf, `<option value="%d"%s>問題 %d: %s</option>`,
				p.ID, selected, p.ID, p.Name)
		}
		fmt.Fprintf(buf, `</select>→<a href="https://icfpc.sx9.jp/static/dist/">→エディタ</a></div>`)
		r := regexp.MustCompile(`<label for="problem_id">[^\a]*?</label>`)
		return r.ReplaceAllString(page, buf.String())
	}()

	page = strings.ReplaceAll(page, "</textarea>", html.EscapeString(params.Solution)+"</textarea>")

	fmt.Fprint(buf, page)
	fmt.Fprint(buf, "<script>updateOutput()</script>")
}
