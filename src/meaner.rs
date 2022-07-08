use crate::reader::{VECTOR_DIM};

pub struct MeanField{
	average: [f32;VECTOR_DIM],
	/// None if only one vect is present
	sigmas: Option<[f32;VECTOR_DIM]> // "weights" to the difference between array and average 

}

impl MeanField{
	pub fn new(vectors: Vec<[f32;VECTOR_DIM]>) -> MeanField{
		let n = vectors.len();

		if n == 1{
			return Self::from_single(vectors[0]);
		}
		let mut average = [0.0; VECTOR_DIM];
		let mut sigma = [0.0; VECTOR_DIM];

		for j in 0..VECTOR_DIM{
			for i in 0..n{
				average[j] += vectors[i][j];
			}
			average[j] /= n as f32;

			if n > 1{
				for i in 0..n{
					sigma[j] += (vectors[i][j] - average[j]).powf(2.0);
				}
				sigma[j] /= (n - 1) as f32;
				sigma[j] = sigma[j].sqrt();
			}
		}
		MeanField{average: average, sigmas: Some(sigma)}
	}

	fn from_single(vector: [f32;VECTOR_DIM]) -> MeanField{
		MeanField{average: vector, sigmas: None}
	}

	pub fn dist(&self, vector: [f32;VECTOR_DIM]) -> f32{

		if let Some(sigma) = self.sigmas{
			let mut dist: f32 = 0.0;
			for i in 0..VECTOR_DIM{
				dist += (vector[i] - self.average[i]).abs().powf(2.0)/sigma[i]/33.0;
			}
			dist
		}
		else{
			dist_arrays(self.average, vector)
		}
		
	}
}

pub fn dist_arrays(v1: [f32;VECTOR_DIM], v2: [f32;VECTOR_DIM]) -> f32{
	let mut sum = 0.0;
	for i in 0..VECTOR_DIM{
		sum += (v1[i] - v2[i]).powf(2.0);
	}
	sum
}
