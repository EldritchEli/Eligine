#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoord;

layout(location = 0) out vec4 outColor;
layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
    mat4 inv_view;
    float time;
} ubo;

layout(binding = 1) uniform sampler2D texSampler;
void main() {
    float mult = sin(5.0 * ubo.time);
    outColor = texture(texSampler, fragTexCoord * 1.0);
}
