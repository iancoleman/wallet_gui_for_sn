(function() {

    let existingEl;

    async function getWalletList() {
        // get the list of wallet names from the rust backend
        let walletNames = await invoke("get_wallet_list");
        // show the names in the ui
        if (walletNames.length == 0) {
            // TODO decide if a wallet should be created automatically, or
            // whether to show 'no wallets' message
            return;
        }
        walletNames.sort();
        existingEl.textContent = "";
        let wallets = walletNames.map((name) => {
            let wallet = new Wallet(name);
            existingEl.appendChild(wallet.el);
            return wallet;
        });
        // select the first wallet
        wallets[0].select();
    }

    window.addEventListener("DOMContentLoaded", () => {
        existingEl = document.querySelector(".wallets .existing");
        getWalletList();
    });

    // TODO
    // create new wallet
    // change wallet name
    // change wallet password
    // delete wallet

})();
