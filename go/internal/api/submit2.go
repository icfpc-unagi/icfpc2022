package api

type SubmitRequest struct {
	ProblemID int    `json:"problem_id"`
	Name      string `json:"name"`
	ISL       []byte `json:"isl"`
}

type SubmitResponse struct {
	TaskID int `json:"task_id"`
}

func Submit(req *SubmitRequest) (*SubmitRequest, error) {
	return nil, nil
}
