use super::Error;
// use super::Error;

use std::fmt;

/// Trait for creating methods for timeseries
///
/// # Regular methods usage
///
/// ### Iterate over vector's values
///
/// ```
/// use yata::methods::SMA;
/// use yata::prelude::*;
///
/// let s: Vec<_> = vec![1.,2.,3.,4.,5.,6.,7.,8.,9.,10.];
/// let mut ma = SMA::new(2, &s[0]).unwrap();
///
/// s.iter().enumerate().for_each(|(index, value)| {
///     assert_eq!(ma.next(value), (value + s[index.saturating_sub(1)])/2.);
/// });
/// ```
///
/// ### Get a whole new vector over the input vector
///
/// ```
/// use yata::methods::SMA;
/// use yata::prelude::*;
///
/// let s: Vec<_> = vec![1.,2.,3.,4.,5.,6.,7.,8.,9.,10.];
/// let mut ma = SMA::new(2, &s[0]).unwrap();
///
/// let result = ma.over(s);
/// assert_eq!(result.as_slice(), &[1., 1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5, 8.5, 9.5]);
/// ```
///
/// # Be advised
/// There is no `reset` method on the trait. If you need reset a state of the `Method` instance, you should just create a new one.
pub trait Method: fmt::Debug {
	/// Method parameters
	type Params;
	/// Input value type
	type Input: ?Sized;
	/// Output value type
	type Output: Copy;

	/// Static method for creating an instance of the method with given `parameters` and initial `value` (simply first input value)
	fn new(parameters: Self::Params, initial_value: &Self::Input) -> Result<Self, Error>
	where
		Self: Sized;

	/// Generates next output value based on the given input `value`
	fn next(&mut self, value: &Self::Input) -> Self::Output;

	/// Returns a name of the method
	fn name(&self) -> &str {
		let parts = std::any::type_name::<Self>().split("::");
		parts.last().unwrap_or_default()
	}

	/// Returns memory size of the method `(size, align)`
	fn memsize(&self) -> (usize, usize)
	where
		Self: Sized,
	{
		(std::mem::size_of::<Self>(), std::mem::align_of::<Self>())
	}

	/// Iterates the `Method` over the given `inputs` slice and returns `Vec` of output values.
	///
	/// # Guarantees
	///
	/// The length of an output `Vec` is always equal to the length of an `inputs` slice.
	/// ```
	/// use yata::methods::SMA;
	/// use yata::prelude::*;
	///
	/// let s: Vec<_> = vec![1.,2.,3.,4.,5.,6.,7.,8.,9.,10.];
	/// let mut ma = SMA::new(5, &s[0]).unwrap();
	///
	/// let result = ma.over(&s);
	/// assert_eq!(result.len(), s.len());
	/// ```
	///
	/// ```
	/// use yata::methods::SMA;
	/// use yata::prelude::*;
	///
	/// let s: Vec<_> = vec![1.,2.,3.,4.,5.,6.,7.,8.,9.,10.];
	/// let mut ma = SMA::new(100, &s[0]).unwrap();
	///
	/// let result = ma.over(&s);
	/// assert_eq!(result.len(), s.len());
	/// ```
	#[inline]
	fn over<S>(&mut self, inputs: S) -> Vec<Self::Output>
	where
		S: AsRef<[Self::Input]>,
		Self::Input: Sized,
		Self: Sized,
	{
		inputs.as_ref().iter().map(|x| self.next(x)).collect()
	}

	/// Creates new `Method` instance and iterates it over the given `inputs` slice and returns `Vec` of output values.
	///
	/// # Guarantees
	///
	/// The length of an output `Vec` is always equal to the length of an `inputs` slice.
	fn new_over<S>(parameters: Self::Params, inputs: S) -> Result<Vec<Self::Output>, Error>
	where
		S: AsRef<[Self::Input]>,
		Self::Input: Sized,
		Self: Sized,
	{
		let inputs_ref = inputs.as_ref();

		if inputs_ref.is_empty() {
			return Ok(Vec::new());
		}

		let mut method = Self::new(parameters, &inputs_ref[0])?;

		Ok(method.over(inputs))
	}
}
