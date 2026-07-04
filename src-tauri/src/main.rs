// Windows release mode disables console window pop-ups
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    hoardster_lib::run()
}
