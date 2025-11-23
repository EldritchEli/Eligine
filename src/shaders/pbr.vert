#version 450

layout (binding = 0) uniform Global {
    mat4 view;
    mat4 proj;
    int x;
    int y;
} global_ubo;

layout(binding = 1) uniform UniformBufferObject {
    mat4 model[10];
    vec4 base;
} ubo;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inColor;
layout(location = 2) in vec2 inTexCoord;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 fragTexCoord;

layout (push_constant ) uniform constants {
    mat4 proj_inv_view;
} PushConstants;

void main() {
    gl_Position = global_ubo.proj * inverse(global_ubo.view) * ubo.model[gl_InstanceIndex] * vec4(inPosition, 1.0);
    fragColor = inColor;
    fragTexCoord = inTexCoord;
}
