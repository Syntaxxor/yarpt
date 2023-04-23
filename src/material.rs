use lerp::Lerp;
use cgmath::{ElementWise, InnerSpace};
use rand::{Rng, thread_rng};
use rand::distributions::Bernoulli;
use crate::transform::{Point, Vector};

pub trait Material {
    fn hit_surface(&self, ray_orig: &mut Point, ray_dir: &mut Vector, hit_point: Point, normal: Vector) -> (Vector, Vector);
}

#[derive(Debug, Copy, Clone)]
pub struct PhysicalMaterial {
    pub diffuse: Vector,
    pub roughness: f64,
    pub metallic: f64,
    pub emissive: f64,
}

impl PhysicalMaterial {
    pub fn new(diffuse: Vector, roughness: f64, metallic: f64, emissive: f64) -> Self {
        Self {
            diffuse,
            roughness,
            metallic,
            emissive,
        }
    }
}

impl Material for PhysicalMaterial {
    fn hit_surface(&self, ray_orig: &mut Point, ray_dir: &mut Vector, hit_point: Point, normal: Vector) -> (Vector, Vector) {
        let mut rng = thread_rng();
        
        let mut rng_iter = rng.clone().sample_iter::<f64, _>(rand_distr::StandardNormal);
        
        let mut diffuse_direction = Vector::new(
            rng_iter.next().unwrap(),
            rng_iter.next().unwrap(),
            rng_iter.next().unwrap(),
        ).normalize().add_element_wise(normal).normalize();
        
        let mut reflect_dir = (*ray_dir - 2.0 * normal * (ray_dir.dot(normal))).normalize();
        
        reflect_dir = reflect_dir.lerp(diffuse_direction, self.roughness).normalize();
        
        let is_diffuse_ray = rng.sample(Bernoulli::new((self.roughness * 0.5 + 0.5).lerp(1.0, self.metallic)).unwrap());

        *ray_orig = hit_point;
        if is_diffuse_ray {
            diffuse_direction = diffuse_direction.lerp(reflect_dir, self.metallic).normalize();
            *ray_dir = diffuse_direction;
            (self.diffuse, self.diffuse * self.emissive)
        }else {
            *ray_dir = reflect_dir;
            (Vector::new(1.0, 1.0, 1.0), self.diffuse * self.emissive)
        }
    }
}

impl Default for PhysicalMaterial {
    fn default() -> Self {
        Self {
            diffuse: Vector::new(1.0, 1.0, 1.0),
            roughness: 1.0,
            metallic: 0.0,
            emissive: 0.0,
        }
    }
}