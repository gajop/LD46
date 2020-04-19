use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::path;

use rand::prelude::*;

use ggez::audio;
use ggez::audio::SoundSource;
use ggez::conf;
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::nalgebra as na;
use ggez::{graphics, Context, ContextBuilder, GameResult};

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

const SCREEN_SIZE_X: f32 = 1000.0;
const SCREEN_SIZE_Y: f32 = 1000.0;

const MAX_ACC_X: f32 = 0.00005;
const MAX_ACC_Y: f32 = 0.00005;
const MAX_SPEED_X: f32 = 0.005;
const MAX_SPEED_Y: f32 = 0.005;
const ACC_STEP_X: f32 = 0.00001;
const ACC_STEP_Y: f32 = 0.00001;

const METEOR_MAX_SIZE: f32 = 0.02;
const METEOR_MIN_SIZE: f32 = 0.007;
const METEOR_DESTROY_RADIUS: f32 = 0.001;
const METEOR_SPAWN_INTERVAL: f32 = 0.5;

const POPULATION_START: f32 = 2500.0;
const POP_MULTI_FACTOR: f32 = 1.0005;
const VICTORY_PROGRESS_TICK: f32 = 0.0001;

const OVERPOP_NUMBER: f32 = 10000.0;
const OVERPOP_MIN_WARNING_INTERVAL: f32 = 30.0;
const OVERPOP_WARNING_TTL: f32 = 400.0;

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

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }

    Ok(())
}

struct SaveThePinkSkin {
    // spaceship: GameObject,
    // earth: GameObject,
    id_generator: usize,
    objects: HashMap<usize, GameObject>,
    spaceship_id: Option<usize>,
    earth_id: Option<usize>,
    controls: Controls,
    rng: ThreadRng,
    next_meteor_spawn: Option<f32>,
    game_resources: GameResources,
    victory_result: Option<GameVictoryResult>,
    text_population_id: Option<usize>,
    text_spaceship_hp_id: Option<usize>,
    text_victory_progress_id: Option<usize>,
    population_million: f32,
    victory_progress: f32,
    spaceship_hp: f32,
    next_overpop_warning: f32,
    next_overpop_warning_enabled: bool,
}

struct GameResources {
    font: graphics::Font,
    death_sound: audio::Source,
    earth_meteor_sound: audio::Source,
    earth_end_sound: audio::Source,
    meteor_bounce_sound: audio::Source,
    meteor_explosion_sound: audio::Source,
    overpopulation_warning_sound: audio::Source,
    overpopulation_end_sound: audio::Source,
    ship_meteor_sound: audio::Source,
    shoot_sound: audio::Source,
    victory_sound: audio::Source,
}

#[derive(Clone, Debug)]
enum GameVictoryResult {
    ShipDestroyed,
    EveryoneDead,
    OverPopulation,
    Victory,
}

#[derive(Clone, Debug)]
enum Shape {
    Circle,
    Text,
}

#[derive(Clone, Debug)]
struct CircleData {
    radius: f32,
    color: graphics::Color,
}

#[derive(Clone, Debug)]
struct TextData {
    text: graphics::Text,
    expiration_time: Option<i32>,
    font_size: f32,
}

#[derive(Clone, Debug, PartialEq)]
enum ObjType {
    Ship,
    Earth,
    Meteor,
    Projectile,
    UI,
}

#[derive(Clone, Debug)]
struct GameObject {
    id: usize,
    transform: Transform,

    // FIXME: there must be a better way...
    object_type: ObjType,
    shape: Shape,
    circle_data: Option<CircleData>,
    text_data: Option<TextData>,
    ttl: Option<f32>,

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

#[derive(Default, Clone, Debug)]
struct Controls {
    left_right: Option<Direction>,
    up_down: Option<Direction>,
}

impl SaveThePinkSkin {
    pub fn new(ctx: &mut Context) -> GameResult<SaveThePinkSkin> {
        // Load/create resources such as images here.
        let font = graphics::Font::new(ctx, "/PixelEmulator-xq08.ttf")?;
        let death_sound = audio::Source::new(ctx, "/death.wav")?;
        let earth_meteor_sound = audio::Source::new(ctx, "/earth-meteor.wav")?;
        let earth_end_sound = audio::Source::new(ctx, "/earth-end.wav")?;
        let meteor_bounce_sound = audio::Source::new(ctx, "/meteor-bounce.wav")?;
        let meteor_explosion_sound = audio::Source::new(ctx, "/meteor-explosion.wav")?;
        let overpopulation_warning_sound = audio::Source::new(ctx, "/overpopulation-warning.wav")?;
        let overpopulation_end_sound = audio::Source::new(ctx, "/overpop-end.wav")?;
        let ship_meteor_sound = audio::Source::new(ctx, "/ship-meteor.wav")?;
        let shoot_sound = audio::Source::new(ctx, "/shoot.wav")?;
        let victory_sound = audio::Source::new(ctx, "/victory.wav")?;

        let game = SaveThePinkSkin::init(GameResources {
            font,
            death_sound,
            earth_meteor_sound,
            earth_end_sound,
            meteor_bounce_sound,
            meteor_explosion_sound,
            overpopulation_warning_sound,
            overpopulation_end_sound,
            ship_meteor_sound,
            shoot_sound,
            victory_sound,
        });

        Ok(game)
    }

    fn init(game_resources: GameResources) -> SaveThePinkSkin {
        let mut game = SaveThePinkSkin {
            id_generator: 0,
            objects: HashMap::new(),
            controls: Default::default(),
            spaceship_id: None,
            earth_id: None,
            rng: rand::thread_rng(),
            next_meteor_spawn: None,
            game_resources,
            victory_result: None,
            text_population_id: None,
            text_spaceship_hp_id: None,
            text_victory_progress_id: None,
            population_million: POPULATION_START,
            spaceship_hp: 100.0,
            victory_progress: 0.0,
            next_overpop_warning: 0.0,
            next_overpop_warning_enabled: true,
        };
        game.add_spaceship();
        game.add_earth();
        game.add_text_population();
        game.add_text_spaceship_hp();
        game.add_text_victory_progress();

        game
    }

    fn restart(&mut self) {
        // *self = SaveThePinkSkin::init(self.game_resources);
        self.id_generator = 0;
        self.objects = HashMap::new();
        self.controls = Default::default();
        self.spaceship_id = None;
        self.earth_id = None;
        self.rng = rand::thread_rng();
        self.next_meteor_spawn = None;
        self.victory_result = None;
        self.text_population_id = None;
        self.text_spaceship_hp_id = None;
        self.population_million = POPULATION_START;
        self.spaceship_hp = 100.0;
        self.victory_progress = 0.0;
        self.next_overpop_warning = 0.0;
        self.next_overpop_warning_enabled = true;
        self.add_spaceship();
        self.add_earth();
        self.add_text_population();
        self.add_text_spaceship_hp();
        self.add_text_victory_progress();
    }

    fn make_object(
        &mut self,
        transform: Transform,
        object_type: ObjType,
        shape: Shape,
        circle_data: Option<CircleData>,
        text_data: Option<TextData>,
    ) -> usize {
        self.id_generator += 1;
        let id = self.id_generator;
        let ttl = if object_type == ObjType::Projectile {
            Some(50.0)
        } else {
            None
        };
        self.objects.insert(
            id,
            GameObject {
                id,
                transform,
                shape,
                object_type,
                circle_data: circle_data,
                text_data: text_data,
                ttl,
                collidable: true,
            },
        );
        id
    }

    fn add_spaceship(&mut self) {
        let id = self.make_object(
            Transform {
                pos_x: 0.1,
                pos_y: 0.3,
                vel_x: 0.0,
                vel_y: 0.0,
                acc_x: 0.0,
                acc_y: 0.0,
            },
            ObjType::Ship,
            Shape::Circle,
            Some(CircleData {
                radius: 0.01,
                color: graphics::Color::new(0.5, 0.5, 0.7, 1.0),
            }),
            None,
        );
        self.spaceship_id = Some(id);
    }

    fn add_earth(&mut self) {
        let id = self.make_object(
            Transform {
                pos_x: 0.5,
                pos_y: 0.5,
                vel_x: 0.0,
                vel_y: 0.0,
                acc_x: 0.0,
                acc_y: 0.0,
            },
            ObjType::Earth,
            Shape::Circle,
            Some(CircleData {
                radius: 0.1,
                color: graphics::Color::new(0.3, 0.7, 0.3, 1.0),
            }),
            None,
        );
        self.earth_id = Some(id);
    }

    fn add_text_population(&mut self) {
        let id = self.make_object(
            Transform {
                pos_x: 0.2,
                pos_y: 0.0,
                vel_x: 0.0,
                vel_y: 0.0,
                acc_x: 0.0,
                acc_y: 0.0,
            },
            ObjType::UI,
            Shape::Text,
            None,
            Some(TextData {
                text: graphics::Text::default(),
                expiration_time: None,
                font_size: 48.0,
            }),
        );
        self.get_mut(id).collidable = false;
        self.text_population_id = Some(id);
    }

    fn add_text_spaceship_hp(&mut self) {
        let id = self.make_object(
            Transform {
                pos_x: 0.4,
                pos_y: 1.0 - 34.0 / SCREEN_SIZE_Y,
                vel_x: 0.0,
                vel_y: 0.0,
                acc_x: 0.0,
                acc_y: 0.0,
            },
            ObjType::UI,
            Shape::Text,
            None,
            Some(TextData {
                text: graphics::Text::default(),
                expiration_time: None,
                font_size: 34.0,
            }),
        );
        self.get_mut(id).collidable = false;
        self.text_spaceship_hp_id = Some(id);
    }

    fn add_text_victory_result(&mut self) {
        let end_text = match self.victory_result {
            Some(GameVictoryResult::EveryoneDead) => "Catastrophic event.",
            Some(GameVictoryResult::OverPopulation) => "Overpopulation:\nFamine and War.",
            Some(GameVictoryResult::ShipDestroyed) => "You have died.",
            Some(GameVictoryResult::Victory) => "Nursery finished.\nReady for space travel.",
            None => "Well that didn't work",
        };
        let end_text_full = match self.victory_result {
            Some(GameVictoryResult::Victory) => format!("{}", end_text),
            _ => format!("{}\n{}", end_text, "R to Restart"),
        };
        let id = self.make_object(
            Transform {
                pos_x: 0.35,
                pos_y: 0.35,
                vel_x: 0.0,
                vel_y: 0.0,
                acc_x: 0.0,
                acc_y: 0.0,
            },
            ObjType::UI,
            Shape::Text,
            None,
            Some(TextData {
                text: graphics::Text::new((end_text_full, self.game_resources.font, 34.0)),
                expiration_time: None,
                font_size: 34.0,
            }),
        );
        self.get_mut(id).collidable = false;
    }

    fn add_text_victory_progress(&mut self) {
        let id = self.make_object(
            Transform {
                pos_x: 0.2,
                pos_y: 0.0 + 48.0 / SCREEN_SIZE_Y,
                vel_x: 0.0,
                vel_y: 0.0,
                acc_x: 0.0,
                acc_y: 0.0,
            },
            ObjType::UI,
            Shape::Text,
            None,
            Some(TextData {
                text: graphics::Text::default(),
                expiration_time: None,
                font_size: 21.0,
            }),
        );
        self.get_mut(id).collidable = false;
        self.text_victory_progress_id = Some(id);
    }

    fn add_meteor_impact_text(&mut self, pos_x: f32, pos_y: f32, damage: f32) {
        let id = self.make_object(
            Transform {
                pos_x: pos_x - 0.1,
                pos_y: pos_y - 13.0 / SCREEN_SIZE_Y,
                vel_x: 0.0,
                vel_y: -0.00001,
                acc_x: 0.0,
                acc_y: 0.0,
            },
            ObjType::UI,
            Shape::Text,
            None,
            Some(TextData {
                text: graphics::Text::new((
                    format!("{} dead", population_to_string(damage)),
                    self.game_resources.font,
                    13.0,
                )),
                expiration_time: None,
                font_size: 13.0,
            }),
        );
        let object = self.get_mut(id);
        object.ttl = Some(300.0);
        self.get_mut(id).collidable = false;
    }

    fn maybe_make_overpopulation_warning(&mut self, time: f32) {
        if time < self.next_overpop_warning || !self.next_overpop_warning_enabled {
            return;
        }

        self.next_overpop_warning_enabled = false;

        let _ = self.game_resources.overpopulation_warning_sound.play();
        self.next_overpop_warning += OVERPOP_MIN_WARNING_INTERVAL;
        let id = self.make_object(
            Transform {
                pos_x: 0.25,
                pos_y: 0.3,
                vel_x: 0.0,
                vel_y: -0.00001,
                acc_x: 0.0,
                acc_y: 0.0,
            },
            ObjType::UI,
            Shape::Text,
            None,
            Some(TextData {
                text: graphics::Text::new((
                    "Overpopulation imminent",
                    self.game_resources.font,
                    32.0,
                )),
                expiration_time: None,
                font_size: 32.0,
            }),
        );
        let object = self.get_mut(id);
        object.ttl = Some(OVERPOP_WARNING_TTL);
        self.get_mut(id).collidable = false;
    }

    fn generate_meteor(&mut self) {
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

        let radius = self.rng.gen_range(METEOR_MIN_SIZE, METEOR_MAX_SIZE);

        match dir {
            Direction::Up => {
                meteor.pos_x = pos;
                meteor.pos_y = 0.0 - radius;
                if self.rng.gen::<f32>() > 0.5 {
                    meteor.vel_x *= -1.0;
                }
            }
            Direction::Down => {
                meteor.pos_x = pos;
                meteor.pos_y = 1.0 + radius;
                meteor.vel_y *= -1.0;
                if self.rng.gen::<f32>() > 0.5 {
                    meteor.vel_x *= -1.0;
                }
            }
            Direction::Left => {
                meteor.pos_x = 0.0 - radius;
                meteor.pos_y = pos;
                if self.rng.gen::<f32>() > 0.5 {
                    meteor.vel_y *= -1.0;
                }
            }
            Direction::Right => {
                meteor.pos_x = 1.0 + radius;
                meteor.pos_y = pos;
                meteor.vel_x *= -1.0;
                if self.rng.gen::<f32>() > 0.5 {
                    meteor.vel_y *= -1.0;
                }
            }
        };
        self.add_meteor(meteor, radius);
    }

    fn add_meteor(&mut self, transform: Transform, radius: f32) {
        self.make_object(
            transform,
            ObjType::Meteor,
            Shape::Circle,
            Some(CircleData {
                radius: radius,
                color: graphics::Color::new(0.878, 0.603, 0.282, 1.0),
            }),
            None,
        );
    }

    fn remove_object(&mut self, id: usize) {
        self.objects.remove(&id);
        if self.spaceship_id == Some(id) {
            self.spaceship_id = None;
        }
        if self.earth_id == Some(id) {
            self.earth_id = None;
        }
    }

    fn shoot(&mut self, x: f32, y: f32) {
        const PROJECTILE_RADIUS: f32 = 0.001;
        const PROJECTILE_SPEED: f32 = 0.01;

        if let Some(spaceship_id) = self.spaceship_id {
            let &spaceship = self.objects.get(&spaceship_id).as_ref().unwrap();
            let pos_x = spaceship.transform.pos_x;
            let pos_y = spaceship.transform.pos_y;
            let dx = x - pos_x;
            let dy = y - pos_y;
            let d = (dx * dx + dy * dy).sqrt();

            let _ = self.game_resources.shoot_sound.play();
            self.make_object(
                Transform {
                    pos_x: pos_x,
                    pos_y: pos_y,
                    vel_x: PROJECTILE_SPEED * dx / d,
                    vel_y: PROJECTILE_SPEED * dy / d,
                    acc_x: 0.0,
                    acc_y: 0.0,
                },
                ObjType::Projectile,
                Shape::Circle,
                Some(CircleData {
                    radius: PROJECTILE_RADIUS,
                    color: graphics::Color::new(0.7, 0.9, 0.2, 1.0),
                }),
                None,
            );
        }
    }

    fn get(&self, id: usize) -> &GameObject {
        return self.objects.get(&id).unwrap();
    }

    fn get_mut(&mut self, id: usize) -> &mut GameObject {
        self.objects.get_mut(&id).unwrap()
    }
}

fn dist_object(first: &GameObject, second: &GameObject) -> f32 {
    let size_dist = match (&first.shape, &second.shape) {
        (Shape::Circle, Shape::Circle) => {
            first.circle_data.as_ref().unwrap().radius + second.circle_data.as_ref().unwrap().radius
        }
        _ => 0.0,
    };
    dist_transform(&first.transform, &second.transform) - size_dist
}

fn dist_transform(first: &Transform, second: &Transform) -> f32 {
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
    let mut iter1 = game.objects.values();
    loop {
        match iter1.next() {
            Some(obj1) => {
                let iter2 = iter1.clone();
                for obj2 in iter2 {
                    if obj1.id == obj2.id {
                        continue;
                    }

                    if obj1.collidable && obj2.collidable && dist_object(&obj1, &obj2) <= 0.0 {
                        collisions.push(Collision {
                            first: obj1.id,
                            second: obj2.id,
                        });
                    }
                }
            }
            None => {
                break;
            }
        }
    }

    collisions
}

struct MeteorData {
    transform: Transform,
    radius: f32,
}

struct CollisionResults {
    created: Vec<MeteorData>,
    destroyed_ids: Vec<usize>,
    ship_damage: f32,
    population_damage: f32,
}

fn gen_safe_range(rng: &mut ThreadRng, first: f32, second: f32) -> f32 {
    if first == second {
        first
    } else if first > second {
        rng.gen_range(second, first)
    } else {
        rng.gen_range(first, second)
    }
}

fn radius_to_earth_damage(radius: f32) -> f32 {
    radius * radius * 100.0 * 100.0 * 13.0 * 100.0
}

fn radius_to_ship_damage(radius: f32) -> f32 {
    radius * radius * 100.0 * 100.0 * 13.0 * 2.0
}

fn population_to_string(population: f32) -> String {
    if population > 400.0 {
        format!("{:.1}B", population / 1000.0)
    } else {
        format!("{:.0}M", population)
    }
}

fn process_collisions(game: &mut SaveThePinkSkin, collisions: &Vec<Collision>) -> CollisionResults {
    let mut results = CollisionResults {
        created: Vec::new(),
        destroyed_ids: Vec::new(),
        ship_damage: 0.0,
        population_damage: 0.0,
    };
    let mut destroyed_unique = HashSet::<usize>::new();

    for collision in collisions {
        let first_type = game.get(collision.first).object_type.clone();
        let second_type = game.get(collision.second).object_type.clone();
        match (&first_type, &second_type) {
            (ObjType::Ship, ObjType::Earth) | (ObjType::Earth, ObjType::Ship) => {
                results.ship_damage = 1000.0;
            }
            (ObjType::Ship, ObjType::Meteor) | (ObjType::Meteor, ObjType::Ship) => {
                let collider = if first_type == ObjType::Meteor {
                    collision.first
                } else {
                    collision.second
                };
                results.ship_damage +=
                    radius_to_ship_damage(game.get(collider).circle_data.as_ref().unwrap().radius);
                destroyed_unique.insert(collider);
                let _ = game.game_resources.ship_meteor_sound.play();
            }
            (ObjType::Earth, ObjType::Meteor) | (ObjType::Meteor, ObjType::Earth) => {
                let collider = if first_type == ObjType::Meteor {
                    collision.first
                } else {
                    collision.second
                };
                let collider_object = game.get(collider);
                let pos_x = collider_object.transform.pos_x;
                let pos_y = collider_object.transform.pos_y;
                let damage =
                    radius_to_earth_damage(game.get(collider).circle_data.as_ref().unwrap().radius);
                results.population_damage += damage;
                destroyed_unique.insert(collider);
                let _ = game.game_resources.earth_meteor_sound.play();
                game.add_meteor_impact_text(pos_x, pos_y, damage);
            }
            (ObjType::Earth, ObjType::Projectile) => {
                destroyed_unique.insert(collision.second);
            }
            (ObjType::Projectile, ObjType::Earth) => {
                destroyed_unique.insert(collision.first);
            }
            (ObjType::Meteor, ObjType::Projectile) | (ObjType::Projectile, ObjType::Meteor) => {
                let collider = if first_type == ObjType::Meteor {
                    collision.first
                } else {
                    collision.second
                };

                let meteor = game.objects.get(&collider).unwrap();
                let transform = &meteor.transform;
                let radius_ratio: f32 = game.rng.gen_range(0.2, 0.5);
                let radius = meteor.circle_data.as_ref().unwrap().radius * radius_ratio;
                let vel_x = gen_safe_range(
                    &mut game.rng,
                    -transform.vel_x,
                    -transform.vel_x / radius_ratio,
                );
                let vel_y = gen_safe_range(
                    &mut game.rng,
                    -transform.vel_y,
                    -transform.vel_y / radius_ratio,
                );
                const MAX_GENERATED_VELOCITY: f32 = 0.001;
                let meteor = MeteorData {
                    transform: Transform {
                        pos_x: transform.pos_x,
                        pos_y: transform.pos_y,
                        vel_x: vel_x.abs().min(MAX_GENERATED_VELOCITY) * vel_y.signum(),
                        vel_y: vel_y.abs().min(MAX_GENERATED_VELOCITY) * vel_x.signum(),
                        acc_x: 0.0,
                        acc_y: 0.0,
                    },
                    radius: radius,
                };
                if meteor.radius > METEOR_DESTROY_RADIUS
                    && meteor.transform.pos_x.abs() > 0.02
                    && meteor.transform.pos_x.abs() < 0.98
                    && meteor.transform.pos_y.abs() > 0.02
                    && meteor.transform.pos_y.abs() < 0.98
                {
                    println!("{} X {}", collision.first, collision.second);
                    println!("add from collision: {}", meteor.radius);
                    results.created.push(meteor);
                }
                let _ = game.game_resources.meteor_explosion_sound.play();

                destroyed_unique.insert(collision.first);
                destroyed_unique.insert(collision.second);
            }
            (ObjType::Meteor, ObjType::Meteor) => {
                let m1 = game.objects.get(&collision.first).unwrap();
                let m2 = game.objects.get(&collision.second).unwrap();
                let t1 = &m1.transform;
                let t2 = &m2.transform;
                let r1 = m1.circle_data.as_ref().unwrap().radius;
                let r2 = m2.circle_data.as_ref().unwrap().radius;

                let dx = t1.pos_x - t2.pos_x;
                let dy = t1.pos_y - t2.pos_y;
                let radius_ratio = r1 / (r1 + r2);

                const MAX_GENERATED_VELOCITY: f32 = 0.001;

                let vel_x1 = gen_safe_range(&mut game.rng, -t1.vel_x, -t1.vel_x / radius_ratio);
                let vel_y1 = gen_safe_range(&mut game.rng, -t1.vel_y, -t1.vel_y / radius_ratio);
                let meteor = MeteorData {
                    transform: Transform {
                        // pos_x: gen_safe_range(&mut game.rng, t1.pos_x, t1.pos_x + dx),
                        // pos_y: gen_safe_range(&mut game.rng, t1.pos_y, t1.pos_y + dy),
                        pos_x: t1.pos_x,
                        pos_y: t1.pos_y,
                        vel_x: vel_x1.abs().min(MAX_GENERATED_VELOCITY) * vel_y1.signum(),
                        vel_y: vel_y1.abs().min(MAX_GENERATED_VELOCITY) * vel_x1.signum(),
                        acc_x: 0.0,
                        acc_y: 0.0,
                    },
                    radius: r1 * 0.7,
                };
                if meteor.radius > METEOR_DESTROY_RADIUS
                    && meteor.transform.pos_x.abs() > 0.01
                    && meteor.transform.pos_x.abs() < 0.99
                    && meteor.transform.pos_y.abs() > 0.01
                    && meteor.transform.pos_y.abs() < 0.99
                {
                    results.created.push(meteor);
                }

                let vel2_x = gen_safe_range(&mut game.rng, -t1.vel_x, -t2.vel_x / radius_ratio);
                let vel2_y = gen_safe_range(&mut game.rng, -t1.vel_y, -t2.vel_y / radius_ratio);
                let meteor = MeteorData {
                    transform: Transform {
                        // pos_x: gen_safe_range(&mut game.rng, t2.pos_x, t2.pos_x - dx),
                        // pos_y: gen_safe_range(&mut game.rng, t2.pos_y, t2.pos_y - dy),
                        // vel_x: gen_safe_range(&mut game.rng, -t2.vel_x, -t2.vel_x * radius_ratio),
                        // vel_y: gen_safe_range(&mut game.rng, -t2.vel_y, -t2.vel_y * radius_ratio),
                        pos_x: t2.pos_x,
                        pos_y: t2.pos_y,
                        vel_x: vel2_x.abs().min(MAX_GENERATED_VELOCITY) * vel2_y.signum(),
                        vel_y: vel2_y.abs().min(MAX_GENERATED_VELOCITY) * vel2_x.signum(),
                        acc_x: 0.0,
                        acc_y: 0.0,
                    },
                    radius: r2 * 0.7,
                };
                results.created.push(meteor);

                // const MIN_SPAWN: u32 = 1;
                // const MAX_SPAWN: u32 = 3;
                // let spawn_count: usize = game.rng.gen_range(MIN_SPAWN, MAX_SPAWN) as usize;

                // let total_radius: f32 = (r1 + r2) * 0.3;
                // let mut radius_weights: Vec<f32> = Vec::new();
                // for _ in 0..spawn_count {
                //     radius_weights.push(game.rng.gen());
                // }
                // let weight_factor: f32 = total_radius / radius_weights.iter().sum::<f32>();

                // for i in 0..spawn_count {
                //     let meteor = MeteorData {
                //         transform: Transform {
                //             pos_x: gen_safe_range(&mut game.rng, t1.pos_x, t2.pos_x),
                //             pos_y: gen_safe_range(&mut game.rng, t1.pos_y, t2.pos_y),
                //             vel_x: gen_safe_range(&mut game.rng, -t1.vel_x, -t2.vel_x),
                //             vel_y: gen_safe_range(&mut game.rng, -t1.vel_y, -t2.vel_y),
                //             acc_x: 0.0,
                //             acc_y: 0.0,
                //         },
                //         radius: radius_weights[i] * weight_factor,
                //     };
                //     const MIN_RADIUS: f32 = 0.007;
                //     if meteor.radius > MIN_RADIUS {
                //         results.created.push(meteor);
                //     }
                // }
                let _ = game.game_resources.meteor_bounce_sound.play();
                println!("Collision: {} x {}", collision.first, collision.second);
                destroyed_unique.insert(collision.first);
                destroyed_unique.insert(collision.second);
            }
            _ => {}
        };
    }
    for destroyed in destroyed_unique {
        results.destroyed_ids.push(destroyed);
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

fn add_new(game: &mut SaveThePinkSkin, created: Vec<MeteorData>) {
    for meteor in created {
        game.add_meteor(meteor.transform, meteor.radius);
    }
}

impl EventHandler for SaveThePinkSkin {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const TARGET_FPS: u32 = 60;

        let time: f32 = ggez::timer::time_since_start(&ctx).as_secs() as f32;

        if let Some(next_meteor_spawn) = self.next_meteor_spawn {
            while time > self.next_meteor_spawn.unwrap() {
                self.generate_meteor();
                self.next_meteor_spawn =
                    Some(self.next_meteor_spawn.unwrap() + METEOR_SPAWN_INTERVAL);
            }
        } else {
            self.next_meteor_spawn = Some(time + METEOR_SPAWN_INTERVAL);
        }

        while ggez::timer::check_update_time(ctx, TARGET_FPS) {
            if let Some(spaceship_id) = self.spaceship_id {
                let controls = self.controls.clone();
                let spaceship = self.get_mut(spaceship_id);
                let spaceship_tr = &mut spaceship.transform;
                match controls.left_right {
                    Some(Direction::Left) => spaceship_tr.acc_x -= ACC_STEP_X,
                    Some(Direction::Right) => spaceship_tr.acc_x += ACC_STEP_X,
                    _ => {
                        spaceship_tr.acc_x =
                            spaceship_tr.acc_x.signum() * (spaceship_tr.acc_x.abs() - ACC_STEP_X)
                    }
                };

                match controls.up_down {
                    Some(Direction::Up) => spaceship_tr.acc_y -= ACC_STEP_Y,
                    Some(Direction::Down) => spaceship_tr.acc_y += ACC_STEP_Y,
                    _ => {
                        spaceship_tr.acc_y =
                            spaceship_tr.acc_y.signum() * (spaceship_tr.acc_y.abs() - ACC_STEP_Y)
                    }
                };

                spaceship_tr.acc_x = na::clamp(spaceship_tr.acc_x, -MAX_ACC_X, MAX_ACC_X);
                spaceship_tr.acc_y = na::clamp(spaceship_tr.acc_y, -MAX_ACC_Y, MAX_ACC_Y);
                spaceship_tr.vel_x = na::clamp(spaceship_tr.vel_x, -MAX_SPEED_X, MAX_SPEED_X);
                spaceship_tr.vel_y = na::clamp(spaceship_tr.vel_y, -MAX_SPEED_Y, MAX_SPEED_Y);
            }

            for object in &mut self.objects.values_mut() {
                let transform = &mut object.transform;
                transform.vel_x += transform.acc_x;
                transform.vel_y += transform.acc_y;

                transform.pos_x += transform.vel_x;
                transform.pos_y += transform.vel_y;

                let size_dist = match object.shape {
                    Shape::Circle => object.circle_data.as_ref().unwrap().radius,
                    _ => 0.0,
                };

                if transform.pos_x > 1.0 + size_dist * 1.1 {
                    transform.pos_x = -size_dist;
                } else if transform.pos_x < -size_dist * 1.1 {
                    transform.pos_x = 1.0 + size_dist;
                }
                if transform.pos_y > 1.0 + size_dist * 1.1 {
                    transform.pos_y = -size_dist;
                } else if transform.pos_y < -size_dist * 1.1 {
                    transform.pos_y = 1.0 + size_dist;
                }
            }

            let mut to_destroy = vec![];
            for object in &mut self.objects.values_mut() {
                if let Some(ttl) = object.ttl {
                    object.ttl = Some(ttl - 1.0);
                    if object.ttl <= Some(0.0) {
                        to_destroy.push(object.id);
                    }
                }
            }
            for destroy in to_destroy {
                self.remove_object(destroy);
            }

            let mut to_destroy = vec![];
            for object in &mut self.objects.values_mut() {
                if object.collidable && object.object_type == ObjType::Meteor {
                    if let Some(circle_data) = &mut object.circle_data {
                        let relative_size = (circle_data.radius - METEOR_DESTROY_RADIUS)
                            / (METEOR_MAX_SIZE - METEOR_DESTROY_RADIUS);

                        let size_factor = (0.3 - relative_size).max(0.0);
                        let decay_rate = 0.0005 + size_factor * 0.03;
                        circle_data.radius *= 1.0 - decay_rate;
                        if circle_data.radius < METEOR_DESTROY_RADIUS {
                            to_destroy.push(object.id);
                        }
                    }
                }
            }
            for destroy in to_destroy {
                self.remove_object(destroy);
            }

            let collisions = find_collisions(self);
            let results = process_collisions(self, &collisions);
            self.spaceship_hp -= results.ship_damage;
            self.population_million -= results.population_damage;
            cleanup_destroyed(self, &results.destroyed_ids);
            add_new(self, results.created);

            self.population_million *= POP_MULTI_FACTOR;
            self.victory_progress += VICTORY_PROGRESS_TICK;

            match self.victory_result {
                None => {
                    let mut finished = true;
                    if self.population_million <= 0.0 {
                        self.victory_result = Some(GameVictoryResult::EveryoneDead);
                        let _ = self.game_resources.earth_end_sound.play();
                    } else if self.spaceship_hp <= 0.0 {
                        self.victory_result = Some(GameVictoryResult::ShipDestroyed);
                        let _ = self.game_resources.death_sound.play();
                        if let Some(spaceship_id) = self.spaceship_id {
                            self.remove_object(spaceship_id);
                        }
                    } else if self.population_million >= OVERPOP_NUMBER {
                        self.victory_result = Some(GameVictoryResult::OverPopulation);
                        let _ = self.game_resources.overpopulation_end_sound.play();
                    } else if self.victory_progress >= 1.0 {
                        self.victory_result = Some(GameVictoryResult::Victory);
                        let _ = self.game_resources.victory_sound.play();
                    } else {
                        finished = false;
                    }
                    if finished {
                        self.add_text_victory_result()
                    }
                }
                _ => {}
            }

            if self.population_million > 7000.0 {
                self.maybe_make_overpopulation_warning(time);
            } else {
                self.next_overpop_warning_enabled = true;
            }

            self.spaceship_hp = self.spaceship_hp.max(0.0);
            self.population_million = self.population_million.max(0.0);
            self.victory_progress = self.victory_progress.min(1.0);

            if let Some(text_population_id) = self.text_population_id {
                let text_str = format!(
                    "Population: {}",
                    population_to_string(self.population_million)
                );
                let text_str = if self.population_million > 7000.0 {
                    format!("{} (!)", text_str)
                } else {
                    text_str
                };
                let object = self.objects.get_mut(&text_population_id).unwrap();
                if let Some(text_data) = &mut object.text_data {
                    text_data.text = graphics::Text::new((
                        text_str,
                        self.game_resources.font,
                        text_data.font_size,
                    ));
                }
            }
            if let Some(text_spaceship_hp_id) = self.text_spaceship_hp_id {
                let text_str = format!("HP: {:.0}", self.spaceship_hp);
                let object = self.objects.get_mut(&text_spaceship_hp_id).unwrap();
                if let Some(text_data) = &mut object.text_data {
                    text_data.text = graphics::Text::new((
                        text_str,
                        self.game_resources.font,
                        text_data.font_size,
                    ));
                }
            }
            if let Some(text_victory_progress_id) = self.text_victory_progress_id {
                let text_str = format!("Space Age Progress: {:.0}%", 100.0 * self.victory_progress);
                let object = self.objects.get_mut(&text_victory_progress_id).unwrap();
                if let Some(text_data) = &mut object.text_data {
                    text_data.text = graphics::Text::new((
                        text_str,
                        self.game_resources.font,
                        text_data.font_size,
                    ));
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        for obj in self.objects.values() {
            match obj.shape {
                Shape::Circle => {
                    let circle_data = obj.circle_data.as_ref().unwrap();
                    let circle = graphics::Mesh::new_circle(
                        ctx,
                        graphics::DrawMode::fill(),
                        na::Point2::new(0.0, 0.0),
                        circle_data.radius * SCREEN_SIZE_X,
                        0.1,
                        circle_data.color,
                    )?;
                    graphics::draw(
                        ctx,
                        &circle,
                        (na::Point2::new(
                            obj.transform.pos_x * SCREEN_SIZE_X,
                            obj.transform.pos_y * SCREEN_SIZE_Y,
                        ),),
                    )?;
                }
                _ => {}
            }
        }

        for obj in self.objects.values() {
            match obj.shape {
                Shape::Text => {
                    let text_data = &obj.text_data.as_ref().unwrap();
                    graphics::draw(
                        ctx,
                        &text_data.text,
                        (na::Point2::new(
                            obj.transform.pos_x * SCREEN_SIZE_X,
                            obj.transform.pos_y * SCREEN_SIZE_Y,
                        ),),
                    )?;
                }
                _ => {}
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
        if keycode == KeyCode::R && self.victory_result.is_some() {
            self.restart();
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

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            self.shoot(x / SCREEN_SIZE_X, y / SCREEN_SIZE_Y);
        }
    }
}
