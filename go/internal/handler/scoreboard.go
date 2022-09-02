package handler

import (
	"bytes"
	"fmt"
	"github.com/icfpc-unagi/icfpc2022/go/internal/official"
	"html"
	"net/http"
	"sort"
)

func init() {
	http.HandleFunc("/scoreboard", scoreboardTemplate)
}

func scoreboardTemplate(w http.ResponseWriter, r *http.Request) {
	buf := &bytes.Buffer{}
	defer func() { Template(w, buf.Bytes()) }()

	fmt.Fprintf(buf, "<h1>Scoreboard</h1>")

	scoreboard, err := official.Scoreboard()
	if err != nil {
		fmt.Fprintf(buf, "<pre>%s</pre>",
			html.EscapeString(fmt.Sprintf("%+v", err)))
		return
	}

	problems := []int{}
	seenProblems := map[int]struct{}{}
	for _, user := range scoreboard.Users {
		for _, result := range user.Results {
			if _, ok := seenProblems[result.ProblemID]; !ok {
				problems = append(problems, result.ProblemID)
				seenProblems[result.ProblemID] = struct{}{}
			}
		}
	}
	sort.Ints(problems)

	fmt.Fprint(buf, `<div style="overflow-x:scroll;width:100%;">`)
	fmt.Fprint(buf, `<table style="font-size:50%;"><tr><td>Team</td>`)
	for _, problem := range problems {
		fmt.Fprintf(buf, "<td>P%d</td>", problem)
	}
	fmt.Fprintf(buf, "</tr>")

	for _, user := range scoreboard.Users {
		fmt.Fprintf(buf, "<tr><td>%s</td>",
			html.EscapeString(user.TeamName))
		results := map[int]official.ScoreboardResult{}
		for _, result := range user.Results {
			if result.SubmissionCount > 0 {
				results[result.ProblemID] = result
			}
		}
		for _, problem := range problems {
			if result, ok := results[problem]; ok {
				fmt.Fprintf(buf, "<td>%d<br>%s</td>",
					result.MinCost, ToElapsedTime(result.LastSubmittedAt))
			} else {
				fmt.Fprintf(buf, "<td>-</td>")
			}
		}
		fmt.Fprintf(buf, "</tr>")
	}

	fmt.Fprintf(buf, "</table>")
	fmt.Fprintf(buf, "</div>")
}
