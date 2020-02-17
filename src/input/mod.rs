pub mod controller;
pub mod keyboard;

use std::thread;
use super::Action as Action;
use std::sync::mpsc::channel;

pub enum ControlMsg {
    Stop
}

pub fn spawn_input_threads_with_sender(tx_orig: &std::sync::mpsc::Sender<Action>) {
    let tx_keyboard = tx_orig.clone();
    let (keyboard_control_tx, keyboard_control_rx) = channel::<ControlMsg>();
    thread::spawn(|| {
        keyboard::read_keyboard(tx_keyboard);
    });

    let (controller_control_tx, controller_control_rx) = channel::<ControlMsg>();
    let tx_controller = tx_orig.clone();
    thread::spawn(|| {
        controller::read_controller(tx_controller);
    });


}