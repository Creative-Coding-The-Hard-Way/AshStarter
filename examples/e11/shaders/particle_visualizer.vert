#version 450
#extension GL_ARB_separate_shader_objects: enable

struct Particle {
  vec2 pos;
  vec2 vel;
};

layout(binding = 0) uniform UniformBufferObject {
  mat4 projection;
} ubo;

layout(std430, binding = 1) readonly buffer SBO {
    Particle particles[];
} sbo;

layout(location = 0) out vec4 vertex_color;

void main() {
    Particle particle = sbo.particles[gl_VertexIndex];
    vertex_color = vec4(1.0, 1.0, 1.0, 1.0);
    gl_PointSize = 1.0;
    gl_Position = ubo.projection * vec4(particle.pos, 0.0, 1.0);
}
