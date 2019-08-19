use crate::vec3::*;
use crate::rng::Random;

pub struct Perlin {
    perm_x : [usize; 256],
    perm_y : [usize; 256],
    perm_z : [usize; 256],
    random_floats : [Vec3; 256]
}

impl Perlin {
    pub fn turb(&self, p : &Vec3, depth : i32) -> f32 {
        let mut accum = 0.0;
        let mut temp_p : Vec3 = p.clone();
        let mut weight = 1.0;

        for _ in 0..depth {
            accum += weight * self.noise(&temp_p);
            weight *= 0.5;
            temp_p *= 2.0;
        }
        accum.abs()
    }

    pub fn noise(&self, p : &Vec3) -> f32 {
        let u = p.x() - p.x().floor();
        let v = p.y() - p.y().floor();
        let w = p.z() - p.z().floor();
        let i = p.x().floor() as i32;
        let j = p.y().floor() as i32;
        let k = p.z().floor() as i32;
        let mut c = [[[Vec3::from(0.0,0.0,0.0);2];2];2];
        for di in 0..2 {
            let i_ind = ((i + di as i32) & 255) as usize;
            for dj in 0..2 {
                let j_ind = ((j + dj as i32) & 255) as usize;
                for dk in 0..2 {
                    let k_ind = ((k + dk as i32) & 255) as usize;
                    c[di][dj][dk] = self.random_floats[self.perm_x[i_ind] ^ self.perm_y[j_ind] ^ self.perm_z[k_ind]];
                }
            }
        }

        Perlin::interpolate(u, v, w, c)
    }

    pub fn build(rnd : &mut Random) -> Perlin {
        Perlin {
            perm_x : Perlin::perlin_generate_perm(rnd),
            perm_y : Perlin::perlin_generate_perm(rnd),
            perm_z : Perlin::perlin_generate_perm(rnd),
            random_floats : Perlin::perlin_generate(rnd),
        }
    }

    fn perlin_generate(rnd : &mut Random) -> [Vec3; 256] {
        let mut array : [Vec3; 256] = [Vec3::from(0.0,0.0,0.0); 256];
        for index in 0..array.len() {
            array[index] = (&Vec3::from(rnd.gen() - 0.5, rnd.gen() - 0.5, rnd.gen() - 0.5)*2.0).make_normalised();
        }
        array
    }

    fn perlin_generate_perm(rnd : &mut Random) -> [usize; 256] {
        let mut array : [usize; 256] = [0;256];
        for index in 0..array.len() {
            array[index] = index;
        }
        Perlin::permute(&mut array, rnd);
        array
    }

    fn permute(array : &mut [usize; 256], rnd : &mut Random) {
        for index in (0..256).rev() {
            let target = (rnd.gen() * (index + 1) as f32) as usize;
            let tmp = array[index];
            array[index] = array[target];
            array[target] = tmp;
        }
    }

    fn interpolate(u : f32, v : f32, w : f32, c : [[[Vec3; 2]; 2]; 2]) -> f32 {
        let uu = u*u*(3.0 - 2.0*u);
        let vv = v*v*(3.0 - 2.0*v);
        let ww = w*w*(3.0 - 2.0*w);
        let mut accum = 0.0;
        for i in 0..2 {
            let ix = i as f32;
            let i_component = ix*uu + (1.0 - ix) * (1.0 - uu);
            for j in 0..2 {
                let jx = j as f32;
                let j_component = jx*vv + (1.0-jx)*(1.0-vv);
                for k in 0..2 {
                    let kx = k as f32;
                    let weight_v = Vec3::from(uu - ix, vv - jx, ww - kx);
                    let k_component = kx*ww + (1.0-kx)*(1.0-ww);
                    accum += i_component * j_component * k_component * dot(&c[i][j][k], &weight_v);
                }
            }
        }

        accum
    }
}