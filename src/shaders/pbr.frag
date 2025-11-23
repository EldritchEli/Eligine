#version 450


layout(binding = 1) uniform UniformBufferObject {
    mat4 model[10];
    vec4 base;
} ubo;

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoord;

layout(location = 0) out vec4 outColor;

layout(binding = 2) uniform sampler2D texSampler;
void main() {
    outColor = ubo.base*vec4(fragColor,1.0)*texture(texSampler, fragTexCoord * 1.0);
    
   // outColor = vec4(1000.0);
}
