use bytemuck::{Zeroable, Pod};
use ultraviolet::{projection, Mat4, Vec3};

pub struct Camera {
    pub pos: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub aspect: f32,
    pub fov: f32,
    pub z_near: f32,
    pub z_far: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
pub struct CameraMatrices {
    pub view: [f32; 16],
    pub projection: [f32; 16],
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            pos: Vec3::new(0.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            aspect,
            fov: 90.0 / 100.0,
            z_near: 0.1,
            z_far: 1000.0,
        }
    }

    pub fn direction(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * (1.0 - self.pitch.sin().abs()),
            self.pitch.sin(),
            self.yaw.sin() * (1.0 - self.pitch.sin().abs()),
        )
    }

    pub fn matrices(&self) -> CameraMatrices {
        CameraMatrices {
            view: Mat4::look_at(self.pos, self.pos + self.direction(), Vec3::unit_y()).as_array().to_owned(),
            projection: projection::rh_yup::perspective_gl(self.fov, self.aspect, self.z_near, self.z_far).as_array().to_owned(),
        }
    }
}
