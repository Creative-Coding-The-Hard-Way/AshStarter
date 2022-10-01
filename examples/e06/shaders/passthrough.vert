#version 450
#extension GL_ARB_separate_shader_objects: enable

layout(binding = 0) uniform UniformBufferObject {
  mat4 proj;
} ubo;

layout(location = 0) in vec2 pos;
layout(location = 1) in vec4 color;

layout(location = 0) out vec4 vertex_color;

layout(push_constant) uniform PushConstants {
    vec4 color;
} pc;

void main() {
    vertex_color = pc.color * color;
    gl_Position = ubo.proj * vec4(pos, 0.0, 1.0);
}
