#version 430

out vec4 out_color;

uniform float iGlobalTime;
uniform vec2 iResolution;

uniform float H_y;
uniform float i_y;
uniform float ground_y;
uniform float interlace;
uniform float white_noise;

float sdCircle(vec2 p, float radius);
float sdBox(vec2 p, vec2 size, float radius);
float fillMask(float dist);
vec3 hashOld33(vec3 p);

float vert_bar(vec2 p) {
  float dist = 0.0;
  vec2 q = p;
  q *= vec2(1.8, 0.8);
  float a = sdCircle(q - vec2(70.0, 0.0), 100.0);
  float b = sdCircle(q - vec2(-70.0, 0.0), 100.0);
  dist = max(a, b);
  a = sdBox(q - vec2(0.0, -65.0), vec2(40.0, 20.0), 0.0);
  dist = max(dist, -a);
  return dist;
}

float H_bar(vec2 p) {
  float dist = vert_bar(p);
  vec2 q = p;
  q *= vec2(1.8, 0.8);
  float a = sdBox(q - vec2(0.0, 0.0), vec2(40.0, 10.0), 0.0);
  dist = min(dist, a);
  return dist;
}

float letter_H(vec2 p) {
  float dist = 0.0;

  float a = H_bar(p - vec2(40.0, 0.0));
  float b = H_bar(p + vec2(40.0, 0.0));
  dist = min(a, b);

  a = sdBox(p - vec2(0.0, 0.0), vec2(20.0, 8.0), 0.0);
  dist = min(dist, a);

  return dist;
}

float letter_i(vec2 p) {
  float dist = 0.0;

  float a = vert_bar(p);
  vec2 q = p;
  q *= vec2(1.0, 2.5);
  float b = vert_bar(q * vec2(1.0, -1.0) + vec2(0.0, 100.0));
  float c = max(a, -b);

  dist = c;

  a = vert_bar(p);
  b = vert_bar(p * vec2(1.0, -1.0) + vec2(0.0, 120.0));
  c = max(a, b);

  dist = min(dist, c);

  return dist;
}

float sceneDist(vec2 p) {
  p *= 150;
  float dist = 0.0;

  float a = letter_H(p + vec2( 50.0, 30.0 + (H_y * 100.0)));
  float b = letter_i(p + vec2(-50.0, 30.0 + (i_y * 100.0)));

  dist = min(a, b);
  return dist;
}

void mainImage( out vec4 fragColor, in vec2 fragCoord ) {
  vec2 uv = -1.0 + 2.0 * fragCoord.xy / iResolution.xy;
  uv.x *= iResolution.x / iResolution.y;

  vec3 col_feind      = vec3(0.35, 0.31, 0.31);
  vec3 col_rock       = vec3(0.33, 0.47, 0.50);
  vec3 col_green_blue = vec3(0.27, 0.68, 0.66);
  vec3 col_fetch      = vec3(0.61, 0.88, 0.68);
  vec3 col_pancakes   = vec3(0.90, 0.99, 0.76);
  vec3 col_young      = vec3(0.99, 0.70, 0.06);

  // background
  vec3 col = vec3(0.1);

  // halo
  col = mix(col, col_green_blue * 0.7, (1.0 - smoothstep(2.3, 2.31, distance(uv, vec2(0.0, -1.2 + iGlobalTime / 100.0)))));
  // ground
  col = mix(col, col_rock * 0.5, (1.0 - smoothstep(10.0, 10.01, distance(uv, vec2(0.0, -10.4 + ground_y)))));

  // Draw "Hi"
  float d = sceneDist(uv);
	col = mix(col, col_young, fillMask(d));

  // Interlace
  col = clamp(col -
              col * interlace * sin(uv.y * 300.0),
              vec3(0.), vec3(1.));

  // White Noise
  vec3 c = vec3(hashOld33(vec3(uv.y + iGlobalTime)).r);
  col = mix(col, c, white_noise);

  fragColor = vec4(col, 1.0);
}

void main() {
  vec4 col = vec4(0.0, 0.0, 0.0, 1.0);
  mainImage(col, gl_FragCoord.xy);
  out_color = col;
}

// 2d signed distance functions - Maarten
// https://www.shadertoy.com/view/4dfXDn

float sdCircle(vec2 p, float radius) {
  return length(p) - radius;
}

float sdBox(vec2 p, vec2 size, float radius) {
	size -= vec2(radius);
	vec2 d = abs(p) - size;
  return min(max(d.x, d.y), 0.0) + length(max(d, 0.0)) - radius;
}

float fillMask(float dist) {
	return clamp(-dist, 0.0, 1.0);
}

// Noise
// David Hoskins https://www.shadertoy.com/view/4djSRW
vec3 hashOld33(vec3 p) {
	p = vec3( dot(p,vec3(127.1,311.7, 74.7)),
            dot(p,vec3(269.5,183.3,246.1)),
            dot(p,vec3(113.5,271.9,124.6)));

	return fract(sin(p)*43758.5453123);
}

// Palette by alpen
// http://www.colourlovers.com/palette/443995/i_demand_a_pancake
// feind      = 89,79,79    = vec3(0.35, 0.31, 0.31)
// rock       = 84,121,128  = vec3(0.33, 0.47, 0.50)
// green_blue = 69,173,168  = vec3(0.27, 0.68, 0.66)
// fetch      = 157,224,173 = vec3(0.61, 0.88, 0.68)
// pancakes   = 229,252,194 = vec3(0.90, 0.99, 0.76)
