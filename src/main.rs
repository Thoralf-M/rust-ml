use activations::SIGMOID;
use network::Network;

pub mod activations;
pub mod matrix;
pub mod network;
pub mod world;
use world::*;

use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{event_loop::*, input::*, window::WindowSettings};
use std::{collections::LinkedList, iter::FromIterator};

fn main() {
	let inputs = vec![
		// field in the direction
		// 0 -> neutral
		// 1 -> positive
		// -1 -> negative
		//   🡔    🡑    🡕    🡐    🡒   🡗    🡓    🡖
		vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
		vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
		vec![0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
		vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
	];
	let targets = vec![
		// direction to go
		//   🡑    🡐    🡒   🡓
		vec![1.0, 0.0, 0.0, 0.0],
		vec![0.0, 1.0, 0.0, 0.0],
		vec![0.0, 0.0, 1.0, 0.0],
		vec![0.0, 0.0, 0.0, 1.0],
	];

	let mut network = Network::new(vec![inputs[0].len(), 6, targets[0].len()], 0.5, SIGMOID);

	// pretrain with main targets
	network.train(inputs, targets, 10000);

	let inputs = vec![
		// field in the direction
		// 0 -> neutral
		// 1 -> positive
		// -1 -> negative
		//   🡔    🡑    🡕    🡐    🡒   🡗    🡓    🡖
		vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
		vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
		vec![0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
		vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
		// 🡔
		vec![1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
		// 🡕
		vec![0.0, -1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
		// 🡗
		vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, -1.0, 0.0],
		// 🡖
		vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 1.0],
		// all bottom negative
		vec![0.0, 0.0, 0.0, 0.0, 0.0, -1.0, -1.0, -1.0],
		// all top negative
		vec![-1.0, -1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
		// all right negative
		vec![0.0, 0.0, -1.0, 0.0, -1.0, 0.0, 0.0, -1.0],
		// all left negative
		vec![-1.0, 0.0, 0.0, -1.0, 0.0, -1.0, 0.0, 0.0],
		// all top and bottom negative
		vec![-1.0, -1.0, -1.0, 0.0, 0.0, -1.0, -1.0, -1.0],
		// all left and right negative
		vec![-1.0, 0.0, -1.0, -1.0, 0.0, -1.0, 0.0, -1.0],
	];
	let targets = vec![
		// direction to go
		//   🡑    🡐    🡒   🡓
		vec![1.0, 0.0, 0.0, 0.0],
		vec![0.0, 1.0, 0.0, 0.0],
		vec![0.0, 0.0, 1.0, 0.0],
		vec![0.0, 0.0, 0.0, 1.0],
		vec![-1.0, 1.0, 0.0, 0.0],
		vec![-1.0, 0.0, 1.0, 0.0],
		vec![0.0, 1.0, 0.0, -1.0],
		vec![0.0, 0.0, 1.0, -1.0],
		vec![1.0, 0.0, 0.0, -1.0],
		vec![-1.0, 0.0, 0.0, 1.0],
		vec![0.0, 1.0, -1.0, 0.0],
		vec![0.0, -1.0, 1.0, 0.0],
		vec![-1.0, 1.0, 1.0, -1.0],
		vec![1.0, -1.0, -1.0, 1.0],
	];
	// train some more with more complex environment
	network.train(inputs, targets, 1000);

	// println!(
	// 	"{:?}",
	// 	network.feed_forward(vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
	// );

	let opengl = OpenGL::V4_5;

	const COLS: u32 = 10;
	const ROWS: u32 = 10;
	const SQUARE_WIDTH: u32 = 40;

	const WIDTH: u32 = COLS * SQUARE_WIDTH;
	const HEIGHT: u32 = ROWS * SQUARE_WIDTH;

	let mut window: GlutinWindow = WindowSettings::new("Snake Game", [WIDTH, HEIGHT])
		.graphics_api(opengl)
		.exit_on_esc(true)
		.build()
		.unwrap();

	let mut world = World {
		gl: GlGraphics::new(opengl),
		rows: ROWS,
		cols: COLS,
		square_width: SQUARE_WIDTH,
		just_eaten: false,
		food: Food { x: 1, y: 1 },
		suggested_direction: SuggestedDirection { x: 0, y: 0 },
		score: 0,
		snake: Snake {
			gl: GlGraphics::new(opengl),
			// start with 2 pieces, because it then knows that it shouldn't go into its own body
			snake_parts: LinkedList::from_iter(
				(vec![SnakePiece(COLS / 2, ROWS / 2); 2]).into_iter(),
			),
			width: SQUARE_WIDTH,
			d: Direction::DOWN,
		},
		network,
	};

	let mut events = Events::new(EventSettings::new()).ups(10);
	while let Some(e) = events.next(&mut window) {
		if let Some(r) = e.render_args() {
			world.render(&r);
		}

		if let Some(u) = e.update_args() {
			if !world.update(&u) {
				// break closes the window
				break;
			}
		}

		if let Some(k) = e.button_args() {
			if k.state == ButtonState::Press {
				world.pressed(&k.button);
			}
		} else if e.update_args().is_some() {
			world.automatic()
		}
	}
	println!("Congratulations, your score was: {}", world.score);
}
