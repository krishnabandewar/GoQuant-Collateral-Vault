const http = require('http');

async function request(method, path, body = null) {
    return new Promise((resolve, reject) => {
        const options = {
            hostname: '127.0.0.1',
            port: 8080,
            path: path,
            method: method,
            headers: {
                'Content-Type': 'application/json',
            },
        };

        const req = http.request(options, (res) => {
            let data = '';
            res.on('data', (chunk) => data += chunk);
            res.on('end', () => {
                if (res.statusCode >= 200 && res.statusCode < 300) {
                    try {
                        resolve(JSON.parse(data || '{}'));
                    } catch (e) {
                        resolve(data);
                    }
                } else {
                    reject(new Error(`Status ${res.statusCode}: ${data}`));
                }
            });
        });

        req.on('error', (e) => reject(e));

        if (body) {
            req.write(JSON.stringify(body));
        }
        req.end();
    });
}

async function runDemo() {
    console.log("🚀 Starting GoQuant Demo Client...");

    try {
        // 1. Health Check
        console.log("\n1. Checking System Health...");
        const health = await request('GET', '/health');
        console.log("✅ Health Status:", health);

        // 2. Register Vault
        const vaultData = {
            owner: "DemoUser_" + Date.now(),
            pubkey: "Vault_" + Math.random().toString(36).substring(7)
        };
        console.log(`\n2. Registering new Vault for owner: ${vaultData.owner}...`);
        const created = await request('POST', '/vaults', vaultData);
        console.log("✅ Vault Created:", created);

        // 3. Deposit logic (Simulated)
        console.log(`\n3. Simulating Deposit of 1000...`);
        const deposit = await request('POST', '/vaults/deposit', {
            vault_pubkey: vaultData.pubkey,
            amount: 1000,
            signature: "mock_sig_deposit"
        });
        console.log("✅ Deposit Result:", deposit);

        // 4. Withdraw logic (Simulated)
        console.log(`\n4. Simulating Withdrawal of 200...`);
        const withdraw = await request('POST', '/vaults/withdraw', {
            vault_pubkey: vaultData.pubkey,
            amount: 200,
            signature: "mock_sig_withdraw"
        });
        console.log("✅ Withdraw Result:", withdraw);

        // 5. Transaction History
        console.log(`\n5. Fetching Transaction History...`);
        const history = await request('GET', `/vaults/${vaultData.pubkey}/transactions`);
        console.log("✅ Transaction History:", history);


        // 6. Get TVL
        console.log(`\n6. Checking Total Value Locked (TVL)...`);
        const tvl = await request('GET', '/tvl');
        console.log("✅ Current TVL:", tvl);

        // 7. Get Vault Details with Balance
        console.log(`\n7. Fetching updated details for vault...`);
        const updatedVault = await request('GET', `/vaults/${vaultData.pubkey}`);
        console.log("✅ Updated Vault Details:", updatedVault);

        console.log("\n🎉 Demo Completed Successfully!");

    } catch (error) {
        console.error("❌ Demo Failed:", error.message);
    }
}

runDemo();
