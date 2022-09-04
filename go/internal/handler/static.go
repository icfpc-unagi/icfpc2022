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
			rw.Header().Set("Access-Control-Allow-Origin", "*")
			problemsDir.ServeHTTP(rw, r)
		} else {
			http.NotFound(rw, r)
		}
	})

	webDir := http.StripPrefix("/web/", http.FileServer(http.Dir("/work/web")))
	http.HandleFunc("/web/", func(rw http.ResponseWriter, r *http.Request) {
		if r.Method == "GET" && strings.HasPrefix(r.URL.Path, "/web/") {
			webDir.ServeHTTP(rw, r)
		} else {
			http.NotFound(rw, r)
		}
	})
}
