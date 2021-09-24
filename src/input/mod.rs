pub mod controller;

#[cfg(target_os = "windows")]
pub mod keyboard_windows;

#[cfg(target_os = "linux")]
pub mod keyboard_linux;

use super::Action;
use std::sync::mpsc::channel;
use std::thread;

pub fn spawn_input_threads_with_sender(tx_orig: &std::sync::mpsc::Sender<Action>) {
    let tx_keyboard = tx_orig.clone();
    let (_keyboard_control_tx, _keyboard_control_rx) = channel::<()>();
    let _keyboard_join_handle = thread::spawn(|| {
        #[cfg(target_os = "windows")]
        keyboard_windows::read_keyboard(tx_keyboard);

        #[cfg(target_os = "linux")]
        keyboard_linux::read_keyboard(tx_keyboard);
    });

    let (_controller_control_tx, _controller_control_rx) = channel::<()>();
    let tx_controller = tx_orig.clone();
    let _controller_join_handle = thread::spawn(|| {
        controller::read_controller(tx_controller);
    });
}
