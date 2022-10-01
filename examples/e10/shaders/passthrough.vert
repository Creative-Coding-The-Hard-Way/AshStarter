#version 450
#extension GL_ARB_separate_shader_objects: enable

struct Vertex {
  vec2 pos;
  vec4 color;
};

layout(push_constant) uniform PushConstants {
    float angle;
} pc;

layout(binding = 0) uniform UniformBufferObject {
  mat4 proj;
} ubo;

layout(std430, binding = 1) readonly buffer SBO {
    Vertex data[];
} sbo;

layout(location = 0) out vec4 vertex_color;

void main() {
    Vertex vertex = sbo.data[gl_VertexIndex];
    mat2 rot = mat2(
        cos(pc.angle), -sin(pc.angle),
        sin(pc.angle), cos(pc.angle)
    );
    vertex_color = vertex.color;
    gl_Position = ubo.proj * vec4(vertex.pos * rot, 0.0, 1.0);
}
