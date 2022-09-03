package api

import (
	"context"

	"github.com/golang/glog"
	"github.com/icfpc-unagi/icfpc2022/go/internal/util"

	"github.com/icfpc-unagi/icfpc2022/go/pkg/db"
	"github.com/pkg/errors"
)

type InsertProgramRequest struct {
	ProgramName string `json:"program_id"`
	ProgramCode string `json:"program_code"`
}

type InsertProgramResponse struct {
	ProgramID int   `json:"program_id"`
	TaskIDs   []int `json:"task_ids"`
}

func InsertProgram(req *InsertProgramRequest) (*InsertProgramResponse, error) {
	if req.ProgramName == "" {
		return nil, errors.Errorf("program_name is required")
	}
	if req.ProgramCode == "" {
		return nil, errors.Errorf("program_code is required")
	}

	result, err := db.Execute(context.Background(), `
INSERT INTO programs SET program_name = ?, program_code = ?
`,
		req.ProgramName, req.ProgramCode)
	if err != nil {
		return nil, errors.Errorf("failed to insert a task: %+v", err)
	}

	programID, err := result.LastInsertId()
	if err != nil {
		return nil, errors.Errorf("failed to get the last inserted ID: %+v", err)
	}

	resp := &InsertProgramResponse{ProgramID: int(programID)}

	failure := 0
	for _, p := range util.Problems() {
		for i := 0; i < 3; i++ {
			iResp, err := InsertTask(&InsertTaskRequest{
				ProblemID: p.ID,
				ProgramID: resp.ProgramID,
			})
			if err != nil {
				glog.Errorf("Failed to insert a task: %+v", err)
				failure++
				continue
			}
			resp.TaskIDs = append(resp.TaskIDs, iResp.TaskID)
			break
		}
	}
	return resp, nil
}
