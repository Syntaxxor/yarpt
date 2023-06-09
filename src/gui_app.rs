use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{channel};
use crate::path_tracer::*;

use fltk::{app, prelude::*, window::Window};
use fltk::button::{Button, CheckButton};
use fltk::dialog::file_chooser;
use fltk::draw::{draw_image, draw_rect_fill};
use fltk::enums::{Color, ColorDepth};
use fltk::frame::Frame;
use fltk::group::{Pack, PackType, Scroll, ScrollType};
use fltk::input::{Input, IntInput};
use fltk::surface::ImageSurface;
use image::ColorType;
use crate::scene::{Sky};
use crate::scene_loader;
use crate::transform::*;

pub fn run (){
    let app = app::App::default();
    let mut wind = Window::default()
        .with_size(800, 600)
        .center_screen()
        .with_label("Yet Another Rust Path Tracer");
    
    let mut scroll = Scroll::default().size_of_parent();
    scroll.set_type(ScrollType::Both);
    scroll.set_clip_children(false);
    
    let mut input_pack = Pack::default().size_of_parent();
    input_pack.set_type(PackType::Vertical);
    input_pack.set_clip_children(false);
    
    let mut scene_path_input = Input::default()
        .with_size(100, 30)
        .with_label("Scene");
    scene_path_input.set_value("test_scene.ypt");
    
    let mut w_input = IntInput::default()
        .with_size(100, 30)
        .with_label("Width");
    w_input.set_value("480");
    
    let mut h_input = IntInput::default()
        .with_size(1, 30)
        .with_label("Height");
    h_input.set_value("360");

    let mut sample_input = IntInput::default()
        .with_size(1, 30)
        .with_label("Samples");
    sample_input.set_value("256");

    let mut bounces_input = IntInput::default()
        .with_size(1, 30)
        .with_label("Bounces");
    bounces_input.set_value("8");

    let mut tile_size_input = IntInput::default()
        .with_size(1, 30)
        .with_label("Tile Size");
    tile_size_input.set_value("64");

    let mut threads_input = IntInput::default()
        .with_size(1, 30)
        .with_label("Threads");
    threads_input.set_value("6");
    
    let mut denoise_checkbox = CheckButton::default()
        .with_size(1, 30)
        .with_label("Denoise");
    denoise_checkbox.set_value(true);
    
    let mut render_button = Button::default()
        .with_size(100, 30)
        .with_label("Render");
    
    let mut save_button = Button::default()
        .with_size(100, 30)
        .with_label("Save Rendered Image");
    save_button.deactivate();
    
    let mut render_result = Frame::default();
    
    input_pack.end();
    scroll.end();
    
    wind.end();
    wind.make_resizable(true);
    
    wind.resize_callback({
        let mut scroll = scroll.clone();
        let mut input_pack = input_pack.clone();
        move|_, _, _, w, h| {
            scroll.set_size(w, h);
            input_pack.resize(64, 0, w - 64, h);
        }
    });
    
    let (render_sender, render_receiver) = app::channel::<RenderMessages>();
    
    let (thread_sender, thread_receiver) = channel();

    render_button.set_callback({
        let mut render_result = render_result.clone();
        let mut save_button = save_button.clone();
        let scene_path_input = scene_path_input.clone();
        let w_input = w_input.clone();
        let h_input = h_input.clone();
        let sample_input = sample_input.clone();
        let bounces_input = bounces_input.clone();
        let tile_size_input = tile_size_input.clone();
        let threads_input = threads_input.clone();
        let denoise_checkbox = denoise_checkbox.clone();
        move |render_button| {
            let scene_path = scene_path_input.value();
            let width = w_input.value().parse().unwrap();
            let height = h_input.value().parse().unwrap();
            let samples = sample_input.value().parse().unwrap();
            let bounces = bounces_input.value().parse().unwrap();
            let tile_size = tile_size_input.value().parse().unwrap();
            let num_threads = threads_input.value().parse().unwrap();
            let denoise = denoise_checkbox.value();
            
            render_result.set_size(width as i32, height as i32);
            render_button.deactivate();
            save_button.deactivate();
            
            let mut sky = Sky::default();
            sky.set_sun_dir(Vector::new(1.0, 2.0, 1.5));
            
            let mut scene = scene_loader::load(scene_path).unwrap();
            
            println!("{}", scene.get_object_count());
            //scene.sky = sky;
            
            let render_settings = RenderSettings{
                width,
                height,
                samples,
                bounces,
                tile_size,
                denoise,
            };
            
            let path_tracer = PathTracer::new(
                render_settings,
                scene,
                num_threads,
            );
            
            let render_sender = render_sender.clone();
            
            render_sender.send(RenderMessages::StartRender(width, height));
            thread_sender.send(std::thread::spawn(move||{path_tracer.clone().render(render_sender);})).unwrap();
        }
    });
    
    let mut render_surface = ImageSurface::new(800, 600, false);
    ImageSurface::push_current(&render_surface);
    draw_rect_fill(0, 0, 800, 600, Color::White);
    ImageSurface::pop_current();
    
    let mut surf = Rc::from(RefCell::from(render_surface));
    
    wind.show();
    while app.wait() {
        if let Some(msg) = render_receiver.recv() {
            match msg {
                RenderMessages::StartRender(width, height) => {
                    render_surface = ImageSurface::new(width as i32, height as i32, false);
                    surf = Rc::from(RefCell::from(render_surface));
                    render_result.draw({
                        let surf = surf.clone();
                        move |f| {
                            let surf = surf.borrow();
                            surf.image().unwrap().draw(f.x(), f.y(), f.w(), f.h());
                        }
                    })
                }
                RenderMessages::UpdateRender(x, y, tile_size, image) => {
                    let mut data: Vec<u8> = Vec::new();
                    data.resize(image.len() * 3, 0);
                    for i in 0..image.len() {
                        for j in 0..3 {
                            data[i * 3 + j] = (image[i][j] * 255.0) as u8;
                        }
                    }
                    
                    ImageSurface::push_current(&surf.borrow());
                    draw_image(
                        data.as_slice(), 
                        x as i32, 
                        y as i32, 
                        tile_size as i32, 
                        tile_size as i32, 
                        ColorDepth::Rgb8
                    ).unwrap();
                    ImageSurface::pop_current();
                    
                    app.redraw();
                }
                RenderMessages::FinishRender(width, height, image) => {
                    if let Some(image) = image {
                        let mut res_data: Vec<u8> = Vec::new();
                        res_data.resize(width * height * 3, 0);
                        for i in 0..image.len() {
                            for j in 0..3 {
                                res_data[i * 3 + j] = (image[i][j] * 255.0) as u8;
                            }
                        }

                        ImageSurface::push_current(&surf.borrow());
                        draw_image(
                            res_data.as_slice(), 
                            0, 
                            0, 
                            width as i32, 
                            height as i32, 
                            ColorDepth::Rgb8
                        ).unwrap();
                        ImageSurface::pop_current();
                        app.redraw();
                    }
                    
                    render_button.activate();
                    save_button.activate();
                    save_button.set_callback({
                        let surf = surf.clone();
                        move |_| {
                            let img = surf.borrow().image().unwrap();
                            let image_data = img.to_rgb_data();
                            let image_width = width as u32;
                            let image_height = height as u32;
                            if let Some(file_name) = file_chooser("Save image as...", "*.png", ".", false) {
                                image::save_buffer(file_name, image_data.as_slice(), image_width, image_height, ColorType::Rgb8).unwrap();
                            }
                        }
                    });
                    thread_receiver.recv().unwrap().join().unwrap();
                    println!("Joined!");
                }
            }
        }
    }
}
