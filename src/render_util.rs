use ggez::nalgebra as na;
use ggez::{graphics, Context, ContextBuilder, GameResult};

fn uv_wrap(v: f32) -> f32 {
	v
}

pub fn build_textured_circle(
	ctx: &mut Context,
	radius: f32,
	samples: usize,
	image: Option<graphics::Image>,
	uv_offset: Option<na::Point2<f32>>,
	uv_scale: Option<na::Point2<f32>>,
) -> GameResult<graphics::Mesh> {
	let mb = &mut graphics::MeshBuilder::new();
	let uv_offset = uv_offset.unwrap_or(na::Point2::new(0.0, 0.0));
	let uv_scale = uv_scale.unwrap_or(na::Point2::new(1.0, 1.0));

	let mut triangle_verts = Vec::new();
	let mut triangle_indices = Vec::new();
	let middle = graphics::Vertex {
		pos: [0.0, 0.0],
		uv: [
			uv_wrap(uv_scale.x * (0.0 / 2.0 + 0.5 + uv_offset.x)),
			uv_wrap(uv_scale.y * (0.0 / 2.0 + 0.5 + uv_offset.y)),
		],
		color: [1.0, 1.0, 1.0, 1.0],
	};

	triangle_verts.push(middle);
	let angle: f32 = 0.0;
	let u = angle.sin();
	let v = angle.cos();
	let x = radius * u;
	let y = radius * v;

	triangle_verts.push(graphics::Vertex {
		pos: [x, y],
		uv: [
			uv_wrap(uv_scale.x * (u / 2.0 + 0.5 + uv_offset.x)),
			uv_wrap(uv_scale.y * (v / 2.0 + 0.5 + uv_offset.y)),
		],
		color: [1.0, 1.0, 1.0, 1.0],
	});

	for i in 1..(samples as u32) + 1 {
		triangle_indices.push(i);
		triangle_indices.push(0);
		triangle_indices.push(i + 1);

		let angle = (i as f32) * 2.0 * std::f32::consts::PI / (samples as f32);
		let u = angle.sin();
		let v = angle.cos();
		let x = radius * u;
		let y = radius * v;
		triangle_verts.push(graphics::Vertex {
			pos: [x, y],
			uv: [
				uv_wrap(uv_scale.x * (u / 2.0 + 0.5 + uv_offset.x)),
				uv_wrap(uv_scale.y * (v / 2.0 + 0.5 + uv_offset.y)),
			],
			color: [1.0, 1.0, 1.0, 1.0],
		});
	}
	mb.raw(&triangle_verts, &triangle_indices, image);
	mb.build(ctx)
}

pub fn build_textured_circle_earth(
	ctx: &mut Context,
	radius: f32,
	samples: usize,
	image: Option<graphics::Image>,
	uv_offset: Option<na::Point2<f32>>,
	uv_scale: Option<na::Point2<f32>>,
) -> GameResult<graphics::Mesh> {
	let mb = &mut graphics::MeshBuilder::new();
	let uv_offset = uv_offset.unwrap_or(na::Point2::new(0.0, 0.0));
	let uv_scale = uv_scale.unwrap_or(na::Point2::new(1.0, 1.0));

	let mut triangle_verts = Vec::new();
	let mut triangle_indices = Vec::new();
	let middle = graphics::Vertex {
		pos: [0.0, 0.0],
		uv: [
			uv_wrap(uv_scale.x * (0.0 / 2.0 + 0.5 + uv_offset.x)),
			uv_wrap(uv_scale.y * (0.0 / 2.0 + 0.5 + uv_offset.y)),
		],
		color: [0.5, 0.5, 0.5, 1.0],
	};

	triangle_verts.push(middle);
	let angle: f32 = 0.0;
	let u = angle.sin();
	let v = angle.cos();
	let x = radius * u;
	let y = radius * v;
	// let light = 1.0 - u;
	let light = 1.0;

	triangle_verts.push(graphics::Vertex {
		pos: [x, y],
		uv: [
			uv_wrap(uv_scale.x * (u / 2.0 + 0.5 + uv_offset.x)),
			uv_wrap(uv_scale.y * (v / 2.0 + 0.5 + uv_offset.y)),
		],
		color: [light, light, light, 1.0],
	});

	for i in 1..(samples as u32) + 1 {
		triangle_indices.push(i);
		triangle_indices.push(0);
		triangle_indices.push(i + 1);

		let angle = (i as f32) * 2.0 * std::f32::consts::PI / (samples as f32);
		let u = angle.sin();
		let v = angle.cos();
		let x = radius * u;
		let y = radius * v;
		triangle_verts.push(graphics::Vertex {
			pos: [x, y],
			uv: [
				uv_wrap(uv_scale.x * (u / 2.0 + 0.5 + uv_offset.x)),
				uv_wrap(uv_scale.y * (v / 2.0 + 0.5 + uv_offset.y)),
			],
			color: [light, light, light, 1.0],
		});
	}
	mb.raw(&triangle_verts, &triangle_indices, image);
	mb.build(ctx)
}

const METEOR_BUMPINESS: f32 = 0.1;

fn rng_meteor(id: usize, angle: f32) -> f32 {
	(id as f32).sin() * (angle).sin() * METEOR_BUMPINESS
}

pub fn build_textured_circle_meteor(
	ctx: &mut Context,
	id: usize,
	radius: f32,
	samples: usize,
	image: Option<graphics::Image>,
	uv_offset: Option<na::Point2<f32>>,
	uv_scale: Option<na::Point2<f32>>,
) -> GameResult<graphics::Mesh> {
	let mb = &mut graphics::MeshBuilder::new();
	let uv_offset = uv_offset.unwrap_or(na::Point2::new(0.0, 0.0));
	let uv_scale = uv_scale.unwrap_or(na::Point2::new(1.0, 1.0));

	let mut triangle_verts = Vec::new();
	let mut triangle_indices = Vec::new();
	let middle = graphics::Vertex {
		pos: [0.0, 0.0],
		uv: [
			uv_wrap(uv_scale.x * (0.0 / 2.0 + 0.5 + uv_offset.x)),
			uv_wrap(uv_scale.y * (0.0 / 2.0 + 0.5 + uv_offset.y)),
		],
		color: [0.5, 0.5, 0.5, 1.0],
	};

	triangle_verts.push(middle);
	let angle: f32 = 0.0;
	let u = angle.sin() + rng_meteor(id, angle);
	let v = angle.cos() + rng_meteor(id, angle);
	let x = radius * u;
	let y = radius * v;

	triangle_verts.push(graphics::Vertex {
		pos: [x, y],
		uv: [
			uv_wrap(uv_scale.x * (u / 2.0 + 0.5 + uv_offset.x)),
			uv_wrap(uv_scale.y * (v / 2.0 + 0.5 + uv_offset.y)),
		],
		color: [1.0, 1.0, 1.0, 1.0],
	});

	for i in 1..(samples as u32) + 1 {
		triangle_indices.push(i);
		triangle_indices.push(0);
		triangle_indices.push(i + 1);

		let angle = (i as f32) * 2.0 * std::f32::consts::PI / (samples as f32);
		let u = angle.sin() + rng_meteor(id, angle);
		let v = angle.cos() + rng_meteor(id, angle);
		let x = radius * u;
		let y = radius * v;
		triangle_verts.push(graphics::Vertex {
			pos: [x, y],
			uv: [
				uv_wrap(uv_scale.x * (u / 2.0 + 0.5 + uv_offset.x)),
				uv_wrap(uv_scale.y * (v / 2.0 + 0.5 + uv_offset.y)),
			],
			color: [1.0, 1.0, 1.0, 1.0],
		});
	}
	mb.raw(&triangle_verts, &triangle_indices, image);
	mb.build(ctx)
}
