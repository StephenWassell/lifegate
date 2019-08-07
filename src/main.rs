#[macro_use]
extern crate gate;

use gate::{App, AppContext, AppInfo, KeyCode};
use gate::renderer::{Renderer, Affine};

use std::collections::HashMap;
use std::collections::HashSet;

mod asset_id { include!(concat!(env!("OUT_DIR"), "/asset_id.rs")); }
use asset_id::{AssetId, SpriteId}; //, MusicId, SoundId};

gate_header!();

type Cell = (i32, i32);
type Colony = HashSet<Cell>;

fn neighbours(&(x,y): &Cell) -> Vec<Cell> {
    vec![
        (x-1,y-1), (x,y-1), (x+1,y-1),
        (x-1,y),            (x+1,y),
        (x-1,y+1), (x,y+1), (x+1,y+1),
    ]
}
 
fn neighbour_counts(col: &Colony) -> HashMap<Cell, i32> {
    let mut ncnts = HashMap::new();
    for cell in col.iter().flat_map(neighbours) {
        *ncnts.entry(cell).or_insert(0) += 1;
    }
    ncnts
}
 
fn generation(col: &Colony) -> Colony {
    neighbour_counts(col)
        .into_iter()
        .filter_map(|(cell, cnt)|
            match (cnt, col.contains(&cell)) {
                (2, true) |
                (3, ..) => Some(cell),
                _ => None
        })
        .collect()
}

fn toggle(col: &mut Colony, cell: Cell) {
	if col.contains(&cell) {
		col.remove(&cell);
	} else {
		col.insert(cell);
	}
}

fn fullscreen(ctx: &mut AppContext<AssetId>) {
	if ctx.is_fullscreen() {
		ctx.cancel_fullscreen();
	} else {
		ctx.request_fullscreen();
	}
}

struct LifeGame {
	col: Colony,
	fps: f64,
	time: f64,
	centre: Cell,
	saved_col: Colony,
	saved_centre: Cell,
	zoomed_out: bool,
	zoom_level: f64
}

impl LifeGame {
	fn new() -> LifeGame {
		LifeGame {
			col: Colony::new(),
			fps: 0.,
			time: 0.,
			centre: (0, 0),
			saved_col: Colony::new(),
			saved_centre: (0, 0),
			zoomed_out: false,
			zoom_level: 1.
		}
	}

	fn clear(&mut self) {
		self.col.clear();
		self.centre = (0, 0);
		self.fps = 0.;
	}
	
	fn save(&mut self) {
		self.saved_col = self.col.clone();
		self.saved_centre = self.centre;
	}
	
	fn rewind(&mut self) {
		self.col = self.saved_col.clone();
		self.centre = self.saved_centre;
		self.fps = 0.;
	}
	
	fn zoom(&mut self) {
		self.zoomed_out = !self.zoomed_out;
	}
	
	fn cursor_to_cell(&self, pos: (f64, f64)) -> Cell {
		let x = (pos.0 * self.zoom_level / 16.) as i32;
		let y = (pos.1 * self.zoom_level / 16.) as i32;
		(x, y)
	}
}

impl App<AssetId> for LifeGame {
    //fn start(&mut self, ctx: &mut AppContext<AssetId>) {
        //ctx.audio.loop_music(MusicId::Tick);
    //}

    fn advance(&mut self, seconds: f64, _ctx: &mut AppContext<AssetId>) {
        if self.fps > 0. {
			self.time += seconds;
			while self.time >= 1./self.fps {
				self.time -= 1./self.fps;
				self.col = generation(&self.col);
			}
		}
		
		self.zoom_level = if self.zoomed_out {
			(self.zoom_level + seconds*100.).min(8.)
		} else {
			(self.zoom_level - seconds*100.).max(1.)
		}
	}

    fn key_down(&mut self, key: KeyCode, ctx: &mut AppContext<AssetId>) {
		match key {
			KeyCode::Num1 => fullscreen(ctx),
			KeyCode::Num2 => self.clear(),
            KeyCode::Num3 => self.rewind(),
            KeyCode::Num4 => self.fps = 0.,
            KeyCode::Num5 => self.fps = 3.,
            KeyCode::Num6 => self.fps = 15.,
            KeyCode::Num7 => self.zoom(),
            KeyCode::MouseLeft => {
            	let cell = self.cursor_to_cell(ctx.cursor());
				if self.fps > 0. || self.zoom_level > 1. {
					self.centre = cell;
				} else {
					toggle(&mut self.col, cell);
					self.save();
				}
			},
			_ => (),
        };
    }

    fn render(&mut self, renderer: &mut Renderer<AssetId>, ctx: &AppContext<AssetId>) {
        let (app_width, app_height) = ctx.dims();
        let mut renderer = renderer.sprite_mode();
        let w = (app_width * self.zoom_level / 16.).ceil() as i32;
        let h = (app_height * self.zoom_level / 16.).ceil() as i32;
        let half_w = w / 2;
        let half_h = h / 2;
        for x in 0..w {
            for y in 0..h {
				let affine = Affine::translate(
						(8 + x * 16) as f64,
						(8 + y * 16) as f64).
					post_scale(1./self.zoom_level);
                
				let tile = if (x + y) % 2 == 0 {
					SpriteId::BgTileR0C0
				} else {
					SpriteId::BgTileR0C1
				};
				renderer.draw(&affine, tile);
				
				let xc = x; // + self.centre.0 - half_w;
				let yc = y; // + self.centre.1 - half_h;
				
				if self.col.contains(&(xc, yc)) {
					renderer.draw(&affine, SpriteId::ItemsR5C1);
				}
            }
        }
        // controls at 0,0
    }
}

fn main() {
	let size_min = 8. * 16.;
	let size_max = 16. * 16.;
	
    let info = AppInfo::with_max_dims(size_max, size_max)
                       .min_dims(size_min, size_min)
                       .target_fps(30.)
                       .tile_width(16)
                       .title("Life");

    gate::run(info, LifeGame::new());
}
