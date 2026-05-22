use macroquad::prelude::*;
use std::cell::RefCell;
use ::rand::{rng, RngExt};
use noise::{NoiseFn, Perlin};

const WORLD_SIZE: usize = 300;
const TOTAL_BLOCKS: usize = WORLD_SIZE * 16;
const DRAW_DISTANCE_X: usize = 5;
const DRAW_DISTANCE_Y: usize = 5;

struct text {
    text: String,
    x: f32,
    y: f32,
    font_size: f32,
    color: Color,
}

struct Item {
    name: String,
    item_id: usize,
    quantity: usize,
}

impl Item {
    fn new(name: String, item_id: usize, quantity: usize) -> Self {
        Self {
            name,
            item_id,
            quantity,
        }
    }
}

struct Inventory {
    items: Vec<Item>,
    active_slot: usize,
}

impl Inventory {
    fn new() -> Self {
        let mut items = Vec::new();
        for _i in 0..10 {
            items.push(Item::new("".to_string(), 0, 0));
        }
        Self {
            items,
            active_slot: 0,
        }
    }

    fn add_item(&mut self, item: Item) {
        for inv_item in &mut self.items {
            if inv_item.item_id == item.item_id && inv_item.item_id != 0 {
                inv_item.quantity += item.quantity;
                return;
            }
        }
        for inv_item in &mut self.items {
            if inv_item.item_id == 0 {
                *inv_item = item;
                return;
            }
        }
    }

    fn change_active_slot(&mut self, delta: isize) {
        let delta = - delta;
        let new_slot = (self.active_slot as isize + delta).rem_euclid(self.items.len() as isize) as usize;
        self.active_slot = new_slot;
    }
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
    jump_cooldown: usize,
    health: i32,
    isAlive: bool,
}

impl Mob {
    fn new(x: f32, y: f32, game: &Game) -> Self {
        Self {
            x,
            y,
            velocity: vec![0.0, 0.0],
            isGrounded: false,
            size_x: 1.0,
            size_y: 1.0,
            speed: 0.1,
            hitbox: Vec::new(),
            jump_cooldown: 0,
            health: 10,
            isAlive: true,
        }
    }

    fn resize(&mut self, _new_block_size: f32) {}

    fn calculate_hitbox(&mut self, _game: &Game) {
        self.hitbox = vec![
            vec![self.x, self.y],
            vec![self.x + self.size_x, self.y],
            vec![self.x + self.size_x, self.y + self.size_y],
            vec![self.x, self.y + self.size_y],
        ];
    }

    fn does_collide(&self, chunks: &Vec<Vec<Chunk>>, _game: &Game) -> bool {
        for hitbox_point in &self.hitbox {
            let global_block_x = hitbox_point[0].floor() as i32;
            let global_block_y = hitbox_point[1].floor() as i32;
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

    fn movee(&mut self, player: &Player, _chunks: &Vec<Vec<Chunk>>, _game: &Game) {
        let dx = player.x - self.x;
        let dx = dx.clamp(-1.0, 1.0);
        self.velocity[0] = dx * self.speed;
    }

    fn jump(&mut self) {
        if self.isGrounded {
            self.velocity[1] = -0.35; 
            self.isGrounded = false;
        }
    }

    fn update(&mut self, player: &Player, chunks: &Vec<Vec<Chunk>>, game: &Game) {
        self.velocity[1] += 0.01;
        self.velocity[1] *= 0.99;
        
        if self.velocity[1] > 1.0 {
            self.velocity[1] = 1.0; 
        }

        if !self.isGrounded {
            self.x += self.velocity[0];
            self.calculate_hitbox(game);
            if self.does_collide(chunks, game) {
                self.x -= self.velocity[0];
                self.calculate_hitbox(game);
            }
        }
        else {
            self.velocity[0] = 0.0;
            self.jump_cooldown += 1;
            if self.jump_cooldown > 144 {
                self.movee(player, chunks, game);
                self.jump();
                self.jump_cooldown = 0;
            }
        }

        self.y += self.velocity[1];
        self.calculate_hitbox(game);
        if self.does_collide(chunks, game) {
            self.y -= self.velocity[1];
            if self.velocity[1] > 0.0 {
                self.y = self.y.floor() + 0.99;
                self.isGrounded = true;
            }
            self.velocity[1] = 0.0;
            self.calculate_hitbox(game);
        } else {
            self.isGrounded = false;
        }
    }

    fn draw(&self, player: &Player, game: &Game) {
        let screen_x = (self.x - player.x) * game.block_size + game.screen_width / 2.0;
        let screen_y = (self.y - player.y) * game.block_size + game.screen_height / 2.0;
        draw_rectangle(screen_x, screen_y, self.size_x * game.block_size, self.size_y * game.block_size, GREEN);
        //draw health bar
        let health_ratio = self.health as f32 / 10.0;
        draw_rectangle(screen_x, screen_y - 5.0, self.size_x * game.block_size * health_ratio, 3.0, RED);
    }

    fn take_damage(&mut self, amount: i32) {
        self.health -= amount;
        if self.health <= 0 {
            self.isAlive = false;
        }
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
    inventory: Inventory,
}

impl Player {
    fn new(game: &Game, chunks: &Vec<Vec<Chunk>>) -> Self {
        let middle = (TOTAL_BLOCKS as f32) / 2.0;
        let chunk_num = (middle / 16.0).floor() as usize;
        let block_index = (middle % 16.0).floor() as usize;
        let mut highest_block = 0.0;
        
        'find_ground: for i in 0..chunks.len() {
            for j in 0..16 {
                if chunks[chunk_num][i].blocks[block_index][j] != Blocks::Air {
                    highest_block = (i * 16 + j) as f32;
                    break 'find_ground;
                }
            }
        }

        let mut p = Self {
            x: middle,
            y: highest_block - 2.0,
            velocity: vec![0.0, 0.0],
            isGrounded: false,
            size_x: 1.0,
            size_y: 2.0,
            hitbox: Vec::new(),
            speed: 0.01,
            inventory: Inventory::new(),
        };
        p.calculate_hitbox(game);
        p
    }

    fn resize(&mut self, _new_block_size: f32) {}

    fn draw(&self, game: &Game) {
        draw_rectangle(
            game.screen_width / 2.0, 
            game.screen_height / 2.0, 
            game.block_size * 0.5, 
            game.block_size * 1.5, 
            RED
        );
    }

    fn movee(&mut self, dx: f32, dy: f32, _chunks: &Vec<Vec<Chunk>>, _game: &Game) {
        self.velocity[0] += dx * self.speed;
        self.velocity[1] += dy * self.speed;
    }

    fn update(&mut self, chunks: &Vec<Vec<Chunk>>, game: &Game) {
        self.velocity[1] += 0.01;
        self.velocity[0] *= 0.90; 
        self.velocity[1] *= 0.99;
        
        if self.velocity[1] > 1.0 {
            self.velocity[1] = 1.0; 
        }

        self.x += self.velocity[0];
        self.calculate_hitbox(game);
        if self.does_collide(chunks, game) {
            self.x -= self.velocity[0];
            self.velocity[0] = 0.0;
            self.calculate_hitbox(game);
        }

        self.y += self.velocity[1];
        self.calculate_hitbox(game);
        if self.does_collide(chunks, game) {
            self.isGrounded = self.velocity[1] > 0.0;
            self.y -= self.velocity[1];
            self.velocity[1] = 0.0;
            self.calculate_hitbox(game);
        } else {
            self.isGrounded = false;
        }
    }

    fn jump(&mut self) {
        if self.isGrounded {
            self.velocity[1] = -0.35; 
            self.isGrounded = false;
        }
    }

    fn get_chunk(&self, chunks: &Vec<Vec<Chunk>>, _game: &Game) -> (i32, i32) {
        let chunk_x = (self.x / 16.0).floor() as i32;
        let chunk_y = (self.y / 16.0).floor() as i32;
        if chunk_x < 0 || chunk_y < 0 || chunk_x as usize >= chunks.len() || chunk_y as usize >= chunks[chunk_x as usize].len() {
            return (-1, -1);
        }
        (chunk_x, chunk_y)
    }

    fn does_collide(&self, chunks: &Vec<Vec<Chunk>>, _game: &Game) -> bool {
        for hitbox_point in &self.hitbox {
            if hitbox_point[0] < 0.0 || hitbox_point[1] < 0.0 || hitbox_point[0] >= TOTAL_BLOCKS as f32 || hitbox_point[1] >= TOTAL_BLOCKS as f32 {
                return true;
            }

            let global_block_x = hitbox_point[0].floor() as i32;
            let global_block_y = hitbox_point[1].floor() as i32;

            let chunk_x = global_block_x / 16;
            let chunk_y = global_block_y / 16;

            if chunk_x >= 0 && chunk_y >= 0 && (chunk_x as usize) < chunks.len() && (chunk_y as usize) < chunks[chunk_x as usize].len() {
                let local_x = (global_block_x % 16).abs() as usize;
                let local_y = (global_block_y % 16).abs() as usize;
                if chunks[chunk_x as usize][chunk_y as usize].blocks[local_x][local_y] != Blocks::Air {
                    return true;
                }
            }
        }
        false
    }

    fn calculate_hitbox(&mut self, _game: &Game) {
        self.hitbox = vec![
            vec![self.x, self.y],
            vec![self.x + 0.5, self.y],
            vec![self.x + 0.5, self.y + 1.0],
            vec![self.x + 0.5, self.y + 1.5],
            vec![self.x, self.y + 1.5],
            vec![self.x, self.y + 1.0],
        ];
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Blocks {
    Air,
    Dirt,
    Grass,
}

impl Blocks {
    // Defines how long a block takes to break in seconds
    fn hardness(&self) -> f32 {
        match self {
            Blocks::Air => 0.0,
            Blocks::Dirt => 0.4,  // 0.4 seconds
            Blocks::Grass => 0.6, // 0.6 seconds
        }
    }
}

struct Chunk {
    blocks: [[Blocks; 16]; 16],
}

impl Chunk {
    fn draw(&self, x: usize, y: usize, player: &Player, game: &Game) {
        let player_chunk_x = (player.x / 16.0).floor() as i32;
        let player_chunk_y = (player.y / 16.0).floor() as i32;

        if (player_chunk_x - x as i32).abs() > 3 || (player_chunk_y - y as i32).abs() > 3 {
            return;
        }

        let chunk_screen_x = (x as f32 * 16.0 - player.x) * game.block_size + game.screen_width / 2.0;
        let chunk_screen_y = (y as f32 * 16.0 - player.y) * game.block_size + game.screen_height / 2.0;

        draw_rectangle_lines(chunk_screen_x, chunk_screen_y, 16.0 * game.block_size, 16.0 * game.block_size, 2.0, GRAY);
        
        for i in 0..16 {
            for j in 0..16 {
                let block_x = (x as f32 * 16.0 + i as f32 - player.x) * game.block_size + game.screen_width / 2.0;
                let block_y = (y as f32 * 16.0 + j as f32 - player.y) * game.block_size + game.screen_height / 2.0;
                match self.blocks[i][j] {
                    Blocks::Air => {},
                    Blocks::Dirt => draw_rectangle(block_x, block_y, game.block_size, game.block_size, BROWN),
                    Blocks::Grass => draw_rectangle(block_x, block_y, game.block_size, game.block_size, GREEN),
                }
            }
        }
    }
}

// Struct to store active block-breaking tracking metrics
struct BreakingBlock {
    pos: (i32, i32),
    progress: f32, // Time tracking (in seconds)
}

struct Game {
    screen_width: f32,
    screen_height: f32,
    block_size: f32,
    ctrl_pressed: bool,
    breaking_block: Option<BreakingBlock>, // Active break instance tracking
}

impl Game {
    fn new(screen_width: f32, screen_height: f32, block_size: f32) -> Self {
        Self {
            screen_width,
            screen_height,
            block_size,
            ctrl_pressed: false,
            breaking_block: None,
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
    let octaves = 4;
    let persistence = 0.5;
    let lacunarity = 2.0;
    let scale = 0.02;

    for i in 0..WORLD_SIZE {
        let mut row: Vec<Chunk> = Vec::new();
        for j in 0..WORLD_SIZE {
            let mut blocks = [[Blocks::Air; 16]; 16];
            for x in 0..16 {
                let global_x = (i * 16 + x) as f64;
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

                let normalized_noise = (noise_height / max_value + 1.0) / 2.0;
                let base_height = (WORLD_SIZE as f64 * 16.0) * 0.4;
                let variation = 40.0;
                let limit = base_height + (normalized_noise * variation);

                for y in 0..16 {
                    let global_y = (j * 16 + y) as f64;
                    if global_y > limit + 1.0 {
                        blocks[x][y] = Blocks::Dirt;
                    } else if global_y > limit {
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
        window_title: "Macroquad World".to_string(),
        ..Default::default()
    }
}

fn handle_input(player: &mut Player, chunks: &Vec<Vec<Chunk>>, game: &mut Game) {
    if is_key_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
    if is_key_down(KeyCode::W) { player.movee(0.0, -1.0, chunks, game); }
    if is_key_down(KeyCode::S) { player.movee(0.0, 1.0, chunks, game); }
    if is_key_down(KeyCode::A) { player.movee(-1.0, 0.0, chunks, game); }
    if is_key_down(KeyCode::D) { player.movee(1.0, 0.0, chunks, game); }
    if is_key_down(KeyCode::Space) { player.jump(); }
    if is_key_down(KeyCode::LeftControl) { game.ctrl_pressed = true; }
    else { game.ctrl_pressed = false; }
    player.update(chunks, game);
}

fn draw_fps_counter() {
    let fps = get_fps();
    let color = if fps >= 60 { GREEN } else if fps >= 30 { YELLOW } else { RED };
    draw_rectangle(10.0, 10.0, 90.0, 30.0, Color::new(0.0, 0.0, 0.0, 0.5));
    draw_text(&format!("FPS: {}", fps), 20.0, 32.0, 30.0, color);
}

fn draw_inventory(player: &Player) {
    let inventory_width = 55.0 * 10.0 + 5.0;
    let item_size = (inventory_width - 5.0) / 10.0 - 5.0;
    let inventory_height = item_size + 10.0;
    let x = (screen_width() - inventory_width) / 2.0;
    let y = screen_height() - inventory_height - 10.0;
    draw_rectangle(x, y, inventory_width, inventory_height, Color::new(0.0, 0.0, 0.0, 0.5));
    for (i, item) in player.inventory.items.iter().enumerate() {
        let item_x = 5.0 + i as f32 * (inventory_width - 5.0) / 10.0 + x;
        let item_y = y + 5.0;
        if i == player.inventory.active_slot {
            draw_rectangle(item_x, item_y, item_size, inventory_height - 10.0, YELLOW);
        }
        else {
            draw_rectangle(item_x, item_y, item_size, inventory_height - 10.0, GRAY);
        }
        if item.item_id != 0 {
            draw_text(&item.name, item_x + 4.0, item_y + 12.0, 12.0, WHITE);
        }
        if item.quantity > 0 {
            draw_text(&format!("{}", item.quantity), item_x + 4.0, item_y + item_size - 5.0, 12.0, WHITE);
        }
    }
}

// Draws cracking effects onto the targeted block depending on breaking progress ratios
fn draw_breaking_progress(player: &Player, game: &Game, chunks: &Vec<Vec<Chunk>>) {
    if let Some(ref breaking) = game.breaking_block {
        let (bx, by) = breaking.pos;
        let chunk_x = bx / 16;
        let chunk_y = by / 16;
        let local_x = (bx % 16).abs() as usize;
        let local_y = (by % 16).abs() as usize;

        if chunk_x >= 0 && chunk_y >= 0 && (chunk_x as usize) < chunks.len() && (chunk_y as usize) < chunks[chunk_x as usize].len() {
            let block = chunks[chunk_x as usize][chunk_y as usize].blocks[local_x][local_y];
            if block != Blocks::Air {
                let screen_x = (bx as f32 - player.x) * game.block_size + game.screen_width / 2.0;
                let screen_y = (by as f32 - player.y) * game.block_size + game.screen_height / 2.0;
                
                let ratio = (breaking.progress / block.hardness()).clamp(0.0, 1.0);
                
                // Draws a black overlay layer that gets opaque as it nears completion
                draw_rectangle(
                    screen_x, 
                    screen_y, 
                    game.block_size, 
                    game.block_size, 
                    Color::new(0.0, 0.0, 0.0, ratio * 0.6)
                );
            }
        }
    }
}

fn handle_drawing(player: &Player, chunks: &Vec<Vec<Chunk>>, game: &Game, mobs: &Vec<Mob>) {
    clear_background(BLUE);
    for i in 0..chunks.len() {
        for j in 0..chunks[i].len() {
            chunks[i][j].draw(i, j, player, game);
        }
    }
    
    // Render the visual overlay showing the block mining progress
    draw_breaking_progress(player, game, chunks);

    for i in 0..mobs.len() {
        mobs[i].draw(&player, &game);
    }
    player.draw(&game);
    draw_fps_counter();
    draw_inventory(player);
}

fn handle_frame_count(frame_count: usize, game: &mut Game) {
    if frame_count % 10 == 0 {
        game.screen_width = screen_width();
        game.screen_height = screen_height();
    }
}

fn handle_mouse_input(player: &mut Player, chunks: &mut Vec<Vec<Chunk>>, game: &mut Game, mobs: &mut Vec<Mob>) {
    // Check if player is holding the Pickaxe (item_id == 1)
    let active_item = &player.inventory.items[player.inventory.active_slot];
    let holding_pickaxe = active_item.item_id == 1;
    let holding_sword = active_item.item_id == 2;
    //calculate if the mouse if not too far from the player (within 5 blocks)
    let mouse_world_x = (mouse_position().0 - game.screen_width / 2.0) / game.block_size + player.x;
    let mouse_world_y = (mouse_position().1 - game.screen_height / 2.0) / game.block_size + player.y;
    let distance = ((mouse_world_x - player.x).powi(2) + (mouse_world_y - player.y).powi(2)).sqrt();
    if distance > 3.0 {
        return; // Too far to interact
    }

    if is_mouse_button_down(MouseButton::Left) && holding_pickaxe {
        let mouse_world_x = (mouse_position().0 - game.screen_width / 2.0) / game.block_size + player.x;
        let mouse_world_y = (mouse_position().1 - game.screen_height / 2.0) / game.block_size + player.y;

        let block_x = mouse_world_x.floor() as i32;
        let block_y = mouse_world_y.floor() as i32;

        let chunk_x = block_x / 16;
        let chunk_y = block_y / 16;
        let local_x = (block_x % 16).abs() as usize;
        let local_y = (block_y % 16).abs() as usize;

        if chunk_x >= 0 && chunk_y >= 0 && (chunk_x as usize) < chunks.len() && (chunk_y as usize) < chunks[chunk_x as usize].len() {
            let target_block = chunks[chunk_x as usize][chunk_y as usize].blocks[local_x][local_y];

            if target_block != Blocks::Air {
                if let Some(ref mut current) = game.breaking_block {
                    if current.pos == (block_x, block_y) {
                        // Progress added via delta time across frames
                        current.progress += get_frame_time();

                        if current.progress >= target_block.hardness() {
                            chunks[chunk_x as usize][chunk_y as usize].blocks[local_x][local_y] = Blocks::Air;
                            game.breaking_block = None;
                        }
                    } else {
                        // Mouse moved onto another block: Reset tracking
                        game.breaking_block = Some(BreakingBlock {
                            pos: (block_x, block_y),
                            progress: 0.0,
                        });
                    }
                } else {
                    // Start breaking track state instance
                    game.breaking_block = Some(BreakingBlock {
                        pos: (block_x, block_y),
                        progress: 0.0,
                    });
                }
                return; // Early return prevents clearing track data below
            }
        }
    }
    else if is_mouse_button_pressed(MouseButton::Left) && holding_sword {
        //mob hurting logic here
        //&mut mobs in not an iterator!!
        println!("HURT!!!");
        for mob in &mut mobs.iter_mut() {
            let mob_screen_x = (mob.x - player.x) * game.block_size + game.screen_width / 2.0;
            let mob_screen_y = (mob.y - player.y) * game.block_size + game.screen_height / 2.0;
            let (mouse_x, mouse_y) = mouse_position();
            if mouse_x >= mob_screen_x && mouse_x <= mob_screen_x + mob.size_x * game.block_size &&
                mouse_y >= mob_screen_y && mouse_y <= mob_screen_y + mob.size_y * game.block_size {
                mob.take_damage(3);
            }
        }
    }
    
    if is_mouse_button_down(MouseButton::Right) {
        let mouse_world_x = (mouse_position().0 - game.screen_width / 2.0) / game.block_size + player.x;
        let mouse_world_y = (mouse_position().1 - game.screen_height / 2.0) / game.block_size + player.y;
        let block_x = mouse_world_x.floor() as i32;
        let block_y = mouse_world_y.floor() as i32;
        let chunk_x = block_x / 16;
        let chunk_y = block_y / 16;
        let local_x = (block_x % 16).abs() as usize;
        let local_y = (block_y % 16).abs() as usize;

        if chunk_x >= 0 && chunk_y >= 0 && (chunk_x as usize) < chunks.len() && (chunk_y as usize) < chunks[chunk_x as usize].len() {
            chunks[chunk_x as usize][chunk_y as usize].blocks[local_x][local_y] = Blocks::Dirt;
        }
        return;
    }

    // Left click was lifted, or pickaxe wasn't equipped: Reset progress!
    game.breaking_block = None;
}

fn handle_resizing(game: &mut Game, player: &mut Player, mob: &mut Mob) {
    let (_, wheel_y) = mouse_wheel();
    if wheel_y != 0.0 && game.ctrl_pressed {
        let mut new_block_size = game.block_size + wheel_y * 2.0;
        new_block_size = new_block_size.clamp(1.0, 64.0);
        player.resize(new_block_size);
        mob.resize(new_block_size);
        game.block_size = new_block_size;
    }
    else if wheel_y != 0.0 {
        player.inventory.change_active_slot(wheel_y as isize);
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let width = screen_width();
    let height = screen_height();
    let mut game = Game::new(width, height, 32.0);
    let mut chunks = generate_chunks();
    let mut player = Player::new(&game, &chunks);
    player.inventory.add_item(Item::new("Sword".to_string(), 2, 1));
    player.inventory.add_item(Item::new("Pickaxe".to_string(), 1, 1));
    player.inventory.add_item(Item::new("Axe".to_string(), 3, 1));
    let mut mob = Mob::new(player.x + 5.0, player.y - 2.0, &game);
    let mut frame_count: usize = 0;
    let mut mobs: Vec<Mob> = Vec::new();
    let mob_count = 10;
    for i in 0..mob_count {
        mobs.push(Mob::new(player.x + 5.0 - i as f32 * 5.0, player.y - 2.0 - i as f32 * 5.0, &game));
    }

    loop {
        handle_resizing(&mut game, &mut player, &mut mob);
        for i in 0..mobs.len() {
            mobs[i].update(&player, &chunks, &game);
        }
        // In your update loop:
        for i in (0..mobs.len()).rev() {
            if !mobs[i].isAlive {
                mobs.swap_remove(i);
            }
        }
        handle_input(&mut player, &chunks, &mut game);
        handle_drawing(&player, &chunks, &game, &mobs);
        handle_frame_count(frame_count, &mut game);
        handle_mouse_input(&mut player, &mut chunks, &mut game, &mut mobs);
        frame_count += 1;

        next_frame().await;
    }
}