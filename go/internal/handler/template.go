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
	<script src="https://ajax.googleapis.com/ajax/libs/jquery/3.3.1/jquery.min.js"></script>
	<script src="/static/jquery-linedtextarea.js"></script>
	<link href="/static/jquery-linedtextarea.css" rel="stylesheet"/>
</header>
<body>
<nav>
<a href="/"></a>
<ul>
<li><a href="/scoreboard">順位表</a></li>
<li><a href="/problems">問題</a></li>
<li><a href="/programs">ソルバー</a></li>
<li><a href="/submit">Submit</a></li>
<li><a href="/visualizer">可視化</a></li>
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
