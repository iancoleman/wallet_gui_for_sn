(function() {

    let nav = document.querySelectorAll(".nav")[0];
    let panes = document.querySelectorAll(".panes")[0];
    let tabChanged = new Event("tab-changed");

    function TabPane(selector) {

        let self = this;
        self.tab = nav.querySelectorAll(selector)[0];
        self.pane = panes.querySelectorAll(selector)[0];

        self.tab.addEventListener("click", function() {
            hideAllPanes();
            self.pane.classList.remove("hidden");
            self.tab.classList.add("active");
            window.dispatchEvent(tabChanged);
        });
    }

    let tabPanes = [
        new TabPane(".receive"),
        new TabPane(".send"),
        new TabPane(".backup"),
        new TabPane(".restore"),
        new TabPane(".network"),
    ];

    function hideAllPanes() {
        tabPanes.map((tp) => {
            tp.pane.classList.add("hidden");
            tp.tab.classList.remove("active");
        });
    }

})()
