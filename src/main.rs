use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::path;

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

fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    // Make a Context.
    let (mut ctx, mut event_loop) = ContextBuilder::new("save_the_pink_skins", "gajop")
        .window_mode(conf::WindowMode::default().dimensions(SCREEN_SIZE_X, SCREEN_SIZE_Y))
        .add_resource_path(resource_dir)
        .build()
        .expect("Failed to create create ggez context. Please report this error");

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let mut my_game = SaveThePinkSkin::new(&mut ctx)?;
    my_game.add_spaceship();
    my_game.add_earth();
    my_game.add_text_population();

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct GameObject {
    id: usize,
    transform: Transform,
    collidable: bool,
}

#[derive(Clone, Debug)]
struct Transform {
    pos_x: f32,
    pos_y: f32,
    vel_x: f32,
    vel_y: f32,
    acc_x: f32,
    acc_y: f32,
}

#[derive(Clone, Debug)]
struct CircleObject {
    id: usize,
    radius: f32,
    color: graphics::Color,
}

#[derive(Clone, Debug)]
struct TextObject {
    id: usize,
    text: graphics::Text,
    expiration_time: Option<i32>,
}

#[derive(Default, Clone, Debug)]
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
    id_generator: usize,
    objects: HashMap<usize, GameObject>,
    circles: Vec<CircleObject>,
    texts: Vec<TextObject>,
    spaceship_id: Option<usize>,
    earth_id: Option<usize>,
    controls: Controls,
    rng: ThreadRng,
    next_meteor_spawn: Option<f32>,
    font: graphics::Font,
    text_population_id: Option<usize>,
    population_million: f32,
}

impl SaveThePinkSkin {
    pub fn new(ctx: &mut Context) -> GameResult<SaveThePinkSkin> {
        // Load/create resources such as images here.
        let font = graphics::Font::new(ctx, "/PixelEmulator-xq08.ttf")?;

        Ok(SaveThePinkSkin {
            id_generator: 0,
            objects: HashMap::new(),
            circles: vec![],
            texts: vec![],
            controls: Default::default(),
            spaceship_id: None,
            earth_id: None,
            rng: rand::thread_rng(),
            next_meteor_spawn: None,
            font: font,
            text_population_id: None,
            population_million: 5000.0,
        })
    }

    fn make_object(&mut self, transform: Transform) -> usize {
        self.id_generator += 1;
        let id = self.id_generator;
        self.objects.insert(
            id,
            GameObject {
                id,
                transform,
                collidable: true,
            },
        );
        id
    }

    fn add_circle(&mut self, circle: CircleObject) {
        self.circles.push(circle);
    }

    fn add_spaceship(&mut self) {
        let id = self.make_object(Transform {
            pos_x: 0.1,
            pos_y: 0.3,
            vel_x: 0.0,
            vel_y: 0.0,
            acc_x: 0.0,
            acc_y: 0.0,
        });
        self.circles.push(CircleObject {
            id: id,
            radius: 0.01,
            color: graphics::Color::new(0.5, 0.5, 0.7, 1.0),
        });
        self.spaceship_id = Some(id);
    }

    fn add_earth(&mut self) {
        let id = self.make_object(Transform {
            pos_x: 0.5,
            pos_y: 0.5,
            vel_x: 0.0,
            vel_y: 0.0,
            acc_x: 0.0,
            acc_y: 0.0,
        });
        self.circles.push(CircleObject {
            id: id,
            radius: 0.1,
            color: graphics::Color::new(0.3, 0.7, 0.3, 1.0),
        });
        self.earth_id = Some(id);
    }

    fn add_text_population(&mut self) {
        let id = self.make_object(Transform {
            pos_x: 0.3,
            pos_y: 0.0,
            vel_x: 0.0,
            vel_y: 0.0,
            acc_x: 0.0,
            acc_y: 0.0,
        });
        self.objects.get_mut(&id).unwrap().collidable = false;
        self.texts.push(TextObject {
            id: id,
            text: graphics::Text::default(),
            expiration_time: None,
        });
        self.text_population_id = Some(self.texts.len() - 1);
    }

    fn add_meteor(&mut self) {
        const MAX_SIZE: f32 = 0.02;
        const MIN_SIZE: f32 = 0.007;
        const MAX_VELOCITY: f32 = 0.001;
        const MIN_VELOCITY: f32 = 0.0003;

        let mut meteor = Transform {
            pos_x: 0.0,
            pos_y: 0.0,
            vel_x: self.rng.gen_range(MIN_VELOCITY, MAX_VELOCITY),
            vel_y: self.rng.gen_range(MIN_VELOCITY, MAX_VELOCITY),
            acc_x: 0.0,
            acc_y: 0.0,
        };

        let dir: Direction = rand::random();
        let pos: f32 = self.rng.gen();

        match dir {
            Direction::Up => {
                meteor.pos_x = pos;
                meteor.pos_y = 0.0;
                if self.rng.gen::<f32>() > 0.5 {
                    meteor.vel_x *= -1.0;
                }
            }
            Direction::Down => {
                meteor.pos_x = pos;
                meteor.pos_y = 1.0;
                meteor.vel_y *= -1.0;
                if self.rng.gen::<f32>() > 0.5 {
                    meteor.vel_x *= -1.0;
                }
            }
            Direction::Left => {
                meteor.pos_x = 0.0;
                meteor.pos_y = pos;
                if self.rng.gen::<f32>() > 0.5 {
                    meteor.vel_y *= -1.0;
                }
            }
            Direction::Right => {
                meteor.pos_x = 1.0;
                meteor.pos_y = pos;
                meteor.vel_x *= -1.0;
                if self.rng.gen::<f32>() > 0.5 {
                    meteor.vel_y *= -1.0;
                }
            }
        };
        let id = self.make_object(meteor);
        let radius = self.rng.gen_range(MIN_SIZE, MAX_SIZE);
        self.add_circle(CircleObject {
            id: id,
            radius: radius,
            color: graphics::Color::new(0.878, 0.603, 0.282, 1.0),
        });
    }

    fn dist(&self, first: &CircleObject, second: &CircleObject) -> f32 {
        let t1 = &self.objects.get(&first.id).unwrap().transform;
        let t2 = &self.objects.get(&second.id).unwrap().transform;
        dist(t1, t2) - first.radius - second.radius
    }

    fn remove_object(&mut self, id: usize) {
        println!("Removed: {}", id);
        self.circles.retain(|c| c.id != id);
        self.texts.retain(|c| c.id != id);
        self.objects.remove(&id);
        if self.spaceship_id == Some(id) {
            self.spaceship_id = None;
        }
        if self.earth_id == Some(id) {
            self.earth_id = None;
        }
    }
}

fn dist(first: &Transform, second: &Transform) -> f32 {
    let dx = first.pos_x - second.pos_x;
    let dy = first.pos_y - second.pos_y;
    (dx * dx + dy * dy).sqrt()
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
    for i in 0..game.circles.len() {
        for j in (i + 1)..game.circles.len() {
            let circle1 = &game.circles[i];
            let circle2 = &game.circles[j];
            if game.dist(&circle1, &circle2) <= 0.0 {
                let object1 = game.objects.get(&circle1.id).unwrap();
                let object2 = game.objects.get(&circle2.id).unwrap();
                if object1.collidable && object2.collidable {
                    collisions.push(Collision {
                        first: object1.id,
                        second: object2.id,
                    });
                }
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
        if Some(destroyed) != game.earth_id && Some(destroyed) != game.spaceship_id {
            results.destroyed_ids.push(destroyed);
        }
    }

    results
}

fn cleanup_destroyed(game: &mut SaveThePinkSkin, destroyed_ids: &Vec<usize>) {
    if game.objects.is_empty() {
        return;
    }
    for id in destroyed_ids {
        game.remove_object(*id);
    }
}

fn add_new(game: &mut SaveThePinkSkin, created: Vec<GameObject>) {
    for object in created {
        game.objects.insert(object.id, object);
    }
}

impl EventHandler for SaveThePinkSkin {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const TARGET_FPS: u32 = 60;
        const SPAWN_INTERVAL: f32 = 0.10;

        let time: f32 = ggez::timer::time_since_start(&ctx).as_secs() as f32;

        if let Some(next_meteor_spawn) = self.next_meteor_spawn {
            if time > next_meteor_spawn {
                self.add_meteor();
                self.next_meteor_spawn = Some(time + SPAWN_INTERVAL);
            }
        } else {
            self.next_meteor_spawn = Some(time + SPAWN_INTERVAL);
        }

        while ggez::timer::check_update_time(ctx, TARGET_FPS) {
            if let Some(spaceship_id) = self.spaceship_id {
                let controls = self.controls.clone();
                if let Some(spaceship) = self.objects.get_mut(&spaceship_id) {
                    let spaceship_tr = &mut spaceship.transform;
                    match controls.left_right {
                        Some(Direction::Left) => spaceship_tr.acc_x -= ACC_STEP_X,
                        Some(Direction::Right) => spaceship_tr.acc_x += ACC_STEP_X,
                        _ => {
                            spaceship_tr.acc_x = spaceship_tr.acc_x.signum()
                                * (spaceship_tr.acc_x.abs() - ACC_STEP_X)
                        }
                    };

                    match controls.up_down {
                        Some(Direction::Up) => spaceship_tr.acc_y -= ACC_STEP_Y,
                        Some(Direction::Down) => spaceship_tr.acc_y += ACC_STEP_Y,
                        _ => {
                            spaceship_tr.acc_y = spaceship_tr.acc_y.signum()
                                * (spaceship_tr.acc_y.abs() - ACC_STEP_Y)
                        }
                    };

                    spaceship_tr.acc_x = na::clamp(spaceship_tr.acc_x, -MAX_ACC_X, MAX_ACC_X);
                    spaceship_tr.acc_y = na::clamp(spaceship_tr.acc_y, -MAX_ACC_Y, MAX_ACC_Y);
                }
            }

            for object in &mut self.objects.values_mut() {
                let transform = &mut object.transform;
                transform.vel_x += transform.acc_x;
                transform.vel_y += transform.acc_y;

                transform.pos_x += transform.vel_x;
                transform.pos_y += transform.vel_y;

                if transform.pos_x > 1.0 {
                    transform.pos_x = 0.0;
                } else if transform.pos_x < 0.0 {
                    transform.pos_x = 1.0;
                }
                if transform.pos_y > 1.0 {
                    transform.pos_y = 0.0;
                } else if transform.pos_y < 0.0 {
                    transform.pos_y = 1.0;
                }
            }

            let collisions = find_collisions(self);
            let results = process_collisions(self, &collisions);
            cleanup_destroyed(self, &results.destroyed_ids);
            add_new(self, results.created);

            if let Some(text_population_id) = self.text_population_id {
                let text_population = &mut self.texts[text_population_id];
                let population_str = format!("Population: {}M", self.population_million);
                text_population.text = graphics::Text::new((population_str, self.font, 48.0));
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        for circle in &self.circles {
            if let Some(object) = self.objects.get(&circle.id) {
                let circle = graphics::Mesh::new_circle(
                    ctx,
                    graphics::DrawMode::fill(),
                    na::Point2::new(0.0, 0.0),
                    circle.radius * SCREEN_SIZE_X,
                    0.1,
                    circle.color,
                )?;
                graphics::draw(
                    ctx,
                    &circle,
                    (na::Point2::new(
                        object.transform.pos_x * SCREEN_SIZE_X,
                        object.transform.pos_y * SCREEN_SIZE_Y,
                    ),),
                )?;
            }
        }

        for text in &self.texts {
            if let Some(object) = self.objects.get(&text.id) {
                graphics::draw(
                    ctx,
                    &text.text,
                    (na::Point2::new(
                        object.transform.pos_x * SCREEN_SIZE_X,
                        object.transform.pos_y * SCREEN_SIZE_Y,
                    ),),
                )?;
                graphics::present(ctx)?;
            }
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
