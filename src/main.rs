use std::collections::btree_set::BTreeSet;

use fltk::{dialog::{FileChooserType, FileDialogType}, prelude::*};
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
    CalibrateNear,
    CalibrateFar,
    SetStartFrame,
    SetEndFrame,
}

fn run_with_fltk() {
    let app = fltk::app::App::default().with_scheme(fltk::app::AppScheme::Gtk);
    let mut win = fltk::window::Window::new(100, 100, 800, 600, "Media Player");
    //win.fullscreen(true);

    // Create inner window to act as embedded media player
    let mut vlc_win = fltk::window::Window::new(10, 10, 780, 520, "");
    vlc_win.end();
    vlc_win.set_color(fltk::enums::Color::Black);

    let (s, r) = fltk::app::channel::<GuiActions>();

    let mut button_acm_exe = fltk::button::Button::new(180, 545, 200, 40, "Choose ACM Executable..");
    button_acm_exe.emit(s, GuiActions::ChooseACMExe);
  
    let mut button_analyze = fltk::button::Button::new(420, 545, 80, 40, "Analyze");
    button_analyze.emit(s, GuiActions::Analyze);

    let mut button_calibrate_near = fltk::button::Button::new(500, 545, 80, 40, "Calibrate Near");
    button_calibrate_near.emit(s, GuiActions::CalibrateNear);

    let mut start_frame_input = fltk::input::IntInput::new(0, 545, 50, 40, "start frame");
    let mut start_frame_button = fltk::button::Button::new(0, 585, 50, 40, "set start frame");
    start_frame_button.emit(s, GuiActions::SetStartFrame);

    win.end();
    win.show();
    win.make_resizable(true);

    let handle = vlc_win.raw_handle();

    start_vlc(Some((app, r, start_frame_input)), Some(handle));
}

fn start_vlc(
    mut fltk_app: Option<(fltk::app::App, fltk::app::Receiver<GuiActions>, fltk::input::IntInput)>,
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
    let (tx, rx) = channel::<Action>();

    input::spawn_input_threads_with_sender(&tx);

    let cutmarks: BTreeSet<i64> = {
        //TODO: execute AutoCutMarks and read from result file
        let _cutmark_frames = vec![255, 1057, 2222];
        /*let fps = 30;
        let cutmark_times = cutmark_frames.iter().map(move |frame| frame*fps);*/

        let mut set = BTreeSet::new();
        set.insert(5102);
        set.insert(44069);
        set.insert(66322);
        set.insert(94823);

        set
    };

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

    let mut action_handler = ActionHandler::new(&instance, mdp, &media_paths, cutmarks);

    // start playing
    action_handler.handle(Action::TogglePlayPause).unwrap();

    let mut acm_exe_path: Option<PathBuf> = None; 

    loop {
        //std::thread::sleep(Duration::from_millis(100));
        let event_happened = fltk::app::wait_for(0.01).unwrap();
        if event_happened {
            let event = fltk::app::event();
            match event {
                fltk::enums::Event::NoEvent => {}, // happens on windows according to: https://docs.rs/fltk/1.2.3/fltk/app/fn.wait_for.html
                fltk::enums::Event::Close => {
                    println!("FLTK main window closed, exiting");
                    break;
                }
                _ => {}
            }
        }
            
        /*
        let shortcut = fltk::app::event_state();
        let a_key = fltk::enums::Key::from_char('a');
        let a_short = fltk::enums::Shortcut::from_char('a');
        println!("key for a looks like: {:?}", a_key);
        println!("shortcut for a looks like: {:?}", a_short);
        println!("shortcut: {:?}", shortcut);*/



        /* TODO(obr)!!!!: wieder einkommentieren
        if loop_end != -1 && mdp.get_time().unwrap() >= loop_end {
            mdp.set_time(loop_start);
        }*/

        if let Some((_app, receiver, ref mut start_frame_input)) = fltk_app {
            if let Some(gui_action) = receiver.recv() {
                match gui_action {
                    GuiActions::ChooseACMExe => {
                        let mut acm_exe_chooser = fltk::dialog::FileDialog::new(FileDialogType::BrowseFile);
                        acm_exe_chooser.show();
                        acm_exe_path = Some(acm_exe_chooser.filename());  
                    }

                    GuiActions::Analyze => {
                        match acm_exe_path {
                            Some(ref path) => {
                                let start_frame: i64 = start_frame_input.value().parse().unwrap(); //TODO error handling
                                analyze_autocutmarks(path, action_handler.get_current_media_path(), start_frame);
                            }

                            None => {
                                println!("Executable for AutoCutMarks was not set");
                            }
                        }
                        
                    }

                    GuiActions::SetStartFrame => {
                        let start_frame = action_handler.get_current_frame();
                        let s = start_frame.to_string();
                        start_frame_input.set_value(&s);
                    }

                    _ => {}
                }
            }
        }

        let result = rx.try_recv();
        if result.is_err() {
            continue;
            //println!("VAC Error: connection to controller or keyboard has been lost.");
            //break;
        }
        let action = result.unwrap();

        if let Err(e) = action_handler.handle(action) {
            println!("exiting because of: {}", e);
            break;
        }
    }

    println!("exiting");
    std::process::exit(0);
}

fn analyze_autocutmarks(acm_exe_path: &PathBuf, videofile: &PathBuf, start_frame: i64) {
    let end_frame = start_frame + 200;
    let status = 
        std::process::Command::new("python3")
        .arg(acm_exe_path)
        .arg(format!("-s {}", start_frame))
        .arg(format!("-e {}", end_frame))
        .arg(videofile)
        .arg("snaps.txt")
        .status().unwrap();
    assert!(status.success() == true);  // TODO: error handling
}