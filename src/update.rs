use std::collections::HashSet;
use std::hash::Hasher;
use std::{collections::hash_map::DefaultHasher, iter::repeat};

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
    pub options: Vec<Vec<Vec<bool>>>,
    pub borders_hash: Vec<u64>,
}

impl Context {
    pub fn new() -> Self {
        let width = 60;
        let height = 60;
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
            options: vec![vec![vec![true; 4]; width]; height],
            borders_hash: vec![],
        }
    }

    pub fn set_image(&mut self, image: ImageData, webgl_context: &WebGl2RenderingContext) {
        let texture = webgl_context.create_texture().unwrap();
        webgl_context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
        webgl_context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        webgl_context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );

        webgl_context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::MIRRORED_REPEAT as i32,
        );
        webgl_context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::MIRRORED_REPEAT as i32,
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

        for i in 0..4 {
            self.borders_hash.push(self.get_border_of_image(i));
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
        if time - self.cooldown_start > 100. / 60. * 1. {
            // if time > 300. {
            //     return None;
            // }
            log!("in");
            for y in 0..self.options.len() {
                for x in 0..self.options[0].len() {
                    if self.map[y * self.map_width + x] == 0
                        && self.options[y][x].iter().filter(|b| **b).count() == 1
                    {
                        let position = y * self.map_width + x;
                        // log!("position {}", position);
                        self.map[position] =
                            self.options[y][x].iter().position(|b| *b).unwrap() as u8 + 1;
                        return Some(position);
                    }
                }
            }
            let counts = &mut self.counts;
            let minimum_entropy = self
                .options
                .iter()
                .enumerate()
                .flat_map(|(y, v)| {
                    repeat(y).zip(
                        v.iter()
                            .enumerate()
                            .filter(|(_x, v)| v.iter().filter(|b| **b).count() >= 2)
                            .flat_map(move |(x, v)| {
                                // if y == 0 && x == 0 {
                                //     log!("y {}, x {}, {:?}", y, x, v);
                                // }
                                repeat(x).zip(v.iter().enumerate())
                            }),
                    )
                })
                .filter_map(|(y, (x, (spin, b)))| if *b { Some((y, x, spin)) } else { None })
                .map(|(y, x, spin)| {
                    counts[spin] += 1;
                    let entropy = Self::calculate_entropy(&counts);
                    counts[spin] -= 1;
                    (y, x, spin, entropy)
                })
                .fold(
                    (usize::max_value(), usize::max_value(), 0, -f64::MAX),
                    |(y_acc, x_acc, spin_acc, entropy_acc), (y, x, spin, entropy)| {
                        // log!("entropy {:?}", entropy);
                        if entropy > entropy_acc {
                            (y, x, spin, entropy)
                        } else {
                            (y_acc, x_acc, spin_acc, entropy_acc)
                        }
                    },
                );
            if minimum_entropy.0 == usize::max_value() {
                return None;
            }
            log!(
                "options {:?}",
                self.options[minimum_entropy.0][minimum_entropy.1]
            );
            log!("first_option {:?}", minimum_entropy);
            for i in self.options[minimum_entropy.0][minimum_entropy.1].iter_mut() {
                *i = false;
            }
            self.options[minimum_entropy.0][minimum_entropy.1][minimum_entropy.2] = true;
            counts[minimum_entropy.2] += 1;
            self.branch_out(minimum_entropy.1, minimum_entropy.0);
            return None;
        }
        return None;
    }

    fn calculate_entropy(sum_of_options: &Vec<u64>) -> f64 {
        // log!("sum_of_options {:?}", sum_of_options);

        let sum: u64 = sum_of_options.iter().sum();
        let entropy = sum_of_options
            .iter()
            .map(|option| {
                let p: f64 = *option as f64 / sum as f64;
                if p == 0. {
                    0.
                } else {
                    -p * p.log2()
                }
            })
            .sum();
        entropy
    }

    fn branch_out(&mut self, x: usize, y: usize) {
        let mut changed_cells = HashSet::new();
        changed_cells.insert((x, y));
        while changed_cells.len() != 0 {
            let (x, y) = *changed_cells.iter().next().unwrap();
            changed_cells.remove(&(x, y));
            // log!("changed_cell y {} x {}", y, x);

            if x >= self.options[0].len() || y >= self.options.len() {
                return;
            }
            let offsets: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

            'big_loop: for (orientation, offset) in offsets.iter().enumerate() {
                {
                    // log!("offset {:?} orientation {}", offset, orientation);
                    if y.wrapping_add(offset.0 as usize) >= self.options.len()
                        || x.wrapping_add(offset.1 as usize) >= self.options[0].len()
                    {
                        continue;
                    }
                    let mut filtered_options = (self.options[y][x])
                        .iter()
                        .enumerate()
                        .filter_map(|(spin, option)| if *option { Some(spin) } else { None });
                    let first_hash =
                        self.borders_hash[(filtered_options.next().unwrap() + 4 - orientation) % 4];
                    for spin in filtered_options {
                        if self.borders_hash[(spin + 4 - orientation) % 4] != first_hash {
                            continue 'big_loop;
                        }
                    }

                    for (spin, option) in self.options[y.wrapping_add(offset.0 as usize)]
                        [x.wrapping_add(offset.1 as usize)]
                    .iter_mut()
                    .enumerate()
                    .filter(|(_spin, option)| **option)
                    {
                        if self.borders_hash[(spin + 4 - orientation + 2) % 4] != first_hash {
                            *option = false;
                            changed_cells.insert((
                                x.wrapping_add(offset.1 as usize),
                                y.wrapping_add(offset.0 as usize),
                            ));
                            // log!(
                            //     "insert y {} x {}",
                            //     y.wrapping_add(offset.0 as usize),
                            //     x.wrapping_add(offset.1 as usize)
                            // );
                        }
                    }
                }
            }
        }
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
