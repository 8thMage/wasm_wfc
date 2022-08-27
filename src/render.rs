use super::update::Context;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

impl Context {
    pub fn get_program(webgl_context: &WebGl2RenderingContext) -> Result<WebGlProgram, String> {
        let pixel_shader = Self::get_pixel_shader(webgl_context)?;
        let vertex_shader = Self::get_vertex_shader(webgl_context)?;
        let program = webgl_context.create_program().unwrap();
        webgl_context.attach_shader(&program, &pixel_shader);
        webgl_context.attach_shader(&program, &vertex_shader);
        webgl_context.link_program(&program);
        if webgl_context
            .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(program)
        } else {
            Err(webgl_context
                .get_program_info_log(&program)
                .unwrap_or(String::from("walla lo yodea")))
        }
    }

    pub fn get_pixel_shader(webgl_context: &WebGl2RenderingContext) -> Result<WebGlShader, String> {
        Self::compile_shader(
            webgl_context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            r##"#version 300 es
        precision highp float;
        uniform mediump sampler2D atlas;
        uniform mediump usampler2D map;
        uniform mediump uvec2 window_size;
        out vec4 outColor;
        void main() {
            uint min_size = min(window_size.x, window_size.y);
            vec2 preOutPosition = (gl_FragCoord.xy - vec2(window_size)*0.5 + vec2(min_size)*0.5) / vec2(min_size);
            if (any(greaterThanEqual(preOutPosition,vec2(1.0))) || any(lessThan(preOutPosition, vec2(0.)))) {
                outColor = vec4(0,0,0,0);
                return;
            }
            vec2 outPosition = vec2(preOutPosition.x, 1.0 - preOutPosition.y);
            uint map_entry = texture(map, outPosition).x;
            if (map_entry == uint(0)) {
                outColor = vec4(0.9,0.9,0.9,1.);
                return;
            }
            vec2 size = vec2(textureSize(map,0));
            vec2 position = mod(outPosition.xy * size, 1.0);
            uint rotation = (map_entry - uint(1))%uint(4);
            if (rotation == uint(0)){
                outColor = texture(atlas, position);
            } else if (rotation == uint(1)) {
                outColor = texture(atlas, vec2(1.-position.y, position.x));
            }
            else if (rotation == uint(2)) {
                outColor = texture(atlas, vec2(1.-position.x, 1.-position.y));
            }
            else if (rotation == uint(3)) {
                outColor = texture(atlas, vec2(position.y, 1.-position.x));
            }
        }
        "##,
        )
    }

    pub fn get_vertex_shader(
        webgl_context: &WebGl2RenderingContext,
    ) -> Result<WebGlShader, String> {
        Self::compile_shader(
            webgl_context,
            WebGl2RenderingContext::VERTEX_SHADER,
            r##"#version 300 es
        precision highp float;
        uniform mediump vec2 screen_size;
        void main() {
            vec4 position = vec4(float(gl_VertexID % 2 * 2 - 1), float(gl_VertexID / 2 * 2 - 1),1,1);
            gl_Position = position;
        }
        "##,
        )
    }

    fn compile_shader(
        webgl_context: &WebGl2RenderingContext,
        shader_type: u32,
        shader_str: &str,
    ) -> Result<WebGlShader, String> {
        let shader = webgl_context.create_shader(shader_type).unwrap();
        webgl_context.shader_source(&shader, shader_str);
        webgl_context.compile_shader(&shader);
        if webgl_context
            .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(shader)
        } else {
            Err(webgl_context
                .get_shader_info_log(&shader)
                .unwrap_or_else(|| String::from("unknown error creating shader")))
        }
    }
}
