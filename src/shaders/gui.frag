
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


const float GAMMA = 2.2;
const float INV_GAMMA = 1.0 / GAMMA;

vec3 LINEARtoSRGB(vec3 color) {
    return pow(color, vec3(INV_GAMMA));
}
vec4 LINEARtoSRGB(vec4 color) {
    return vec4(LINEARtoSRGB(color.rgb), color.a);
}
void main() {
	// If your skybox is a cubemap
    outColor =  texture(texSampler,fragCoord);
    outColor *= fragColor;
    //outColor = vec4(outColor.w);
}


