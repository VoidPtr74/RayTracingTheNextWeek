use crate::rng::Random;
use crate::texture::*;
use crate::material::*;
use crate::hitable::*;
use crate::camera::*;
use crate::vec3::*;

pub fn final_render(nx : usize, ny : usize, rnd : &mut Random) -> (Vec<Box<Hitable>>, Camera) {
    let nb = 20;
    let mut list : Vec<Box<Hitable>> = Vec::with_capacity(30);
    let mut boxlist : Vec<Box<Hitable>> = Vec::with_capacity(10000);
    let mut boxlist2 : Vec<Box<Hitable>> = Vec::with_capacity(10000);
    for i in 0..nb {
        for j in 0..nb {
            let w = 100.0;
            let x0 = -1000.0 + (i as f32)*w;
            let z0 = -1000.0 + (j as f32)*w;
            let y0 = 0.0;
            let x1 = x0 + w;
            let y1 = 100.0*(rnd.gen() + 0.01);
            let z1 = z0 + w;
            let ground = Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.48, 0.83, 0.53)));
            boxlist.push(BoxShape::new_from(&Vec3::from(x0, y0, z0), &Vec3::from(x1, y1, z1), Box::new(ground)));
        }
    }
    list.push(BvhTree::build(&mut boxlist, rnd, 0.0, 0.0).root);
    let light = Box::new(DiffuseLight { emit: ConstantTexture::new_with_colour(Vec3::from(7.0, 7.0, 7.0))});
    list.push(Box::new(XzRect { x0: 123.0, x1: 423.0, z0: 147.0, z1: 412.0, y: 554.0, material: light}));
    let center = Vec3::from(400.0, 400.0, 200.0);
    let moving_material = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.7, 0.3, 0.1))));
    list.push(build_moving_sphere(center, center + Vec3::from(30.0, 0.0, 0.0), 50.0, moving_material, 0.0, 1.0));
    list.push(build_sphere(Vec3::from(260.0, 150.0, 45.0), 50.0, Box::new(Dielectric::with_refraction_index(1.5))));
    list.push(build_sphere(Vec3::from(-60.0, 115.0, 25.0), 50.0, Box::new(Dielectric::with_refraction_index(1.5))));
    list.push(build_sphere(Vec3::from(0.0, 150.0, 145.0), 50.0, Metal::build_new(Vec3::from(0.8, 0.8, 0.9), 10.0)));
    let boundary = build_sphere(Vec3::from(360.0, 150.0, 145.0), 70.0, Box::new(Dielectric::with_refraction_index(1.5)));
    list.push(boundary);
    let boundary = build_sphere(Vec3::from(360.0, 150.0, 145.0), 70.0, Box::new(Dielectric::with_refraction_index(1.5)));
    list.push(ConstantMedium::build_new(0.2, boundary, ConstantTexture::new_with_colour(Vec3::from(0.2, 0.4, 0.9))));
    let boundary = build_sphere(Vec3::from(0.0, 0.0, 0.0), 5000.0, Box::new(Dielectric::with_refraction_index(1.5)));
    list.push(ConstantMedium::build_new(0.0001, boundary, ConstantTexture::new_with_colour(Vec3::from(1.0, 1.0, 1.0))));

    let img = Box::new(ImageTexture::load(String::from("land_ocean_ice_cloud_2048.png")));
    let mat = Box::new(Lambertian::with_texture(img));
    list.push(build_sphere(Vec3::from(400.0, 200.0, 400.0), 100.0, mat));

    let pertext = NoiseTexture::build(rnd, 0.1);
    list.push(build_sphere(Vec3::from(220.0, 280.0, 300.0),  80.0, Box::new(Lambertian::with_texture(pertext))));

    for j in 0..1000 {
        let white = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.73, 0.73, 0.73))));
        boxlist2.push(build_sphere(Vec3::from(165.0*rnd.gen(), 165.0 * rnd.gen(), 165.0*rnd.gen()), 10.0, white));
    }
    let collection = Translate {
        offset: Vec3::from(-100.0, 270.0, 395.0),
        obj : RotateY::create_new(
            BvhTree::build(&mut boxlist2, rnd, 0.0, 1.0).root, 15.0)
    };
    list.push(Box::new(collection));

    let look_from = Vec3::from(478.0, 278.0, -600.0);
    let look_at = Vec3::from(278.0, 278.0, 0.0);
    let focus_distance = 10.0;
    let aperture = 0.0;
    let camera = Camera::build(&look_from, &look_at, &Vec3::from(0.0, 1.0, 0.0), 
        40.0, (nx as f32) / (ny as f32), aperture, focus_distance, 0.0, 1.0);

    (list, camera)
}

pub fn cornell_smoke(nx : usize, ny : usize) -> (Vec<Box<Hitable>>, Camera) {
    let mut list : Vec<Box<Hitable>> = Vec::with_capacity(8);
    
    let red = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.65, 0.05, 0.05))));
    let white = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.73, 0.73, 0.73))));
    let white2 = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.73, 0.73, 0.73))));
    let white3 = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.73, 0.73, 0.73))));
    let white4 = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.73, 0.73, 0.73))));
    let white5 = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.73, 0.73, 0.73))));
    let green = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.12, 0.45, 0.15))));
    let light = Box::new(DiffuseLight {emit: ConstantTexture::new_with_colour(Vec3::from(7.0, 7.0, 7.0))});

    list.push(FlipNormals::new_with_obj(Box::new(YzRect {y0: 0.0, y1: 555.0, z0: 0.0, z1: 555.0, x: 555.0, material: green})));
    list.push(Box::new(YzRect {y0: 0.0, y1: 555.0, z0: 0.0, z1: 555.0, x: 0.0, material: red}));
    list.push(Box::new(XzRect {x0: 113.0, x1: 443.0, z0: 127.0, z1: 432.0, y: 554.0, material: light}));
    list.push(FlipNormals::new_with_obj(Box::new(XzRect {x0: 0.0, x1: 555.0, z0: 0.0, z1: 555.0, y: 555.0, material: white})));
    list.push(Box::new(XzRect {x0: 0.0, x1: 555.0, z0: 0.0, z1: 555.0, y: 0.0, material: white2}));
    list.push(FlipNormals::new_with_obj(Box::new(XyRect {x0: 0.0, x1: 555.0, y0: 0.0, y1: 555.0, z: 555.0, material: white3})));

    let box1 = 
        Box::new(Translate { 
            offset: Vec3::from(130.0, 0.0,  65.0), 
            obj: RotateY::create_new(
                BoxShape::new_from(&Vec3::from(0.0, 0.0, 0.0), &Vec3::from(165.0, 165.0, 165.0), white4),
                -18.0
            )
        });

    let box2 = 
        Box::new(Translate { 
            offset: Vec3::from(265.0, 0.0, 295.0), 
            obj: RotateY::create_new(
                BoxShape::new_from(&Vec3::from(0.0, 0.0, 0.0), &Vec3::from(165.0, 330.0, 165.0), white5),
                15.0
            )
        });
    list.push(ConstantMedium::build_new(0.01, box1, ConstantTexture::new_with_colour(Vec3::from(1.0, 1.0, 1.0))));
    list.push(ConstantMedium::build_new(0.01, box2, ConstantTexture::new_with_colour(Vec3::from(0.0, 0.0, 0.0))));

    let look_from = Vec3::from(278.0, 278.0, -800.0);
    let look_at = Vec3::from(278.0, 278.0, 0.0);
    let focus_distance = 10.0;
    let aperture = 0.0;
    let camera = Camera::build(&look_from, &look_at, &Vec3::from(0.0, 1.0, 0.0), 
        40.0, (nx as f32) / (ny as f32), aperture, focus_distance, 0.0, 1.0);
    (list, camera)
}

pub fn cornell_box(nx : usize, ny : usize) -> (Vec<Box<Hitable>>, Camera) {
    let mut list : Vec<Box<Hitable>> = Vec::with_capacity(7);

    let red = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.65, 0.05, 0.05))));
    let white = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.73, 0.73, 0.73))));
    let white2 = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.73, 0.73, 0.73))));
    let white3 = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.73, 0.73, 0.73))));
    let white4 = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.73, 0.73, 0.73))));
    let white5 = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.73, 0.73, 0.73))));
    let green = Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.12, 0.45, 0.15))));
    let light = Box::new(DiffuseLight {emit: ConstantTexture::new_with_colour(Vec3::from(15.0, 15.0, 15.0))});
    list.push(FlipNormals::new_with_obj(Box::new(YzRect {y0: 0.0, y1: 555.0, z0: 0.0, z1: 555.0, x: 555.0, material: green})));
    list.push(Box::new(YzRect {y0: 0.0, y1: 555.0, z0: 0.0, z1: 555.0, x: 0.0, material: red}));
    list.push(Box::new(XzRect {x0: 213.0, x1: 343.0, z0: 227.0, z1: 332.0, y: 554.0, material: light}));
    list.push(FlipNormals::new_with_obj(Box::new(XzRect {x0: 0.0, x1: 555.0, z0: 0.0, z1: 555.0, y: 555.0, material: white})));
    list.push(Box::new(XzRect {x0: 0.0, x1: 555.0, z0: 0.0, z1: 555.0, y: 0.0, material: white2}));
    list.push(FlipNormals::new_with_obj(Box::new(XyRect {x0: 0.0, x1: 555.0, y0: 0.0, y1: 555.0, z: 555.0, material: white3})));

    list.push(
        Box::new(Translate { 
            offset: Vec3::from(130.0, 0.0,  65.0), 
            obj: RotateY::create_new(
                BoxShape::new_from(&Vec3::from(0.0, 0.0, 0.0), &Vec3::from(165.0, 165.0, 165.0), white4),
                -18.0
            )
        })
    );

    list.push(
        Box::new(Translate { 
            offset: Vec3::from(265.0, 0.0, 295.0), 
            obj: RotateY::create_new(
                BoxShape::new_from(&Vec3::from(0.0, 0.0, 0.0), &Vec3::from(165.0, 330.0, 165.0), white5),
                15.0
            )
        })
    );

    let look_from = Vec3::from(278.0, 278.0, -800.0);
    let look_at = Vec3::from(278.0, 278.0, 0.0);
    let focus_distance = 10.0;
    let aperture = 0.0;
    let camera = Camera::build(&look_from, &look_at, &Vec3::from(0.0, 1.0, 0.0), 
        40.0, (nx as f32) / (ny as f32), aperture, focus_distance, 0.0, 1.0);
    (list, camera)
}

pub fn simple_light(nx : usize, ny : usize, rnd: &mut Random) -> (Vec<Box<Hitable>>, Camera) {
    let mut list : Vec<Box<Hitable>> = Vec::with_capacity(4);

    let perlin = NoiseTexture::build(rnd, 4.0);
    list.push(build_sphere(Vec3::from(0.0, -1000.0, 0.0), 1000.0, Box::new(Lambertian::with_texture(perlin))));
    // let perlin = NoiseTexture::build(rnd, 4.0);
    let perlin = Box::new(ImageTexture::load(String::from("land_ocean_ice_cloud_2048.png")));
    list.push(build_sphere(Vec3::from(0.0, 2.0, 0.0), 2.0, Box::new(Lambertian::with_texture(perlin))));
    list.push(build_sphere(Vec3::from(0.0, 7.0, 0.0), 2.0, Box::new(DiffuseLight { emit: ConstantTexture::new_with_colour(Vec3::from(4.0, 4.0, 4.0))})));
    list.push(Box::new(XyRect {x0: 3.0, x1: 5.0, y0: 1.0, y1: 3.0, z: -2.0, material: Box::new(DiffuseLight { emit: ConstantTexture::new_with_colour(Vec3::from(4.0, 4.0, 4.0))})} ));

    let look_from = Vec3::from(13.0, 2.0, 3.0);
    let look_at = Vec3::from(0.0, 2.0, 0.0);
    let focus_distance = 10.0;
    let aperture = 0.0;
    let camera = Camera::build(&look_from, &look_at, &Vec3::from(0.0, 1.0, 0.0), 
        40.0, (nx as f32) / (ny as f32), aperture, focus_distance, 0.0, 1.0);
    (list, camera)
}

pub fn earth_scene(nx : usize, ny : usize) -> (Vec<Box<Hitable>>, Camera) {
    let image_texture = ImageTexture::load(String::from("land_ocean_ice_cloud_2048.png"));
    let mut list : Vec<Box<Hitable>> = Vec::with_capacity(1);
    let earth = build_sphere(Vec3::from(0.0, 0.0, 0.0), 2.0, Box::new(Lambertian::with_texture(Box::new(image_texture))));
    list.push(earth);

    let look_from = Vec3::from(13.0, 2.0, 3.0);
    let look_at = Vec3::from(0.0, 0.0, 0.0);
    let focus_distance = 10.0;
    let aperture = 0.0;
        let camera = Camera::build(&look_from, &look_at, &Vec3::from(0.0, 1.0, 0.0), 
        20.0, (nx as f32) / (ny as f32), aperture, focus_distance, 0.0, 1.0);
        (list, camera)
}

pub fn two_perlin_spheres(nx : usize, ny : usize, rnd : &mut Random) -> (Vec<Box<Hitable>>, Camera) {
    let noise_texture = NoiseTexture::build(rnd, 4.0);
    let mut list : Vec<Box<Hitable>> = Vec::with_capacity(2);
    list.push(build_sphere(Vec3::from(0.0, -1000.0, 0.0), 1000.0, Box::new(Lambertian::with_texture(noise_texture))));
    let noise_texture = NoiseTexture::build(rnd, 4.0);
    list.push(build_sphere(Vec3::from(0.0, 2.0, 0.0), 2.0, Box::new(Lambertian::with_texture(noise_texture))));

    let look_from = Vec3::from(13.0, 2.0, 3.0);
    let look_at = Vec3::from(0.0, 0.0, 0.0);
    let focus_distance = 10.0;
    let aperture = 0.0;
    let camera = Camera::build(&look_from, &look_at, &Vec3::from(0.0, 1.0, 0.0), 
        20.0, (nx as f32) / (ny as f32), aperture, focus_distance, 0.0, 1.0);
    (list, camera)
}

pub fn two_spheres(nx : usize, ny : usize) -> (Vec<Box<Hitable>>, Camera) {
    let look_from = Vec3::from(13.0, 2.0, 3.0);
    let look_at = Vec3::from(0.0, 0.0, 0.0);
    let focus_distance = 10.0;
    let aperture = 0.0;
    let camera = Camera::build(&look_from, &look_at, &Vec3::from(0.0, 1.0, 0.0), 
        20.0, (nx as f32) / (ny as f32), aperture, focus_distance, 0.0, 1.0);
    let checker_texture = CheckerTexture::new_with_textures(
        ConstantTexture::new_with_colour(Vec3::from(0.2,0.3,0.1)), 
        ConstantTexture::new_with_colour(Vec3::from(0.9,0.9,0.9))
        );
    let mut list: Vec<Box<Hitable>> = Vec::with_capacity(2);
    list.push(build_sphere(Vec3::from(0.0, -10.0, 0.0), 10.0, Box::new(Lambertian::with_texture(checker_texture))));
    let checker_texture = CheckerTexture::new_with_textures(
        ConstantTexture::new_with_colour(Vec3::from(0.2,0.3,0.1)), 
        ConstantTexture::new_with_colour(Vec3::from(0.9,0.9,0.9))
        );
    list.push(build_sphere(Vec3::from(0.0,  10.0, 0.0), 10.0, Box::new(Lambertian::with_texture(checker_texture))));
    
    (list, camera)
}

pub fn random_moving_scene(nx : usize, ny : usize, rnd: &mut Random, time_start: f32, time_end: f32) -> (Vec<Box<Hitable>>, Camera) {
    let look_from = Vec3::from(13.0, 2.0, 3.0);
    let look_at = Vec3::from(0.0, 0.0, 0.0);
    let aperture = 0.1;
    let dist_to_focus = 10.0;
    let time_start = 0.0;
    let time_end = 1.0;
    let camera = Camera::build(
        &look_from,
        &look_at,
        &Vec3::from(0.0, 1.0, 0.0),
        10.0,
        nx as f32 / (ny as f32),
        aperture,
        dist_to_focus,
        time_start,
        time_end
    );

    let n = 500;
    let mut list: Vec<Box<Hitable>> = Vec::with_capacity(n + 1);
    let checker_texture = CheckerTexture::new_with_textures(
        ConstantTexture::new_with_colour(Vec3::from(0.2,0.3,0.1)), 
        ConstantTexture::new_with_colour(Vec3::from(0.9,0.9,0.9))
        );
    list.push(build_sphere(
        Vec3::from(0.0, -1000.0, 0.0),
        1000.0,
        Box::new(Lambertian::with_texture(checker_texture)),
    ));
    for a in -11..11i16 {
        for b in -11..11i16 {
            let choose_mat = rnd.gen();
            let center = Vec3::from(
                f32::from(a) + 0.9 * rnd.gen(),
                0.2,
                f32::from(b) + 0.9 * rnd.gen(),
            );
            if (center - Vec3::from(4.0, 0.2, 0.0)).length() > 0.9 {
                let sphere : Box<Hitable> = match choose_mat {
                    x if x < 0.8 => build_moving_sphere(center, center + Vec3::from(0.0, 0.5*rnd.gen(), 0.0), 0.2, Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(
                        rnd.gen() * rnd.gen(),
                        rnd.gen() * rnd.gen(),
                        rnd.gen() * rnd.gen(),
                    )))), time_start, time_end),
                    x if x < 0.95 => build_sphere(center, 0.2, Box::new(Metal::with_albedo(Vec3::from(
                        0.5 * (1.0 + rnd.gen()),
                        0.5 * (1.0 + rnd.gen()),
                        0.5 * (1.0 + rnd.gen()),
                    )))),
                    _ => build_sphere(center, 0.2, Box::new(Dielectric::with_refraction_index(1.5))),
                };
                list.push(sphere);
            }
        }
    }

    list.push(build_sphere(
        Vec3::from(0.0, 1.0, 0.0),
        1.0,
        Box::new(Dielectric::with_refraction_index(1.5)),
    ));
    list.push(build_sphere(
        Vec3::from(-4.0, 1.0, 0.0),
        1.0,
        Box::new(Lambertian::with_texture(ConstantTexture::new_with_colour(Vec3::from(0.4, 0.2, 0.1)))),
    ));
    list.push(build_sphere(
        Vec3::from(4.0, 1.0, 0.0),
        1.0,
        Box::new(Metal::with_albedo(Vec3::from(0.7, 0.6, 0.5))),
    ));

    (list, camera)
}



fn build_sphere(center: Vec3, radius: f32, material: Box<Material>) -> Box<Hitable> {
    let sphere = Sphere {
        center,
        radius,
        material,
    };
    Box::new(sphere)
}

fn build_moving_sphere(center_start: Vec3, center_end: Vec3, radius: f32, material: Box<Material>, time_start: f32, time_end: f32) -> Box<Hitable> {
    let sphere = MovingSphere {
        center_start,
        center_end,
        radius,
        material,
        time_start,
        time_end
    };
    Box::new(sphere)
}
