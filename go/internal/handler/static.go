package handler

import (
	"net/http"
	"strings"
)

func init() {
	staticDir := http.StripPrefix("/static/", http.FileServer(http.Dir("/work/static")))
	http.HandleFunc("/static/", func(rw http.ResponseWriter, r *http.Request) {
		if r.Method == "GET" && strings.HasPrefix(r.URL.Path, "/static/") {
			staticDir.ServeHTTP(rw, r)
		} else {
			http.NotFound(rw, r)
		}
	})

	problemsDir := http.StripPrefix("/problems/", http.FileServer(http.Dir("/work/problems")))
	http.HandleFunc("/problems/", func(rw http.ResponseWriter, r *http.Request) {
		if r.Method == "GET" && strings.HasPrefix(r.URL.Path, "/problems/") {
			problemsDir.ServeHTTP(rw, r)
		} else {
			http.NotFound(rw, r)
		}
	})
}
