(function() {

    let createWalletBtn,
        directRadio,
        errorEl,
        mnemonicEl,
        nameEl,
        passwordEl,
        randomRadio,
        walletListEl,
        walletNames,
        warningEl;

    async function getWalletList() {
        // get the list of wallet names from the rust backend
        walletNames = await invoke("get_wallet_list");
        // show the names in the ui
        if (walletNames.length == 0) {
            // TODO decide if a wallet should be created automatically, or
            // whether to show 'no wallets' message
            return;
        }
        walletNames.sort();
        walletListEl.textContent = "";
        let wallets = walletNames.map((name) => {
            let wallet = new Wallet(name);
            walletListEl.appendChild(wallet.el);
            return wallet;
        });
        // select the first wallet
        wallets[0].select();
    }

    async function createWallet() {
        clearWalletError();
        let name = nameEl.value;
        let password = passwordEl.value;
        let mnemonic = mnemonicEl.value;
        // validate inputs
        // Name can't be blank
        if (name == "") {
            showWalletError("Name must not be blank");
            return;
        }
        // Name should be a valid OS filename.
        // TODO This feels a bit of an arbitrary restriction, eg the filename
        // could be set by the backend to be safe and the name stored within
        // the wallet data structure. For now it's good enough.
        if (!isValidFilename(name)) {
            showWalletError("Name must be a valid filename");
            return;
        }
        // Wallet name can't already exist, it would overwrite the existing
        // wallet.
        if (walletNames.indexOf(name) > -1) {
            showWalletError("Wallet name already in use, must be unique");
            return;
        }
        // Password can be anything, even blank.
        // If mnemonic is set, it's validated on the backend. Keeps BIP39 logic
        // all in one place (ie rust, not javascript).
        // send to backend for creating
        if (directRadio.checked) {
            await invoke("restore_wallet", {
                name: name,
                decryptor: password,
                mnemonic: mnemonic,
            });
            // TODO show error or success
        }
        else {
            let mnemonic = await invoke("create_new_random_wallet", {
                name: name,
                decryptor: password,
            });
            // TODO show mnemonic so a backup can be created
        }
    }

    function checkNewWalletWarnings() {
        clearWalletWarning();
        let name = nameEl.value;
        let password = passwordEl.value;
        // Warn if name has leading or trailing spaces
        if (name != name.trim(/\s/g)) {
            showWalletWarning("Name has leading or trailing space, did you really mean to?");
        }
        if (password != password.trim(/\s/g)) {
            showWalletWarning("Password has leading or trailing space, did you really mean to?");
        }
    }

    function clearWalletWarning() {
        warningEl.textContent = "";
    }

    function showWalletWarning(msg) {
        warningEl.textContent = msg;
    }

    function clearWalletError() {
        errorEl.textContent = "";
    }

    function showWalletError(msg) {
        errorEl.textContent = msg;
    }

    // https://stackoverflow.com/a/11101624
    function isValidFilename(filename) {
        // forbidden characters \ / : * ? " < > |
        let rg1=/^[^\\/:\*\?"<>\|]+$/;
        // cannot start with dot (.)
        let rg2=/^\./;
        // forbidden file names
        let rg3=/^(nul|prn|con|lpt[0-9]|com[0-9])(\.|$)/i;
        return rg1.test(filename)&&!rg2.test(filename)&&!rg3.test(filename);
    }

    function setNewMnemonicType() {
        if (directRadio.checked) {
            mnemonicEl.classList.remove("hidden");
        }
        else {
            mnemonicEl.classList.add("hidden");
            mnemonicEl.value = "";
        }
    }

    window.addEventListener("DOMContentLoaded", () => {
        // DOM
        createWalletBtn = document.querySelector(".wallets .create");
        directRadio = document.querySelector(".wallets .direct");
        errorEl = document.querySelector(".wallets .error");
        mnemonicEl = document.querySelector(".wallets .mnemonic");
        nameEl = document.querySelector(".wallets .name");
        passwordEl = document.querySelector(".wallets .password");
        randomRadio = document.querySelector(".wallets .random");
        walletListEl = document.querySelector(".wallets .list");
        warningEl = document.querySelector(".wallets .warning");
        // Events
        createWalletBtn.addEventListener("click", createWallet);
        directRadio.addEventListener("change", setNewMnemonicType);
        randomRadio.addEventListener("change", setNewMnemonicType);
        nameEl.addEventListener("keyup", checkNewWalletWarnings);
        passwordEl.addEventListener("keyup", checkNewWalletWarnings);
        getWalletList();
    });

    // TODO
    // change wallet name
    // change wallet password
    // delete wallet

})();
