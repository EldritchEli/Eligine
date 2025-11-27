#version 450


layout(binding = 0) uniform Sun {
    vec4 dir;
    vec4 color;
} sun;
layout (binding = 1) uniform Global {
    mat4 view;
    mat4 proj;
    float x;
    float y;
} global_ubo;

layout(binding = 2) uniform UniformBufferObject {
    mat4 model[10];
    vec4 base;
} ubo;

layout(binding = 3) uniform sampler2D texSampler;

layout(location = 0) in vec3 fragNormal;
layout(location = 1) in vec2 fragTexCoord;

layout(location = 0) out vec4 outColor;


void main() {

    vec4 surfacePos = global_ubo.view*inverse(global_ubo.proj)*gl_FragCoord;
    vec3 cameraDir = normalize((global_ubo.view*vec4(0.0,0.0,0.0,1.0)).xyz );
    vec3 normal = normalize(fragNormal);
    vec3 lightDir   = sun.dir.xyz;
    vec3 viewDir    = normalize(-global_ubo.view*vec4(0.0,0.0,0.0,1.0) - surfacePos).xyz;
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), 10.0);
    vec3 specular = sun.color.xyz * spec;
    vec4 color = ubo.base*texture(texSampler, fragTexCoord);
    float lambertian = pow((dot(normal,sun.dir.xyz)*0.5 +0.5),2.0);
    color *= lambertian;
    color += vec4(specular,0.0);
    outColor = color;
    //outColor = vec4(global_ubo.x,global_ubo.y,0.0,1.0);
   //outColor = vec4(1000.0);
}
