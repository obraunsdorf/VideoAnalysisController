#![feature(duration_float)]


use std::sync::mpsc::channel;
use std::thread;
use std::process::Command;
use std::string::{ToString, String};
use std::collections::btree_set::BTreeSet;

use vlc::{Instance, Media, MediaPlayer, Event, EventType, State};
use vlc::EventType as VLCEventType;

use gilrs::{Gilrs, Button};

use gilrs::EventType as GilrsEventType;



enum Action {
    TogglePlayPause,
    Rewind(f32),
    Forward(f32),
    IncreaseSpeed,
    DecreaseSpeed,
    StartLoop,
    EndLoop,
    CheckLoopEnd(f32),
    CutCurrentLoop,
    NextClip,
    PreviousClip,
    RestartClip,
    Exit,
}


static MAX_SPEED: f32 = 16.0;
static BREAKPOINT: f32 = 0.5;

fn read_controller(tx: std::sync::mpsc::Sender<Action>) {
    let mut gilrs = Gilrs::new().unwrap();

    println!("list gamepads:");
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

   let mut last_DPadLeft_pressed= std::time::Instant::now();

    loop {
        // Examine new events
        while let Some(gilrs::Event { id, event, time }) = gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event);
            match event {
                gilrs::EventType::ButtonPressed(btn, _) => {
                    match btn {
                        gilrs::Button::South => tx.send(Action::TogglePlayPause).unwrap(),
                        gilrs::Button::West => tx.send(Action::StartLoop).unwrap(),
                        gilrs::Button::East => tx.send(Action::EndLoop).unwrap(),
                        gilrs::Button::North => tx.send(Action::CutCurrentLoop).unwrap(),
                        gilrs::Button::LeftTrigger => tx.send(Action::DecreaseSpeed).unwrap(),
                        gilrs::Button::RightTrigger => tx.send(Action::IncreaseSpeed).unwrap(),
                        Button::DPadRight => tx.send(Action::NextClip).unwrap(),
                        Button::DPadLeft => {
                            last_DPadLeft_pressed = std::time::Instant::now();
                        },
                        _ => {}
                    }
                }

                gilrs::EventType::ButtonReleased(btn, _) => {
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

                gilrs::EventType::ButtonChanged(btn, pos, _) => {
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

fn main() {
    let args: Vec<String> = std::env::args().collect();

    println!("args: {:?}", args);
    let path = match args.get(1) {
        Some(s) => s,
        None => {
            println!("Usage: gac path_to_a_media_file");
            return;
        }
    };
    let (tx, rx) = channel::<Action>();

    let tx_1 = tx.clone();
    let tx_2 = tx.clone();
    thread::spawn(|| {
        read_controller(tx_1);
    });

    let vlc_args: Vec<String> = vec![
        String::from("--verbose=2")
    ];
    let instance = Instance::new().unwrap();
    //let instance = Instance::with_args(Some(vlc_args)).unwrap();
    //instance.add_intf("qt");


    /*if let Some(filter_list) = instance.video_filter_list_get() {
        for filter in filter_list.into_iter() {
            println!("video filter: {:?}", filter.name);
        }
    }*/

    //let md = Media::new_location(&instance, "https://www.youtube.com/watch?v=M3Q8rIgveO0").unwrap();
    let md = Media::new_path(&instance, path).unwrap();
    let em = md.event_manager();
    let _ = em.attach(EventType::MediaStateChanged, move |e, _| {
        match e {
            Event::MediaStateChanged(s) => {
                println!("State : {:?}", s);
                if s == State::Ended || s == State::Error {
                    tx.send(Action::Exit).unwrap();
                }
            }
            _ => (),
        }
    });
    let _ = em.attach(EventType::MediaPlayerPositionChanged, move |e, _| {
        match e {
            Event::MediaPlayerPositionChanged(pos) => {
                println!("position changed, new pos: {:?}", pos);
                tx_2.send(Action::CheckLoopEnd(pos)).unwrap();
            }
            _ => (),
        }
    });

    let mdp = MediaPlayer::new(&instance).unwrap();
    mdp.set_media(&md);
    mdp.play();

    let mut loop_start: i64 = -1;
    let mut loop_end: i64 = -1;
    let mut clipcount = 0;

    let mut clips: BTreeSet<i64> = BTreeSet::new();

    loop {
        match rx.recv().unwrap() {
            Action::TogglePlayPause => {
                if mdp.is_playing() {
                    mdp.pause();
                } else {
                    mdp.play();
                }
            }
            Action::Forward(speed) => {
                /*//mdp.pause();
                let mut y;
                if speed < 0.1 {
                    y = 0.0;
                } else if speed >= 0.1 {
                    y = 0.1
                } else if speed >= 0.2 {
                    y = 0.2
                } else if speed >= 0.3 {
                    y = 0.3
                } else if speed >= 0.4 {
                    y = 0.6
                } else if speed >= 0.5 {
                    y = 0.8
                } else if speed >= 0.6 {
                    y = 1.0
                } else if speed >= 0.7 {
                    y = 2.0
                } else if speed >= 0.8 {
                    y = 4.0
                } else if speed >= 0.9 {
                    y = 8.0
                } else if speed >= 1.0 {
                    y = 16.0;
                }
                let cur_time = mdp.get_time().unwrap();
                mdp.set_time(cur_time + speed as i64 * 10);
                //mdp.pause();*/
                let new_time = mdp.get_time().unwrap() + (speed*1000.0) as i64;
                mdp.set_time(new_time);

            }

            Action::Rewind(speed) => {
                let new_time = mdp.get_time().unwrap() - (speed*1000.0) as i64;
                mdp.set_time(new_time);
            }

            Action::CutCurrentLoop => {
                clips.insert(loop_start);
                println!("cutting from {:?} to {:?}...", loop_start, loop_end);
                let out_file_name =  loop_start.to_string() + "_" + path;
                let vlc_output_arg = ":sout=#file{dst=".to_owned() + &out_file_name + "}";

                debug_assert!(loop_start >= 0 && loop_end > loop_start);

                let start = loop_start as f32 / 1000.0;
                let end = loop_end as f32 / 1000.0;

                /*let cmd_args = format!("-ss {:?} -i {} -to {:?} -c copy /home/oliver/Desktop/rust-vlc/{}",
                                  start,
                                  path,
                                  end,
                                  out_file_name);*/
                let cmd_args = format!("{} --start-time {:?} --stop-time {:?} {} :no-sout-rtp-sap :no-sout-standard-sap :sout-keep", path, start, end, vlc_output_arg);
                //println!("my vlc command: {:?}", cmd_args);

                let child_proc = Command::new("ffmpeg")
                    .arg("-ss")
                    .arg(format!("{}", start))
                    .arg("-i")
                    .arg(path)
                    .arg("-to")
                    .arg(format!("{}", end))
                    .arg("-c")
                    .arg("copy")
                    .arg(out_file_name)
                    .spawn()
                    .expect("failed to execute vlc app");

                println!("command executed");
            }


            Action::IncreaseSpeed => {
                let current_speed = mdp.get_rate();
                mdp.set_rate(current_speed + 0.1);
            }

            Action::DecreaseSpeed => {
                let current_speed = mdp.get_rate();
                mdp.set_rate(current_speed - 0.1);
            }
            Action::StartLoop => {
                match mdp.get_time() {
                    Some(start) => {
                        loop_start = start;
                        if loop_start >= loop_end {
                            loop_end = -1;
                        }
                    }
                    None => println!("error getting time")
                }
                println!("set loop start at {:?}", loop_start)
            }

            Action::PreviousClip => {
                let cur_time = mdp.get_time().unwrap();

                let mut iter = clips.iter().rev();
                while let Some(clip) = iter.next() {
                    if clip <= &cur_time {
                        if let Some(prev_clip) = iter.next() {
                            mdp.set_time(*prev_clip);
                            println!("previous clip from {}", *prev_clip);
                        } else {
                            mdp.set_time(*clip);
                            println!("previous clip from {}", *clip);
                        }
                        break;
                    }
                }
            }

            Action::NextClip => {
                let cur_time = mdp.get_time().unwrap();
                for clip in &mut clips.iter() {
                    if clip >= &cur_time {
                        mdp.set_time(*clip);
                        println!("jumping to {}", *clip);
                        break;
                    }
                }
            }

            Action::RestartClip => {
                let cur_time = mdp.get_time().unwrap();
                for clip in &mut clips.iter().rev() {
                    if clip <= &cur_time {
                        mdp.set_time(*clip);
                        println!("restarting clip from to {}", *clip);
                        break;
                    }
                }
            }

            Action::EndLoop => {
                match mdp.get_time() {
                    Some(end) => {
                        loop_end = end;
                        if loop_end <= loop_start {
                            loop_start = -1;
                        }
                    }

                    None => println!("error getting time")
                }
                println!("set loop end at {:?}", loop_end);
                mdp.set_time(loop_start);
            }

            /*Action::CheckLoopEnd(pos) => {
                if loop_end != -1 && pos >= loop_end {
                    mdp.set_time(loop_start);
                }
            }*/

            Action::Exit => {
                mdp.stop()
            },
            _ => {}
        };
    }
    println!("exiting");
}


