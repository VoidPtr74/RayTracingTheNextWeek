use crate::vec3::*;
use crate::ray::*;
use crate::aabb::Aabb;
use crate::rng::Random;
use crate::perlin::Perlin;

use std::cmp::Ordering;
use std::f32;

pub struct SphereData {
    pub center: Vec3,
    pub radius: f32,
    pub material: MaterialId
}

pub struct MovingSphereData {
    pub center_start : Vec3,
    pub center_end : Vec3,
    pub radius : f32,
    pub material: MaterialId,
    pub time_start : f32,
    pub time_end : f32
}

#[derive(Copy, Clone)]
pub struct InstanceId {
    value : usize
}

pub enum Instance {
    Empty,
    BvhNode(Aabb, InstanceId, InstanceId),
    FlipNormals(InstanceId),
    Sphere(SphereData),
    MovingSphere(MovingSphereData)
}

pub enum MaterialInstance {
    Lambertian(TextureId),
    Metal(Vec3, f32),
    Dielectric(f32),
    DiffuseLight(TextureId), 
    Isotropic(TextureId)
}

fn scatter_metal(albedo : &Vec3, fuzz: f32, ray: &Ray, rec: &InstanceHitRecord, rnd: &mut Random, attenuation: &mut Vec3, scattered: &mut Ray) -> bool {
    let reflected = reflect(&ray.direction.make_normalised(), &rec.normal);
    scattered.origin = rec.p;
    scattered.direction = reflected + &random_in_unit_sphere(rnd) * fuzz;
    scattered.time = ray.time;
    attenuation.set(&albedo);
    dot(&scattered.direction, &rec.normal) > 0.0
}

fn schlick(cosine: f32, refraction_index: f32) -> f32 {
    let r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

fn scatter_dielectric(refraction_index : f32, ray: &Ray, rec: &InstanceHitRecord, rnd: &mut Random, attenuation: &mut Vec3, scattered: &mut Ray) -> bool {
    let reflected = reflect(&ray.direction, &rec.normal);
    attenuation.set(&Vec3::from(1.0, 1.0, 1.0));
    let outward_normal: Vec3;
    let ni_over_nt: f32;
    let incident_dot_normal = dot(&ray.direction, &rec.normal);
    let cosine = if incident_dot_normal > 0.0 {
        outward_normal = &rec.normal * -1.0;
        ni_over_nt = refraction_index;
        refraction_index * incident_dot_normal / ray.direction.length()
    } else {
        outward_normal = rec.normal;
        ni_over_nt = 1.0 / refraction_index;
        -incident_dot_normal / ray.direction.length()
    };

    scattered.origin = rec.p;
    scattered.time = ray.time;
    let refracted_maybe = refract(&ray.direction, &outward_normal, ni_over_nt);
    match &refracted_maybe {
        None => {
            scattered.direction = reflected;
        }
        Some(refracted) => {
            let reflect_prob = schlick(cosine, refraction_index);
            if rnd.gen() < reflect_prob {
                scattered.direction = reflected
            } else {
                scattered.direction.set(refracted)
            }
        }
    };

    true
}

pub enum TextureInstance {
    ConstantTexture(Vec3),
    CheckerTexture(TextureId, TextureId),
    NoiseTexture(Box<Perlin>, f32), // noise + scale
    ImageTexture(Vec<u8>, usize, usize)
}

#[derive(Copy, Clone)]
pub struct MaterialId {
    value : usize
}

#[derive(Copy, Clone)]
pub struct TextureId {
    value : usize
}

pub struct InstanceHitRecord {
    pub t: f32,
    pub p: Vec3,
    pub normal: Vec3,
    pub material_id : MaterialId,
    pub u : f32,
    pub v : f32
}

pub struct World {
    bvh_root : InstanceId,
    materials : Vec<MaterialInstance>,
    textures : Vec<TextureInstance>,
    object_instances : Vec<Instance>
}

pub struct WorldBuilder {
    materials : Vec<MaterialInstance>,
    textures : Vec<TextureInstance>,
    object_instances : Vec<Instance>
}

fn add_instance(instances: &mut Vec<Instance>, instance : Instance) -> InstanceId {
    let id = InstanceId { value: instances.len() };
    instances.push(instance);
    id
}

impl WorldBuilder {
    const EMPTY_INSTANCE_ID : InstanceId = InstanceId { value: 0 };

    pub fn create() -> Self {
        let mut object_instances = Vec::new();
        // "null" instance
        object_instances.push(Instance::Empty);
        WorldBuilder { materials: Vec::new(), textures: Vec::new(), object_instances: object_instances }
    }

    pub fn create_material(&mut self, material : MaterialInstance) -> MaterialId {
        let id = MaterialId { value: self.materials.len() };
        self.materials.push(material);
        id
    }

    pub fn create_texture(&mut self, texture: TextureInstance) -> TextureId {
        let id = TextureId { value: self.textures.len() };
        self.textures.push(texture);
        id
    }

    pub fn create_instance(&mut self, instance: Instance) -> InstanceId {
        add_instance(&mut self.object_instances, instance)
    }

    pub fn build(mut self, time_start : f32, time_end : f32) -> World {
        let bvh = Self::build_bvh(&mut self.object_instances, time_start, time_end);
        World {bvh_root: bvh, object_instances: self.object_instances, materials: self.materials, textures: self.textures}
    }
}

fn moving_sphere_center(data : &MovingSphereData, time: f32) -> Vec3 {
    data.center_start + (&(data.center_end - data.center_start) * ((time - data.time_start) / (data.time_end - data.time_start)))
}

fn hit_moving_sphere(data : &MovingSphereData, ray: &Ray, t_min : f32, t_max : f32) -> Option<InstanceHitRecord> {
    let center = moving_sphere_center(data, ray.time);
    let oc = ray.origin - center;
    let a = ray.direction.square_length();
    let b = dot(&oc, &ray.direction);
    let c = oc.square_length() - data.radius * data.radius;
    let discriminant = b * b - a * c;
    if discriminant > 0.0 {
        let tmp = (-b - discriminant.sqrt()) / a;
        if tmp < t_max && tmp > t_min {
            let hit_point = ray.point_at_parameter(tmp);
            let (u,v) = get_sphere_uv(&center, &hit_point);
            let record = InstanceHitRecord {
                t: tmp,
                p: hit_point,
                normal: &(hit_point - center) / data.radius,
                material_id: data.material,
                u, v
            };
            return Option::Some(record);
        }

        let tmp = (-b + discriminant.sqrt()) / a;
        if tmp < t_max && tmp > t_min {
            let hit_point = ray.point_at_parameter(tmp);
            let (u,v) = get_sphere_uv(&center, &hit_point);
            let record = InstanceHitRecord {
                t: tmp,
                p: hit_point,
                normal: &(hit_point - center) / data.radius,
                material_id: data.material,
                u, v
            };
            return Option::Some(record);
        }
    }

    Option::None
}

impl World {
    pub fn texture_colour(&self, texture_id : TextureId, u : f32, v : f32, p : &Vec3) -> Vec3 {
        // These are bounds-checked when they really don't need to be. Consider unsafe{}-ing with get_unchecked if we need a bit more perf.
        let texture = &self.textures[texture_id.value];
        match texture {
            TextureInstance::ConstantTexture(colour) => *colour,
            TextureInstance::CheckerTexture(odd_id, even_id) => {
                let sines = (10.0 * p.x()).sin() * (10.0 * p.y()).sin() * (10.0 * p.z()).sin();
                let used_id = if sines < 0.0 { odd_id } else { even_id };
                self.texture_colour(*used_id, u, v, p)
            },
            TextureInstance::NoiseTexture(perlin, scale) => {
                &Vec3::from(1.0, 1.0, 1.0) * (0.5*(1.0 + (scale*p.z() + 10.0*perlin.turb(&(p**scale), 7)).sin()))
            },
            TextureInstance::ImageTexture(data, w, h) => {
                let width = *w;
                let height = *h;
                let i = (u * width as f32) as i32;
                let j = ((1.0 - v) * height as f32 - 0.001) as i32;
                let i = if i < 0 { 0 } else {if i > width as i32 - 1 { width - 1 } else {i as usize}};
                let j = if j < 0 { 0 } else {if j > height as i32 - 1 { height - 1 } else {j as usize}};

                let base_address = 3*(i + width*j);
                &Vec3::from(f32::from(data[base_address]),f32::from(data[base_address + 1]), f32::from(data[base_address + 2])) / 255.0
            }
        }
    }

    pub fn material_scatter(&self, ray: &Ray, rec: &InstanceHitRecord, rnd: &mut Random, attenuation: &mut Vec3, scattered: &mut Ray) -> bool {
        let material = &self.materials[rec.material_id.value];
        match material {
            MaterialInstance::Lambertian(texture_id) => {
                let target = rec.p + rec.normal + random_in_unit_sphere(rnd);
                scattered.origin = rec.p;
                scattered.direction = target - rec.p;
                scattered.time = ray.time;
                let colour = self.texture_colour(*texture_id, rec.u, rec.v, &rec.p);
                attenuation.set(&colour);
                true
            },
            MaterialInstance::Metal(albedo, fuzz) => {scatter_metal(albedo, *fuzz, ray, rec, rnd, attenuation, scattered)},
            MaterialInstance::Dielectric(refraction_index) => {scatter_dielectric(*refraction_index, ray, rec, rnd, attenuation, scattered)},
            MaterialInstance::DiffuseLight(_) => {false},
            MaterialInstance::Isotropic(texture_id) => {
                *scattered = Ray {
                    origin: rec.p,
                    direction: random_in_unit_sphere(rnd),
                    time: ray.time
                };
                *attenuation = self.texture_colour(*texture_id, rec.u, rec.v, &rec.p);
                true
            }

        }
    }
    
    pub fn material_emitted(&self, material_id : MaterialId, u : f32, v : f32, p : &Vec3) -> Vec3 {
        let material = &self.materials[material_id.value];
        match material {
            MaterialInstance::DiffuseLight(texture_id) => {
                self.texture_colour(*texture_id, u, v, p)
            },
            _ => Vec3::from(0.0,0.0,0.0)
        }
    }

    pub fn hit(&self, ray : &Ray, t_min : f32, t_max: f32) -> Option<InstanceHitRecord> {
        self.hit_index(self.bvh_root, ray, t_min, t_max)
    }

    fn hit_index(&self, id : InstanceId, ray: &Ray, t_min : f32, t_max : f32) -> Option<InstanceHitRecord> {
        let instance = &self.object_instances[id.value];
        match instance {
            Instance::Empty => Option::None,
            Instance::Sphere(data) => {
                let oc = ray.origin - data.center;
                let a = ray.direction.square_length();
                let b = dot(&oc, &ray.direction);
                let c = oc.square_length() - data.radius * data.radius;
                let discriminant = b * b - a * c;
                if discriminant > 0.0 {
                    let tmp = (-b - discriminant.sqrt()) / a;
                    if tmp < t_max && tmp > t_min {
                        let hit_point = ray.point_at_parameter(tmp);
                        let (u,v) = get_sphere_uv(&data.center, &hit_point);
                        let record = InstanceHitRecord {
                            t: tmp,
                            p: hit_point,
                            normal: &(hit_point - data.center) / data.radius,
                            material_id: data.material,
                            u, v
                        };
                        return Option::Some(record);
                    }

                    let tmp = (-b + discriminant.sqrt()) / a;
                    if tmp < t_max && tmp > t_min {
                        let hit_point = ray.point_at_parameter(tmp);
                        let (u,v) = get_sphere_uv(&data.center, &hit_point);
                        let record = InstanceHitRecord {
                            t: tmp,
                            p: hit_point,
                            normal: &(hit_point - data.center) / data.radius,
                            material_id: data.material,
                            u, v
                        };
                        return Option::Some(record);
                    }
                }

                Option::None
            },
            Instance::FlipNormals(id) =>
            {
                let result = self.hit_index(*id, ray, t_min, t_max);
                match result {
                    None => Option::None,
                    Some(mut record) => {
                        record.normal = &record.normal * -1.0;
                        Option::Some(record)
                    }
                }
            },
            Instance::BvhNode(bb, left, right) => {
                if bb.hit(ray, t_min, t_max) {
                    let hit_left = self.hit_index(*left, ray, t_min, t_max);
                    let hit_right = self.hit_index(*right, ray, t_min, t_max);
                    return match (&hit_left, &hit_right) {
                        (Some(left), Some(right)) => {
                            if left.t < right.t {
                                hit_left
                            } else {
                                hit_right
                            }
                        }
                        (Some(_), None) => hit_left,
                        (None, Some(_)) => hit_right,
                        _ => Option::None,
                    };
                } else {
                    return Option::None
                }
            },
            Instance::MovingSphere(data) => {
                hit_moving_sphere(data, ray, t_min, t_max)
            }
        }
    }
}

//////////////////////////
/// NEW BVH Tree implementation from here down..
//////////////////////////


fn get_sphere_uv(center : &Vec3, p : &Vec3) -> (f32, f32) {
    let p = (p - center).make_normalised();
    let phi = p.z().atan2(*p.x());
    let theta = p.y().asin();
    let u = 1.0 - (phi + f32::consts::PI) / (2.0*f32::consts::PI);
    let v = (theta + 0.5 * f32::consts::PI) / f32::consts::PI;
    (u,v)
}

fn get_bounding_box(id : &InstanceId, instances : &Vec<Instance>, time_start : f32, time_end : f32) -> Aabb {
    let instance = &instances[id.value];
    match instance {
        Instance::Empty => Aabb::empty(),
        Instance::FlipNormals(sub_id) => get_bounding_box(sub_id, instances, time_start, time_end),
        Instance::Sphere(data) => 
        {
            let radial_length = Vec3::from(data.radius, data.radius, data.radius);
            Aabb::build(data.center - radial_length, data.center + radial_length)
        },
        Instance::MovingSphere(data) => {
            let radial_length = Vec3::from(data.radius, data.radius, data.radius);
            let bb0 = Aabb::build(moving_sphere_center(data, time_start) - radial_length, moving_sphere_center(data, time_start) + radial_length);
            let bb1 = Aabb::build(moving_sphere_center(data, time_end) - radial_length, moving_sphere_center(data, time_end) + radial_length);
            Aabb::surrounding_box(&bb0, &bb1)
        },
        Instance::BvhNode(bb, _, _) => {
            *bb
        },
    }
}

impl WorldBuilder {
    fn build_bvh(instance_nodes: &mut Vec<Instance>, time_start : f32, time_end : f32) -> InstanceId {
        let mut node_ids : Vec<InstanceId> = Vec::with_capacity(instance_nodes.len());
        for index in 0..instance_nodes.len() {
            match &instance_nodes[index] {
                Instance::Empty => {},
                _ => {node_ids.push(InstanceId {value:index});}
            }
        }

        Self::build_node(instance_nodes, &mut node_ids, 0, time_start, time_end)
    }

    fn build_node(instances : &mut Vec<Instance>, ids : &mut Vec<InstanceId>, axis : usize, time_start: f32, time_end: f32) -> InstanceId {
        if ids.len() == 0 {
            return WorldBuilder::EMPTY_INSTANCE_ID
        }

        if ids.len() == 1 {
            return ids[0]
        }

        if ids.len() == 2 {
            let left_box = get_bounding_box(&ids[0], instances, time_start, time_end);
            let right_box = get_bounding_box(&ids[1], instances, time_start, time_end);
            let surrounding_box = Aabb::surrounding_box(&left_box, &right_box);
            return add_instance(instances, Instance::BvhNode(surrounding_box, ids[0], ids[1]));
        }

        // let left_segment = ...
        ids.sort_by(|left, right| {
            let bb_left = *get_bounding_box(left, instances, time_start, time_end).min.get(axis);
            let bb_right = *get_bounding_box(right, instances, time_start, time_end).min.get(axis);
            if bb_left < bb_right {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });

        let mut split = ids.split_off(ids.len() / 2);
        let next_axis = (axis + 1) % 3;
        let left = Self::build_node(instances, ids, next_axis, time_start, time_end);
        let right = Self::build_node(instances, &mut split, next_axis, time_start, time_end);
        let left_box = get_bounding_box(&left, instances, time_start, time_end);
        let right_box = get_bounding_box(&right, instances, time_start, time_end);
        let surrounding_box = Aabb::surrounding_box(&left_box, &right_box);
        add_instance(instances, Instance::BvhNode(surrounding_box, left, right))
    }
}