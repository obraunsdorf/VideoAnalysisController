use fltk::enums::Key;
use std::collections::BTreeMap;

use crate::{Action, ClipType};

fn default_keymap() -> BTreeMap<Key, Option<Action>> {
    let mut map = BTreeMap::new();

    map.insert(Key::from_char(' '), Some(Action::TogglePlayPause));
    map.insert(Key::Left, Some(Action::Rewind(0.7)));
    map.insert(Key::Right, Some(Action::Forward(0.7)));
    map.insert(Key::Up, Some(Action::IncreaseSpeed));
    map.insert(Key::Down, Some(Action::TogglePlayPause));
    map.insert(Key::from_char('t'), Some(Action::StartLoop));
    map.insert(Key::from_char('z'), Some(Action::EndLoop));
    map.insert(Key::from_char('b'), Some(Action::BreakLoop));
    map.insert(
        Key::from_char('o'),
        Some(Action::CutCurrentLoop(Some(ClipType::Offense))),
    );
    map.insert(
        Key::from_char('d'),
        Some(Action::CutCurrentLoop(Some(ClipType::Defense))),
    );
    map.insert(Key::from_char('c'), Some(Action::CutCurrentLoop(None)));
    map.insert(Key::from_char('i'), Some(Action::NextMedia));
    map.insert(Key::from_char('k'), Some(Action::PreviousMedia));
    map.insert(Key::from_char('m'), Some(Action::RestartMedia));
    map.insert(Key::from_char('w'), Some(Action::NextClip));
    map.insert(Key::from_char('s'), Some(Action::PreviousClip));
    map.insert(Key::from_char('y'), Some(Action::RestartClip));
    map.insert(Key::from_char('u'), Some(Action::ConcatClips));
    map.insert(Key::from_char('1'), Some(Action::PreviousCutmark));
    map.insert(Key::from_char('2'), Some(Action::NextCutmark));
    map.insert(Key::Escape, Some(Action::Exit));
    map
}

pub(crate) fn action_from_pressed_key(key: fltk::enums::Key) -> Option<Action> {
    let keymap = default_keymap();
    println!("key {} pressed", key.bits() as u32);
    if let Some(action_option) = keymap.get(&key) {
        println!(
            "key {} yields action {:?}",
            key.bits() as u32,
            action_option
        );

        action_option.clone()
    } else {
        None
    }
}
