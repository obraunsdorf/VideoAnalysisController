pub mod controller;
pub mod keyboard;

use std::thread;
use super::Action as Action;

pub fn spawn_input_threads_with_sender(tx_orig: &std::sync::mpsc::Sender<Action>) {
    let tx_keyboard = tx_orig.clone();
    thread::spawn(|| {
        keyboard::read_keyboard2(tx_keyboard);
    });

    let tx_controller = tx_orig.clone();
    thread::spawn(|| {
        controller::read_controller(tx_controller);
    });


}