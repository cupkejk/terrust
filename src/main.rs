use macroquad::prelude::*;
use std::cell::RefCell;
use ::rand::{rng, RngExt};
use noise::{NoiseFn, Perlin};

const WORLD_SIZE: usize = 300;
const TOTAL_BLOCKS: usize = WORLD_SIZE * 16;

struct text {
    text: String,
    x: f32,
    y: f32,
    font_size: f32,
    color: Color,
}

struct Mob {
    x: f32,
    y: f32,
    velocity: Vec<f32>,
    isGrounded: bool,
    size_x: f32,
    size_y: f32,
    speed: f32,
    hitbox: Vec<Vec<f32>>,
}

impl Mob {
    fn new(x: f32, y: f32, game: &Game) -> Self {
        Self {
            x,
            y,
            velocity: vec![0.0, 0.0],
            isGrounded: false,
            size_x: game.block_size,
            size_y: game.block_size, // 1 block tall
            speed: 0.2,
            hitbox: Vec::new(),
        }
    }

    fn resize(&mut self, new_block_size: f32) {
        let scale_factor = new_block_size / self.size_x;
        self.size_x = new_block_size;
        self.size_y *= scale_factor;
        self.x *= scale_factor;
        self.y *= scale_factor;
    }

    fn calculate_hitbox(&mut self, game: &Game) {
        self.hitbox = vec![
            vec![self.x, self.y],
            vec![self.x + self.size_x, self.y],
            vec![self.x + self.size_x, self.y + self.size_y],
            vec![self.x, self.y + self.size_y],
        ];
    }

    fn does_collide(&self, chunks: &Vec<Vec<Chunk>>, game: &Game) -> bool {
        for hitbox_point in &self.hitbox {
            let global_block_x = (hitbox_point[0] / game.block_size).floor() as i32;
            let global_block_y = (hitbox_point[1] / game.block_size).floor() as i32;
            let chunk_x = global_block_x / 16;
            let chunk_y = global_block_y / 16;
            let local_x = (global_block_x % 16).abs() as usize;
            let local_y = (global_block_y % 16).abs() as usize;

            if chunk_x >= 0 && chunk_y >= 0 && (chunk_x as usize) < chunks.len() && (chunk_y as usize) < chunks[chunk_x as usize].len() {
                if chunks[chunk_x as usize][chunk_y as usize].blocks[local_x][local_y] != Blocks::Air {
                    return true;
                }
            }
        }
        false
    }

    fn update(&mut self, chunks: &Vec<Vec<Chunk>>, game: &Game) {
        // AI Logic: Move right and jump constantly
        self.velocity[0] += self.speed;
        if self.isGrounded {
            self.velocity[1] = -8.0; // Hop!
        }

        // Physics
        self.velocity[1] += 0.3; // Gravity
        self.velocity[0] *= 0.8; // Friction
        
        // Move X
        self.x += self.velocity[0];
        self.calculate_hitbox(game);
        if self.does_collide(chunks, game) {
            self.x -= self.velocity[0];
            self.velocity[0] = 0.0;
        }

        // Move Y
        self.y += self.velocity[1];
        self.calculate_hitbox(game);
        if self.does_collide(chunks, game) {
            self.isGrounded = self.velocity[1] > 0.0;
            self.y -= self.velocity[1];
            self.velocity[1] = 0.0;
        } else {
            self.isGrounded = false;
        }
    }

    fn draw(&self, player: &Player, game: &Game) {
        let screen_x = self.x - player.x + game.screen_width / 2.0;
        let screen_y = self.y - player.y + game.screen_height / 2.0;
        draw_rectangle(screen_x, screen_y, self.size_x, self.size_y, GREEN);
    }
}

struct Player {
    x: f32,
    y: f32,
    velocity: Vec<f32>,
    isGrounded: bool,
    size_x: f32,
    size_y: f32,
    hitbox: Vec<Vec<f32>>,
    speed: f32,
}

impl Player {
    fn new(game: &Game, chunks: &Vec<Vec<Chunk>>) -> Self {
        let hitbox = Vec::new();
        //calculate the coordinates of the middle of the world in y and the highest block in y from the middle, and spawn the player above that
        let middle = (TOTAL_BLOCKS as f32) / 2.0;
        let mut highest_block = 0.0;
        let chunk_num = (middle / 16.0).floor() as usize;
        let block_index = (middle % 16.0).floor() as usize;
        let mut highest_block = 0.0;
        for i in 0..chunks.len() {
            for j in 0..16 {
                while chunks[chunk_num][i].blocks[block_index][j] == Blocks::Air {
                    highest_block += 1.0 * game.block_size;
                    if block_index as i32 - (highest_block as i32) < 0 {
                        break;
                    }
                }
            }
        }
        let mut p = Self {
            x: middle * game.block_size,
            y: highest_block - game.block_size * 2.0,
            velocity: vec![0.0, 0.0],
            isGrounded: false,
            size_x: game.block_size,
            size_y: game.block_size * 2.0,
            hitbox,
            speed: 0.5,
        };
        p.calculate_hitbox(game);
        p
    }

    fn resize(&mut self, new_block_size: f32) {
        let scale_factor = new_block_size / self.size_x;
        self.size_x = new_block_size;
        self.size_y *= scale_factor;
        self.x *= scale_factor;
        self.y *= scale_factor;
    }

    fn draw(&self, game: &Game) {
        draw_rectangle(game.screen_width / 2.0, game.screen_height / 2.0 - game.block_size + game.block_size * 0.5, game.block_size * 0.5, game.block_size * 1.5, RED);
    }

    fn movee(&mut self, dx: f32, dy: f32, chunks: &Vec<Vec<Chunk>>, game: &Game) {
        self.velocity[0] += dx * self.speed;
        self.velocity[1] += dy * self.speed;
    }

    fn update(&mut self, chunks: &Vec<Vec<Chunk>>, game: &Game) {
        // 1. Apply Friction (Drag)
        // This slows the player down gradually when no keys are pressed
        //gravty
        self.velocity[1] += 0.2; // Gravity pulling down
        self.velocity[0] *= 0.85; 
        self.velocity[1] *= 0.99;
        if self.velocity[1] > 20.0 {
            self.velocity[1] = 10.0; // Terminal velocity
        }

        // 2. Move X and check collisions
        self.x += self.velocity[0] * game.block_size / 32.0; // Scale velocity by block size and a factor to control speed
        self.calculate_hitbox(game);
        if self.does_collide(chunks, game) {
            self.x -= self.velocity[0] * game.block_size / 32.0; // Move back to previous position
            self.velocity[0] = 0.0; // Stop horizontal momentum on hit
            self.calculate_hitbox(game);
        }

        // 3. Move Y and check collisions
        self.y += self.velocity[1] * game.block_size / 32.0;
        self.calculate_hitbox(game);
        if self.does_collide(chunks, game) {
            self.isGrounded = self.velocity[1] > 0.0; // If we were moving down, we're now grounded
            self.y -= self.velocity[1] * game.block_size / 32.0;
            self.velocity[1] = 0.0; // Stop vertical momentum on hit
            self.calculate_hitbox(game);
        }
        else {
            self.isGrounded = false;
        }
    }

    fn jump(&mut self) {
        if self.isGrounded {
            self.velocity[1] = -10.0; // Jumping up with a strong negative velocity
            self.isGrounded = false;
        }
    }

    fn get_chunk(&self, chunks: &Vec<Vec<Chunk>>, game: &Game) -> (i32, i32) {
        let chunk_x = (self.x / (game.block_size * 16.0)).floor() as i32;
        let chunk_y = (self.y / (game.block_size * 16.0)).floor() as i32;
        if chunk_x < 0 {
            return (-1, -1);
        }
        if chunk_y < 0 {
            return (-1, -1);
        }
        if chunk_x as usize >= chunks.len() {
            return (-1, -1);
        }
        if chunk_y as usize >= chunks[chunk_x as usize].len() {
            return (-1, -1);
        }
        (chunk_x, chunk_y)
    }

    fn does_collide(&self, chunks: &Vec<Vec<Chunk>>, game: &Game) -> bool {
        for hitbox_point in &self.hitbox {

            if hitbox_point[0] < 0.0 || hitbox_point[1] < 0.0 {
                return true;
            }
            if hitbox_point[0] >= (TOTAL_BLOCKS as f32) * game.block_size || hitbox_point[1] >= (TOTAL_BLOCKS as f32) * game.block_size {
                return true;
            }

            // 1. Get global block coordinates
            let global_block_x = (hitbox_point[0] / game.block_size).floor() as i32;
            let global_block_y = (hitbox_point[1] / game.block_size).floor() as i32;

            // 2. Determine which chunk these coordinates belong to
            let chunk_x = global_block_x / 16;
            let chunk_y = global_block_y / 16;

            println!("{} {}", chunk_x, hitbox_point[0] / game.block_size);

            if chunk_x < 0 || chunk_y < 0 {
                return true;
            }
            else if (chunk_x as usize) >= chunks.len() || (chunk_y as usize) >= chunks[chunk_x as usize].len() {
                return true;
            }

            // 3. Determine the local block index inside that chunk (0 to 15)
            let local_x = (global_block_x % 16).abs() as usize;
            let local_y = (global_block_y % 16).abs() as usize;

            // 4. Bounds check: Is the chunk within the world array?
            if chunk_x >= 0 && chunk_y >= 0 && 
            (chunk_x as usize) < chunks.len() && 
            (chunk_y as usize) < chunks[chunk_x as usize].len() {
                
                if chunks[chunk_x as usize][chunk_y as usize].blocks[local_x][local_y] != Blocks::Air {
                    return true;
                }
            }
        }
        false
    }

    fn calculate_hitbox(&mut self, game: &Game) {
        self.hitbox = vec![
            vec![self.x, self.y - game.block_size * 0.5],
            vec![self.x + game.block_size * 0.5, self.y - game.block_size * 0.5],
            vec![self.x + game.block_size * 0.5, self.y],
            vec![self.x + game.block_size * 0.5, self.y + game.block_size],
            vec![self.x, self.y + game.block_size],
            vec![self.x, self.y],
        ];
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Blocks {
    Air,
    Dirt,
    Grass,
}

struct Chunk {
    blocks: [[Blocks; 16]; 16],
}

impl Chunk {
    fn draw(&self, x: usize, y: usize, player: &Player, game: &Game) {
        //draw the chunk relative to the player

        //calculate the chunk in which the player is
        let player_chunk_x = (player.x / (game.block_size * 16.0)).floor() as i32;
        let player_chunk_y = (player.y / (game.block_size * 16.0)).floor() as i32;

        if (player_chunk_x - x as i32).abs() > 3 || (player_chunk_y - y as i32).abs() > 3 {
            return;
        }

        draw_rectangle_lines((x as f32 * 16.0) * game.block_size - player.x + game.screen_width / 2.0, (y as f32 * 16.0) * game.block_size - player.y + game.screen_height / 2.0, 16.0 * game.block_size, 16.0 * game.block_size, 2.0, GRAY);
        for i in 0..16 {
            for j in 0..16 {
                let block_x = (x as f32 * 16.0 + i as f32) * game.block_size - player.x + game.screen_width / 2.0;
                let block_y = (y as f32 * 16.0 + j as f32) * game.block_size - player.y + game.screen_height / 2.0;
                match self.blocks[i][j] {
                    Blocks::Air => {},
                    Blocks::Dirt => {
                        //square with outlines
                        draw_rectangle(block_x, block_y, game.block_size, game.block_size, BROWN);
                        //draw_rectangle_lines(block_x, block_y, game.block_size, game.block_size, 1.0, WHITE);
                    },
                    Blocks::Grass => {
                        draw_rectangle(block_x, block_y, game.block_size, game.block_size, GREEN);
                        //draw_rectangle_lines(block_x, block_y, game.block_size, game.block_size, 1.0, WHITE);
                    },
                }
            }
        }
    }
}

struct Game {
    screen_width: f32,
    screen_height: f32,
    block_size: f32,
}

impl Game {
    fn new(screen_width: f32, screen_height: f32, block_size: f32) -> Self {
        Self {
            screen_width,
            screen_height,
            block_size,
        }
    }
}

impl text {
    fn new(text: String, x: f32, y: f32) -> Self {
        Self {
            text,
            x,
            y,
            font_size: 30.0,
            color: WHITE,
        }
    }

    fn draw(&self) {
        let text_dimensions = measure_text(&self.text, None, self.font_size as u16, 1.0);
        let text_x = self.x - text_dimensions.width / 2.0;
        let text_y = self.y + text_dimensions.height / 2.0;
        draw_text(&self.text, text_x, text_y, self.font_size, self.color);
    }

    fn set_pos(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
}

fn generate_chunks() -> Vec<Vec<Chunk>> {
    let perlin = Perlin::new(534324);
    let mut chunks: Vec<Vec<Chunk>> = Vec::new();

    // Configuration for "Organic" feel
    let octaves = 4;
    let persistence = 0.5; // How much each octave contributes (smoothness)
    let lacunarity = 2.0;  // Gap between octave frequencies (detail)
    let scale = 0.02;      // General "stretch" of the terrain

    for i in 0..WORLD_SIZE {
        let mut row: Vec<Chunk> = Vec::new();
        for j in 0..WORLD_SIZE {
            let mut blocks = [[Blocks::Air; 16]; 16];
            
            for x in 0..16 {
                // 1. Calculate Global X Coordinate
                let global_x = (i * 16 + x) as f64;

                // 2. Multi-layered (Fractal) Perlin Noise
                let mut amplitude = 1.0;
                let mut frequency = scale;
                let mut noise_height = 0.0;
                let mut max_value = 0.0;

                for _ in 0..octaves {
                    let p = perlin.get([global_x * frequency, 0.0]);
                    noise_height += p * amplitude;
                    max_value += amplitude;
                    amplitude *= persistence;
                    frequency *= lacunarity;
                }

                // Normalize and scale the result
                // (noise_height / max_value) gives us a range from -1 to 1
                let normalized_noise = (noise_height / max_value + 1.0) / 2.0;
                
                // 3. Define a "Sea Level" and varying height
                // The terrain will vary around the middle of the world height
                let base_height = (WORLD_SIZE as f64 * 16.0) * 0.4; // 40% height is ground
                let variation = 40.0; // Max height of hills in blocks
                let limit = base_height + (normalized_noise * variation);

                for y in 0..16 {
                    let global_y = (j * 16 + y) as f64;
                    
                    // If the current block's Y is below the calculated limit, it's ground
                    if global_y > limit + 1.0 {
                        blocks[x][y] = Blocks::Dirt;
                    }
                    else if global_y > limit {
                        // Add some randomness for a more natural look
                        blocks[x][y] = Blocks::Grass;
                    }
                }
            }
            row.push(Chunk { blocks });
        }
        chunks.push(row);
    }
    chunks
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Hello, world!".to_string(),
        //fullscreen: true,
        ..Default::default()
    }
}

fn handle_input(player: &mut Player, chunks: &Vec<Vec<Chunk>>, game: &Game) {
    if is_key_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
    if is_key_down(KeyCode::W) {
        player.movee(0.0, -1.0, chunks, game);
    }
    if is_key_down(KeyCode::S) {
        player.movee(0.0, 1.0, chunks, game);
    }
    if is_key_down(KeyCode::A) {
        player.movee(-1.0, 0.0, chunks, game);
    }
    if is_key_down(KeyCode::D) {
        player.movee(1.0, 0.0, chunks, game);
    }
    if is_key_down(KeyCode::Space) {
        player.jump();
    }
    player.update(chunks, game);
}

fn draw_fps_counter() {
    let fps = get_fps();
    let color = if fps >= 60 { GREEN } else if fps >= 30 { YELLOW } else { RED };
    
    // Draw a small background for readability
    draw_rectangle(10.0, 10.0, 90.0, 30.0, Color::new(0.0, 0.0, 0.0, 0.5));
    draw_text(&format!("FPS: {}", fps), 20.0, 32.0, 30.0, color);
}

fn handle_drawing(player: &Player, chunks: &Vec<Vec<Chunk>>, game: &Game) {
    clear_background(BLACK);
    for i in 0..chunks.len() {
        for j in 0..chunks[i].len() {
            chunks[i][j].draw(i, j, player, game);
        }
    }
    player.draw(&game);
    draw_fps_counter();
}

fn handle_frame_count(frame_count: usize, game: &mut Game) {
    if frame_count % 10 == 0 {
        game.screen_width = screen_width();
        game.screen_height = screen_height();
    }
}

fn handle_mouse_input(player: &mut Player, chunks: &mut Vec<Vec<Chunk>>, game: &Game) {
    if is_mouse_button_down(MouseButton::Left) || is_mouse_button_down(MouseButton::Right) {
        //get mouse position in world coordinates
        let mouse_x = mouse_position().0 + player.x - game.screen_width / 2.0;
        let mouse_y = mouse_position().1 + player.y - game.screen_height / 2.0;

        //convert to block coordinates
        let block_x = (mouse_x / game.block_size).floor() as i32;
        let block_y = (mouse_y / game.block_size).floor() as i32;

        //determine which chunk these coordinates belong to
        let chunk_x = block_x / 16;
        let chunk_y = block_y / 16;

        //determine the local block index inside that chunk (0 to 15)
        let local_x = (block_x % 16).abs() as usize;
        let local_y = (block_y % 16).abs() as usize;

        if is_mouse_button_down(MouseButton::Left) {
            if chunk_x >= 0 && chunk_y >= 0 && (chunk_x as usize) < chunks.len() && (chunk_y as usize) < chunks[chunk_x as usize].len() {
                chunks[chunk_x as usize][chunk_y as usize].blocks[local_x][local_y] = Blocks::Air;
            }
        }
        else if is_mouse_button_down(MouseButton::Right) {
            if chunk_x >= 0 && chunk_y >= 0 && (chunk_x as usize) < chunks.len() && (chunk_y as usize) < chunks[chunk_x as usize].len() {
                chunks[chunk_x as usize][chunk_y as usize].blocks[local_x][local_y] = Blocks::Dirt;
            }
        }
        
    }
}

fn handle_resizing(game: &mut Game, player: &mut Player, mob: &mut Mob) {
    let (wheel_x, wheel_y) = mouse_wheel();
    if wheel_y != 0.0 {
        let mut new_block_size = game.block_size + wheel_y * 2.0;
        if new_block_size < 16.0 {
            new_block_size = 16.0;
        }
        if new_block_size > 64.0 {
            new_block_size = 64.0;
        }
        player.resize(new_block_size);
        mob.resize(new_block_size);
        game.block_size = new_block_size;
    }
}

//fullscreen
#[macroquad::main(window_conf)]
async fn main() {
    let width = screen_width();
    let height = screen_height();
    let mut text = text::new("Hello, world!".to_string(), width / 2.0, height / 2.0);
    let mut game = Game::new(width, height, 32.0);
    let mut chunks = generate_chunks();
    chunks[1][1].blocks[0][0] = Blocks::Dirt;
    let mut player = Player::new(&game, &chunks);
    let mut frame_count: usize = 0;
    let mut mob = Mob::new(player.x, player.y, &game);
    loop {

        handle_resizing(&mut game, &mut player, &mut mob);
        mob.update(&chunks, &game);
        handle_input(&mut player, &chunks, &game);
        handle_drawing(&player, &chunks, &game);
        mob.draw(&player, &game);
        handle_frame_count(frame_count, &mut game);
        handle_mouse_input(&mut player, &mut chunks, &game);
        frame_count += 1;

        //println!("Player chunk: {:?}", player.get_chunk(&chunks));

        next_frame().await;
    }
}
