package api

import (
	"context"
	"encoding/json"
	"io/ioutil"
	"net/http"

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
}

func init() {
	http.HandleFunc("/"+PATH_PREFIX+"/run/acquire", handleRunAcquire)
	http.HandleFunc("/"+PATH_PREFIX+"/run/extend", handleRunExtend)
	http.HandleFunc("/"+PATH_PREFIX+"/run/flush", handleRunFlush)
}

func handleRunAcquire(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	resp, err := doRunAcquire(ctx)
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

func doRunAcquire(ctx context.Context) (*RunAcquireResponse, error) {
	var resp RunAcquireResponse
	signature := uuid.New().String()
	result, err := db.Execute(ctx, `
UPDATE runs
SET
	run_signature = ?,
	run_locked = CURRENT_TIMESTAMP() + INTERVAL 1 MINUTE 
WHERE run_locked < CURRENT_TIMESTAMP()
ORDER BY run_locked LIMIT 1
`, signature)
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
	result, err := db.Execute(ctx, `
INSERT INTO solutions
SET solution_isl = ?`,
		req.SolutionISL)
	if err != nil {
		return errors.Wrapf(err, "failed to insert an ISL")
	}
	id, err := result.LastInsertId()
	if err != nil {
		return errors.Wrapf(err, "failed to get an insert ID")
	}
	result, err = db.Execute(ctx, `
UPDATE runs
SET
	run_locked = NULL,
	run_signature = NULL,
	run_code = ?,
	solution_id = ?
WHERE run_signature = ?
LIMIT 1
`,
		req.RunCode,
		id,
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
	return nil
}
