package util

import (
	"time"

	"github.com/pkg/errors"
)

func StrToTime(s string) (time.Time, error) {
	for _, f := range []string{
		"2006-01-02T15:04:05",
		"2006-01-02 15:04:05",
	} {
		if t, err := time.Parse(f, s[:len(f)]); err == nil {
			return t, nil
		}
	}
	return time.Time{}, errors.Errorf("incomprehensive time format: %s", s)
}
