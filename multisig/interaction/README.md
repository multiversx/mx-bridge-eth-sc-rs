## Using snippets script for the bridge SCs

### Step 1: Update [configs.cfg](config/configs.cfg)
- Update Alice(owner) and initial relayers' pem location.
- Update any other SC setting that you would like to change.

### Step 2: Run script file:
Run script.sh with a given command.
Available commands are:
- deploy-aggregator
- deploy-wrapper
- deploy-bridge-contracts
- add-relayer
- remove-relayer
- whitelist-token
- set-safe-max-tx
- set-safe-batch-block-duration
- change-quorum
- pause-contracts
- unpause-contracts

All the commands that are changing any SC settings will automatically update also [configs.cfg](config/configs.cfg). However, there are some points (like token issueing) when the admin will be ask to first update the configs before proceeding with next steps.