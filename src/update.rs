use crate::log;
use web_sys::{
    ImageData, WebGl2RenderingContext, WebGlProgram, WebGlTexture, WebGlUniformLocation,
};
pub struct Context {
    pub image: Option<ImageData>,
    pub texture: Option<WebGlTexture>,
    pub map: Vec<u8>,
    pub map_height: u32,
    pub map_width: u32,
    pub map_texture: Option<WebGlTexture>,
    pub program: Option<WebGlProgram>,
    pub texture_uniform_index: Option<WebGlUniformLocation>,
    pub map_uniform_index: Option<WebGlUniformLocation>,
    pub window_size_uniform_index: Option<WebGlUniformLocation>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            image: None,
            texture: None,
            map: vec![],
            map_height: 32,
            map_width: 32,
            map_texture: None,
            program: None,
            map_uniform_index: None,
            texture_uniform_index: None,
            window_size_uniform_index: None,
        }
    }

    pub fn set_image(&mut self, image: ImageData, webgl_context: &WebGl2RenderingContext) {
        let texture = webgl_context.create_texture().unwrap();
        webgl_context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
        webgl_context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        webgl_context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );

        webgl_context
            .tex_image_2d_with_u32_and_u32_and_image_data(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::RGBA as i32,
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                &image,
            )
            .unwrap();
        self.image = Some(image);
        self.texture = Some(texture);

        let mut map = vec![];
        map.resize((self.map_height * self.map_width) as usize, 0);
        self.map = map;
        let map_texture = webgl_context.create_texture().unwrap();
        webgl_context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&map_texture));
        webgl_context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        webgl_context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        webgl_context
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::R8UI as i32,
                self.map_width as i32,
                self.map_height as i32,
                0,
                WebGl2RenderingContext::RED_INTEGER,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                Some(&self.map[0..]),
            )
            .unwrap();
        self.map_texture = Some(map_texture);
    }

    pub fn render(
        &mut self,
        webgl_context: &WebGl2RenderingContext,
        window_width: u32,
        window_height: u32,
        frame_number: u64,
    ) {
        if self.texture.is_some() {
            let changed_pixel = frame_number % (self.map_width * self.map_height *12) as u64/12;
            log!("changed pixel {}", changed_pixel);
            // let changed_pixel = 0;
            if frame_number % 12 == 0 {
                self.map[changed_pixel as usize] = (self.map[changed_pixel as usize] + 1) % 4;
                log!("changed pixel value {}", self.map[changed_pixel as usize]);
            }
            webgl_context.use_program(self.program.as_ref());
            webgl_context.uniform1i(self.map_uniform_index.as_ref(), 1);
            webgl_context.uniform1i(self.texture_uniform_index.as_ref(), 0);
            webgl_context.uniform2ui(
                self.window_size_uniform_index.as_ref(),
                window_width,
                window_height,
            );
            webgl_context.active_texture(WebGl2RenderingContext::TEXTURE0);
            webgl_context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, self.texture.as_ref());
            webgl_context.active_texture(WebGl2RenderingContext::TEXTURE1);
            webgl_context.bind_texture(
                WebGl2RenderingContext::TEXTURE_2D,
                self.map_texture.as_ref(),
            );
            webgl_context
                .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
                    WebGl2RenderingContext::TEXTURE_2D,
                    0,
                    (changed_pixel % self.map_width as u64) as i32,
                    (changed_pixel / self.map_width as u64) as i32,
                    1,
                    1,
                    WebGl2RenderingContext::RED_INTEGER,
                    WebGl2RenderingContext::UNSIGNED_BYTE,
                    Some(&self.map[changed_pixel as usize..]),
                )
                .unwrap();

            webgl_context.draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);
        }
    }
}
