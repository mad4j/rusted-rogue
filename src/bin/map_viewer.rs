/// Map Viewer Binary
/// Generates and displays dungeon maps with visual output
/// 
/// Usage: cargo run --bin map_viewer [seed] [levels]
///   seed:   RNG seed (default: 42)
///   levels: Number of levels to generate (default: 1)

use std::env;
use rusted_rogue::core_types::{TileFlags, DCOLS, DROWS};
use rusted_rogue::rng::GameRng;
use rusted_rogue::world_gen::generate_level_with_depth;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let seed = args.get(1)
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(42);
    
    let num_levels = args.get(2)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1);
    
    println!("╔════════════════════════════════════════════════════════════════════════════════╗");
    println!("║                        RUSTED ROGUE - MAP VIEWER                             ║");
    println!("╚════════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Configuration:");
    println!("  Seed: {}", seed);
    println!("  Levels to generate: {}", num_levels);
    println!("  Grid dimensions: {}x{}", DROWS, DCOLS);
    println!();
    
    for level_num in 1..=num_levels {
        let mut rng = GameRng::new(seed + (level_num as i32 - 1) * 1000);
        let generated = generate_level_with_depth(&mut rng, level_num as i16);
        
        println!("┌─ Level {} ─────────────────────────────────────────────────────────────────────┐", level_num);
        println!();
        
        // Print rooms info
        println!("Rooms: {}", generated.rooms.len());
        for (i, room) in generated.rooms.iter().enumerate() {
            println!("  Room {}: top={}, bottom={}, left={}, right={}", 
                i, room.top_row, room.bottom_row, room.left_col, room.right_col);
        }
        println!();
        
        // Print map
        println!("Map:");
        print_map(&generated.grid);
        
        println!();
        println!("Legend:");
        println!("  ─ = Horizontal wall");
        println!("  │ = Vertical wall");
        println!("  · = Floor");
        println!("  # = Tunnel");
        println!("  + = Door");
        println!("  ^ = Stairs");
        println!("  ~ = Trap");
        println!("  . = Empty space");
        println!();
        
        if level_num < num_levels {
            println!("└────────────────────────────────────────────────────────────────────────────────┘");
            println!();
        }
    }
    
    println!("└────────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("Map dimensions: {} rows × {} columns", DROWS, DCOLS);
}

fn print_map(grid: &rusted_rogue::world_gen::DungeonGrid) {
    let (rows, cols) = grid.dimensions();
    
    // Print top border
    print!("╭");
    for _ in 0..cols {
        print!("─");
    }
    println!("╮");
    
    // Print each row
    for row in 0..rows {
        print!("│");
        for col in 0..cols {
            let tile = grid.get(row as i16, col as i16).unwrap_or(TileFlags::NOTHING);
            let ch = tile_to_char(tile);
            print!("{}", ch);
        }
        println!("│");
    }
    
    // Print bottom border
    print!("╰");
    for _ in 0..cols {
        print!("─");
    }
    println!("╯");
}

fn tile_to_char(tile: TileFlags) -> char {
    if tile == TileFlags::NOTHING {
        '.'
    } else if tile.contains(TileFlags::HORWALL) {
        '─'
    } else if tile.contains(TileFlags::VERTWALL) {
        '│'
    } else if tile.contains(TileFlags::FLOOR) {
        '·'
    } else if tile.contains(TileFlags::TUNNEL) {
        '#'
    } else if tile.contains(TileFlags::DOOR) {
        '+'
    } else if tile.contains(TileFlags::STAIRS) {
        '^'
    } else if tile.contains(TileFlags::TRAP) {
        '~'
    } else {
        '?'
    }
}
