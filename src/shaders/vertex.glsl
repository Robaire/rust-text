#version 330 core

layout(location = 0) in vec2 vertex;
layout(location = 1) in vec2 texture_vertex;

uniform mat4 projection;

out vec2 texture_coordinate;

void main() {
    texture_coordinate = texture_vertex;
    gl_Position = projection * vec4(vertex.xy, 0.0, 1.0);
}