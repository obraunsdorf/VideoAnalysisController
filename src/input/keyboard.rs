//use crossterm::{RawScreen, AsyncReader, InputEvent, KeyEvent, SyncReader};

use super::super::Action;
use super::super::ClipOf_O_D;

use std::collections::BTreeMap;
use std::iter::Map;
use std::sync::mpsc::SendError;
use std::time::Duration;

use std::cmp::Ordering;
use winapi_easy::keyboard::{GlobalHotkeySet, Key, KeyCombination, Modifier};

fn translate_key_id(s: &str) -> Result<Key, (String)> {
    match s.to_lowercase().as_str() {
        "escape" => Ok(Key::Esc),
        "return" => Ok(Key::Return),
        "backspace" => Ok(Key::Backspace),
        "left" => Ok(Key::LeftArrow),
        "right" => Ok(Key::RightArrow),
        "up" => Ok(Key::UpArrow),
        "down" => Ok(Key::DownArrow),
        "space" => Ok(Key::Space),
        "a" => Ok(Key::A),
        "b" => Ok(Key::B),
        "c" => Ok(Key::C),
        "d" => Ok(Key::D),
        "e" => Ok(Key::E),
        "f" => Ok(Key::F),
        "g" => Ok(Key::G),
        "h" => Ok(Key::H),
        "i" => Ok(Key::I),
        "j" => Ok(Key::J),
        "k" => Ok(Key::K),
        "l" => Ok(Key::L),
        "m" => Ok(Key::M),
        "n" => Ok(Key::N),
        "o" => Ok(Key::O),
        "p" => Ok(Key::P),
        "q" => Ok(Key::Q),
        "r" => Ok(Key::R),
        "s" => Ok(Key::S),
        "t" => Ok(Key::T),
        "u" => Ok(Key::U),
        "v" => Ok(Key::V),
        "w" => Ok(Key::W),
        "x" => Ok(Key::X),
        "y" => Ok(Key::Y),
        "z" => Ok(Key::Z),
        "f1" => Ok(Key::F1),
        "f2" => Ok(Key::F2),
        "f3" => Ok(Key::F3),
        "f4" => Ok(Key::F4),
        "f5" => Ok(Key::F5),
        "f6" => Ok(Key::F6),
        "f7" => Ok(Key::F7),
        "f8" => Ok(Key::F8),
        "f9" => Ok(Key::F9),
        "f10" => Ok(Key::F10),
        "f11" => Ok(Key::F11),
        "f12" => Ok(Key::F12),
        "0" => Ok(Key::Number0),
        "1" => Ok(Key::Number1),
        "2" => Ok(Key::Number2),
        "3" => Ok(Key::Number3),
        "4" => Ok(Key::Number4),
        "5" => Ok(Key::Number5),
        "6" => Ok(Key::Number6),
        "7" => Ok(Key::Number7),
        "8" => Ok(Key::Number8),
        "9" => Ok(Key::Number9),
        /*"pause" => Ok(Key::Pause),
        "pageup" => Ok(Key::PageUp),
        "pagedown" => Ok(Key::PageDown),
        "printscreen" => Ok(Key::PrintScreen),
        "insert" => Ok(Key::Insert),
        "end" => Ok(Key::End),
        "home" => Ok(Key::Home),
        "delete" => Ok(Key::Delete),
        "add" => Ok(Key::Add),
        "subtract" => Ok(Key::Subtract),
        "multiply" => Ok(Key::Multiply),
        "separator" => Ok(Key::Separator),
        "decimal" => Ok(Key::Decimal),
        "divide" => Ok(Key::Divide),
        "backtick" => Ok(Key::BackTick),
        "backslash" => Ok(Key::BackSlash),
        "forwardslash" => Ok(Key::ForwardSlash),
        "plus" => Ok(Key::Plus),
        "minus" => Ok(Key::Minus),
        "fullstop" => Ok(Key::FullStop),
        "comma" => Ok(Key::Comma),
        "tab" => Ok(Key::Tab),
        "numlock" => Ok(Key::Numlock),
        "leftsquarebracket" => Ok(Key::LeftSquareBracket),
        "rightsquarebracket" => Ok(Key::RightSquareBracket),
        "semicolon" => Ok(Key::SemiColon),
        "apostrophe" => Ok(Key::Apostrophe),
        "hash" => Ok(Key::Hash),*/
        _ => Err(format!("no corresponding key id for '{}'", s)),
    }
}

fn default_keymap() -> BTreeMap<Key, Option<Action>> {
    let mut map = BTreeMap::new();
    map.insert(Key::Space, Some(Action::TogglePlayPause));
    map.insert(Key::LeftArrow, Some(Action::Rewind(0.7)));
    map.insert(Key::RightArrow, Some(Action::Forward(0.7)));
    map.insert(Key::UpArrow, Some(Action::IncreaseSpeed));
    map.insert(Key::DownArrow, Some(Action::DecreaseSpeed));
    map.insert(Key::T, Some(Action::StartLoop));
    map.insert(Key::Z, Some(Action::EndLoop));
    map.insert(Key::B, Some(Action::BreakLoop));
    map.insert(
        Key::O,
        Some(Action::CutCurrentLoop(Some(ClipOf_O_D::Offense))),
    );
    map.insert(
        Key::D,
        Some(Action::CutCurrentLoop(Some(ClipOf_O_D::Defense))),
    );
    map.insert(Key::C, Some(Action::CutCurrentLoop(None)));
    map.insert(Key::I, Some(Action::NextMedia));
    map.insert(Key::K, Some(Action::PreviousMedia));
    map.insert(Key::M, Some(Action::RestartMedia));
    map.insert(Key::W, Some(Action::NextClip));
    map.insert(Key::S, Some(Action::PreviousClip));
    map.insert(Key::Y, Some(Action::RestartClip));
    map.insert(Key::U, Some(Action::ConcatClips));
    //map.insert(translate_key_id(x)?, Some(Action::Stop));
    map.insert(Key::Esc, Some(Action::Exit));

    map
}

fn key_map_from_config(config_file_path: &str) -> Result<BTreeMap<Key, Option<Action>>, String> {
    if let Ok(config_tree) = configuration::format::TOML::open(config_file_path) {
        let mut map = BTreeMap::new();

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
    } else {
        let map = default_keymap();
        println!(
            "Could not open keyboard config '{}'. Using he following default key  map: {:#?}",
            config_file_path, map
        );
        Ok(map)
    }
}

pub fn read_keyboard(tx: std::sync::mpsc::Sender<Action>) {
    /*let fg_window = WindowHandle::get_foreground_window().unwrap();
    let c_window = WindowHandle::get_console_window().unwrap();
    let d_window = WindowHandle::get_desktop_window().unwrap();

    for w in [fg_window, c_window, d_window].iter() {
        println!("window: {}", w.get_caption_text());
    }*/

    let mut hotkeys = GlobalHotkeySet::new();
    let key_map = match std::env::current_exe() {
        Ok(mut config_file_path) => {
            config_file_path = config_file_path.parent().unwrap().join("keymap.toml");
            println!(
                "looking for keymap at {}",
                config_file_path.to_str().unwrap()
            );
            key_map_from_config(config_file_path.to_str().unwrap()).unwrap()
        }
        Err(_) => default_keymap(),
    };
    for (key, action_option) in key_map.iter() {
        if let Some(action) = action_option {
            hotkeys = hotkeys.add_global_hotkey(action.clone(), key.clone());
        }
    }

    /*.add_global_hotkey(Action::TogglePlayPause, Key::Space)
    .add_global_hotkey(Action::Rewind(0.7), Key::LeftArrow)
    .add_global_hotkey(Action::Forward(0.7), Key::RightArrow)
    .add_global_hotkey(Action::IncreaseSpeed, Key::UpArrow)
    .add_global_hotkey(Action::DecreaseSpeed, Key::DownArrow)
    .add_global_hotkey(Action::StartLoop, Key::T)
    .add_global_hotkey(Action::EndLoop, Key::Z)
    .add_global_hotkey(Action::BreakLoop, Key::B)
    .add_global_hotkey(Action::CutCurrentLoop(Some(ClipOf_O_D::Defense)), Key::D)
    .add_global_hotkey(Action::CutCurrentLoop(Some(ClipOf_O_D::Offense)), Key::O)
    .add_global_hotkey(Action::CutCurrentLoop(None), Key::C)
    .add_global_hotkey(Action::NextMedia, Key::I)
    .add_global_hotkey(Action::PreviousMedia, Key::K)
    .add_global_hotkey(Action::RestartMedia, Key::M)
    .add_global_hotkey(Action::NextClip, Key::W)
    .add_global_hotkey(Action::PreviousClip, Key::S)
    .add_global_hotkey(Action::RestartClip, Key::Y)
    .add_global_hotkey(Action::ConcatClips, Key::U)
    .add_global_hotkey(Action::Exit, Key::Esc);*/

    for action_result in hotkeys
        .listen_for_hotkeys_with_sleeptime(Some(Duration::from_millis(20)))
        .unwrap()
    {
        if let Ok(action) = action_result {
            tx.send(action.clone()).unwrap();
        }
    }

    /*let mut manager = RawInputManager::new().unwrap();
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
    }*/
}

mod test {
    use crate::input::keyboard::*;
    use crate::Action;

    #[test]
    fn read_config_test() {
        let key_map = key_map_from_config("keymap.toml").unwrap();
        println!("{:?}", key_map);
        assert!(key_map.len() > 0);
    }
}
