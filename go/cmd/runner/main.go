package main

import (
	"context"
	"crypto/sha1"
	"flag"
	"fmt"
	"io"
	"io/ioutil"
	"os"
	"os/exec"
	"path"
	"strings"
	"time"

	"github.com/google/uuid"

	"github.com/icfpc-unagi/icfpc2022/go/internal/api"

	"github.com/golang/glog"
	"github.com/icfpc-unagi/icfpc2022/go/internal/client"
)

const CONTAINER_IMAGE = "asia-docker.pkg.dev/icfpc-primary/asia/runner"

var pipelineID = flag.Int("pipeline", 0, "Pipeline ID")

func main() {
	flag.Parse()
	glog.Info("Pulling a container image...")
	cmd := exec.Command("docker", "pull", CONTAINER_IMAGE)
	cmd.Stdout, cmd.Stderr = os.Stdout, os.Stderr
	if err := cmd.Run(); err != nil {
		glog.Fatalf("Failed to pull a container image from: %s: %+v",
			CONTAINER_IMAGE, err)
	}

	glog.Info("Started")
	for !Exists("/shutdown") {
		if err := Loop(); err != nil {
			glog.Errorf("ERROR: %+v", err)
		}
		time.Sleep(time.Second * 10)
	}
}

func Loop() error {
	ctx := context.Background()

	glog.Info("Acquiring a new run...")
	resp, err := client.RunAcquire(ctx, *pipelineID)
	if err != nil {
		return err
	}
	if resp.RunID == 0 {
		glog.Info("No runs acquired")
		return nil
	}
	glog.Infof("Acquired a run: run_id=%d", resp.RunID)
	defer glog.Infof("Closing a run: run_id=%d", resp.RunID)

	dir, err := os.MkdirTemp(os.TempDir(), "executor")
	hash := sha1.Sum([]byte(fmt.Sprintf("%d@%d@", os.Getpid(), time.Now().UnixNano())))
	name := fmt.Sprintf("r%x", hash[:4])
	glog.Infof("Running command: %s", resp.RunCommand)
	cmd := exec.CommandContext(ctx,
		"docker", "run", "--rm", "--name", name,
		CONTAINER_IMAGE, "bash", "-c",
		strings.ReplaceAll(resp.RunCommand, "\r", ""))
	cmd.Dir = dir
	stdout, err := os.Create(path.Join(dir, "stdout"))
	cmd.Stdout = stdout
	stderr, err := os.Create(path.Join(dir, "stderr"))
	cmd.Stderr = io.MultiWriter(stderr, os.Stderr)
	cmd.Start()

	glog.Infof("Start running run_id=%d...", resp.RunID)

	c := make(chan struct{})
	go func() {
		count := 0
		for {
			select {
			case _, ok := <-c:
				if !ok {
					return
				}
			case <-time.After(time.Second * 10):
				if err := client.RunExtend(ctx, resp.RunSignature); err == nil {
					count = 0
					exec.Command(
						"docker", "exec", name,
						"touch", "/watchdog").Run()
				} else {
					count += 1
					if count > 5 {
						return
					}
				}
			}
		}
	}()

	cmd.Wait()
	close(c)
	stdout.Close()
	stderr.Close()

	glog.Infof("A process stopped: run_id=%d", resp.RunID)

	exitCode := cmd.ProcessState.ExitCode()
	logID := uuid.New().String()
	result := api.RunFlushRequest{
		RunSignature: resp.RunSignature,
		RunCode:      int64(exitCode),
		SolutionISL:  Summary(path.Join(dir, "stdout")),
		LogID:        logID,
	}

	glog.Infof("Exporting log files: %s", logID)
	cmd = exec.CommandContext(ctx,
		"gsutil", "-m", "cp", "-Z",
		path.Join(dir, "stdout"), path.Join(dir, "stderr"),
		"gs://icfpc2022/log/"+logID+"/")
	cmd.Dir = dir
	cmd.Run()

	glog.Infof("Flushing a run: run_id=%d", resp.RunID)
	return client.RunFlush(ctx, &result)
}

func Summary(file string) string {
	buf, _ := ioutil.ReadFile(file)
	if len(buf) <= 200000 {
		return string(buf)
	}
	b := []byte{}
	b = append(b, buf[:100000]...)
	b = append(b, []byte("...")...)
	b = append(b, buf[len(buf)-100000:]...)
	return string(b)
}

func Exists(name string) bool {
	_, err := os.Stat(name)
	return !os.IsNotExist(err)
}
