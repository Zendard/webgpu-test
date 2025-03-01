use cgmath::{num_traits::Signed, InnerSpace, MetricSpace, Zero};

use crate::rendering::{Instance, INSTANCE_HEIGTH, INSTANCE_WIDTH};
use std::time::Duration;

const GRAVITATIONAL_CONSTANT: f32 = 4.0;

pub fn update_instances(instances: &mut Vec<Instance>, delta_time: Duration) {
    let mut new_instances = instances.clone();
    new_instances.iter_mut().for_each(|instance| {
        let positions: Vec<_> = instances
            .iter()
            .map(|instance| &instance.position)
            .collect();

        // Increase position with speed
        instance.position.x += instance.speed.x * delta_time.as_secs_f32();
        instance.position.y += instance.speed.y * delta_time.as_secs_f32();

        // Increase speed with gravitational constant
        instance.speed.y += -GRAVITATIONAL_CONSTANT * delta_time.as_secs_f32();

        // Set vertical speed to 0 and y to -1 if touching the ground
        if instance.position.y <= -1.0 {
            instance.speed.y = 0.0;
            instance.position.y = -0.95;
        }

        // Set horizontal speed to zero and x to -1 if touching left edge
        if instance.position.x <= -1.0 {
            instance.speed.x = 0.0;
            instance.position.x = -1.0;
        }
        // Set horizontal speed to zero and x to 1 if touching right edge
        if instance.position.x >= 1.0 {
            instance.speed.x = 0.0;
            instance.position.x = 0.95;
        }

        let collision = check_collisions(&instance.position, &positions);

        if let None = collision {
            return;
        }
        let vector_between = collision.unwrap();

        dbg!(vector_between);

        let angle = vector_between.angle(cgmath::Vector3::unit_x());
        instance.position -= (vector_between
            + cgmath::Vector3 {
                x: INSTANCE_WIDTH * cgmath::Angle::cos(angle),
                y: INSTANCE_WIDTH * cgmath::Angle::sin(angle),
                z: 0.0,
            })
            * 0.5;

        instance.speed = cgmath::Vector2::zero();
    });
    *instances = new_instances;
}

fn check_collisions(
    position: &cgmath::Vector3<f32>,
    other_positions: &Vec<&cgmath::Vector3<f32>>,
) -> Option<cgmath::Vector3<f32>> {
    other_positions.iter().find_map(|other_position| {
        let center = position
            + cgmath::Vector3 {
                x: INSTANCE_WIDTH * 0.5,
                y: INSTANCE_HEIGTH * 0.5,
                z: 0.0,
            };
        let other_center = *other_position
            + cgmath::Vector3 {
                x: INSTANCE_WIDTH * 0.5,
                y: INSTANCE_HEIGTH * 0.5,
                z: 0.0,
            };
        let distance = other_center.distance(center);
        let vector_between = other_center - center;

        // We don't want to record collisions with ourselves
        if distance <= 0.01 {
            return None;
        }

        if distance <= crate::rendering::INSTANCE_WIDTH * 0.5
            && distance >= crate::rendering::INSTANCE_WIDTH * -0.5
        {
            Some(vector_between)
        } else {
            None
        }
    })
}
