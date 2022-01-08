use gilrs::{Button, Event, EventType, Gilrs};

use super::super::Action;
use super::super::ClipType;
use std::time::Duration;
use std::time::Instant;

struct LastPressed {
    btn: Button,
    pressed_time: Instant,
}

impl LastPressed {
    fn has_been_pressed_within(&self, btn: Button, duration: Duration) -> bool {
        btn == self.btn && self.pressed_time.elapsed() > duration
    }
}

pub(crate) struct Controller {
    engine: Gilrs,
    last_pressed: Option<LastPressed>,
}

impl Controller {
    pub fn new() -> Controller {
        let gilrs = Gilrs::new().unwrap();

        println!("list gamepads:");
        for (_id, gamepad) in gilrs.gamepads() {
            println!("{} is {:?}", gamepad.name(), gamepad.power_info());
        }

        Controller {
            engine: gilrs,
            last_pressed: None,
        }
    }

    fn long_press_map(btn: Button) -> Option<Action> {
        match btn {
            Button::DPadLeft => Some(Action::PreviousMedia),
            Button::DPadRight => Some(Action::NextMedia),
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
            Button::DPadLeft => Some(Action::PreviousClip),
            Button::DPadUp => Some(Action::CutCurrentLoop(Some(ClipType::Offense))),
            Button::DPadDown => Some(Action::CutCurrentLoop(Some(ClipType::Defense))),
            _ => None,
        }
    }

    pub fn next_action(&mut self) -> Option<Action> {
        if let Some(Event { id, event, time }) = self.engine.next_event() {
            dbg!("{:?} New event from {}: {:?}", time, id, event);
            match event {
                EventType::ButtonPressed(btn, _) => {
                    self.last_pressed = Some(LastPressed {
                        btn,
                        pressed_time: std::time::Instant::now(),
                    });
                    None
                }

                EventType::ButtonReleased(btn, _) => match &self.last_pressed {
                    Some(x) if x.has_been_pressed_within(btn, Duration::from_millis(500)) => {
                        Controller::long_press_map(btn)
                    }
                    _ => Controller::short_press_map(btn),
                },

                EventType::ButtonChanged(btn, pos, _) => match btn {
                    Button::LeftTrigger => Some(Action::Rewind(pos)),
                    Button::LeftTrigger2 => Some(Action::Rewind(pos)),
                    Button::RightTrigger => Some(Action::Forward(pos)),
                    Button::RightTrigger2 => Some(Action::Forward(pos)),
                    _ => None,
                },

                _ => None,
            }
        } else {
            None
        }
    }
}
