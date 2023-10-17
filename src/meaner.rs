/*
Rust implementation of advanced ryhmes finder
Copyright (C) 2022  Andrej Sitnikov (sitandr, andr-sitnikov@mail.ru)

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.


Module that keeps logic for getting meaning distances and creating "themes"
*/

use crate::finder::WordCollector;
use crate::reader::{MeaningSettings, VECTOR_DIM};
use ordered_float::NotNan;
use std::ops::Deref;

pub struct MeanTheme {
    average: [f32; VECTOR_DIM],
    /// None if only one vect is present
    sigmas: Option<[f32; VECTOR_DIM]>, // "weights" to the difference between array and average
}

impl MeanTheme {
    /// will panic if no vectors are provided
    pub fn new(vectors: Vec<[f32; VECTOR_DIM]>) -> Self {
        let n = vectors.len();

        if n == 1 {
            return Self::from_single(vectors[0]);
        }
        let mut average = [0.0; VECTOR_DIM];
        let mut sigma = [0.0; VECTOR_DIM];
        let mut norm = 0.0;

        for j in 0..VECTOR_DIM {
            for i in 0..n {
                average[j] += vectors[i][j];
            }

            average[j] /= n as f32;
            norm += average[j].powf(2.0);

            if n > 1 {
                for i in 0..n {
                    sigma[j] += (vectors[i][j] - average[j]).powf(2.0);
                }
                sigma[j] /= (n - 1) as f32;
                sigma[j] = sigma[j].sqrt();
            }
        }
        norm = norm.sqrt();
        for j in 0..VECTOR_DIM {
            average[j] /= norm;
        }

        if n > 1 {
            let m = sigma
                .iter()
                .map(|x| NotNan::new(*x).unwrap())
                .min()
                .unwrap()
                .into_inner();
            for j in 0..VECTOR_DIM {
                sigma[j] /= m;
            }
        }

        MeanTheme {
            average,
            sigmas: Some(sigma),
        }
    }

    pub fn try_new(vectors: Vec<[f32; VECTOR_DIM]>) -> Option<Self> {
        if vectors.is_empty() {
            None
        } else {
            Some(Self::new(vectors))
        }
    }

    /// just skips the words word collector doesn't know
    /// returns None if no words are left
    #[allow(dead_code)]
    pub fn from_strings_filter<S>(wc: &WordCollector, strs: &Vec<S>) -> Option<Self>
    where
        S: Deref<Target = str>,
    {
        let vects: Vec<[f32; VECTOR_DIM]> = strs.iter().filter_map(|s| wc.get_meaning(s)).collect();
        Self::try_new(vects)
    }

    /// empty vec err if no strings are provided
    pub fn from_str<'a, S>(wc: &WordCollector, strs: &'a Vec<S>) -> Result<Self, Vec<&'a S>>
    where
        S: Deref<Target = str>,
    {
        let vects: Vec<[f32; VECTOR_DIM]> = map_with_failures(strs.iter(), |s| wc.get_meaning(s))?;
        Self::try_new(vects).ok_or(vec![])
    }

    fn from_single(vector: [f32; VECTOR_DIM]) -> Self {
        MeanTheme {
            average: vector,
            sigmas: None,
        }
    }

    pub fn dist(&self, vector: [f32; VECTOR_DIM], sett: &MeaningSettings) -> f32 {
        if let Some(sigma) = self.sigmas {
            let mut dist: f32 = 0.0;
            for i in 0..VECTOR_DIM {
                dist += ((vector[i] - self.average[i]).abs()).powf(sett.pow) / (sigma[i]);
                //println!("{}", sigma[i]);
            }
            dist
        } else {
            dist_arrays(self.average, vector, sett.single_pow) * sett.single_weight
        }
    }
}

pub fn map_with_failures<'a, T, U, F, I>(iter: I, f: F) -> Result<Vec<U>, Vec<T>>
where
    F: Fn(&T) -> Option<U>,
    I: Iterator<Item = T>,
{
    use itertools::{Either, Itertools};
    let (success, failured): (Vec<U>, Vec<T>) = iter.partition_map(|s| match f(&s) {
        Some(vec) => Either::Left(vec),
        None => Either::Right(s),
    });

    if failured.is_empty() {
        Ok(success)
    } else {
        Err(failured)
    }
}

pub fn dist_arrays(v1: [f32; VECTOR_DIM], v2: [f32; VECTOR_DIM], single_pow: f32) -> f32 {
    let mut sum = 0.0;
    for i in 0..VECTOR_DIM {
        sum += (v1[i] - v2[i]).abs().powf(single_pow);
    }
    sum
}
