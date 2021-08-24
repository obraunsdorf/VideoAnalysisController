//use crossterm::{RawScreen, AsyncReader, InputEvent, KeyEvent, SyncReader};

use super::super::Action;
use super::super::ClipOf_O_D;

use std::collections::BTreeMap;
use std::iter::Map;
use std::sync::mpsc::SendError;
use std::time::Duration;

use std::cmp::Ordering;
use winapi_easy::keyboard::{GlobalHotkeySet, Key, KeyCombination, Modifier};

fn translate_key_id(s: &str) -> Result<i32, (String)> {
    match s.to_lowercase().as_str() {
        "escape" => Ok(Key::Esc as i32),
        "return" => Ok(Key::Return as i32),
        "backspace" => Ok(Key::Backspace as i32),
        "left" => Ok(Key::LeftArrow as i32),
        "right" => Ok(Key::RightArrow as i32),
        "up" => Ok(Key::UpArrow as i32),
        "down" => Ok(Key::DownArrow as i32),
        "space" => Ok(Key::Space as i32),
        "a" => Ok(Key::A as i32),
        "b" => Ok(Key::B as i32),
        "c" => Ok(Key::C as i32),
        "d" => Ok(Key::D as i32),
        "e" => Ok(Key::E as i32),
        "f" => Ok(Key::F as i32),
        "g" => Ok(Key::G as i32),
        "h" => Ok(Key::H as i32),
        "i" => Ok(Key::I as i32),
        "j" => Ok(Key::J as i32),
        "k" => Ok(Key::K as i32),
        "l" => Ok(Key::L as i32),
        "m" => Ok(Key::M as i32),
        "n" => Ok(Key::N as i32),
        "o" => Ok(Key::O as i32),
        "p" => Ok(Key::P as i32),
        "q" => Ok(Key::Q as i32),
        "r" => Ok(Key::R as i32),
        "s" => Ok(Key::S as i32),
        "t" => Ok(Key::T as i32),
        "u" => Ok(Key::U as i32),
        "v" => Ok(Key::V as i32),
        "w" => Ok(Key::W as i32),
        "x" => Ok(Key::X as i32),
        "y" => Ok(Key::Y as i32),
        "z" => Ok(Key::Z as i32),
        "f1" => Ok(Key::F1 as i32),
        "f2" => Ok(Key::F2 as i32),
        "f3" => Ok(Key::F3 as i32),
        "f4" => Ok(Key::F4 as i32),
        "f5" => Ok(Key::F5 as i32),
        "f6" => Ok(Key::F6 as i32),
        "f7" => Ok(Key::F7 as i32),
        "f8" => Ok(Key::F8 as i32),
        "f9" => Ok(Key::F9 as i32),
        "f10" => Ok(Key::F10 as i32),
        "f11" => Ok(Key::F11 as i32),
        "f12" => Ok(Key::F12 as i32),
        "0" => Ok(Key::Number0 as i32),
        "1" => Ok(Key::Number1 as i32),
        "2" => Ok(Key::Number2 as i32),
        "3" => Ok(Key::Number3 as i32),
        "4" => Ok(Key::Number4 as i32),
        "5" => Ok(Key::Number5 as i32),
        "6" => Ok(Key::Number6 as i32),
        "7" => Ok(Key::Number7 as i32),
        "8" => Ok(Key::Number8 as i32),
        "9" => Ok(Key::Number9 as i32),
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

fn default_keymap() -> BTreeMap<i32, Option<Action>> {
    let mut map = BTreeMap::new();
    map.insert(Key::Space as i32, Some(Action::TogglePlayPause));
    map.insert(Key::LeftArrow as i32, Some(Action::Rewind(0.7)));
    map.insert(Key::RightArrow as i32, Some(Action::Forward(0.7)));
    map.insert(Key::UpArrow as i32, Some(Action::IncreaseSpeed));
    map.insert(Key::DownArrow as i32, Some(Action::DecreaseSpeed));
    map.insert(Key::T as i32, Some(Action::StartLoop));
    map.insert(Key::Z as i32, Some(Action::EndLoop));
    map.insert(Key::B as i32, Some(Action::BreakLoop));
    map.insert(
        Key::O as i32,
        Some(Action::CutCurrentLoop(Some(ClipOf_O_D::Offense))),
    );
    map.insert(
        Key::D as i32,
        Some(Action::CutCurrentLoop(Some(ClipOf_O_D::Defense))),
    );
    map.insert(Key::C as i32, Some(Action::CutCurrentLoop(None)));
    map.insert(Key::I as i32, Some(Action::NextMedia));
    map.insert(Key::K as i32, Some(Action::PreviousMedia));
    map.insert(Key::M as i32, Some(Action::RestartMedia));
    map.insert(Key::W as i32, Some(Action::NextClip));
    map.insert(Key::S as i32, Some(Action::PreviousClip));
    map.insert(Key::Y as i32, Some(Action::RestartClip));
    map.insert(Key::U as i32, Some(Action::ConcatClips));
    //map.insert(translate_key_id(x)?, Some(Action::Stop));
    map.insert(Key::Esc as i32, Some(Action::Exit));

    map
}

fn key_map_from_config(config_file_path: &str) -> Result<BTreeMap<i32, Option<Action>>, String> {
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
    for (key_id, action_option) in key_map.iter() {
        if let Some(action) = action_option {
            let key: Key = unsafe { std::mem::transmute_copy::<i32, Key>(key_id) };
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
        //.listen_for_hotkeys_with_sleeptime(Some(Duration::from_millis(20)))
        .listen_for_hotkeys()
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
    use super::*;
    use crate::Action;

    #[test]
    fn read_config_test() {
        let key_map = key_map_from_config("keymap.toml").unwrap();
        println!("{:?}", key_map);
        assert!(key_map.len() > 0);
    }
}
