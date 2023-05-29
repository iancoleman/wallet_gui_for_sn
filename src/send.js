(function() {

    let changeEl = document.querySelector(".send .change");
    let changeTextarea = document.querySelector(".send .change textarea");

    // If the dbcs have leftover funds, send them to our own address
    function fillChangeDetails() {
        // TODO calculate amount as sum(in) - sum(out) - fee
        let amount = 100;
        // hide the change info if amount is 0
        if (amount == 0) {
            changeEl.classList.add("hidden");
            return;
        }
        // show an error if not enough funds
        if (amount < 0) {
            // TODO show error
        }
        changeEl.classList.remove("hidden");
        // TODO get change address from backend
        let address = "my change address hex";
        // create the change info recipient
        let changeRecipient = amount + "," + address;
        // show the change info
        changeTextarea.value = changeRecipient;
    }

})();
