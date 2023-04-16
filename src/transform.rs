use std::ops::Add;
use cgmath::{Point3, Vector3, Basis3, Euler, Deg, Rotation, ElementWise, InnerSpace, EuclideanSpace};

pub type Point = Point3<f64>;
pub type Vector = Vector3<f64>;
pub type Rot = Euler<Deg<f64>>;
pub type Basis = Basis3<f64>;

#[derive(Debug, Copy, Clone)]
pub struct Transform {
    pub position: Point,
    pub rotation: Rot,
    pub scale: Vector,
}

impl Transform {
    pub fn new(position: Point, rotation: Rot, scale: Vector) -> Self {
        Self{
            position,
            rotation,
            scale,
        }
    }
    
    pub fn get_basis(&self) -> Basis {
        Basis3::from(self.rotation)
    }
    
    pub fn to_local_point(&self, point: Point) -> Point {
        self.get_basis().rotate_point(point.sub_element_wise(self.position))
            .div_element_wise(Point::from_vec(self.scale))
    }
    
    pub fn to_local_vector(&self, vector: Vector) -> Vector {
        self.get_basis().rotate_vector(vector).div_element_wise(self.scale).normalize()
    }
    
    pub fn to_global_point(&self, point: Point) -> Point {
        self.get_basis().invert().rotate_point(
            point.mul_element_wise(
                Point::from_vec(self.scale)
            )
        ).add_element_wise(self.position)
    }
    
    pub fn to_global_vector(&self, vector: Vector) -> Vector {
        self.get_basis().invert().rotate_vector(vector.div_element_wise(self.scale)).normalize()
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self{
            position: Point::new(0.0, 0.0, 0.0),
            rotation: Rot::new(Deg(0.0), Deg(0.0), Deg(0.0)),
            scale: Vector::new(1.0, 1.0, 1.0),
        }
    }
}