package api

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"

	"github.com/icfpc-unagi/icfpc2022/go/pkg/db"
	"github.com/pkg/errors"
)

func init() {
	http.HandleFunc("/"+PATH_PREFIX+"/acquire_task", acquireTaskHandler)
}

func acquireTaskHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != "POST" {
		w.WriteHeader(400)
		fmt.Fprintf(w, "Use POST method.")
		return
	}

	resp, err := AcquireTask()
	if err != nil {
		w.WriteHeader(500)
		fmt.Fprintf(w, "Failed to insert a task: %+v", err)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	j, _ := json.Marshal(resp)
	fmt.Fprintf(w, "%s", j)
}

type AcquireTaskResponse struct {
	TaskID      int    `json:"task_id",db:"task_id"`
	ProgramID   int64  `json:"program_id",db:"program_id"`
	ProgramName string `json:"program_name",db:"program_name"`
	ProgramCode string `json:"program_code",db:"program_code"`
}

func AcquireTask() (*AcquireTaskResponse, error) {
	taskIDs := make([]struct {
		TaskID int `db:"task_id"`
	}, 0)
	if err := db.Select(context.Background(), &taskIDs, `
SELECT task_id FROM tasks
WHERE task_locked < CURRENT_TIMESTAMP()
ORDER BY task_locked LIMIT 10`); err != nil {
		return nil, errors.Errorf("failed to get candidate tasks: %+v", err)
	}

	for _, taskID := range taskIDs {
		result, err := db.Execute(context.Background(), `
UPDATE tasks
SET
	task_locked = CURRENT_TIMESTAMP() + INTERVAL 1 + POWER(3, task_trial) MINUTE,
	task_trial = task_trial + 1,
WHERE
	task_id = ? AND task_locked < CURRENT_TIMESTAMP()
LIMIT 1
`, taskID)
		if err != nil {
			return nil, errors.Errorf("failed to acquire a task: %+v", err)
		}
		if numRows, err := result.RowsAffected(); err != nil {
			return nil, errors.Errorf("failed to get # of affected rows: %+v", err)
		} else if numRows == 0 {
			continue
		}

		resp := &AcquireTaskResponse{}
		if err := db.Row(context.Background(), resp, `
SELECT task_id, program_id, program_name, program_code
FROM tasks NATURAL JOIN programs
WHERE task_id = ? LIMIT 1`); err != nil {
			return nil, errors.Errorf("failed to get program information: %+v", err)
		}
		return resp, nil
	}
	return &AcquireTaskResponse{}, nil
}
