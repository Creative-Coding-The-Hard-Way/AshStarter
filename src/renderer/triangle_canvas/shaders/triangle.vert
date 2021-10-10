#version 450
#extension GL_ARB_separate_shader_objects: enable

struct Vertex
{
    float x, y;
    float r, g, b, a;
};

layout(set=0, binding=0) readonly buffer SBO { Vertex data[]; } sbo;

layout(location = 0) out vec4 vertex_color;

void main() {
    Vertex vert = sbo.data[gl_VertexIndex];
    vertex_color = vec4(vert.r, vert.g, vert.b, vert.a);
    gl_Position = vec4(vert.x, vert.y, 0.0, 1.0);
}
