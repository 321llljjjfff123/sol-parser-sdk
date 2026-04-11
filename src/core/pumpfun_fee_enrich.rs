//! 同笔交易中 CreateV2 与 Buy 分离为多个 `DexEvent` 时，将 Buy 指令账户 #2（fee recipient）
//! 回填到 `PumpFunCreateV2TokenEvent::observed_fee_recipient`，与链上 `tradeEvent.feeRecipient` 一致。

use solana_sdk::pubkey::Pubkey;

use crate::core::events::DexEvent;

fn pumpfun_buy_like_mint_fee(e: &DexEvent) -> Option<(Pubkey, Pubkey)> {
    match e {
        DexEvent::PumpFunTrade(t) if t.is_buy && t.mint != Pubkey::default() => {
            Some((t.mint, t.fee_recipient))
        }
        DexEvent::PumpFunBuy(t) if t.mint != Pubkey::default() => Some((t.mint, t.fee_recipient)),
        DexEvent::PumpFunBuyExactSolIn(t) if t.mint != Pubkey::default() => {
            Some((t.mint, t.fee_recipient))
        }
        _ => None,
    }
}

/// 扫描同签名下的买入类事件，按 mint 记录 `fee_recipient`（ShredStream 外层的 buy 已从 accounts[1] 解析）。
pub fn enrich_create_v2_observed_fee_recipient(events: &mut [DexEvent]) {
    let mut mint_to_fee: std::collections::HashMap<Pubkey, Pubkey> =
        std::collections::HashMap::new();
    for e in events.iter() {
        if let Some((mint, fee)) = pumpfun_buy_like_mint_fee(e) {
            if fee != Pubkey::default() {
                mint_to_fee.entry(mint).or_insert(fee);
            }
        }
    }
    if mint_to_fee.is_empty() {
        return;
    }
    for e in events.iter_mut() {
        if let DexEvent::PumpFunCreateV2(c) = e {
            if c.observed_fee_recipient == Pubkey::default() {
                if let Some(&f) = mint_to_fee.get(&c.mint) {
                    c.observed_fee_recipient = f;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::events::{EventMetadata, PumpFunCreateV2TokenEvent, PumpFunTradeEvent};
    use solana_sdk::signature::Signature;

    #[test]
    fn enrich_fills_create_v2_from_same_tx_buy() {
        let sig = Signature::default();
        let mint = Pubkey::new_unique();
        let fee = Pubkey::new_unique();
        let meta = EventMetadata {
            signature: sig,
            slot: 1,
            tx_index: 0,
            block_time_us: 0,
            grpc_recv_us: 0,
            recent_blockhash: None,
        };
        let mut events: Vec<DexEvent> = vec![
            DexEvent::PumpFunCreateV2(PumpFunCreateV2TokenEvent {
                metadata: meta.clone(),
                mint,
                ..Default::default()
            }),
            DexEvent::PumpFunTrade(PumpFunTradeEvent {
                metadata: meta,
                mint,
                fee_recipient: fee,
                is_buy: true,
                ..Default::default()
            }),
        ];
        enrich_create_v2_observed_fee_recipient(&mut events);
        if let DexEvent::PumpFunCreateV2(c) = &events[0] {
            assert_eq!(c.observed_fee_recipient, fee);
        } else {
            panic!("expected CreateV2");
        }
    }
}
