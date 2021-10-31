use std::{
    collections::btree_set::BTreeSet,
    fs::File,
    io::{self, BufRead},
    sync::{mpsc::Sender, Arc, Mutex},
};

use fltk::{dialog::FileDialogType, prelude::*};
use std::string::String;
use std::sync::mpsc::channel;

use std::convert::TryInto;

use vlc::{Instance, MediaPlayer, MediaPlayerVideoEx};

pub mod ffmpeg;
mod input;
use std::path::PathBuf;

mod action_handling;
use action_handling::ActionHandler;

#[cfg(target_os = "windows")]
use libc::c_void;

use crate::input::{controller::Controller, keyboard_fltk::action_from_pressed_key};

const CLIP_SUFFIX_OFFENSE: &str = "Off";
const CLIP_SUFFIX_DEFENSE: &str = "Def";

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum ClipType {
    Offense,
    Defense,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
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
impl Into<&str> for Action {
    fn into(self) -> &'static str {
        match self {
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

#[cfg(target_os = "windows")]
type WindowHandle = *mut c_void;

#[cfg(target_os = "linux")]
type WindowHandle = u64;

#[derive(Copy, Clone)]
enum GuiActions {
    ChooseACMExe,
    Analyze,
    AnalyzeCached,
    CalibrateNear,
    CalibrateFar,
    SetStartFrame,
    SetEndFrame,
    KeyEvent(fltk::enums::Key),
    SetMediaPosition(f64),
}

fn run_with_fltk() {
    let app = fltk::app::App::default().with_scheme(fltk::app::AppScheme::Gtk);
    let mut win = fltk::window::Window::new(100, 100, 800, 650, "Media Player");

    // Create inner window to act as embedded media player
    let mut vlc_win = fltk::window::Window::new(10, 10, 780, 520, "");
    vlc_win.end();
    vlc_win.set_color(fltk::enums::Color::Black);

    let gui_elements_start_x = 10;
    let gui_elements_start_y = vlc_win.y() + vlc_win.height() + 10;

    let (s, r) = fltk::app::channel::<GuiActions>();

    let mut start_frame_input =
        fltk::input::IntInput::new(10, gui_elements_start_y + 20, 100, 30, None);
    let mut start_frame_button =
        fltk::button::Button::new(10, gui_elements_start_y + 50, 100, 30, "set start frame");
    start_frame_button.emit(s, GuiActions::SetStartFrame);

    let mut end_frame_input =
        fltk::input::IntInput::new(110, gui_elements_start_y + 20, 100, 30, None);
    let mut end_frame_button =
        fltk::button::Button::new(110, gui_elements_start_y + 50, 100, 30, "set end frame");
    end_frame_button.emit(s, GuiActions::SetEndFrame);

    let mut slider = fltk::valuator::HorNiceSlider::new(30, gui_elements_start_y, 750, 15, None);
    let slider_sender = s.clone();
    slider.handle(move |w, event| match event {
        fltk::enums::Event::Drag => {
            let pos = w.value();
            slider_sender.send(GuiActions::SetMediaPosition(pos));
            true
        }

        fltk::enums::Event::Released => {
            w.clear_visible_focus();
            true
        }

        _ => false,
    });

    let mut button_acm_exe = fltk::button::Button::new(
        gui_elements_start_x + 300,
        gui_elements_start_y + 20,
        200,
        20,
        "Choose ACM Executable..",
    );
    button_acm_exe.emit(s, GuiActions::ChooseACMExe);

    let mut button_analyze = fltk::button::Button::new(
        gui_elements_start_x + 300,
        gui_elements_start_y + 50,
        200,
        20,
        "Analyze",
    );
    button_analyze.emit(s, GuiActions::Analyze);

    let mut button_analyze_cached = fltk::button::Button::new(
        gui_elements_start_x + 300,
        gui_elements_start_y + 90,
        200,
        20,
        "Analyze cached",
    );
    button_analyze_cached.emit(s, GuiActions::AnalyzeCached);

    let mut calib_near_input =
        fltk::input::IntInput::new(600, gui_elements_start_y + 20, 100, 30, None);
    calib_near_input.set_value("900");
    let mut calib_near_button =
        fltk::button::Button::new(600, gui_elements_start_y + 50, 100, 30, "Calibrate Near");
    calib_near_button.emit(s, GuiActions::CalibrateNear);

    let mut calib_far_input =
        fltk::input::IntInput::new(700, gui_elements_start_y + 20, 100, 30, None);
    calib_far_input.set_value("500");
    let mut calib_far_button =
        fltk::button::Button::new(700, gui_elements_start_y + 50, 100, 30, "Calibrate Far");
    calib_far_button.emit(s, GuiActions::CalibrateFar);

    win.make_resizable(true);
    //win.fullscreen(true);
    win.end();
    win.show();

    let (key_event_sender, key_event_receiver) = fltk::app::channel::<fltk::enums::Key>();
    win.handle(move |_w, ev| match ev {
        fltk::enums::Event::NoEvent => false, // happens on windows according to: https://docs.rs/fltk/1.2.3/fltk/app/fn.wait_for.html

        fltk::enums::Event::Close => {
            println!("FLTK main window closed, exiting");
            //TODO(obr): exit the application
            false
        }

        fltk::enums::Event::KeyUp => {
            let key = fltk::app::event_key();
            s.send(GuiActions::KeyEvent(key));
            true
        }

        _ => false,
    });

    let handle = vlc_win.raw_handle();

    start_vlc(
        Some((
            app,
            r,
            start_frame_input,
            end_frame_input,
            calib_near_input,
            calib_far_input,
            key_event_receiver,
            slider,
        )),
        Some(handle),
    );
}

fn start_vlc(
    mut fltk_app: Option<(
        fltk::app::App,
        fltk::app::Receiver<GuiActions>,
        fltk::input::IntInput,
        fltk::input::IntInput,
        fltk::input::IntInput,
        fltk::input::IntInput,
        fltk::app::Receiver<fltk::enums::Key>,
        fltk::valuator::HorNiceSlider,
    )>,
    render_window: Option<WindowHandle>,
) {
    let args: Vec<String> = std::env::args().collect();

    println!("args: {:?}", args);

    if args.len() < 2 {
        println!("Please specify a video file");
        println!("Usage: gac path_to_a_media_file");
        return;
    }

    let mut media_paths: Vec<PathBuf> = Vec::new();
    for arg in args[1..].iter() {
        let p = PathBuf::from(arg);
        if p.is_dir() {
            for dir_entry_result in p.read_dir().unwrap() {
                if let Ok(directory_entry) = dir_entry_result {
                    if let Some(s) = directory_entry.path().extension() {
                        if let Some(extension) = s.to_str() {
                            if VIDEO_EXTENSIONS.contains(&extension.to_uppercase().as_str()) {
                                media_paths.push(directory_entry.path());
                            }
                        }
                    }
                }
            }
            break;
        }
        media_paths.push(p);
    }

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

    if let Some(handle) = render_window {
        #[cfg(target_os = "windows")]
        mdp.set_hwnd(handle);

        #[cfg(target_os = "linux")]
        mdp.set_xwindow(handle.try_into().unwrap()); // TODO unchecked u64 -> u32 conversion

        // Disable event handling on vlc's side
        // Do it thru fltk
        mdp.set_key_input(false);
        mdp.set_mouse_input(false);
    } else {
        if mdp.get_fullscreen() == false {
            mdp.toggle_fullscreen();
        }
    }

    let mut action_handler = ActionHandler::new(&instance, mdp, &media_paths);

    // start playing
    action_handler.handle(Action::TogglePlayPause).unwrap();

    let mut acm_exe_path: Option<PathBuf> = None;

    let (tx_cutmarks_ready, rx_cutmarks_ready) = channel::<Arc<Mutex<Box<Cutmarks>>>>();

    loop {
        let event_happened = fltk::app::wait_for(0.01).unwrap();

        action_handler.check_loop_end();

        if let Ok(cutmark_mutex) = rx_cutmarks_ready.try_recv() {
            let guard = cutmark_mutex.lock().unwrap();
            let cutmarks = &*guard;
            action_handler.set_cutmarks(cutmarks)
        }

        if let Some((
            _app,
            gui_actions_receiver,
            ref mut start_frame_input,
            ref mut end_frame_input,
            ref mut calib_near_input,
            ref mut calib_far_input,
            key_event_receiver,
            ref mut slider,
        )) = fltk_app
        {
            if !slider.has_focus() {
                slider.set_value(action_handler.get_media_relative_position() as f64);
            }

            if event_happened {
                if let Some(gui_action) = gui_actions_receiver.recv() {
                    match gui_action {
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
                                let threshold_near: u64 = calib_near_input.value().parse().unwrap();
                                autocutmarks_calibrate_near(
                                    path,
                                    action_handler.get_current_media_path(),
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
                                let threshold_far: u64 = calib_far_input.value().parse().unwrap();
                                autocutmarks_calibrate_far(
                                    path,
                                    action_handler.get_current_media_path(),
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
                                    let start_frame_value = start_frame_input.value();
                                    let start_frame: Option<i64> = if start_frame_value.is_empty() {
                                        None
                                    } else {
                                        Some(start_frame_value.parse().unwrap())
                                        //TODO error handling
                                    };

                                    let end_frame_value = end_frame_input.value();
                                    let end_frame: Option<i64> = if end_frame_value.is_empty() {
                                        None
                                    } else {
                                        Some(end_frame_value.parse().unwrap()) //TODO error handling
                                    };
                                    analyze_autocutmarks(
                                        path,
                                        action_handler.get_current_media_path(),
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
                                analyze_autocutmarks_cached(
                                    path,
                                    action_handler.get_current_media_path(),
                                    action_handler.get_fps(),
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
                            start_frame_input.set_value(&s);
                        }

                        GuiActions::SetEndFrame => {
                            let end_frame = action_handler.get_current_frame();
                            let s = end_frame.to_string();
                            end_frame_input.set_value(&s);
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
    acm_exe_path: &PathBuf,
    videofile: &PathBuf,
    start_frame: Option<i64>,
    end_frame: Option<i64>,
    fps: f32,
    tx: Sender<Arc<Mutex<Box<Cutmarks>>>>,
) {
    let acm_exe_path = acm_exe_path.clone();
    let videofile = videofile.clone();
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
    acm_exe_path: &PathBuf,
    videofile: &PathBuf,
    fps: f32,
    tx: Sender<Arc<Mutex<Box<Cutmarks>>>>,
) {
    let acm_exe_path = acm_exe_path.clone();
    let videofile = videofile.clone();
    std::thread::spawn(move || {
        let status = std::process::Command::new(acm_exe_path)
            .arg("--mode=use-cached")
            .arg(videofile)
            .arg("snaps.txt")
            .status()
            .unwrap();

        if status.success() {
            let cutmarks = read_cutmarks_file(fps);
            tx.send(Arc::new(Mutex::new(cutmarks))).unwrap();
        }
    });
}

fn autocutmarks_calibrate_near(
    acm_exe_path: &PathBuf,
    videofile: &PathBuf,
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
    acm_exe_path: &PathBuf,
    videofile: &PathBuf,
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
    for line in lines {
        if let Ok(l) = line {
            let cutmark: u64 = l.parse().unwrap();
            let time = (cutmark as f32 / fps * 1000.0) as i64;
            cutmarks.insert(time);
        }
    }

    cutmarks
}
