const WebSocket = require('ws');
const axios = require('axios');

const AUTO_API_URL = 'http://127.0.0.1:8080';
const WS_URL = 'ws://127.0.0.1:8080/ws';

const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));

async function runDemo() {
    console.log('--- GoQuant System Demo Starting ---');

    console.log(`1. Connecting to WebSocket at ${WS_URL}...`);
    const ws = new WebSocket(WS_URL);

    // Promise to handle WS connection open
    await new Promise((resolve, reject) => {
        ws.on('open', resolve);
        ws.on('error', reject);
    });
    console.log('✅ WebSocket Connected!');

    // Listener for messages
    ws.on('message', (data) => {
        const msg = JSON.parse(data);
        const type = msg.type;
        console.log(`\n🔔 [WEBSOCKET BROADCAST] Received Event: ${type.toUpperCase()}`);
        console.log(JSON.stringify(msg, null, 2));
    });

    const vaultPubkey = `vault-${Date.now()}`;
    const owner = "User_Video_Demo";

    // 2. Create Vault
    console.log('\n2. Creating Vault via REST API...');
    try {
        const createRes = await axios.post(`${AUTO_API_URL}/vaults`, {
            owner: owner,
            pubkey: vaultPubkey
        });
        console.log('✅ Vault Created:', createRes.data);
    } catch (e) {
        console.error('Failed to create vault:', e.message);
        process.exit(1);
    }

    await sleep(1000);

    // 3. Deposit Funds
    console.log('\n3. Depositing 5000 USDT via REST API...');
    try {
        const depoRes = await axios.post(`${AUTO_API_URL}/vaults/deposit`, {
            vault_pubkey: vaultPubkey,
            amount: 5000,
            signature: "sig_deposit_123"
        });
        console.log('✅ Deposit REST Response:', depoRes.data);
    } catch (e) {
        console.error('Deposit failed:', e.message);
    }

    // Wait for WS message to arrive
    console.log('   (Waiting for WebSocket notification...)');
    await sleep(2000);

    // 4. Withdraw Funds
    console.log('\n4. Withdrawing 2000 USDT via REST API...');
    try {
         const withdrawRes = await axios.post(`${AUTO_API_URL}/vaults/withdraw`, {
            vault_pubkey: vaultPubkey,
            amount: 2000,
            signature: "sig_draw_456"
         });
         console.log('✅ Withdrawal REST Response:', withdrawRes.data);
    } catch (e) {
        console.error('Withdrawal failed:', e.message);
    }

    console.log('   (Waiting for WebSocket notification...)');
    await sleep(2000);

    // 5. Check TVL
    console.log('\n5. Checking Total Value Locked (TVL)...');
    try {
        const tvlRes = await axios.get(`${AUTO_API_URL}/tvl`);
        console.log('✅ Current TVL:', tvlRes.data.total_value_locked);
    } catch (e) {
        console.error('TVL check failed:', e.message);
    }

    // 6. Get Transactions
    console.log('\n6. Fetching Transaction History...');
    try {
        const txRes = await axios.get(`${AUTO_API_URL}/vaults/${vaultPubkey}/transactions`);
        console.log(`✅ Found ${txRes.data.length} transactions.`);
        // console.table(txRes.data);
    } catch (e) {
        console.error('History check failed:', e.message);
    }

    console.log('\n--- Demo Complete ---');
    ws.close();
}

runDemo();
