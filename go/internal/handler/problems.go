package handler

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"regexp"
	"sort"
	"strconv"
)

func init() {
	http.HandleFunc("/problems", problemsTemplate)
}

func problemsTemplate(w http.ResponseWriter, r *http.Request) {
	buf := &bytes.Buffer{}
	defer func() { Template(w, buf.Bytes()) }()

	fmt.Fprintf(buf, "<h1>Problems</h1>")

	files, _ := os.ReadDir("/work/problems")

	problemIDs := []int{}
	e := regexp.MustCompile(`^(\d+)\.json$`)
	for _, file := range files {
		if file.IsDir() {
			continue
		}
		m := e.FindStringSubmatch(file.Name())
		if len(m) > 0 && m[1] != "" {
			id, _ := strconv.ParseInt(m[1], 10, 64)
			if id != 0 {
				problemIDs = append(problemIDs, int(id))
			}
		}
	}
	sort.Ints(problemIDs)

	for _, problemID := range problemIDs {
		data, _ := os.ReadFile(fmt.Sprintf("/work/problems/%d.json", problemID))
		info := struct {
			ID   int    `json:"int"`
			Name string `json:"name"`
		}{}
		json.Unmarshal(data, &info)
		fmt.Fprintf(buf, `<h2 id="problem_%d">Problem %d: %s</h2>`,
			problemID, problemID, info.Name)
		fmt.Fprintf(buf, `<img src="/problems/%d.png">`, problemID)
	}
}
