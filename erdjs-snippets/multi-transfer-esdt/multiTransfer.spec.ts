import { TokenPayment } from "@elrondnetwork/erdjs";
import { createAirdropService, createESDTInteractor, INetworkProvider, ITestSession, ITestUser, TestSession } from "@elrondnetwork/erdjs-snippets";
import { assert } from "chai";
import { createInteractor } from "./multiTransferInteractor";

describe("multi-transfer-esdt snippet", async function () {
    this.bail(true);

    let suite = this;
    let session: ITestSession;
    let provider: INetworkProvider;
    let whale: ITestUser;
    let userOne: ITestUser; // No balance for token, no connection for token ONE
    let userTwo: ITestUser; // Has balance, token TWO isn't frozen
    let userThree: ITestUser; // Has no balance, but token THREE is frozen
    let userFour: ITestUser; // Has balance, token FOUR is frozen

    this.beforeAll(async function () {
        session = await TestSession.loadOnSuite("devnet", suite);
        provider = session.networkProvider;
        whale = session.users.getUser("whale");
        userOne = session.users.getUser("one");
        userTwo = session.users.getUser("one");
        userThree = session.users.getUser("one");
        userFour = session.users.getUser("one");
        await session.syncNetworkConfig();
    });

    it("airdrop EGLD", async function () {
        session.expectLongInteraction(this);

        let payment = TokenPayment.egldFromAmount(0.1);
        await session.syncUsers([whale]);
        await createAirdropService(session).sendToEachUser(whale, [userOne, userTwo, userThree, userFour], [payment]);
    });

    it("issue test tokens", async function () {
        session.expectLongInteraction(this);

        let interactor = await createESDTInteractor(session);
        await session.syncUsers([whale]);
        await session.saveToken("testTokenOne", await interactor.issueFungibleToken(whale, { name: "ONE", ticker: "ONE", decimals: 0, supply: "100000000" }));
        await session.saveToken("testTokenTwo", await interactor.issueFungibleToken(whale, { name: "TWO", ticker: "TWO", decimals: 0, supply: "100000000" }));
        await session.saveToken("testTokenThree", await interactor.issueFungibleToken(whale, { name: "THREE", ticker: "THREE", decimals: 0, supply: "100000000" }));
        await session.saveToken("testTokenFour", await interactor.issueFungibleToken(whale, { name: "FOUR", ticker: "FOUR", decimals: 0, supply: "100000000" }));
    });

    it("airdrop test token", async function () {
        session.expectLongInteraction(this);

        // User one does not receive anything
        // User two receives some TWO
        // User three receives some THREE, but then spends it all
        // User four receives some FOUR

        let tokenTwo = await session.loadToken("testTokenTwo");
        let tokenThree = await session.loadToken("testTokenThree");
        let tokenFour = await session.loadToken("testTokenFour");
        let airdrop = createAirdropService(session);

        await session.syncUsers([whale, userThree]);

        await airdrop.sendToEachUser(whale, [userTwo], [TokenPayment.fungibleFromAmount(tokenTwo.identifier, "1000", tokenTwo.decimals)]);
        await airdrop.sendToEachUser(whale, [userThree], [TokenPayment.fungibleFromAmount(tokenThree.identifier, "1000", tokenThree.decimals)]);
        await airdrop.sendToEachUser(userThree, [whale], [TokenPayment.fungibleFromAmount(tokenThree.identifier, "1000", tokenThree.decimals)]);
        await airdrop.sendToEachUser(whale, [userFour], [TokenPayment.fungibleFromAmount(tokenFour.identifier, "1000", tokenFour.decimals)]);
    });

    it("setup", async function () {
        session.expectLongInteraction(this);

        await session.syncUsers([whale]);

        let interactor = await createInteractor(session);
        let { address, returnCode } = await interactor.deploy(whale);

        assert.isTrue(returnCode.isSuccess());

        await session.saveAddress("contractAddress", address);
    });

    it("am I frozen?", async function () {
        let contractAddress = await session.loadAddress("contractAddress");
        
        let tokenOne = await session.loadToken("testTokenOne");
        let tokenTwo = await session.loadToken("testTokenTwo");
        let tokenThree = await session.loadToken("testTokenThree");
        let tokenFour = await session.loadToken("testTokenFour");
        
        let interactor = await createInteractor(session, contractAddress);
        
        console.log("one:", await interactor.amIFrozen(userOne, tokenOne.identifier));
        console.log("two:", await interactor.amIFrozen(userTwo, tokenTwo.identifier));
        console.log("three:", await interactor.amIFrozen(userThree, tokenThree.identifier));
        console.log("four:", await interactor.amIFrozen(userFour, tokenFour.identifier));
    });
});
