# Ethereum fee estimator - prepay smart contract

Estimates the fee required for an ethereum transaction and transfers it from a deposit that the user has paid in advance.

## API

### Balance
All users have their own EGLD balance. The following endpoints can be used to interact with it:
- `deposit` - payable endpoint; adds the paid sum to the user's balance
- `withdraw` - requests a withdrawal from the user's balance back to the user's wallet
Arguments:
  - `amount` - the amount the user wishes to withdraw
- `getDepositBalance` - check the caller's available balance

### Fee payment
Callable only by whitelisted addresses.
- `payFee` - computes an estimate of the fee based on live data received from the chainlink aggregator and transfers it to the relayer
Arguments:
  - `address` - the user who wants to make the payment; the source of the estimated sum transfer
  - `relayer` - the relayer's address; the transfer's destination
  - `action` - the transaction type (ethereum, erc20 etc.) - this is used to compute the gas limit
  - `priority` - the priority (fast, average, low) - used to compute the gas price

### Whitelist management
Callable only by the smart contract owner.
- `addWhitelist` - adds the given `address` to the whitelist
- `removeWhitelist` - removes the given `address` from the whitelist
- `isWhitelisted` - returns true if the given `address` is in the whitelist
- `getWhitelist` - returns a list of all the whitelisted addresses

## Interaction

The user adds EGLD to his own balance via `deposit`.
An estimation can be transfered at any time after that via `pay_fee`, by which the funds are transfered from the user's balance to the relayer's balance.
The relayer can withdraw his fee estimation by calling `withdraw`.
The user can withdraw the remainder of his balance via `withdraw` as well.
