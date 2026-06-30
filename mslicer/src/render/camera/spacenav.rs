use std::io::Read;
#[cfg(unix)]
use std::os::unix::net::UnixStream;

use common::serde::{Deserializer, SliceDeserializer};
use nalgebra::Vector3;

use crate::app_ref_type;

const BASE_GAIN: f32 = 0.0005;

pub struct SpaceNav {
    #[cfg(unix)]
    stream: Option<UnixStream>,
    #[cfg(not(unix))]
    stream: Option<()>,
}

app_ref_type!(SpaceNav, spacenav);

#[derive(Debug)]
pub enum Event {
    Button {
        id: i32,
        press: bool,
    },
    Motion {
        translation: Vector3<i32>,
        rotation: Vector3<i32>,
    },
}

impl SpaceNav {
    pub fn unconnected() -> Self {
        Self { stream: None }
    }

    pub fn try_connect(&mut self) {
        #[cfg(unix)]
        {
            self.stream = UnixStream::connect("/run/spnav.sock")
                .and_then(|s| {
                    s.set_nonblocking(true)?;
                    Ok(s)
                })
                .ok();
        }
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    #[cfg(unix)]
    pub fn poll(&mut self) -> Option<Event> {
        let Some(stream) = &mut self.stream else {
            return None;
        };

        let mut buffer = [0; 8 * 4];
        match stream.read_exact(&mut buffer) {
            Ok(()) => {}
            Err(_) => return None,
        }

        let mut des = SliceDeserializer::new(&buffer);
        let event = des.read_i32_le();
        match event {
            // motion
            0 => {
                let translation =
                    Vector3::new(des.read_i32_le(), des.read_i32_le(), des.read_i32_le());
                let rotation =
                    Vector3::new(des.read_i32_le(), des.read_i32_le(), des.read_i32_le());
                Some(Event::Motion {
                    translation,
                    rotation,
                })
            }
            // button press
            1 => Some(Event::Button {
                id: des.read_i32_le(),
                press: true,
            }),
            // button release
            2 => Some(Event::Button {
                id: des.read_i32_le(),
                press: false,
            }),
            _ => None,
        }
    }

    #[cfg(not(unix))]
    fn poll(&mut self) -> Option<Event> {
        None
    }
}

impl SpaceNavRef<'_> {
    pub fn handle_movement(&mut self, focused: bool) -> bool {
        let mut is_moving = false;

        while let Some(event) = self.poll() {
            if !focused {
                continue;
            }

            match event {
                Event::Button { id: 0, press: true } => self.app.camera = Default::default(),
                Event::Button { id: 1, press: true } => self.app.slice(),
                Event::Motion {
                    translation,
                    rotation,
                } => {
                    let config = &self.app.config.spacenav;
                    let p_gain = BASE_GAIN * 10.0 * config.gain * config.position_gain;
                    let r_gain = BASE_GAIN * 0.5 * config.gain * config.rotation_gain;

                    let camera = &mut self.app.camera;
                    camera.angle.x -= rotation.y as f32 * r_gain;
                    camera.angle.y -= rotation.x as f32 * r_gain;
                    camera.distance += translation.y as f32 * p_gain;

                    let camera_pos = camera.position(camera.distance) + camera.target;
                    let forward = (camera.target - camera_pos).normalize();
                    let right = forward.cross(&camera.up()).normalize();

                    camera.target += right * translation.x as f32 * p_gain;
                    camera.target += forward * translation.z as f32 * p_gain;

                    is_moving |= translation != Vector3::zeros() || rotation != Vector3::zeros();
                }
                _ => {}
            };
        }

        is_moving
    }
}
