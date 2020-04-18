use ggez::event::{self, EventHandler};
use ggez::nalgebra as na;
use ggez::{graphics, Context, ContextBuilder, GameResult};

fn main() {
    // Make a Context.
    let (mut ctx, mut event_loop) = ContextBuilder::new("save_the_pink_skins", "gajop")
        .build()
        .expect("Failed to create create ggez context. Please report this error");

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let mut my_game = SaveThePinkSkin::new(&mut ctx);
    add_spaceship(
        &mut my_game,
        GameObject {
            pos_x: 80.0,
            pos_y: 340.0,
            size_x: 10.0,
            size_y: 10.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            color: graphics::Color::new(0.5, 0.5, 0.7, 1.0),
        },
    );
    add_earth(
        &mut my_game,
        GameObject {
            pos_x: 500.0,
            pos_y: 500.0,
            size_x: 50.0,
            size_y: 50.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            color: graphics::Color::new(0.3, 0.7, 0.3, 1.0),
        },
    );

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}

struct GameObject {
    pos_x: f32,
    pos_y: f32,
    size_x: f32,
    size_y: f32,
    velocity_x: f32,
    velocity_y: f32,
    color: graphics::Color,
}

struct SaveThePinkSkin {
    // spaceship: GameObject,
    // earth: GameObject,
    objects: Vec<GameObject>,
}

fn add_spaceship(game: &mut SaveThePinkSkin, game_object: GameObject) {
    game.objects.push(game_object)
}

fn add_earth(game: &mut SaveThePinkSkin, game_object: GameObject) {
    game.objects.push(game_object)
}

impl SaveThePinkSkin {
    pub fn new(_ctx: &mut Context) -> SaveThePinkSkin {
        // Load/create resources such as images here.
        SaveThePinkSkin { objects: vec![] }
    }
}

impl EventHandler for SaveThePinkSkin {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        // Update code here...
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
                object.size_x,
                0.1,
                object.color,
            )?;
            graphics::draw(ctx, &circle, (na::Point2::new(object.pos_x, object.pos_y),))?;
        }

        graphics::present(ctx)
    }
}
