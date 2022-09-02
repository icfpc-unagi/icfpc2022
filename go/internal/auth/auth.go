package auth

import (
	"crypto/sha256"
	"crypto/subtle"
	"net/http"
	"os"
)

func BasicAuth(next http.HandlerFunc) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_, password, ok := r.BasicAuth()
		if ok {
			actual := sha256.Sum256([]byte(password))
			expected := sha256.Sum256([]byte(os.Getenv("UNAGI_PASSWORD")))
			if subtle.ConstantTimeCompare(actual[:], expected[:]) == 1 {
				next.ServeHTTP(w, r)
				return
			}
		}
		w.Header().Set(
			"WWW-Authenticate", `Basic realm="restricted", charset="UTF-8"`)
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
	})
}
