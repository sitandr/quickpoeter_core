use crate::translator_struct::Word;
use crate::reader::{VECTOR_DIM};
use crate::finder::WordCollector;
use ordered_float::NotNan;

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

		if n > 1{
			let m = sigma.iter().map(|x| NotNan::new(*x).unwrap()).min().unwrap().into_inner();
			for j in 0..VECTOR_DIM{
				sigma[j] /= m;
			}
		}

		MeanField{average: average, sigmas: Some(sigma)}
	}

	/// this method is needed because words can miss meaning; this would not be possible if using words from WordCollector
	pub fn from_words<'a>(words: &'a Vec<Word>) -> Result<Self, Vec<&'a str>>{
		map_with_failures(words.iter(), |w| w.meaning)
			.and_then(|vecs| Ok(Self::new(vecs)))
			.or_else(|err_words| Err(err_words.iter().map(|w| &*w.src).collect()))
	}

	pub fn from_strings<'a>(wc: &WordCollector, strs: &'a Vec<String>) -> Result<Self, Vec<&'a str>>{
		Self::from_str(wc, &strs.iter().map(|s| &**s).collect())
	}

	pub fn from_str<'a>(wc: &WordCollector, strs: &Vec<&'a str>) -> Result<Self, Vec<&'a str>>{
		map_with_failures(strs.iter().map(|s| *s), |s| wc.get_meaning(s)).and_then(|vecs| Ok(Self::new(vecs)))
	}

	fn from_single(vector: [f32;VECTOR_DIM]) -> MeanField{
		MeanField{average: vector, sigmas: None}
	}

	pub fn dist(&self, vector: [f32;VECTOR_DIM]) -> f32{

		if let Some(sigma) = self.sigmas{
			let mut dist: f32 = 0.0;
			for i in 0..VECTOR_DIM{
				dist += ((vector[i] - self.average[i]).abs()).powf(0.5)/sigma[i]/10.0;
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

