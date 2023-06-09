use cgmath::{ElementWise, EuclideanSpace, InnerSpace};
use crate::material::{PhysicalMaterial};
use crate::transform::*;

#[derive(Debug, Copy, Clone)]
pub struct Renderable {
    pub transform: Transform,
    pub material: PhysicalMaterial,
    pub shape: RenderShape,
}

impl Renderable {
    pub fn new(transform: Transform, material: PhysicalMaterial, shape: RenderShape) -> Self {
        Self {
            transform,
            material,
            shape,
        }
    }
    
    pub fn new_sphere(transform: Transform, material: PhysicalMaterial, radius: f64) -> Self {
        Self {
            transform,
            material,
            shape: RenderShape::Sphere(radius),
        }
    }

    pub fn new_box(transform: Transform, material: PhysicalMaterial, bounds: Vector) -> Self {
        Self {
            transform,
            material,
            shape: RenderShape::Box(bounds),
        }
    }
    
    // Returns hit position and hit normal.
    pub fn trace(&self, ray_orig: Point, ray_dir: Vector) -> Option<(Point, Vector)> {
        let ray_orig = self.transform.to_local_point(ray_orig);
        let ray_dir = self.transform.to_local_vector(ray_dir);
        
        if let Some((hit_position, normal)) = self.shape.trace(ray_orig, ray_dir){
            Some((self.transform.to_global_point(hit_position), self.transform.to_global_vector(normal)))
        } else {
            None
        }
    }
    
    pub fn get_aabb(&self) -> (Point, Vector) {
        let mut min = Point::new(f64::MAX, f64::MAX, f64::MAX);
        let mut max = Point::new(f64::MIN, f64::MIN, f64::MIN);
        
        let points = self.shape.get_box_points();
        
        for p in points {
            let p = self.transform.to_global_point(p);
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            min.z = min.z.min(p.z);
            
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
            max.z = max.z.max(p.z);
        }

        (max.add_element_wise(min) / 2.0, max.sub_element_wise(min).to_vec() / 2.0)
    }
    
    pub fn get_material(&self) -> &PhysicalMaterial {
        &self.material
    }
}

#[derive(Debug, Copy, Clone)]
pub enum RenderShape {
    None,
    Sphere(f64),
    Box(Vector),
}

impl RenderShape {
    pub fn trace(&self, ray_orig: Point, ray_dir: Vector) -> Option<(Point, Vector)> {
        match self {
            RenderShape::None => None,
            RenderShape::Sphere(radius) => RenderShape::trace_sphere(*radius, ray_orig, ray_dir),
            RenderShape::Box(bounds) => RenderShape::trace_box(*bounds, ray_orig, ray_dir),
        }
    }
    
    fn get_box_points(&self) -> [Point; 8] {
        match self {
            RenderShape::None => [Point::new(0.0, 0.0, 0.0); 8],
            RenderShape::Sphere(radius) => RenderShape::sphere_points(*radius),
            RenderShape::Box(size) => RenderShape::box_points(*size),
        }
    }
    
    fn trace_sphere(radius: f64, ray_orig: Point, ray_dir: Vector) -> Option<(Point, Vector)> {
        if ray_orig.dot(ray_dir) > 0.0 {
            return None;
        }
        
        let a = ray_dir.dot(ray_dir);
        let b = 2.0 * ray_orig.dot(ray_dir);
        let c = ray_orig.dot(ray_orig.to_vec()) - radius * radius;
        if b*b - 4.0*a*c < 0.0 {
            return None;
        }
        
        let dist = (-b - (b*b-4.0*a*c).sqrt()) / (2.0*a);
        let hit_point = ray_orig + ray_dir * dist;
        let normal = hit_point.to_vec().normalize();
        
        Some((hit_point, normal))
    }
    
    fn trace_box(bounds: Vector, ray_orig: Point, ray_dir: Vector) -> Option<(Point, Vector)> {
        let b_min = Point::from_vec(-bounds);
        let b_max = Point::from_vec(bounds);
        if ray_orig.x > b_min.x && ray_orig.y > b_min.y && ray_orig.z > b_min.z && ray_orig.x < b_max.x && ray_orig.y < b_max.y && ray_orig.z < b_max.z {
            return None;
        }
        let inv_dir = 1.0 / ray_dir;
        
        let t0 = (b_min - ray_orig).mul_element_wise(inv_dir);
        let t1 = (b_max - ray_orig).mul_element_wise(inv_dir);

        let v_min = t0.zip(t1, |a, b| a.min(b));
        let v_max = t0.zip(t1, |a, b| a.max(b));

        let t_min = v_min.x.max(v_min.y.max(v_min.z));
        let t_max = v_max.x.min(v_max.y.min(v_max.z));
        
        if t_max <= t_min || t_min <= 0.0 {
            return None;
        }
        
        let hit_position = ray_orig + ray_dir * t_min;
        
        let normal = {
            if t_min == t0.x {
                -Vector::unit_x()
            } else if t_min == t1.x {
                Vector::unit_x()
            } else if t_min == t0.y {
                -Vector::unit_y()
            } else if t_min == t1.y {
                Vector::unit_y()
            } else if t_min == t0.z {
                -Vector::unit_z()
            } else {
                Vector::unit_z()
            }
        };
        
        Some((hit_position, normal))
    }
    
    fn sphere_points(radius: f64) -> [Point; 8] {
        //let radius = radius / 2.0;
        [
            Point::new(-radius, -radius, -radius),
            Point::new(radius, -radius, -radius),
            Point::new(-radius, radius, -radius),
            Point::new(radius, radius, -radius),
            Point::new(-radius, -radius, radius),
            Point::new(radius, -radius, radius),
            Point::new(-radius, radius, radius),
            Point::new(radius, radius, radius),
        ]
    }
    
    fn box_points(size: Vector) -> [Point; 8] {
        //let size = size / 2.0;
        [
            Point::new(-size.x, -size.y, -size.z),
            Point::new(size.x, -size.y, -size.z),
            Point::new(-size.x, size.y, -size.z),
            Point::new(size.x, size.y, -size.z),
            Point::new(-size.x, -size.y, size.z),
            Point::new(size.x, -size.y, size.z),
            Point::new(-size.x, size.y, size.z),
            Point::new(size.x, size.y, size.z),
        ]
    }
}