#version 450
layout(binding = 0) uniform UniformBufferObject {
	mat4 view;
	mat4 proj;
    float x;
    float y;

} ubo;
layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec2 inTexCoord;
layout(location = 2) in vec4 inColor;

layout (location = 0) out vec2 fragCoord;
layout (location = 1) out vec4 fragColor;

const float GAMMA = 2.2;
vec3 SRGBtoLINEAR(vec3 color) {
    return pow(color, vec3(GAMMA));
}
vec4 SRGBtoLINEAR(vec4 color) {
    return vec4(SRGBtoLINEAR(color.rgb), color.a);
}

void main() {

	//const array of positions for the triangle
	vec3 positions[4] = vec3[4](
		vec3(-1.0f,-1.0f, 0.0f),
		vec3(-1.0f,1.0f, 0.0f),
		vec3(1.0f,-1.0f,0.0f),
        vec3(1.0f,1.0f,0.0f)
	);

	//const array of colors for the triangle
	vec2 coords[4] = vec2[4](
		vec2(0.0f, 0.0f), //red
		vec2(0.0f, 1.0f), //green
		vec2(1.f, 0.0f), //blue,
        vec2(1.f, 1.0f)  //blue
	);
	float x = ubo.x;
    float y = ubo.y;
    vec2 pos= inPosition/vec2(x,y);
    pos *=2.0;
    pos -=vec2(1.0);

    gl_Position = vec4(pos,0.0f,1.0f);
    //fragCoord = coords[gl_VertexIndex];
    fragCoord = inTexCoord;
	//fragColor = SRGBtoLINEAR(inColor);
	fragColor = inColor;
}