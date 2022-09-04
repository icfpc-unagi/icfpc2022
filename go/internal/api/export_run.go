package api

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"

	"github.com/golang/glog"

	"github.com/icfpc-unagi/icfpc2022/go/pkg/db"
	"github.com/pkg/errors"
)

type ExportResponse struct {
	RunID      int    `json:"id" db:"run_id"`
	ProblemID  int    `json:"problem_id" db:"problem_id"`
	RunCreated string `json:"submitted_at" db:"run_created"`
	RunScore   *int64 `json:"cost" db:"run_score"`
	ISL        string `json:"-" db:"solution_isl"`
}

func init() {
	http.HandleFunc("/"+PATH_PREFIX+"/export/json", exportJSONHandler)
	http.HandleFunc("/"+PATH_PREFIX+"/export/isl", exportISLHandler)
}

func exportJSONHandler(w http.ResponseWriter, r *http.Request) {
	exportHandler(w, r, true)
}

func exportISLHandler(w http.ResponseWriter, r *http.Request) {
	exportHandler(w, r, false)
}

func exportHandler(w http.ResponseWriter, r *http.Request, isJSON bool) {
	runID, _ := strconv.Atoi(r.URL.Query().Get("run_id"))
	if runID == 0 {
		w.WriteHeader(400)
		fmt.Fprintf(w, "Invalid run_id")
		return
	}

	resp, err := exportRun(runID)
	if err != nil {
		w.WriteHeader(500)
		glog.Errorf("Failed to export: %+v", err)
		fmt.Fprintf(w, "Failed to export: %+v", err)
		return
	}

	if resp.RunID == 0 {
		w.WriteHeader(404)
		fmt.Fprintf(w, "No such run ID: %d", resp.RunID)
	}

	if isJSON {
		buf, _ := json.Marshal(resp)
		w.Write(buf)
	} else {
		fmt.Fprint(w, resp.ISL)
	}
}

func exportRun(runID int) (*ExportResponse, error) {
	resp := &ExportResponse{}
	if err := db.Row(context.Background(), resp, `
SELECT
    run_id,
    problem_id,
    IFNULL(
        solution_isl,
        submission_solution
    ) AS solution_isl,
    run_score,
    run_created
FROM
    runs
NATURAL LEFT JOIN submissions NATURAL LEFT JOIN solutions WHERE run_id = ?`, runID); err != nil {
		return nil, errors.Wrapf(err, "failed to get a run")
	}
	return resp, nil
}
