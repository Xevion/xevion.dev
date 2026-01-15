<script lang="ts" module>
  // UniformManager at module scope - shared across all component instances
  class UniformManager {
    private gl: WebGL2RenderingContext;
    private locations = new Map<string, WebGLUniformLocation>();

    constructor(
      gl: WebGL2RenderingContext,
      program: WebGLProgram,
      uniforms: string[],
    ) {
      this.gl = gl;
      uniforms.forEach((name) => {
        const loc = gl.getUniformLocation(program, name);
        if (loc) this.locations.set(name, loc);
      });
    }

    set(
      name: string,
      value: number | [number, number] | [number, number, number],
    ) {
      const loc = this.locations.get(name);
      if (!loc) return;

      if (Array.isArray(value)) {
        if (value.length === 2) {
          this.gl.uniform2f(loc, value[0], value[1]);
        } else if (value.length === 3) {
          this.gl.uniform3f(loc, value[0], value[1], value[2]);
        }
      } else {
        this.gl.uniform1f(loc, value);
      }
    }

    set1i(name: string, value: number) {
      const loc = this.locations.get(name);
      if (loc) this.gl.uniform1i(loc, value);
    }

    set1fv(name: string, values: number[]) {
      const loc = this.locations.get(name);
      if (loc) this.gl.uniform1fv(loc, values);
    }

    setVec2(name: string, vec: [number, number]) {
      const loc = this.locations.get(name);
      if (loc) this.gl.uniform2f(loc, vec[0], vec[1]);
    }
  }

  // Shader compilation utilities
  function compileShader(
    gl: WebGL2RenderingContext,
    type: number,
    source: string,
  ): WebGLShader | null {
    const shader = gl.createShader(type);
    if (!shader) return null;

    gl.shaderSource(shader, source);
    gl.compileShader(shader);

    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      console.error("Shader compile error:", gl.getShaderInfoLog(shader));
      gl.deleteShader(shader);
      return null;
    }

    return shader;
  }

  function createProgram(
    gl: WebGL2RenderingContext,
    vertexShader: WebGLShader,
    fragmentShader: WebGLShader,
  ): WebGLProgram | null {
    const program = gl.createProgram();
    if (!program) return null;

    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);
    gl.linkProgram(program);

    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      console.error("Program link error:", gl.getProgramInfoLog(program));
      gl.deleteProgram(program);
      return null;
    }

    return program;
  }

  function generateSeed(): number {
    return Math.floor(Math.random() * 10000);
  }
</script>

<script lang="ts">
  import { cn } from "$lib/utils";
  import type { ClassValue } from "clsx";
  import { onMount, onDestroy } from "svelte";
  import { themeStore } from "$lib/stores/theme.svelte";

  // Type for shader settings that can be theme-overridden
  type ShaderSettings = {
    cellSize: number;
    waveAmplitude: number;
    waveSpeed: number;
    noiseIntensity: number;
    timeSpeed: number;
    radialVignetteIntensity: number;
    radialVignetteRadius: number;
    horizontalVignetteIntensity: number;
    horizontalVignetteRadius: number;
    brightnessAdjust: number;
    contrastAdjust: number;
    thresholds: [number, number, number, number, number];
    opacity: number;
    glyphColor: [number, number, number];
  };

  // Base defaults (shared between themes)
  const baseDefaults: ShaderSettings = {
    cellSize: 10,
    waveAmplitude: 0.25,
    waveSpeed: 0.1,
    noiseIntensity: 0.025,
    timeSpeed: 0.4,
    radialVignetteIntensity: 0.2,
    radialVignetteRadius: 0.1,
    horizontalVignetteIntensity: 0.5,
    horizontalVignetteRadius: 0.7,
    brightnessAdjust: 0.09,
    contrastAdjust: 1.0,
    thresholds: [0.35, 0.4, 0.45, 0.6, 0.75],
    opacity: 0.9,
    glyphColor: [0.502, 0.502, 0.502], // 128/255 pre-computed
  };

  // Dark mode overrides
  const darkModeOverrides: Partial<ShaderSettings> = {
    glyphColor: [0.784, 0.784, 0.784], // 200/255 pre-computed
  };

  // Light mode overrides - more contrast and variation
  const lightModeOverrides: Partial<ShaderSettings> = {
    glyphColor: [55 / 255, 55 / 255, 55 / 255], // 40/255 pre-computed
    brightnessAdjust: 0.15,
    contrastAdjust: 1.2,
    thresholds: [0.4, 0.45, 0.53, 0.65, 0.72],
    opacity: 0.55,
  };

  let {
    class: className = "",
    style = "",
  }: {
    class?: ClassValue;
    style?: string;
  } = $props();

  // Compute effective settings based on theme
  const settings = $derived({
    ...baseDefaults,
    ...(themeStore.isDark ? darkModeOverrides : lightModeOverrides),
  });

  // svelte-ignore non_reactive_update
  let canvas: HTMLCanvasElement;
  let cleanupFns: (() => void)[] = [];
  let ready = $state(false);
  let webglFailed = $state(false);

  function addCleanup(fn: () => void) {
    cleanupFns.push(fn);
  }

  // Vertex shader - simple fullscreen quad
  const vertexShaderSource = `#version 300 es
    in vec2 a_position;
    out vec2 v_uv;

    void main() {
      v_uv = a_position * 0.5 + 0.5;
      gl_Position = vec4(a_position, 0.0, 1.0);
    }
  `;

  // Pass 1: Noise generation with FBM and domain warping
  const noiseFragmentShaderSource = `#version 300 es
    precision highp float;

    in vec2 v_uv;
    out vec4 fragColor;

    uniform float u_time;
    uniform vec2 u_resolution;
    uniform float u_waveAmplitude;
    uniform float u_waveSpeed;
    uniform float u_noiseIntensity;
    uniform float u_radialVignetteIntensity;
    uniform float u_radialVignetteRadius;
    uniform float u_horizontalVignetteIntensity;
    uniform float u_horizontalVignetteRadius;
    uniform float u_brightnessAdjust;
    uniform float u_contrastAdjust;
    uniform float u_noiseSeed;

    // Simplex 3D noise implementation
    // Based on Ashima Arts webgl-noise: https://github.com/ashima/webgl-noise
    vec3 mod289(vec3 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
    vec4 mod289(vec4 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
    // Permutation polynomial: (34x + 1) * x mod 289
    vec4 permute(vec4 x) { return mod289(((x*34.0)+1.0)*x); }
    // Fast inverse square root approximation
    vec4 taylorInvSqrt(vec4 r) { return 1.79284291400159 - 0.85373472095314 * r; }

    float snoise(vec3 v) {
      // Skewing factors for 3D simplex grid
      const vec2 C = vec2(1.0/6.0, 1.0/3.0);
      const vec4 D = vec4(0.0, 0.5, 1.0, 2.0);

      // First corner (skewed simplex cell origin)
      vec3 i  = floor(v + dot(v, C.yyy));
      vec3 x0 = v - i + dot(i, C.xxx);

      // Determine which simplex we're in by comparing coordinates
      vec3 g = step(x0.yzx, x0.xyz);
      vec3 l = 1.0 - g;
      vec3 i1 = min(g.xyz, l.zxy);
      vec3 i2 = max(g.xyz, l.zxy);

      // Offsets for remaining corners
      vec3 x1 = x0 - i1 + C.xxx;
      vec3 x2 = x0 - i2 + C.yyy;
      vec3 x3 = x0 - D.yyy;

      // Permutation for pseudo-random gradient selection
      i = mod289(i);
      vec4 p = permute(permute(permute(
                i.z + vec4(0.0, i1.z, i2.z, 1.0))
              + i.y + vec4(0.0, i1.y, i2.y, 1.0))
              + i.x + vec4(0.0, i1.x, i2.x, 1.0));

      // Gradient calculation using 7x7 grid mapped to sphere surface
      const float ONE_SEVENTH = 1.0 / 7.0;
      vec3 ns = ONE_SEVENTH * D.wyz - D.xzx;

      vec4 j = p - 49.0 * floor(p * ns.z * ns.z);

      vec4 x_ = floor(j * ns.z);
      vec4 y_ = floor(j - 7.0 * x_);

      vec4 x = x_ *ns.x + ns.yyyy;
      vec4 y = y_ *ns.x + ns.yyyy;
      vec4 h = 1.0 - abs(x) - abs(y);

      vec4 b0 = vec4(x.xy, y.xy);
      vec4 b1 = vec4(x.zw, y.zw);

      vec4 s0 = floor(b0)*2.0 + 1.0;
      vec4 s1 = floor(b1)*2.0 + 1.0;
      vec4 sh = -step(h, vec4(0.0));

      vec4 a0 = b0.xzyw + s0.xzyw*sh.xxyy;
      vec4 a1 = b1.xzyw + s1.xzyw*sh.zzww;

      vec3 p0 = vec3(a0.xy, h.x);
      vec3 p1 = vec3(a0.zw, h.y);
      vec3 p2 = vec3(a1.xy, h.z);
      vec3 p3 = vec3(a1.zw, h.w);

      // Normalize gradients
      vec4 norm = taylorInvSqrt(vec4(dot(p0,p0), dot(p1,p1), dot(p2,p2), dot(p3,p3)));
      p0 *= norm.x;
      p1 *= norm.y;
      p2 *= norm.z;
      p3 *= norm.w;

      // Radial falloff from each corner, then sum gradient contributions
      // 42.0 is the normalization factor to map output to approximately [-1, 1]
      vec4 m = max(0.6 - vec4(dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3)), 0.0);
      m = m * m;
      return 42.0 * dot(m*m, vec4(dot(p0,x0), dot(p1,x1), dot(p2,x2), dot(p3,x3)));
    }

    // Fractional Brownian Motion - smooth version
    float fbm(vec3 p) {
      float value = 0.0;
      float amplitude = 0.5;
      float frequency = 1.0;
      for (int i = 0; i < 4; i++) {
        value += amplitude * snoise(p * frequency);
        amplitude *= 0.5;
        frequency *= 2.1; // Slightly irregular lacunarity for more organic feel
      }
      return value;
    }

    // Ridged FBM - creates sharp mountain/cloud ridges
    float fbmRidged(vec3 p) {
      float value = 0.0;
      float amplitude = 0.5;
      float frequency = 1.0;
      float prev = 1.0;
      for (int i = 0; i < 4; i++) {
        float n = 1.0 - abs(snoise(p * frequency));
        n = n * n; // Square for sharper ridges
        value += amplitude * n * prev;
        prev = n;
        amplitude *= 0.5;
        frequency *= 2.1;
      }
      return value;
    }

    // Billowy FBM - puffy cloud-like shapes
    float fbmBillowy(vec3 p) {
      float value = 0.0;
      float amplitude = 0.5;
      float frequency = 1.0;
      for (int i = 0; i < 4; i++) {
        value += amplitude * abs(snoise(p * frequency));
        amplitude *= 0.5;
        frequency *= 2.1;
      }
      return value;
    }

    void main() {
      // Calculate vignette from original UV (before aspect correction) so it stays circular
      vec2 center = vec2(0.5, 0.5);
      float dist = length(v_uv - center);

      // Fix aspect ratio - reveal more pattern instead of stretching
      float aspect = u_resolution.x / u_resolution.y;
      vec2 uv = v_uv;
      uv.x *= aspect;

      // Unified drift - all layers move together for coherent motion
      vec2 drift = u_time * (0.02 + 0.02 * u_waveSpeed) * vec2(0.3, 0.2);
      float warpTime = u_time * max(0.025, 0.04 * u_waveSpeed);

      // IQ-style domain warping with unified drift
      // First layer: q = (fbm(p + drift), fbm(p + drift + offset))
      vec2 q = vec2(
        fbm(vec3(uv + drift, warpTime + u_noiseSeed)),
        fbm(vec3(uv + drift + vec2(5.2, 1.3), warpTime + u_noiseSeed))
      );

      // Second layer: r uses same drift for coherent motion
      vec2 r = vec2(
        fbm(vec3(uv + 4.0 * q + vec2(1.7, 9.2) + drift, warpTime + u_noiseSeed)),
        fbm(vec3(uv + 4.0 * q + vec2(8.3, 2.8) + drift, warpTime + u_noiseSeed))
      );

      // Apply domain warping
      float warpStrength = u_waveAmplitude * 1.5;
      vec2 warpedUV = uv + warpStrength * r + drift;

      // Multi-layer density combining different noise types
      vec3 samplePos = vec3(warpedUV * 3.0, warpTime * 0.5 + u_noiseSeed); // Lower freq = larger shapes

      // Base: smooth FBM for overall cloud mass
      float smoothLayer = fbm(samplePos) * 0.5 + 0.5;

      // Ridged: subtle sharp accents at lower frequency
      float ridgedLayer = fbmRidged(samplePos * 0.5);

      // Billowy: soft puffy texture
      float billowyLayer = fbmBillowy(samplePos * 0.6);

      // Blend: heavily favor smooth base, subtle accents from others
      float density = smoothLayer * 0.75 + ridgedLayer * 0.12 + billowyLayer * 0.13;

      // Subtle edge detail - less aggressive
      float edgeMask = smoothstep(0.25, 0.45, density) * smoothstep(0.75, 0.55, density);
      float detailNoise = snoise(vec3(warpedUV * 10.0, warpTime * 0.4 + u_noiseSeed)) * 0.5 + 0.5;
      density = mix(density, density + (detailNoise - 0.5) * 0.15, edgeMask);

      // Add subtle grain noise
      density += (snoise(vec3(uv * 50.0 + drift * 10.0, u_noiseSeed)) * 0.5 + 0.5) * u_noiseIntensity;

      // Gentle S-curve instead of hard threshold - preserves gradient across full range
      // This keeps the smooth transitions rather than clamping to plateaus
      float visible = smoothstep(0.0, 1.0, density);

      // Radial vignette (uses original UV distance so it stays circular)
      float edgeFade = 1.0 - smoothstep(u_radialVignetteRadius * 0.5, u_radialVignetteRadius, dist) * u_radialVignetteIntensity;
      visible *= edgeFade;

      // Horizontal vignette - fades center, stronger at left/right edges
      float hDist = abs(v_uv.x - 0.5); // 0 at center, 0.5 at edges
      float hFade = mix(1.0 - u_horizontalVignetteIntensity, 1.0, smoothstep(u_horizontalVignetteRadius * 0.1, u_horizontalVignetteRadius * 0.5, hDist));
      visible *= hFade;

      // Brightness and contrast adjustments
      visible = (visible + u_brightnessAdjust) * u_contrastAdjust;
      visible = clamp(visible, 0.0, 1.0);

      fragColor = vec4(vec3(visible), 1.0);
    }
  `;

  // Pass 2: Glyph rendering (grayscale, theme-aware)
  const glyphFragmentShaderSource = `#version 300 es
    precision highp float;

    in vec2 v_uv;
    out vec4 fragColor;

    uniform sampler2D u_noiseTexture;
    uniform vec2 u_resolution;
    uniform float u_cellSize;
    uniform float u_opacity;
    uniform float u_thresholds[5];
    uniform vec3 u_glyphColor;

    // Glyph drawing functions (SDF-based)
    float drawDot(vec2 uv) {
      vec2 center = vec2(0.5, 0.5);
      float dist = length(uv - center);
      return smoothstep(0.2, 0.15, dist);
    }

    float drawDash(vec2 uv) {
      float h = smoothstep(0.35, 0.4, uv.y) * smoothstep(0.65, 0.6, uv.y);
      float w = smoothstep(0.15, 0.2, uv.x) * smoothstep(0.85, 0.8, uv.x);
      return h * w;
    }

    float drawPlus(vec2 uv) {
      float horiz = smoothstep(0.35, 0.4, uv.y) * smoothstep(0.65, 0.6, uv.y) *
                    smoothstep(0.1, 0.15, uv.x) * smoothstep(0.9, 0.85, uv.x);
      float vert = smoothstep(0.35, 0.4, uv.x) * smoothstep(0.65, 0.6, uv.x) *
                   smoothstep(0.1, 0.15, uv.y) * smoothstep(0.9, 0.85, uv.y);
      return max(horiz, vert);
    }

    float drawO(vec2 uv) {
      vec2 center = vec2(0.5, 0.5);
      float dist = length(uv - center);
      float outer = smoothstep(0.4, 0.35, dist);
      float inner = smoothstep(0.2, 0.25, dist);
      return outer * inner;
    }

    float drawX(vec2 uv) {
      vec2 c = uv - 0.5;
      float d1 = abs(c.x - c.y);
      float d2 = abs(c.x + c.y);
      float line1 = smoothstep(0.15, 0.1, d1);
      float line2 = smoothstep(0.15, 0.1, d2);
      float bounds = smoothstep(0.45, 0.4, abs(c.x)) * smoothstep(0.45, 0.4, abs(c.y));
      return max(line1, line2) * bounds;
    }

    float getGlyph(float brightness, vec2 localUV) {
      if (brightness < u_thresholds[0]) {
        return 0.0; // Empty
      } else if (brightness < u_thresholds[1]) {
        return drawDot(localUV);
      } else if (brightness < u_thresholds[2]) {
        return drawDash(localUV);
      } else if (brightness < u_thresholds[3]) {
        return drawPlus(localUV);
      } else if (brightness < u_thresholds[4]) {
        return drawO(localUV);
      } else {
        return drawX(localUV);
      }
    }

    void main() {
      // Calculate cell coordinates
      vec2 cellCount = u_resolution / u_cellSize;
      vec2 cellCoord = floor(v_uv * cellCount);
      vec2 cellUV = (cellCoord + 0.5) / cellCount;

      // Sample brightness at cell center
      float brightness = texture(u_noiseTexture, cellUV).r;

      // Get local position within cell (0-1)
      vec2 localUV = fract(v_uv * cellCount);

      // Get glyph value
      float glyphValue = getGlyph(brightness, localUV);

      // Output with alpha for transparency (background handled by CSS)
      float alpha = glyphValue * brightness * u_opacity;

      fragColor = vec4(u_glyphColor, alpha);
    }
  `;

  function initWebGL2(canvas: HTMLCanvasElement): {
    gl: WebGL2RenderingContext;
    noiseProgram: WebGLProgram;
    glyphProgram: WebGLProgram;
    noiseVAO: WebGLVertexArrayObject;
    glyphVAO: WebGLVertexArrayObject;
    noiseUniforms: UniformManager;
    glyphUniforms: UniformManager;
  } | null {
    const gl = canvas.getContext("webgl2", {
      alpha: true,
      premultipliedAlpha: false,
    });

    if (!gl) {
      console.warn("WebGL2 not supported, Clouds will not render");
      return null;
    }

    // Compile vertex shader once and reuse for both programs
    const vertexShader = compileShader(
      gl,
      gl.VERTEX_SHADER,
      vertexShaderSource,
    );
    if (!vertexShader) {
      console.error("Vertex shader compilation failed");
      return null;
    }

    const fShaderNoise = compileShader(
      gl,
      gl.FRAGMENT_SHADER,
      noiseFragmentShaderSource,
    );
    const fShaderGlyph = compileShader(
      gl,
      gl.FRAGMENT_SHADER,
      glyphFragmentShaderSource,
    );

    if (!fShaderNoise || !fShaderGlyph) {
      console.error("Fragment shader compilation failed");
      return null;
    }

    const noiseProgram = createProgram(gl, vertexShader, fShaderNoise);
    const glyphProgram = createProgram(gl, vertexShader, fShaderGlyph);

    if (!noiseProgram || !glyphProgram) {
      console.error("Program linking failed");
      return null;
    }

    // Create fullscreen quad buffer
    const quadBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, quadBuffer);
    gl.bufferData(
      gl.ARRAY_BUFFER,
      new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]),
      gl.STATIC_DRAW,
    );

    // Create VAO for noise program
    const noiseVAO = gl.createVertexArray();
    if (!noiseVAO) {
      console.error("Failed to create noise VAO");
      return null;
    }
    gl.bindVertexArray(noiseVAO);
    const noisePosLoc = gl.getAttribLocation(noiseProgram, "a_position");
    gl.enableVertexAttribArray(noisePosLoc);
    gl.vertexAttribPointer(noisePosLoc, 2, gl.FLOAT, false, 0, 0);

    // Create VAO for glyph program
    const glyphVAO = gl.createVertexArray();
    if (!glyphVAO) {
      console.error("Failed to create glyph VAO");
      return null;
    }
    gl.bindVertexArray(glyphVAO);
    const glyphPosLoc = gl.getAttribLocation(glyphProgram, "a_position");
    gl.enableVertexAttribArray(glyphPosLoc);
    gl.vertexAttribPointer(glyphPosLoc, 2, gl.FLOAT, false, 0, 0);

    // Uniform managers
    const noiseUniforms = new UniformManager(gl, noiseProgram, [
      "u_time",
      "u_resolution",
      "u_waveAmplitude",
      "u_waveSpeed",
      "u_noiseIntensity",
      "u_radialVignetteIntensity",
      "u_radialVignetteRadius",
      "u_horizontalVignetteIntensity",
      "u_horizontalVignetteRadius",
      "u_brightnessAdjust",
      "u_contrastAdjust",
      "u_noiseSeed",
    ]);

    const glyphUniforms = new UniformManager(gl, glyphProgram, [
      "u_noiseTexture",
      "u_resolution",
      "u_cellSize",
      "u_opacity",
      "u_thresholds",
      "u_glyphColor",
    ]);

    return {
      gl,
      noiseProgram,
      glyphProgram,
      noiseVAO,
      glyphVAO,
      noiseUniforms,
      glyphUniforms,
    };
  }

  onMount(() => {
    const context = initWebGL2(canvas);
    if (!context) {
      webglFailed = true;
      return;
    }

    const {
      gl,
      noiseProgram,
      glyphProgram,
      noiseVAO,
      glyphVAO,
      noiseUniforms,
      glyphUniforms,
    } = context;

    // Framebuffer and texture for noise pass (half resolution for performance)
    let framebuffer: WebGLFramebuffer | null = null;
    let noiseTexture: WebGLTexture | null = null;
    let noiseWidth = 0;
    let noiseHeight = 0;

    function createFramebuffer(width: number, height: number) {
      if (framebuffer) gl.deleteFramebuffer(framebuffer);
      if (noiseTexture) gl.deleteTexture(noiseTexture);

      // Half resolution for noise pass - glyph shader samples at cell centers anyway
      noiseWidth = Math.floor(width / 2);
      noiseHeight = Math.floor(height / 2);

      noiseTexture = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, noiseTexture);
      gl.texImage2D(
        gl.TEXTURE_2D,
        0,
        gl.RGBA,
        noiseWidth,
        noiseHeight,
        0,
        gl.RGBA,
        gl.UNSIGNED_BYTE,
        null,
      );
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

      framebuffer = gl.createFramebuffer();
      gl.bindFramebuffer(gl.FRAMEBUFFER, framebuffer);
      gl.framebufferTexture2D(
        gl.FRAMEBUFFER,
        gl.COLOR_ATTACHMENT0,
        gl.TEXTURE_2D,
        noiseTexture,
        0,
      );

      gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    }

    const dpr = Math.max(1, Math.min(2, window.devicePixelRatio || 1));
    const seed = generateSeed();

    // Pre-allocated arrays for uniforms (avoid allocation in render loop)
    const noiseResolutionVec: [number, number] = [0, 0];
    const glyphResolutionVec: [number, number] = [0, 0];

    let canvasWidth = 0;
    let canvasHeight = 0;

    // Debounced resize handler
    let resizeTimeout: ReturnType<typeof setTimeout> | null = null;
    const RESIZE_DEBOUNCE_MS = 150;

    const resizeCanvas = () => {
      const width = Math.floor(window.innerWidth * dpr);
      const height = Math.floor(window.innerHeight * dpr);

      if (canvas.width !== width || canvas.height !== height) {
        canvas.width = width;
        canvas.height = height;
        canvas.style.width = `${window.innerWidth}px`;
        canvas.style.height = `${window.innerHeight}px`;
        canvasWidth = width;
        canvasHeight = height;
        createFramebuffer(width, height);

        // Update pre-allocated resolution vectors
        noiseResolutionVec[0] = noiseWidth;
        noiseResolutionVec[1] = noiseHeight;
        glyphResolutionVec[0] = width;
        glyphResolutionVec[1] = height;
      }
    };

    const debouncedResize = () => {
      if (resizeTimeout) clearTimeout(resizeTimeout);
      resizeTimeout = setTimeout(resizeCanvas, RESIZE_DEBOUNCE_MS);
    };

    // Initial resize (immediate, not debounced)
    resizeCanvas();
    window.addEventListener("resize", debouncedResize);
    addCleanup(() => {
      window.removeEventListener("resize", debouncedResize);
      if (resizeTimeout) clearTimeout(resizeTimeout);
    });

    // Enable blending for transparent output
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

    const startTime = performance.now();
    let animationId: number | null = null;
    let firstFrameRendered = false;
    let readyRafIds: number[] = [];

    // Track settings version to detect theme changes
    let lastSettingsRef = settings;

    // Set static uniforms initially
    function setStaticNoiseUniforms() {
      gl.useProgram(noiseProgram);
      noiseUniforms.set("u_waveAmplitude", settings.waveAmplitude);
      noiseUniforms.set("u_waveSpeed", settings.waveSpeed);
      noiseUniforms.set("u_noiseIntensity", settings.noiseIntensity);
      noiseUniforms.set(
        "u_radialVignetteIntensity",
        settings.radialVignetteIntensity,
      );
      noiseUniforms.set(
        "u_radialVignetteRadius",
        settings.radialVignetteRadius,
      );
      noiseUniforms.set(
        "u_horizontalVignetteIntensity",
        settings.horizontalVignetteIntensity,
      );
      noiseUniforms.set(
        "u_horizontalVignetteRadius",
        settings.horizontalVignetteRadius,
      );
      noiseUniforms.set("u_brightnessAdjust", settings.brightnessAdjust);
      noiseUniforms.set("u_contrastAdjust", settings.contrastAdjust);
      noiseUniforms.set("u_noiseSeed", seed);
    }

    function setStaticGlyphUniforms() {
      gl.useProgram(glyphProgram);
      glyphUniforms.set1i("u_noiseTexture", 0);
      glyphUniforms.set("u_cellSize", settings.cellSize * dpr);
      glyphUniforms.set("u_opacity", settings.opacity);
      glyphUniforms.set1fv("u_thresholds", settings.thresholds);
      glyphUniforms.set("u_glyphColor", settings.glyphColor);
    }

    // Set static uniforms on init
    setStaticNoiseUniforms();
    setStaticGlyphUniforms();

    function render() {
      // Check if settings changed (theme switch) and update static uniforms
      if (lastSettingsRef !== settings) {
        lastSettingsRef = settings;
        setStaticNoiseUniforms();
        setStaticGlyphUniforms();
      }

      const time =
        ((performance.now() - startTime) / 1000) * settings.timeSpeed;

      // Pass 1: Render noise to framebuffer (half resolution)
      gl.bindFramebuffer(gl.FRAMEBUFFER, framebuffer);
      gl.viewport(0, 0, noiseWidth, noiseHeight);
      gl.useProgram(noiseProgram);
      gl.bindVertexArray(noiseVAO);

      // Only dynamic uniforms in render loop
      noiseUniforms.set("u_time", time);
      noiseUniforms.setVec2("u_resolution", noiseResolutionVec);

      gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);

      // Pass 2: Render glyphs to screen (full resolution)
      gl.bindFramebuffer(gl.FRAMEBUFFER, null);
      gl.viewport(0, 0, canvasWidth, canvasHeight);
      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);

      gl.useProgram(glyphProgram);
      gl.bindVertexArray(glyphVAO);

      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D, noiseTexture);

      // Only dynamic uniforms in render loop
      glyphUniforms.setVec2("u_resolution", glyphResolutionVec);

      gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);

      // Wait for browser to complete paint before showing canvas
      // Two frames ensures the GPU has flushed the rendered content
      if (!firstFrameRendered) {
        firstFrameRendered = true;
        const raf1 = requestAnimationFrame(() => {
          const raf2 = requestAnimationFrame(() => {
            ready = true;
          });
          readyRafIds.push(raf2);
        });
        readyRafIds.push(raf1);
      }

      animationId = requestAnimationFrame(render);
    }

    // Visibility change handling - stop RAF loop when hidden, restart when visible
    let isVisible = !document.hidden;

    const handleVisibilityChange = () => {
      if (document.hidden) {
        isVisible = false;
        if (animationId !== null) {
          cancelAnimationFrame(animationId);
          animationId = null;
        }
      } else {
        if (!isVisible) {
          isVisible = true;
          animationId = requestAnimationFrame(render);
        }
      }
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);
    addCleanup(() =>
      document.removeEventListener("visibilitychange", handleVisibilityChange),
    );

    // Start rendering
    render();

    addCleanup(() => {
      if (animationId !== null) cancelAnimationFrame(animationId);
      readyRafIds.forEach((id) => cancelAnimationFrame(id));
    });
    addCleanup(() => {
      if (framebuffer) gl.deleteFramebuffer(framebuffer);
      if (noiseTexture) gl.deleteTexture(noiseTexture);
      gl.getExtension("WEBGL_lose_context")?.loseContext();
    });
  });

  onDestroy(() => {
    cleanupFns.forEach((fn) => fn());
  });
</script>

<!-- Wrapper for background + ASCII clouds canvas -->
<div class="pointer-events-none fixed inset-0 -z-20" {style}>
  <!-- Background overlay (also serves as fallback when WebGL fails) -->
  <div
    class="absolute inset-0 bg-white dark:bg-black transition-colors duration-300"
  ></div>

  <!-- ASCII Clouds canvas (hidden if WebGL failed) -->
  {#if !webglFailed}
    <canvas
      bind:this={canvas}
      class={cn(
        "absolute inset-0 z-10 transition-opacity duration-1300 ease-out",
        ready ? "opacity-100" : "opacity-0",
        className,
      )}
    ></canvas>
  {/if}
</div>
