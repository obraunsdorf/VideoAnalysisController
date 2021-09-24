use std::collections::btree_set::BTreeSet;

use std::string::{String};
use std::sync::mpsc::channel;
use fltk::*;

use std::convert::TryInto;

use vlc::{Instance, MediaPlayer, MediaPlayerVideoEx};

pub mod ffmpeg;
mod input;
use std::path::PathBuf;


mod action_handling;
use action_handling::ActionHandler;

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

fn run_with_fltk() {
    let app = app::App::default().with_scheme(app::AppScheme::Gtk);
    let mut win = window::Window::new(100, 100, 800, 600, "Media Player");

    // Create inner window to act as embedded media player
    let mut vlc_win = window::Window::new(10, 10, 780, 520, "");
    vlc_win.end();
    vlc_win.set_color(Color::Black);

    let (s, r) = app::channel::<Action>();

    let mut but_play = button::Button::new(320, 545, 80, 40, "Play/Pause");
    but_play.emit(s, Action::TogglePlayPause);
    
    let mut but_stop = button::Button::new(400, 545, 80, 40, "Stop");
    but_stop.emit(s, Action::Stop);

    win.end();
    win.show();
    win.make_resizable(true);

    let handle = vlc_win.raw_handle();

   start_vlc(Some((app, r)), Some(handle));
}

fn start_vlc(fltk_app: Option<(fltk::app::App, fltk::app::Receiver<Action>)>, render_window: Option<WindowHandle>) {
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

    //let instance = Instance::new().unwrap();
    let vlc_args: Vec<String> = vec![
        String::from("--verbose=1"),
        String::from("--file-logging"),
        String::from("--logfile=libvlc.log"),
    ];
    let instance = Instance::with_args(Some(vlc_args)).unwrap();
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
        println!("############## set window handle ################");
        // Disable event handling on vlc's side
        // Do it thru fltk
        //mdp.set_key_input(false);
        //mdp.set_mouse_input(false);
    } else {
        if mdp.get_fullscreen() == false {
            mdp.toggle_fullscreen();
        }
    }
    
    


    



       /* if let Some(app) = fltk_app {
        let (s, r) = app::channel::<Action>();
        while app.wait().unwrap() {
            match r.recv() {
                Some(val) => match val {
                    Action::TogglePlayPause => mdp.play().unwrap(),
                    Action::Stop => mdp.stop(),
                },
                None => (),
            }
        }
    }*/


    let mut action_handler = ActionHandler::new(&instance, mdp, &media_paths, cutmarks);
    
    //start playing
    action_handler.handle(Action::TogglePlayPause).unwrap();
    loop {
        //std::thread::sleep(Duration::from_millis(100));
        fltk::app::wait_for(0.01).unwrap();
        
        /* TODO(obr)!!!!: wieder einkommentieren
        if loop_end != -1 && mdp.get_time().unwrap() >= loop_end {
            mdp.set_time(loop_start);
        }*/

        if let Some((_app, receiver)) = fltk_app {
            if let Some(action) = receiver.recv() {
                if let Err(e) = action_handler.handle(action) {
                    println!("exiting because of {}", e);
                    break;
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
}
