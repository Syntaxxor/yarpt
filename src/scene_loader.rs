use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use cgmath::Deg;
use crate::camera::Camera;
use crate::material::{PhysicalMaterial};
use crate::renderable::{Renderable, RenderShape};
use crate::scene::{Scene, Sky};
use crate::transform::{Point, Rot, Transform, Vector};

enum LoadState{
    Main,
    Camera,
    Materials,
    Scene,
}

pub fn load<P: AsRef<Path>>(path: P) -> Option<Scene> {
    let file = File::open(path);
    if let Ok(file) = file {
        let mut scene = Scene::new(
             Camera::default(),
             Sky::black(),
        );
        let mut reader = BufReader::new(file);
        
        let mut load_state = LoadState::Main;
        
        let mut materials: HashMap<String, PhysicalMaterial> = HashMap::new();
        
        let mut renderable = Renderable::new(Transform::default(), PhysicalMaterial::default(), RenderShape::None);
        
        let mut line = "".to_string();
        while reader.read_line(&mut line).unwrap() != 0 {
            line = line.trim().to_string();
            //println!("{line}");
            
            if line.starts_with('#') {
                line.clear();
                continue;
            }
            
            if line.contains("}") {
                load_state = LoadState::Main;
            }
            match load_state {
                LoadState::Main => {
                    if line.starts_with("camera") {
                        load_state = LoadState::Camera;
                    }
                    if line.starts_with("materials") {
                        load_state = LoadState::Materials;
                    }
                    if line.starts_with("scene") {
                        load_state = LoadState::Scene;
                    }
                }
                LoadState::Camera => {
                    let split_line = line.split_once(':');
                    if let Some((name, cam_data)) = split_line {
                        if name == "transform" {
                            scene.camera.transform = parse_transform(cam_data.trim());
                        }
                        if name == "focal_length" {
                            scene.camera.focal_length = cam_data.trim().parse().unwrap();
                        }
                        if name == "focal_plane" {
                            scene.camera.focal_plane = cam_data.trim().parse().unwrap();
                        }
                        if name == "f_stop" {
                            scene.camera.f_stop = cam_data.trim().parse().unwrap();
                        }
                    }
                }
                LoadState::Materials => {
                    let split_line = line.split_once(':');
                    if let Some((name, mat_data)) = split_line {
                        let material = parse_material(mat_data);
                        materials.insert(name.to_string(), material);
                    }
                }
                LoadState::Scene => {
                    if line.starts_with("sphere") {
                        renderable.shape = RenderShape::Sphere(1.0);
                    } else if line.starts_with("box") {
                        renderable.shape = RenderShape::Box(Vector::new(1.0, 1.0, 1.0));
                    } else {
                        let split_line = line.split_once(':');
                        if let Some((name, obj_data)) = split_line {
                            if name == "material" {
                                renderable.material = materials[obj_data.trim()];
                            } else if name == "transform" {
                                renderable.transform = parse_transform(obj_data.trim());
                            } else {
                                match &mut renderable.shape {
                                    RenderShape::None => {}
                                    RenderShape::Sphere(radius) => {
                                        if name == "radius" {
                                            *radius = obj_data.trim().parse().unwrap();
                                        }
                                    }
                                    RenderShape::Box(bounds) => {
                                        if name == "bounds" {
                                            let mut bounds_vec = obj_data.trim().split_whitespace().collect::<VecDeque::<_>>();
                                            *bounds = get_vec(&mut bounds_vec);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if line.contains(')') {
                        scene.add_object(renderable);
                    }
                }
            }
            
            line.clear();
        }
        
        return Some(scene);
    }
    None
}

fn parse_material(mat_data: &str) -> PhysicalMaterial {
    let mut material = PhysicalMaterial::default();
    let mut mat_data = mat_data.trim().split_whitespace().collect::<VecDeque<_>>();
    while !mat_data.is_empty() {
        let val = mat_data.pop_front().unwrap().trim();
        if val == "diffuse" {
            material.diffuse = get_vec(&mut mat_data);
        }
        if val == "roughness" {
            material.roughness = get_float(&mut mat_data);
        }
        if val == "metallic" {
            material.metallic = get_float(&mut mat_data);
        }
        if val == "emissive" {
            material.emissive = get_float(&mut mat_data);
        }
    }
    material
}

fn parse_transform(trans_data: &str) -> Transform {
    let mut transform = Transform::default();
    let mut trans_data = trans_data.trim().split_whitespace().collect::<VecDeque<_>>();
    while !trans_data.is_empty() {
        let val = trans_data.pop_front().unwrap().trim();
        if val == "position" {
            transform.position = get_point(&mut trans_data);
        }
        if val == "rotation" {
            transform.rotation = get_rot(&mut trans_data);
        }
        if val == "scale" {
            transform.scale = get_vec(&mut trans_data);
        }
    }
    
    transform
}

fn get_float(float_iter: &mut VecDeque<&str>) -> f64 {
    float_iter.pop_front().unwrap().parse::<f64>().unwrap()
}

fn get_point(point_iter: &mut VecDeque<&str>) -> Point {
    let x = get_float(point_iter);
    let y = get_float(point_iter);
    let z = get_float(point_iter);
    Point::new(x, y, z)
}

fn get_vec(vec_iter: &mut VecDeque<&str>) -> Vector {
    let x = get_float(vec_iter);
    let y = get_float(vec_iter);
    let z = get_float(vec_iter);
    Vector::new(x, y, z)
}

fn get_rot(rot_iter: &mut VecDeque<&str>) -> Rot {
    let x = get_float(rot_iter);
    let y = get_float(rot_iter);
    let z = get_float(rot_iter);
    Rot::new(Deg(x), Deg(y), Deg(z))
}