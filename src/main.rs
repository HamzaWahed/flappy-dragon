use bracket_lib::prelude::*;

enum GameMode {
    Menu,
    Playing,
    End,
}

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 75.0;

struct Player {
    x: f32, // world position
    y: f32,
    velocity: f32, // downward velocity
}

impl Player {
    fn new(x: f32, y: f32) -> Self {
        Player {
            x,
            y,
            velocity: 0.0,
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set(0, self.y as i32, YELLOW, BLACK, to_cp437('@'));
    }

    fn gravity_and_move(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }

        self.y += self.velocity;
        self.x += 1.0; // keep track of level progress
        if self.y < 0.0 {
            self.y = 0.0;
        }
    }

    fn flap(&mut self) {
        self.velocity = -2.0;
    }
}

struct Obstacle {
    x: f32,     // world-space position
    gap_y: i32, // center of the gap
    size: i32,  // length of the gap
}

impl Obstacle {
    fn new(x: f32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range(10, 40),   // between 10 and 39
            size: i32::max(2, 20 - score), // make walls smaller as player progresses
        }
    }

    fn render(&mut self, ctx: &mut BTerm, player_x: f32) {
        let obstacle_screen_x = self.x - player_x;
        let half_size = self.size / 2;

        for y in 0..self.gap_y - half_size {
            ctx.set(obstacle_screen_x as i32, y, RED, BLACK, to_cp437('|'));
        }

        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(obstacle_screen_x as i32, y, RED, YELLOW, to_cp437('|'));
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = player.x == self.x;
        let player_above_gap = (player.y as i32) < self.gap_y - half_size;
        let player_below_gap = (player.y as i32) > self.gap_y + half_size;
        does_x_match && (player_below_gap || player_above_gap)
    }
}

struct State {
    mode: GameMode,
    player: Player,
    frame_time: f32,
    score: i32,
    obstacle: Obstacle,
}

impl State {
    fn new() -> Self {
        State {
            mode: GameMode::Menu,
            player: Player::new(5.0, 25.0),
            frame_time: 0.0,
            score: 0,
            obstacle: Obstacle::new(SCREEN_WIDTH as f32, 0),
        }
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Welcome to Flappy Dragon");
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        // clear the context and make the background navy
        ctx.cls_bg(NAVY);
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.gravity_and_move();
        }

        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }

        self.player.render(ctx);
        ctx.print(0, 0, "Press SPACE to flap.");
        ctx.print(0, 1, format!("Score: {}", self.score));

        self.obstacle.render(ctx, self.player.x);

        // passed the obstacle, create a new obstacle
        if self.player.x > self.obstacle.x {
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH as f32, self.score);
        }

        // going upwards decreases height
        if self.player.y as i32 > SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player) {
            self.mode = GameMode::End;
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "You are dead!");
        ctx.print_centered(6, format!("You earned {} points", self.score));
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn restart(&mut self) {
        self.mode = GameMode::Playing;
        self.player = Player::new(5.0, 25.0);
        self.frame_time = 0.0;
        self.score = 0;
        self.obstacle = Obstacle::new(SCREEN_WIDTH as f32, 0);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::End => self.dead(ctx),
        }
    }
}

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Dragon")
        .build()?;

    main_loop(context, State::new())
}
