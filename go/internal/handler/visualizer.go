package handler

import (
	"bytes"
	"fmt"
	"net/http"
	"strings"

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

	fmt.Fprintf(buf, "<h1>ビジュアライザ</h1>")

	fmt.Fprintf(buf, `
    <p>
      <label>
        Problem ID:
        <input type="number" id="problem_id" value="1" min="1" max="18446744073709551615" onchange="updateOutput()"/>
      </label>
    </p>
    <p>
      <label>
        Output:<br>
        <textarea id="output" rows="4" data-gramm_editor="false" onchange="updateOutput()"></textarea>
      </label>
    </p>
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
      <input type="checkbox" id="show_blocks" onchange="visualize()">show blocks&emsp;
      <input type="checkbox" id="show_diff" onchange="visualize()">show diff
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
      const { gen, vis, get_max_turn } = wasm_bindgen;
      async function run() {
        await wasm_bindgen('./web_bg.wasm');
        visualize();
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
    </script>
`)
}
