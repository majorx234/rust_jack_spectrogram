use apodize;
use itertools::izip;
use num::complex::Complex;
use num::traits::{Float, Signed, Zero};
use realfft::{FftNum, RealFftPlanner, RealToComplex};
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
    pub rfft: Arc<dyn RealToComplex<T>>,
    pub window: Option<Vec<T>>,
    pub real_input: Vec<T>,
    pub scratch_space: Vec<Complex<T>>,
}

impl<T> STFT<T>
where
    T: FftNum + FromF64 + num::Float + std::ops::MulAssign,
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
        let mut real_planner = RealFftPlanner::<T>::new();
        let rfft = real_planner.plan_fft_forward(window_size);
        let scratch_space = rfft.make_scratch_vec();
        let real_input = rfft.make_input_vec();

        STFT {
            window_size: window_size,
            step_size: step_size,
            rfft: rfft,
            window: window,
            real_input: real_input,
            scratch_space: scratch_space,
        }
    }

    #[inline]
    pub fn output_size(&self) -> usize {
        self.window_size / 2
    }

    pub fn compute_into_complex_output(&mut self, input: &[T], output: &mut [Complex<T>]) {
        assert_eq!(input.len(), self.window_size);

        // multiply real_input with window
        if let Some(ref window) = self.window {
            for (dst, src, window_elem) in
                izip!(self.real_input.iter_mut(), input.iter(), window.iter())
            {
                *dst = *src * *window_elem;
            }
        } else {
            for (dst, src) in self.real_input.iter_mut().zip(input.iter()) {
                *dst = *src;
            }
        };
        // compute fft
        let _ =
            self.rfft
                .process_with_scratch(&mut self.real_input, output, &mut self.scratch_space);
    }

    /// # Panics
    /// panics unless `self.output_size() == output.len()`
    pub fn compute_magnitude_column(&mut self, input: &mut [T], output: &mut [T]) {
        assert_eq!(self.output_size(), output.len());
        let mut complex_output = self.rfft.make_output_vec();
        self.compute_into_complex_output(input, &mut complex_output);

        for (dst, src) in output.iter_mut().zip(complex_output.iter()) {
            *dst = src.norm();
        }
    }

    /// computes a column of the spectrogram
    /// # Panics
    /// panics unless `self.output_size() == output.len()`
    pub fn compute_column(&mut self, input: &[T], output: &mut [T]) {
        assert_eq!(self.output_size(), output.len());

        let mut complex_output = self.rfft.make_output_vec();
        self.compute_into_complex_output(input, &mut complex_output);

        for (dst, src) in output.iter_mut().zip(complex_output.iter()) {
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
