// Originally taken from https://github.com/KhronosGroup/glTF-WebGL-PBR
// Commit a94655275e5e4e8ae580b1d95ce678b74ab87426

#version 330 core
layout (location = 0) in vec4 a_Position;
#ifdef HAS_NORMALS
layout (location = 1) in vec4 a_Normal;
#endif
#ifdef HAS_TANGENTS
layout (location = 2) in vec4 a_Tangent;
#endif
#ifdef HAS_UV
layout (location = 3) in vec2 a_UV_0; // TEXCOORD_0
layout (location = 4) in vec2 a_UV_1; // TEXCOORD_1
#endif
// TODO!: tex_coord_1, joints_0, weights_0
#ifdef HAS_COLORS
layout (location = 5) in vec4 a_Color; // COLOR_0
#endif

uniform mat4 u_MVPMatrix;
uniform mat4 u_ModelMatrix;

out vec3 v_Position;
out vec2 v_UV[2];
out vec4 v_Color;

#ifdef HAS_NORMALS
#ifdef HAS_TANGENTS
out mat3 v_TBN;
#else
out vec3 v_Normal;
#endif
#endif


void main()
{
  vec4 pos = u_ModelMatrix * a_Position;
  v_Position = vec3(pos.xyz) / pos.w;

  #ifdef HAS_NORMALS
  #ifdef HAS_TANGENTS
  // TODO!: the reference shader was updated to use the normal matrix here
  vec3 normalW = normalize(vec3(u_ModelMatrix * vec4(a_Normal.xyz, 0.0)));
  vec3 tangentW = normalize(vec3(u_ModelMatrix * vec4(a_Tangent.xyz, 0.0)));
  vec3 bitangentW = cross(normalW, tangentW) * a_Tangent.w;
  v_TBN = mat3(tangentW, bitangentW, normalW);
  #else // HAS_TANGENTS != 1
  v_Normal = normalize(vec3(u_ModelMatrix * vec4(a_Normal.xyz, 0.0)));
  #endif
  #endif

  #ifdef HAS_UV
  v_UV[0] = a_UV_0;
  v_UV[1] = a_UV_1;
  #else
  v_UV[0] = vec2(0.,0.);
  v_UV[1] = vec2(0.,0.);
  #endif

  #ifdef HAS_COLORS
  v_Color = a_Color;
  #else
  v_Color = vec4(1.0);
  #endif

  gl_Position = u_MVPMatrix * a_Position; // needs w for proper perspective correction
}


