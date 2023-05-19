#version 460
#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) in vec2 uv;
layout(location = 1) in vec4 color;
layout(location = 2) flat in int textureIndex;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 1) uniform sampler2D tex[];

void main() {
  vec4 tex_color = vec4(1.0);
  if (textureIndex >= 0) {
    tex_color = texture(tex[nonuniformEXT(textureIndex)], uv);
  }

  out_color = tex_color * color;
}
