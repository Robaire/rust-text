#version 330 core

precision mediump float;

in vec2 texture_coordinate;

uniform sampler2D texture_sampler;

out vec4 Color;

void main() {

    // Get the red channel from the texture and use it as the alpha
    vec4 sampled = vec4(1.0, 1.0, 1.0, texture(texture_sampler, texture_coordinate).r);

    // Multiple our text color by the alpha
    // Color = vec4(1.0, 0.0, 1.0, 1.0) * sampled;
    Color = vec4(1.0, 0.0, 1.0, 1.0);
}