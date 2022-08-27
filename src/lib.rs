mod render;
mod update;
mod utils;

use js_sys::Date;
use std::cell::RefCell;
use std::rc::Rc;
use update::Context;
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::window;
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;
use web_sys::HtmlImageElement;
use web_sys::ImageData;
// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    set_panic_hook();
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = Rc::new(RefCell::new(
        document
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()?,
    ));
    let element = document.document_element().unwrap();
    canvas
        .borrow_mut()
        .set_height(element.client_height() as u32 - 20);
    canvas
        .borrow_mut()
        .set_width(element.client_width() as u32 - 20);

    {
        let canvas = canvas.clone();
        let closure = Closure::<dyn FnMut() -> Result<(), JsValue>>::new(move || {
            let document = web_sys::window().unwrap().document().unwrap();

            let element = document.document_element().unwrap();
            canvas
                .borrow_mut()
                .set_height(element.client_height() as u32 - 20);
            canvas
                .borrow_mut()
                .set_width(element.client_width() as u32 - 20);
            Ok(())
        });
        web_sys::window()
            .unwrap()
            .set_onresize(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
    let context = Rc::new(
        canvas
            .borrow_mut()
            .get_context("webgl2")?
            .unwrap()
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .unwrap(),
    );
    let image = Rc::new(web_sys::HtmlImageElement::new()?);
    image.set_src("corner.png");
    let world_context = Rc::new(RefCell::new(Context::new()));
    {
        let mut world_context = world_context.borrow_mut();
        world_context.program = Some(Context::get_program(&context).unwrap());
        world_context.map_uniform_index =
            context.get_uniform_location(&world_context.program.as_ref().unwrap(), "map");
        world_context.texture_uniform_index =
            context.get_uniform_location(&world_context.program.as_ref().unwrap(), "atlas");
        world_context.window_size_uniform_index =
            context.get_uniform_location(&world_context.program.as_ref().unwrap(), "window_size");
    }
    {
        let context = context.clone();
        let world_context = world_context.clone();
        let new_image = image.clone();
        let closure = Closure::<dyn FnMut() -> Result<(), JsValue>>::new(move || {
            let result = on_load_image(&new_image)?;
            world_context.borrow_mut().set_image(result, &context);
            Ok(())
        });
        image.set_onload(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
    {
        let canvas = canvas.clone();
        let context = context.clone();
        let world_context: Rc<RefCell<Context>> = world_context.clone();
        let callback: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
        let g = callback.clone();
        let start_time = Date::new_0().get_time();
        *g.borrow_mut() = Some(Closure::<dyn FnMut()>::new(move || {
            // let world_context: &Context = world_context.borrow();
            let date = Date::new_0();
            let changed_pixel = world_context
                .borrow_mut()
                .update(date.get_time() - start_time);
            world_context.borrow_mut().render(
                &context,
                changed_pixel,
                canvas.borrow().width(),
                canvas.borrow().height(),
            );
            window()
                .unwrap()
                .request_animation_frame(
                    callback.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
                )
                .unwrap();
        }));

        window()
            .unwrap()
            .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;
    }

    Ok(())
}

pub fn on_load_image(image: &HtmlImageElement) -> Result<ImageData, JsValue> {
    let document = window().unwrap().document().unwrap();

    let canvas2 = document.create_element("canvas")?;
    canvas2.set_class_name("canvas");
    let canvas2 = canvas2.dyn_into::<HtmlCanvasElement>().unwrap();

    canvas2.set_width(image.width());
    canvas2.set_height(image.height());
    let context = canvas2
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    context.draw_image_with_html_image_element(image, 0., 0.)?;
    let bytes = context.get_image_data(0., 0., image.width() as f64, image.height() as f64)?;
    Ok(bytes)
}

#[wasm_bindgen]
pub fn greet() {}
