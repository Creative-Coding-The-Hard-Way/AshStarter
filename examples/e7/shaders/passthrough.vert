#version 450
#extension GL_ARB_separate_shader_objects: enable

layout(binding = 0) uniform UniformBufferObject {
  mat4 proj;
} ubo;

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 vertex_uv;

void main() {
    vertex_uv = uv;
    gl_Position = ubo.proj * vec4(pos, 0.0, 1.0);
}
