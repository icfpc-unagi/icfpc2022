package handler

import (
	"bytes"
	"fmt"
	"html"
	"net/http"
	"sort"

	"github.com/icfpc-unagi/icfpc2022/go/internal/official"
)

func init() {
	http.HandleFunc("/scoreboard", scoreboardTemplate)
}

func scoreboardTemplate(w http.ResponseWriter, r *http.Request) {
	buf := &bytes.Buffer{}
	defer func() { Template(w, buf.Bytes()) }()

	fmt.Fprintf(buf, "<h1>順位表</h1>")

	scoreboard, err := official.Scoreboard()
	if err != nil {
		fmt.Fprintf(buf, "<pre>%s</pre>",
			html.EscapeString(fmt.Sprintf("%+v", err)))
		return
	}

	thresholds := []int{35, 30, 25, 20}
	type rankingRecord struct {
		TeamName string
		Score    int
	}
	rankings := make([][]*rankingRecord, len(thresholds))

	for _, u := range scoreboard.Users {
		solved := map[int]int{}
		for _, r := range u.Results {
			if r.SubmissionCount != 0 {
				solved[r.ProblemID] = r.MinCost
			}
		}

		for tIndex, t := range thresholds {
			score := 0
			for i := 1; i <= t; i++ {
				if s, ok := solved[i]; ok {
					score = score + s
					continue
				}
				score = -1
				break
			}
			if score > 0 {
				rankings[tIndex] = append(rankings[tIndex], &rankingRecord{
					TeamName: u.TeamName,
					Score:    score,
				})
			}
		}
	}
	for _, r := range rankings {
		sort.SliceStable(r, func(i, j int) bool {
			return r[i].Score < r[j].Score
		})
	}

	fmt.Fprintf(buf, "<h2>レベル別ランキング</h2>")
	fmt.Fprintf(buf, `<table style="width: 100%%; table-layout: fixed; white-space: nowrap; font-size: 70%%"><tr><td style="width: 5ex">順位</td>`)
	for _, t := range thresholds {
		fmt.Fprintf(buf, `<td width="30%%">%d問級</td><td style="width: 8ex"></td>`, t)
	}
	fmt.Fprintf(buf, `</tr>`)

	for i := 0; i < 20; i++ {
		fmt.Fprintf(buf, `<tr><td>%d 位</td>`, i+1)
		for t := range thresholds {
			style := "overflow: hidden;"
			if rankings[t][i].TeamName == "Unagi" {
				style += "background: #cdf; color: red; font-weight: bold;"
			}
			fmt.Fprintf(buf, `<td style="%s">%s</td>`, style, html.EscapeString(rankings[t][i].TeamName))
			fmt.Fprintf(buf, `<td style="%s; text-align: right">%d</td>`, style, rankings[t][i].Score)
		}
		fmt.Fprintf(buf, "</tr>")
	}
	fmt.Fprintf(buf, "</table>")

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

	fmt.Fprintf(buf, "<h2>スコア詳細</h2>")
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
