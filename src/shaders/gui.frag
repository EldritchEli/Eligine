
#version 450
layout(binding = 0) uniform UniformBufferObject {
	mat4 view;
	mat4 proj;
    float x;
    float y;

} ubo;
layout(binding = 1) uniform sampler2D texSampler;
layout(location = 0) out vec4 outColor;
layout(location = 0) in vec2 fragCoord;
layout(location = 1) in vec4 fragColor;


void main() {
	// If your skybox is a cubemap
    outColor =  texture(texSampler,fragCoord);
    outColor += fragColor;
   //outColor = vec4(ubo.x,ubo.y,0.0,1.0);
}


