package handler

import (
	"net/http"
	"strings"
)

func init() {
	fileServer := http.StripPrefix("/static/", http.FileServer(http.Dir("/work/static")))
	http.HandleFunc("/static/", func(rw http.ResponseWriter, r *http.Request) {
		if r.Method == "GET" && strings.HasPrefix(r.URL.Path, "/static/") {
			fileServer.ServeHTTP(rw, r)
		} else {
			http.NotFound(rw, r)
		}
	})
}
