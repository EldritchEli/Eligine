#version 450
#define INF 100

layout(binding = 0) uniform UniformBufferObject {
	mat4 view;
	mat4 proj;
    int x;
    int y;

} ubo;
layout (location = 0) out vec3 world_space_pos;
void main() {

	//const array of positions for the triangle
	vec3 positions[4] = vec3[4](
		vec3(-1.0f,-1.0f, 0.999999),
		vec3(-1.0f,1.0f,  0.999999),
		vec3(1.0f,-1.0f,  0.999999),
        vec3(1.0f,1.0f,   0.999999)
	);

	//const array of colors for the triangle
	vec3 coords[4] = vec3[4](
		vec3(0.0f, 0.0f, 0.0f), //red
		vec3(0.0f, 1.0f, 0.0f), //green
		vec3(1.f, 0.0f, 1.0f), //blue,
        vec3(1.f, 1.0f, 1.0f)  //blue
	);
	mat4 inverseTransposeProj= transpose(inverse(ubo.view));
	world_space_pos = mat3(inverseTransposeProj) * ((inverse(ubo.proj) * vec4(positions[gl_VertexIndex],1.0))).xyz;
	gl_Position = vec4(1.0*positions[gl_VertexIndex], 1.0f);

}
