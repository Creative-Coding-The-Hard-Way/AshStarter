#version 460

layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 1) uniform sampler2D tex;

void main() {
  //out_color = vec4(0.0, uv.y, uv.x, 1.0);
  out_color = texture(tex, uv);
}
