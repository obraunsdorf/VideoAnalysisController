use fltk::{prelude::*, window::DoubleWindow};

#[derive(Clone)]
pub(crate) enum GuiActions {
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

pub(crate) struct FltkGui {
    _app: fltk::app::App,
    pub(crate) gui_actions_receiver: fltk::app::Receiver<GuiActions>,
    pub(crate) start_frame_input: fltk::input::IntInput,
    pub(crate) end_frame_input: fltk::input::IntInput,
    pub(crate) calib_near_input: fltk::input::IntInput,
    pub(crate) calib_far_input: fltk::input::IntInput,
    pub(crate) sensitivity_input: fltk::input::FloatInput,
    pub(crate) slider: fltk::valuator::HorNiceSlider,
    pub(crate) vlc_win: DoubleWindow,
}

impl FltkGui {
    pub(crate) fn new() -> FltkGui {
        let app = fltk::app::App::default().with_scheme(fltk::app::AppScheme::Gtk);
        let win_width = 1920;
        let win_height = 1080;
        let mut win = fltk::window::Window::new(0, 0, 1920, 1080, "Media Player");

        // Create inner window to act as embedded media player
        let vlc_horizotal_margin = 10;
        let vlc_vertical_margin = 10;
        let fltk_button_space = 150;
        let vlc_win_width = win_width - vlc_horizotal_margin * 2;
        let vlc_win_heigth = win_height - vlc_vertical_margin * 2 - fltk_button_space;
        let mut vlc_win = fltk::window::Window::new(
            vlc_horizotal_margin,
            vlc_vertical_margin,
            vlc_win_width,
            vlc_win_heigth,
            "",
        );
        vlc_win.end();
        vlc_win.set_color(fltk::enums::Color::Black);

        let gui_elements_start_x = vlc_vertical_margin;
        let gui_elements_start_y = vlc_win.y() + vlc_win.height() + 10;

        let (s, r) = fltk::app::channel::<GuiActions>();

        let start_frame_input =
            fltk::input::IntInput::new(10, gui_elements_start_y + 20, 100, 30, None);
        let mut start_frame_button =
            fltk::button::Button::new(10, gui_elements_start_y + 50, 100, 30, "set start frame");
        start_frame_button.emit(s.clone(), GuiActions::SetStartFrame);

        let end_frame_input =
            fltk::input::IntInput::new(110, gui_elements_start_y + 20, 100, 30, None);
        let mut end_frame_button =
            fltk::button::Button::new(110, gui_elements_start_y + 50, 100, 30, "set end frame");
        end_frame_button.emit(s.clone(), GuiActions::SetEndFrame);

        let mut slider = fltk::valuator::HorNiceSlider::new(
            gui_elements_start_x,
            gui_elements_start_y,
            vlc_win_width,
            15,
            None,
        );
        let slider_sender = s.clone();
        slider.handle(move |w, event| match event {
            fltk::enums::Event::Drag => {
                let pos = w.value();
                slider_sender.send(GuiActions::SetMediaPosition(pos));
                true
            }

            /* fltk::enums::Event::Released => {
                w.clear_visible_focus();
                true
            }*/
            _ => false,
        });

        let mut button_acm_exe = fltk::button::Button::new(
            gui_elements_start_x + 300,
            gui_elements_start_y + 20,
            200,
            20,
            "Choose ACM Executable..",
        );
        button_acm_exe.emit(s.clone(), GuiActions::ChooseACMExe);

        let mut button_analyze = fltk::button::Button::new(
            gui_elements_start_x + 300,
            gui_elements_start_y + 40,
            200,
            20,
            "Analyze",
        );
        button_analyze.emit(s.clone(), GuiActions::Analyze);

        let mut calib_near_input =
            fltk::input::IntInput::new(600, gui_elements_start_y + 20, 120, 30, None);
        calib_near_input.set_value("900");
        let mut calib_near_button =
            fltk::button::Button::new(600, gui_elements_start_y + 50, 120, 30, "Calibrate Near");
        calib_near_button.emit(s.clone(), GuiActions::CalibrateNear);

        let mut calib_far_input =
            fltk::input::IntInput::new(720, gui_elements_start_y + 20, 120, 30, None);
        calib_far_input.set_value("500");
        let mut calib_far_button =
            fltk::button::Button::new(720, gui_elements_start_y + 50, 120, 30, "Calibrate Far");
        calib_far_button.emit(s.clone(), GuiActions::CalibrateFar);

        let mut sensitivity_input = fltk::input::FloatInput::new(
            gui_elements_start_x + 840,
            gui_elements_start_y + 20,
            120,
            30,
            None,
        );
        sensitivity_input.set_value("0.6");
        let mut button_analyze_cached = fltk::button::Button::new(
            gui_elements_start_x + 840,
            gui_elements_start_y + 50,
            120,
            30,
            "Analyze cached",
        );
        button_analyze_cached.emit(s.clone(), GuiActions::AnalyzeCached);

        win.make_resizable(true);
        //win.fullscreen(true);
        win.end();
        win.show();

        //let (key_event_sender, key_event_receiver) = fltk::app::channel::<fltk::enums::Key>();
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

        FltkGui {
            _app: app,
            gui_actions_receiver: r,
            start_frame_input,
            end_frame_input,
            calib_near_input,
            calib_far_input,
            sensitivity_input,
            slider,
            vlc_win,
        }
    }
}
