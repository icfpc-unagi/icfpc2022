package util

import (
	"encoding/json"
	"fmt"
	"os"
	"regexp"
	"sort"
	"strconv"
)

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
