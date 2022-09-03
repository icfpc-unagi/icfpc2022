package api

import (
	"encoding/json"
	"fmt"
	"github.com/icfpc-unagi/icfpc2022/go/internal/official"
	"github.com/pkg/errors"
	"net/http"
	"strconv"
)

func init() {
	http.HandleFunc("/"+PATH_PREFIX+"/submit", submitHandler)
}

func submitHandler(w http.ResponseWriter, r *http.Request) {
	resp, err := func() (*official.SubmitResponse, error) {
		problemID, _ := strconv.Atoi(r.PostFormValue("problem_id"))
		isl := r.PostFormValue("isl")

		if problemID == 0 {
			return nil, errors.Errorf("problem ID is invalid.")
		}

		return official.Submit(problemID, []byte(isl))
	}()
	if err != nil {
		buf, _ := json.Marshal(map[string]string{
			"message": fmt.Sprintf("%+v", err),
		})
		fmt.Fprintf(w, "%s", buf)
	} else {
		buf, _ := json.Marshal(resp)
		fmt.Fprintf(w, "%s", buf)
	}
}
