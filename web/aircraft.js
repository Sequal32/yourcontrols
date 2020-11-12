var aircraftList = document.getElementById("aircraft-list")

aircraftList.addAircraft = function(aircraftName) {
    const newButton = document.createElement("button")
    newButton.type = "button"
    newButton.className = "list-group-item list-group-item-action aircraft-list-entry bg-dark text-white"
    newButton.innerHTML = aircraftName

    newButton.onclick = function() {
        invoke({"type":"load_aircraft", "name": aircraftName})

        aircraftList.selectActive(newButton)
    }

    aircraftList.appendChild(newButton)
}

aircraftList.selectActive = function(button) {
    if (aircraftList.activeSelection) {
        aircraftList.activeSelection.classList.remove("active")
    }

    button.classList.add("active")
    aircraftList.activeSelection = button
}

aircraftList.searchSelectActive = function(name) {
    for (i=0; i < aircraftList.children.length; i++) {
        const button = aircraftList.children[i]
        if (button.innerHTML == name) {
            aircraftList.selectActive(button)
        }
    }
}