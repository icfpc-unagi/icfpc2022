package api

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"
	"strings"

	"github.com/pkg/errors"
)

func init() {
	http.HandleFunc("/"+PATH_PREFIX+"/internal_submit", internalSubmitHandler)
}

func internalSubmitHandler(w http.ResponseWriter, r *http.Request) {
	resp, err := func() (*RunAddResponse, error) {
		problemID, _ := strconv.Atoi(r.PostFormValue("problem_id"))
		if problemID == 0 {
			return nil, errors.Errorf("problem ID is invalid.")
		}

		isl := r.PostFormValue("isl")
		name := strings.SplitN(isl, "\n", 2)[0]
		if !strings.HasPrefix(name, "#") {
			name = ""
		} else {
			name = strings.TrimPrefix(name, "#")
		}
		name = strings.SplitN(strings.TrimSpace(name), " ", 2)[0]

		return RunAdd(context.Background(), &RunAddRequest{
			ProblemID:   problemID,
			SolutionISL: isl,
			RunName:     name,
		})
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
