use crate::model::CameraKeyframe;

pub fn get_camera_at_frame(keyframes: &[CameraKeyframe], frame: u32) -> CameraKeyframe {
    if keyframes.is_empty() {
        return CameraKeyframe { frame, x: 0.0, y: 0.0, scale: 1.0, rotation: 0.0, width: 1920.0, height: 1080.0 };
    }
    if keyframes.len() == 1 || frame <= keyframes[0].frame {
        return keyframes[0].clone();
    }
    let last = keyframes.last().unwrap();
    if frame >= last.frame {
        return last.clone();
    }
    let mut before = &keyframes[0];
    let mut after = last;
    for w in keyframes.windows(2) {
        if w[0].frame <= frame && w[1].frame >= frame {
            before = &w[0];
            after = &w[1];
            break;
        }
    }
    let t = (frame - before.frame) as f64 / (after.frame - before.frame) as f64;
    CameraKeyframe {
        frame,
        x: lerp(before.x, after.x, t),
        y: lerp(before.y, after.y, t),
        scale: lerp(before.scale, after.scale, t),
        rotation: lerp(before.rotation, after.rotation, t),
        width: lerp(before.width, after.width, t),
        height: lerp(before.height, after.height, t),
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 { a + (b - a) * t }
