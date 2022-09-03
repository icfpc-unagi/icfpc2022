package handler

import (
	"bytes"
	"fmt"
	"net/http"
)

func init() {
	http.HandleFunc("/problems", problemsTemplate)
}

func problemsTemplate(w http.ResponseWriter, r *http.Request) {
	buf := &bytes.Buffer{}
	defer func() { Template(w, buf.Bytes()) }()

	fmt.Fprintf(buf, "<h1>Problems</h1>")

	for _, problem := range Problems() {
		fmt.Fprintf(buf, `<h2 id="problem_%d">Problem %d: %s</h2>`,
			problem.ID, problem.ID, problem.Name)
		fmt.Fprintf(buf, `<img src="/problems/%d.png">`, problem.ID)
	}
}
