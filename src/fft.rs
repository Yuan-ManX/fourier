use crate::radix;
//use multiversion::target_clones;
use num_complex::Complex;

enum Operation<T> {
    Radix2(radix::RadixConfig<T>),
}

//#[target_clones("x86_64+avx")]
#[inline]
fn apply_f32(operation: &Operation<f32>, input: &[Complex<f32>], output: &mut [Complex<f32>]) {
    use radix::radix2_f32;
    match operation {
        Operation::Radix2(config) => radix2_f32(input, output, config),
    }
}

//#[target_clones("x86_64+avx")]
#[inline]
fn forward_f32_in_place(
    operations: &Vec<Operation<f32>>,
    input: &mut [Complex<f32>],
    work: &mut [Complex<f32>],
) {
    //#[static_dispatch]
    //use apply_f32;
    let mut data_in_work = false;
    for op in operations {
        let (from, to): (&mut _, &mut _) = if data_in_work {
            (work, input)
        } else {
            (input, work)
        };
        apply_f32(op, from, to);
        data_in_work ^= true;
    }
    if data_in_work {
        input.copy_from_slice(&work);
    }
}

//#[target_clones("x86_64+avx")]
#[inline]
fn inverse_f32_in_place(
    operations: &Vec<Operation<f32>>,
    input: &mut [Complex<f32>],
    work: &mut [Complex<f32>],
) {
    //#[static_dispatch]
    //use apply_f32;
    let mut data_in_work = false;
    for op in operations {
        let (from, to): (&mut _, &mut _) = if data_in_work {
            (work, input)
        } else {
            (input, work)
        };
        apply_f32(op, from, to);
        data_in_work ^= true;
    }
    let scale = input.len() as f32;
    if data_in_work {
        for (x, y) in work.iter().zip(input.iter_mut()) {
            *y = x / scale;
        }
    } else {
        for x in input.iter_mut() {
            *x /= scale;
        }
    }
}

pub struct Fft32 {
    size: usize,
    operations: Vec<Operation<f32>>,
    work: Box<[Complex<f32>]>,
}

impl Fft32 {
    pub fn new(size: usize) -> Self {
        let mut operations = Vec::new();
        let mut subsize = size;
        let mut stride = 1usize;
        while subsize != 1 {
            if subsize % 2 == 0 {
                let config = radix::RadixConfig::forward(subsize, stride, 2);
                operations.push(Operation::Radix2(config));
                subsize /= 2;
                stride *= 2;
            } else {
                unimplemented!("only radix-2 supported");
            }
        }
        Self {
            size,
            operations,
            work: vec![Complex::default(); size].into_boxed_slice(),
        }
    }

    pub fn fft_in_place(&mut self, input: &mut [Complex<f32>]) {
        assert_eq!(input.len(), self.size, "input must match configured size");
        forward_f32_in_place(&self.operations, input, &mut self.work);
    }

    pub fn ifft_in_place(&mut self, input: &mut [Complex<f32>]) {
        assert_eq!(input.len(), self.size, "input must match configured size");
        inverse_f32_in_place(&self.operations, input, &mut self.work);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unit_128_forward_in_place_f32() {
        let mut input = vec![Complex::new(0f32, 0f32); 128];
        input[0] = Complex::new(1f32, 0f32);
        let mut fft = Fft32::new(128);
        fft.fft_in_place(&mut input);
        for x in input {
            assert!((x - 1.0).norm() < 1e-10);
        }
    }

    #[test]
    fn unit_128_inverse_in_place_f32() {
        let mut input = vec![Complex::new(1f32, 0f32); 128];
        let mut fft = Fft32::new(128);
        fft.ifft_in_place(&mut input);
        assert!((input[0] - 1.0).norm() < 1e-10);
        for x in input.iter().skip(1) {
            assert!(x.norm() < 1e-10);
        }
    }
}
