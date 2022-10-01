#version 450
#extension GL_ARB_separate_shader_objects: enable

layout(location = 0) in vec2 pos;
layout(location = 1) in vec4 color;

layout(location = 0) out vec4 vertex_color;

void main() {
    vertex_color = color;
    gl_Position = vec4(pos, 0.0, 1.0);
}
