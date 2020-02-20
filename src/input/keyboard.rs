//use crossterm::{RawScreen, AsyncReader, InputEvent, KeyEvent, SyncReader};

use super::super::Action;
use super::super::ClipOf_O_D;

use multiinput::DeviceInfo::Keyboard;
use multiinput::{DeviceType, KeyId, RawEvent, RawInputManager, State};
use std::collections::HashMap;
use std::intrinsics::transmute;
use std::iter::Map;
use std::sync::mpsc::SendError;
use std::time::Duration;

fn translate_key_id(s: &str) -> Result<KeyId, (String)> {
    match s.to_lowercase().as_str() {
        "escape" => Ok(KeyId::Escape),
        "return" => Ok(KeyId::Return),
        "backspace" => Ok(KeyId::Backspace),
        "left" => Ok(KeyId::Left),
        "right" => Ok(KeyId::Right),
        "up" => Ok(KeyId::Up),
        "down" => Ok(KeyId::Down),
        "space" => Ok(KeyId::Space),
        "a" => Ok(KeyId::A),
        "b" => Ok(KeyId::B),
        "c" => Ok(KeyId::C),
        "d" => Ok(KeyId::D),
        "e" => Ok(KeyId::E),
        "f" => Ok(KeyId::F),
        "g" => Ok(KeyId::G),
        "h" => Ok(KeyId::H),
        "i" => Ok(KeyId::I),
        "j" => Ok(KeyId::J),
        "k" => Ok(KeyId::K),
        "l" => Ok(KeyId::L),
        "m" => Ok(KeyId::M),
        "n" => Ok(KeyId::N),
        "o" => Ok(KeyId::O),
        "p" => Ok(KeyId::P),
        "q" => Ok(KeyId::Q),
        "r" => Ok(KeyId::R),
        "s" => Ok(KeyId::S),
        "t" => Ok(KeyId::T),
        "u" => Ok(KeyId::U),
        "v" => Ok(KeyId::V),
        "w" => Ok(KeyId::W),
        "x" => Ok(KeyId::X),
        "y" => Ok(KeyId::Y),
        "z" => Ok(KeyId::Z),
        "f1" => Ok(KeyId::F1),
        "f2" => Ok(KeyId::F2),
        "f3" => Ok(KeyId::F3),
        "f4" => Ok(KeyId::F4),
        "f5" => Ok(KeyId::F5),
        "f6" => Ok(KeyId::F6),
        "f7" => Ok(KeyId::F7),
        "f8" => Ok(KeyId::F8),
        "f9" => Ok(KeyId::F9),
        "f10" => Ok(KeyId::F10),
        "f11" => Ok(KeyId::F11),
        "f12" => Ok(KeyId::F12),
        "zero" => Ok(KeyId::Zero),
        "one" => Ok(KeyId::One),
        "two" => Ok(KeyId::Two),
        "three" => Ok(KeyId::Three),
        "four" => Ok(KeyId::Four),
        "five" => Ok(KeyId::Five),
        "six" => Ok(KeyId::Six),
        "seven" => Ok(KeyId::Seven),
        "eight" => Ok(KeyId::Eight),
        "nine" => Ok(KeyId::Nine),
        /*"pause" => Ok(KeyId::Pause),
        "pageup" => Ok(KeyId::PageUp),
        "pagedown" => Ok(KeyId::PageDown),
        "printscreen" => Ok(KeyId::PrintScreen),
        "insert" => Ok(KeyId::Insert),
        "end" => Ok(KeyId::End),
        "home" => Ok(KeyId::Home),
        "delete" => Ok(KeyId::Delete),
        "add" => Ok(KeyId::Add),
        "subtract" => Ok(KeyId::Subtract),
        "multiply" => Ok(KeyId::Multiply),
        "separator" => Ok(KeyId::Separator),
        "decimal" => Ok(KeyId::Decimal),
        "divide" => Ok(KeyId::Divide),
        "backtick" => Ok(KeyId::BackTick),
        "backslash" => Ok(KeyId::BackSlash),
        "forwardslash" => Ok(KeyId::ForwardSlash),
        "plus" => Ok(KeyId::Plus),
        "minus" => Ok(KeyId::Minus),
        "fullstop" => Ok(KeyId::FullStop),
        "comma" => Ok(KeyId::Comma),
        "tab" => Ok(KeyId::Tab),
        "numlock" => Ok(KeyId::Numlock),
        "leftsquarebracket" => Ok(KeyId::LeftSquareBracket),
        "rightsquarebracket" => Ok(KeyId::RightSquareBracket),
        "semicolon" => Ok(KeyId::SemiColon),
        "apostrophe" => Ok(KeyId::Apostrophe),
        "hash" => Ok(KeyId::Hash),*/
        _ => Err(format!("no corresponding key id for '{}'", s)),
    }
}

fn key_map_from_config(config_file_path: &str) -> Result<HashMap<KeyId, Option<Action>>, String> {
    let config_tree = configuration::format::TOML::open(config_file_path).unwrap();

    let mut map = HashMap::new();
    //Action::TogglePlayPause.
    if let Some(x) = config_tree.get::<String>(Action::TogglePlayPause.into()) {
        map.insert(translate_key_id(x)?, Some(Action::TogglePlayPause));
    }

    if let Some(x) = config_tree.get::<String>(Action::Rewind(0.7).into()) {
        map.insert(translate_key_id(x)?, Some(Action::Rewind(0.7)));
    }

    if let Some(x) = config_tree.get::<String>(Action::Forward(0.7).into()) {
        map.insert(translate_key_id(x)?, Some(Action::Forward(0.7)));
    }

    if let Some(x) = config_tree.get::<String>(Action::IncreaseSpeed.into()) {
        map.insert(translate_key_id(x)?, Some(Action::IncreaseSpeed));
    }

    if let Some(x) = config_tree.get::<String>(Action::DecreaseSpeed.into()) {
        map.insert(translate_key_id(x)?, Some(Action::DecreaseSpeed));
    }

    if let Some(x) = config_tree.get::<String>(Action::StartLoop.into()) {
        map.insert(translate_key_id(x)?, Some(Action::StartLoop));
    }

    if let Some(x) = config_tree.get::<String>(Action::EndLoop.into()) {
        map.insert(translate_key_id(x)?, Some(Action::EndLoop));
    }

    if let Some(x) = config_tree.get::<String>(Action::BreakLoop.into()) {
        map.insert(translate_key_id(x)?, Some(Action::BreakLoop));
    }

    if let Some(x) =
        config_tree.get::<String>(Action::CutCurrentLoop(Some(ClipOf_O_D::Offense)).into())
    {
        map.insert(
            translate_key_id(x)?,
            Some(Action::CutCurrentLoop(Some(ClipOf_O_D::Offense))),
        );
    }

    if let Some(x) =
        config_tree.get::<String>(Action::CutCurrentLoop(Some(ClipOf_O_D::Defense)).into())
    {
        map.insert(
            translate_key_id(x)?,
            Some(Action::CutCurrentLoop(Some(ClipOf_O_D::Defense))),
        );
    }

    if let Some(x) = config_tree.get::<String>(Action::CutCurrentLoop(None).into()) {
        map.insert(translate_key_id(x)?, Some(Action::CutCurrentLoop(None)));
    }

    if let Some(x) = config_tree.get::<String>(Action::NextMedia.into()) {
        map.insert(translate_key_id(x)?, Some(Action::NextMedia));
    }

    if let Some(x) = config_tree.get::<String>(Action::PreviousMedia.into()) {
        map.insert(translate_key_id(x)?, Some(Action::PreviousMedia));
    }

    if let Some(x) = config_tree.get::<String>(Action::RestartMedia.into()) {
        map.insert(translate_key_id(x)?, Some(Action::RestartMedia));
    }

    if let Some(x) = config_tree.get::<String>(Action::NextClip.into()) {
        map.insert(translate_key_id(x)?, Some(Action::NextClip));
    }

    if let Some(x) = config_tree.get::<String>(Action::PreviousClip.into()) {
        map.insert(translate_key_id(x)?, Some(Action::PreviousClip));
    }

    if let Some(x) = config_tree.get::<String>(Action::RestartClip.into()) {
        map.insert(translate_key_id(x)?, Some(Action::RestartClip));
    }

    if let Some(x) = config_tree.get::<String>(Action::ConcatClips.into()) {
        map.insert(translate_key_id(x)?, Some(Action::ConcatClips));
    }

    if let Some(x) = config_tree.get::<String>(Action::Stop.into()) {
        map.insert(translate_key_id(x)?, Some(Action::Stop));
    }

    if let Some(x) = config_tree.get::<String>(Action::Exit.into()) {
        map.insert(translate_key_id(x)?, Some(Action::Exit));
    }

    Ok(map)
}

pub fn read_keyboard(tx: std::sync::mpsc::Sender<Action>) {
    println!("starting to read");

    let mut manager = RawInputManager::new().unwrap();
    //manager.register_devices(DeviceType::Joysticks(XInputInclude::True));
    manager.register_devices(DeviceType::Keyboards);
    //manager.register_devices(DeviceType::Mice);
    let key_map = key_map_from_config("keymap.toml").unwrap();
    println!("keymap:\n{:#?}", key_map);
    loop {
        if let Some(event) = manager.get_event() {
            let action_option = match &event {
                RawEvent::KeyboardEvent(_devId, key_id, State::Released) => {
                    println!("{:?}", event);
                    if let Some(mut action_option) = key_map.get(key_id) {
                        println!("doing {:?}", action_option);
                        action_option
                    } else {
                        println!("nothing corresponding");
                        &None
                    }
                }
                _ => &None,
            };

            if let Some(action) = action_option {
                if let Err(_) = tx.send(action.clone()) {
                    println!("other end seems to be terminated. terminating too");
                    break;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

mod test {
    use crate::input::keyboard::*;
    use crate::Action;
    use multiinput::KeyId;

    #[test]
    fn read_config_test() {
        let key_map = key_map_from_config("keymap.toml").unwrap();
        println!("{:?}", key_map);
        assert!(key_map.len() > 0);
    }
}
