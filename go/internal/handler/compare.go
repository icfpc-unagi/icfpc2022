package handler

import (
	"bytes"
	"fmt"
	"net/http"
)

func init() {
	http.HandleFunc("/compare", compareTemplate)
}

func compareTemplate(w http.ResponseWriter, r *http.Request) {
	buf := &bytes.Buffer{}
	defer func() { Template(w, buf.Bytes()) }()

	fmt.Fprintf(buf, "<h1>比較表</h1><table><tr><td>問題番号</td><td>自分のベストスコア</td><td>他チームのベストスコア</td></tr>")
	records := getAllRecords(buf)
	for _, p := range Problems() {
		myBest := int64(100000000)
		othersBest := int64(100000000)
		for _, r := range records {
			isMine := r.IsInternal || r.UserName == "Unagi"
			if isMine && r.Score < myBest {
				myBest = r.Score
			} else if !isMine && r.Score < othersBest {
				othersBest = r.Score
			}
		}
		fmt.Fprintf(buf, "<tr><td>%d</td><td>%d</td><td>%d</td></table>", p.ID, myBest, othersBest)
	}
}
