use crate::reader::{VECTOR_DIM};

pub struct MeanField{
	average: [f32;VECTOR_DIM],
	sigmas: [f32;VECTOR_DIM] // "weights" to the difference between array and average 
}

impl MeanField{
	fn new(vectors: Vec<[f32;VECTOR_DIM]>) -> MeanField{
		let N = vector.len();
		let mut average = [0; VECTOR_DIM];
		let mut sigma = [0; VECTOR_DIM];

		for j in 0..VECTOR_DIM{
			for i in 0..N{
				sigma[j] += vectors[i][j].powf(2.0);
				average[j] += vectors[i][j];
			}
			sigma[j] /= N;
			average[j] /= N;
			sigma[j] = sigma[j].sqrt();
		}
		MeanField{average: average, sigma: sigma}
	}

	fn dist(vector: [f32;VECTOR_DIM]) -> f32{
		let mut dist = 0.0;

		for i in 0..VECTOR_DIM{
			dist += (vector[j] - average[j]).abs().powf(2.0);
		}

		dist.powf(4)
	}
}

fn dist_arrays(v1: [f32;VECTOR_DIM], v2: [f32;VECTOR_DIM]) -> f32{
	let mut sum = 0.0;
	for i in 0..VECTOR_DIM{
		sum += (v1[i] - v2[i]).powf(2.0);
	}
	sum
}
