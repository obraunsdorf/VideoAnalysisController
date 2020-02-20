use std::collections::btree_set::BTreeSet;
use std::process::Command;
use std::string::{String, ToString};
use std::sync::mpsc::channel;

use vlc::{
    Event, EventType, Instance, MarqueeOption, Media, MediaPlayer, MediaPlayerVideoEx, State,
};

pub mod ffmpeg;
mod input;
use std::path::{Path, PathBuf};
use std::time::Duration;

const CLIP_SUFFIX_OFFENSE: &str = "Off";
const CLIP_SUFFIX_DEFENSE: &str = "Def";

#[derive(Debug, Clone)]
pub enum ClipOf_O_D {
    Offense,
    Defense,
}

#[derive(Debug, Clone)]
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
    CutCurrentLoop(Option<ClipOf_O_D>),
    NextMedia,
    PreviousMedia,
    RestartMedia,
    NextClip,
    PreviousClip,
    RestartClip,
    ConcatClips,
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
                Some(ClipOf_O_D::Offense) => "CutLoop_Offense",
                Some(ClipOf_O_D::Defense) => "CutLoop_Defense",
                None => "CutLoop",
            },
            Action::NextMedia => "NextMedia",
            Action::PreviousMedia => "PreviousMedia",
            Action::RestartMedia => "RestartMedia",
            Action::NextClip => "NextClip",
            Action::PreviousClip => "PreviousClip",
            Action::RestartClip => "RestartClip",
            Action::ConcatClips => "ConcatClips",
            Action::Stop => "Stop",
            Action::Exit => "Exit",
        }
    }
}

static MAX_SPEED: f32 = 16.0;
static BREAKPOINT: f32 = 0.5;

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

fn load_media(
    vlc_instance: &vlc::Instance,
    path: &Path,
    tx_0: &std::sync::mpsc::Sender<Action>,
) -> Media {
    //let md = Media::new_location(&instance, "https://www.youtube.com/watch?v=M3Q8rIgveO0").unwrap();
    let tx = tx_0.clone();
    let tx_2 = tx_0.clone();
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

fn main() {
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
    let mut media_iter = media_paths.iter().cycle();
    let mut path = media_iter.next().unwrap();

    let (tx, rx) = channel::<Action>();

    input::spawn_input_threads_with_sender(&tx);

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
    let mut md = load_media(&instance, path, &tx);
    mdp.set_media(&md);
    if mdp.get_fullscreen() == false {
        mdp.toggle_fullscreen();
    }
    mdp.play().unwrap();

    let mut loop_start: i64 = -1;
    let mut loop_end: i64 = -1;
    let mut clipcount = 0;

    let mut clips: BTreeSet<i64> = BTreeSet::new();

    let mut marquee_option: MarqueeOption = Default::default();
    marquee_option.position = Some(0);
    marquee_option.opacity = Some(70);
    marquee_option.timeout = Some(1000);

    loop {
        std::thread::sleep(Duration::from_millis(100));
        if loop_end != -1 && mdp.get_time().unwrap() >= loop_end {
            mdp.set_time(loop_start);
        }
        let result = rx.try_recv();
        if result.is_err() {
            continue;
            //println!("VAC Error: connection to controller or keyboard has been lost.");
            //break;
        }
        let action = result.unwrap();
        match action {
            Action::TogglePlayPause => {
                if mdp.is_playing() {
                    mdp.pause();
                } else {
                    mdp.play().unwrap();
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
                let new_time = mdp.get_time().unwrap() + (speed * 1000.0) as i64;
                mdp.set_time(new_time);
            }

            Action::Rewind(speed) => {
                let new_time = mdp.get_time().unwrap() - (speed * 1000.0) as i64;
                mdp.set_time(new_time);
            }

            Action::IncreaseSpeed => {
                let current_speed = mdp.get_rate();
                mdp.set_rate(current_speed + 0.1);
            }

            Action::DecreaseSpeed => {
                let current_speed = mdp.get_rate();
                mdp.set_rate(current_speed - 0.1);
            }

            Action::ConcatClips => {
                let s = String::from(path.to_str().unwrap()) + "_clips";
                let clips_dir_path = Path::new(s.as_str());
                if clips_dir_path.exists() == false {
                    std::fs::create_dir(&clips_dir_path).expect("unable to create directory");
                }

                let s2 = path.to_str().unwrap().to_string() + "_condensed";
                let condensed_dir_path = Path::new(s2.as_str());
                std::fs::create_dir(&condensed_dir_path);
                let result = ffmpeg::concat(clips_dir_path, condensed_dir_path);
                let msg = if let Err(e) = result {
                    println!("{}", e);
                    "error concatenating"
                } else {
                    "concatenating clips"
                };

                mdp.show_marqee_text(&msg, &marquee_option);
            }

            Action::CutCurrentLoop(o_d_option) => {
                clips.insert(loop_start);
                println!("cutting from {:?} to {:?}...", loop_start, loop_end);

                let s = String::from(path.to_str().unwrap()) + "_clips";
                let clips_dir_path = Path::new(s.as_str());
                if clips_dir_path.exists() == false {
                    std::fs::create_dir(&clips_dir_path).expect("unable to create directory");
                }

                let mut extension = "";
                if let Some(s) = path.extension() {
                    if let Some(ext) = s.to_str() {
                        extension = ext;
                    }
                }
                let mut out_file_name = loop_start.to_string();
                let mut user_hint = "";
                if let Some(off_def) = o_d_option {
                    match off_def {
                        ClipOf_O_D::Offense => {
                            out_file_name.push_str(CLIP_SUFFIX_OFFENSE);
                            user_hint = " as Offense"
                        }
                        ClipOf_O_D::Defense => {
                            out_file_name.push_str(CLIP_SUFFIX_DEFENSE);
                            user_hint = " as Defense"
                        }
                    }
                }
                out_file_name = out_file_name + "." + extension;
                let out_file_path = clips_dir_path.join(out_file_name);

                assert!(loop_start >= 0 && loop_end > loop_start);

                let start = loop_start as f32 / 1000.0;
                let end = loop_end as f32 / 1000.0;
                let duration = end - start;

                if let Ok(child_proc) = Command::new("ffmpeg")
                    .arg("-ss")
                    .arg(format!("{}", start))
                    .arg("-i")
                    .arg(path)
                    .arg("-t")
                    .arg(format!("{}", duration))
                    .arg("-c")
                    .arg("copy")
                    .arg(out_file_path)
                    .spawn()
                {
                    let msg = "cut clip".to_owned() + user_hint;
                    mdp.show_marqee_text(&msg, &marquee_option);
                    println!("command executed: {:?}", child_proc);
                } else {
                    mdp.show_marqee_text("error on creating clip", &marquee_option);
                }

                loop_start = -1;
                loop_end = -1;
            }
            Action::StartLoop => {
                match mdp.get_time() {
                    Some(start) => {
                        loop_start = start;
                        if loop_start >= loop_end {
                            loop_end = -1;
                        }
                    }
                    None => println!("error getting time"),
                }
                mdp.show_marqee_text("start loop", &marquee_option);
                println!("set loop start at {:?}", loop_start)
            }

            Action::PreviousClip => {
                if clips.len() == 0 {
                    tx.send(Action::PreviousMedia).unwrap();
                }
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
                if clips.len() == 0 {
                    tx.send(Action::NextMedia).unwrap();
                }
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
                if clips.len() == 0 {
                    tx.send(Action::RestartMedia).unwrap();
                }
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

                    None => println!("error getting time"),
                }
                println!("set loop end at {:?}", loop_end);
                mdp.show_marqee_text("end loop", &marquee_option);
                mdp.set_time(loop_start);
                //check_loop_end(&tx, mdp, loop_start, loop_end);
            }

            /* Action::CheckLoopEnd(pos) => {
                let duration = mdp.get_media().unwrap().duration().unwrap();
                let cur_time = (duration as f64 * pos as f64) as i64;
                println!("checking loop end: cur_time={:?} loop_end={:?}", cur_time, loop_end);
                if cur_time >= loop_end {
                    println!("yes");
                    tx.send(Action::PreviousClip);
                } else {
                    println!("no");
                }
            }*/
            Action::BreakLoop => {
                mdp.show_marqee_text("break loop", &marquee_option);
                loop_end = -1;
            }

            Action::Stop | Action::NextMedia => {
                path = media_iter.next().unwrap();
                md = load_media(&instance, path, &tx);
                mdp.set_media(&md);
                mdp.play().unwrap();
            }

            Action::PreviousMedia => {
                println!("playing media previous to {:?}", path);
                let mut previous = media_iter.next().unwrap();
                while let Some(media) = media_iter.next() {
                    if media == path {
                        path = previous;
                        println!("previous is {:?}", path);
                        let md = load_media(&instance, path, &tx);
                        mdp.set_media(&md);
                        mdp.play().unwrap();
                        break;
                    }
                    previous = media;
                }
            }

            Action::RestartMedia => {
                mdp.set_media(&md);
                mdp.play().unwrap();
            }

            Action::Exit => {
                break;
            }
        };
    }
    println!("exiting");
}
