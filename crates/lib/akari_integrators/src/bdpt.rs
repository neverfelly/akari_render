use bumpalo::Bump;

use crate::bidir::*;
use crate::film::*;
use crate::sampler::PCGSampler;
use crate::sampler::Pcg;
use crate::sampler::Sampler;
use crate::util::PerThread;
use crate::*;
pub struct Bdpt {
    pub spp: u32,
    pub max_depth: usize,
    pub debug: bool,
}

impl Integrator for Bdpt {
    fn render(&mut self, scene: &Scene) -> Film {
        let npixels = (scene.camera.resolution().x * scene.camera.resolution().y) as usize;
        let film = Film::new(&scene.camera.resolution());
        let mut pyramid = Vec::new();
        for _t in 1..=self.max_depth + 2 {
            for _s in 0..self.max_depth + 2 {
                pyramid.push(Film::new(&scene.camera.resolution()));
            }
        }
        let get_index = |s, t| (t - 1) as usize * (3 + self.max_depth) + s as usize;
        let chunks = (npixels + 255) / 256;
        let progress = crate::util::create_progess_bar(chunks, "chunks");
        let arenas = PerThread::new(|| Bump::new());
        parallel_for(npixels, 256, |id| {
            let mut sampler = PCGSampler {
                rng: Pcg::new(id as u64),
            };
            let x = (id as u32) % scene.camera.resolution().x;
            let y = (id as u32) / scene.camera.resolution().x;
            let pixel = uvec2(x, y);
            let mut acc_li = Spectrum::zero();

            let mut debug_acc = vec![];
            if self.debug {
                for _t in 1..=self.max_depth + 2 {
                    for _s in 0..=self.max_depth + 2 {
                        debug_acc.push(Spectrum::zero());
                    }
                }
            }
            for _ in 0..self.spp {
                let arena = arenas.get_mut();
                {
                    let mut camera_path = Path::new(arena, self.max_depth + 2);
                    let mut light_path = Path::new(arena, self.max_depth + 1);
                    let mut new_camera_path = Path::new(arena, self.max_depth + 2);
                    let mut new_light_path = Path::new(arena, self.max_depth + 1);
                    sampler.start_next_sample();
                    bdpt::generate_camera_path(
                        scene,
                        pixel,
                        &mut sampler,
                        self.max_depth + 2,
                        &mut camera_path,
                        arena,
                    );
                    bdpt::generate_light_path(
                        scene,
                        &mut sampler,
                        self.max_depth + 1,
                        &mut light_path,
                        arena,
                    );
                    for t in 1..=camera_path.len() as isize {
                        for s in 0..=light_path.len() as isize {
                            let depth = s + t - 2;
                            if (s == 1 && t == 1) || depth < 0 || depth > self.max_depth as isize {
                                continue;
                            }
                            let (li, weight, raster) = bdpt::connect_paths(
                                scene,
                                bdpt::ConnectionStrategy {
                                    s: s as usize,
                                    t: t as usize,
                                },
                                &light_path,
                                &camera_path,
                                &mut sampler,
                                &mut new_light_path,
                                &mut new_camera_path,
                            );
                            if t == 1 {
                                if let Some(raster) = raster {
                                    film.add_splat(raster, li * weight / self.spp as f32);
                                    if self.debug {
                                        // let l =li * weight;
                                        pyramid[get_index(s, t)]
                                            .add_splat(raster, li * weight / self.spp as f32);
                                    }
                                }
                            } else {
                                if self.debug {
                                    // let l =li * weight;
                                    debug_acc[get_index(s, t)] += li * weight;
                                }
                                acc_li += li * weight;
                            }
                        }
                    }
                    light_path.clear();
                    camera_path.clear();
                }
                arena.reset();
            }
            acc_li = acc_li / (self.spp as f32);

            film.add_sample(uvec2(x, y), acc_li, 1.0);

            if self.debug {
                for t in 2..=(self.max_depth + 2) as isize {
                    for s in 0..=(self.max_depth + 2) as isize {
                        let depth = s + t - 2;
                        if (s == 1 && t == 1) || depth < 0 || depth > self.max_depth as isize {
                            continue;
                        }
                        let idx = get_index(s, t);
                        pyramid[idx].add_sample(
                            uvec2(x, y),
                            debug_acc[idx] / (self.spp as f32) as f32,
                            1.0,
                        );
                    }
                }
            }
            if (id + 1) % 256 == 0 {
                progress.inc(1);
            }
        });
        progress.finish();
        if self.debug {
            for t in 1..=(self.max_depth + 2) as isize {
                for s in 0..=(self.max_depth + 2) as isize {
                    let depth = s + t - 2;
                    if (s == 1 && t == 1) || depth < 0 || depth > self.max_depth as isize {
                        continue;
                    }
                    let idx = get_index(s, t);
                    let film = &pyramid[idx];
                    film.write_exr(&format!("bdpt-d{}-s{}-t{}.exr", depth, s, t));
                }
            }
        }
        film
    }
}
