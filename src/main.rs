mod aabb;
mod camera;
mod hitable;
mod material;
mod ray;
mod rng;
mod vec3;
mod texture;
mod perlin;
mod scenes;

extern crate stb_image;

use camera::Camera;
use hitable::*;
use material::*;
use ray::Ray;
use rng::Random;
use std::f32;
use vec3::Vec3;
use texture::*;
use perlin::*;
use scenes::*;


fn main() {
    let mut rnd = Random::create_with_seed(42);
    let nx = 1280;
    let ny =  720;

    let time_start = 0.0;
    let time_end = 1.0;

    // let (mut hitable_list, camera) = random_moving_scene(nx, ny, &mut rnd, time_start, time_end);
    // let (mut hitable_list, camera) = two_spheres(nx, ny);
    // let (mut hitable_list, camera) = two_perlin_spheres(nx, ny, &mut rnd);
    // let (mut hitable_list, camera) = earth_scene(nx, ny);
    // let (mut hitable_list, camera) = simple_light(nx, ny, &mut rnd);
    // let (mut hitable_list, camera) = cornell_box(nx, ny);
    // let (mut hitable_list, camera) = cornell_smoke(nx, ny);
    let (mut hitable_list, camera) = final_render(nx, ny, &mut rnd);
    let bvh_tree = BvhTree::build(&mut hitable_list, &mut rnd, time_start, time_end);

    let samples_per_pixel = 16384;
    let thread_count = 24;

    // let cols = render_single_thread(&camera, nx, ny, samples_per_pixel, &bvh_tree, &mut rnd);
    let cols = render_multi_thread(
        camera,
        nx,
        ny,
        samples_per_pixel,
        bvh_tree,
        &mut rnd,
        thread_count,
    );

    print!("P3\n{} {}\n255\n", nx, ny);
    for col in cols.iter() {
        let c = col.clamp(0.0, 1.0);
        let ir = (255.99 * c.r()) as i32;
        let ig = (255.99 * c.g()) as i32;
        let ib = (255.99 * c.b()) as i32;
        println!("{} {} {}", ir, ig, ib);
    }
}

fn render_multi_thread(
    camera: Camera,
    nx: usize,
    ny: usize,
    samples_per_pixel: i16,
    bvh_tree: BvhTree,
    _: &mut Random,
    thread_count: usize,
) -> Vec<Vec3> {
    let mut cols: Vec<Vec3> = Vec::with_capacity(nx * ny);
    let mut workers: Vec<std::thread::JoinHandle<std::vec::Vec<vec3::Vec3>>> =
        Vec::with_capacity(thread_count);
    let nxd = nx as f32;
    let nyd = ny as f32;

    let arc_tree = std::sync::Arc::new(bvh_tree);

    for thread_index in (0..thread_count).rev() {
        let local_bvh = arc_tree.clone();
        let thread_seed = 1234 * thread_index as u64;
        let (y0, y1) = get_segment(thread_count, thread_index, ny);

        let thd = std::thread::spawn(move || {
            let mut rnd = Random::create_with_seed(thread_seed);
            let mut cols: Vec<Vec3> = Vec::with_capacity(nx * ny);
            for y in (y0..y1).rev() {
                let yd = y as f32;
                for x in 0..nx {
                    let mut col = Vec3::from(0.0, 0.0, 0.0);
                    let xd = x as f32;
                    for _ in 0..samples_per_pixel {
                        let u = (xd + rnd.gen()) / nxd;
                        let v = (yd + rnd.gen()) / nyd;
                        let r = camera.get_ray(u, v, &mut rnd);
                        col += &colour(&r, local_bvh.as_ref(), &mut rnd);
                    }

                    col /= f32::from(samples_per_pixel);
                    col = Vec3::from(col.x().sqrt(), col.y().sqrt(), col.z().sqrt());
                    cols.push(col);
                }
            }

            cols
        });

        workers.push(thd);
    }

    for waiter in workers {
        let mut result = waiter.join().unwrap();
        cols.append(&mut result);
    }

    cols
}

fn render_single_thread(
    camera: &Camera,
    nx: usize,
    ny: usize,
    samples_per_pixel: i16,
    bvh_tree: &BvhTree,
    rnd: &mut Random,
) -> Vec<Vec3> {
    let nxd = nx as f32;
    let nyd = ny as f32;

    let mut cols: Vec<Vec3> = Vec::with_capacity(nx * ny);
    for y in (0..ny).rev() {
        let yd = y as f32;
        for x in 0..nx {
            let mut col = Vec3::from(0.0, 0.0, 0.0);
            let xd = x as f32;
            for _ in 0..samples_per_pixel {
                let u = (xd + rnd.gen()) / nxd;
                let v = (yd + rnd.gen()) / nyd;
                let r = camera.get_ray(u, v, rnd);
                col += &colour(&r, &bvh_tree, rnd);
            }

            col /= f32::from(samples_per_pixel);
            col = Vec3::from(col.x().sqrt(), col.y().sqrt(), col.z().sqrt());
            cols.push(col);
        }
    }

    cols
}

fn colour(ray: &Ray, world: &BvhTree, rnd: &mut Random) -> Vec3 {
    const MAX_DEPTH : usize = 20;
    const MAX_THING: f32 = 1.0e10;
    let mut accumulated_colour = Vec3::from(0.0, 0.0, 0.0);
    let mut go = true;
    let mut depth_stack : [(Vec3, Vec3); MAX_DEPTH] = [(Vec3::from(0.0, 0.0, 0.0), Vec3::from(0.0, 0.0, 0.0)); MAX_DEPTH];
    let mut index = 0;
    let mut current_ray = *ray;
    while index < MAX_DEPTH && go {
        let record = world.root.hit(&current_ray, 0.001, MAX_THING);
        depth_stack[index] = match record {
            None => {
                // Render "Sky"
                // let direction = ray.direction.make_normalised();
                // let t = 0.5 * (direction.y() + 1.0);

                // (&Vec3::from(1.0, 1.0, 1.0) * (1.0 - t)) + (&Vec3::from(0.5, 0.7, 1.0) * t)
                go = false;
                (Vec3::from(0.0,0.0,0.0),Vec3::from(0.0,0.0,0.0))
            }
            Some(rec) => {
                let mut scattered = Ray::default();
                let mut attenuation = Vec3::default();
                let emitted = rec.material.emitted(rec.u, rec.v, &rec.p);
                if rec.material.scatter(&current_ray, &rec, rnd, &mut attenuation, &mut scattered)
                {
                    current_ray = scattered;
                    (emitted, attenuation)
                } else {
                    go = false;
                    (emitted, Vec3::from(0.0, 0.0, 0.0))
                }
            }
        };
        index = index + 1;
    }

    for j in (0..index).rev() {
        let (emitted, attenuation) = &depth_stack[j];
        accumulated_colour = &accumulated_colour.direct_product(attenuation) + emitted;
    }


    accumulated_colour
}

fn get_segment(thread_count: usize, thread_index: usize, ny: usize) -> (usize, usize) {
    let segment_size = ny / thread_count;
    let lower = segment_size * thread_index;
    let upper = if thread_index == thread_count - 1 {
        ny
    } else {
        segment_size * (thread_index + 1)
    };
    (lower, upper)
}
