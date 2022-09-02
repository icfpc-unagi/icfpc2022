package handler

import (
	"fmt"
	"net/http"
)

func init() {
	http.HandleFunc("/template", handleTemplate)
}

func handleTemplate(w http.ResponseWriter, r *http.Request) {
	Template(w, []byte("<h1>test</h1>foobar"))
}

func Template(w http.ResponseWriter, body []byte) {
	fmt.Fprint(w,
		`<html lang="ja">
<header>
	<meta charset="UTF-8">
	<meta charset="utf-8">
	<meta name="viewport" content="width=device-width,initial-scale=1.0,user-scalable=yes">
	<link rel="stylesheet" type="text/css" href="/static/style.css">
</header>
<body>
<nav>
<a href="/"></a>
<ul>
<li><a href="/">Standings</a></li>
<li><a href="/scoreboard">Scoreboard</a></li>
</ul>
</nav>
<main>
<article>
`, string(body), `
</article>
</main>
</body>
</html>
`)
}
