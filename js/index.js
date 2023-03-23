import {Contract, Keypair, Networks, Server, xdr, TransactionBuilder,} from 'soroban-client';

import { Account, Operation,  } from "stellar-base";

// TODO: Transaction is immutable, so we need to re-build it here. :(
export function addFootprint(raw, networkPassphrase, footprint) {
    if (!(footprint instanceof xdr.LedgerFootprint)) {
        footprint = xdr.LedgerFootprint.fromXDR(footprint.toString(), "base64");
    }
    if ("innerTransaction" in raw) {
        // TODO: Handle feebump transactions
        return addFootprint(raw.innerTransaction, networkPassphrase, footprint);
    }
    // TODO: Figure out a cleaner way to clone this transaction.
    const source = new Account(raw.source, `${parseInt(raw.sequence, 10) - 1}`);
    const txn = new TransactionBuilder(source, {
        fee: raw.fee,
        memo: raw.memo,
        networkPassphrase,
        timebounds: raw.timeBounds,
        ledgerbounds: raw.ledgerBounds,
        minAccountSequence: raw.minAccountSequence,
        minAccountSequenceAge: raw.minAccountSequenceAge,
        minAccountSequenceLedgerGap: raw.minAccountSequenceLedgerGap,
        extraSigners: raw.extraSigners,
    });
    for (const rawOp of raw.operations) {
        if ("function" in rawOp) {
            // TODO: Figure out a cleaner way to clone these operations
            txn.addOperation(Operation.invokeHostFunction({
                function: rawOp.function,
                parameters: rawOp.parameters,
                footprint,
                auth: []
            }));
        } else {
            // TODO: Handle this.
            throw new Error("Unsupported operation type");
        }
    }
    return txn.build();
}

let server = new Server("https://rpc-futurenet.stellar.org:443/")


//GDXNABYNPAIW2CUK7VN5NLAI7YKWMMGTTLEELWML2J7QNYZVV23NPQVK
let keypair1 = Keypair.fromSecret("SCAIDMBI5BX234O7WZPKBCDVPNHKGZX22UV6R2DO2OGPX3ODMDIQNMNQ")



// dao hash : 1b15c98f783fd8b00d26672fc9bd49eff150d7b8ea73fe0bf056cd4f4ff85abd

//dao token: ba30bd6b7d519e0abadf6dc0fe100eda72fe6be5137a99fc8f44a8cd51d92b92

let account = await server.getAccount(keypair1.publicKey())

 const dao_id = "1b15c98f783fd8b00d26672fc9bd49eff150d7b8ea73fe0bf056cd4f4ff85abd"
const dao_contract = new Contract(dao_id)

const dao_token_id = "ba30bd6b7d519e0abadf6dc0fe100eda72fe6be5137a99fc8f44a8cd51d92b92"
const token_contract = new Contract(dao_token_id);


async function init_token(){
    let fee = '1000';
    let transaction = new TransactionBuilder(account, { fee, networkPassphrase: Networks.FUTURENET })
    .addOperation(
        // An operation to call increment on the contract
        token_contract.call("initialize",
        xdr.ScVal.scvObject(xdr.ScObject.scoAddress(xdr.ScAddress.scAddressTypeContract(Buffer.from("1b15c98f783fd8b00d26672fc9bd49eff150d7b8ea73fe0bf056cd4f4ff85abd","hex")))),
        xdr.ScVal.scvU32(7),
        xdr.ScVal.scvObject(xdr.ScObject.scoBytes(Buffer.from("DAO TOKEN","utf-8"))),
        xdr.ScVal.scvObject(xdr.ScObject.scoBytes(Buffer.from("DCOIN","utf-8")))
        )
    )
    .setTimeout(30)
    .build();


    const simulate_result = await server.simulateTransaction(transaction)

    transaction = addFootprint(transaction,Networks.FUTURENET,simulate_result.results[0].footprint)
    // sign the transaction
    transaction.sign(keypair1);

    try {
        const transactionResult = await server.sendTransaction(transaction);
        console.log(transactionResult);
    } catch (err) {
        console.error(err);
    }
}

async function init_dao(){
    let fee = '1000';
    let transaction = new TransactionBuilder(account, { fee, networkPassphrase: Networks.FUTURENET })
        .addOperation(
            // An operation to call increment on the contract
            dao_contract.call("init",
                xdr.ScVal.scvObject(xdr.ScObject.scoBytes(Buffer.from("ba30bd6b7d519e0abadf6dc0fe100eda72fe6be5137a99fc8f44a8cd51d92b92","hex"))),
                xdr.ScVal.scvU32(3600),
                xdr.ScVal.scvU32(10),
                xdr.ScVal.scvObject(xdr.ScObject.scoI128(new xdr.Int128Parts({lo:xdr.Uint64.fromString("1"),hi:xdr.Uint64.fromString("0")}))),
            )
        )
        .setTimeout(30)
        .build();

    console.log(transaction.toXDR())
    const simulate_result = await server.simulateTransaction(transaction)

    // console.log(simulate_result)

    transaction = addFootprint(transaction,Networks.FUTURENET,simulate_result.results[0].footprint)
    // sign the transaction
    transaction.sign(keypair1);

    try {
        const transactionResult = await server.sendTransaction(transaction);
        console.log(transactionResult);
    } catch (err) {
        console.error(err);
    }
}

function bigintToBuf(bn) {
    var hex = BigInt(bn).toString(16).replace(/^-/, '');
    if (hex.length % 2) { hex = '0' + hex; }

    var len = hex.length / 2;
    var u8 = new Uint8Array(len);

    var i = 0;
    var j = 0;
    while (i < len) {
        u8[i] = parseInt(hex.slice(j, j+2), 16);
        i += 1;
        j += 2;
    }

    if (bn < BigInt(0)) {
        // Set the top bit
        u8[0] |= 0x80;
    }

    return Buffer.from(u8);
}
function bigNumberToI128(value) {
    const b = BigInt(value);
    const buf = bigintToBuf(b);
    if (buf.length > 16) {
        throw new Error("BigNumber overflows i128");
    }

    if (value < 0) {
        // Clear the top bit
        buf[0] &= 0x7f;
    }

    // left-pad with zeros up to 16 bytes
    let padded = Buffer.alloc(16);
    buf.copy(padded, padded.length-buf.length);
    console.debug({value: value.toString(), padded});

    if (value < 0) {
        // Set the top bit
        padded[0] |= 0x80;
    }

    const hi = new xdr.Uint64(
        bigNumberFromBytes(false, ...padded.slice(4, 8)),
        bigNumberFromBytes(false, ...padded.slice(0, 4))
    );
    const lo = new xdr.Uint64(
        bigNumberFromBytes(false, ...padded.slice(12, 16)),
        bigNumberFromBytes(false, ...padded.slice(8, 12))
    );

    return new xdr.Int128Parts({lo, hi});
}
function bigNumberFromBytes( ...bytes) {
    let sign = 1;

    let b = BigInt(0);
    for (let byte of bytes) {
        b <<= BigInt(8);
        b |= BigInt(byte);
    }
    return Number(b)
}
async function get_min_prop_power(){
    let fee = '1000';
    let transaction = new TransactionBuilder(account, { fee, networkPassphrase: Networks.FUTURENET })
        .addOperation(
            // An operation to call increment on the contract
            dao_contract.call("min_prop_p")
        )
        .setTimeout(30)
        .build();

    const simulate_result = await server.simulateTransaction(transaction)
    const xdri128 = simulate_result.results[0].xdr
    console.log(xdri128)
    const num = xdr.Int128Parts.fromXDR(xdri128, "base64")
    const a = num.hi()
    const b = num.lo()
    console.log(bigNumberFromBytes(a.high,a.low,b.high,b.low))
    console.log();
}

// console.log((new xdr.Int128Parts({lo:xdr.Uint64.fromString("1"),hi:xdr.Uint64.fromString("0")})))
// console.log((new xdr.Int128Parts({low:xdr.Uint64.fromString("1"),high:xdr.Uint64.fromString("0")})).toXDR("base64"))
// console.log("hi")
await get_min_prop_power()
// const num = bigNumberToI128(1)
// const a = num.hi()
// const b = num.lo()
// console.log(bigNumberFromBytes(a.high,a.low,b.high,b.low))