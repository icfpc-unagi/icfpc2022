package api

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strconv"

	"github.com/icfpc-unagi/icfpc2022/go/pkg/db"
	"github.com/pkg/errors"
)

func init() {
	http.HandleFunc("/"+PATH_PREFIX+"/insert_task", insertTaskHandler)
}

func insertTaskHandler(w http.ResponseWriter, r *http.Request) {
	req := &InsertTaskRequest{}
	if r.Header.Get("Content-Type") == "application/json" {
		body, _ := io.ReadAll(r.Body)
		if err := json.Unmarshal(body, req); err != nil {
			w.WriteHeader(400)
			fmt.Fprintf(w, "Invalid request: %+v", err)
			return
		}
	} else {
		problemID, _ := strconv.Atoi(r.PostFormValue("problem_id"))
		req.ProblemID = problemID
		programID, _ := strconv.Atoi(r.PostFormValue("program_id"))
		req.ProgramID = programID
		req.TaskSolution = r.PostFormValue("task_solution")
		req.TaskName = r.PostFormValue("task_name")
	}
	resp, err := InsertTask(req)
	if err != nil {
		w.WriteHeader(500)
		fmt.Fprintf(w, "Failed to insert a task: %+v", err)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	j, _ := json.Marshal(resp)
	fmt.Fprintf(w, "%s", j)
}

type InsertTaskRequest struct {
	ProblemID int `json:"problem_id"`
	// Set 1 if manual.
	ProgramID int `json:"program_id"`
	// For manual usage only.
	TaskSolution string `json:"task_solution"`
	// Required if manual.
	TaskName string `json:"task_name"`
}

type InsertTaskResponse struct {
	TaskID int `json:"task_id"`
	// If 0, the solution is broken or the server is busy.
	TaskSolutionScore int64 `json:"task_solution_score"`
}

func InsertTask(req *InsertTaskRequest) (*InsertTaskResponse, error) {
	if req == nil {
		return nil, errors.Errorf("request must not be nil")
	}

	if req.ProgramID == 0 {
		req.ProgramID = 1
	}
	if req.ProblemID == 0 {
		return nil, errors.Errorf("problem_id is required")
	}

	// If manual.
	if req.ProgramID == 1 {
		if req.TaskSolution == "" {
			return nil, errors.Errorf("task_solution is required when manual")
		}
		if req.TaskName == "" {
			return nil, errors.Errorf("task_name is required when manual")
		}
	} else {
		if req.TaskSolution != "" {
			return nil, errors.Errorf("task_solution must be empty if not manual")
		}
	}

	result, err := db.Execute(context.Background(), `
INSERT INTO tasks SET
	problem_id = ?, program_id = ?, task_name = ?,
	task_locked = CURRENT_TIMESTAMP() - INTERVAL (1 + RAND()) * 3600 * 24 SECOND
`,
		req.ProblemID, req.ProgramID, req.TaskName)
	if err != nil {
		return nil, errors.Errorf("failed to insert a task: %+v", err)
	}

	taskID, err := result.LastInsertId()
	if err != nil {
		return nil, errors.Errorf("failed to get the last inserted ID: %+v", err)
	}

	resp := &InsertTaskResponse{TaskID: int(taskID)}
	if req.TaskSolution != "" {
		uResp, err := UpdateTask(&UpdateTaskRequest{
			TaskID:       int(taskID),
			TaskSolution: req.TaskSolution,
		})
		if err != nil {
			return nil, err
		}
		resp.TaskID = uResp.TaskID
		resp.TaskSolutionScore = uResp.TaskSolutionScore
	}
	return resp, nil
}
