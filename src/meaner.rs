use std::ops::Deref;
use crate::reader::VECTOR_DIM;
use crate::finder::WordCollector;
use ordered_float::NotNan;

pub struct MeanField{
	average: [f32;VECTOR_DIM],
	/// None if only one vect is present
	sigmas: Option<[f32;VECTOR_DIM]> // "weights" to the difference between array and average 

}

impl MeanField{
	/// will panic if no vectors are provided
	pub fn new(vectors: Vec<[f32;VECTOR_DIM]>) -> Self{
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

		if n > 1{
			let m = sigma.iter().map(|x| NotNan::new(*x).unwrap()).min().unwrap().into_inner();
			for j in 0..VECTOR_DIM{
				sigma[j] /= m;
			}
		}

		MeanField{average: average, sigmas: Some(sigma)}
	}

	pub fn try_new(vectors: Vec<[f32;VECTOR_DIM]>) -> Option<Self>{
		if vectors.is_empty(){
			None
		}
		else{
			Some(Self::new(vectors))
		}
	}

	
	/// just skips the words word collector doesn't know
	/// returns None if no words are left
	#[allow(dead_code)]
	pub fn from_strings_filter<S>(wc: &WordCollector, strs: &Vec<S>) -> Option<Self>
	where S: Deref<Target = str>
	{
		 let vects: Vec<[f32; VECTOR_DIM]> = strs.iter().filter_map(|s| wc.get_meaning(s)).collect();
		 Self::try_new(vects)
	}

	/// empty vec err if no strings are provided
	pub fn from_str<'a, S>(wc: &WordCollector, strs: &'a Vec<S>) -> Result<Self, Vec<&'a S>>
	where S: Deref<Target = str>
	{
		let vects: Vec<[f32; VECTOR_DIM]> = map_with_failures(strs.iter(), |s| wc.get_meaning(s))?;
		Self::try_new(vects).ok_or(vec![])
	}

	fn from_single(vector: [f32;VECTOR_DIM]) -> Self{
		MeanField{average: vector, sigmas: None}
	}

	pub fn dist(&self, vector: [f32;VECTOR_DIM]) -> f32{

		if let Some(sigma) = self.sigmas{
			let mut dist: f32 = 0.0;
			for i in 0..VECTOR_DIM{
				dist += ((vector[i] - self.average[i]).abs()).powf(0.57)/sigma[i]/10.0;
			}
			dist
		}
		else{
			dist_arrays(self.average, vector)
		}
		
	}
}

pub fn map_with_failures<'a, T, U, F, I>(iter: I, f: F) -> Result<Vec<U>, Vec<T>>
	where F: Fn(&T) -> Option<U>,
	I: Iterator<Item = T>
	{
	use itertools::{Itertools, Either};
	let (success, failured): (Vec<U>, Vec<T>) = iter.partition_map(|s| 
		match f(&s){
			Some(vec) => Either::Left(vec),
			None => Either::Right(s),
		}
	);

	if failured.len() == 0{
		Ok(success)
	}
	else{
		Err(failured)
	}
}

pub fn dist_arrays(v1: [f32;VECTOR_DIM], v2: [f32;VECTOR_DIM]) -> f32{
	let mut sum = 0.0;
	for i in 0..VECTOR_DIM{
		sum += (v1[i] - v2[i]).powf(2.0);
	}
	sum
}

