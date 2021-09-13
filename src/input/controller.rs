use gilrs::{Button, Event, EventType, Gilrs};

use super::super::Action;
use super::super::ClipOf_O_D;
use std::collections::HashMap;

fn long_press_map(btn: Button) -> Option<Action> {
    match btn {
        Button::DPadLeft => Some(Action::PreviousClip),
        Button::South => Some(Action::BreakLoop),
        Button::West => Some(Action::PreviousCutmark),
        Button::East => Some(Action::NextCutmark),
        Button::North => Some(Action::ConcatClips),
        _ => None,
    }
}

fn short_press_map(btn: Button) -> Option<Action> {
    match btn {
        Button::Start => Some(Action::Exit),
        Button::South => Some(Action::TogglePlayPause),
        Button::West => Some(Action::StartLoop),
        Button::East => Some(Action::EndLoop),
        Button::North => Some(Action::CutCurrentLoop(None)),
        Button::LeftTrigger => Some(Action::DecreaseSpeed),
        Button::RightTrigger => Some(Action::IncreaseSpeed),
        Button::DPadRight => Some(Action::NextClip),
        Button::DPadLeft => Some(Action::RestartClip),
        Button::DPadUp => Some(Action::CutCurrentLoop(Some(ClipOf_O_D::Offense))),
        Button::DPadDown => Some(Action::CutCurrentLoop(Some(ClipOf_O_D::Defense))),
        _ => None,
    }
}

pub fn read_controller(tx: std::sync::mpsc::Sender<Action>) {
    let mut gilrs = Gilrs::new().unwrap();

    println!("list gamepads:");
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    let mut last_pressed = HashMap::new();

    loop {
        while let Some(Event { id, event, time }) = gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event);
            match event {
                EventType::ButtonPressed(btn, _) => {
                    last_pressed.insert(btn, std::time::Instant::now());
                }

                EventType::ButtonReleased(btn, _) => {
                    if let Some(lp) = last_pressed.get(&btn) {
                        if lp.elapsed() > std::time::Duration::from_millis(500) {
                            if let Some(action) = long_press_map(btn) {
                                tx.send(action).unwrap();
                            }
                        } else {
                            if let Some(action) = short_press_map(btn) {
                                tx.send(action).unwrap();
                            }
                        }
                    } else {
                        println!("unexpected error");
                    }
                }

                EventType::ButtonChanged(btn, pos, _) => match btn {
                    Button::LeftTrigger => tx.send(Action::Rewind(pos)).unwrap(),
                    Button::LeftTrigger2 => tx.send(Action::Rewind(pos)).unwrap(),
                    Button::RightTrigger => tx.send(Action::Forward(pos)).unwrap(),
                    Button::RightTrigger2 => tx.send(Action::Forward(pos)).unwrap(),
                    _ => {}
                },

                _ => {}
            };
        }
    }
}
