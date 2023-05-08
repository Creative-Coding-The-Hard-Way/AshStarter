#version 460

struct Vertex{
    vec4 pos;
};

layout(std140, set = 0, binding = 0) readonly buffer Data{
    Vertex vertices[];
} data;

void main() {
    Vertex vertex = data.vertices[gl_VertexIndex];
    gl_Position = vec4(vertex.pos.x, vertex.pos.y, 0.0, 1.0);
}
