use std::ops::DivAssign;
use crate::scene::*;

use threadpool::ThreadPool;

use std::sync::mpsc::{channel};
use std::sync::Arc;
use std::time::{Instant};
use cgmath::ElementWise;
use fltk::app::{Sender};
use rand::{Rng, thread_rng};
use crate::transform::Vector;

pub type Pixel = [f64; 3];

pub enum RenderMessages{
    StartRender(usize, usize),
    UpdateRender(usize, usize, usize, Vec<Pixel>),
    FinishRender(usize, usize, Option<Vec<Pixel>>),
}

#[derive(Debug, Copy, Clone)]
pub struct RenderSettings{
    pub width: usize,
    pub height: usize,
    pub samples: usize,
    pub bounces: usize,
    pub tile_size: usize,
    pub denoise: bool,
}

#[derive(Clone)]
pub struct PathTracer {
    render_settings: RenderSettings,
    image: Vec<Pixel>,
    scene: Arc<Scene>,
    num_threads: usize,
}

impl PathTracer {
    pub fn new(render_settings: RenderSettings, scene: Scene, num_threads: usize) -> Self {
        Self {
            render_settings,
            image: Vec::new(),
            scene: Arc::new(scene),
            num_threads,
        }
    }
    
    pub fn render(&mut self, sender: Sender<RenderMessages>) {
        let render_start = Instant::now();
        self.image.resize(self.render_settings.width * self.render_settings.height, [0.0; 3]);
        
        let pool = ThreadPool::new(self.num_threads);
        
        let render_settings = self.render_settings;
        
        let mut created = 0;
        let (tx, rx) = channel();
        for x in (0..render_settings.width).step_by(render_settings.tile_size) {
            for y in (0..render_settings.height).step_by(render_settings.tile_size) {
                let tx = tx.clone();
                let scene = self.scene.clone();
                created += 1;
                pool.execute(move || {
                    tx.send(render_region(scene, x, y, render_settings)).expect("Render job failed.")
                });
            }
        }
        
        println!("{}", created);
        
        let mut finished = 0;
        loop {
            let received = rx.recv();
            if received.is_err() {
                break;
            }
            
            finished += 1;
            
            let (x, y, pixels) = received.unwrap();
            
            for tx in 0..self.render_settings.tile_size {
                for ty in 0..self.render_settings.tile_size {
                    self.set_pixel(x + tx, y + ty, pixels[ty * self.render_settings.tile_size + tx]);
                }
            }
            
            let is_done = finished == created;

            sender.send(RenderMessages::UpdateRender(x, y, self.render_settings.tile_size, pixels));
            
            println!("{}", finished);
            if is_done {
                pool.join();
                if self.render_settings.denoise {
                    let device = oidn::Device::new();
                    
                    let mut denoise_data: Vec<f32> = Vec::new();
                    denoise_data.resize(self.render_settings.width * self.render_settings.height * 3, 0.0);
                    for i in 0..self.image.len() {
                        for j in 0..3 {
                            denoise_data[i * 3 + j] = self.image[i][j] as f32;
                        }
                    }
                    
                    oidn::RayTracing::new(&device)
                        .image_dimensions(self.render_settings.width, self.render_settings.height)
                        .hdr(true)
                        .filter_in_place(denoise_data.as_mut_slice())
                        .expect("Denoising error.");
                    
                    for i in 0..self.image.len() {
                        for j in 0..3 {
                            self.image[i][j] = denoise_data[i * 3 + j] as f64;
                        }
                    }
                    
                    sender.send(RenderMessages::FinishRender(
                        self.render_settings.width, 
                        self.render_settings.height, 
                        Some(self.image.clone())
                    ));
                } else {
                    sender.send(RenderMessages::FinishRender(
                        self.render_settings.width, 
                        self.render_settings.height, 
                        None
                    ));
                }
                let render_end = Instant::now();
                let render_time = render_end - render_start;
                println!("Finished render in {} seconds.", render_time.as_secs_f64());
                break;
            }
        }
    }
    
    fn set_pixel(&mut self, x: usize, y: usize, pixel: Pixel) {
        if x < self.render_settings.width && y < self.render_settings.height {
            self.image[y * self.render_settings.width + x] = pixel;
        }
    }
}

fn render_region(scene: Arc<Scene>, x: usize, y: usize, render_settings: RenderSettings) -> (usize, usize, Vec<Pixel>) {
    let mut result = Vec::new();
    result.resize(render_settings.tile_size * render_settings.tile_size, [0.0; 3]);

    let mut rng = thread_rng();
    
    let aspect = render_settings.width as f64 / render_settings.height as f64;
    
    for tx in 0..render_settings.tile_size {
        for ty in 0..render_settings.tile_size {
            let mut col = Vector::new(0.0, 0.0, 0.0);
            for _i in 0..render_settings.samples {
                let x = x + tx;
                let y = y + ty;
                let x_coord = (((x as f64 + rng.gen::<f64>()) / render_settings.width as f64) * 2.0 - 1.0) * aspect;
                let y_coord = (1.0 - (y as f64 + rng.gen::<f64>()) / render_settings.height as f64) * 2.0 - 1.0;
                
                let lighting = scene.trace_pixel(x_coord, y_coord, render_settings.bounces);
                col.add_assign_element_wise(lighting);
            }
            col.div_assign(render_settings.samples as f64);
            result[ty * render_settings.tile_size + tx] = [col.x, col.y, col.z];
        }
    }
    
    println!("Finished rendering region: ({}, {})", x, y);
    (x, y, result)
}