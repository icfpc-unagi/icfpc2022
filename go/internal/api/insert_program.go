package api

import (
	"context"
	"fmt"
	"strings"

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
	RunIDs    []int `json:"run_ids"`
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
		code := req.ProgramCode
		code = strings.ReplaceAll(
			code, "{{PROBLEM_ID}}", fmt.Sprintf("%d", p.ID))
		for i := 0; i < 3; i++ {
			runResp, err := RunAdd(context.Background(), &RunAddRequest{
				ProblemID:  p.ID,
				ProgramID:  resp.ProgramID,
				RunCommand: code,
			})
			if err != nil {
				glog.Errorf("Failed to insert a task: %+v", err)
				failure++
				if failure > 10 {
					return resp, nil
				}
				continue
			}
			resp.RunIDs = append(resp.RunIDs, runResp.RunID)
			break
		}
	}
	return resp, nil
}
