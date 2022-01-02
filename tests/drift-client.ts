import * as anchor from '@project-serum/anchor';
import { assert } from 'chai';

import { Admin } from '../deps/protocol-v1/sdk/src';

import { mockUSDCMint, mockUserUSDCAccount, mockOracle } from './testHelpers';

import { Keypair, PublicKey } from '@solana/web3.js';

import { DriftClient } from '../ts/driftClient';
import { BN } from '@project-serum/anchor';
import {
	MARK_PRICE_PRECISION,
	PositionDirection,
	ZERO,
} from '../deps/protocol-v1/sdk';

describe('drift client', () => {
	const provider = anchor.Provider.local();
	anchor.setProvider(provider);
	let usdcMint: Keypair;
	let userUSDCAccount;

	const clearingHousePublicKey = new PublicKey(
		'AsW7LnXB9UA1uec9wi9MctYTgTz7YH9snhxd16GsFaGX'
	);
	const clearingHouse = Admin.from(
		provider.connection,
		provider.wallet,
		clearingHousePublicKey
	);

	const program = anchor.workspace.DriftClient;

	let driftClient: DriftClient;

	const usdcAmount = new BN(10 * 10 ** 6);

	// ammInvariant == k == x * y
	const mantissaSqrtScale = new BN(Math.sqrt(MARK_PRICE_PRECISION.toNumber()));
	const ammInitialQuoteAssetAmount = new anchor.BN(5 * 10 ** 13).mul(
		mantissaSqrtScale
	);
	const ammInitialBaseAssetAmount = new anchor.BN(5 * 10 ** 13).mul(
		mantissaSqrtScale
	);
	const marketIndex = new BN(0);

	before(async () => {
		usdcMint = await mockUSDCMint(provider);
		userUSDCAccount = await mockUserUSDCAccount(usdcMint, usdcAmount, provider);
		await clearingHouse.initialize(usdcMint.publicKey, true);
		await clearingHouse.subscribeToAll();

		const solUsd = await mockOracle(1);
		const periodicity = new BN(60 * 60); // 1 HOUR

		await clearingHouse.initializeMarket(
			marketIndex,
			solUsd,
			ammInitialBaseAssetAmount,
			ammInitialQuoteAssetAmount,
			periodicity
		);

		driftClient = new DriftClient(program, clearingHouse);
	});

	after(async () => {
		await clearingHouse.unsubscribe();
	});

	it('initialize', async () => {
		await driftClient.initialize();
		const config = await driftClient.getConfig();
		assert(config.admin.equals(provider.wallet.publicKey));
		assert(
			config.collateralVault.equals(
				await driftClient.getCollateralVaultPublicKey()
			)
		);
		assert(config.authority.equals(await driftClient.getAuthorityPublicKey()));
	});

	it('initialize user', async () => {
		await driftClient.initializeUser();
		const userAccountPublicKey =
			await driftClient.getClearingHouseUserAccountPublicKey();
		const userAccount = await clearingHouse.program.account.user.fetch(
			userAccountPublicKey
		);
		const expectedAuthority = await driftClient.getAuthorityPublicKey();
		assert(userAccount.authority.equals(expectedAuthority));
	});

	it('deposit collateral', async () => {
		await driftClient.depositCollateral(usdcAmount, userUSDCAccount.publicKey);
		const userAccount = await driftClient.getUserAccount();
		assert(usdcAmount.eq(userAccount.collateral));
	});

	it('open position', async () => {
		await driftClient.openPosition(
			PositionDirection.LONG,
			usdcAmount,
			marketIndex
		);
		const userPositionsAccount = await driftClient.getUserPositionsAccount();
		const position = userPositionsAccount.positions[0];
		assert(usdcAmount.eq(position.quoteAssetAmount));
	});

	it('close position', async () => {
		await driftClient.closePosition(marketIndex);
		const userPositionsAccount = await driftClient.getUserPositionsAccount();
		const position = userPositionsAccount.positions[0];
		assert(ZERO.eq(position.quoteAssetAmount));
	});

	it('withdraw collateral', async () => {
		const withdrawAmount = new BN(9980000);
		await driftClient.withdrawCollateral(
			withdrawAmount,
			userUSDCAccount.publicKey
		);
		const userAccount = await driftClient.getUserAccount();
		assert(userAccount.collateral.eq(ZERO));
		const adminTokenBalance = new BN(
			(
				await provider.connection.getTokenAccountBalance(
					userUSDCAccount.publicKey
				)
			).value.amount
		);
		assert(adminTokenBalance.eq(withdrawAmount));
	});
});
