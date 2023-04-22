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


use {
	precompile_utils::{EvmResult, prelude::*, testing::PrecompileTesterExt},
	sp_core::H160
};

pub struct PrecompileSet;

#[precompile_utils_macro::precompile]
#[precompile::precompile_set]
impl PrecompileSet {
	#[precompile::discriminant]
	fn discriminant(_: H160) -> Option<()> {
		Some(())
	}

	#[precompile::public("default()")]
	fn default(_: (), _: &mut impl PrecompileHandle) -> EvmResult {
		Ok(())
	}

	#[precompile::public("view()")]
	#[precompile::view]
	fn view(_: (), _: &mut impl PrecompileHandle) -> EvmResult {
		Ok(())
	}

	#[precompile::public("payable()")]
	#[precompile::payable]
	fn payable(_: (), _: &mut impl PrecompileHandle) -> EvmResult {
		Ok(())
	}
}

fn main() {
	PrecompileSet.prepare_test(
		[0u8;20],
		[0u8;20],
		PrecompileSetCall::default {}
	).with_value(1)
	.execute_reverts(|output| output == b"Function is not payable");

	PrecompileSet.prepare_test(
		[0u8;20],
		[0u8;20],
		PrecompileSetCall::default {}
	).with_static_call(true)
	.execute_reverts(|output| output == b"Can't call non-static function in static context");

	PrecompileSet.prepare_test(
		[0u8;20],
		[0u8;20],
		PrecompileSetCall::view {}
	).with_value(1)
	.execute_reverts(|output| output == b"Function is not payable");

	PrecompileSet.prepare_test(
		[0u8;20],
		[0u8;20],
		PrecompileSetCall::view {}
	).with_static_call(true)
	.execute_returns_encoded(());

	PrecompileSet.prepare_test(
		[0u8;20],
		[0u8;20],
		PrecompileSetCall::payable {}
	).with_value(1)
	.execute_returns_encoded(());

	PrecompileSet.prepare_test(
		[0u8;20],
		[0u8;20],
		PrecompileSetCall::payable {}
	).with_static_call(true)
	.execute_reverts(|output| output == b"Can't call non-static function in static context");
}