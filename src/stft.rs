use apodize;
use itertools::izip;
use num::complex::Complex;
use num::traits::{Float, Signed, Zero};
use rustfft::{Fft, FftDirection, FftNum, FftPlanner};
use std::str::FromStr;
use std::sync::Arc;

/// easy to use function to calc stft
///
/// implementation can be used as basic example,
/// if length of input_signal is unequal an integer multiple of stepsize + windowsize,
/// the last fitting one is taken and rest of values are thrown away
/// formular: Int * step_size + (windowsize-1) + rest = input_signal.len()
///
/// * `input_signal` - Signal in f32 values to analyze
/// * `window_size` - length of values to anlalyze can be used for overlapping
/// * `step_size` - number of samples in input_signal till next windowed fft
/// * `return` - vector of single FFTs = spectrogram
/// # ToDo: support overlapping windowsize
pub fn calculate_stft(input_signal: &[f32], window_size: usize, step_size: usize) -> Vec<Vec<f32>> {
    let mut spectrogram: Vec<Vec<f32>> = Vec::new();

    // let's initialize our short-time fourier transform
    let window_type: WindowType = WindowType::Hanning;
    let mut stft = STFT::<f32>::new(window_type, window_size, step_size);
    let mut spectrogram_column: Vec<f32> = vec![0.0; stft.output_size()];
    // iterate over all the samples in chunks of step_size samples.
    for some_samples in input_signal.windows(window_size).step_by(step_size) {
        stft.compute_column(some_samples, &mut spectrogram_column[..]);
        spectrogram.push(spectrogram_column.clone());
    }
    spectrogram
}

/// returns `0` if `log10(value).is_negative()`.
/// otherwise returns `log10(value)`.
/// `log10` turns values in domain `0..1` into values
/// in range `-inf..0`.
/// `log10_positive` turns values in domain `0..1` into `0`.
/// this sets very small values to zero which may not be
/// what you want depending on your application.
#[inline]
pub fn log10_positive<T: Float + Signed + Zero>(value: T) -> T {
    // Float.log10
    // Signed.is_negative
    // Zero.zero
    let log = value.log10();
    if log.is_negative() {
        T::zero()
    } else {
        log
    }
}

/// the type of apodization window to use
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum WindowType {
    Hanning,
    Hamming,
    Blackman,
    Nuttall,
    None,
}

impl FromStr for WindowType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        match &lower[..] {
            "hanning" => Ok(WindowType::Hanning),
            "hann" => Ok(WindowType::Hanning),
            "hamming" => Ok(WindowType::Hamming),
            "blackman" => Ok(WindowType::Blackman),
            "nuttall" => Ok(WindowType::Nuttall),
            "none" => Ok(WindowType::None),
            _ => Err("no match"),
        }
    }
}

// this also implements ToString::to_string
impl std::fmt::Display for WindowType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

// TODO write a macro that does this automatically for any enum
static WINDOW_TYPES: [WindowType; 5] = [
    WindowType::Hanning,
    WindowType::Hamming,
    WindowType::Blackman,
    WindowType::Nuttall,
    WindowType::None,
];

impl WindowType {
    pub fn values() -> [WindowType; 5] {
        WINDOW_TYPES
    }
}

pub struct STFT<T>
where
    T: FftNum + FromF64 + num::Float,
{
    pub window_size: usize,
    pub step_size: usize,
    pub fft: Arc<dyn Fft<T>>,
    pub window: Option<Vec<T>>,
    pub real_input: Vec<T>,
    pub complex_input: Vec<Complex<T>>,
    pub complex_output: Vec<Complex<T>>,
    pub scratch_space: Vec<Complex<T>>,
}

impl<T> STFT<T>
where
    T: FftNum + FromF64 + num::Float,
{
    pub fn window_type_to_window_vec(
        window_type: WindowType,
        window_size: usize,
    ) -> Option<Vec<T>> {
        match window_type {
            WindowType::Hanning => Some(
                apodize::hanning_iter(window_size)
                    .map(FromF64::from_f64)
                    .collect(),
            ),
            WindowType::Hamming => Some(
                apodize::hamming_iter(window_size)
                    .map(FromF64::from_f64)
                    .collect(),
            ),
            WindowType::Blackman => Some(
                apodize::blackman_iter(window_size)
                    .map(FromF64::from_f64)
                    .collect(),
            ),
            WindowType::Nuttall => Some(
                apodize::nuttall_iter(window_size)
                    .map(FromF64::from_f64)
                    .collect(),
            ),
            WindowType::None => None,
        }
    }

    pub fn new(window_type: WindowType, window_size: usize, step_size: usize) -> Self {
        let window = Self::window_type_to_window_vec(window_type, window_size);
        Self::new_with_window_vec(window, window_size, step_size)
    }

    // TODO this should ideally take an iterator and not a vec
    pub fn new_with_window_vec(
        window: Option<Vec<T>>,
        window_size: usize,
        step_size: usize,
    ) -> Self {
        // TODO more assertions:
        // window_size is power of two
        // step_size > 0
        assert!(step_size <= window_size);
        let inverse = false;
        let mut planner = FftPlanner::new(); //inverse);
        let fft_dir: FftDirection = if inverse {
            FftDirection::Inverse
        } else {
            FftDirection::Forward
        };
        let fft = planner.plan_fft(window_size, fft_dir);
        let scratch_size = fft.get_inplace_scratch_len();
        STFT {
            window_size: window_size,
            step_size: step_size,
            fft: fft,
            window: window,
            real_input: std::iter::repeat(T::zero()).take(window_size).collect(),
            complex_input: std::iter::repeat(Complex::<T>::zero())
                .take(window_size)
                .collect(),
            complex_output: std::iter::repeat(Complex::<T>::zero())
                .take(window_size)
                .collect(),
            scratch_space: std::iter::repeat(Complex::<T>::zero())
                .take(scratch_size)
                .collect(),
        }
    }

    #[inline]
    pub fn output_size(&self) -> usize {
        self.window_size / 2
    }

    pub fn compute_into_complex_output(&mut self, input: &[T]) {
        assert_eq!(input.len(), self.window_size);

        // multiply real_input with window
        if let Some(ref window) = self.window {
            // copy windowed input as real parts into complex_input
            for (dst, src, window_elem) in
                izip!(self.complex_input.iter_mut(), input.iter(), window.iter(),)
            {
                dst.re = *src * *window_elem;
                dst.im = T::zero();
            }
        } else {
            // copy input as real parts into complex_input
            for (dst, src) in self.complex_input.iter_mut().zip(input.iter()) {
                dst.re = *src;
                dst.im = T::zero();
            }
        }

        // compute fft
        self.fft
            .process_with_scratch(&mut self.complex_input, &mut self.scratch_space);
    }

    /// # Panics
    /// panics unless `self.output_size() == output.len()`
    pub fn compute_complex_column(&mut self, input: &[T], output: &mut [Complex<T>]) {
        assert_eq!(self.output_size(), output.len());

        self.compute_into_complex_output(&input);

        // copy inplace result of fft to output
        for (dst, src) in output.iter_mut().zip(self.complex_input.iter()) {
            *dst = src.clone();
        }
    }

    /// # Panics
    /// panics unless `self.output_size() == output.len()`
    pub fn compute_magnitude_column(&mut self, input: &[T], output: &mut [T]) {
        assert_eq!(self.output_size(), output.len());

        self.compute_into_complex_output(&input);

        // copy inplace result of fft to output
        for (dst, src) in output.iter_mut().zip(self.complex_input.iter()) {
            *dst = src.norm();
        }
    }

    /// computes a column of the spectrogram
    /// # Panics
    /// panics unless `self.output_size() == output.len()`
    pub fn compute_column(&mut self, input: &[T], output: &mut [T]) {
        assert_eq!(self.output_size(), output.len());

        self.compute_into_complex_output(&input);

        // copy inplace result of fft to output
        for (dst, src) in output.iter_mut().zip(self.complex_input.iter()) {
            *dst = log10_positive(src.norm());
        }
    }
}

pub trait FromF64 {
    fn from_f64(n: f64) -> Self;
}

impl FromF64 for f64 {
    fn from_f64(n: f64) -> Self {
        n
    }
}

impl FromF64 for f32 {
    fn from_f64(n: f64) -> Self {
        n as f32
    }
}
