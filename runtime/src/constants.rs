// This file is part of Diora.

// Copyright (C) 2019-2022 Diora-Network.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

pub use currency::*;
/// DIOR, the native token, uses 18 decimals of precision.
pub mod currency {
	use crate::Balance;

	// Provide a common factor between runtimes based on a supply of 10_000_000 tokens.
	pub const SUPPLY_FACTOR: Balance = 100;

	pub const WEI: Balance = 1;
	pub const KILOWEI: Balance = 1_000;
	pub const MEGAWEI: Balance = 1_000_000;
	pub const GIGAWEI: Balance = 1_000_000_000;
	pub const MICRODIOR: Balance = 1_000_000_000_000;
	pub const MILLIDIOR: Balance = 1_000_000_000_000_000;
	pub const DIOR: Balance = 1_000_000_000_000_000_000;
	pub const KILODIOR: Balance = 1_000_000_000_000_000_000_000;

	pub const TRANSACTION_BYTE_FEE: Balance = 10 * MICRODIOR * SUPPLY_FACTOR;
	pub const STORAGE_BYTE_FEE: Balance = 100 * MICRODIOR * SUPPLY_FACTOR;
	pub const WEIGHT_FEE: Balance = 100 * KILOWEI * SUPPLY_FACTOR;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 100 * MILLIDIOR * SUPPLY_FACTOR + (bytes as Balance) * STORAGE_BYTE_FEE
	}
}
/// Fee-related.
pub mod fee {
	use crate::Balance;
	use frame_support::weights::{
		constants::ExtrinsicBaseWeight, WeightToFeeCoefficient, WeightToFeeCoefficients,
		WeightToFeePolynomial,
	};
	use smallvec::smallvec;
	pub use sp_runtime::Perbill;

	/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
	/// node's balance type.
	///
	/// This should typically create a mapping between the following ranges:
	///   - `[0, MAXIMUM_BLOCK_WEIGHT]`
	///   - `[Balance::min, Balance::max]`
	///
	/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
	///   - Setting it to `0` will essentially disable the weight fee.
	///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
	pub struct WeightToFee;
	impl WeightToFeePolynomial for WeightToFee {
		type Balance = Balance;
		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// in Rococo, extrinsic base weight (smallest non-zero weight) is mapped to 1 MILLIUNIT:
			// in our template, we map to 1/10 of that, or 1/10 MILLIUNIT
			let p = super::currency::MILLIDIOR / 10;
			let q = 100 * Balance::from(ExtrinsicBaseWeight::get().ref_time());
			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}
}
