#version 450

layout(binding = 0) uniform UniformBufferObject {
    int x;
    int y;
	mat4 view;
	mat4 proj;

} ubo;
layout (binding = 1) uniform samplerCube cubemap;
layout(location = 0) in vec3 world_space_pos;
layout(location = 0) out vec4 outColor;



void main() {
	// If your skybox is a cubemap
    outColor = texture(cubemap,world_space_pos);
}

