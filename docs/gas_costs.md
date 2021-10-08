# Gas costs

M = million
All costs are through multisig SC.
Keep in mind extra gas is returned (unless gas limit is 10X more than actual gas used), so this doesn't have to be exact.

## User Actions

EsdtSafe CreateTransaction: 50M to 60M. ~75M should cover all cases.

## Relayer actions

`stake/unstake` - 35M

generic `propose` with 1-2 arguments (like changeQuorum, for example): 40M

`sign`: 35M

`proposeMultiTransferEsdtBatch`: Base cost, with only batchID argument seems to be around 35M, increasing by approximately 6M per extra pair of (dest_address, tokenId, amount). To cover for most cases, cost should be calculated as 35M base + 15M * nr_transactions_in_batch. (15M might seem a lot, but keep in mind in tests I'm working with numbers in the million range, whereas in real cases, numbers will all be in the ranges of 10^18)

`proposeEsdtSafeSetCurrentTransactionBatchStatus`: Since arguments are statuses (which are all 1 byte), this function is quite cheap regardless of how big the batch is. 40M should cover all cases, but 50M can be used for extra-safety.

`performAction(Ethereum to Elrond batch)`: If all transactions in the batch use a different token, this will cost around 60M base + 20M * nr_transactions. The reason these are so expensive is because the multiTransfer SC has to perform calls to a price-aggregator to estimate the fees. Either way, there is an optimization in place to cache costs if multiple transfers use the same token. Hence, most of the time, the cost per transaction won't be as high. Even so, this estimate should cover for most, if not all, cases.

`fetchNextTransactionBatch`: Base cost seems to be around 40M, with ~10M extra per transaction fetched. This is a bit tricky to estimate, since the relayer has no way of knowing how many transactions will be in the batch before actually fetching it. Having said that, the max number of transactions per batch is currently 10, so around 40M + 10 * 20M = 240M should cover for most cases, so let's say 250M hard-coded gas limit. There is no risk of assigning too much gas here, as even with no transactions, 250M gas limit with actual 40M gas used does not exceed the 10x limitation.


`performAction(Elrond to Ethereum batch, i.e. SetStatus)`: Same ~35M base cost, with approximately 10M more per transaction. Cheaper than the multiTransfer, as fees are calculated when the user creates a deposit. Still, the contract requires minting tokens for each transaction, so to be safe, I suggest 35M + 15M * nr_transactions.

## Summary

| Action      | Recommended Gas limit |
| ----------- | ----------- |
| EsdtSafe CreateTransaction | 75M |
| stake/unstake   | 35M |
| simple propose | 40M |
| sign | 35M |
| proposeMultiTransferEsdtBatch | 35M + 15M * nr_transactions_in_batch |
| proposeEsdtSafeSetCurrentTransactionBatchStatus | 50M |
| performAction(Ethereum to Elrond batch) | 60M + 20M * nr_transactions |
| fetchNextTransactionBatch | 250 M |
| performAction(Elrond to Ethereum batch) | 35M + 15M * nr_transactions |
