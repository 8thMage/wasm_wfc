use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

#[allow(unused)]
use crate::log;
use web_sys::{
    ImageData, WebGl2RenderingContext, WebGlProgram, WebGlTexture, WebGlUniformLocation,
};

pub struct Context {
    pub image: Option<ImageData>,
    pub texture: Option<WebGlTexture>,
    pub map: Vec<u8>,
    pub map_height: usize,
    pub map_width: usize,
    pub map_texture: Option<WebGlTexture>,
    pub program: Option<WebGlProgram>,
    pub texture_uniform_index: Option<WebGlUniformLocation>,
    pub map_uniform_index: Option<WebGlUniformLocation>,
    pub window_size_uniform_index: Option<WebGlUniformLocation>,

    pub cooldown_start: f64,
    pub counts: Vec<u64>,
    pub options: Vec<Vec<bool>>,
    pub possible_connection: Vec<Vec<Vec<bool>>>,
}

impl Context {
    pub fn new() -> Self {
        let width = 32;
        let height = 32;
        Context {
            image: None,
            texture: None,
            map: vec![],
            map_height: height,
            map_width: width,
            map_texture: None,
            program: None,
            map_uniform_index: None,
            texture_uniform_index: None,
            window_size_uniform_index: None,

            cooldown_start: 0.,
            counts: vec![0; 4],
            options: vec![vec![true; 4]; height * width],
            possible_connection: vec![],
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
        self.image = Some(image.clone());
        self.texture = Some(texture);

        for first_orientation in 0..4 {
            self.possible_connection.push(vec![]);
            for second_orientation in 0..4 {
                self.possible_connection.last_mut().unwrap().push(vec![]);
                for dir in 0..4 {
                    let first_border = (first_orientation - dir + 4) % 4;
                    let second_border = (second_orientation - dir + 2 + 4) % 4;
                    let is_possible = self.get_border_of_image(first_border)
                        == self.get_border_of_image(second_border);
                    let updated_correlation = self
                        .possible_connection
                        .last_mut()
                        .unwrap()
                        .last_mut()
                        .unwrap();

                    updated_correlation.push(is_possible);
                }
            }
        }

        self.map = vec![0; self.map_height * self.map_width];
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

    pub fn update(&mut self, time: f64) -> Option<usize> {
        if time - self.cooldown_start > 1000. / 60. * 12. {
            for (idx, i) in self.options[0].iter().enumerate() {
                log!("i {} {}", idx, i);
            }
            for (idx, i) in self.options[1].iter().enumerate() {
                log!("i {} {}", idx, i);
            }
            let position = self
                .options
                .iter()
                .enumerate()
                .position(|(e, v)| self.map[e] == 0 && v.iter().filter(|b| **b).count() == 1);
            if let Some(position) = position {
                log!("position {}", position);
                self.map[position] =
                    self.options[position].iter().position(|b| *b).unwrap() as u8 + 1;
                return Some(position);
            }
            let option_count = self
                .options
                .iter()
                .map(|v| v.iter().filter(|b| **b).count());
            let changer = option_count
                .enumerate()
                .filter(|(_e, p)| *p >= 2)
                .min_by_key(|(_e, v)| *v)
                .map(|(e, _v)| e);
            if let Some(changer) = changer {
                log!("changer {}", changer);
                for i in self.options[changer].iter_mut() {
                    *i = false;
                }
                self.options[changer][changer % 4] = true;
                return None;
            }
        }
        return None;
    }

    pub fn render(
        &mut self,
        webgl_context: &WebGl2RenderingContext,
        changed_pixel: Option<usize>,
        window_width: u32,
        window_height: u32,
    ) {
        if self.texture.is_some() {
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
            if let Some(changed_pixel) = changed_pixel {
                webgl_context
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
                        WebGl2RenderingContext::TEXTURE_2D,
                        0,
                        (changed_pixel % self.map_width) as i32,
                        (changed_pixel / self.map_width) as i32,
                        1,
                        1,
                        WebGl2RenderingContext::RED_INTEGER,
                        WebGl2RenderingContext::UNSIGNED_BYTE,
                        Some(&self.map[changed_pixel as usize..]),
                    )
                    .unwrap();
            }
            webgl_context.draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);
        }
    }

    fn get_border_of_image(&self, border: i32) -> u64 {
        let image = self.image.as_ref().unwrap().clone();
        match border {
            0 => {
                let mut hash = DefaultHasher::new();
                hash.write(
                    &image
                        .data()
                        .chunks(image.width() as usize * 4)
                        .last()
                        .unwrap(),
                );

                hash.finish()
            }
            1 => {
                let mut hash = DefaultHasher::new();
                image
                    .data()
                    .0
                    .chunks(4)
                    .into_iter()
                    .step_by(image.width() as usize)
                    .for_each(|v| hash.write(v));

                hash.finish()
            }
            2 => {
                let mut hash = DefaultHasher::new();
                hash.write(
                    &image
                        .data()
                        .chunks(image.width() as usize * 4)
                        .next()
                        .unwrap(),
                );
                hash.finish()
            }
            3 => {
                let mut hash = DefaultHasher::new();
                image
                    .data()
                    .0
                    .chunks(4)
                    .into_iter()
                    .skip(image.width() as usize - 1)
                    .step_by(image.width() as usize)
                    .for_each(|v| hash.write(v));

                hash.finish()
            }
            _ => 0,
        }
    }
}
