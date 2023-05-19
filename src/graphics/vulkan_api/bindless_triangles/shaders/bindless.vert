#version 460

struct Vertex{
    vec4 pos;
    vec3 uv;
    vec4 rgba;
};

layout(std140, set = 0, binding = 0) readonly buffer Data{
    Vertex vertices[];
} data;

layout(location = 0) out vec2 uv;
layout(location = 1) out vec4 rgba;
layout(location = 2) flat out int textureIndex;

void main() {
    Vertex vertex = data.vertices[gl_VertexIndex];

    uv = vertex.uv.xy;
    rgba = vertex.rgba;
    textureIndex = int(vertex.uv.z);

    gl_Position = vec4(vertex.pos.x, vertex.pos.y, 0.0, 1.0);
}
