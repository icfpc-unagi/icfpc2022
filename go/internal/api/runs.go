package api

import (
	"context"
	"encoding/json"
	"io/ioutil"
	"net/http"
	"strconv"

	"github.com/golang/glog"
	"github.com/google/uuid"
	"github.com/icfpc-unagi/icfpc2022/go/pkg/db"
	"github.com/pkg/errors"
)

type RunAcquireResponse struct {
	RunID        int64  `json:"run_id" db:"run_id"`
	RunCommand   string `json:"run_command" db:"run_command"`
	RunSignature string `json:"run_signature" db:"run_signature"`
}

type RunExtendRequest struct {
	RunSignature string `json:"run_signature" db:"run_signature"`
}

type RunFlushRequest struct {
	RunSignature string `json:"run_signature" db:"run_signature"`
	RunCode      int64  `json:"run_code" db:"run_code"`
	SolutionISL  string `json:"solution_isl" db:"solution_isl"`
	LogID        string `json:"log_id"`
}

type RunAddRequest struct {
	ProblemID   int    `json:"problem_id"`
	ProgramID   int    `json:"program_id"`
	SolutionISL string `json:"solution_isl"`
	RunPipeline int    `json:"run_pipeline"`
	RunName     string `json:"run_name"`
	RunCommand  string `json:"run_command"`
}

type RunAddResponse struct {
	RunID    int   `json:"run_id"`
	RunScore int64 `json:"run_score"`
}

type RunEvaluateRequest struct {
	RunID int `json:"run_id"`
}

type RunEvaluateResponse struct {
	RunScore int64 `json:"run_score"`
}

func init() {
	http.HandleFunc("/"+PATH_PREFIX+"/run/acquire", handleRunAcquire)
	http.HandleFunc("/"+PATH_PREFIX+"/run/extend", handleRunExtend)
	http.HandleFunc("/"+PATH_PREFIX+"/run/flush", handleRunFlush)
	http.HandleFunc("/"+PATH_PREFIX+"/run/add", handleRunAdd)
	http.HandleFunc("/"+PATH_PREFIX+"/run/evaluate", handleRunEvaluate)
}

func handleRunAcquire(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	pipeline, _ := strconv.Atoi(r.URL.Query().Get("pipeline"))
	resp, err := doRunAcquire(ctx, pipeline)
	if err != nil {
		glog.Errorf("Failed to do run_acquire: %+v", err)
		w.WriteHeader(500)
		return
	}
	buf, err := json.Marshal(resp)
	if err != nil {
		glog.Errorf("Failed to marshal a response: %+v", err)
		w.WriteHeader(500)
		return
	}
	if _, err := w.Write(buf); err != nil {
		glog.Errorf("Failed to write buffer: %+v", err)
		w.WriteHeader(500)
		return
	}
}

func doRunAcquire(ctx context.Context, pipeline int) (*RunAcquireResponse, error) {
	var resp RunAcquireResponse
	signature := uuid.New().String()
	result, err := db.Execute(ctx, `
UPDATE runs
SET
	run_signature = ?,
	run_locked = CURRENT_TIMESTAMP() + INTERVAL 1 MINUTE 
WHERE
	run_locked < CURRENT_TIMESTAMP() AND
	run_pipeline = ?
ORDER BY run_locked LIMIT 1
`, signature, pipeline)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to run an SQL command")
	}
	n, err := result.RowsAffected()
	if err != nil {
		return nil, errors.Wrapf(err, "failed to get # of affected rows")
	}
	if n == 0 {
		return &resp, nil
	}
	if err := db.Row(ctx, &resp, `
SELECT run_id, run_command, run_signature
FROM runs WHERE run_signature = ?
LIMIT 1
`,
		signature); err != nil {
		return nil, errors.Wrapf(err, "")
	}
	return &resp, nil
}

func handleRunExtend(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	if r.Body == nil {
		w.WriteHeader(400)
		return
	}
	defer r.Body.Close()
	buf, err := ioutil.ReadAll(r.Body)
	if err != nil {
		w.WriteHeader(500)
		return
	}
	var req RunExtendRequest
	if err := json.Unmarshal(buf, &req); err != nil {
		glog.Errorf("Failed to parse a rqeuest: %+v", req)
		w.WriteHeader(400)
		return
	}
	err = doRunExtend(ctx, &req)
	if err != nil {
		glog.Errorf("Failed to extend the lock: %+v", err)
		w.WriteHeader(500)
		return
	}
}

func doRunExtend(ctx context.Context, req *RunExtendRequest) error {
	result, err := db.Execute(ctx, `
UPDATE runs
SET run_locked = CURRENT_TIMESTAMP() + INTERVAL 1 MINUTE
WHERE run_signature = ? AND CURRENT_TIMESTAMP() < run_locked
LIMIT 1
`, req.RunSignature)
	if err != nil {
		return errors.Wrapf(err, "failed to extend the lock")
	}
	n, err := result.RowsAffected()
	if err != nil {
		return errors.Wrapf(err, "failed to get # of rows affected")
	}
	if n == 0 {
		return errors.New("failed to extend the lock")
	}
	return nil
}

func handleRunFlush(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	if r.Body == nil {
		w.WriteHeader(400)
		return
	}
	defer r.Body.Close()
	buf, err := ioutil.ReadAll(r.Body)
	if err != nil {
		w.WriteHeader(500)
		return
	}
	var req RunFlushRequest
	if err := json.Unmarshal(buf, &req); err != nil {
		glog.Errorf("Failed to parse a rqeuest: %+v", req)
		w.WriteHeader(400)
		return
	}
	err = doRunFlush(ctx, &req)
	if err != nil {
		glog.Errorf("Failed to extend the lock: %+v", err)
		w.WriteHeader(500)
		return
	}
}

func doRunFlush(ctx context.Context, req *RunFlushRequest) error {
	solutionID, err := func() (*int, error) {
		if req.RunCode != 0 {
			return nil, nil
		}

		result, err := db.Execute(ctx, `
INSERT INTO solutions
SET solution_isl = ?`,
			req.SolutionISL)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to insert an ISL")
		}
		id, err := result.LastInsertId()
		if err != nil {
			return nil, errors.Wrapf(err, "failed to get an insert ID")
		}
		solutionID := int(id)
		return &solutionID, nil
	}()
	if err != nil {
		return err
	}

	var runID int
	if err := db.Cell(ctx, &runID, "SELECT run_id FROM runs WHERE run_signature = ? LIMIT 1", req.RunSignature); err != nil {
		return errors.Wrapf(err, "failed to get an run ID")
	}

	result, err := db.Execute(ctx, `
UPDATE runs
SET
	run_locked = NULL,
	run_signature = NULL,
	run_code = ?,
	solution_id = ?,
	log_id = ?
WHERE run_signature = ?
LIMIT 1
`,
		req.RunCode,
		solutionID,
		req.LogID,
		req.RunSignature)
	if err != nil {
		return errors.Wrapf(err, "failed to flush")
	}
	n, err := result.RowsAffected()
	if err != nil {
		return errors.Wrapf(err, "failed to flush")
	}
	if n == 0 {
		return errors.New("no run to flush")
	}

	if solutionID == nil {
		if _, err := RunEvaluate(ctx, &RunEvaluateRequest{RunID: runID}); err != nil {
			glog.Errorf("Failed to evaluate: %+v", err)
		}
	}

	return nil
}

func handleRunAdd(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	if r.Body == nil {
		w.WriteHeader(400)
		return
	}
	defer r.Body.Close()
	buf, err := ioutil.ReadAll(r.Body)
	if err != nil {
		w.WriteHeader(500)
		return
	}
	var req RunAddRequest
	if err := json.Unmarshal(buf, &req); err != nil {
		glog.Errorf("Failed to parse a rqeuest: %+v", req)
		w.WriteHeader(400)
		return
	}
	resp, err := RunAdd(ctx, &req)
	if err != nil {
		glog.Errorf("Failed to extend the lock: %+v", err)
		w.WriteHeader(500)
		return
	}
	buf, err = json.Marshal(resp)
	if err != nil {
		glog.Errorf("Failed to marshal a response: %+v", err)
		w.WriteHeader(500)
		return
	}
	if _, err := w.Write(buf); err != nil {
		glog.Errorf("Failed to write buffer: %+v", err)
		w.WriteHeader(500)
		return
	}
}

func RunAdd(ctx context.Context, req *RunAddRequest) (*RunAddResponse, error) {
	if req.SolutionISL != "" {
		return doRunAddWithSolution(ctx, req)
	}
	result, err := db.Execute(ctx, `
INSERT runs
SET
	problem_id = ?,
	program_id = ?,
	run_pipeline = ?
	run_name = ?,
	run_command = ?,
	run_locked = CURRENT_TIMESTAMP() - INTERVAL (1 + RAND()) * 3600 * 24 SECOND
`, req.ProblemID, req.ProgramID, req.RunPipeline, req.RunName, req.RunCommand)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to add a run")
	}
	id, err := result.LastInsertId()
	if err != nil {
		return nil, errors.Wrapf(err, "failed to get an insert ID")
	}
	return &RunAddResponse{RunID: int(id)}, nil
}

func doRunAddWithSolution(ctx context.Context, req *RunAddRequest) (*RunAddResponse, error) {
	result, err := db.Execute(ctx, `
INSERT INTO solutions
SET solution_isl = ?
`, req.SolutionISL)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to add a solution")
	}
	solutionID, err := result.LastInsertId()
	if err != nil {
		return nil, errors.Wrapf(err, "failed to get an insert ID")
	}

	result, err = db.Execute(ctx, `
INSERT INTO runs
SET
	problem_id = ?,
	program_id = ?,
	solution_id = ?,
	run_name = ?,
	run_command = ?,
	run_locked = NULL
`, req.ProblemID, req.ProgramID, solutionID, req.RunName, req.RunCommand)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to add a run")
	}
	id, err := result.LastInsertId()
	if err != nil {
		return nil, errors.Wrapf(err, "failed to get an insert ID")
	}

	resp := &RunAddResponse{RunID: int(id)}
	if eResp, err := RunEvaluate(ctx, &RunEvaluateRequest{RunID: resp.RunID}); err != nil {
		glog.Errorf("Failed to evaluate: %+v", err)
	} else {
		resp.RunScore = eResp.RunScore
	}

	if result, err := db.Execute(ctx, `
UPDATE runs SET run_score = ? WHERE run_id = ? LIMIT 1
`, resp.RunScore, resp.RunID); err != nil {
		glog.Errorf("Failed to update run_score: %+v", err)
	} else if n, err := result.RowsAffected(); err != nil {
		glog.Errorf("Failed to get # of affected rows: %+v", err)
	} else if n != 1 {
		glog.Errorf("Failed to update run_score: %d", n)
	}

	//if _, err := http.Get(fmt.Sprintf(
	//	"https://eoiu3fomuutxetv.m.pipedream.net/?run_id=%d", resp.RunID)); err != nil {
	//	glog.Errorf("Failed to trigger Pipedream: %+v", err)
	//}

	return resp, nil
}

func handleRunEvaluate(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	if r.Body == nil {
		w.WriteHeader(400)
		return
	}
	defer r.Body.Close()
	buf, err := ioutil.ReadAll(r.Body)
	if err != nil {
		w.WriteHeader(500)
		return
	}
	var req RunEvaluateRequest
	if err := json.Unmarshal(buf, &req); err != nil {
		glog.Errorf("Failed to parse a rqeuest: %+v", req)
		w.WriteHeader(400)
		return
	}
	resp, err := RunEvaluate(ctx, &req)
	if err != nil {
		glog.Errorf("Failed to evaluate: %+v", err)
		w.WriteHeader(500)
		return
	}
	buf, err = json.Marshal(resp)
	if err != nil {
		glog.Errorf("Failed to marshal a response: %+v", err)
		w.WriteHeader(500)
		return
	}
	if _, err := w.Write(buf); err != nil {
		glog.Errorf("Failed to write buffer: %+v", err)
		w.WriteHeader(500)
		return
	}
}

func RunEvaluate(ctx context.Context, req *RunEvaluateRequest) (*RunEvaluateResponse, error) {
	record := struct {
		SolutionISL string `db:"solution_isl"`
		ProblemID   int    `db:"problem_id"`
	}{}
	if err := db.Row(ctx, &record, `
SELECT solution_isl, problem_id FROM (solutions NATURAL JOIN (
	SELECT solution_id, problem_id FROM runs WHERE run_id = ? LIMIT 1
) t)
LIMIT 1
`, req.RunID); err != nil {
		return nil, errors.Wrapf(err, "failed to fetch a solution")
	}
	if record.ProblemID == 0 {
		return nil, errors.Errorf("no solution: run_id=%d", req.RunID)
	}

	resp, err := Evaluate(&EvaluateRequest{
		ProblemID: record.ProblemID,
		ISL:       record.SolutionISL,
	})
	if err != nil {
		return nil, errors.Wrapf(err, "failed to evaluate")
	}

	score := resp.Cost + resp.Similarity
	if score == 0 {
		glog.Errorf("Score is 0: run_id=%d", req.RunID)
	} else {
		if result, err := db.Execute(ctx, `
UPDATE runs SET run_score = ? WHERE run_id = ? LIMIT 1`, score, req.RunID); err != nil {
			return nil, errors.Errorf("failed to update score: %+v", err)
		} else if n, err := result.RowsAffected(); err != nil {
			return nil, errors.Errorf("failed to update score: %+v", err)
		} else if n != 1 {
			return nil, errors.Errorf("failed to update score: %+v", err)
		}
	}

	return &RunEvaluateResponse{RunScore: score}, nil
}
