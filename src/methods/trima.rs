use crate::core::Method;
use crate::core::{PeriodType, ValueType};
use crate::methods::SMA;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Triangular Moving Average of specified `length` for timeseries of type [`ValueType`]
///
/// # Parameters
///
/// Has a single parameter `length`: [`PeriodType`]
///
/// `length` should be > 0
///
/// # Input type
///
/// Input type is [`ValueType`]
///
/// # Output type
///
/// Output type is [`ValueType`]
///
/// # Examples
///
/// ```
/// use yata::prelude::*;
/// use yata::methods::TRIMA;
///
/// // TRIMA of length=3
/// let mut trima = TRIMA::new(4, 1.0);
///
/// trima.next(1.0);
/// trima.next(2.0);
///
/// assert_eq!(trima.next(3.0), 1.25);
/// assert_eq!(trima.next(4.0), 1.625);
/// ```
///
/// # Perfomance
///
/// O(1)
///
/// [`ValueType`]: crate::core::ValueType
/// [`PeriodType`]: crate::core::PeriodType

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TRIMA {
	sma1: SMA,
	sma2: SMA,
}

impl Method for TRIMA {
	type Params = PeriodType;
	type Input = ValueType;
	type Output = Self::Input;

	fn new(length: Self::Params, value: Self::Input) -> Self {
		debug_assert!(length > 0, "TRIMA: length should be > 0");

		Self {
			sma1: SMA::new(length, value),
			sma2: SMA::new(length, value),
		}
	}

	#[inline]
	fn next(&mut self, value: Self::Input) -> Self::Output {
		self.sma2.next(self.sma1.next(value))
	}
}

#[cfg(test)]
mod tests {
	#![allow(unused_imports)]
	use super::{Method, TRIMA as TestingMethod};
	use crate::core::ValueType;
	use crate::helpers::RandomCandles;

	#[allow(dead_code)]
	const SIGMA: ValueType = 1e-5;

	#[test]
	fn test_trima_const() {
		use super::*;
		use crate::core::{Candle, Method};
		use crate::methods::tests::test_const;

		for i in 1..30 {
			let input = (i as ValueType + 56.0) / 16.3251;
			let mut method = TestingMethod::new(i, input);

			let output = method.next(input);
			test_const(&mut method, input, output);
		}
	}

	#[test]
	fn test_trima1() {
		let mut candles = RandomCandles::default();

		let mut ma = TestingMethod::new(1, candles.first().close);

		candles.take(100).for_each(|x| {
			assert!((x.close - ma.next(x.close)).abs() < SIGMA);
		});
	}

	#[test]
	fn test_trima() {
		let candles = RandomCandles::default();

		let src: Vec<ValueType> = candles.take(100).map(|x| x.close).collect();

		(1..20).for_each(|sma_length| {
			let mut ma = TestingMethod::new(sma_length, src[0]);
			let mut level2 = Vec::new();

			src.iter().enumerate().for_each(|(i, &x)| {
				let value = ma.next(x);
				let slice_from = i.saturating_sub((sma_length - 1) as usize);
				let slice_to = i;
				let slice = &src[slice_from..=slice_to];

				let mut sum: ValueType = slice.iter().sum();
				if slice.len() < sma_length as usize {
					sum += (sma_length as usize - slice.len()) as ValueType * src.first().unwrap();
				}

				level2.push(sum / sma_length as ValueType);

				let mut sum: ValueType = level2.iter().rev().take(sma_length as usize).sum();
				if level2.len() < sma_length as usize {
					sum += (sma_length as usize - level2.len()) as ValueType * src.first().unwrap();
				}

				assert!((sum / sma_length as ValueType - value).abs() < SIGMA);
			});
		});
	}
}