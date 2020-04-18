use std::collections::HashSet;

use rand::prelude::*;

use ggez::conf;
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::nalgebra as na;
use ggez::{graphics, Context, ContextBuilder, GameResult};

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

const SCREEN_SIZE_X: f32 = 1000.0;
const SCREEN_SIZE_Y: f32 = 1000.0;

fn main() {
    // Make a Context.
    let (mut ctx, mut event_loop) = ContextBuilder::new("save_the_pink_skins", "gajop")
        .window_mode(conf::WindowMode::default().dimensions(SCREEN_SIZE_X, SCREEN_SIZE_Y))
        .build()
        .expect("Failed to create create ggez context. Please report this error");

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let mut my_game = SaveThePinkSkin::new(&mut ctx);
    add_spaceship(&mut my_game);
    add_earth(&mut my_game);

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}

struct GameObject {
    pos_x: f32,
    pos_y: f32,
    radius: f32,
    vel_x: f32,
    vel_y: f32,
    acc_x: f32,
    acc_y: f32,
    color: graphics::Color,
}

#[derive(Default)]
struct Controls {
    left_right: Option<Direction>,
    up_down: Option<Direction>,
}
const MAX_ACC_X: f32 = 0.00005;
const MAX_ACC_Y: f32 = 0.00005;
const ACC_STEP_X: f32 = 0.00001;
const ACC_STEP_Y: f32 = 0.00001;

struct SaveThePinkSkin {
    // spaceship: GameObject,
    // earth: GameObject,
    objects: Vec<GameObject>,
    spaceship_id: Option<usize>,
    earth_id: Option<usize>,
    controls: Controls,
    rng: ThreadRng,
    next_meteor_spawn: Option<f32>,
}

fn add_spaceship(game: &mut SaveThePinkSkin) {
    game.objects.push(GameObject {
        pos_x: 0.1,
        pos_y: 0.3,
        radius: 0.01,
        color: graphics::Color::new(0.5, 0.5, 0.7, 1.0),
        vel_x: 0.0,
        vel_y: 0.0,
        acc_x: 0.0,
        acc_y: 0.0,
    });
    game.spaceship_id = Some(game.objects.len() - 1);
}

fn add_earth(game: &mut SaveThePinkSkin) {
    game.objects.push(GameObject {
        pos_x: 0.5,
        pos_y: 0.5,
        radius: 0.1,
        color: graphics::Color::new(0.3, 0.7, 0.3, 1.0),
        vel_x: 0.0,
        vel_y: 0.0,
        acc_x: 0.0,
        acc_y: 0.0,
    });
    game.earth_id = Some(game.objects.len() - 1);
}

fn dist(first: &GameObject, second: &GameObject) -> f32 {
    let dx = first.pos_x - second.pos_x;
    let dy = first.pos_y - second.pos_y;
    return (dx * dx + dy * dy).sqrt() - first.radius - second.radius;
}

fn add_meteor(game: &mut SaveThePinkSkin) {
    const MAX_SIZE: f32 = 0.02;
    const MIN_SIZE: f32 = 0.007;
    const MAX_VELOCITY: f32 = 0.001;
    const MIN_VELOCITY: f32 = 0.0003;

    let mut meteor = GameObject {
        pos_x: 0.0,
        pos_y: 0.0,
        radius: 0.0,
        color: graphics::Color::new(0.878, 0.603, 0.282, 1.0),
        vel_x: 0.0,
        vel_y: 0.0,
        acc_x: 0.0,
        acc_y: 0.0,
    };

    meteor.radius = game.rng.gen_range(MIN_SIZE, MAX_SIZE);
    meteor.vel_x = game.rng.gen_range(MIN_VELOCITY, MAX_VELOCITY);
    meteor.vel_y = game.rng.gen_range(MIN_VELOCITY, MAX_VELOCITY);

    let dir: Direction = rand::random();
    let pos: f32 = game.rng.gen();

    match dir {
        Direction::Up => {
            meteor.pos_x = pos;
            meteor.pos_y = 0.0;
            if game.rng.gen::<f32>() > 0.5 {
                meteor.vel_x *= -1.0;
            }
        }
        Direction::Down => {
            meteor.pos_x = pos;
            meteor.pos_y = 1.0;
            meteor.vel_y *= -1.0;
            if game.rng.gen::<f32>() > 0.5 {
                meteor.vel_x *= -1.0;
            }
        }
        Direction::Left => {
            meteor.pos_x = 0.0;
            meteor.pos_y = pos;
            if game.rng.gen::<f32>() > 0.5 {
                meteor.vel_y *= -1.0;
            }
        }
        Direction::Right => {
            meteor.pos_x = 1.0;
            meteor.pos_y = pos;
            meteor.vel_x *= -1.0;
            if game.rng.gen::<f32>() > 0.5 {
                meteor.vel_y *= -1.0;
            }
        }
    };
    game.objects.push(meteor);
}

impl SaveThePinkSkin {
    pub fn new(_ctx: &mut Context) -> SaveThePinkSkin {
        // Load/create resources such as images here.
        SaveThePinkSkin {
            objects: vec![],
            controls: Default::default(),
            spaceship_id: None,
            earth_id: None,
            rng: rand::thread_rng(),
            next_meteor_spawn: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0, 4) {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            _ => Direction::Right,
        }
    }
}

fn from_keycode(key: KeyCode) -> Option<Direction> {
    match key {
        KeyCode::Up | KeyCode::W => Some(Direction::Up),
        KeyCode::Down | KeyCode::S => Some(Direction::Down),
        KeyCode::Left | KeyCode::A => Some(Direction::Left),
        KeyCode::Right | KeyCode::D => Some(Direction::Right),
        _ => None,
    }
}

struct Collision {
    first: usize,
    second: usize,
}

fn find_collisions(game: &SaveThePinkSkin) -> Vec<Collision> {
    let mut collisions = Vec::<Collision>::new();
    for i in 0..game.objects.len() - 1 {
        for j in (i + 1)..game.objects.len() {
            let object1 = &game.objects[i];
            let object2 = &game.objects[j];
            if dist(object1, object2) <= 0.0 {
                collisions.push(Collision {
                    first: i,
                    second: j,
                });
                // println!("earth colided with meteor: {}", i);
            }
        }
    }

    collisions
}

struct CollisionResults {
    created: Vec<GameObject>,
    destroyed_ids: Vec<usize>,
}

fn process_collisions(game: &mut SaveThePinkSkin, collisions: &Vec<Collision>) -> CollisionResults {
    let mut results = CollisionResults {
        created: Vec::new(),
        destroyed_ids: Vec::new(),
    };
    let mut destroyed_unique = HashSet::<usize>::new();

    for collision in collisions {
        destroyed_unique.insert(collision.first);
        destroyed_unique.insert(collision.second);
        if let Some(earth_id) = game.earth_id {
            if earth_id == collision.first || earth_id == collision.second {
                // println!("collided with earth");
            }
        }
        if let Some(spaceship_id) = game.spaceship_id {
            if spaceship_id == collision.first || spaceship_id == collision.second {
                // println!("collided with spaceship");
            }
        }
    }
    for destroyed in destroyed_unique {
        if destroyed >= 2 {
            results.destroyed_ids.push(destroyed);
        }
    }

    results
}

fn cleanup_destroyed(game: &mut SaveThePinkSkin, destroyed_ids: &Vec<usize>) {
    if game.objects.is_empty() {
        return;
    }
    let mut index: usize = 0;
    game.objects.retain(|_| {
        let retain = !destroyed_ids.contains(&index);
        index += 1;
        retain
    })
}

fn add_new(game: &mut SaveThePinkSkin, created: Vec<GameObject>) {
    for object in created {
        game.objects.push(object);
    }
}

impl EventHandler for SaveThePinkSkin {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const TARGET_FPS: u32 = 60;
        const SPAWN_INTERVAL: f32 = 1.0;

        let time: f32 = ggez::timer::time_since_start(&ctx).as_secs() as f32;

        if let Some(next_meteor_spawn) = self.next_meteor_spawn {
            if time > next_meteor_spawn {
                add_meteor(self);
                self.next_meteor_spawn = Some(time + SPAWN_INTERVAL);
            }
        } else {
            self.next_meteor_spawn = Some(time + SPAWN_INTERVAL);
        }

        while ggez::timer::check_update_time(ctx, TARGET_FPS) {
            if let Some(spaceship_id) = self.spaceship_id {
                let spaceship = &mut self.objects[spaceship_id];
                match self.controls.left_right {
                    Some(Direction::Left) => spaceship.acc_x -= ACC_STEP_X,
                    Some(Direction::Right) => spaceship.acc_x += ACC_STEP_X,
                    _ => {
                        spaceship.acc_x =
                            spaceship.acc_x.signum() * (spaceship.acc_x.abs() - ACC_STEP_X)
                    }
                };

                match self.controls.up_down {
                    Some(Direction::Up) => spaceship.acc_y -= ACC_STEP_Y,
                    Some(Direction::Down) => spaceship.acc_y += ACC_STEP_Y,
                    _ => {
                        spaceship.acc_y =
                            spaceship.acc_y.signum() * (spaceship.acc_y.abs() - ACC_STEP_Y)
                    }
                };

                spaceship.acc_x = na::clamp(spaceship.acc_x, -MAX_ACC_X, MAX_ACC_X);
                spaceship.acc_y = na::clamp(spaceship.acc_y, -MAX_ACC_Y, MAX_ACC_Y);
            }

            for mut object in &mut self.objects {
                object.vel_x += object.acc_x;
                object.vel_y += object.acc_y;

                object.pos_x += object.vel_x;
                object.pos_y += object.vel_y;

                if object.pos_x - object.radius > 1.0 {
                    object.pos_x = 0.0;
                } else if object.pos_x + object.radius < 0.0 {
                    object.pos_x = 1.0;
                }
                if object.pos_y - object.radius > 1.0 {
                    object.pos_y = 0.0;
                } else if object.pos_y + object.radius < 0.0 {
                    object.pos_y = 1.0;
                }
            }

            let collisions = find_collisions(self);
            let results = process_collisions(self, &collisions);
            cleanup_destroyed(self, &results.destroyed_ids);
            add_new(self, results.created);

            for object in &mut self.objects {
                object.vel_x += object.acc_x;
                object.vel_y += object.acc_y;

                object.pos_x += object.vel_x;
                object.pos_y += object.vel_y;

                if object.pos_x - object.radius > 1.0 {
                    object.pos_x = 0.0;
                } else if object.pos_x + object.radius < 0.0 {
                    object.pos_x = 1.0;
                }
                if object.pos_y - object.radius > 1.0 {
                    object.pos_y = 0.0;
                } else if object.pos_y + object.radius < 0.0 {
                    object.pos_y = 1.0;
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        // Draw code here...

        for object in &self.objects {
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                na::Point2::new(0.0, 0.0),
                object.radius * SCREEN_SIZE_X,
                0.1,
                object.color,
            )?;
            graphics::draw(
                ctx,
                &circle,
                (na::Point2::new(
                    object.pos_x * SCREEN_SIZE_X,
                    object.pos_y * SCREEN_SIZE_Y,
                ),),
            )?;
        }

        graphics::present(ctx)
    }

    /// key_down_event gets fired when a key gets pressed.
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        if let Some(dir) = from_keycode(keycode) {
            match dir {
                Direction::Up | Direction::Down => self.controls.up_down = Some(dir),
                Direction::Left | Direction::Right => self.controls.left_right = Some(dir),
            }
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        if let Some(dir) = from_keycode(keycode) {
            match dir {
                Direction::Up | Direction::Down => {
                    if self.controls.up_down == Some(dir) {
                        self.controls.up_down = None;
                    }
                }
                Direction::Left | Direction::Right => {
                    if self.controls.left_right == Some(dir) {
                        self.controls.left_right = None
                    }
                }
            }
        }
    }
}
