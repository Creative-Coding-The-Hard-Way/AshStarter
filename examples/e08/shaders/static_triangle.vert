#version 460

struct Vertex{
    vec2 pos;
    vec2 uv;
};

layout(std140, set = 0, binding = 0) readonly buffer Data{
    Vertex vertices[];
} data;

layout(location = 0) out vec2 uv;

void main() {
    Vertex vertex = data.vertices[gl_VertexIndex];
    uv = vertex.uv;
    gl_Position = vec4(vertex.pos.x, vertex.pos.y, 0.0, 1.0);
}
