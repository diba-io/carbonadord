use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Mint {
  #[clap(long, help = "Use <FEE_RATE> sats/vbyte for mint transaction.")]
  fee_rate: FeeRate,
  #[clap(long, help = "Mint <RUNE>. May contain `.` or `•`as spacers.")]
  rune: SpacedRune,
  #[clap(
    long,
    help = "Include <AMOUNT> postage with mint output. [default: 10000sat]"
  )]
  postage: Option<Amount>,
  #[clap(long, help = "Send minted runes to <DESTINATION>.")]
  destination: Option<Address<NetworkUnchecked>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
  pub rune: SpacedRune,
  pub pile: Pile,
  pub mint: String,
}

#[derive(Deserialize)]
pub struct MintRequest {
  rune: SpacedRune,
  // destination: Address<NetworkUnchecked>,
  // postage: u64,
}

impl Mint {
  pub(crate) fn post(mint_req: MintRequest) -> Result<Output> {
    let settings = Settings::load(Options::default())?;

    let wallet = WalletConstructor::construct(
      "ord".to_owned(),
      true,
      settings.clone(),
      settings
        .server_url()
        .unwrap_or("http://127.0.0.1:80")
        .parse::<Url>()
        .context("invalid server URL")?,
    )?;

    ensure!(
      wallet.has_rune_index(),
      "`ord wallet mint` requires index created with `--index-runes` flag",
    );

    let rune = mint_req.rune.rune;

    let bitcoin_client = wallet.bitcoin_client();

    let block_height = bitcoin_client.get_block_count()?;

    let Some((id, rune_entry, _)) = wallet.get_rune(rune)? else {
      bail!("rune {rune} has not been etched");
    };

    // let postage = Amount::from_sat(mint_req.postage);

    let amount = rune_entry
      .mintable(block_height)
      .map_err(|err| anyhow!("rune {rune} {err}"))?;

    // let chain = wallet.chain();

    // let destination = mint_req.destination.require_network(chain.network())?;

    // ensure!(
    //   destination.script_pubkey().dust_value() < postage,
    //   "postage below dust limit of {}sat",
    //   destination.script_pubkey().dust_value().to_sat()
    // );

    let runestone = Runestone {
      mint: Some(id),
      ..default()
    };

    let script_pubkey = runestone.encipher();

    // Libre Relay exempts this
    // ensure!(
    //   script_pubkey.len() <= 82,
    //   "runestone greater than maximum OP_RETURN size: {} > 82",
    //   script_pubkey.len()
    // );

    // let unfunded_transaction = Transaction {
    //   version: 2,
    //   lock_time: LockTime::ZERO,
    //   input: Vec::new(),
    //   output: vec![
    //     TxOut {
    //       script_pubkey,
    //       value: 0,
    //     },
    //     TxOut {
    //       script_pubkey: destination.script_pubkey(),
    //       value: mint_req.postage,
    //     },
    //   ],
    // };

    // unfunded_transaction.raw_hex();

    // let psbt = consensus::encode::serialize_hex(&unfunded_transaction);

    // wallet.lock_non_cardinal_outputs()?;

    // let unsigned_transaction =
    //   fund_raw_transaction(bitcoin_client, self.fee_rate, &unfunded_transaction)?;

    // let signed_transaction = bitcoin_client
    //   .sign_raw_transaction_with_wallet(&unsigned_transaction, None, None)?
    //   .hex;

    // let signed_transaction = consensus::encode::deserialize(&signed_transaction)?;

    // assert_eq!(
    //   Runestone::decipher(&signed_transaction),
    //   Some(Artifact::Runestone(runestone)),
    // );

    // let transaction = bitcoin_client.send_raw_transaction(&signed_transaction)?;

    Ok(Output {
      rune: mint_req.rune,
      pile: Pile {
        amount,
        divisibility: rune_entry.divisibility,
        symbol: rune_entry.symbol,
      },
      mint: script_pubkey.to_hex_string(),
    })
  }
}
