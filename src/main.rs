#![feature(duration_float)]


use std::sync::mpsc::channel;
use std::process::Command;
use std::string::{ToString, String};
use std::collections::btree_set::BTreeSet;

use vlc::{Instance, Media, MediaPlayer, Event, EventType, State, MediaPlayerVideoEx};


mod input;
use std::path::{Path, PathBuf};

pub enum ClipOf_O_D {
    Offense,
    Defense,
}

pub enum MoveZoomAxis {
    Horizontal,
    Vertical,
}

pub enum Action {
    TogglePlayPause,
    Rewind(f32),
    Forward(f32),
    IncreaseSpeed,
    DecreaseSpeed,
    StartLoop,
    EndLoop,
    BreakLoop,
    CheckLoopEnd(f32),
    CutCurrentLoop(Option<ClipOf_O_D>),
    NextMedia,
    PreviousMedia,
    RestartMedia,
    NextClip,
    PreviousClip,
    RestartClip,
    Zoom(f32),
    MoveZoom(MoveZoomAxis, f32),
    Stop,
    Exit,
}


static MAX_SPEED: f32 = 16.0;
static BREAKPOINT: f32 = 0.5;

const VIDEO_EXTENSIONS: &[&str] = &["MOV", "MPEG", "MP4"];


/*fn check_loop_end(tx_orig: &std::sync::mpsc::Sender<Action>,
                  mdp: &MediaPlayer,
                  loop_start: i64,
                  loop_end: i64
) {
    let tx = tx_orig.clone();
    std::thread::spawn(|| {
        loop
    })
}*/

fn load_media(vlc_instance: &vlc::Instance, path: &Path, tx_0: &std::sync::mpsc::Sender<Action>)
              -> Media {
    //let md = Media::new_location(&instance, "https://www.youtube.com/watch?v=M3Q8rIgveO0").unwrap();
    let tx = tx_0.clone();
    let tx_2 = tx_0.clone();
    let md = Media::new_path(&vlc_instance, path).unwrap();
    let em = md.event_manager();
    let _ = em.attach(EventType::MediaStateChanged, move |e, _| {
        match e {
            Event::MediaStateChanged(s) => {
                println!("State : {:?}", s);
                match s {
                    State::Ended => tx.send(Action::Stop).unwrap(),
                    State::Error => tx.send(Action::Exit).unwrap(),
                    _ => {}
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

    return md;
}


struct Zoom<'player> {
    mdp: &'player vlc::MediaPlayer,
    max_width: u32,
    max_height: u32,
    zoom_width: u32,
    zoom_height: u32,
    zoom_posx: u32,
    zoom_posy: u32
}
pub enum ZoomUpdateKind {
    WindowWidth(i16), // zoom to the width of ...
    PosX(i16), // move the zoom window along horizontal axis by ...
    PosY(i16), // move the zoom window along vertical axis by ...
}

impl<'player> Zoom<'player> {

    pub fn new(mdp: & vlc::MediaPlayer, resolution: (u32, u32)) -> Zoom {
        let (max_width, max_heigth) = resolution;
        return Zoom {
            mdp,
            max_width,
            max_height: max_heigth,
            zoom_width: max_width,
            zoom_height: max_heigth,
            zoom_posx : 0,
            zoom_posy : 0
        }
    }

    pub fn updateZoom(&mut self, kind: ZoomUpdateKind) {
        if let Ok(geometry_string) = self.mdp.get_video_crop_geometry() {
            println!("geometry string = {:?}", geometry_string);
        }

        if let ZoomUpdateKind::WindowWidth(delta) = kind {
            let aspect_ratio = 16.0 / 9.0;
            let zoom_delta = delta * 100; //TODO: convert good
            println!("current zoom_widht={:?}, zoom delta={:?}", self.zoom_width, zoom_delta);
            if zoom_delta as i32 > self.zoom_width as i32  {

            } else {
                self.zoom_width = self.max_width.min((self.zoom_width as i32 - zoom_delta as i32) as u32);
            }
            self.zoom_height = (self.zoom_width as f32 / aspect_ratio) as u32;
        }

        let max_move_x : u32 = self. max_width as u32 - self.zoom_width as u32;
        let max_move_y : u32 = self.max_height as u32 - self.zoom_height as u32;

        match kind {
            ZoomUpdateKind::PosX(move_delta) => {
                if (self.zoom_posx as i32 + move_delta as i32) < 0 {
                    // leave zoom_posx unmodified
                } else {
                    self.zoom_posx = max_move_x.min((self.zoom_posx as i32 + move_delta as i32) as u32)
                }
            }

            ZoomUpdateKind::PosY(move_delta) => {
                if (self.zoom_posy as i32 + move_delta as i32) < 0 {
                    // leave zoom_posy unmodified
                } else {
                    self.zoom_posy = max_move_y.min((self.zoom_posy as i32 + move_delta as i32) as u32)
                }
            }

            ZoomUpdateKind::WindowWidth(_) => {}
        };

        let geometry_string = format!("{:?}x{:?}+{:?}+{:?}", self.zoom_width, self.zoom_height, self.zoom_posy, self.zoom_posx);
        match self.mdp.set_video_crop_geometry(&geometry_string) {
            Err(e) => println!("error setting geometry: {:?}", e),
            _ => {}
        }

        if let Ok(geometry_string) = self.mdp.get_video_crop_geometry() {
            println!("geometry string = {:?}", geometry_string);
        }
        println!("after x={:?} y={:?}", self.zoom_posx, self.zoom_posy);
    }

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

    let instance = Instance::new().unwrap();
    /*let vlc_args: Vec<String> = vec![
        String::from("--verbose=2")
    ];
    let instance = Instance::with_args(Some(vlc_args)).unwrap();*/
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
    mdp.play();

    let mut loop_start: i64 = -1;
    let mut loop_end: i64 = -1;
    let mut clipcount = 0;

    let mut clips: BTreeSet<i64> = BTreeSet::new();

    let mut max_width: u32 = 1920;
    let mut max_height: u32 = 1080;
    if let Some((width, height)) = mdp.get_size(0) {
        max_width = width;
        max_height = height;
    }
    let mut zoom = Zoom::new(&mdp, (max_width, max_height));

    loop {
        if loop_end != -1 && mdp.get_time().unwrap() >= loop_end {
            mdp.set_time(loop_start);
        }
        let result = rx.recv();
        if result.is_err() {
            println!("VAC Error: connection to controller or keyboard has been lost.");
            break;
        }
        let action = result.unwrap();
        match action {
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
                if let Some(off_def) = o_d_option {
                    match off_def {
                        ClipOf_O_D::Offense => out_file_name.push_str("Off"),
                        ClipOf_O_D::Defense => out_file_name.push_str("Def"),
                    }
                }
                out_file_name = out_file_name + "." + extension;
                let out_file_path = clips_dir_path.join(out_file_name);

                assert!(loop_start >= 0 && loop_end > loop_start);

                let start = loop_start as f32 / 1000.0;
                let end = loop_end as f32 / 1000.0;
                let duration = end - start;

                let child_proc = Command::new("ffmpeg")
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
                    .expect("failed to execute vlc app");

                println!("command executed: {:?}", child_proc);

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
                    None => println!("error getting time")
                }
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

                    None => println!("error getting time")
                }
                println!("set loop end at {:?}", loop_end);
                mdp.set_time(loop_start);
                //check_loop_end(&tx, &mdp, loop_start, loop_end);
            }

            Action::BreakLoop => {
                loop_end = -1;
            }

            Action::Stop |
            Action::NextMedia => {
                path = media_iter.next().unwrap();
                md = load_media(&instance, path, &tx);
                mdp.set_media(&md);
                mdp.play();
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
                        mdp.play();
                        break;
                    }
                    previous = media;
                }
            }

            Action::MoveZoom(axis, pos) => {
                let move_delta =  (pos * 100.0) as i16;
                match axis {
                    MoveZoomAxis::Horizontal => {
                        zoom.updateZoom(ZoomUpdateKind::PosX(move_delta))
                    }
                    MoveZoomAxis::Vertical => {
                        zoom.updateZoom(ZoomUpdateKind::PosY(move_delta))
                    }
                }
            }

            Action::Zoom(pos) => {
                /*let zoomfactor: f32 = 0.1;
                if pos <= 0.0 {
                    cur_zoom = cur_zoom - zoomfactor;
                } else {
                    cur_zoom = cur_zoom + zoomfactor;
                }
                println!("setting zoom to {:?}", cur_zoom);
                mdp.set_scale(cur_zoom);*/

                let zoom_delta = (pos * 100.0) as i16;
                zoom.updateZoom(ZoomUpdateKind::WindowWidth(zoom_delta));
            }

            Action::RestartMedia => {
                mdp.set_media(&md);
                mdp.play();
            }

            Action::Exit => {
                break;
            }
            _ => {}
        };
    }
    println!("exiting");
}


