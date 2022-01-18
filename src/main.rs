use std::{
    collections::btree_set::BTreeSet,
    fs::File,
    io::{self, BufRead},
    sync::{mpsc::Sender, Arc, Mutex},
};

use std::string::String;
use std::sync::mpsc::channel;

use std::convert::TryInto;

use fltk::{
    dialog::FileDialogType,
    prelude::{InputExt, ValuatorExt, WidgetExt, WindowExt},
};
use fltk_gui::{FltkGui, GuiActions};
use vlc::{Instance, MediaPlayer, MediaPlayerVideoEx};

pub mod ffmpeg;
mod input;
use std::path::Path;
use std::path::PathBuf;

mod action_handling;
use action_handling::ActionHandler;

mod fltk_gui;

use crate::input::{controller::Controller, keyboard_fltk::action_from_pressed_key};

const CLIP_SUFFIX_OFFENSE: &str = "Off";
const CLIP_SUFFIX_DEFENSE: &str = "Def";

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum ClipType {
    Offense,
    Defense,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Action {
    TogglePlayPause,
    Rewind(f32),
    Forward(f32),
    IncreaseSpeed,
    DecreaseSpeed,
    StartLoop,
    EndLoop,
    BreakLoop,
    //CheckLoopEnd(f32),
    CutCurrentLoop(Option<ClipType>),
    NextMedia,
    PreviousMedia,
    RestartMedia,
    NextClip,
    PreviousClip,
    RestartClip,
    ConcatClips,
    PreviousCutmark,
    NextCutmark,
    Stop,
    Exit,
}

impl From<Action> for &str {
    fn from(action: Action) -> Self {
        match action {
            Action::TogglePlayPause => "TogglePlayPause",
            Action::Rewind(_) => "Rewind",
            Action::Forward(_) => "Forward",
            Action::IncreaseSpeed => "IncreaseSpeed",
            Action::DecreaseSpeed => "DecreaseSpeed",
            Action::StartLoop => "StartLoop",
            Action::EndLoop => "EndLoop",
            Action::BreakLoop => "BreakLoop",
            Action::CutCurrentLoop(od) => match od {
                Some(ClipType::Offense) => "CutLoop_Offense",
                Some(ClipType::Defense) => "CutLoop_Defense",
                None => "CutLoop",
            },
            Action::NextMedia => "NextMedia",
            Action::PreviousMedia => "PreviousMedia",
            Action::RestartMedia => "RestartMedia",
            Action::NextClip => "NextClip",
            Action::PreviousClip => "PreviousClip",
            Action::RestartClip => "RestartClip",
            Action::ConcatClips => "ConcatClips",
            Action::PreviousCutmark => "PreviousCutmark",
            Action::NextCutmark => "NextCutmarks",
            Action::Stop => "Stop",
            Action::Exit => "Exit",
        }
    }
}

const VIDEO_EXTENSIONS: &[&str] = &["MOV", "MPEG", "MP4"];

/*fn check_loop_end(tx_orig: &std::sync::mpsc::Sender<Action>,
                  mdp: MediaPlayer,
                  loop_start: i64,
                  loop_end: i64
) {
    let tx = tx_orig.clone();
    std::thread::spawn(|| {
        let mut error_count = 0;
        loop{
            if let Some(time) = mdp.get_time() {
                if time >= loop_end  {
                    tx.send(Action::RestartClip);
                    return;
                }
            } else {
                error_count = error_count + 1;
            }

            if error_count >= 3 {
                println!("error getting time while checking for loop end");
                return;
            }

        }

    });
}*/

/*
fn load_media(
    vlc_instance: &vlc::Instance,
    path: &Path,
    tx_0: &std::sync::mpsc::Sender<Action>,
) -> Media {
    //let md = Media::new_location(&instance, "https://www.youtube.com/watch?v=M3Q8rIgveO0").unwrap();
    let tx = tx_0.clone();
    let _tx_2 = tx_0.clone();
    let md = Media::new_path(&vlc_instance, path).unwrap();
    let em = md.event_manager();
    let _ = em.attach(EventType::MediaStateChanged, move |e, _| match e {
        Event::MediaStateChanged(s) => {
            println!("State : {:?}", s);
            match s {
                State::Ended => tx.send(Action::Stop).unwrap(),
                State::Error => tx.send(Action::Exit).unwrap(),
                _ => {}
            }
        }
        _ => (),
    });

    /*let _ = em.attach(EventType::MediaPlayerPositionChanged, move |e, _| {
        match e {
            Event::MediaPlayerPositionChanged(pos) => {
                println!("position changed, new pos: {:?}", pos);
                tx_2.send(Action::CheckLoopEnd(pos)).unwrap();
            }
            _ => (),
        }
    });*/

    return md;
}
 */

fn main() {
    //startVLC(None, None);
    run_with_fltk();
}

fn run_with_fltk() {
    let fltk_gui = FltkGui::new();
    start_vlc(Some(fltk_gui))
}

fn start_vlc(mut fltk_gui: Option<FltkGui>) {
    let args: Vec<String> = std::env::args().collect();
    let mut controller = Controller::new();

    let instance = Instance::new().unwrap();
    /*let vlc_args: Vec<String> = vec![
        String::from("--verbose=1"),
        String::from("--file-logging"),
        String::from("--logfile=libvlc.log"),
    ];*/
    //let instance = Instance::with_args(Some(vlc_args)).unwrap();
    //instance.add_intf("qt");

    /*if let Some(filter_list) = instance.video_filter_list_get() {
        for filter in filter_list.into_iter() {
            println!("video filter: {:?}", filter.name);
        }
    }*/

    let mdp = MediaPlayer::new(&instance).unwrap();

    if let Some(gui) = &fltk_gui {
        let handle: fltk::window::RawHandle = gui.vlc_win.raw_handle();

        #[cfg(target_os = "windows")]
        mdp.set_hwnd(handle);

        #[cfg(target_os = "linux")]
        mdp.set_xwindow(handle.try_into().unwrap()); // TODO unchecked u64 -> u32 conversion

        // Disable event handling on vlc's side
        // Do it thru fltk
        mdp.set_key_input(false);
        mdp.set_mouse_input(false);
    } else if !mdp.get_fullscreen() {
        mdp.toggle_fullscreen();
    }

    let mut acm_exe_path: Option<PathBuf> = None;

    let (tx_cutmarks_ready, rx_cutmarks_ready) = channel::<Arc<Mutex<Box<Cutmarks>>>>();

    let project_dir = if let Some(s) = args.get(1) {
        PathBuf::from(s.clone())
    } else {
        loop {
            if fltk::app::wait() {
                if let Some(GuiActions::SetProjectDirectory(dir)) =
                    fltk_gui.as_mut().unwrap().gui_actions_receiver.recv()
                {
                    break PathBuf::from(dir);
                }
            }
        }
    };

    let mut action_handler = ActionHandler::new(&instance, mdp, project_dir).unwrap();

    loop {
        let event_happened = fltk::app::wait_for(0.01).unwrap();

        action_handler.check_loop_end();

        if let Ok(cutmark_mutex) = rx_cutmarks_ready.try_recv() {
            let guard = cutmark_mutex.lock().unwrap();
            let cutmarks = guard.clone(); //TODO: does this clone the BTreeSet? If yes, rather use cutmark_mutex.into_inner()?
            action_handler.set_cutmarks(cutmarks)
        }

        if let Some(gui) = &mut fltk_gui {
            if !gui.slider.has_focus() {
                gui.slider
                    .set_value(action_handler.get_media_relative_position() as f64);
            }

            if event_happened {
                if let Some(gui_action) = gui.gui_actions_receiver.recv() {
                    match gui_action {
                        GuiActions::SetProjectDirectory(dir) => {
                            action_handler
                                .set_project_directory(PathBuf::from(dir))
                                .unwrap();
                        }

                        GuiActions::SetMediaPosition(pos) => {
                            action_handler.set_media_relative_position(pos as f32)
                        }

                        GuiActions::KeyEvent(key) => {
                            if let Some(action) = action_from_pressed_key(key) {
                                if let Err(e) = action_handler.handle(action) {
                                    println!("exiting because of: {}", e);
                                    break;
                                }
                            }
                        }

                        GuiActions::ChooseACMExe => {
                            let mut acm_exe_chooser =
                                fltk::dialog::FileDialog::new(FileDialogType::BrowseFile);
                            acm_exe_chooser.show();
                            acm_exe_path = Some(acm_exe_chooser.filename());
                        }

                        GuiActions::CalibrateNear => match acm_exe_path {
                            Some(ref path) => {
                                let threshold_near: u64 =
                                    gui.calib_near_input.value().parse().unwrap();
                                autocutmarks_calibrate_near(
                                    path,
                                    action_handler.get_current_media_path().unwrap(),
                                    action_handler.get_current_frame(),
                                    threshold_near,
                                );
                            }

                            None => {
                                println!("Executable for AutoCutMarks was not set");
                            }
                        },

                        GuiActions::CalibrateFar => match acm_exe_path {
                            Some(ref path) => {
                                let threshold_far: u64 =
                                    gui.calib_far_input.value().parse().unwrap();
                                autocutmarks_calibrate_far(
                                    path,
                                    action_handler.get_current_media_path().unwrap(),
                                    action_handler.get_current_frame(),
                                    threshold_far,
                                );
                            }

                            None => {
                                println!("Executable for AutoCutMarks was not set");
                            }
                        },

                        GuiActions::Analyze => {
                            match acm_exe_path {
                                Some(ref path) => {
                                    let start_frame_value = gui.start_frame_input.value();
                                    let start_frame: Option<i64> = if start_frame_value.is_empty() {
                                        None
                                    } else {
                                        Some(start_frame_value.parse().unwrap())
                                        //TODO error handling
                                    };

                                    let end_frame_value = gui.end_frame_input.value();
                                    let end_frame: Option<i64> = if end_frame_value.is_empty() {
                                        None
                                    } else {
                                        Some(end_frame_value.parse().unwrap()) //TODO error handling
                                    };
                                    analyze_autocutmarks(
                                        path,
                                        action_handler.get_current_media_path().unwrap(),
                                        start_frame,
                                        end_frame,
                                        action_handler.get_fps(),
                                        tx_cutmarks_ready.clone(),
                                    );
                                }

                                None => {
                                    println!("Executable for AutoCutMarks was not set");
                                }
                            }
                        }

                        GuiActions::AnalyzeCached => match acm_exe_path {
                            Some(ref path) => {
                                let sensitivity_value = gui.sensitivity_input.value();
                                let sensitivity: Option<f32> = if sensitivity_value.is_empty() {
                                    None
                                } else {
                                    Some(sensitivity_value.parse().unwrap())
                                    //TODO error handling
                                };
                                analyze_autocutmarks_cached(
                                    path,
                                    action_handler.get_current_media_path().unwrap(),
                                    action_handler.get_fps(),
                                    sensitivity,
                                    tx_cutmarks_ready.clone(),
                                );
                            }

                            None => {
                                println!("Executable for AutoCutMarks was not set");
                            }
                        },

                        GuiActions::SetStartFrame => {
                            let start_frame = action_handler.get_current_frame();
                            dbg!("set start frame to {}", start_frame);
                            let s = start_frame.to_string();
                            gui.start_frame_input.set_value(&s);
                        }

                        GuiActions::SetEndFrame => {
                            let end_frame = action_handler.get_current_frame();
                            let s = end_frame.to_string();
                            gui.end_frame_input.set_value(&s);
                        }
                    }
                }
            }
        }

        if let Some(action) = controller.next_action() {
            if let Err(e) = action_handler.handle(action) {
                println!("exiting because of: {}", e);
                break;
            }
        }
    }

    println!("exiting");
    std::process::exit(0);
}

type Cutmarks = BTreeSet<i64>;

fn analyze_autocutmarks(
    acm_exe_path: &Path,
    videofile: &Path,
    start_frame: Option<i64>,
    end_frame: Option<i64>,
    fps: f32,
    tx: Sender<Arc<Mutex<Box<Cutmarks>>>>,
) {
    let acm_exe_path = acm_exe_path.to_path_buf();
    let videofile = videofile.to_path_buf();
    std::thread::spawn(move || {
        let mut cmd = std::process::Command::new(acm_exe_path);

        if let Some(start) = start_frame {
            cmd.arg(format!("-s {}", start));
        }

        if let Some(end) = end_frame {
            cmd.arg(format!("-e {}", end));
        }
        let status = cmd.arg(videofile).arg("snaps.txt").status().unwrap();

        if status.success() {
            let cutmarks = read_cutmarks_file(fps);
            tx.send(Arc::new(Mutex::new(cutmarks))).unwrap();
        }
    });
}

fn analyze_autocutmarks_cached(
    acm_exe_path: &Path,
    videofile: &Path,
    fps: f32,
    sensitivity: Option<f32>,
    tx: Sender<Arc<Mutex<Box<Cutmarks>>>>,
) {
    let acm_exe_path = acm_exe_path.to_path_buf();
    let videofile = videofile.to_path_buf();
    std::thread::spawn(move || {
        let mut cmd = std::process::Command::new(acm_exe_path);
        cmd.arg("--mode=use-cached");

        if let Some(sensitivity) = sensitivity {
            cmd.arg(format!("--sensitivity={}", sensitivity));
        };

        let status = cmd.arg(videofile).arg("snaps.txt").status().unwrap();

        if status.success() {
            let cutmarks = read_cutmarks_file(fps);
            tx.send(Arc::new(Mutex::new(cutmarks))).unwrap();
        }
    });
}

fn autocutmarks_calibrate_near(
    acm_exe_path: &Path,
    videofile: &Path,
    start_frame: i64,
    threshold: u64,
) {
    let status = std::process::Command::new(acm_exe_path)
        .arg("--mode=calibrate-near")
        .arg(format!("--thresholdNear={}", threshold))
        .arg(format!("-s {}", start_frame))
        .arg(videofile)
        .arg("snaps.txt")
        .status()
        .unwrap();

    assert!(status.success());
}

fn autocutmarks_calibrate_far(
    acm_exe_path: &Path,
    videofile: &Path,
    start_frame: i64,
    threshold: u64,
) {
    let status = std::process::Command::new(acm_exe_path)
        .arg("--mode=calibrate-far")
        .arg(format!("--thresholdFar={}", threshold))
        .arg(format!("-s {}", start_frame))
        .arg(videofile)
        .arg("snaps.txt")
        .status()
        .unwrap();

    assert!(status.success());
}

fn read_cutmarks_file(fps: f32) -> Box<Cutmarks> {
    let cutmarks_file = File::open("snaps.txt").unwrap();
    let lines = io::BufReader::new(cutmarks_file).lines();
    let mut cutmarks = Box::new(Cutmarks::new());
    for line in lines.flatten() {
        let cutmark: u64 = line.parse().unwrap();
        let time = (cutmark as f32 / fps * 1000.0) as i64;
        cutmarks.insert(time);
    }

    cutmarks
}
