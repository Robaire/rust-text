#version 330 core

layout(location = 0) in vec4 vertex;

uniform mat4 projection;

out vec2 texture_coordinate;

void main() {
    texture_coordinate = vertex.zw;
    gl_Position = projection * vec4(vertex.xy, 0.0, 1.0);
}