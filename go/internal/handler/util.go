package handler

import (
	"encoding/json"
	"fmt"
	"os"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"time"
)

func ParseTimestamp(t string) int64 {
	t = strings.Split(t, ".")[0]
	t = strings.TrimSuffix(t, "Z")
	x, _ := time.Parse("2006-01-02T15:04:05", t)
	start, _ := time.Parse(
		"2006-01-02T15:04:05", "2022-09-02T12:00:00")
	return x.Unix() - start.Unix()
}

func ToElapsedTime(t string) string {
	e := ParseTimestamp(t)
	return fmt.Sprintf("%02d:%02d", e/3600, e/60%60)
}

type Problem struct {
	ID   int    `json:"int"`
	Name string `json:"name"`
}

func Problems() []Problem {
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

	problems := []Problem{}
	for _, problemID := range problemIDs {
		data, _ := os.ReadFile(fmt.Sprintf("/work/problems/%d.json", problemID))
		problem := Problem{}
		problem.ID = problemID
		json.Unmarshal(data, &problem)
		problems = append(problems, problem)
	}

	return problems
}
