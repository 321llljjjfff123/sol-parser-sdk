# Pump / Pump AMM 解析账号与数据说明

本文档说明 sol-parser-sdk 中 Pump（pumpfun）与 Pump AMM（pumpswap）交易解析所依赖的 IDL、账号索引及已覆盖/曾遗漏的账号与数据。IDL 已与 sol-trade-sdk 同步。

## IDL 同步

- **来源**: sol-trade-sdk `idl/`
- **目标**: sol-parser-sdk `idls/`
- **已同步文件**:
  - `idl/pump.json` → `idls/pumpfun.json`（Pump 程序，同 program id）
  - `idl/pump_amm.json` → `idls/pump_amm.json`
  - `idl/pump_fees.json` → `idls/pump_fees.json`（可选，供后续 fee sharing 等解析使用）

## Pump（Bonding Curve）Buy / Sell

### 账号索引（与 pump.json 一致）

**Buy：共 15 个固定账户**

| 索引 | 账户名 |
|-----|--------|
| 0 | global |
| 1 | fee_recipient |
| 2 | mint |
| 3 | bonding_curve |
| 4 | associated_bonding_curve |
| 5 | associated_user |
| 6 | user |
| 7 | system_program |
| 8 | token_program |
| 9 | creator_vault |
| 10 | event_authority |
| 11 | program |
| 12 | global_volume_accumulator |
| 13 | user_volume_accumulator |
| 14 | fee_config |

remaining_accounts 可能含 bonding_curve_v2 等。

**Sell：共 14 个固定账户**

| 索引 | 账户名 |
|-----|--------|
| 0 | global |
| 1 | fee_recipient |
| 2 | mint |
| 3 | bonding_curve |
| 4 | associated_bonding_curve |
| 5 | associated_user |
| 6 | user |
| 7 | system_program |
| 8 | creator_vault |
| 9 | token_program |
| 10 | event_authority |
| 11 | program |
| 12 | fee_config |
| 13 | fee_program |

remaining_accounts 可能含 user_volume_accumulator（返现）、bonding_curve_v2 等。

### 解析与填充

- **日志解析**（logs/pump.rs, pump_inner.rs）: TradeEvent 含 `creator`、`creator_fee` 等；**creator_vault 不在事件数据中**，在日志解析里置为 `Pubkey::default()`。
- **账户填充**（account_fillers/pumpfun.rs）: 根据指令账户补全 `creator_vault`、`token_program` 等；Buy 使用索引 9 的 creator_vault，Sell 使用索引 8。**必须通过指令账户填充才能得到正确的 creator_vault**（对 Creator Rewards Sharing 的币，该地址可能为 sharing config PDA）。
- **指令解析**（instr/pump.rs）: Buy/Sell 的完整解析当前未用于主路径（事件来自日志），注释已按 IDL 更新为包含 7–9 及 creator_vault 位置。

### 曾遗漏 / 注意点

- **creator_vault**：事件数据中无此字段，若不做账户填充会一直为 default。sol-trade-sdk 卖出时需要最新 creator_vault（见 README Creator Rewards Sharing）。确保在 gRPC/RPC 解析链路中调用 `fill_trade_accounts`，以便从指令账户 8/9 填入 creator_vault。

## Pump AMM（PumpSwap）Buy / Sell

### 账号索引（与 pump_amm.json 一致）

**Buy：共 23 个固定账户**

| 索引 | 账户名 |
|-----|--------|
| 0 | pool |
| 1 | user |
| 2 | global_config |
| 3 | base_mint |
| 4 | quote_mint |
| 5 | user_base_token_account |
| 6 | user_quote_token_account |
| 7 | pool_base_token_account |
| 8 | pool_quote_token_account |
| 9 | protocol_fee_recipient |
| 10 | protocol_fee_recipient_token_account |
| 11 | base_token_program |
| 12 | quote_token_program |
| 13 | system_program |
| 14 | associated_token_program |
| 15 | event_authority |
| 16 | program |
| 17 | coin_creator_vault_ata |
| 18 | coin_creator_vault_authority |
| 19 | global_volume_accumulator |
| 20 | user_volume_accumulator |
| 21 | fee_config |
| 22 | fee_program |

**Sell：共 21 个固定账户**

| 索引 | 账户名 |
|-----|--------|
| 0 | pool |
| 1 | user |
| 2 | global_config |
| 3 | base_mint |
| 4 | quote_mint |
| 5 | user_base_token_account |
| 6 | user_quote_token_account |
| 7 | pool_base_token_account |
| 8 | pool_quote_token_account |
| 9 | protocol_fee_recipient |
| 10 | protocol_fee_recipient_token_account |
| 11 | base_token_program |
| 12 | quote_token_program |
| 13 | system_program |
| 14 | associated_token_program |
| 15 | event_authority |
| 16 | program |
| 17 | coin_creator_vault_ata |
| 18 | coin_creator_vault_authority |
| 19 | fee_config |
| 20 | fee_program |

### 解析与填充

- **指令解析**（instr/pump_amm.rs）: 已按 IDL 补全 17、18；当 `accounts.len() >= 19` 时，从 17、18 读取并写入 `coin_creator_vault_ata`、`coin_creator_vault_authority`。0–12 的解析保持不变。
- **账户填充**（account_fillers/pumpswap.rs）: 仍从 17、18 填充 `coin_creator_vault_ata`、`coin_creator_vault_authority`（与 IDL 一致）。
- **日志解析**（logs/pump_amm.rs）: 事件数据中含 `coin_creator`、`coin_creator_fee` 等；`coin_creator_vault_ata` / `coin_creator_vault_authority` 需由指令账户或填充器提供。

### 曾遗漏 / 已修复

- **coin_creator_vault_ata、coin_creator_vault_authority**：原先指令解析只用到 0–12，未读 17、18。现已在 `parse_buy_instruction`、`parse_buy_exact_quote_in_instruction`、`parse_sell_instruction` 中在 `accounts.len() >= 19` 时写入上述两字段。

## 数据字段小结

| 程序 | 字段 | 来源 | 说明 |
|------|------|------|------|
| Pump | creator_vault | 指令账户 8(sell)/9(buy)，经 fill_trade_accounts | 必填；Creator Rewards Sharing 时需最新值 |
| Pump | creator, creator_fee 等 | 日志 TradeEvent | 已有 |
| Pump AMM | coin_creator_vault_ata, coin_creator_vault_authority | 指令账户 17、18 | 已补全到指令解析与填充器 |
| Pump AMM | coin_creator, coin_creator_fee 等 | 日志事件 | 已有 |

## 建议

1. 使用 Pump 事件构建卖出参数时，务必在合并/下发前调用 **fill_trade_accounts**，以便 `creator_vault` 来自当前指令账户，避免 2006 seeds 错误。
2. 保持 IDL 与 sol-trade-sdk 定期同步（复制 `idl/*.json` → `idls/`），以便新指令或新账户加入时解析与注释仍正确。
