mod actors;
mod core_types;
mod game_loop;
mod inventory_items;
mod persistence;
mod platform;
mod rng;
mod ui_terminal;
mod world_gen;

fn main() {
    if let Some(code) = run_script_mode_if_requested() {
        std::process::exit(code);
    }

    platform::init_platform();
    let _save_fn: fn() = persistence::save;
    let game = game_loop::run();

    ui_terminal::run(game);
}

fn run_script_mode_if_requested() -> Option<i32> {
    let script_path = std::env::var("RUSTED_ROGUE_SCRIPT_FILE").ok()?;

    let script = match std::fs::read_to_string(&script_path) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("script mode read error: {error}");
            return Some(2);
        }
    };

    let seed = std::env::var("RUSTED_ROGUE_SEED")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(12345);

    let mut game = game_loop::GameLoop::new(seed);
    let outcome = game.run_script(&script);

    println!(
        "scenario_summary level={} turns={} hp={} max_hp={} defeated={} quit={} outcome={:?}",
        game.state().level,
        game.state().turns,
        game.state().player_hit_points,
        game.state().player_max_hit_points,
        game.state().monsters_defeated,
        game.state().quit_requested,
        outcome,
    );

    Some(0)
}
