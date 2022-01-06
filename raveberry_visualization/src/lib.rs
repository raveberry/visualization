#[macro_use]
extern crate glium;

use crate::glium::glutin::platform::run_return::EventLoopExtRunReturn;
use glium::{glutin, Surface};
use lazy_static::lazy_static;
use palette::{Hsv, IntoColor, Srgb};
use pyo3::prelude::*;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime};

pub const BARS: u32 = 256;
const PARTICLE_SPAWN_Z: f32 = 2.0;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);

#[derive(Copy, Clone)]
struct Particle {
    translation: [f32; 2],
    start_z: f32,
    speed: f32,
}
implement_vertex!(Particle, translation, start_z, speed);

static mut ACTIVE: AtomicBool = AtomicBool::new(false);
static mut SHOULD_EXIT: AtomicBool = AtomicBool::new(false);
lazy_static! {
    static ref MODULE_ROOT: Mutex<String> = Mutex::new("./".to_string());
    static ref AVG_FPS: Mutex<f32> = Mutex::new(-1.0);
    static ref ALARM_FACTOR: Mutex<f32> = Mutex::new(-1.0);
    static ref CURRENT_FRAME: Mutex<[f32; BARS as usize]> = Mutex::new([0.0; BARS as usize]);
}

#[pyclass]
pub struct Controller {}

#[pymethods]
impl Controller {
    #[new]
    pub fn new() -> Self {
        Controller {}
    }

    pub fn start(&self, variant: &str, ups: f32, num_particles: u32, fps_measure_window: f32) {
        let variant = variant.to_string();

        if self.get_variants().iter().all(|s| s != &variant) {
            eprintln!("Unknown variant given: {}", variant);
            return;
        }

        // stringly typed spawn description so we don't have to expose the enum to python
        *AVG_FPS.lock().unwrap() = ups;
        unsafe {
            ACTIVE.store(true, Ordering::Relaxed);
            SHOULD_EXIT.store(false, Ordering::Relaxed);
        }
        // We need the main thread to return, so we give up cross-platform compatibility
        // and commit to unix threads so we can run in a non-main thread.
        thread::spawn(move || {
            // the event_loop can not be part of Visualization because when calling event_loop.run
            // the struct would be moved, including the event_loop, resulting in a partially moved struct
            //let event_loop = glutin::event_loop::EventLoop::new();
            let event_loop = glutin::platform::unix::EventLoopExtUnix::new_any_thread();
            let visualization =
                Visualization::new(&event_loop, variant, ups, num_particles, fps_measure_window);
            visualization.start(event_loop);
            // start a second event loop that does nothing to destroy the previous window
            let mut event_loop: glutin::event_loop::EventLoop<()> =
                glutin::platform::unix::EventLoopExtUnix::new_any_thread();
            event_loop.run_return(move |_, _, control_flow| {
                *control_flow = glutin::event_loop::ControlFlow::Exit;
            });
            unsafe {
                ACTIVE.store(false, Ordering::Relaxed);
            }
        });
    }

    pub fn stop(&self) {
        unsafe {
            SHOULD_EXIT.store(true, Ordering::Relaxed);
        }
    }

    pub fn is_active(&self) -> bool {
        unsafe { ACTIVE.load(Ordering::Relaxed) }
    }

    pub fn get_variants(&self) -> Vec<String> {
        fs::read_dir(format!("{}/shaders/", *MODULE_ROOT.lock().unwrap()))
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
    }

    pub fn get_fps(&self) -> f32 {
        *AVG_FPS.lock().unwrap()
    }

    pub fn set_parameters(&self, alarm_factor: f32, current_frame: [f32; BARS as usize]) {
        *ALARM_FACTOR.lock().unwrap() = alarm_factor;
        *CURRENT_FRAME.lock().unwrap() = current_frame;
    }
}

struct Visualization {
    ups: f32,
    resolution: (f32, f32),
    display: glium::Display,
    quad_v: glium::VertexBuffer<Vertex>,
    quad_i: glium::index::NoIndices,
    background_program: glium::Program,
    foreground_program: glium::Program,
    spectrum_texture: glium::texture::Texture2d,
    logo_texture: glium::texture::Texture2d,
    particle_v: glium::VertexBuffer<Vertex>,
    particle_i: glium::index::NoIndices,
    particle_buffer: glium::VertexBuffer<Particle>,
    particle_program: glium::Program,
    last_loop: SystemTime,
    time_elapsed: Duration,
    total_intensity: f32,
    fps_counter: u32,
    last_fps_calc: SystemTime,
    fps_measure_window: f32,
}

impl Visualization {
    fn new(
        event_loop: &glutin::event_loop::EventLoop<()>,
        variant: String,
        ups: f32,
        num_particles: u32,
        fps_measure_window: f32,
    ) -> Visualization {
        let monitor_handle = event_loop.primary_monitor().unwrap();
        let resolution = (
            monitor_handle.size().width as f32,
            monitor_handle.size().height as f32,
        );
        let fs = glutin::window::Fullscreen::Borderless(Some(monitor_handle));
        let wb = glutin::window::WindowBuilder::new()
            .with_inner_size(glium::glutin::dpi::LogicalSize::new(
                resolution.0,
                resolution.1,
            ))
            .with_fullscreen(Some(fs))
            .with_title("Raveberry");
        let cb = glutin::ContextBuilder::new().with_vsync(true);
        let display = glium::Display::new(wb, cb, &event_loop).unwrap();

        let quad_v: glium::VertexBuffer<Vertex> = glium::VertexBuffer::empty(&display, 3).unwrap();
        let quad_i = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        let quad_vs = fs::read_to_string(format!(
            "{}/shaders/{}/quad.vs",
            *MODULE_ROOT.lock().unwrap(),
            variant
        ))
        .expect("Could not read vertex shader");
        let background_fs = fs::read_to_string(format!(
            "{}/shaders/{}/background.fs",
            *MODULE_ROOT.lock().unwrap(),
            variant
        ))
        .expect("Could not read vertex shader");
        let foreground_fs = fs::read_to_string(format!(
            "{}/shaders/{}/foreground.fs",
            *MODULE_ROOT.lock().unwrap(),
            variant
        ))
        .expect("Could not read vertex shader");

        // specify outputs_srgb in every shader for correct color space output
        // https://github.com/rust-windowing/glutin/issues/1175
        let background_program = match glium::Program::new(
            &display,
            glium::program::ProgramCreationInput::SourceCode {
                vertex_shader: &quad_vs,
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                geometry_shader: None,
                fragment_shader: &background_fs,
                transform_feedback_varyings: None,
                outputs_srgb: true,
                uses_point_size: false,
            },
        ) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{}", e);
                panic!();
            }
        };
        let foreground_program = match glium::Program::new(
            &display,
            glium::program::ProgramCreationInput::SourceCode {
                vertex_shader: &quad_vs,
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                geometry_shader: None,
                fragment_shader: &foreground_fs,
                transform_feedback_varyings: None,
                outputs_srgb: true,
                uses_point_size: false,
            },
        ) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{}", e);
                panic!();
            }
        };

        let spectrum_texture = glium::texture::Texture2d::empty(&display, BARS, 1).unwrap();

        let image = image::io::Reader::open(format!(
            "{}/images/raveberry.png",
            *MODULE_ROOT.lock().unwrap()
        ))
        .unwrap()
        .decode()
        .unwrap()
        .to_rgba8();
        let image_dimensions = image.dimensions();
        let image =
            glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        // don't use an sRGB texture because the shader is already configured to output sRGB
        let logo_texture = glium::texture::Texture2d::new(&display, image).unwrap();

        let mut vertices: Vec<Vertex> = Vec::new();
        vertices.push(Vertex {
            position: [0.0, 0.0],
        });
        let particle_v = glium::VertexBuffer::new(&display, &vertices).unwrap();
        let particle_i = glium::index::NoIndices(glium::index::PrimitiveType::Points);

        let mut particles = Vec::new();
        for _ in 0..num_particles {
            let (x, y, z) = if variant == "Circle" {
                let phi = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
                let radius_diff = rand::random::<f32>();
                let resolution_correction = resolution.1 / resolution.0;
                let x = phi.cos() * (0.6 + radius_diff * 0.2) * resolution_correction;
                let y = phi.sin() * (0.6 + radius_diff * 0.2);
                let z = PARTICLE_SPAWN_Z * rand::random::<f32>();
                (x, y, z)
            } else if variant == "SnowyCircle" {
                let x = rand::random::<f32>() * 2. - 1.;
                let y = rand::random::<f32>() * 4. - 2.;
                let z = rand::random::<f32>() * 2.;
                (x, y, z)
            } else {
                (0.0, 0.0, 0.0)
            };
            let speed = 0.3 * (rand::random::<f32>() * 0.75 + 0.3);

            particles.push(Particle {
                translation: [x, y],
                start_z: z,
                speed: speed,
            });
        }

        let particle_vs = fs::read_to_string(format!(
            "{}/shaders/{}/particle.vs",
            *MODULE_ROOT.lock().unwrap(),
            variant
        ))
        .expect("Could not read vertex shader");
        let particle_fs = fs::read_to_string(format!(
            "{}/shaders/{}/particle.fs",
            *MODULE_ROOT.lock().unwrap(),
            variant
        ))
        .expect("Could not read vertex shader");
        let particle_program =
            match glium::Program::from_source(&display, &particle_vs, &particle_fs, None) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{}", e);
                    panic!();
                }
            };
        let particle_buffer = glium::VertexBuffer::dynamic(&display, &particles).unwrap();

        Visualization {
            ups: ups,
            resolution: resolution,
            display: display,
            quad_v: quad_v,
            quad_i: quad_i,
            background_program: background_program,
            foreground_program: foreground_program,
            spectrum_texture: spectrum_texture,
            logo_texture: logo_texture,
            particle_v: particle_v,
            particle_i: particle_i,
            particle_buffer: particle_buffer,
            particle_program: particle_program,
            last_loop: SystemTime::now(),
            time_elapsed: Duration::new(0, 0),
            total_intensity: 0.0,
            fps_counter: 0,
            last_fps_calc: SystemTime::now(),
            fps_measure_window: fps_measure_window,
        }
    }

    fn start(mut self, mut event_loop: glutin::event_loop::EventLoop<()>) {
        event_loop.run_return(move |event, _, control_flow| {
            match event {
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    glutin::event::WindowEvent::CloseRequested => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                        return;
                    }
                    _ => return,
                },
                glutin::event::Event::NewEvents(cause) => match cause {
                    glutin::event::StartCause::ResumeTimeReached { .. } => (),
                    glutin::event::StartCause::Init => (),
                    _ => return,
                },
                _ => return,
            }

            if unsafe {
                SHOULD_EXIT.load(Ordering::Relaxed)
            } {
                *control_flow = glutin::event_loop::ControlFlow::Exit;
                return;
            }

            let next_frame_time =
                std::time::Instant::now() + std::time::Duration::from_secs_f32(1.0 / self.ups);
            *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

            let seconds_elapsed = self.time_elapsed.as_secs_f32();

            let alarm_factor = *ALARM_FACTOR.lock().unwrap();
            let current_frame_short = (*CURRENT_FRAME.lock().unwrap() as [f32;BARS as usize]).to_vec();
            // manual implementation of a gauss filter with sigma 1.5, kernel size 11 (4 sigma)
            // truncate values after 4 sigma -> 7 values of gaussian function (precalculated)
            // within 1% of scipy's version, good enough for us
            let gauss = [0.2659615202676218, 0.2129653370149015, 0.10934004978399577, 0.035993977675458706, 0.007597324015864964, 0.001028185997527405, 8.92201505099236e-05];
            let truncate = gauss.len() - 1;

            // creating a larger vector beforehand to get rid of clamping is not faster
            // using par_iter is ~5 times slower
            let current_frame_smooth: Vec<f32> = vec![0.0;BARS as usize].iter().enumerate().map(|(i,_)| {
                let mut sum: f32 = 0.0;
                for neighbor in -(truncate as i32)..=truncate as i32 {
                    let index = std::cmp::max(0, std::cmp::min(current_frame_short.len() - 1, (i as i32 + neighbor).abs() as usize));
                    sum += gauss[neighbor.abs() as usize] * current_frame_short[index];
                }
                sum
            }).collect();

            // quadruple the frame in size so it matches the rgba texture format
            let mut current_frame = Vec::with_capacity(BARS as usize * 4);
            for value in current_frame_smooth {
                current_frame.push(value);
                current_frame.push(0.0);
                current_frame.push(0.0);
                current_frame.push(0.0);
            }

            let mut target = self.display.draw();
            target.clear_all((0.0, 0.0, 0.0, 1.0), 0.0, 0);

            let mut current_intensity: f32 = current_frame.iter().sum::<f32>() / BARS as f32;
            if alarm_factor >= 0.0 {
                current_intensity = alarm_factor;
            }
            self.total_intensity += current_intensity;
            // the fraction of time the spectrum was intense
            let intensity_fraction = self.total_intensity / seconds_elapsed / self.ups;

            // This could easily be computed in the shader,
            // but due to performance issues on the Pi this was moved to the CPU
            let shake = ((seconds_elapsed * 9.0 + self.total_intensity * 0.3).cos() * 0.003, (seconds_elapsed * 5.0 + self.total_intensity * 0.3).cos() * 0.003);
            let saturation = 0.6;
            let value = 0.7;
            let start_hue = 0.0;
            let top_hue = ((seconds_elapsed * 0.15 - self.total_intensity * 0.05) * 0.1 + start_hue) * 360.0;
            let bot_hue = ((seconds_elapsed * 0.25 + self.total_intensity * 0.05) * 0.02 + start_hue) * 360.0;
            let mut top_color: Srgb = Hsv::new(top_hue, saturation, value).into_color();
            let mut bot_color: Srgb = Hsv::new(bot_hue, saturation, value).into_color();
            if alarm_factor >= 0.0 {
                top_color = Srgb::new(alarm_factor, 0.0, 0.0);
                bot_color = Srgb::new(alarm_factor, 0.0, 0.0);
            }
            let recent_color = top_color;
            let past_color: Srgb = Hsv::new(top_hue + 120.0, saturation, value).into_color();

            let image = glium::texture::RawImage2d::from_raw_rgba(current_frame, (BARS, 1));
            let rect = glium::Rect {
                left: 0,
                bottom: 0,
                width: BARS,
                height: 1,
            };
            self.spectrum_texture.write(rect, image);

            let uniforms = uniform! {
                RESOLUTION: self.resolution,
                top_color: (top_color.red, top_color.green, top_color.blue),
                bot_color: (bot_color.red, bot_color.green, bot_color.blue),
            };
            let draw_parameters = glium::DrawParameters {
                .. Default::default()
            };
            target
                .draw(
                    &self.quad_v,
                    &self.quad_i,
                    &self.background_program,
                    &uniforms,
                    &draw_parameters,
                )
                .unwrap();

            let uniforms = uniform! {
                RESOLUTION: self.resolution,
                PARTICLE_SPAWN_Z: PARTICLE_SPAWN_Z,
                time_elapsed: seconds_elapsed,
                intensity_fraction: intensity_fraction,
            };
            let draw_parameters = glium::DrawParameters {
                blend: glium::Blend {
                    color: glium::BlendingFunction::Addition {
                        source: glium::LinearBlendingFactor::One,
                        destination: glium::LinearBlendingFactor::One,
                    },
                    alpha: glium::BlendingFunction::Addition {
                        source: glium::LinearBlendingFactor::One,
                        destination: glium::LinearBlendingFactor::One,
                    },
                    constant_value: (0.0, 0.0, 0.0, 0.0)
                },
                point_size: Some(25.0),
                .. Default::default()
            };
            target.draw(
                (&self.particle_v, self.particle_buffer.per_instance().unwrap()),
                &self.particle_i,
                &self.particle_program,
                &uniforms,
                &draw_parameters,
            ).unwrap();

            let uniforms = uniform! {
                RESOLUTION: self.resolution,
                BARS: BARS as f32,
                time_elapsed: seconds_elapsed,
                current_intensity: current_intensity,
                shake: shake,
                recent_color: (recent_color.red, recent_color.green, recent_color.blue),
                past_color: (past_color.red, past_color.green, past_color.blue),
                logo: self.logo_texture.sampled().wrap_function(glium::uniforms::SamplerWrapFunction::BorderClamp),
                spectrum: self.spectrum_texture.sampled().minify_filter(glium::uniforms::MinifySamplerFilter::Linear),
            };
            let draw_parameters = glium::DrawParameters {
                blend: glium::Blend::alpha_blending(),
                .. Default::default()
            };
            target
                .draw(
                    &self.quad_v,
                    &self.quad_i,
                    &self.foreground_program,
                    &uniforms,
                    &draw_parameters,
                )
                .unwrap();
            target.finish().unwrap();

            let now = SystemTime::now();
            self.time_elapsed += now.duration_since(self.last_loop).unwrap();
            self.last_loop = now;

            self.fps_counter += 1;
            let since_last_fps_measure = now.duration_since(self.last_fps_calc).unwrap().as_secs_f32();
            if since_last_fps_measure >= self.fps_measure_window || self.fps_counter as f32 >= self.fps_measure_window * self.ups {
                let avg_fps = self.fps_counter as f32 / since_last_fps_measure;
                self.fps_counter = 0;
                self.last_fps_calc = now;
                *AVG_FPS.lock().unwrap() = avg_fps;
            }
        });
    }
}

#[pyfunction]
pub fn set_module_root(module_root: &str) {
    // The image and shaders need to be identified by path
    // In order to find them with a relative path, the root of the module needs to be known
    // Since this is far easier in python, it is passed once during initialization
    *MODULE_ROOT.lock().unwrap() = module_root.to_string();
}

#[pymodule]
fn raveberry_visualization(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Controller>()?;
    m.add_function(wrap_pyfunction!(set_module_root, m)?)?;
    Ok(())
}
