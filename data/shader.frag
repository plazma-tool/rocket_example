#version 430

out vec4 out_color;

uniform vec2 iResolution;

uniform float H_y_offset;
uniform float H_opacity;
uniform float i_y_offset;
uniform float i_opacity;

void mainImage( out vec4 fragColor, in vec2 fragCoord ) {
  vec2 uv = -1.0 + 2.0 * fragCoord.xy / iResolution.xy;
  uv.x *= iResolution.x / iResolution.y;

  vec3 col = vec3(0.1, 0.2, 0.3);
  col += vec3(0., 0., 1.) * (1.0 - smoothstep(0.3, 0.31, distance(uv, vec2(0.0, H_y_offset))));

  fragColor = vec4(col, 1.0);
}

void main() {
  vec4 col = vec4(0.0, 0.0, 0.0, 1.0);
  mainImage(col, gl_FragCoord.xy);
  out_color = col;
}
