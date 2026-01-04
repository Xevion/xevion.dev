<script lang="ts">
  import { cn } from "$lib/utils";
  import type { ClassValue } from "clsx";
  import { onMount, onDestroy } from "svelte";

  let { class: className = "" }: { class?: ClassValue } = $props();

  let canvas: HTMLCanvasElement;
  let gl: WebGLRenderingContext | null = null;
  let animationId: number | null = null;

  // Noise sampling scale - larger values create smoother, more gradual flow patterns
  const SCALE = 1000;
  // Maximum displacement distance from grid position (in pixels)
  const LENGTH = 10;
  // Distance between grid points (in pixels) - controls dot density
  const SPACING = 20;
  // Global animation speed multiplier - higher values make everything move faster
  const TIMESCALE = 10.25 / 1000;
  // Rotation/angle animation speed multiplier
  const ANGLE_TIME_SCALE = 2.0;
  // Pulsing/length animation speed multiplier
  const LENGTH_TIME_SCALE = 1.5;
  // Base opacity of dots (0-1)
  const OPACITY = 0.9;
  // Radius of each dot (in pixels)
  const RADIUS = 3.5;
  // How much opacity varies with angle (0-1)
  const ANGLE_OPACITY_AMPLITUDE = 0.8;
  // Minimum opacity from angle calculation
  const ANGLE_OPACITY_FLOOR = 0.1;
  // Lower bound of random per-dot opacity
  const RANDOM_OPACITY_MIN = 0.5;
  // Upper bound of random per-dot opacity
  const RANDOM_OPACITY_MAX = 1.0;

  // Simplex noise GLSL implementation
  const vertexShader = `
    attribute vec2 a_position;
    void main() {
      gl_Position = vec4(a_position, 0.0, 1.0);
    }
  `;

  const fragmentShader = `
    precision mediump float;

    uniform vec2 u_resolution;
    uniform float u_time;
    uniform float u_seed;
    uniform float u_dpr;
    uniform float u_scale;
    uniform float u_length;
    uniform float u_spacing;
    uniform float u_opacity;
    uniform float u_radius;
    uniform float u_angleTimeScale;
    uniform float u_lengthTimeScale;
    uniform float u_angleOpacityAmp;
    uniform float u_angleOpacityFloor;
    uniform float u_randomOpacityMin;
    uniform float u_randomOpacityMax;

    const float PI = 3.14159265359;

    // Simplex 3D noise
    vec3 mod289(vec3 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
    vec4 mod289(vec4 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
    vec4 permute(vec4 x) { return mod289(((x*34.0)+1.0)*x); }
    vec4 taylorInvSqrt(vec4 r) { return 1.79284291400159 - 0.85373472095314 * r; }

    float snoise(vec3 v) {
      const vec2 C = vec2(1.0/6.0, 1.0/3.0);
      const vec4 D = vec4(0.0, 0.5, 1.0, 2.0);

      vec3 i  = floor(v + dot(v, C.yyy));
      vec3 x0 = v - i + dot(i, C.xxx);

      vec3 g = step(x0.yzx, x0.xyz);
      vec3 l = 1.0 - g;
      vec3 i1 = min(g.xyz, l.zxy);
      vec3 i2 = max(g.xyz, l.zxy);

      vec3 x1 = x0 - i1 + C.xxx;
      vec3 x2 = x0 - i2 + C.yyy;
      vec3 x3 = x0 - D.yyy;

      i = mod289(i + u_seed);
      vec4 p = permute(permute(permute(
                i.z + vec4(0.0, i1.z, i2.z, 1.0))
              + i.y + vec4(0.0, i1.y, i2.y, 1.0))
              + i.x + vec4(0.0, i1.x, i2.x, 1.0));

      float n_ = 0.142857142857;
      vec3 ns = n_ * D.wyz - D.xzx;

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

      vec4 norm = taylorInvSqrt(vec4(dot(p0,p0), dot(p1,p1), dot(p2,p2), dot(p3,p3)));
      p0 *= norm.x;
      p1 *= norm.y;
      p2 *= norm.z;
      p3 *= norm.w;

      vec4 m = max(0.6 - vec4(dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3)), 0.0);
      m = m * m;
      return 42.0 * dot(m*m, vec4(dot(p0,x0), dot(p1,x1), dot(p2,x2), dot(p3,x3)));
    }

    // Hash function for random per-point opacity
    float hash(vec2 p) {
      return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
    }

    // Convert snoise [-1,1] to p5-style noise [0,1]
    float noise01(vec3 v) {
      return (snoise(v) + 1.0) * 0.5;
    }

    void main() {
      vec2 pixelCoord = gl_FragCoord.xy;

      // Find nearest grid point (account for DPR)
      float spacing = u_spacing * u_dpr;
      float scaleDpr = u_scale * u_dpr;
      vec2 gridCoord = floor(pixelCoord / spacing) * spacing;

      // Calculate distance to all nearby grid points (9 neighbors)
      float minDist = 1000000.0;
      vec2 closestPoint = vec2(0.0);
      float pointOpacity = 0.0;

      for (float dx = -1.0; dx <= 1.0; dx += 1.0) {
        for (float dy = -1.0; dy <= 1.0; dy += 1.0) {
          vec2 testGrid = gridCoord + vec2(dx * spacing, dy * spacing);

          // Get force direction at this grid point (matching original p5 formula)
          // Original: (noise(x/SCALE, y/SCALE, z) - 0.5) * 2 * TWO_PI
          float rad = (noise01(vec3(testGrid / scaleDpr, u_time * u_angleTimeScale)) - 0.5) * 4.0 * PI;
          // Original: (noise(x/SCALE, y/SCALE, z*2) + 0.5) * LENGTH
          float len = (noise01(vec3(testGrid / scaleDpr, u_time * u_lengthTimeScale)) + 0.5) * u_length * u_dpr;

          // Calculate displaced position
          vec2 displacedPoint = testGrid + vec2(cos(rad), sin(rad)) * len;

          float dist = distance(pixelCoord, displacedPoint);

          if (dist < minDist) {
            minDist = dist;
            closestPoint = testGrid;
            pointOpacity = hash(testGrid) * (u_randomOpacityMax - u_randomOpacityMin) + u_randomOpacityMin;
          }
        }
      }

      // Recalculate angle for opacity calculation
      float rad = (noise01(vec3(closestPoint / scaleDpr, u_time * u_angleTimeScale)) - 0.5) * 4.0 * PI;

      // Draw circle with configurable radius
      float circle = 1.0 - smoothstep(0.0, u_radius * u_dpr, minDist);

      // Calculate opacity based on angle
      float angleOpacity = (abs(cos(rad)) * u_angleOpacityAmp + u_angleOpacityFloor) * pointOpacity * u_opacity;

      // Light gray dots with calculated opacity
      gl_FragColor = vec4(vec3(200.0/255.0), circle * angleOpacity);
    }
  `;

  function createShader(gl: WebGLRenderingContext, type: number, source: string): WebGLShader | null {
    const shader = gl.createShader(type);
    if (!shader) return null;

    gl.shaderSource(shader, source);
    gl.compileShader(shader);

    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      console.error('Shader compile error:', gl.getShaderInfoLog(shader));
      gl.deleteShader(shader);
      return null;
    }

    return shader;
  }

  function createProgram(gl: WebGLRenderingContext, vertexShader: WebGLShader, fragmentShader: WebGLShader): WebGLProgram | null {
    const program = gl.createProgram();
    if (!program) return null;

    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);
    gl.linkProgram(program);

    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      console.error('Program link error:', gl.getProgramInfoLog(program));
      gl.deleteProgram(program);
      return null;
    }

    return program;
  }

  function resizeCanvas() {
    if (!canvas) return;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = window.innerWidth * dpr;
    canvas.height = window.innerHeight * dpr;
    canvas.style.width = `${window.innerWidth}px`;
    canvas.style.height = `${window.innerHeight}px`;

    if (gl) {
      gl.viewport(0, 0, canvas.width, canvas.height);
    }
  }

  onMount(() => {
    gl = canvas.getContext('webgl', { alpha: true, premultipliedAlpha: false });
    if (!gl) {
      console.error('WebGL not supported');
      return;
    }

    console.log('WebGL context created');

    const vShader = createShader(gl, gl.VERTEX_SHADER, vertexShader);
    const fShader = createShader(gl, gl.FRAGMENT_SHADER, fragmentShader);

    if (!vShader || !fShader) {
      console.error('Shader creation failed');
      return;
    }

    console.log('Shaders compiled successfully');

    const program = createProgram(gl, vShader, fShader);
    if (!program) return;

    // Set up geometry (full screen quad)
    const positionBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
      -1, -1,
       1, -1,
      -1,  1,
      -1,  1,
       1, -1,
       1,  1,
    ]), gl.STATIC_DRAW);

    const positionLocation = gl.getAttribLocation(program, 'a_position');
    const resolutionLocation = gl.getUniformLocation(program, 'u_resolution');
    const timeLocation = gl.getUniformLocation(program, 'u_time');
    const seedLocation = gl.getUniformLocation(program, 'u_seed');
    const dprLocation = gl.getUniformLocation(program, 'u_dpr');
    const scaleLocation = gl.getUniformLocation(program, 'u_scale');
    const lengthLocation = gl.getUniformLocation(program, 'u_length');
    const spacingLocation = gl.getUniformLocation(program, 'u_spacing');
    const opacityLocation = gl.getUniformLocation(program, 'u_opacity');
    const radiusLocation = gl.getUniformLocation(program, 'u_radius');
    const angleTimeScaleLocation = gl.getUniformLocation(program, 'u_angleTimeScale');
    const lengthTimeScaleLocation = gl.getUniformLocation(program, 'u_lengthTimeScale');
    const angleOpacityAmpLocation = gl.getUniformLocation(program, 'u_angleOpacityAmp');
    const angleOpacityFloorLocation = gl.getUniformLocation(program, 'u_angleOpacityFloor');
    const randomOpacityMinLocation = gl.getUniformLocation(program, 'u_randomOpacityMin');
    const randomOpacityMaxLocation = gl.getUniformLocation(program, 'u_randomOpacityMax');

    gl.useProgram(program);
    gl.enableVertexAttribArray(positionLocation);
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    gl.vertexAttribPointer(positionLocation, 2, gl.FLOAT, false, 0, 0);

    // Enable blending for transparency
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

    const dpr = window.devicePixelRatio || 1;

    // Set static uniforms (these don't change per frame)
    gl.uniform1f(seedLocation, Math.random() * 1000);
    gl.uniform1f(dprLocation, dpr);
    gl.uniform1f(scaleLocation, SCALE);
    gl.uniform1f(lengthLocation, LENGTH);
    gl.uniform1f(spacingLocation, SPACING);
    gl.uniform1f(opacityLocation, OPACITY);
    gl.uniform1f(radiusLocation, RADIUS);
    gl.uniform1f(angleTimeScaleLocation, ANGLE_TIME_SCALE);
    gl.uniform1f(lengthTimeScaleLocation, LENGTH_TIME_SCALE);
    gl.uniform1f(angleOpacityAmpLocation, ANGLE_OPACITY_AMPLITUDE);
    gl.uniform1f(angleOpacityFloorLocation, ANGLE_OPACITY_FLOOR);
    gl.uniform1f(randomOpacityMinLocation, RANDOM_OPACITY_MIN);
    gl.uniform1f(randomOpacityMaxLocation, RANDOM_OPACITY_MAX);

    resizeCanvas();

    let startTime = Date.now();

    function render() {
      if (!gl || !canvas) return;

      const time = (Date.now() - startTime) / 1000 * TIMESCALE;

      gl.uniform2f(resolutionLocation, canvas.width, canvas.height);
      gl.uniform1f(timeLocation, time);

      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.TRIANGLES, 0, 6);

      animationId = requestAnimationFrame(render);
    }

    render();

    window.addEventListener('resize', resizeCanvas);

    return () => {
      window.removeEventListener('resize', resizeCanvas);
    };
  });

  onDestroy(() => {
    if (animationId !== null) {
      cancelAnimationFrame(animationId);
    }
    if (gl) {
      gl.getExtension('WEBGL_lose_context')?.loseContext();
    }
  });
</script>

<!-- Dots overlay with fade-in animation -->
<canvas
  bind:this={canvas}
  class={cn(
    "pointer-events-none fixed inset-0 -z-10",
    className
  )}
></canvas>
