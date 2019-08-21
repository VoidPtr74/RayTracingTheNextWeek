use crate::ray::*;
use crate::rng::Random;
use crate::vec3::*;
use crate::texture::Texture;

pub trait Material: Send + Sync {
    fn scatter(
        &self,
        ray: &Ray,
        rec: &HitRecord,
        rnd: &mut Random,
        attenuation: &mut Vec3,
        scattered: &mut Ray,
    ) -> bool;

    fn emitted(&self, _u : f32, _v : f32, _p : &Vec3) -> Vec3 {
        Vec3::from(0.0,0.0,0.0)
    }
}

pub struct Lambertian {
    albedo : Box<Texture>
}

pub struct Metal {
    albedo: Vec3,
    fuzz: f32
}

pub struct Dielectric {
    refraction_index: f32,
}

impl Lambertian {
    pub fn with_texture(texture: Box<Texture>) -> Lambertian {
        Lambertian { albedo : texture }
    }
}

impl Material for Lambertian {
    fn scatter(
        &self,
        ray: &Ray,
        rec: &HitRecord,
        rnd: &mut Random,
        attenuation: &mut Vec3,
        scattered: &mut Ray,
    ) -> bool {
        let target = rec.p + rec.normal + random_in_unit_sphere(rnd);
        scattered.origin = rec.p;
        scattered.direction = target - rec.p;
        scattered.time = ray.time;
        attenuation.set(&self.albedo.colour(rec.u, rec.v, &rec.p));
        true
    }
}

impl Metal {
    pub fn with_albedo(albedo: Vec3) -> Metal {
        Metal { albedo, fuzz: 0.0 }
    }

    pub fn build_new(albedo: Vec3, fuzz : f32) -> Box<Metal> {
        Box::new(Metal { albedo, fuzz })
    }
}

impl Material for Metal {
    fn scatter(
        &self,
        ray: &Ray,
        rec: &HitRecord,
        rnd: &mut Random,
        attenuation: &mut Vec3,
        scattered: &mut Ray,
    ) -> bool {
        let reflected = reflect(&ray.direction.make_normalised(), &rec.normal);
        scattered.origin = rec.p;
        scattered.direction = reflected + &random_in_unit_sphere(rnd) * self.fuzz;
        scattered.time = ray.time;
        attenuation.set(&self.albedo);
        dot(&scattered.direction, &rec.normal) > 0.0
    }
}

impl Dielectric {
    pub fn with_refraction_index(refraction_index: f32) -> Dielectric {
        Dielectric { refraction_index }
    }
}

fn schlick(cosine: f32, refraction_index: f32) -> f32 {
    let r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

impl Material for Dielectric {
    fn scatter(
        &self,
        ray: &Ray,
        rec: &HitRecord,
        rnd: &mut Random,
        attenuation: &mut Vec3,
        scattered: &mut Ray,
    ) -> bool {
        let reflected = reflect(&ray.direction, &rec.normal);
        attenuation.set(&Vec3::from(1.0, 1.0, 1.0));
        let outward_normal: Vec3;
        let ni_over_nt: f32;
        let incident_dot_normal = dot(&ray.direction, &rec.normal);
        let cosine = if incident_dot_normal > 0.0 {
            outward_normal = &rec.normal * -1.0;
            ni_over_nt = self.refraction_index;
            self.refraction_index * incident_dot_normal / ray.direction.length()
        } else {
            outward_normal = rec.normal;
            ni_over_nt = 1.0 / self.refraction_index;
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
                let reflect_prob = schlick(cosine, self.refraction_index);
                if rnd.gen() < reflect_prob {
                    scattered.direction = reflected
                } else {
                    scattered.direction.set(refracted)
                }
            }
        };

        true
    }
}

pub struct DiffuseLight {
    pub emit : Box<Texture>
}

impl Material for DiffuseLight {
    fn scatter(
        &self,
        ray: &Ray,
        rec: &HitRecord,
        rnd: &mut Random,
        attenuation: &mut Vec3,
        scattered: &mut Ray,
    ) -> bool {
        false
    }

    fn emitted(&self, u : f32, v : f32, p : &Vec3) -> Vec3 {
        self.emit.colour(u, v, p)
    }
}

pub struct Isotropic {
    pub albedo : Box<Texture>
}

impl Material for Isotropic {
    fn scatter(
        &self,
        ray: &Ray,
        rec: &HitRecord,
        rnd: &mut Random,
        attenuation: &mut Vec3,
        scattered: &mut Ray,
    ) -> bool {
        *scattered = Ray {
            origin: rec.p,
            direction: random_in_unit_sphere(rnd),
            time: ray.time
        };
        *attenuation = self.albedo.colour(rec.u, rec.v, &rec.p);
        true
    }
}