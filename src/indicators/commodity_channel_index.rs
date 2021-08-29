#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::core::{Action, Error, Method, PeriodType, Source, ValueType, OHLCV};
use crate::core::{IndicatorConfig, IndicatorInstance, IndicatorResult};
use crate::methods::CCI;

const SCALE: ValueType = 1.0 / 1.5;
/// Commodity Channel Index
///
/// ## Links
///
/// * <https://en.wikipedia.org/wiki/Commodity_channel_index>
///
/// # 1 value
///
/// * `oscillator` value. Most of the time value is in the range around \[`-1.0`; `+1.0`\]
///
/// Range in \(`-inf`; `+inf`\)
///
/// # 1 signal
///
/// When `oscillator` value goes above `zone`, then returns full sell signal.
/// When `oscillator` value goes below `-zone`, then returns full buy signal.
/// Otherwise no signal
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CommodityChannelIndex {
	/// Main period size. Default is `18`.
	///
	/// Range in \[`2`; [`PeriodType::MAX`](crate::core::PeriodType)\).
	pub period: PeriodType,

	/// Signal zone size. Default is `1.0`.
	///
	/// Range in \[`0.0`; `+inf`\).
	pub zone: ValueType,

	/// Source type. Default is [`Close`](crate::core::Source::Close).
	pub source: Source,
}

impl IndicatorConfig for CommodityChannelIndex {
	type Instance = CommodityChannelIndexInstance;

	const NAME: &'static str = "CommodityChannelIndex";

	fn init<T: OHLCV>(self, candle: &T) -> Result<Self::Instance, Error> {
		if !self.validate() {
			return Err(Error::WrongConfig);
		}

		let cfg = self;
		let value = candle.source(cfg.source);

		Ok(Self::Instance {
			last_cci: 0.,
			last_signal: 0,
			cci: CCI::new(cfg.period, &value)?,

			cfg,
		})
	}

	fn validate(&self) -> bool {
		self.zone >= 0.0 && self.period > 1 && self.period < PeriodType::MAX
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

impl Default for CommodityChannelIndex {
	fn default() -> Self {
		Self {
			period: 18,
			zone: 1.0,
			source: Source::Close,
		}
	}
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CommodityChannelIndexInstance {
	cfg: CommodityChannelIndex,

	cci: CCI,
	last_cci: ValueType,
	last_signal: i8,
}

impl IndicatorInstance for CommodityChannelIndexInstance {
	type Config = CommodityChannelIndex;

	fn config(&self) -> &Self::Config {
		&self.cfg
	}

	fn next<T: OHLCV>(&mut self, candle: &T) -> IndicatorResult {
		let value = candle.source(self.cfg.source);

		let cci = self.cci.next(&value) * SCALE;

		// let mut t_signal = 0;
		// let mut signal = 0;

		// if cci > self.cfg.zone && self.last_cci <= self.cfg.zone {
		// 	t_signal += 1;
		// }

		// if cci < -self.cfg.zone && self.last_cci >= -self.cfg.zone {
		// 	t_signal -= 1;
		// }

		let t_signal = (cci < -self.cfg.zone && self.last_cci >= -self.cfg.zone) as i8
			- (cci > self.cfg.zone && self.last_cci <= self.cfg.zone) as i8;

		// if t_signal != 0 && self.last_signal != t_signal {
		// 	signal = t_signal;
		// }

		let signal = (t_signal != 0 && self.last_signal != t_signal) as i8 * t_signal;

		self.last_cci = cci;
		self.last_signal = signal;

		IndicatorResult::new(&[cci], &[Action::from(signal)])
	}
}
