import path from "path";
import { CodeMetadata, IAddress, Interaction, ResultsParser, ReturnCode, SmartContract, SmartContractAbi, Struct, TransactionWatcher } from "@elrondnetwork/erdjs";
import { INetworkProvider, ITestSession, ITestUser, loadAbiRegistry, loadCode } from "@elrondnetwork/erdjs-snippets";
import { NetworkConfig } from "@elrondnetwork/erdjs-network-providers";

const PathToWasm = path.resolve(__dirname, "..", "..", "multi-transfer-esdt", "output", "multi-transfer-esdt.wasm");
const PathToAbi = path.resolve(__dirname, "..", "..", "multi-transfer-esdt", "output", "multi-transfer-esdt.abi.json");

export async function createInteractor(session: ITestSession, contractAddress?: IAddress): Promise<MultiTransferInteractor> {
    let registry = await loadAbiRegistry(PathToAbi);
    let abi = new SmartContractAbi(registry, ["MultiTransferEsdt"]);
    let contract = new SmartContract({ address: contractAddress, abi: abi });
    let networkProvider = session.networkProvider;
    let networkConfig = session.getNetworkConfig();
    let interactor = new MultiTransferInteractor(contract, networkProvider, networkConfig);
    return interactor;
}

export class MultiTransferInteractor {
    private readonly contract: SmartContract;
    private readonly networkProvider: INetworkProvider;
    private readonly networkConfig: NetworkConfig;
    private readonly transactionWatcher: TransactionWatcher;
    private readonly resultsParser: ResultsParser;

    constructor(contract: SmartContract, networkProvider: INetworkProvider, networkConfig: NetworkConfig) {
        this.contract = contract;
        this.networkProvider = networkProvider;
        this.networkConfig = networkConfig;
        this.transactionWatcher = new TransactionWatcher(networkProvider);
        this.resultsParser = new ResultsParser();
    }

    async deploy(deployer: ITestUser): Promise<{ address: IAddress, returnCode: ReturnCode }> {
        // Load the bytecode from a file.
        let code = await loadCode(PathToWasm);

        // Prepare the deploy transaction.
        let transaction = this.contract.deploy({
            code: code,
            codeMetadata: new CodeMetadata(),
            initArguments: [],
            gasLimit: 60000000,
            chainID: this.networkConfig.ChainID
        });

        // Set the transaction nonce. The account nonce must be synchronized beforehand.
        // Also, locally increment the nonce of the deployer (optional).
        transaction.setNonce(deployer.account.getNonceThenIncrement());

        // Let's sign the transaction. For dApps, use a wallet provider instead.
        await deployer.signer.sign(transaction);

        // The contract address is deterministically computable:
        let address = SmartContract.computeAddress(transaction.getSender(), transaction.getNonce());

        // Let's broadcast the transaction and await its completion:
        await this.networkProvider.sendTransaction(transaction);
        let transactionOnNetwork = await this.transactionWatcher.awaitCompleted(transaction);

        // In the end, parse the results:
        let { returnCode } = this.resultsParser.parseUntypedOutcome(transactionOnNetwork);

        console.log(`MultiTransferInteractor.deploy(): contract = ${address}`);
        return { address, returnCode };
    }

    async amIFrozen(user: ITestUser, tokenIdentifier: string): Promise<string> {
        // Prepare the interaction
        let interaction = <Interaction>this.contract.methods.amIFrozen([tokenIdentifier]).withQuerent(user.address);
        let query = interaction.check().buildQuery();

        // Let's run the query and parse the results:
        let queryResponse = await this.networkProvider.queryContract(query);
        let { firstValue } = this.resultsParser.parseQueryResponse(queryResponse, interaction.getEndpoint());

        // Now let's interpret the results.
        let firstValueAsBuffer = <Buffer>firstValue!.valueOf();
        return firstValueAsBuffer.toString();
    }
}
