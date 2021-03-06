VideoAnalysisController
======================
[![Build status](https://ci.appveyor.com/api/projects/status/ijnogdj1n1cj673x?svg=true)](https://ci.appveyor.com/project/obraunsdorf/videoanalysiscontroller) 

VideoAnalysisController (VAC) is a VLC-based video player with focus on sports video analysis. It is written to be controlled by any Game Controller similar to Xbox / Playstation.
You can use it to

 - Analyze your game footage
 - Loop a specific play
 - Watch it in slow motion or do fast-forward/rewind
 - Cut it into clips (optionally tagged with "Offense" / "Defense") 

VAC is primarily dedicated to the flag football community as most of the teams (at least in Germany) cannot afford a commercial tool.

VideoAnalysisController is distributed for Windows (only 64-Bit at the moment), but can also be built for Linux.


## How to Install
Installation files for VAC are currently only provided **for 64-Bit Windows** (tested on Windows 10). Follow the instructions below, to install VAC for Windows.
1. Download the latest release of the Windows Setup at https://github.com/obraunsdorf/VideoAnalysisController/releases
2. Install VLC Media Player version 3.0.6 for 64-Bit Windows: https://get.videolan.org/vlc/3.0.6/win64/vlc-3.0.6-win64.exe
3. In Windows' system settings: modify the PATH environment variable to add the VLC installation directory (most likely "C:\Program Files\VideoLAN\VLC") 

**For Linux and MacOS** you can build the software yourself or submit an issue here at Github if you need really need pre-compiled installation files and cannot build VAC on your own for your OS.
## How to Use
### Controls
 - For the button mapping on gamepads (tested with XBox One) see [src/input/controller.rs](src/input/controller.rs)
 - For the mapping on keyboards, you can look at and modify  [keymap.toml](keymap.toml). Possible key identifiers can be found in [src/input/keyboard.rs](src/input/keyboard.rs)
### Opening videos
 - To start one video in VAC, right-click on the video -> open with -> select the VAC executable (most likely C:\Program Files\VideoAnalysisController\VideoAnalysisController.exe)

 - To start multiple videos in one directory, open the windows command line and type 
```
"C:\Program Files\VideoAnalysisController\VideoAnalysisController.exe" Path\To\Directory
```
### Cutting videos
To cut a sequence out of a video...
 1. Press `StartLoop` to set a starting point.
 2. Press `EndLoop` to set a end point.
 3. Press `CutLoop` to generate the video clip. You can optionally tag video clips as _Offense_ or _Defense_ by pressing `CutLoop_Offense` or `CutLoop_Defense` respectively.
 
A new directory `<videofilename>_clips` will be created containing all cutted videos named with a timestamp to reconstruct their order in the original video file. Offense clips have their file names suffixed with "Off", Defense clips are suffixed with "Def".
 
 ### Concatenating videos
 To concatenate all videos in this directory, press `ConcatClips`.  
 This will create a new directory `<videofilename>_condensed` containing 3 files: 
  - one video consisting of concatenated offense clips
  - one video consisting of concatenated defense clips
  - one video consisiting of all clips


## How to Build
 1. Clone this repository
 2. Get the nightly rust toolchain
 3. Download and unzip VLC (<https://ftp.fau.de/videolan/vlc/3.0.6/win64/vlc-3.0.6-win64.7z>) to the cloned repository folder
 4. Download ffmpeg (<https://ffmpeg.zeranoe.com/builds/win64/static/ffmpeg-4.1.3-win64-static.zip>) and extract the ffmpeg.exe to the cloned repository folder
 5. To test, execute `cargo test`
 6. To run, execute `cargo run -- <videofilename>`


## How to Contribute
If you have any concrete idea, bug fixes, or even new code, you can either send me an email at <oliver.braunsdorf@gmx.de> or you can create an issue at <https://github.com/obraunsdorf/playbook-creator/issues>

If you are used to the github workflow, I am happy about every fork and pull request ;)
