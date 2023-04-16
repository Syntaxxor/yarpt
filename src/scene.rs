use std::sync::Arc;
use cgmath::{ElementWise, InnerSpace, MetricSpace, Vector2, VectorSpace, Zero};
use crate::camera::*;
use crate::material::{Material, PhysicalMaterial};
use crate::renderable::Renderable;
use crate::transform::*;

pub struct Scene {
    pub camera: Camera,
    pub sky: Sky,
    objects: Vec<Renderable>,
}

impl Scene {
    pub fn new(camera: Camera, sky: Sky) -> Self {
        Self{
            camera,
            sky,
            objects: Vec::new(),
        }
    }
    
    pub fn add_object(&mut self, object: Renderable) {
        self.objects.push(object);
    }
    
    pub fn trace_pixel(&self, x_coord: f64, y_coord: f64, num_bounces: usize) -> Vector {
        let (mut ray_orig, mut ray_dir) = self.camera.get_ray(Vector2::new(x_coord, y_coord));
        
        let mut diffuse = Vector::new(1.0, 1.0, 1.0);
        let mut lighting = Vector::new(0.0, 0.0, 0.0);
        
        for _i in 0..num_bounces {
            if let Some((hit_point, normal, material)) = self.trace_scene(ray_orig, ray_dir) {
                //return normal;
                let (hit_diffuse, hit_emissive) = material.hit_surface(&mut ray_orig, &mut ray_dir, hit_point, normal);
                diffuse.mul_assign_element_wise(hit_diffuse);
                lighting.add_assign_element_wise(hit_emissive.mul_element_wise(diffuse));
            } else {
                lighting.add_assign_element_wise(self.sky.get_sky_color(ray_dir).mul_element_wise(diffuse));
                break;
            }
        }
        
        return lighting;
    }
    
    fn trace_scene(&self, ray_orig: Point, ray_dir: Vector) -> Option<(Point, Vector, &PhysicalMaterial)> {
        let mut res = None;
        let mut closest = 0.0;

        for object in &self.objects {
            if let Some((hit_point, normal)) = object.trace(ray_orig, ray_dir) {
                let dist = hit_point.distance(ray_orig);
                if res.is_none() || dist < closest {
                    res = Some((hit_point, normal, object.get_material()));
                    closest = dist;
                }
            }
        }
        
        res
    }
}


pub struct Sky {
    sun_dir: Vector,
    pub sun_size: f64,
    pub sun_color: Vector,
    pub ground_color: Vector,
    pub horizon_color: Vector,
    pub zenith_color: Vector,
}

impl Sky {
    pub fn new(sun_dir: Vector, sun_size: f64, sun_color: Vector, ground_color: Vector, horizon_color: Vector, zenith_color: Vector) -> Self {
        Self {
            sun_dir: sun_dir.normalize(),
            sun_size,
            sun_color,
            ground_color,
            horizon_color,
            zenith_color,
        }
    }
    
    pub fn black() -> Self {
        Self::new(
            Vector::unit_x(),
            0.0,
            Vector::zero(),
            Vector::zero(),
            Vector::zero(),
            Vector::zero(),
        )
    }
    
    pub fn set_sun_dir(&mut self, sun_dir: Vector) {
        self.sun_dir = sun_dir.normalize();
    }
    
    pub fn get_sky_color(&self, ray_dir: Vector) -> Vector {
        if ray_dir.dot(self.sun_dir) > (1.0 - self.sun_size) {
            return self.sun_color;
        }
        let sky_gradient_t = smoothstep(0.0, 0.4, ray_dir.y).powf(0.35);
        let sky_gradient = self.horizon_color.lerp(self.zenith_color, sky_gradient_t);
        let ground_to_sky_t = smoothstep(-0.01, 0.0, ray_dir.y);
        
        self.ground_color.lerp(sky_gradient, ground_to_sky_t)
    }
}

impl Default for Sky {
    fn default() -> Self {
        Sky::new(
            Vector::unit_y(),
            0.01,
            Vector::new(8.0, 8.0, 8.0),
            Vector::new(0.5, 0.5, 0.5),
            Vector::new(1.0, 1.0, 1.0),
            Vector::new(0.8, 0.8, 1.0),
        )
    }
}

fn smoothstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}
