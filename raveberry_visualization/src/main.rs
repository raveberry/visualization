use std::thread;
use std::time::{Duration, SystemTime};

fn main() {
    let variant = "Circle";
    let module_root = "./raveberry_visualization";
    const UPS: f32 = 30.0;
    const NUM_PARTICLES: u32 = 400;
    const FPS_MEASURE_WINDOW: f32 = 5.0;
    raveberry_visualization::set_module_root(module_root);
    let controller = raveberry_visualization::Controller {};
    controller.start(variant, UPS, NUM_PARTICLES, FPS_MEASURE_WINDOW);
    let mut time_elapsed = Duration::new(0, 0);
    let mut last_loop = SystemTime::now();
    loop {
        if !controller.is_active() {
            break;
        }
        let seconds_elapsed = time_elapsed.as_secs_f32();
        let mut current_frame = [0.0; raveberry_visualization::BARS as usize];
        for (i, val) in current_frame.iter_mut().enumerate() {
            *val = 0.8
                * 0.5
                * (1.0 + (4.0 * seconds_elapsed).sin())
                * 0.5
                * (1.0 + (-5.0 * seconds_elapsed + 2.0 * i as f32).sin())
        }
        let now = SystemTime::now();
        time_elapsed += now.duration_since(last_loop).unwrap();
        last_loop = now;
        controller.set_parameters(-1.0, current_frame);
        thread::sleep(Duration::from_secs_f32(1.0 / UPS as f32));
    }
}
