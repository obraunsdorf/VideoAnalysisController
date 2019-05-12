use gilrs::{Gilrs, Button, Event, EventType};

use super::super::Action as Action;
use super::super::ClipOf_O_D as ClipOf_O_D;

pub fn read_controller(tx: std::sync::mpsc::Sender<Action>) {
    let mut gilrs = Gilrs::new().unwrap();

    println!("list gamepads:");
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    let mut last_DPadLeft_pressed= std::time::Instant::now();

    loop {
        // Examine new events
        while let Some(Event { id, event, time }) = gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event);
            match event {
                EventType::ButtonPressed(btn, _) => {
                    match btn {
                        Button::Start => tx.send(Action::Exit).unwrap(),
                        Button::South => tx.send(Action::TogglePlayPause).unwrap(),
                        Button::West => tx.send(Action::StartLoop).unwrap(),
                        Button::East => tx.send(Action::EndLoop).unwrap(),
                        Button::North => tx.send(Action::CutCurrentLoop(None)).unwrap(),
                        Button::LeftTrigger => tx.send(Action::DecreaseSpeed).unwrap(),
                        Button::RightTrigger => tx.send(Action::IncreaseSpeed).unwrap(),
                        Button::DPadRight => tx.send(Action::NextClip).unwrap(),
                        Button::DPadLeft => {
                            last_DPadLeft_pressed = std::time::Instant::now();
                        },
                        Button::DPadUp => tx.send(Action::CutCurrentLoop(Some(ClipOf_O_D::Offense))).unwrap(),
                        Button::DPadDown => tx.send(Action::CutCurrentLoop(Some(ClipOf_O_D::Defense))).unwrap(),
                        _ => {}
                    }
                }

                EventType::ButtonReleased(btn, _) => {
                    match btn {
                        Button::DPadLeft => {
                            if last_DPadLeft_pressed.elapsed() > std::time::Duration::from_millis(500) {
                                tx.send(Action::PreviousClip).unwrap();
                            } else {
                                tx.send(Action::RestartClip).unwrap();
                            }
                        },
                        _ => {}
                    }
                }

                EventType::ButtonChanged(btn, pos, _) => {
                    match btn {
                        Button::LeftTrigger => tx.send(Action::Rewind(pos)).unwrap(),
                        Button::LeftTrigger2 => tx.send(Action::Rewind(pos)).unwrap(),
                        Button::RightTrigger => tx.send(Action::Forward(pos)).unwrap(),
                        Button::RightTrigger2 => tx.send(Action::Forward(pos)).unwrap(),
                        _ => {}
                    }
                }
                _ => {}
            };
        }
    }
}