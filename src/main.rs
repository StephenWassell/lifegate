// (c) 2019 Stephen Wassell

#[macro_use]
extern crate gate;

use gate::{App, AppContext, AppInfo, KeyCode};
use gate::renderer::{Renderer, Affine, SpriteRenderer};

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
		(x-1,y),			(x+1,y),
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

fn neighbours_nesw(col: &Colony, &(x, y): &Cell) -> (bool, bool, bool, bool) {
	(
		col.contains(&(x, y+1)), // N
		col.contains(&(x+1, y)), // E
		col.contains(&(x, y-1)), // S
		col.contains(&(x-1, y))  // W
	)
}

fn cell_sprite(col: &Colony, cell: &Cell) -> SpriteId {
	match neighbours_nesw(col, cell) {
		(false, false, false, false) => SpriteId::CellsR0C0,
		(false, false, false, true) => SpriteId::CellsR0C3,
		(false, false, true, false) => SpriteId::CellsR1C0,
		(false, false, true, true) => SpriteId::CellsR1C3,
		(false, true, false, false) => SpriteId::CellsR0C1,
		(false, true, false, true) => SpriteId::CellsR0C2,
		(false, true, true, false) => SpriteId::CellsR1C1,
		(false, true, true, true) => SpriteId::CellsR1C2,
		(true, false, false, false) => SpriteId::CellsR3C0,
		(true, false, false, true) => SpriteId::CellsR3C3,
		(true, false, true, false) => SpriteId::CellsR2C0,
		(true, false, true, true) => SpriteId::CellsR2C3,
		(true, true, false, false) => SpriteId::CellsR3C1,
		(true, true, false, true) => SpriteId::CellsR3C2,
		(true, true, true, false) => SpriteId::CellsR2C1,
		(true, true, true, true) => SpriteId::CellsR2C2
	}
}

fn button_sprite(x: i32, down: bool) -> SpriteId {
	match (x, down) {
		(0, false) => SpriteId::ButtonsR0C0,
		(1, false) => SpriteId::ButtonsR0C1,
		(2, false) => SpriteId::ButtonsR0C2,
		(3, false) => SpriteId::ButtonsR0C3,
		(4, false) => SpriteId::ButtonsR0C4,
		(5, false) => SpriteId::ButtonsR0C5,
		(_, false) => SpriteId::ButtonsR0C6,
		(0, true) => SpriteId::ButtonsR1C0,
		(1, true) => SpriteId::ButtonsR1C1,
		(2, true) => SpriteId::ButtonsR1C2,
		(3, true) => SpriteId::ButtonsR1C3,
		(4, true) => SpriteId::ButtonsR1C4,
		(5, true) => SpriteId::ButtonsR1C5,
		(_, true) => SpriteId::ButtonsR1C6,
	}
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

	fn draw_background(&mut self, renderer: &mut SpriteRenderer<AssetId>, w: i32, h: i32) {
		for x in 0..=(w / 32) {
			for y in 0..=(h / 32) {
				let affine = Affine::translate(
						(16 + x * 32) as f64,
						(16 + y * 32) as f64).
					post_scale(16./self.zoom_level);
				renderer.draw(&affine, SpriteId::Checker32);
			}
		}
	}	

	fn draw_cells(&mut self, renderer: &mut SpriteRenderer<AssetId>, w: i32, h: i32) {
		for x in 0..w {
			for y in 0..h {
				let affine = Affine::translate(
						(8 + x * 16) as f64,
						(8 + y * 16) as f64).
					post_scale(1./self.zoom_level);
				
				let xc = x; // + self.centre.0 - half_w;
				let yc = y; // + self.centre.1 - half_h;
				
				if self.col.contains(&(xc, yc)) {
					renderer.draw(&affine, cell_sprite(&self.col, &(xc, yc)));
				}
			}
		}
	}

	fn draw_buttons(&mut self, renderer: &mut SpriteRenderer<AssetId>) {
		for x in 0..7 {
			let affine = Affine::translate(
					(8 + x * 16) as f64, 8.);

			renderer.draw(&affine, button_sprite(x, false));
		}
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
			KeyCode::Num1 => fullscreen(ctx), // []
			KeyCode::Num2 => self.clear(),    // X
			KeyCode::Num3 => self.rewind(),   // <<
			KeyCode::Num4 => self.fps = 0.,   // ||
			KeyCode::Num5 => self.fps = 3.,   // >
			KeyCode::Num6 => self.fps = 15.,  // >>
			KeyCode::Num7 => self.zoom(),     // +
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
		//let half_w = w / 2;
		//let half_h = h / 2;
		self.draw_background(&mut renderer, w, h);
		self.draw_cells(&mut renderer, w, h);
		self.draw_buttons(&mut renderer);
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
