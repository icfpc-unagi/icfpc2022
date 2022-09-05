import init, * as icfpc2022 from "../../pkg/icfpc2022.js"
(async () => {
  await init();
  (window as any).icfpc2022 = icfpc2022

  const base1 = 'https://icfpc.sx9.jp/problems/'
  const base0 = 'https://cdn.robovinci.xyz/imageframes/'
  const problem_id1 = document.getElementById('problem_id1') as HTMLInputElement
  const isl1 = document.getElementById('isl1') as HTMLTextAreaElement
  const container1 = document.getElementById('container1') as HTMLDivElement
  const canvas1 = document.getElementById('canvas1') as HTMLCanvasElement
  const canvas2 = document.getElementById('canvas2') as HTMLCanvasElement
  const svg1 = document.getElementById('svg1') as unknown as SVGSVGElement
  const cost1 = document.getElementById('cost1') as HTMLSpanElement
  const similarity1 = document.getElementById('similarity1') as HTMLSpanElement
  const error1 = document.getElementById('error1') as HTMLDivElement
  const score1 = document.getElementById('score1') as HTMLSpanElement
  const initial_config = document.getElementById('initial_config') as HTMLTextAreaElement
  const initial_png = document.getElementById('initial_png') as HTMLImageElement
  const target_png = document.getElementById('target_png') as HTMLImageElement

  // wasm 側に持たせているもの
  // TODO: target_png, initial_config, initial_png を渡す
  let managed: icfpc2022.ManagedCanvas = new icfpc2022.ManagedCanvas(canvas1)

  // 新しい問題の読み込み
  async function loadProblem(problemId: string | number) {
    let resp = await fetch(`${base1}${problemId}.json`)
    let json = await resp.json()
    if (json.canvas_link && json.canvas_link.trim()) { // can be " "
      initial_png.style.display = 'default'
      fetch(json.canvas_link.replace(base0, base1))
        .then(resp => resp.blob())
        .then(blob => initial_png.src = URL.createObjectURL(blob))
    } else {
      initial_png.style.display = 'none'
    }
    if (json.initial_config_link && json.initial_config_link.trim()) {
      console.log(json.initial_config_link)
      fetch(json.initial_config_link.replace(base0, base1))
        .then(resp => resp.text())
        .then(text => initial_config.value = text)
    }
    if (json.target_link) {
      fetch(json.target_link.replace(base0, base1))
        .then(resp => resp.blob())
        .then(blob => target_png.src = URL.createObjectURL(blob))
    }
  }
  problem_id1.addEventListener('input', e => loadProblem((e.target as HTMLInputElement).value))
  loadProblem(problem_id1.value)

  // Canvas に対する操作
  const pointerState = {
    drawPointer: false,
    x: 0,
    y: 0,
  }
  function drawInteractions(shift: boolean, ctrl: boolean) {
    let ctx = canvas2.getContext('2d') as CanvasRenderingContext2D
    ctx.clearRect(0, 0, 400, 400)
    if (pointerState.drawPointer) {
      ctx.beginPath()
      if (!ctrl) {
        ctx.moveTo(0, pointerState.y)
        ctx.lineTo(400, pointerState.y)
      }
      if (!shift) {
        ctx.moveTo(pointerState.x, 0)
        ctx.lineTo(pointerState.x, 400)
      }
      ctx.lineWidth = 2
      ctx.strokeStyle = '#ff0e'
      ctx.stroke()
    }
  }
  function pointerEventHandler(e: PointerEvent) {
    let shift = e.getModifierState('Shift')
    let ctrl = e.getModifierState('Control') || e.getModifierState('Meta')
    pointerState.drawPointer = false
    pointerState.x = e.offsetX
    pointerState.y = e.offsetY
    switch (e.type) {
      case 'pointermove':
      case 'pointerenter':
        pointerState.drawPointer = true
        break
      case 'pointerleave':
        pointerState.drawPointer = false
        break;
      default:
        return
    }
    drawInteractions(shift, ctrl)
  }
  function keyEventHandler(e: KeyboardEvent) {
    let shift = e.getModifierState('Shift')
    let ctrl = e.getModifierState('Control') || e.getModifierState('Meta')
    drawInteractions(shift, ctrl)
  }
  container1.addEventListener('pointerenter', pointerEventHandler)
  container1.addEventListener('pointerleave', pointerEventHandler)
  container1.addEventListener('pointermove', pointerEventHandler)
  container1.addEventListener('keydown', keyEventHandler)
  container1.addEventListener('keyup', keyEventHandler)
  container1.addEventListener('click', e => {
    let x = e.offsetX;
    let y = 400 - e.offsetY;
    let rect = findRect(e.target as Element)
    let block_id = rect?.getAttribute('block-id') ?? '0'
    let shift = e.getModifierState('Shift')
    let ctrl = e.getModifierState('Control') || e.getModifierState('Meta')
    if (shift && ctrl) {
      let color = '0,0,0,255'
      // たぶんバグっている。正しい値が得られない。
      // if (rect) {
      //   let bb = rect.getBBox()
      //   console.debug(bb)
      //   let median = managed.geometric_median_4d_on_rect(bb.x, 400 - bb.y - bb.height, bb.x + bb.width, 400 - bb.y)
      //   console.debug(median)
      //   color = median.join(',')
      // }
      append(`color [${block_id}.0] [${color}]`)
    } else if (shift) {
      append(`cut [${block_id}] [y] [${y}]`)
    } else if (ctrl) {
      append(`cut [${block_id}] [x] [${x}]`)
    } else {
      append(`cut [${block_id}] [${x},${y}]`)
    }

    // 操作が難しそうだからとりあえず手入力で
    // append(`swap [${block_id}] [${block_id2}]`)
    // append(`merge [${block_id}] [${block_id2}]`)
    update()
  })

  // Update Canvas
  function update() {
    try {
      managed.clear()
      managed.apply(isl1.value)
      let svgDoc = managed.svg()
      let holder = document.createElement('div')
      holder.innerHTML = svgDoc
      svg1.innerHTML = holder.firstElementChild!.innerHTML
      error1.innerText = ''
      cost1.innerText = managed.cost().toString()
      // TODO: similarity
    } catch (e) {
      console.error(e)
      error1.innerText = e.toString()
      managed = new icfpc2022.ManagedCanvas(canvas1);
    }
  }
  isl1.addEventListener('input', _ => update())
  update()

  // Load best submission
  async function loadSubmission(problemId: string) {
    let resp = await fetch(`https://icfpc.sx9.jp/scvzcaae/submission?problem_id=${problemId}`)
    let json = await resp.json()
    isl1.value = json.SubmissionSolution
    update()
  }
  problem_id1.form?.addEventListener('submit', e => {
    e.preventDefault()
    loadSubmission(problem_id1.value)
  });

  function append(s: string) {
    isl1.value += (isl1.value && !isl1.value.endsWith('\n') ? '\n' : '') + s + '\n'
  }

  function findRect(e: Element | null): SVGRectElement | null {
    while (e) {
      if (e.tagName == 'rect') return e as SVGRectElement
      e = e.parentElement
    }
    return null
  }
})()
