use std::str;

use crate::platform::{compile_shader_and_get_id,GltfViewerRenderer,read_vertex_code,read_fragment_code,UniformHelpers};


use bitflags::bitflags;

pub struct Shader {
    pub id: u32,
}

impl Shader {
    #[allow(dead_code)]
    pub fn new(vertex_path: &str, fragment_path: &str, defines: &[String], renderer: &mut GltfViewerRenderer) -> Shader {
        let vertex_code = read_vertex_code(vertex_path);
        let fragment_code = read_fragment_code(fragment_path);
        Self::from_source(&vertex_code, &fragment_code, defines, renderer)
    }

    pub fn from_source(vertex_code: &str, fragment_code: &str, defines: &[String], renderer: &mut GltfViewerRenderer) -> Shader {
        let mut shader = Shader {
            id: 0
        };

        let vertex_code = Self::add_defines(vertex_code, defines);
        let fragment_code = Self::add_defines(fragment_code, defines);

        unsafe {
            shader.id = compile_shader_and_get_id(&vertex_code.to_string(), &fragment_code.to_string(), renderer).unwrap();
        }

        shader
    }

    fn add_defines(source: &str, defines: &[String]) -> String {
        // insert preprocessor defines after #version if exists
        // (#version must occur before any other statement in the program)
        let defines = defines.iter()
            .map(|define| format!("#define {}", define))
            .collect::<Vec<_>>()
            .join("\n");
        let mut lines: Vec<_> = source.lines().collect();
        if let Some(version_line) = lines.iter().position(|l| l.starts_with("#version")) {
            // replace version line for webgl target
            #[cfg(feature = "use_wasm_bindgen")]
            {
                lines[version_line] = "#version 300 es";
            }

            lines.insert(version_line+1, &defines);
        }
        else {
            lines.insert(0, &defines);
        }
        lines.join("\n")
    }
}

bitflags! {
    /// Flags matching the defines in the PBR shader
    pub struct ShaderFlags: u16 {
        // vertex shader + fragment shader
        const HAS_NORMALS           = 1;
        const HAS_TANGENTS          = 1 << 1;
        const HAS_UV                = 1 << 2;
        const HAS_COLORS            = 1 << 3;

        // fragment shader only
        const USE_IBL               = 1 << 4;
        const HAS_BASECOLORMAP      = 1 << 5;
        const HAS_NORMALMAP         = 1 << 6;
        const HAS_EMISSIVEMAP       = 1 << 7;
        const HAS_METALROUGHNESSMAP = 1 << 8;
        const HAS_OCCLUSIONMAP      = 1 << 9;
        const USE_TEX_LOD           = 1 << 10;
    }
}

impl ShaderFlags {
    pub fn as_strings(self) -> Vec<String> {
        (0..15)
            .map(|i| 1u16 << i)
            .filter(|i| self.bits & i != 0)
            .map(|i| format!("{:?}", ShaderFlags::from_bits_truncate(i)))
            .collect()
    }
}

#[allow(non_snake_case)]
pub struct PbrUniformLocations {
    // uniform locations
    // TODO!: UBO for matrices, camera, light(s)?
    pub u_MVPMatrix: i32,
    pub u_ModelMatrix: i32,
    pub u_Camera: i32,

    pub u_LightDirection: i32,
    pub u_LightColor: i32,

    pub u_AmbientLightColor: i32,
    pub u_AmbientLightIntensity: i32,

    // TODO!: set when integrating IBL (unused now)
    pub u_DiffuseEnvSampler: i32,
    pub u_SpecularEnvSampler: i32,
    pub u_brdfLUT: i32,

    ///

    pub u_BaseColorSampler: i32,
    pub u_BaseColorTexCoord: i32,
    pub u_BaseColorFactor: i32,

    pub u_NormalSampler: i32,
    pub u_NormalTexCoord: i32,
    pub u_NormalScale: i32,

    pub u_EmissiveSampler: i32,
    pub u_EmissiveTexCoord: i32,
    pub u_EmissiveFactor: i32,

    pub u_MetallicRoughnessSampler: i32,
    pub u_MetallicRoughnessTexCoord: i32,
    pub u_MetallicRoughnessValues: i32,

    pub u_OcclusionSampler: i32,
    pub u_OcclusionTexCoord: i32,
    pub u_OcclusionStrength: i32,

    pub u_AlphaBlend: i32,
    pub u_AlphaCutoff: i32,

    // TODO!: use/remove debugging uniforms
    // debugging flags used for shader output of intermediate PBR variables
    pub u_ScaleDiffBaseMR: i32,
    pub u_ScaleFGDSpec: i32,
    pub u_ScaleIBLAmbient: i32,
}

pub struct PbrShader {
    pub shader: Shader,
    pub flags: ShaderFlags,
    pub uniforms: PbrUniformLocations,
}

impl PbrShader {
    pub fn new(flags: ShaderFlags, renderer: &mut GltfViewerRenderer) -> Self {
        let mut shader = Shader::from_source(
            include_str!("shaders/pbr-vert.glsl"),
            include_str!("shaders/pbr-frag.glsl"),
            &flags.as_strings(),
            renderer);

        // NOTE: shader debug version
        // let mut shader = Shader::new(
        //     "src/shaders/pbr-vert.glsl",
        //     "src/shaders/pbr-frag.glsl",
        //     &flags.as_strings());

        let uniforms = unsafe {
            let uniforms = PbrUniformLocations {
                u_MVPMatrix: shader.uniform_location(renderer, "u_MVPMatrix"),
                u_ModelMatrix: shader.uniform_location(renderer, "u_ModelMatrix"),
                u_Camera: shader.uniform_location(renderer, "u_Camera"),

                u_LightDirection: shader.uniform_location(renderer, "u_LightDirection"),
                u_LightColor: shader.uniform_location(renderer, "u_LightColor"),

                u_AmbientLightColor: shader.uniform_location(renderer, "u_AmbientLightColor"),
                u_AmbientLightIntensity: shader.uniform_location(renderer, "u_AmbientLightIntensity"),

                u_DiffuseEnvSampler: shader.uniform_location(renderer, "u_DiffuseEnvSampler"),
                u_SpecularEnvSampler: shader.uniform_location(renderer, "u_SpecularEnvSampler"),
                u_brdfLUT: shader.uniform_location(renderer, "u_brdfLUT"),

                u_BaseColorSampler: shader.uniform_location(renderer, "u_BaseColorSampler"),
                u_BaseColorTexCoord: shader.uniform_location(renderer, "u_BaseColorTexCoord"),
                u_BaseColorFactor: shader.uniform_location(renderer, "u_BaseColorFactor"),

                u_NormalSampler: shader.uniform_location(renderer, "u_NormalSampler"),
                u_NormalTexCoord: shader.uniform_location(renderer, "u_NormalTexCoord"),
                u_NormalScale: shader.uniform_location(renderer, "u_NormalScale"),

                u_EmissiveSampler: shader.uniform_location(renderer, "u_EmissiveSampler"),
                u_EmissiveTexCoord: shader.uniform_location(renderer, "u_EmissiveTexCoord"),
                u_EmissiveFactor: shader.uniform_location(renderer, "u_EmissiveFactor"),

                u_MetallicRoughnessSampler: shader.uniform_location(renderer, "u_MetallicRoughnessSampler"),
                u_MetallicRoughnessTexCoord: shader.uniform_location(renderer, "u_MetallicRoughnessTexCoord"),
                u_MetallicRoughnessValues: shader.uniform_location(renderer, "u_MetallicRoughnessValues"),

                u_OcclusionSampler: shader.uniform_location(renderer, "u_OcclusionSampler"),
                u_OcclusionTexCoord: shader.uniform_location(renderer, "u_OcclusionTexCoord"),
                u_OcclusionStrength: shader.uniform_location(renderer, "u_OcclusionStrength"),

                u_AlphaBlend: shader.uniform_location(renderer, "u_AlphaBlend"),
                u_AlphaCutoff: shader.uniform_location(renderer, "u_AlphaCutoff"),

                u_ScaleDiffBaseMR: shader.uniform_location(renderer, "u_ScaleDiffBaseMR"),
                u_ScaleFGDSpec: shader.uniform_location(renderer, "u_ScaleFGDSpec"),
                u_ScaleIBLAmbient: shader.uniform_location(renderer, "u_ScaleIBLAmbient"),
            };

            shader.use_program(renderer);
            shader.set_int(renderer, uniforms.u_BaseColorSampler, 0);
            shader.set_int(renderer, uniforms.u_NormalSampler, 1);
            shader.set_int(renderer, uniforms.u_EmissiveSampler, 2);
            shader.set_int(renderer, uniforms.u_MetallicRoughnessSampler, 3);
            shader.set_int(renderer, uniforms.u_OcclusionSampler, 4);

            shader.set_vec3(renderer, uniforms.u_LightColor, 5.0, 5.0, 5.0);
            // TODO!: optional minus on z
            shader.set_vec3(renderer, uniforms.u_LightDirection, 0.0, 0.5, 0.5);

            shader.set_vec3(renderer, uniforms.u_AmbientLightColor, 1.0, 1.0, 1.0);
            shader.set_float(renderer, uniforms.u_AmbientLightIntensity, 0.2);

            uniforms
        };

        Self {
            shader,
            flags,
            uniforms
        }
    }
}
