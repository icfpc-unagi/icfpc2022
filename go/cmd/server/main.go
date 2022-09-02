package main

import (
	"context"
	"flag"
	"fmt"
	"github.com/golang/glog"
	"net/http"

	"github.com/icfpc-unagi/icfpc2022/go/pkg/db"
)

var port = flag.String("port", ":8080", "API endpoint")

func handler(w http.ResponseWriter, r *http.Request) {
	glog.Info("Processing request...")
	var output int
	db.Cell(context.Background(), &output, "SELECT 1 + 1")
	glog.Infof("Output: %d", output)
	fmt.Fprintf(w, "Output: %d", output)
	if r.Body == nil {
		glog.Errorf("body is empty")
		w.WriteHeader(400)
		return
	}
	defer r.Body.Close()
}

func main() {
	flag.Parse()
	glog.Info("Initializing...")
	http.HandleFunc("/d/sql", handler)
	glog.Infof("Starting server on %s...", *port)
	if err := http.ListenAndServe(*port, nil); err != nil {
		glog.Fatal(err.Error())
	}
}
