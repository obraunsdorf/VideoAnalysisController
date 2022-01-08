use std::{collections::BTreeSet, iter::Cycle, path::PathBuf, process::Command};

use vlc::{MarqueeOption, MediaPlayer};

use crate::{ffmpeg, ClipType, Cutmarks, CLIP_SUFFIX_DEFENSE, CLIP_SUFFIX_OFFENSE};

use super::Action;

pub(super) struct ActionHandler<'vlc> {
    vlc_instance: &'vlc vlc::Instance,
    mdp: MediaPlayer,
    marquee_option: MarqueeOption,
    project_dir: PathBuf,
    media_iter: Cycle<std::vec::IntoIter<PathBuf>>,
    current_media_path: Option<PathBuf>,
    clips: BTreeSet<i64>,
    cutmarks: Option<Box<Cutmarks>>,
    loop_start: i64,
    loop_end: i64,
}

impl<'vlc> ActionHandler<'vlc> {
    pub(super) fn new(
        vlc_instance: &'vlc vlc::Instance,
        mdp: MediaPlayer,
        project_dir: PathBuf,
    ) -> Result<ActionHandler<'vlc>, std::io::Error> {
        let clips: BTreeSet<i64> = BTreeSet::new();

        // Initialize VLC Marquee -- maybe we don't need this anymore with FLTK
        let mut marquee_option: MarqueeOption = Default::default();
        marquee_option.position = Some(0);
        marquee_option.opacity = Some(70);
        marquee_option.timeout = Some(1000);

        let mut media_paths: Vec<PathBuf> = project_dir
            .read_dir()?
            .flat_map(|x| x)
            .filter(|entry| {
                if let Some(s) = entry.path().extension() {
                    if let Some(extension) = s.to_str() {
                        if super::VIDEO_EXTENSIONS.contains(&extension.to_uppercase().as_str()) {
                            return true;
                        }
                    }
                }
                return false;
            })
            .map(|entry| entry.path())
            .collect();

        media_paths.sort();
        let media_iter = media_paths.into_iter().cycle();
        let mut ah = ActionHandler {
            vlc_instance: vlc_instance,
            mdp: mdp,
            marquee_option,
            project_dir,
            media_iter,
            current_media_path: None,
            clips,
            cutmarks: None,
            loop_start: -1,
            loop_end: -1,
        };
        let next_media = ah.media_iter.next().unwrap();
        ah.play_media(&next_media);

        Ok(ah)
    }

    pub(super) fn set_project_directory(
        &mut self,
        dir_path: PathBuf,
    ) -> Result<(), std::io::Error> {
        let mut media_paths: Vec<PathBuf> = dir_path
            .read_dir()?
            .flat_map(|x| x)
            .filter(|entry| {
                if let Some(s) = entry.path().extension() {
                    if let Some(extension) = s.to_str() {
                        if super::VIDEO_EXTENSIONS.contains(&extension.to_uppercase().as_str()) {
                            return true;
                        }
                    }
                }
                return false;
            })
            .map(|entry| entry.path())
            .collect();

        media_paths.sort();
        let media_iter = media_paths.into_iter().cycle();
        self.project_dir = dir_path;
        self.media_iter = media_iter;
        let next_media = self.media_iter.next().unwrap();
        self.play_media(&next_media);

        Ok(())
    }

    fn play_media(&mut self, current_media_path: &PathBuf) {
        let md = vlc::Media::new_path(&self.vlc_instance, &current_media_path).unwrap();
        self.current_media_path = Some(current_media_path.clone());
        self.mdp.set_media(&md);
        self.loop_start = -1;
        self.loop_end = -1;
        self.mdp.play().unwrap();
    }

    pub(super) fn set_cutmarks(&mut self, cutmarks: &Box<Cutmarks>) {
        self.cutmarks = Some(cutmarks.clone());
    }

    pub(super) fn get_media_relative_position(&self) -> f32 {
        self.mdp.get_position().unwrap()
    }

    pub(super) fn set_media_relative_position(&self, pos: f32) {
        self.mdp.set_position(pos);
    }

    pub(super) fn check_loop_end(&self) {
        if self.loop_end != -1 && self.mdp.get_time().unwrap() >= self.loop_end {
            self.mdp.set_time(self.loop_start);
        }
    }

    pub(super) fn get_current_media_path(&self) -> Option<&PathBuf> {
        self.current_media_path.as_ref()
    }

    pub(super) fn get_fps(&self) -> f32 {
        //TODO check if media file is running and other safety checks
        unsafe { vlc::sys::libvlc_media_player_get_fps(self.mdp.raw()) }
    }

    pub(super) fn get_current_frame(&self) -> i64 {
        let time = self.mdp.get_time().unwrap() as f32; // in millisecons
        let fps = self.get_fps();
        let frame = time / 1000.0 * fps; // TODO: do we need to check for overflow with floats?

        frame as i64
    }

    pub(super) fn handle(&mut self, action: Action) -> Result<(), &'static str> {
        match action {
            Action::TogglePlayPause => {
                if self.mdp.is_playing() {
                    self.mdp.pause();
                } else {
                    self.mdp.play().unwrap();
                }
            }
            Action::Forward(speed) => {
                /*//self.mdp.pause();
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
                let cur_time = self.mdp.get_time().unwrap();
                self.mdp.set_time(cur_time + speed as i64 * 10);
                //self.mdp.pause();*/
                let new_time = self.mdp.get_time().unwrap() + (speed * 1000.0) as i64;
                self.mdp.set_time(new_time);
            }

            Action::Rewind(speed) => {
                let new_time = self.mdp.get_time().unwrap() - (speed * 1000.0) as i64;
                self.mdp.set_time(new_time);
            }

            Action::IncreaseSpeed => {
                let current_speed = self.mdp.get_rate();
                self.mdp.set_rate(current_speed + 0.1).unwrap();
            }

            Action::DecreaseSpeed => {
                let current_speed = self.mdp.get_rate();
                self.mdp.set_rate(current_speed - 0.1).unwrap();
            }

            Action::ConcatClips => {
                let mut clips_dir_path = self.project_dir.clone();
                clips_dir_path.push("_clips");
                if clips_dir_path.exists() == false {
                    std::fs::create_dir(&clips_dir_path).expect("unable to create directory");
                }

                let mut condensed_dir_path = self.project_dir.clone();
                condensed_dir_path.push("_condensed");
                std::fs::create_dir(&condensed_dir_path).unwrap();
                let result = ffmpeg::concat(&clips_dir_path, &condensed_dir_path);
                let msg = if let Err(e) = result {
                    println!("{}", e);
                    "error concatenating"
                } else {
                    "concatenating clips"
                };

                self.mdp
                    .show_marqee_text(&msg, &self.marquee_option)
                    .unwrap();
            }

            Action::CutCurrentLoop(o_d_option) => {
                self.clips.insert(self.loop_start);
                println!(
                    "cutting from {:?} to {:?}...",
                    self.loop_start, self.loop_end
                );

                let mut clips_dir_path = self.project_dir.clone();
                clips_dir_path.push("_clips");
                if clips_dir_path.exists() == false {
                    std::fs::create_dir(&clips_dir_path).expect("unable to create directory");
                }

                let mut extension = "";
                let current_media_path = self.current_media_path.as_ref().unwrap();
                if let Some(s) = current_media_path.extension() {
                    if let Some(ext) = s.to_str() {
                        extension = ext;
                    }
                }

                assert!(self.loop_start < i64::pow(10, 8));
                // 8 leading zeros to be able to store 24h.
                let mut out_file_name = format!(
                    "{}_{:0>8}",
                    current_media_path.file_name().unwrap().to_str().unwrap(),
                    self.loop_start.to_string()
                );
                let user_hint = if let Some(off_def) = o_d_option {
                    match off_def {
                        ClipType::Offense => {
                            out_file_name.push_str(CLIP_SUFFIX_OFFENSE);
                            " as Offense"
                        }
                        ClipType::Defense => {
                            out_file_name.push_str(CLIP_SUFFIX_DEFENSE);
                            " as Defense"
                        }
                    }
                } else {
                    ""
                };
                out_file_name = out_file_name + "." + extension;
                let out_file_path = clips_dir_path.join(out_file_name);

                assert!(self.loop_start >= 0 && self.loop_end > self.loop_start);

                let start = self.loop_start as f32 / 1000.0;
                let end = self.loop_end as f32 / 1000.0;
                let duration = end - start;

                if let Ok(child_proc) = Command::new("ffmpeg")
                    .arg("-ss")
                    .arg(format!("{}", start))
                    .arg("-i")
                    .arg(current_media_path)
                    .arg("-t")
                    .arg(format!("{}", duration))
                    .arg("-c")
                    .arg("copy")
                    .arg(out_file_path)
                    .spawn()
                {
                    let msg = "cut clip".to_owned() + user_hint;
                    self.mdp
                        .show_marqee_text(&msg, &self.marquee_option)
                        .unwrap();
                    println!("command executed: {:?}", child_proc);
                } else {
                    self.mdp
                        .show_marqee_text("error on creating clip", &self.marquee_option)
                        .unwrap();
                }

                self.loop_start = -1;
                self.loop_end = -1;
            }
            Action::StartLoop => {
                match self.mdp.get_time() {
                    Some(start) => {
                        self.loop_start = start;
                        if self.loop_start >= self.loop_end {
                            self.loop_end = -1;
                        }
                    }
                    None => println!("error getting time"),
                }
                self.mdp
                    .show_marqee_text("start loop", &self.marquee_option)
                    .unwrap();
                println!("set loop start at {:?}", self.loop_start)
            }

            Action::PreviousClip => {
                if self.clips.len() == 0 {
                    self.handle(Action::PreviousMedia)?;
                } else {
                    let cur_time = self.mdp.get_time().unwrap();

                    let mut iter = self.clips.iter().rev();
                    while let Some(clip) = iter.next() {
                        if clip <= &cur_time {
                            if let Some(prev_clip) = iter.next() {
                                self.mdp.set_time(*prev_clip);
                                println!("previous clip from {}", *prev_clip);
                            } else {
                                self.mdp.set_time(*clip);
                                println!("previous clip from {}", *clip);
                            }
                            break;
                        }
                    }
                }
            }

            Action::NextClip => {
                if self.clips.len() == 0 {
                    self.handle(Action::NextMedia)?;
                } else {
                    let cur_time = self.mdp.get_time().unwrap();
                    for clip in &mut self.clips.iter() {
                        if clip >= &cur_time {
                            self.mdp.set_time(*clip);
                            println!("jumping to clip {}", *clip);
                            break;
                        }
                    }
                }
            }

            Action::RestartClip => {
                if self.clips.len() == 0 {
                    self.handle(Action::RestartMedia)?
                } else {
                    let cur_time = self.mdp.get_time().unwrap();
                    for clip in &mut self.clips.iter().rev() {
                        if clip <= &cur_time {
                            self.mdp.set_time(*clip);
                            println!("restarting clip from to {}", *clip);
                            break;
                        }
                    }
                }
            }

            Action::PreviousCutmark => {
                let cur_time = self.mdp.get_time().unwrap();

                if let Some(cutmarks) = &self.cutmarks {
                    let mut iter = cutmarks.iter().rev();
                    while let Some(cutmark) = iter.next() {
                        if cutmark <= &cur_time {
                            if let Some(prev_cutmark) = iter.next() {
                                self.mdp.set_time(*prev_cutmark);
                                println!("previous cutmark from {}", *prev_cutmark);
                            } else {
                                self.mdp.set_time(*cutmark);
                                println!("previous cutmark from {}", *cutmark);
                            }
                            self.mdp
                                .show_marqee_text("Previous Cutmark", &self.marquee_option)
                                .unwrap();
                            //self.mdp.play();
                            //tx.send(Action::TogglePlayPause).unwrap();
                            break;
                        }
                    }
                }
            }

            Action::NextCutmark => {
                let cur_time = self.mdp.get_time().unwrap();
                if let Some(cutmarks) = &self.cutmarks {
                    for cutmark in &mut cutmarks.iter() {
                        if cutmark > &cur_time {
                            self.mdp.set_time(*cutmark);
                            self.mdp.play().unwrap();
                            //tx.send(Action::TogglePlayPause).unwrap();
                            println!("jumping to cutmark {}", *cutmark);
                            self.mdp
                                .show_marqee_text("Next Cutmark", &self.marquee_option)
                                .unwrap();
                            break;
                        }
                    }
                }
            }

            Action::EndLoop => {
                match self.mdp.get_time() {
                    Some(end) => {
                        self.loop_end = end;
                        if self.loop_end <= self.loop_start {
                            self.loop_start = -1;
                        }
                    }

                    None => println!("error getting time"),
                }
                println!("set loop end at {:?}", self.loop_end);
                self.mdp
                    .show_marqee_text("end loop", &self.marquee_option)
                    .unwrap();
                self.mdp.set_time(self.loop_start);
                //check_self.loop_end(&tx, self.mdp, self.loop_start, self.loop_end);
            }

            /* Action::CheckLoopEnd(pos) => {
                let duration = self.mdp.get_media().unwrap().duration().unwrap();
                let cur_time = (duration as f64 * pos as f64) as i64;
                println!("checking loop end: cur_time={:?} self.loop_end={:?}", cur_time, self.loop_end);
                if cur_time >= self.loop_end {
                    println!("yes");
                    tx.send(Action::PreviousClip);
                } else {
                    println!("no");
                }
            }*/
            Action::BreakLoop => {
                self.mdp
                    .show_marqee_text("break loop", &self.marquee_option)
                    .unwrap();
                self.loop_end = -1;
            }

            Action::Stop | Action::NextMedia => {
                if let Some(media_path) = self.media_iter.next() {
                    self.play_media(&media_path);
                }
            }

            Action::PreviousMedia => {
                println!("playing media previous to {:?}", self.current_media_path);
                if let Some(current_media_path) = &self.current_media_path {
                    let mut previous = self.media_iter.next().unwrap();
                    while let Some(media) = self.media_iter.next() {
                        if media == *current_media_path {
                            self.play_media(&previous);
                            break;
                        }
                        previous = media;
                    }
                }
            }

            Action::RestartMedia => {
                self.mdp.set_time(0);
            }

            Action::Exit => return Err("No real error. Just exiting"),
        };

        Ok(())
    }
}
