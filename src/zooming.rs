pub(crate) struct Zoom<'player> {
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

    pub(crate) fn new(mdp: & vlc::MediaPlayer, resolution: (u32, u32)) -> Zoom {
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

    pub(crate) fn updateZoom(&mut self, kind: ZoomUpdateKind) {
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