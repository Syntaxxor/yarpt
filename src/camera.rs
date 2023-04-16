use std::f64::consts::TAU;
use crate::transform::*;
use cgmath::{InnerSpace, Rotation, Vector2};
use rand::prelude::*;
use rand_distr::StandardNormal;

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pub transform: Transform,
    pub focal_length: f64,
    pub focal_plane: f64,
    pub f_stop: f64,
}

impl Camera {
    pub fn get_ray(&self, coord: Vector2<f64>) -> (Point, Vector) {
        let mut rng = thread_rng();
        
        let rand_angle = rng.gen::<f64>() * TAU;
        let rand_off = rng.gen::<f64>();
        
        let rand_x = rand_angle.cos() * rand_off;
        let rand_y = rand_angle.sin() * rand_off;
        
        let basis = self.transform.get_basis();
        
        let mut ray_orig = self.transform.position;
        
        let mut ray_dir = basis.rotate_vector(Vector::new(coord.x, coord.y, self.focal_length).normalize());
        
        let target_point = ray_orig + ray_dir * self.focal_plane;
        
        let offset = Vector::new(rand_x, rand_y, 0.0) * 2.0;
        
        ray_orig += basis.rotate_vector(offset * self.f_stop);
        
        ray_dir = (target_point - ray_orig).normalize();
        
        (ray_orig, ray_dir)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            focal_length: 1.0,
            focal_plane: 1.0,
            f_stop: 0.0,
        }
    }
}
