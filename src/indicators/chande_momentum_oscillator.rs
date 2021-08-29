#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::core::{Error, Method, PeriodType, Source, ValueType, Window, OHLCV};
use crate::core::{IndicatorConfig, IndicatorInstance, IndicatorResult};
use crate::methods::{Change, CrossAbove, CrossUnder};

/// Chande Momentum Oscillator
///
/// ## Links
///
/// * <https://www.investopedia.com/terms/c/chandemomentumoscillator.asp>
///
/// # 1 value
///
/// * `oscillator` value
///
/// Range in \[`-1.0`; `1.0`\]
///
/// # 1 signal
///
/// When `oscillator` value goes above `zone`, then returns full sell signal.
/// When `oscillator` value goes below `-zone`, then returns full buy signal.
/// Otherwise no signal
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChandeMomentumOscillator {
	/// main period length. Default is `9`.
	///
	/// Range in \[`2`; [`PeriodType::MAX`](crate::core::PeriodType)\]
	pub period: PeriodType,
	/// Zone size of overbought and oversold. Default is `0.5`.
	///
	/// Range in \[`0.0`; `1.0`\]
	pub zone: ValueType,
	/// Source type. Default is [`Close`](crate::core::Source::Close)
	pub source: Source,
}

impl IndicatorConfig for ChandeMomentumOscillator {
	type Instance = ChandeMomentumOscillatorInstance;

	const NAME: &'static str = "ChandeMomentumOscillator";

	fn init<T: OHLCV>(self, candle: &T) -> Result<Self::Instance, Error> {
		if !self.validate() {
			return Err(Error::WrongConfig);
		}

		let cfg = self;

		Ok(Self::Instance {
			pos_sum: 0.,
			neg_sum: 0.,
			change: Change::new(1, &candle.source(cfg.source))?,
			window: Window::new(cfg.period, 0.),
			cross_under: CrossUnder::default(),
			cross_above: CrossAbove::default(),
			cfg,
		})
	}

	fn validate(&self) -> bool {
		self.zone >= 0. && self.zone <= 1.0 && self.period > 1
	}

	fn set(&mut self, name: &str, value: String) -> Result<(), Error> {
		match name {
			"period" => match value.parse() {
				Err(_) => return Err(Error::ParameterParse(name.to_string(), value.to_string())),
				Ok(value) => self.period = value,
			},
			"zone" => match value.parse() {
				Err(_) => return Err(Error::ParameterParse(name.to_string(), value.to_string())),
				Ok(value) => self.zone = value,
			},
			"source" => match value.parse() {
				Err(_) => return Err(Error::ParameterParse(name.to_string(), value.to_string())),
				Ok(value) => self.source = value,
			},

			_ => {
				return Err(Error::ParameterParse(name.to_string(), value));
			}
		};

		Ok(())
	}

	fn size(&self) -> (u8, u8) {
		(1, 1)
	}
}

impl Default for ChandeMomentumOscillator {
	fn default() -> Self {
		Self {
			period: 9,
			zone: 0.5,
			source: Source::Close,
		}
	}
}

#[derive(Debug, Clone)]
pub struct ChandeMomentumOscillatorInstance {
	cfg: ChandeMomentumOscillator,

	pos_sum: ValueType,
	neg_sum: ValueType,
	change: Change,
	window: Window<ValueType>,
	cross_under: CrossUnder,
	cross_above: CrossAbove,
}

#[inline]
fn change(change: ValueType) -> (ValueType, ValueType) {
	// let pos = if change > 0. { change } else { 0. };
	// let neg = if change < 0. { change * -1. } else { 0. };
	let pos = (change > 0.) as i8 as ValueType * change;
	let neg = (change < 0.) as i8 as ValueType * -change;

	(pos, neg)
}

impl IndicatorInstance for ChandeMomentumOscillatorInstance {
	type Config = ChandeMomentumOscillator;

	fn config(&self) -> &Self::Config {
		&self.cfg
	}

	fn next<T: OHLCV>(&mut self, candle: &T) -> IndicatorResult {
		let ch = self.change.next(&candle.source(self.cfg.source));

		let left_value = self.window.push(ch);

		let (left_pos, left_neg) = change(left_value);
		let (right_pos, right_neg) = change(ch);

		self.pos_sum += right_pos - left_pos;
		self.neg_sum += right_neg - left_neg;

		let value = if self.pos_sum != 0. || self.neg_sum != 0. {
			(self.pos_sum - self.neg_sum) / (self.pos_sum + self.neg_sum)
		} else {
			0.
		};
		let signal = self.cross_under.next(&(value, -self.cfg.zone))
			- self.cross_above.next(&(value, self.cfg.zone));

		IndicatorResult::new(&[value], &[signal])
	}
}
