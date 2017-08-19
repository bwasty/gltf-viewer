#version 330 core
out vec4 FragColor;

// in vec3 Normal;
in vec4 Tangent;
in vec2 TexCoords_0;
in vec2 TexCoords_1;
in vec3 Color;

uniform sampler2D base_color_texture;
uniform vec4 base_color_factor;

void main()
{
    vec4 baseColor = texture(base_color_texture, TexCoords_0);
    // TODO!: HACK
    if (baseColor.x > 0 || baseColor.y > 0 || baseColor.z > 0) {
        FragColor = baseColor * base_color_factor;
    }
    else {
        FragColor = base_color_factor;
    }

    // FragColor = vec4(Normal, 1.0);
    // FragColor = vec4(Tangent.xyz, 1.0);
    // FragColor = vec4(TexCoords_0, 0.0, 1.0);
    // FragColor = vec4(TexCoords_1, 0,0, 1.0);
    // FragColor = vec4(Color, 1.0);
}
