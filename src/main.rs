use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup)
        .add_system(tick)
        .run();
}

fn startup() {
    println!("Hello world!");
}

fn tick() {
    println!("Tick");
}