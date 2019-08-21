
pub struct XyRect {
    pub x0 : f32,
    pub x1 : f32, 
    pub y0 : f32,
    pub y1 : f32,
    pub z  : f32,
    pub material : Box<Material>
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
    pub material : Box<Material>
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
    pub material : Box<Material>
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
