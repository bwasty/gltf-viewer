#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec4 aTangent;
layout (location = 3) in vec2 aTexCoords_0;
layout (location = 4) in vec2 aTexCoords_1;
layout (location = 5) in vec3 aColor;

// out vec3 Normal;
// out vec4 Tangent;
out vec2 TexCoords_0;
// out vec2 TexCoords_1;
// out vec3 Color;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main()
{
    // Normal = aNormal; // TODO: transform
    // Tangent = aTangent;
    TexCoords_0 = aTexCoords_0;
    // TexCoords_1 = aTexCoords_1;
    // Color = aColor;
    gl_Position = projection * view * model * vec4(aPos, 1.0);
}
