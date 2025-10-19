use crate::{
    entity::MAX_ENTITIES,
    map::{MAP, MAP_HEIGHT, MAP_WIDTH},
    spawner,
    tile::{clear_animation_counter, clear_resource, has_resource, set_resource},
    tileset::state,
};

pub const MAX_SPAWNERS: usize = 64;
pub const SPAW_TIME_MASK: usize = 0b11111;
pub const SPAW_TIME: u8 = 23;

pub const LEFT_SPAWNER: u8 = 33;
pub const TOP_SPAWNER: u8 = 34;
pub const RIGHT_SPAWNER: u8 = 35;
pub const DOWN_SPAWNER: u8 = 36;

pub struct Spawners {
    pub count: u8,
    pub spawn_x: [i8; MAX_SPAWNERS],
    pub spawn_y: [i8; MAX_SPAWNERS],
    pub spawn_i: [usize; MAX_SPAWNERS],
    pub spawn_time: [u8; MAX_SPAWNERS],
}

#[inline(never)]
pub unsafe fn update_spawners() {
    let state = &mut *state();

    for i in 0..(state.spawners.count as usize) {
        let current_time = state.spawners.spawn_time[i];

        if current_time == 0 {
            if state.entities.count < MAX_ENTITIES as u8 {
                let spawner_i = state.spawners.spawn_i[i];

                if !has_resource(MAP[spawner_i]) {
                    state.entities.x[state.entities.count as usize] = state.spawners.spawn_x[i];
                    state.entities.y[state.entities.count as usize] = state.spawners.spawn_y[i];

                    state.entities.i[state.entities.count as usize] = spawner_i;
                    MAP[spawner_i] = set_resource(MAP[spawner_i]);

                    state.entities.count += 1;
                }
            }

            state.spawners.spawn_time[i] = SPAW_TIME;
        } else {
            state.spawners.spawn_time[i] -= 1;
        }
    }
}

#[inline(never)]
pub unsafe fn find_initial() {
    let spawners = &mut (&mut *state()).spawners;

    let mut i = 0;

    for x in 0..(MAP_WIDTH as i8) {
        for y in 0..(MAP_HEIGHT as i8) {
            let tile = clear_animation_counter(MAP[i]);

            if tile == LEFT_SPAWNER {
                let index = spawners.count as usize;
                spawners.spawn_x[index] = x - 1;
                spawners.spawn_y[index] = y;
                spawners.spawn_i[index] = i - 1;
                spawners.spawn_time[index] = (i & SPAW_TIME_MASK) as u8;

                spawners.count += 1;
            } else if tile == TOP_SPAWNER {
                let index = spawners.count as usize;
                spawners.spawn_x[index] = x;
                spawners.spawn_y[index] = y - MAP_WIDTH as i8;
                spawners.spawn_i[index] = i - MAP_WIDTH as usize;
                spawners.spawn_time[index] = (i & SPAW_TIME_MASK) as u8;

                spawners.count += 1;
            } else if tile == RIGHT_SPAWNER {
                let index = spawners.count as usize;
                spawners.spawn_x[index] = x + 1;
                spawners.spawn_y[index] = y;
                spawners.spawn_i[index] = i + 1;
                spawners.spawn_time[index] = (i & SPAW_TIME_MASK) as u8;

                spawners.count += 1;
            } else if tile == DOWN_SPAWNER {
                let index = spawners.count as usize;
                spawners.spawn_x[index] = x;
                spawners.spawn_y[index] = y + MAP_WIDTH as i8;
                spawners.spawn_i[index] = i + MAP_WIDTH as usize;
                spawners.spawn_time[index] = (i & SPAW_TIME_MASK) as u8;

                spawners.count += 1;
            }

            i += 1;
        }
    }
}
