// Windows'ta release derlemesinde ek konsol penceresi açılmasını engeller.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    compression_lib::run()
}
