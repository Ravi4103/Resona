import { onMounted, onUnmounted, watch, type Ref } from 'vue'

// Simplified WebGL-based Gaussian blur for the miniplayer glass effect
// Uses two-pass separable Gaussian blur (horizontal then vertical)

const VERT_SRC = `
attribute vec2 a_position;
attribute vec2 a_uv;
varying vec2 vUv;

void main() {
  vUv = a_uv;
  gl_Position = vec4(a_position, 0, 1);
}
`

// Horizontal Gaussian blur with Y-ratio remapping to artwork bottom
const H_FRAG_SRC = `
precision mediump float;
uniform sampler2D uTex;
uniform float uStep;
uniform float uRatio;
varying vec2 vUv;

void main() {
  float y = vUv.y * uRatio;
  vec2 uv = vec2(vUv.x, y);
  
  vec4 c = vec4(0.0);
  c += texture2D(uTex, uv + vec2(-4.0*uStep, 0.0)) * 0.0162;
  c += texture2D(uTex, uv + vec2(-3.0*uStep, 0.0)) * 0.0540;
  c += texture2D(uTex, uv + vec2(-2.0*uStep, 0.0)) * 0.1216;
  c += texture2D(uTex, uv + vec2(-1.0*uStep, 0.0)) * 0.1945;
  c += texture2D(uTex, uv) * 0.2270;
  c += texture2D(uTex, uv + vec2(1.0*uStep, 0.0)) * 0.1945;
  c += texture2D(uTex, uv + vec2(2.0*uStep, 0.0)) * 0.1216;
  c += texture2D(uTex, uv + vec2(3.0*uStep, 0.0)) * 0.0540;
  c += texture2D(uTex, uv + vec2(4.0*uStep, 0.0)) * 0.0162;
  
  gl_FragColor = c;
}
`

// Vertical Gaussian blur with brightness adjustment and alpha gradient fade
const V_FRAG_SRC = `
precision mediump float;
uniform sampler2D uTex;
uniform float uStep;
uniform float uBrightness;
varying vec2 vUv;

void main() {
  vec4 c = vec4(0.0);
  c += texture2D(uTex, vUv + vec2(0.0, -4.0*uStep)) * 0.0162;
  c += texture2D(uTex, vUv + vec2(0.0, -3.0*uStep)) * 0.0540;
  c += texture2D(uTex, vUv + vec2(0.0, -2.0*uStep)) * 0.1216;
  c += texture2D(uTex, vUv + vec2(0.0, -1.0*uStep)) * 0.1945;
  c += texture2D(uTex, vUv) * 0.2270;
  c += texture2D(uTex, vUv + vec2(0.0, 1.0*uStep)) * 0.1945;
  c += texture2D(uTex, vUv + vec2(0.0, 2.0*uStep)) * 0.1216;
  c += texture2D(uTex, vUv + vec2(0.0, 3.0*uStep)) * 0.0540;
  c += texture2D(uTex, vUv + vec2(0.0, 4.0*uStep)) * 0.0162;
  
  c.rgb *= uBrightness;
  c.rgb -= (1.0 - smoothstep(0.0, 0.5, vUv.y)) * 0.25;
  c.rgb = clamp(c.rgb, 0.0, 1.0);
  c.a = 1.0 - smoothstep(0.5, 0.9, vUv.y);
  
  gl_FragColor = c;
}
`

// Quad vertex data: position (x, y) + UV (u, v)
const VERT_DATA = new Float32Array([
  -1, -1, 0, 0,
   1, -1, 1, 0,
  -1,  1, 0, 1,
   1,  1, 1, 1,
])

function compileShader(gl: WebGLRenderingContext, type: number, src: string): WebGLShader | null {
  const s = gl.createShader(type)
  if (!s) return null
  gl.shaderSource(s, src)
  gl.compileShader(s)
  if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) {
    console.error('[useGlassBlur] shader compile error:', gl.getShaderInfoLog(s))
    gl.deleteShader(s)
    return null
  }
  return s
}

function createProgram(gl: WebGLRenderingContext, vsSrc: string, fsSrc: string): WebGLProgram | null {
  const vs = compileShader(gl, gl.VERTEX_SHADER, vsSrc)
  const fs = compileShader(gl, gl.FRAGMENT_SHADER, fsSrc)
  if (!vs || !fs) return null
  const p = gl.createProgram()
  if (!p) return null
  gl.attachShader(p, vs)
  gl.attachShader(p, fs)
  gl.linkProgram(p)
  if (!gl.getProgramParameter(p, gl.LINK_STATUS)) {
    console.error('[useGlassBlur] program link error:', gl.getProgramInfoLog(p))
    return null
  }
  return p
}

const MAX_TEX_SIZE = 256

export function useGlassBlur(
  canvasRef: Ref<HTMLCanvasElement | null>,
  imageUrl: Ref<string | null>,
  panelHeight = 160,
) {
  let gl: WebGLRenderingContext | null = null
  let hProgram: WebGLProgram | null = null
  let vProgram: WebGLProgram | null = null
  let fbo: WebGLFramebuffer | null = null
  let fboTex: WebGLTexture | null = null
  let artTex: WebGLTexture | null = null
  let vbo: WebGLBuffer | null = null
  let pendingImg: HTMLImageElement | null = null

  function getUniformLoc(p: WebGLProgram, name: string): WebGLUniformLocation | null {
    return gl!.getUniformLocation(p, name)
  }

  function drawPass(program: WebGLProgram, uniforms: Record<string, number | WebGLTexture>, targetFbo?: WebGLFramebuffer | null) {
    if (!gl || !vbo) return
    gl.useProgram(program)
    gl.bindFramebuffer(gl.FRAMEBUFFER, targetFbo ?? null)
    for (const [name, val] of Object.entries(uniforms)) {
      const loc = getUniformLoc(program, name)
      if (loc === null) continue
      if (name === 'uTex') {
        gl.activeTexture(gl.TEXTURE0)
        gl.bindTexture(gl.TEXTURE_2D, val as WebGLTexture)
        gl.uniform1i(loc, 0)
      } else {
        gl.uniform1f(loc, val as number)
      }
    }
    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4)
  }

  function render() {
    if (!gl || !hProgram || !vProgram || !fbo || !artTex) return
    gl.viewport(0, 0, gl.canvas.width, gl.canvas.height)
    gl.clear(gl.COLOR_BUFFER_BIT)
    drawPass(hProgram, { uTex: artTex, uStep: 10 / gl.canvas.width, uRatio: panelHeight / window.innerHeight }, fbo)
    drawPass(vProgram, { uTex: fboTex!, uStep: 10 / panelHeight, uBrightness: 0.6 })
  }

  function init(canvas: HTMLCanvasElement) {
    const w = canvas.clientWidth || 300
    canvas.width = w
    canvas.height = panelHeight

    gl = canvas.getContext('webgl', { alpha: true, premultipliedAlpha: false })
    if (!gl) { console.warn('[useGlassBlur] WebGL not supported'); return }
    gl.clearColor(0, 0, 0, 0)
    gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, true)

    // Shader programs
    hProgram = createProgram(gl, VERT_SRC, H_FRAG_SRC)
    vProgram = createProgram(gl, VERT_SRC, V_FRAG_SRC)
    if (!hProgram || !vProgram) return

    // Vertex buffer (full-screen quad)
    vbo = gl.createBuffer()
    gl.bindBuffer(gl.ARRAY_BUFFER, vbo)
    gl.bufferData(gl.ARRAY_BUFFER, VERT_DATA, gl.STATIC_DRAW)

    // Setup attribute pointers for both programs
    const stride = 16 // 4 floats * 4 bytes
    for (const prog of [hProgram, vProgram]) {
      gl.useProgram(prog)
      const posLoc = gl.getAttribLocation(prog, 'a_position')
      const uvLoc = gl.getAttribLocation(prog, 'a_uv')
      if (posLoc >= 0) { gl.enableVertexAttribArray(posLoc); gl.vertexAttribPointer(posLoc, 2, gl.FLOAT, false, stride, 0) }
      if (uvLoc >= 0) { gl.enableVertexAttribArray(uvLoc); gl.vertexAttribPointer(uvLoc, 2, gl.FLOAT, false, stride, 8) }
    }

    // Artwork texture
    artTex = gl.createTexture()
    gl.bindTexture(gl.TEXTURE_2D, artTex)
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE)
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE)
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR)
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR)

    // FBO texture (render target)
    fboTex = gl.createTexture()
    gl.bindTexture(gl.TEXTURE_2D, fboTex)
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, w, panelHeight, 0, gl.RGBA, gl.UNSIGNED_BYTE, null)
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE)
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE)
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR)
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR)

    // Framebuffer
    fbo = gl.createFramebuffer()
    gl.bindFramebuffer(gl.FRAMEBUFFER, fbo)
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, fboTex, 0)
    gl.bindFramebuffer(gl.FRAMEBUFFER, null)
  }

  function loadArtwork(url: string) {
    if (pendingImg) {
      pendingImg.onload = null
      pendingImg.onerror = null
      pendingImg.src = ''
      pendingImg = null
    }

    const img = new Image()
    pendingImg = img
    img.crossOrigin = 'anonymous'
    img.onload = () => {
      pendingImg = null
      if (!gl || !artTex) return
      let source: TexImageSource = img
      if (img.naturalWidth > MAX_TEX_SIZE || img.naturalHeight > MAX_TEX_SIZE) {
        const scale = MAX_TEX_SIZE / Math.max(img.naturalWidth, img.naturalHeight)
        const w = Math.round(img.naturalWidth * scale)
        const h = Math.round(img.naturalHeight * scale)
        const offscreen = document.createElement('canvas')
        offscreen.width = w
        offscreen.height = h
        offscreen.getContext('2d')!.drawImage(img, 0, 0, w, h)
        source = offscreen
      }
      gl.bindTexture(gl.TEXTURE_2D, artTex)
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, source)
      render()
    }
    img.onerror = () => { pendingImg = null }
    img.src = url
  }

  onMounted(() => {
    const canvas = canvasRef.value
    if (!canvas) return
    init(canvas)
    if (imageUrl.value) loadArtwork(imageUrl.value)
  })

  watch(imageUrl, (url) => {
    if (!url) return
    if (!gl && canvasRef.value) init(canvasRef.value)
    loadArtwork(url)
  })

  onUnmounted(() => {
    if (pendingImg) {
      pendingImg.onload = null
      pendingImg.onerror = null
      pendingImg.src = ''
      pendingImg = null
    }
    if (gl) {
      const ext = gl.getExtension('WEBGL_lose_context')
      if (ext) ext.loseContext()
    }
    gl = null
    hProgram = null
    vProgram = null
    fbo = null
    fboTex = null
    artTex = null
    vbo = null
  })
}