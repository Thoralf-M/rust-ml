use opengl_graphics::GlGraphics;
use piston::input::*;
use rand::Rng;
use std::collections::LinkedList;

use crate::network::Network;

// https://www.youtube.com/watch?v=HCwMb0KslX8

pub struct World<'a> {
	pub gl: GlGraphics,
	pub rows: u32,
	pub cols: u32,
	pub snake: Snake,
	pub just_eaten: bool,
	pub square_width: u32,
	pub food: Food,
	pub suggested_direction: SuggestedDirection,
	pub score: u32,
	pub network: Network<'a>,
}

impl<'a> World<'a> {
	pub fn render(&mut self, args: &RenderArgs) {
		// Dark blue
		const BACKGROUN_COLOUR: [f32; 4] = [0.0, 0.07, 0.13, 1.0];

		self.gl.draw(args.viewport(), |_c, gl| {
			graphics::clear(BACKGROUN_COLOUR, gl);
		});

		self.snake.render(args);
		self.food.render(&mut self.gl, args, self.square_width);
		self.suggested_direction
			.render(&mut self.gl, args, self.square_width);
	}

	pub fn update(&mut self, _args: &UpdateArgs) -> bool {
		if !self.snake.update(self.just_eaten, self.cols, self.rows) {
			self.suggested_direction.update(
				&self.snake,
				&self.food,
				self.cols,
				self.rows,
				&mut self.network,
			);
			return false;
		}
		self.suggested_direction.update(
			&self.snake,
			&self.food,
			self.cols,
			self.rows,
			&mut self.network,
		);

		if self.just_eaten {
			self.score += 1;
			self.just_eaten = false;
		}

		self.just_eaten = self.food.update(&self.snake, self.cols, self.rows);
		// if self.just_eaten {
		// 	// try my luck
		// 	let mut r = rand::thread_rng();
		// 	loop {
		// 		let new_x = r.gen_range(0..self.cols);
		// 		let new_y = r.gen_range(0..self.rows);
		// 		if !self.snake.is_collide(new_x, new_y) {
		// 			self.food = Food { x: new_x, y: new_y };
		// 			break;
		// 		}
		// 	}
		// }

		true
	}

	pub fn pressed(&mut self, btn: &Button) {
		let last_direction = self.snake.d.clone();
		let target = match btn {
			Button::Keyboard(Key::Up) if last_direction != Direction::DOWN => vec![1., 0., 0., 0.],
			Button::Keyboard(Key::Left) if last_direction != Direction::RIGHT => {
				vec![0., 1., 0., 0.]
			}
			Button::Keyboard(Key::Right) if last_direction != Direction::LEFT => {
				vec![0., 0., 1., 0.]
			}
			Button::Keyboard(Key::Down) if last_direction != Direction::UP => vec![0., 0., 0., 1.],
			_ => vec![0.; 4],
		};

		// train with user input

		let s = &self.snake;
		let food = &self.food;
		let cols = self.cols;
		let rows = self.rows;
		let front = self.snake.snake_parts.front().unwrap();
		// Calculate the direct environment
		// 0 => neutral, 1 => positive, -1 => negative
		let mut environment = Vec::new();

		// top left
		let (x, y) = (front.0, front.1);
		if x == 0 || y == 0 || s.is_collide(x.saturating_sub(1), y.saturating_sub(1)) {
			environment.push(-1.);
		} else if x.saturating_sub(1) == food.x && y.saturating_sub(1) == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// top
		let (x, y) = (front.0, front.1);
		if y == 0 || s.is_collide(x, y.saturating_sub(1)) {
			environment.push(-1.);
		} else if x == food.x && y.saturating_sub(1) == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// top right
		let (x, y) = (front.0 + 1, front.1);
		if x == 0 || y == 0 || x == cols || s.is_collide(x, y) {
			environment.push(-1.);
		} else if x == food.x && y.saturating_sub(1) == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// left
		let (x, y) = (front.0, front.1);
		if x == 0 || s.is_collide(x.saturating_sub(1), y) {
			environment.push(-1.);
		} else if x.saturating_sub(1) == food.x && y == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// right
		let (x, y) = (front.0 + 1, front.1);
		if x == cols || y == rows || s.is_collide(x, y) {
			environment.push(-1.);
		} else if x == food.x && y == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// bottom left
		let (x, y) = (front.0, front.1 + 1);
		if x == 0 || y == rows || s.is_collide(x.saturating_sub(1), y) {
			environment.push(-1.);
		} else if x.saturating_sub(1) == food.x && y == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// botom
		let (x, y) = (front.0, front.1 + 1);
		if x == cols || y == rows || s.is_collide(x, y) {
			environment.push(-1.);
		} else if x == food.x && y == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// bottom right
		let (x, y) = (front.0 + 1, front.1 + 1);
		if x == cols || y == rows || s.is_collide(x, y) {
			environment.push(-1.);
		} else if x == food.x && y == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		self.network
			.train(vec![environment.clone()], vec![target], 10);

		let network_direction = self.network.feed_forward(environment);

		self.snake.d = match network_direction
			.into_iter()
			.enumerate()
			.max_by(|(_, a), (_, b)| (*a).total_cmp(b))
			.map(|(index, _)| index)
			.expect("no target value")
		{
			0 if last_direction != Direction::DOWN => Direction::UP,
			1 if last_direction != Direction::RIGHT => Direction::LEFT,
			2 if last_direction != Direction::LEFT => Direction::RIGHT,
			3 if last_direction != Direction::UP => Direction::DOWN,
			_ => last_direction,
		};

		// apply user input
		// self.snake.d = match btn {
		// 	&Button::Keyboard(Key::Up) if last_direction != Direction::DOWN => Direction::UP,
		// 	&Button::Keyboard(Key::Down) if last_direction != Direction::UP => Direction::DOWN,
		// 	&Button::Keyboard(Key::Left) if last_direction != Direction::RIGHT => Direction::LEFT,
		// 	&Button::Keyboard(Key::Right) if last_direction != Direction::LEFT => Direction::RIGHT,
		// 	_ => last_direction,
		// };
	}

	pub fn automatic(&mut self) {
		println!("automatic update");
		let last_direction = self.snake.d.clone();

		let s = &self.snake;
		let food = &self.food;
		let cols = self.cols;
		let rows = self.rows;
		let front = self.snake.snake_parts.front().unwrap();
		// Calculate the direct environment
		// 0 => neutral, 1 => positive, -1 => negative
		let mut environment = Vec::new();

		// top left
		let (x, y) = (front.0, front.1);
		if x == 0 || y == 0 || s.is_collide(x.saturating_sub(1), y.saturating_sub(1)) {
			environment.push(-1.);
		} else if x.saturating_sub(1) == food.x && y.saturating_sub(1) == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// top
		let (x, y) = (front.0, front.1);
		if y == 0 || s.is_collide(x, y.saturating_sub(1)) {
			environment.push(-1.);
		} else if x == food.x && y.saturating_sub(1) == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// top right
		let (x, y) = (front.0 + 1, front.1);
		if x == 0 || y == 0 || x == cols || s.is_collide(x, y) {
			environment.push(-1.);
		} else if x == food.x && y.saturating_sub(1) == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// left
		let (x, y) = (front.0, front.1);
		if x == 0 || s.is_collide(x.saturating_sub(1), y) {
			environment.push(-1.);
		} else if x.saturating_sub(1) == food.x && y == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// right
		let (x, y) = (front.0 + 1, front.1);
		if x == cols || y == rows || s.is_collide(x, y) {
			environment.push(-1.);
		} else if x == food.x && y == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// bottom left
		let (x, y) = (front.0, front.1 + 1);
		if x == 0 || y == rows || s.is_collide(x.saturating_sub(1), y) {
			environment.push(-1.);
		} else if x.saturating_sub(1) == food.x && y == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// botom
		let (x, y) = (front.0, front.1 + 1);
		if x == cols || y == rows || s.is_collide(x, y) {
			environment.push(-1.);
		} else if x == food.x && y == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// bottom right
		let (x, y) = (front.0 + 1, front.1 + 1);
		if x == cols || y == rows || s.is_collide(x, y) {
			environment.push(-1.);
		} else if x == food.x && y == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		let network_direction = self.network.feed_forward(environment);

		self.snake.d = match network_direction
			.into_iter()
			.enumerate()
			.max_by(|(_, a), (_, b)| (*a).total_cmp(b))
			.map(|(index, _)| index)
			.expect("no target value")
		{
			0 if last_direction != Direction::DOWN => Direction::UP,
			1 if last_direction != Direction::RIGHT => Direction::LEFT,
			2 if last_direction != Direction::LEFT => Direction::RIGHT,
			3 if last_direction != Direction::UP => Direction::DOWN,
			_ => last_direction,
		};
	}
}

/// The direction the snake moves in.
#[derive(Clone, PartialEq)]
pub enum Direction {
	UP,
	DOWN,
	LEFT,
	RIGHT,
}

pub struct Snake {
	pub gl: GlGraphics,
	pub snake_parts: LinkedList<SnakePiece>,
	pub width: u32,
	pub d: Direction,
}

#[derive(Clone)]
pub struct SnakePiece(pub u32, pub u32);

impl Snake {
	pub fn render(&mut self, args: &RenderArgs) {
		const RED: [f32; 4] = [0.8, 0.0, 0.0, 1.0];

		let squares: Vec<graphics::types::Rectangle> = self
			.snake_parts
			.iter()
			.map(|p| SnakePiece(p.0 * self.width, p.1 * self.width))
			.map(|p| graphics::rectangle::square(p.0 as f64, p.1 as f64, self.width as f64))
			.collect();

		self.gl.draw(args.viewport(), |c, gl| {
			let transform = c.transform;

			squares
				.into_iter()
				.for_each(|square| graphics::rectangle(RED, square, transform, gl));
		})
	}

	/// Move the snake if valid, otherwise returns false.
	pub fn update(&mut self, just_eaten: bool, cols: u32, rows: u32) -> bool {
		let mut new_front: SnakePiece =
			(*self.snake_parts.front().expect("No front of snake found.")).clone();

		// Check if colliding with the border
		if (self.d == Direction::UP && new_front.1 == 0)
			|| (self.d == Direction::LEFT && new_front.0 == 0)
			|| (self.d == Direction::DOWN && new_front.1 == rows - 1)
			|| (self.d == Direction::RIGHT && new_front.0 == cols - 1)
		{
			return false;
		}

		match self.d {
			Direction::UP => new_front.1 -= 1,
			Direction::DOWN => new_front.1 += 1,
			Direction::LEFT => new_front.0 -= 1,
			Direction::RIGHT => new_front.0 += 1,
		}

		if !just_eaten {
			self.snake_parts.pop_back();
		}

		// Checks self collision.
		if self.is_collide(new_front.0, new_front.1) {
			return false;
		}

		self.snake_parts.push_front(new_front);
		true
	}

	fn is_collide(&self, x: u32, y: u32) -> bool {
		self.snake_parts.iter().any(|p| x == p.0 && y == p.1)
	}
}

pub struct Food {
	pub x: u32,
	pub y: u32,
}

impl Food {
	// Return true if snake ate food this update
	fn update(&mut self, s: &Snake, cols: u32, rows: u32) -> bool {
		let front = s.snake_parts.front().unwrap();
		if front.0 == self.x && front.1 == self.y {
			let mut r = rand::thread_rng();
			loop {
				let new_x = r.gen_range(0..cols);
				let new_y = r.gen_range(0..rows);
				if !s.is_collide(new_x, new_y) {
					self.x = new_x;
					self.y = new_y;
					break;
				}
			}
			true
		} else {
			false
		}
	}

	fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs, width: u32) {
		const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

		let x = self.x * width;
		let y = self.y * width;

		let square = graphics::rectangle::square(x as f64, y as f64, width as f64);

		gl.draw(args.viewport(), |c, gl| {
			let transform = c.transform;

			graphics::rectangle(WHITE, square, transform, gl)
		});
	}
}

pub struct SuggestedDirection {
	pub x: u32,
	pub y: u32,
}

impl SuggestedDirection {
	fn update(&mut self, s: &Snake, food: &Food, cols: u32, rows: u32, network: &mut Network) {
		let front = s.snake_parts.front().unwrap();

		// Calculate the direct environment
		// 0 => neutral, 1 => positive, -1 => negative
		let mut environment = Vec::new();

		// top left
		let (x, y) = (front.0, front.1);
		if x == 0 || y == 0 || s.is_collide(x.saturating_sub(1), y.saturating_sub(1)) {
			environment.push(-1.);
		} else if x.saturating_sub(1) == food.x && y.saturating_sub(1) == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// top
		let (x, y) = (front.0, front.1);
		if y == 0 || s.is_collide(x, y.saturating_sub(1)) {
			environment.push(-1.);
		} else if x == food.x && y.saturating_sub(1) == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// top right
		let (x, y) = (front.0 + 1, front.1);
		if x == 0 || y == 0 || x == cols || s.is_collide(x, y) {
			environment.push(-1.);
		} else if x == food.x && y.saturating_sub(1) == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// left
		let (x, y) = (front.0, front.1);
		if x == 0 || s.is_collide(x.saturating_sub(1), y) {
			environment.push(-1.);
		} else if x.saturating_sub(1) == food.x && y == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// right
		let (x, y) = (front.0 + 1, front.1);
		if x == cols || y == rows || s.is_collide(x, y) {
			environment.push(-1.);
		} else if x == food.x && y == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// bottom left
		let (x, y) = (front.0, front.1 + 1);
		if x == 0 || y == rows || s.is_collide(x.saturating_sub(1), y) {
			environment.push(-1.);
		} else if x.saturating_sub(1) == food.x && y == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// botom
		let (x, y) = (front.0, front.1 + 1);
		if x == cols || y == rows || s.is_collide(x, y) {
			environment.push(-1.);
		} else if x == food.x && y == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		// bottom right
		let (x, y) = (front.0 + 1, front.1 + 1);
		if x == cols || y == rows || s.is_collide(x, y) {
			environment.push(-1.);
		} else if x == food.x && y == food.y {
			environment.push(1.);
		} else {
			environment.push(0.);
		}

		println!("environment: {environment:?}");

		let network_direction = network.feed_forward(environment);

		let (suggested_x, suggested_y) = match network_direction
			.into_iter()
			.enumerate()
			.max_by(|(_, a), (_, b)| (*a).total_cmp(b))
			.map(|(index, _)| index)
			.expect("no target value")
		{
			// top
			0 => (front.0, front.1.saturating_sub(1)),
			// left
			1 => (front.0.saturating_sub(1), front.1),
			// right
			2 => (front.0 + 1, front.1),
			// bottom
			3 => (front.0, front.1 + 1),
			_ => (front.0, front.1),
		};

		// manual stuff
		// let pos = if let Some(pos) = environment.iter().position(|&x| x >= 1.0) {
		// 	pos
		// } else {
		// 	if let Some(pos) = environment.iter().position(|&x| x >= 0.0) {
		// 		pos
		// 	} else {
		// 		panic!("snale sourrounded by itself");
		// 	}
		// };

		// TODO: only suggest top, left, right, bottom
		// let (suggested_x, suggested_y) = match pos {
		// 	// top left
		// 	0 => (front.0.saturating_sub(1), front.1.saturating_sub(1)),
		// 	// top
		// 	1 => (front.0, front.1.saturating_sub(1)),
		// 	// top right
		// 	2 => (front.0 + 1, front.1.saturating_sub(1)),
		// 	// left
		// 	3 => (front.0.saturating_sub(1), front.1),
		// 	// right
		// 	4 => (front.0 + 1, front.1),
		// 	// bottom left
		// 	5 => (front.0.saturating_sub(1), front.1 + 1),
		// 	// bottom
		// 	6 => (front.0, front.1 + 1),
		// 	// bottom right
		// 	7 => (front.0 + 1, front.1 + 1),
		// 	// nothing
		// 	_ => (front.0, front.1),
		// };
		self.x = suggested_x;
		self.y = suggested_y;
	}

	fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs, width: u32) {
		const GREEN: [f32; 4] = [0.0, 0.5, 0.0, 1.0];

		let x = self.x * width;
		let y = self.y * width;

		let square = graphics::rectangle::square(x as f64, y as f64, width as f64);

		gl.draw(args.viewport(), |c, gl| {
			let transform = c.transform;

			graphics::rectangle(GREEN, square, transform, gl)
		});
	}
}
