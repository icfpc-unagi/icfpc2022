package handler

import (
	"bytes"
	"context"
	"fmt"
	"html"
	"net/http"
	"sort"
	"strings"

	"github.com/pkg/errors"

	"github.com/icfpc-unagi/icfpc2022/go/pkg/db"

	"github.com/icfpc-unagi/icfpc2022/go/internal/official"
)

func init() {
	http.HandleFunc("/scoreboard", scoreboardTemplate)
}

type scoreboardRecord struct {
	UserKey     string
	UserName    string
	IsInternal  bool
	RunID       int
	ProblemID   int
	Score       int64
	Updated     string
	ProblemRank int
}

func scoreboardTemplate(w http.ResponseWriter, r *http.Request) {
	buf := &bytes.Buffer{}
	defer func() { Template(w, buf.Bytes()) }()

	fmt.Fprintf(buf, "<h1>順位表</h1>")
	records := getAllRecords(buf)
	displayScoreboardByClass(buf, records)
	displayScoreboard(buf, records)
}

func getAllRecords(buf *bytes.Buffer) []*scoreboardRecord {
	r1, err := scoreboardToRecords()
	if err != nil {
		fmt.Fprintf(buf, `<pre class="alert-danger">%s</pre>`,
			html.EscapeString(fmt.Sprintf("%+v", err)))
	}
	r2, err := internalScoreboardToRecords()
	if err != nil {
		fmt.Fprintf(buf, `<pre class="alert-danger">%s</pre>`,
			html.EscapeString(fmt.Sprintf("%+v", err)))
	}
	r3, err := generateMergedRecords(r2)
	if err != nil {
		fmt.Fprintf(buf, `<pre class="alert-danger">%s</pre>`,
			html.EscapeString(fmt.Sprintf("%+v", err)))
	}
	records := append(r1, r2...)
	records = append(records, r3...)

	sort.SliceStable(records, func(i, j int) bool {
		if records[i].ProblemID != records[j].ProblemID {
			return records[i].ProblemID < records[j].ProblemID
		}
		return records[i].Score < records[j].Score
	})

	count := 0
	for i, r := range records {
		if i == 0 || r.ProblemID != records[i-1].ProblemID {
			count = 0
		}
		if count > 0 && r.Score == records[i-1].Score {
			r.ProblemRank = records[i-1].ProblemRank
		} else {
			r.ProblemRank = count + 1
		}
		count++
	}

	return records
}

func scoreboardToRecords() ([]*scoreboardRecord, error) {
	scoreboard, err := official.Scoreboard()
	if err != nil {
		return nil, err
	}

	results := make([]*scoreboardRecord, 0)
	for _, u := range scoreboard.Users {
		for _, r := range u.Results {
			if r.MinCost == 0 {
				continue
			}
			results = append(results, &scoreboardRecord{
				UserKey:   fmt.Sprintf("USER_ID$$$%d", u.UserID),
				UserName:  u.TeamName,
				ProblemID: r.ProblemID,
				Score:     int64(r.MinCost),
				Updated:   ToElapsedTime(r.LastSubmittedAt),
			})
		}
	}
	return results, nil
}

func internalScoreboardToRecords() ([]*scoreboardRecord, error) {
	type runRecord struct {
		RunName    string `db:"run_name"`
		ProblemID  int    `db:"problem_id"`
		RunScore   int    `db:"run_score"`
		RunID      int    `db:"run_id"`
		RunCreated string `db:"run_created"`
	}
	records := make([]runRecord, 0)
	if err := db.Select(context.Background(), &records, `
SELECT
    run_name,
	problem_id,
	run_score,
	run_id,
	run_created
FROM
    (
    SELECT
        run_name AS run_name,
        problem_id,
        run_score,
        MIN(run_id) AS run_id,
        MIN(run_created) AS run_created
    FROM
        (
            runs
        NATURAL JOIN(
            SELECT
                run_name,
                problem_id,
                MIN(run_score) AS run_score
            FROM
                runs
            WHERE
                program_id = 0
            GROUP BY
                run_name,
                problem_id
        ) t
        )
    GROUP BY
        run_name,
        problem_id,
        run_score
    UNION
SELECT
    CONCAT(
        CAST(program_id AS CHAR),
        "$$$",
        program_name
    ) AS run_name,
    problem_id,
    run_score,
    MIN(run_id) AS run_id,
    MIN(run_created) AS run_created
FROM
    (
        runs
    NATURAL JOIN programs NATURAL JOIN(
        SELECT
            program_id,
            problem_id,
            MIN(run_score) AS run_score
        FROM
            runs
        NATURAL JOIN programs WHERE run_score IS NOT NULL
        GROUP BY
            program_id,
            problem_id
    ) t
    )
GROUP BY
    program_id,
    problem_id,
    run_score
) t`); err != nil {
		return nil, errors.Wrapf(err, "failed to fetch internal submissions")
	}

	results := make([]*scoreboardRecord, 0)
	for _, r := range records {
		if r.RunName == "" {
			r.RunName = "unknown-solver"
		}
		name := r.RunName
		if strings.Contains(name, "$$$") {
			name = strings.SplitN(name, "$$$", 2)[1]
		}
		results = append(results, &scoreboardRecord{
			UserKey:    r.RunName,
			UserName:   name,
			IsInternal: true,
			RunID:      r.RunID,
			ProblemID:  r.ProblemID,
			Score:      int64(r.RunScore),
			Updated:    ToElapsedTime(r.RunCreated),
		})
	}
	return results, nil
}

func generateMergedRecords(records []*scoreboardRecord) ([]*scoreboardRecord, error) {
	best := map[int]*scoreboardRecord{}
	for _, r := range records {
		if !r.IsInternal && r.UserName != "Unagi" {
			continue
		}
		if best[r.ProblemID] == nil || r.Score < best[r.ProblemID].Score {
			best[r.ProblemID] = r
		}
	}
	result := make([]*scoreboardRecord, 0)
	for _, r := range best {
		rc := *r
		rc.UserKey = "BEST_RECORD"
		rc.UserName = "Unagi (internal)"
		rc.IsInternal = true
		result = append(result, &rc)
	}
	return result, nil
}

func displayScoreboardByClass(buf *bytes.Buffer, records []*scoreboardRecord) {
	type rankingRecord struct {
		UserKey    string
		UserName   string
		Rank       int
		Solved     int
		Score      int64
		IsInternal bool
	}

	thresholds := []int{35, 30, 25, 20}
	thresholdRanks := make([][]*rankingRecord, 0)
	for _, t := range thresholds {
		teams := map[string]*rankingRecord{}
		for _, r := range records {
			if r.ProblemID > t {
				continue
			}
			if _, ok := teams[r.UserKey]; !ok {
				teams[r.UserKey] = &rankingRecord{
					UserKey:    r.UserKey,
					UserName:   r.UserName,
					IsInternal: r.IsInternal,
				}
			}
			t := teams[r.UserKey]
			t.Score += r.Score
			t.Solved++
		}
		ranks := make([]*rankingRecord, 0)
		for _, r := range teams {
			ranks = append(ranks, r)
		}
		sort.SliceStable(ranks, func(i, j int) bool {
			if ranks[i].Solved != ranks[j].Solved {
				return ranks[i].Solved > ranks[j].Solved
			}
			return ranks[i].Score < ranks[j].Score
		})
		for i := range ranks {
			ranks[i].Rank = i + 1
			if i != 0 && ranks[i].Score == ranks[i-1].Score {
				ranks[i].Rank = ranks[i-1].Rank
			}
		}
		thresholdRanks = append(thresholdRanks, ranks)
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
			if thresholdRanks[t][i].UserName == "Unagi" {
				style += "background: #cdf; color: red; font-weight: bold;"
			} else if thresholdRanks[t][i].UserName == "Unagi (internal)" {
				style += "background: #fdc; color: red; font-weight: bold;"
			} else if thresholdRanks[t][i].IsInternal {
				style += "font-weight: bold;"
			}
			fmt.Fprintf(buf, `<td style="%s">%s</td>`, style, html.EscapeString(thresholdRanks[t][i].UserName))
			fmt.Fprintf(buf, `<td style="%s; text-align: right">%d</td>`, style, thresholdRanks[t][i].Score)
		}
		fmt.Fprintf(buf, "</tr>")
	}
	fmt.Fprintf(buf, "</table>")
}

func displayScoreboard(buf *bytes.Buffer, records []*scoreboardRecord) {
	problemIDs := func() []int {
		m := map[int]struct{}{}
		for _, r := range records {
			m[r.ProblemID] = struct{}{}
		}
		ids := make([]int, 0)
		for i := range m {
			ids = append(ids, i)
		}
		sort.Ints(ids)
		return ids
	}()

	// teams[UserKey][ProblemID]
	teams := map[string]map[int]*scoreboardRecord{}
	teamScore := map[string]int64{}
	teamNames := map[string]*scoreboardRecord{}
	for _, r := range records {
		if _, ok := teams[r.UserKey]; !ok {
			teams[r.UserKey] = map[int]*scoreboardRecord{}
		}
		teams[r.UserKey][r.ProblemID] = r
		teamScore[r.UserKey] = teamScore[r.UserKey] + r.Score
		teamNames[r.UserKey] = r
	}

	ranks := make([]string, 0)
	for k := range teams {
		ranks = append(ranks, k)
	}
	sort.SliceStable(ranks, func(i, j int) bool {
		solvedI, solvedJ := len(teams[ranks[i]]), len(teams[ranks[j]])
		if solvedI != solvedJ {
			return solvedI > solvedJ
		}
		return teamScore[ranks[i]] < teamScore[ranks[j]]
	})

	fmt.Fprintf(buf, "<h2>スコア詳細</h2>")
	fmt.Fprint(buf,
		`凡例: <span style="color:red;font-weight:bold">1位</span>`+
			` <span style="color:#880;font-weight:bold">5位以内</span>`+
			` <span style="font-weight:bold">10位以内</span>`+
			` <span>その他</span><br><br>`)
	fmt.Fprint(buf, `<div style="overflow-x:scroll;width:100%;">`)
	fmt.Fprint(buf, `<table style="font-size:50%;"><tr><th>Team</th>`)
	for _, p := range problemIDs {
		fmt.Fprintf(buf, "<th>問%d</th>", p)
	}
	fmt.Fprintf(buf, "</tr>")

	for _, k := range ranks {
		style := ""
		if teamNames[k].UserName == "Unagi" {
			style += "background: #cdf;"
		} else if teamNames[k].UserName == "Unagi (internal)" {
			style += "background: #fdc;"
		} else if teamNames[k].IsInternal {
			style += "background: #dfc;"
		}
		fmt.Fprintf(buf, `<tr style="%s"><td>%s</td>`,
			style,
			html.EscapeString(teamNames[k].UserName))
		for _, p := range problemIDs {
			if r, ok := teams[k][p]; ok {
				costStr := "&gt;1e6"
				if r.Score < 1000000 {
					costStr = fmt.Sprintf("%d", r.Score)
				}
				style := ""
				if r.ProblemRank == 1 {
					style = "color:red; font-weight: bold;"
				} else if r.ProblemRank < 5 {
					style = "color: #880; font-weight: bold;"
				} else if r.ProblemRank < 10 {
					style = "font-weight: bold;"
				}
				fmt.Fprintf(buf,
					`<td style="text-align:right;%s">%s<br>%s</td>`,
					style, costStr, r.Updated)
			} else {
				fmt.Fprintf(buf, "<td>-</td>")
			}
		}
		fmt.Fprintf(buf, "</tr>")
	}

	fmt.Fprintf(buf, "</table>")
	fmt.Fprintf(buf, "</div>")
}
