package api

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strconv"

	"github.com/golang/glog"

	"github.com/pkg/errors"

	"github.com/icfpc-unagi/icfpc2022/go/pkg/db"
)

func init() {
	http.HandleFunc("/"+PATH_PREFIX+"/update_task", updateTaskHandler)
}

func updateTaskHandler(w http.ResponseWriter, r *http.Request) {
	req := &UpdateTaskRequest{}
	if r.Header.Get("Content-Type") == "application/json" {
		body, _ := io.ReadAll(r.Body)
		if err := json.Unmarshal(body, req); err != nil {
			w.WriteHeader(400)
			fmt.Fprintf(w, "Invalid request: %+v", err)
			return
		}
	} else {
		taskID, _ := strconv.Atoi(r.PostFormValue("task_id"))
		req.TaskID = taskID
		req.TaskSolution = r.PostFormValue("task_solution")
	}
	resp, err := UpdateTask(req)
	if err != nil {
		w.WriteHeader(500)
		fmt.Fprintf(w, "Failed to insert a task: %+v", err)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	j, _ := json.Marshal(resp)
	fmt.Fprintf(w, "%s", j)
}

type UpdateTaskRequest struct {
	TaskID       int    `json:"task_id"`
	TaskSolution string `json:"task_solution"`
}

type UpdateTaskResponse struct {
	TaskID            int   `json:"task_id"`
	TaskSolutionScore int64 `json:"task_solution_score"`
}

func UpdateTask(req *UpdateTaskRequest) (*UpdateTaskResponse, error) {
	result, err := db.Execute(context.Background(), `
UPDATE tasks
SET task_locked = CURRENT_TIMESTAMP() + INTERVAL 1 + POWER(3, task_trial - 1) MINUTE
WHERE task_id = ? AND task_locked < CURRENT_TIMESTAMP()`,
		req.TaskID)
	if err != nil {
		return nil, errors.Errorf("failed to extend the task: %+v", err)
	}

	if numRows, err := result.RowsAffected(); err != nil {
		return nil, errors.Errorf("failed to fetch # of affected rows: %+v", err)
	} else if numRows == 0 {
		return &UpdateTaskResponse{}, nil
	}

	resp := &UpdateTaskResponse{TaskID: req.TaskID}
	if req.TaskSolution == "" {
		return resp, nil
	}

	problemID := 0
	if err := db.Cell(context.Background(), &problemID,
		`SELECT problem_id FROM tasks WHERE task_id = ?`,
		req.TaskID); err != nil {
		return nil, errors.Errorf("failed to get problem ID: %+v", err)
	}

	if eResp, err := Evaluate(&EvaluateRequest{
		ProblemID: problemID,
		ISL:       req.TaskSolution,
	}); err != nil {
		glog.Errorf("Failed to evaluate a solution: %+v", err)
	} else {
		resp.TaskSolutionScore = eResp.Cost + eResp.Similarity
	}

	if result, err := db.Execute(context.Background(), `
UPDATE tasks
SET task_solution = ?, task_solution_score = ?, task_locked = NULL
WHERE task_id = ? LIMIT 1
`, req.TaskSolution, resp.TaskSolutionScore, req.TaskID); err != nil {
		return nil, errors.Errorf("failed to finalize the task: %+v", err)
	} else if numRows, err := result.RowsAffected(); err != nil {
		return nil, errors.Errorf("failed to update task solution: %+v", err)
	} else if numRows != 1 {
		return nil, errors.Errorf("failed to update task solution: %d", numRows)
	}

	return resp, nil
}
