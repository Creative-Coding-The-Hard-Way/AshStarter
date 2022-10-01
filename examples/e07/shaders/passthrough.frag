#version 450
#extension GL_ARB_separate_shader_objects: enable

layout(location = 0) in vec2 vertex_uv;
layout(location = 0) out vec4 frag_color;

layout(binding = 1) uniform sampler2D texSampler;

void main() {
    frag_color = texture(texSampler, vertex_uv);
}
