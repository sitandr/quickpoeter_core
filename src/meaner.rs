use crate::reader::{VECTOR_DIM};

pub struct MeanField{
	average: [f32;VECTOR_DIM],
	sigmas: [f32;VECTOR_DIM] // "weights" to the difference between array and average 
}

impl MeanField{
	fn new(vectors: Vec<[f32;VECTOR_DIM]>) -> MeanField{
		let N = vectors.len();
		let mut average = [0.0; VECTOR_DIM];
		let mut sigma = [0.0; VECTOR_DIM];

		for j in 0..VECTOR_DIM{
			for i in 0..N{
				average[j] += vectors[i][j];
			}
			average[j] /= N as f32;

			for i in 0..N{
				sigma[j] += (vectors[i][j] - average[j]).powf(2.0);
			}
			sigma[j] /= N as f32;
			sigma[j] = sigma[j].sqrt();
		}
		MeanField{average: average, sigmas: sigma}
	}

	fn dist(&self, vector: [f32;VECTOR_DIM]) -> f32{
		let mut dist: f32 = 0.0;

		for i in 0..VECTOR_DIM{
			dist += (vector[i] - self.average[i]).abs().powf(2.0);
		}

		dist.powf(4.0)
	}
}

pub fn dist_arrays(v1: [f32;VECTOR_DIM], v2: [f32;VECTOR_DIM]) -> f32{
	let mut sum = 0.0;
	for i in 0..VECTOR_DIM{
		sum += (v1[i] - v2[i]).powf(2.0);
	}
	sum
}
