use super::super::Action;
use inputbot;

pub fn read_keyboard(tx: std::sync::mpsc::Sender<Action>) {
    println!("controlling VAC from keyboard is not implemented, yet");

    inputbot::KeybdKey::SpaceKey.bind(move || {
        //tx.send(Action::TogglePlayPause).unwrap();
        println!("toggled")
    });

    println!("start handling input events");
    inputbot::handle_input_events();
}
