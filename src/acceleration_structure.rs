use std::sync::Arc;
use cgmath::{ElementWise, EuclideanSpace, MetricSpace, Vector2};
use crate::material::{Material, PhysicalMaterial};
use crate::scene::Scene;
use crate::transform::{Point, Vector};

const MIN_OBJECTS_PER_BOX: usize = 2;

#[derive(Debug, Clone)]
pub struct AccelerationStructure {
    bounding_boxes: Vec<BoundingBox>,
    scene: Arc<Scene>,
}

#[derive(Debug, Clone)]
struct BoundingBox {
    center: Point,
    size: Vector,
    pos_size: Vector,
    left: usize,
    right: usize,
    renderables: Vec<usize>,
}

#[derive(Debug, Copy, Clone)]
enum BoxAxis {
    X,
    Y,
    Z,
}

impl AccelerationStructure {
    pub fn new(scene: Arc<Scene>) -> Self {
        let initial_box = BoundingBox::new((0..scene.get_object_count()).collect(), scene.clone());
        Self {
            bounding_boxes: vec![initial_box],
            scene,
        }
    }
    
    pub fn generate(&mut self) {
        let mut ind = 0;
        while ind < self.bounding_boxes.len() {
            let box_len = self.bounding_boxes.len();
            
            if self.bounding_boxes[ind].renderables.len() > MIN_OBJECTS_PER_BOX {
                let axis = if self.bounding_boxes[ind].pos_size.x > self.bounding_boxes[ind].pos_size.z && self.bounding_boxes[ind].pos_size.x > self.bounding_boxes[ind].pos_size.y {
                    BoxAxis::X
                } else if self.bounding_boxes[ind].pos_size.y > self.bounding_boxes[ind].pos_size.z {
                    BoxAxis::Y
                } else {
                    BoxAxis::Z
                };
                
                
                let mut left_objects = Vec::new();
                let mut right_objects = Vec::new();
                
                let mut average_pos = Point::new(0.0, 0.0, 0.0);
                for r in &self.bounding_boxes[ind].renderables {
                    average_pos.add_assign_element_wise(self.scene.get_object(*r).get_aabb().0);
                }
                average_pos = average_pos / self.bounding_boxes[ind].renderables.len() as f64;
                
                while !self.bounding_boxes[ind].renderables.is_empty() {
                    let renderable = self.bounding_boxes[ind].renderables.pop().unwrap();
                    if axis.is_left(average_pos, self.scene.get_object(renderable).get_aabb().0) {
                        left_objects.push(renderable);
                    } else {
                        right_objects.push(renderable);
                    }
                }
                
                let mut extra = 0;
                if !left_objects.len() > 0 {
                    self.bounding_boxes[ind].left = box_len;
                    let left_box = BoundingBox::new(left_objects, self.scene.clone());
                    self.bounding_boxes.push(left_box);
                    extra = 1;
                } else if left_objects.len() == 1 {
                    self.bounding_boxes[ind].renderables.push(left_objects[0]);
                }
                if !right_objects.len() > 0 {
                    self.bounding_boxes[ind].right = box_len + extra;
                    let right_box = BoundingBox::new(right_objects, self.scene.clone());
                    self.bounding_boxes.push(right_box);
                } else if right_objects.len() == 1 {
                    self.bounding_boxes[ind].renderables.push(right_objects[0]);
                }
            }
            ind += 1
        }
    }
    
    pub fn trace_pixel(&self, x_coord: f64, y_coord: f64, num_bounces: usize) -> Vector {
        //return self.scene.trace_pixel(x_coord, y_coord, num_bounces); // For testing performance improvement.
        
        let (mut ray_orig, mut ray_dir) = self.scene.camera.get_ray(Vector2::new(x_coord, y_coord));

        let mut diffuse = Vector::new(1.0, 1.0, 1.0);
        let mut lighting = Vector::new(0.0, 0.0, 0.0);

        for _i in 0..num_bounces {
            if let Some((hit_point, normal, material)) = self.trace_structure(ray_orig, ray_dir) {
                //return normal;
                let (hit_diffuse, hit_emissive) = material.hit_surface(&mut ray_orig, &mut ray_dir, hit_point, normal);
                diffuse.mul_assign_element_wise(hit_diffuse);
                lighting.add_assign_element_wise(hit_emissive.mul_element_wise(diffuse));
                if material.emissive >= 1.0 {
                    break;
                }
            } else {
                lighting.add_assign_element_wise(self.scene.sky.get_sky_color(ray_dir).mul_element_wise(diffuse));
                break;
            }
        }

        return lighting;
    }
    
    pub fn trace_structure(&self, ray_orig: Point, ray_dir: Vector) -> Option<(Point, Vector, &PhysicalMaterial)> {
        let mut trace_queue = vec![0];
        
        let mut res = None;
        let mut closest = 0.0;
        
        while !trace_queue.is_empty() {
            let box_ind = trace_queue.pop().unwrap();
            let bounding_box = &self.bounding_boxes[box_ind];
            let (hit, box_dist) = bounding_box.trace(ray_orig, ray_dir);
            if hit && (box_dist < closest || res.is_none()) {
                //println!("Hit!");
                if bounding_box.left != 0 {
                    trace_queue.push(bounding_box.left);
                }
                if bounding_box.right != 0 {
                    trace_queue.push(bounding_box.right);
                }
                for renderable in &bounding_box.renderables {
                    let object = self.scene.get_object(*renderable);
                    if let Some((hit_point, hit_normal)) = object.trace(ray_orig, ray_dir) {
                        let dist = hit_point.distance(ray_orig);
                        if dist < closest || res.is_none() {
                            res = Some((hit_point, hit_normal, object.get_material()));
                            closest = dist;
                        }
                    }
                }
            }
        }
        
        res
    }
}

impl BoundingBox {
    fn new(renderables: Vec<usize>, scene: Arc<Scene>) -> Self {
        let mut min = Point::new(f64::MAX, f64::MAX, f64::MAX);
        let mut max = Point::new(f64::MIN, f64::MIN, f64::MIN);
        
        let mut pos_min = min;
        let mut pos_max = max;
        
        for i in &renderables {
            let (center, size) = scene.get_object(*i).get_aabb();
            pos_min.x = pos_min.x.min(center.x);
            pos_min.y = pos_min.y.min(center.y);
            pos_min.z = pos_min.z.min(center.z);
            
            pos_max.x = pos_max.x.max(center.x);
            pos_max.y = pos_max.y.max(center.y);
            pos_max.z = pos_max.z.max(center.z);
            
            let r_min = center - size;
            let r_max = center + size;
            
            min.x = min.x.min(r_min.x);
            min.y = min.y.min(r_min.y);
            min.z = min.z.min(r_min.z);
            
            max.x = max.x.max(r_max.x);
            max.y = max.y.max(r_max.y);
            max.z = max.z.max(r_max.z);
        }
        
        Self {
            center: max.add_element_wise(min) / 2.0,
            size: max.sub_element_wise(min).to_vec() / 2.0,
            pos_size: pos_max.sub_element_wise(pos_min).to_vec() / 2.0,
            left: 0,
            right: 0,
            renderables,
        }
    }
    
    fn trace(&self, ray_orig: Point, ray_dir: Vector) -> (bool, f64) {
        let b_min = self.center - self.size;
        let b_max = self.center + self.size;
        
        if ray_orig.x >= b_min.x && ray_orig.y >= b_min.y && ray_orig.z >= b_min.z && ray_orig.x <= b_max.x && ray_orig.y <= b_max.y && ray_orig.z <= b_max.z {
            return (true, 0.0);
        }
        
        let inv_dir = 1.0 / ray_dir;

        let t0 = (b_min - ray_orig).mul_element_wise(inv_dir);
        let t1 = (b_max - ray_orig).mul_element_wise(inv_dir);

        let v_min = t0.zip(t1, |a, b| a.min(b));
        let v_max = t0.zip(t1, |a, b| a.max(b));

        let t_min = v_min.x.max(v_min.y.max(v_min.z));
        let t_max = v_max.x.min(v_max.y.min(v_max.z));

        (t_max > t_min && t_min > 0.0, t_min)
    }
}

impl BoxAxis {
    fn is_left(&self, center: Point, pos: Point) -> bool {
        match self {
            BoxAxis::X => center.x <= pos.x,
            BoxAxis::Y => center.y <= pos.y,
            BoxAxis::Z => center.z <= pos.z,
        }
    }
}
