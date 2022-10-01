#version 450
#extension GL_ARB_separate_shader_objects: enable

layout(binding = 0) uniform UniformBufferObject {
  mat4 proj;
} ubo;

layout(location = 0) in vec2 pos;
layout(location = 1) in vec4 color;

layout(location = 0) out vec4 vertex_color;

layout(push_constant) uniform PushConstants {
    float angle;
} pc;

void main() {
    mat2 rot = mat2(
        cos(pc.angle), -sin(pc.angle),
        sin(pc.angle), cos(pc.angle)
    );
    vertex_color = color;
    gl_Position = ubo.proj * vec4(pos * rot, 0.0, 1.0);
}
