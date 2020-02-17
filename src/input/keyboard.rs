//use crossterm::{RawScreen, AsyncReader, InputEvent, KeyEvent, SyncReader};

use super::super::Action as Action;
use super::super::ClipOf_O_D as ClipOf_O_D;

use multiinput::{KeyId, RawInputManager, DeviceType, RawEvent, State};
use std::time::Duration;
use std::sync::mpsc::SendError;

fn get_action(key: &KeyId) -> Option<Action> {
    match key {
        KeyId::Space => Some(Action::TogglePlayPause),
        KeyId::Left => Some(Action::Rewind(0.7)),
        KeyId::Right => Some(Action::Forward(0.7)),
        KeyId::Up => Some(Action::IncreaseSpeed),
        KeyId::Down => Some(Action::DecreaseSpeed),
        KeyId::T => Some(Action::StartLoop),
        KeyId::Z => Some(Action::EndLoop),
//'h' => Some(Action::CheckLoopEnd(f32)),
        KeyId::O => Some(Action::CutCurrentLoop(Some(ClipOf_O_D::Offense))),
        KeyId::D => Some(Action::CutCurrentLoop(Some(ClipOf_O_D::Defense))),
        KeyId::I => Some(Action::NextMedia),
        KeyId::K => Some(Action::PreviousMedia),
        KeyId::M => Some(Action::RestartMedia),
        KeyId::W => Some(Action::NextClip),
        KeyId::S => Some(Action::PreviousClip),
        KeyId::Y => Some(Action::RestartClip),
//        KeyId::Q => Some(Action::Stop),
        KeyId::Escape => Some(Action::Exit),
        _ => None
    }
}

pub fn read_keyboard(tx: std::sync::mpsc::Sender<Action>) {
    println!("starting to read");

    let mut manager = RawInputManager::new().unwrap();
    //manager.register_devices(DeviceType::Joysticks(XInputInclude::True));
    manager.register_devices(DeviceType::Keyboards);
    //manager.register_devices(DeviceType::Mice);
    loop {
        if let Some(event) = manager.get_event() {
            let action_option = match &event {
                RawEvent::KeyboardEvent(_devId, key_id, State::Released) => {
                    get_action(key_id)
                },
                _ => None,
            };
            println!("{:?}", event);

            if let Some(action) = action_option {
                if let Err(_) = tx.send(action) {
                    println!("other end seems to be terminated. terminating too");
                    break;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
