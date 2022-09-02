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
	problemCosts := map[int][]int{}
	for _, user := range scoreboard.Users {
		for _, result := range user.Results {
			if _, ok := problemCosts[result.ProblemID]; !ok {
				problems = append(problems, result.ProblemID)
				problemCosts[result.ProblemID] = nil
			}
			if result.SubmissionCount > 0 {
				problemCosts[result.ProblemID] = append(
					problemCosts[result.ProblemID], result.MinCost)
			}
		}
	}
	sort.Ints(problems)
	problemRanks := map[int]map[int]int{}
	for _, problem := range problems {
		problemRanks[problem] = map[int]int{}
		sort.Ints(problemCosts[problem])
		for index, cost := range problemCosts[problem] {
			if _, ok := problemRanks[problem][cost]; !ok {
				problemRanks[problem][cost] = index + 1
			}
		}
	}

	fmt.Fprint(buf,
		`凡例: <span style="color:red;font-weight:bold">1位</span>`+
			` <span style="color:#880;font-weight:bold">5位以内</span>`+
			` <span style="font-weight:bold">10位以内</span>`+
			` <span>その他</span><br><br>`)
	fmt.Fprint(buf, `<div style="overflow-x:scroll;width:100%;">`)
	fmt.Fprint(buf, `<table style="font-size:50%;"><tr><th>Team</th>`)
	for _, problem := range problems {
		fmt.Fprintf(buf, "<th>問%d</th>", problem)
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
				rank := problemRanks[problem][result.MinCost]
				costStr := "&gt;1e6"
				if result.MinCost < 1000000 {
					costStr = fmt.Sprintf("%d", result.MinCost)
				}
				style := ""
				if rank == 1 {
					style = "color:red; font-weight: bold;"
				} else if rank < 5 {
					style = "color: #880; font-weight: bold;"
				} else if rank < 10 {
					style = "font-weight: bold;"
				}
				fmt.Fprintf(buf,
					`<td style="text-align:right;%s">%s<br>%s</td>`,
					style, costStr, ToElapsedTime(result.LastSubmittedAt))
			} else {
				fmt.Fprintf(buf, "<td>-</td>")
			}
		}
		fmt.Fprintf(buf, "</tr>")
	}

	fmt.Fprintf(buf, "</table>")
	fmt.Fprintf(buf, "</div>")
}
