//use crossterm::{RawScreen, AsyncReader, InputEvent, KeyEvent, SyncReader};

use super::super::Action as Action;
use super::super::ClipOf_O_D as ClipOf_O_D;


fn get_action(key: char) -> Option<Action> {
    match key {
        'a' => Some(Action::TogglePlayPause),
        'b' => Some(Action::Rewind(0.7)),
        'c' => Some(Action::Forward(0.7)),
        'd' => Some(Action::IncreaseSpeed),
        'e' => Some(Action::DecreaseSpeed),
        'f' => Some(Action::StartLoop),
        'g' => Some(Action::EndLoop),
//'h' => Some(Action::CheckLoopEnd(f32)),
        'i' => Some(Action::CutCurrentLoop(Some(ClipOf_O_D::Offense))),
        'j' => Some(Action::CutCurrentLoop(Some(ClipOf_O_D::Defense))),
        'k' => Some(Action::NextMedia),
        'l' => Some(Action::PreviousMedia),
        'm' => Some(Action::RestartMedia),
        'n' => Some(Action::NextClip),
        'o' => Some(Action::PreviousClip),
        'p' => Some(Action::RestartClip),
        'q' => Some(Action::Stop),
        'r' => Some(Action::Exit),
        _ => None
    }
}

pub fn read_keyboard2(tx: std::sync::mpsc::Sender<Action>) {
    println!("starting to read");
    /*let mut el = glutin::EventsLoop::new();
    let wb = glutin::WindowBuilder::new()
        .with_title("Hello world!")
        .with_dimensions(glutin::dpi::LogicalSize::new(1024.0, 768.0));
    let windowed_context = glutin::ContextBuilder::new()
        .build_windowed(wb, &el)
        .unwrap();

    el.poll_events(|event| {
        println!("{:?}", event);
    })*/
}

/*pub fn read_keyboard(tx: std::sync::mpsc::Sender<Action>) {
    // make sure to enable raw mode, this will make sure key events won't be handled by the terminal it's self and allows crossterm to read the input and pass it back to you.
    let screen = RawScreen::into_raw_mode();

    let mut input = crossterm::input();

    println!("starting to read input");

    loop {
        let mut stdin = input.read_sync();
        if let Some(key_event) = stdin.next() {
            println!("reading {:?}", key_event);
            match key_event {
                InputEvent::Keyboard(event) => {
                    match event {
                        KeyEvent::Char(key) => {
                            if let Some(action) = get_action(key) {
                                tx.send(action).unwrap()
                            }
                        },
                        _ => println!("Keys other than normal [A-Z] are unsupported")
                    }
                },
                _ => {}
            }
        }
    }
}*/
