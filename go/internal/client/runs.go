package client

import (
	"bytes"
	"context"
	"encoding/json"
	"flag"
	"io/ioutil"
	"net/http"

	"github.com/icfpc-unagi/icfpc2022/go/internal/api"

	"github.com/pkg/errors"
)

var apiHost = flag.String("host", "https://icfpc.sx9.jp/"+api.PATH_PREFIX, "")

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
	RunStdout    string `json:"run_stdout" db:"run_stdout"`
	RunStderr    string `json:"run_stderr" db:"run_stderr"`
}

func RunAcquire(ctx context.Context) (*RunAcquireResponse, error) {
	req, err := http.NewRequest("POST", *apiHost+"/run/acquire", nil)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to create a request")
	}
	client := http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to send a request")
	}
	if resp.Body == nil {
		return nil, errors.New("empty response")
	}
	defer resp.Body.Close()
	buf, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to read body")
	}
	var apiResp RunAcquireResponse
	if err := json.Unmarshal(buf, &apiResp); err != nil {
		return nil, errors.Wrapf(err, "failed to parse a response")
	}
	return &apiResp, nil
}

func RunExtend(ctx context.Context, signature string) error {
	var apiReq RunExtendRequest
	apiReq.RunSignature = signature
	buf, err := json.Marshal(apiReq)
	if err != nil {
		return errors.Wrapf(err, "failed to marshal request")
	}
	req, err := http.NewRequest(
		"POST", *apiHost+"/run/extend",
		bytes.NewBuffer(buf))
	if err != nil {
		return errors.Wrap(err, "failed to create a request")
	}
	client := http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		return errors.Wrapf(err, "failed to send a request")
	}
	if resp.StatusCode != 200 {
		return errors.Errorf(
			"failed to extend: status_code=%d", resp.StatusCode)
	}
	if resp.Body != nil {
		defer resp.Body.Close()
	}
	return nil
}

func RunFlush(ctx context.Context, req *RunFlushRequest) error {
	buf, err := json.Marshal(req)
	if err != nil {
		return errors.Wrapf(err, "failed to marshal request")
	}
	httpReq, err := http.NewRequest(
		"POST", *apiHost+"/run/flush",
		bytes.NewBuffer(buf))
	if err != nil {
		return errors.Wrap(err, "failed to create a request")
	}
	client := http.Client{}
	resp, err := client.Do(httpReq)
	if err != nil {
		return errors.Wrapf(err, "failed to send a request")
	}
	if resp.StatusCode != 200 {
		return errors.Errorf(
			"failed to extend: status_code=%d", resp.StatusCode)
	}
	if resp.Body != nil {
		defer resp.Body.Close()
	}
	return nil
}
