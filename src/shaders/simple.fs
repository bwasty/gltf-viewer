#version 330 core
out vec4 FragColor;

in vec3 Normal;
in vec2 TexCoords;

uniform sampler2D texture_diffuse1;

void main()
{
    // FragColor = texture(texture_diffuse1, TexCoords);
    // FragColor = vec4(0.800000011920929, 0.0, 0.0, 1.0);
    FragColor = vec4(Normal, 1.0);
}
