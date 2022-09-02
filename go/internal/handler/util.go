package handler

import (
	"fmt"
	"strings"
	"time"
)

func ParseTimestamp(t string) int64 {
	t = strings.Split(t, ".")[0]
	t = strings.TrimSuffix(t, "Z")
	x, _ := time.Parse("2006-01-02T15:04:05", t)
	start, _ := time.Parse(
		"2006-01-02T15:04:05", "2022-09-02T12:00:00")
	return x.Unix() - start.Unix()
}

func ToElapsedTime(t string) string {
	e := ParseTimestamp(t)
	return fmt.Sprintf("%02d:%02d", e/3600, e/60%60)
}
