use crate::aabb::Aabb;
use crate::material::*;
use crate::ray::*;
use crate::rng::Random;
use crate::vec3::*;
use crate::texture::*;
extern crate rand;

use rand::*;
use rand::distributions::*;
use std::cmp::Ordering;
use std::vec::Vec;
use std::f32; 

pub trait Hitable: Send + Sync {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
    fn bounding_box(&self, time0 : f32, time1 : f32) -> Aabb;
}

pub struct BvhTree {
    pub root: Box<dyn Hitable>,
}

pub struct BvhNode {
    pub bounding_box: Aabb,
    pub left: Box<dyn Hitable>,
    pub right: Box<dyn Hitable>,
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Box<dyn Material>,
}

pub struct MovingSphere {
    pub center_start : Vec3,
    pub center_end : Vec3,
    pub radius : f32,
    pub material: Box<dyn Material>,
    pub time_start : f32,
    pub time_end : f32
}

impl BvhTree {
    pub fn build(hitables: &mut Vec<Box<dyn Hitable>>, rnd: &mut Random, time_start : f32, time_end : f32) -> Self {
        BvhTree {
            root: BvhNode::build_bvh_tree(hitables, rnd, time_start, time_end),
        }
    }
}

impl BvhNode {
    fn build_bvh_tree(hitables: &mut Vec<Box<dyn Hitable>>, rnd: &mut Random, time_start : f32, time_end : f32) -> Box<dyn Hitable> {
        match hitables.len() {
            1 => return hitables.remove(0),
            2 => {
                let left = hitables.remove(0);
                let right = hitables.remove(0);
                return Box::new(Self::create(left, right, time_start, time_end));
            }
            _ => {}
        };

        let axis = (rnd.gen() * 3.0) as usize;
        hitables.sort_by(|left, right| {
            let bb_left = *left.bounding_box(time_start, time_end).min.get(axis);
            let bb_right = *right.bounding_box(time_start, time_end).min.get(axis);
            if bb_left < bb_right {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });
        let mut split = hitables.split_off(hitables.len() / 2);

        let left = Self::build_bvh_tree(hitables, rnd, time_start, time_end);
        let right = Self::build_bvh_tree(&mut split, rnd, time_start, time_end);
        Box::new(Self::create(left, right, time_start, time_end))
    }

    fn create(left: Box<dyn Hitable>, right: Box<dyn Hitable>, time_start : f32, time_end : f32) -> BvhNode {
        let bounding_box = Aabb::surrounding_box(&left.bounding_box(time_start, time_end), &right.bounding_box(time_start, time_end));
        BvhNode {
            bounding_box,
            left,
            right,
        }
    }
}

impl Hitable for BvhNode {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        if self.bounding_box.hit(ray, t_min, t_max) {
            let hit_left = self.left.hit(ray, t_min, t_max);
            let hit_right = self.right.hit(ray, t_min, t_max);
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
        }

        Option::None
    }

    fn bounding_box(&self, _time0 : f32, _time1 : f32) -> Aabb {
        self.bounding_box
    }
}

// not convinced this works if the sphere isn't centered at 0
fn get_sphere_uv(center : &Vec3, p : &Vec3) -> (f32, f32) {
    let p = (p - center).make_normalised();
    let phi = p.z().atan2(*p.x());
    let theta = p.y().asin();
    let u = 1.0 - (phi + f32::consts::PI) / (2.0*f32::consts::PI);
    let v = (theta + 0.5 * f32::consts::PI) / f32::consts::PI;
    (u,v)
}

impl Hitable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = ray.origin - self.center;
        let a = ray.direction.square_length();
        let b = dot(&oc, &ray.direction);
        let c = oc.square_length() - self.radius * self.radius;
        let discriminant = b * b - a * c;
        if discriminant > 0.0 {
            let tmp = (-b - discriminant.sqrt()) / a;
            if tmp < t_max && tmp > t_min {
                let hit_point = ray.point_at_parameter(tmp);
                let (u,v) = get_sphere_uv(&self.center, &hit_point);
                let record = HitRecord {
                    t: tmp,
                    p: hit_point,
                    normal: &(hit_point - self.center) / self.radius,
                    material: &*self.material,
                    u, v
                };
                return Option::Some(record);
            }

            let tmp = (-b + discriminant.sqrt()) / a;
            if tmp < t_max && tmp > t_min {
                let hit_point = ray.point_at_parameter(tmp);
                let (u,v) = get_sphere_uv(&self.center, &hit_point);
                let record = HitRecord {
                    t: tmp,
                    p: hit_point,
                    normal: &(hit_point - self.center) / self.radius,
                    material: &*self.material,
                    u, v
                };
                return Option::Some(record);
            }
        }

        Option::None
    }

    fn bounding_box(&self, _time0 : f32, _time1 : f32) -> Aabb {
        let radial_length = Vec3::from(self.radius, self.radius, self.radius);
        Aabb::build(self.center - radial_length, self.center + radial_length)
    }
}

impl MovingSphere {
    fn center(&self, time : f32)  -> Vec3 {
        self.center_start + (&(self.center_end - self.center_start) * ((time - self.time_start) / (self.time_end - self.time_start)))
    }
}

impl Hitable for MovingSphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = ray.origin - self.center(ray.time);
        let a = ray.direction.square_length();
        let b = dot(&oc, &ray.direction);
        let c = oc.square_length() - self.radius * self.radius;
        let discriminant = b * b - a * c;
        if discriminant > 0.0 {
            let tmp = (-b - discriminant.sqrt()) / a;
            if tmp < t_max && tmp > t_min {
                let hit_point = ray.point_at_parameter(tmp);
                let (u,v) = get_sphere_uv(&self.center(ray.time), &hit_point);
                let record = HitRecord {
                    t: tmp,
                    p: hit_point,
                    normal: &(hit_point - self.center(ray.time)) / self.radius,
                    material: &*self.material,
                    u, v
                };
                return Option::Some(record);
            }

            let tmp = (-b + discriminant.sqrt()) / a;
            if tmp < t_max && tmp > t_min {
                let hit_point = ray.point_at_parameter(tmp);
                let (u,v) = get_sphere_uv(&self.center(ray.time), &hit_point);
                let record = HitRecord {
                    t: tmp,
                    p: hit_point,
                    normal: &(hit_point - self.center(ray.time)) / self.radius,
                    material: &*self.material,
                    u, v
                };
                return Option::Some(record);
            }
        }

        Option::None
    }

    fn bounding_box(&self, time0 : f32, time1 : f32) -> Aabb {
        let radial_length = Vec3::from(self.radius, self.radius, self.radius);
        let bb0 = Aabb::build(self.center(time0) - radial_length, self.center(time0) + radial_length);
        let bb1 = Aabb::build(self.center(time1) - radial_length, self.center(time1) + radial_length);
        Aabb::surrounding_box(&bb0, &bb1)
    }
}

pub struct XyRect {
    pub x0 : f32,
    pub x1 : f32, 
    pub y0 : f32,
    pub y1 : f32,
    pub z  : f32,
    pub material : Box<dyn Material>
}

impl Hitable for XyRect {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (self.z - ray.origin.z()) / ray.direction.z();
        if t < t_min || t > t_max {
            return Option::None;
        }

        let x = ray.origin.x() + t*ray.direction.x();
        let y = ray.origin.y() + t*ray.direction.y();
        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return Option::None;
        }

        let record = HitRecord { 
            u: (x - self.x0)/(self.x1 - self.x0),
            v: (y - self.y0)/(self.y1 - self.y0),
            t,
            material : &*self.material,
            p : ray.point_at_parameter(t),
            normal : Vec3::from(0.0, 0.0, 1.0)
        };

        Option::Some(record)
    }

    fn bounding_box(&self, _time0 : f32, _time1 : f32) -> Aabb {
        Aabb::build(Vec3::from(self.x0, self.y0, self.z - 0.0001), Vec3::from(self.x1, self.y1, self.z + 0.0001))
    }
}

pub struct XzRect {
    pub x0 : f32,
    pub x1 : f32, 
    pub z0 : f32,
    pub z1 : f32,
    pub y  : f32,
    pub material : Box<dyn Material>
}

impl Hitable for XzRect {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (self.y - ray.origin.y()) / ray.direction.y();
        if t < t_min || t > t_max {
            return Option::None;
        }

        let x = ray.origin.x() + t*ray.direction.x();
        let z = ray.origin.z() + t*ray.direction.z();
        if x < self.x0 || x > self.x1 || z < self.z0 || z > self.z1 {
            return Option::None;
        }

        let record = HitRecord { 
            u: (x - self.x0)/(self.x1 - self.x0),
            v: (z - self.z0)/(self.z1 - self.z0),
            t,
            material : &*self.material,
            p : ray.point_at_parameter(t),
            normal : Vec3::from(0.0, 1.0, 0.0)
        };

        Option::Some(record)
    }

    fn bounding_box(&self, _time0 : f32, _time1 : f32) -> Aabb {
        Aabb::build(Vec3::from(self.x0, self.y - 0.0001, self.z0), Vec3::from(self.x1, self.y + 0.0001, self.z1))
    }
}

pub struct YzRect {
    pub y0 : f32,
    pub y1 : f32, 
    pub z0 : f32,
    pub z1 : f32,
    pub x  : f32,
    pub material : Box<dyn Material>
}

impl Hitable for YzRect {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (self.x - ray.origin.x()) / ray.direction.x();
        if t < t_min || t > t_max {
            return Option::None;
        }

        let y = ray.origin.y() + t*ray.direction.y();
        let z = ray.origin.z() + t*ray.direction.z();
        if y < self.y0 || y > self.y1 || z < self.z0 || z > self.z1 {
            return Option::None;
        }

        let record = HitRecord { 
            u: (y - self.y0)/(self.y1 - self.y0),
            v: (z - self.z0)/(self.z1 - self.z0),
            t,
            material : &*self.material,
            p : ray.point_at_parameter(t),
            normal : Vec3::from(1.0, 0.0, 0.0)
        };

        Option::Some(record)
    }

    fn bounding_box(&self, _time0 : f32, _time1 : f32) -> Aabb {
        Aabb::build(Vec3::from(self.x - 0.0001, self.y0, self.z0), Vec3::from(self.x + 0.0001, self.y1, self.z1))
    }
}

pub struct FlipNormals {
    pub obj : Box<dyn Hitable>
}

impl FlipNormals {
    pub fn new_with_obj(obj : Box<dyn Hitable>) -> Box<FlipNormals> {
        Box::new(FlipNormals {obj})
    }
}

impl Hitable for FlipNormals {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let result = self.obj.hit(ray, t_min, t_max);
        match result {
            None => Option::None,
            Some(record) => {
                Option::Some(HitRecord {t: record.t, p: record.p, normal: &record.normal * -1.0, material: record.material, u: record.u, v: record.v})
            }
        }
    }

    fn bounding_box(&self, time0 : f32, time1 : f32) -> Aabb {
        self.obj.bounding_box(time0, time1)
    }
}

pub struct BoxShape {
    material : Box<dyn Material>,
    faces : Vec<Box<dyn Hitable>>,
    pmin : Vec3,
    pmax : Vec3
}

impl BoxShape {
    pub fn new_from(p0 : &Vec3, p1 : &Vec3, material : Box<dyn Material>) -> Box<BoxShape>{
        let mut list : Vec<Box<dyn Hitable>> = Vec::with_capacity(6);

        // rust gods forgive me..
        let mat = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.0, 0.0, 0.0))));
        list.push(Box::new(XyRect {x0: *p0.x(), x1: *p1.x(), y0: *p0.y(), y1: *p1.y(), z: *p1.z(), material: mat}));
        let mat = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.0, 0.0, 0.0))));
        list.push(FlipNormals::new_with_obj(Box::new(XyRect {x0: *p0.x(), x1: *p1.x(), y0: *p0.y(), y1: *p1.y(), z: *p0.z(), material: mat})));
        
        let mat = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.0, 0.0, 0.0))));
        list.push(Box::new(XzRect {x0: *p0.x(), x1: *p1.x(), z0: *p0.z(), z1: *p1.z(), y: *p1.y(), material: mat}));
        let mat = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.0, 0.0, 0.0))));
        list.push(FlipNormals::new_with_obj(Box::new(XzRect {x0: *p0.x(), x1: *p1.x(), z0: *p0.z(), z1: *p1.z(), y: *p0.y(), material: mat})));

        let mat = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.0, 0.0, 0.0))));
        list.push(Box::new(YzRect {y0: *p0.y(), y1: *p1.y(), z0: *p0.z(), z1: *p1.z(), x: *p1.x(), material: mat}));
        let mat = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.0, 0.0, 0.0))));
        list.push(FlipNormals::new_with_obj(Box::new(YzRect {y0: *p0.y(), y1: *p1.y(), z0: *p0.z(), z1: *p1.z(), x: *p0.x(), material: mat})));

        Box::new(BoxShape { faces: list, material, pmin: *p0, pmax: *p1 })
    }
}

impl Hitable for BoxShape {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut least_t = t_max;
        let mut record : Option<HitRecord> = Option::None;

        for obj in &self.faces {
            let hit  = obj.hit(ray, t_min, least_t);
            if let Some(rec) = hit {
                least_t = rec.t;
                record = Option::Some(HitRecord { t: rec.t, p: rec.p, normal: rec.normal, material : &*self.material, u: rec.u, v: rec.v });
            }
        }

        record
    }

    fn bounding_box(&self, _time0 : f32, _time1 : f32) -> Aabb {
        Aabb { min: self.pmin, max: self.pmax }
    }
}

pub struct Translate {
    pub obj : Box<dyn Hitable>,
    pub offset : Vec3
}

impl Hitable for Translate {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let moved_ray = Ray { origin: ray.origin - self.offset, direction: ray.direction, time : ray.time };
        let result = self.obj.hit(&moved_ray, t_min, t_max);
        match result {
            None => result,
            Some(rec) => {
                Option::Some(
                    HitRecord {
                        t: rec.t,
                        p : rec.p + self.offset,
                        normal : rec.normal,
                        material : rec.material,
                        u: rec.u,
                        v: rec.v
                    }
                )
            }
        }
    }

    fn bounding_box(&self, time0 : f32, time1 : f32) -> Aabb {
        let bb = self.obj.bounding_box(time0, time1);
        Aabb { min: bb.min + self.offset, max: bb.max + self.offset }
    }
}

pub struct RotateY {
    obj : Box<dyn Hitable>,
    sin_theta : f32,
    cos_theta : f32,
    bb : Aabb
}

impl RotateY {
    pub fn create_new(obj : Box<dyn Hitable>, angle : f32) -> Box<RotateY> {
        let radians = angle * (f32::consts::PI / 180.);
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();
        let bb = obj.bounding_box(0.0, 1.0);
        let mut min = Vec3::from(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vec3::from(-f32::MAX, -f32::MAX, -f32::MAX);
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let selector = Vec3::from(i as f32, j as f32, k as f32);
                    let point = selector.direct_product(&bb.max) - (selector - 1.0).direct_product(&bb.min);
                    let x = cos_theta * point.x() + sin_theta*point.z();
                    let z = -sin_theta * point.x() + cos_theta*point.z();
                    let rotated_point = Vec3::from(x, *point.y(), z);

                    min = min.min(&rotated_point);
                    max = max.max(&rotated_point);
                }
            }
        }

        let rotated_bb = Aabb {min, max};
        Box::new(RotateY { obj, sin_theta, cos_theta, bb: rotated_bb })
    }
}

impl Hitable for RotateY {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let rotated_ray = Ray {
            origin: Vec3::from(
                self.cos_theta * ray.origin.x() - self.sin_theta * ray.origin.z(),
                *ray.origin.y(),
                self.sin_theta * ray.origin.x() + self.cos_theta * ray.origin.z()
            ),
            direction: Vec3::from(
                self.cos_theta * ray.direction.x() - self.sin_theta * ray.direction.z(),
                *ray.direction.y(),
                self.sin_theta * ray.direction.x() + self.cos_theta * ray.direction.z()
            ),
            time: ray.time
        };

        let record = self.obj.hit(&rotated_ray, t_min, t_max);

        match record {
            None => record,
            Some(rec) => {
                let point = Vec3::from(
                    self.cos_theta * rec.p.x() + self.sin_theta * rec.p.z(),
                    *rec.p.y(),
                    -self.sin_theta * rec.p.x() + self.cos_theta * rec.p.z()
                );

                let normal = Vec3::from(
                    self.cos_theta * rec.normal.x() + self.sin_theta * rec.normal.z(),
                    *rec.normal.y(),
                    -self.sin_theta * rec.normal.x() + self.cos_theta * rec.normal.z()
                );

                Option::Some(HitRecord { t: rec.t, p: point, normal: normal, material: rec.material, u: rec.u, v: rec.v })
            }
        }
    }

    fn bounding_box(&self, _time0 : f32, _time1 : f32) -> Aabb {
        self.bb
    }
}

pub struct ConstantMedium {
    density : f32,
    boundary : Box<dyn Hitable>,
    phase_function : Box<dyn Material>,
}

impl ConstantMedium {
    pub fn build_new(density : f32, boundary : Box<dyn Hitable>, texture : Box<dyn Texture>) -> Box<ConstantMedium> {
        let mat = Box::new(Isotropic { albedo : texture });
        Box::new(ConstantMedium {
            density,
            boundary,
            phase_function: mat,
        })
    }
}

impl Hitable for ConstantMedium {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let option_rec1 = self.boundary.hit(ray, -f32::MAX, f32::MAX);
        let mut rng = thread_rng();
        let dist = distributions::Uniform::new(0.0f32, 1.0f32);
        
                
        if let Some(rec1) = option_rec1 {
            let option_rec2 = self.boundary.hit(ray, rec1.t + 0.0001, f32::MAX);
            if let Some(rec2) = option_rec2 {
                let mut r1_t = if rec1.t < t_min {t_min} else {rec1.t};
                let r2_t = if rec2.t > t_max {t_max} else {rec2.t};
                if r1_t >= r2_t {
                    return Option::None;
                }
                if r1_t < 0.0 { r1_t = 0.0; }

                let distance_inside_boundary = (r2_t - r1_t) * ray.direction.length();
                let hit_distance = -dist.sample(&mut rng).ln() / self.density;
                if hit_distance < distance_inside_boundary {
                    let t = rec1.t + hit_distance / ray.direction.length();
                    return Option::Some(HitRecord {
                        t,
                        p : ray.point_at_parameter(t),
                        normal : Vec3::from(1.0, 0.0, 0.0), // arbitrary???
                        material : &*self.phase_function,
                        u : 0.0,
                        v : 0.0
                    });
                }
            }
        } 

        Option::None
    }

    fn bounding_box(&self, time0 : f32, time1 : f32) -> Aabb {
        self.boundary.bounding_box(time0, time1)
    }
}