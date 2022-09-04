package handler

import (
	"bytes"
	"fmt"
	"html"
	"net/http"
	"strconv"
	"strings"

	"github.com/icfpc-unagi/icfpc2022/go/internal/api"

	"github.com/icfpc-unagi/icfpc2022/go/internal/util"

	"github.com/icfpc-unagi/icfpc2022/go/internal/auth"
)

func init() {
	webDir := http.StripPrefix("/visualizer/", http.FileServer(http.Dir("/work/web/")))
	http.HandleFunc("/visualizer/", auth.BasicAuth(func(w http.ResponseWriter, r *http.Request) {
		if r.Method == "GET" && strings.HasPrefix(r.URL.Path, "/visualizer/") {
			if r.URL.Path == "/visualizer/" {
				visualizerHandler(w, r)
				return
			}
			webDir.ServeHTTP(w, r)
		} else {
			http.NotFound(w, r)
		}
	}))
}

func visualizerHandler(w http.ResponseWriter, r *http.Request) {
	buf := &bytes.Buffer{}
	defer func() { Template(w, buf.Bytes()) }()

	params := struct {
		ProblemID int
		Solution  string
	}{}

	fmt.Fprintf(buf, "<h1>ビジュアライザ</h1>")

	if id, _ := strconv.Atoi(r.URL.Query().Get("run_id")); id != 0 {
		resp, err := api.ExportRun(id)
		if err != nil {
			fmt.Fprintf(buf, `<pre class="alert-danger">%s</pre>`,
				html.EscapeString(fmt.Sprintf("%+v", err)))
		} else {
			params.ProblemID = resp.ProblemID
			params.Solution = resp.ISL
		}
	}

	fmt.Fprintf(buf, `<div><select name="problem_id" id="problem_id" onchange="updateOutput()" style="margin-left: 0">`)
	for _, p := range util.Problems() {
		selected := ""
		if params.ProblemID == p.ID {
			selected = " selected"
		}
		fmt.Fprintf(buf, `<option value="%d"%s>問題 %d: %s</option>`,
			p.ID, selected, p.ID, p.Name)
	}
	fmt.Fprintf(buf, `</select></div>`)
	fmt.Fprintf(buf, `
      <div>
        <textarea class="lined" id="output" rows="20" data-gramm_editor="false" onchange="updateOutput()" placeholder="ISL コード">%s</textarea>
      </div>
    <p style="display:flex;">
      <input type="button" id="play" value="▶" style="width:32px;height:32px;bottom:5px;position:relative;">&ensp;
      <label>
        slow
        <input type="range" id="speed" min="1" max="60" value="30" style="width:200px;">
        fast&emsp;
      </label>
      <label>
        turn:
        <input type="number" id="turn" value="0" min="0" max="0" style="width:70px;text-align:right;" onchange="update_t(this.value)"/>
      </label>
    </p>
    <p>
      <input type="checkbox" id="show_blocks" onchange="visualize()"><label for="show_blocks">ブロック境界表示</label>&emsp;
      <input type="checkbox" id="show_diff" onchange="visualize()"><label for="show_diff">画像差分表示</label>
    </p>
    <p>
      <input type="range" id="t_bar" min="0" max="0" value="0" style="width:780px;" onchange="update_t(this.value)" oninput="update_t(this.value)">
    </p>
    <hr>
    <p id="score"></p>
    <div id="result">
    </div>
    <script src='/web/web.js'></script>
    <script>
textarea_options = { selectedLine: -1 };
$(function() {
  $(".lined").linedtextarea(textarea_options);
});

      const { gen, vis, get_max_turn } = wasm_bindgen;
      async function run() {
        await wasm_bindgen('./web_bg.wasm');
        visualize();
		updateOutput();
      }
      run();

      function visualize() {
        const problem_id = document.getElementById("problem_id").value;
        const output = document.getElementById("output").value;
        const show_blocks = document.getElementById("show_blocks").checked;
        const show_diff = document.getElementById("show_diff").checked;
        const t = document.getElementById("turn").value;
        try {
          const ret = vis(problem_id, output, t, show_blocks, show_diff);
          document.getElementById("score").innerHTML = "Score = " + ret.score;
          if (ret.error != "") {
            document.getElementById("score").innerHTML += " <font color='red'>(" + ret.error + ")</font>";
          }
          document.getElementById("result").innerHTML = ret.svg;
        } catch (error) {
          document.getElementById("result").innerHTML = "<p>Invalid</p>";
        }
      }

      function update_t(t) {
        const max_turn = Number(document.getElementById("turn").max);
        const new_turn = Math.min(Math.max(0, t), max_turn);
        document.getElementById("turn").value = new_turn;
        document.getElementById("t_bar").value = new_turn;
        visualize();
		textarea_options.selectedLine = new_turn;
		$(".lineno").removeClass("lineselect");
		$("#line_" + new_turn).addClass("lineselect");
      }

      var prev = Date.now();
      const play = document.getElementById("play");
      const speed = document.getElementById("speed");

      function start_autoplay() {
        if (Number(document.getElementById("turn").value) >= Number(document.getElementById("turn").max)) {
          document.getElementById("turn").value = 0;
        }
        prev = Date.now();
        play.value = "■";
        update_t(document.getElementById("turn").value);
      }

      function updateOutput() {
        play.value = "▶";
        const output = document.getElementById("output").value;
        try {
          const t = get_max_turn(output);
          document.getElementById("turn").max = t;
          document.getElementById("t_bar").max = t;
          update_t(t);
        } catch (error) {
          document.getElementById("result").innerHTML = "<p>Invalid</p>";
        }
      }

      play.onclick = event => {
        if (play.value == "■") {
          play.value = "▶";
        } else {
          start_autoplay();
        }
      }

      function autoplay() {
        if (play.value == "■") {
          const now = Date.now();
          let s = 2000;
          if ((now - prev) * speed.value >= s) {
            const inc = Math.floor((now - prev) * speed.value / s);
            prev += Math.floor(inc * s / speed.value);
            update_t(Number(document.getElementById("turn").value) + inc);
            if (Number(document.getElementById("turn").value) >= Number(document.getElementById("turn").max)) {
              play.value = "▶";
            }
          }
        }
        requestAnimationFrame(autoplay);
      }
      autoplay();

      $(updateOutput);
    </script>
`, html.EscapeString(params.Solution))
}
