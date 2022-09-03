package api

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"strconv"
	"strings"
	"time"

	"github.com/pkg/errors"
)

func init() {
	http.HandleFunc("/"+PATH_PREFIX+"/evaluate", evaluateHandler)
}

func evaluateHandler(w http.ResponseWriter, r *http.Request) {
	req := &EvaluateRequest{}
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
		req.ISL = r.PostFormValue("isl")
	}
	resp, err := Evaluate(req)
	if err != nil {
		w.WriteHeader(500)
		fmt.Fprintf(w, "Failed to evaluate: %+v", err)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	j, _ := json.Marshal(resp)
	fmt.Fprintf(w, "%s", j)
}

type EvaluateRequest struct {
	ProblemID int    `json:"problem_id"`
	ISL       string `json:"isl"`
}

type EvaluateResponse struct {
	ProblemID  int    `json:"problem_id"`
	Cost       int64  `json:"cost"`
	Similarity int64  `json:"similarity"`
	Image      string `json:"image"`
}

func Evaluate(req *EvaluateRequest) (*EvaluateResponse, error) {
	if req.ProblemID == 0 {
		return nil, errors.Errorf("problem_id must be given")
	}
	if req.ISL == "" {
		return nil, errors.Errorf("ISL must be given")
	}

	cmd := exec.Command("/usr/local/bin/evaluate", strconv.Itoa(req.ProblemID), "/dev/stdin")
	cmd.Dir = "/work"
	cmd.Stdin = strings.NewReader(req.ISL)
	cmd.Stderr = os.Stderr

	resp := &EvaluateResponse{}
	errChan := make(chan error)

	go func() {
		defer close(errChan)
		buf, err := cmd.Output()
		if err != nil {
			errChan <- errors.Errorf("failed to run command: %+v", err)
			return
		}
		if err := json.Unmarshal(buf, resp); err != nil {
			errChan <- errors.Errorf("failed to parse JSON: %+v", err)
			return
		}
	}()

	select {
	case err := <-errChan:
		if err == nil {
			return resp, nil
		}
		return nil, err
	case <-time.After(30 * time.Second):
		return nil, errors.Errorf("evaluation time out")
	}
}
